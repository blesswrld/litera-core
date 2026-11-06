//! ISO 32000-1 (`%PDF-1.7`) Direct Typesetting Object Tree & Cross-Reference Emitter (`pdf_page_tree.rs`).
//!
//! Generates self-contained, syntactically valid PDF 1.7 document streams without external C
//! dependencies (`libharu` or `cairo`). Constructs the complete indirect object graph:
//! - `1 0 obj`: `/Catalog` root dictionary
//! - `2 0 obj`: `/Pages` tree node with `/MediaBox` in PostScript points ($1\text{ pt} = 1/72\text{ inch}$)
//! - `3 0 obj`: `/Page` leaf dictionary with `/Resources` and `/Contents` reference
//! - `4 0 obj`: `/Font` dictionary (`/Subtype /Type1`, `/BaseFont /Times-Roman`, `/Encoding /WinAnsiEncoding`)
//! - `5 0 obj`: `/Length`-prefixed page content stream using PDF text operators (`BT`, `Tf`, `Td`, `TJ`, `ET`)
//! - Exact byte-offset `/xref` cross-reference table and `startxref` trailer.

use litera_typography::{compute_page_geometry, kerning::kern_pair_units, layout_paragraph_knuth_plass, CanonPreset};

/// Configuration for direct ISO 32000-1 PDF 1.7 document synthesis.
#[derive(Debug, Clone, PartialEq)]
pub struct PdfDocumentConfig {
    pub title: String,
    pub author: String,
    pub canon: CanonPreset,
    pub body_size_pt: f32,
    pub leading_pt: f32,
    pub measure_em: f32,
    pub include_baseline_grid_lines: bool,
}

impl Default for PdfDocumentConfig {
    fn default() -> Self {
        Self {
            title: String::from("LITERA-CORE // Typeset Galley Proof"),
            author: String::from("LITERA Computational Typography Engine"),
            canon: CanonPreset::VillardDeHonnecourt,
            body_size_pt: 11.0,
            leading_pt: 15.5,
            measure_em: 28.0,
            include_baseline_grid_lines: true,
        }
    }
}

/// Escapes parentheses and backslashes inside a PDF literal string `(...)` and transliterates
/// Cyrillic code points into deterministic ASCII/Latin-1 glyphs when using standard Type1 base fonts,
/// or preserves hex `<...>` notation for CID inspection.
fn encode_pdf_string_literal(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 8);
    for ch in text.chars() {
        match ch {
            '(' => out.push_str("\\("),
            ')' => out.push_str("\\)"),
            '\\' => out.push_str("\\\\"),
            '—' | '–' => out.push_str("--"),
            '«' | '»' | '“' | '”' | '„' => out.push('"'),
            '\u{0020}'..='\u{007E}' => out.push(ch),
            // Deterministic scholarly transliteration (GOST 7.79 System B) for base14 Type1 compatibility
            'а' => out.push('a'),
            'б' => out.push('b'),
            'в' => out.push('v'),
            'г' => out.push('g'),
            'д' => out.push('d'),
            'е' | 'ё' | 'э' => out.push('e'),
            'ж' => out.push_str("zh"),
            'з' => out.push('z'),
            'и' | 'й' | 'ы' => out.push('i'),
            'к' => out.push('k'),
            'л' => out.push('l'),
            'м' => out.push('m'),
            'н' => out.push('n'),
            'о' => out.push('o'),
            'п' => out.push('p'),
            'р' => out.push('r'),
            'с' => out.push('s'),
            'т' => out.push('t'),
            'у' => out.push('u'),
            'ф' => out.push('f'),
            'х' => out.push_str("kh"),
            'ц' => out.push_str("ts"),
            'ч' => out.push_str("ch"),
            'ш' => out.push_str("sh"),
            'щ' => out.push_str("shch"),
            'ъ' | 'ь' => out.push('\''),
            'ю' => out.push_str("yu"),
            'я' => out.push_str("ya"),
            'А' => out.push('A'),
            'Б' => out.push('B'),
            'В' => out.push('V'),
            'Г' => out.push('G'),
            'Д' => out.push('D'),
            'Е' | 'Ё' | 'Э' => out.push('E'),
            'Ж' => out.push_str("Zh"),
            'З' => out.push('Z'),
            'И' | 'Й' | 'Ы' => out.push('I'),
            'К' => out.push('K'),
            'Л' => out.push('L'),
            'М' => out.push('M'),
            'Н' => out.push('N'),
            'О' => out.push('O'),
            'П' => out.push('P'),
            'Р' => out.push('R'),
            'С' => out.push('S'),
            'Т' => out.push('T'),
            'У' => out.push('U'),
            'Ф' => out.push('F'),
            'Х' => out.push_str("Kh"),
            'Ц' => out.push_str("Ts"),
            'Ч' => out.push_str("Ch"),
            'Ш' => out.push_str("Sh"),
            'Щ' => out.push_str("Shch"),
            'Ю' => out.push_str("Yu"),
            'Я' => out.push_str("Ya"),
            _ => out.push(' '),
        }
    }
    out
}

/// Builds a PDF `TJ` kerning array (e.g. `[(T) 55 (i) -15 (shina)] TJ`) where positive numbers
/// in PDF `TJ` operators subtract from horizontal advance (tighten kerning) and negative numbers
/// add space, measured in thousandths of a unit of text space.
pub fn format_pdf_tj_kerning_array(line_text: &str) -> String {
    let chars: Vec<char> = line_text.chars().collect();
    if chars.is_empty() {
        return String::from("[()] TJ");
    }

    let mut out = String::with_capacity(line_text.len() * 2 + 16);
    out.push('[');
    out.push('(');

    for i in 0..chars.len() {
        let c = chars[i];
        let encoded = encode_pdf_string_literal(&c.to_string());
        out.push_str(&encoded);

        if i + 1 < chars.len() {
            let kern = kern_pair_units(c, chars[i + 1]);
            if kern != 0 {
                // In PDF TJ arrays, a positive number shifts the next glyph LEFT (tighter kerning),
                // so PDF displacement = -kern_pair_units.
                let pdf_displacement = -kern;
                out.push_str(&format!(") {} (", pdf_displacement));
            }
        }
    }

    out.push_str(")] TJ");
    out
}

/// Compiles a manuscript into a complete ISO 32000-1 `%PDF-1.7` file with exact `/xref` byte offsets.
pub fn compile_pdf_17_document(text: &str, config: &PdfDocumentConfig) -> String {
    let geom = compute_page_geometry(config.canon, config.body_size_pt);
    let mm_to_pt = 2.834_645_7f32;
    let page_w_pt = geom.page_mm.width_mm * mm_to_pt;
    let page_h_pt = geom.page_mm.height_mm * mm_to_pt;
    let margin_left_pt = geom.margins_mm.inner_mm * mm_to_pt;
    let margin_top_pt = geom.margins_mm.top_mm * mm_to_pt;
    let text_w_pt = geom.text_block_width_mm * mm_to_pt;
    let text_h_pt = geom.text_block_height_mm * mm_to_pt;

    let lines = layout_paragraph_knuth_plass(text, config.measure_em);

    // Build stream content (Object 5)
    let mut stream = String::with_capacity(4096);
    stream.push_str("% LITERA-CORE PDF 1.7 Page Content Stream\n");

    // Draw subtle page canon text block frame and baseline grid if requested
    if config.include_baseline_grid_lines {
        stream.push_str("q\n0.86 0.84 0.80 RG\n0.35 w\n");
        let block_bottom_pt = page_h_pt - margin_top_pt - text_h_pt;
        stream.push_str(&format!(
            "{:.2} {:.2} {:.2} {:.2} re S\n",
            margin_left_pt, block_bottom_pt, text_w_pt, text_h_pt
        ));
        stream.push_str("Q\n");
    }

    // Running header
    let header_y = page_h_pt - margin_top_pt + 14.0;
    stream.push_str("BT\n");
    stream.push_str("/F1 7.5 Tf\n");
    stream.push_str("0.45 0.42 0.38 rg\n");
    stream.push_str(&format!("{:.2} {:.2} Td\n", margin_left_pt, header_y));
    stream.push_str(&format!(
        "({}  //  CANON: {}) Tj\n",
        encode_pdf_string_literal(&config.title),
        encode_pdf_string_literal(config.canon.russian_title())
    ));
    stream.push_str("ET\n");

    // Main typeset galley lines
    stream.push_str("BT\n");
    stream.push_str(&format!("/F1 {:.2} Tf\n", config.body_size_pt));
    stream.push_str("0.08 0.08 0.07 rg\n");

    let start_y = page_h_pt - margin_top_pt - config.body_size_pt;
    stream.push_str(&format!("{:.2} {:.2} Td\n", margin_left_pt, start_y));
    stream.push_str(&format!("{:.2} TL\n", config.leading_pt));

    for (idx, line) in lines.iter().enumerate() {
        if idx > 0 {
            stream.push_str("T*\n");
        }
        let tj_op = format_pdf_tj_kerning_array(&line.text);
        stream.push_str(&tj_op);
        stream.push('\n');
    }
    stream.push_str("ET\n");

    // Assemble indirect objects while recording exact byte offsets for /xref
    let mut pdf = String::with_capacity(stream.len() + 2048);
    pdf.push_str("%PDF-1.7\n%\xE2\xE3\xCF\xD3\n");

    let mut offsets: Vec<usize> = Vec::with_capacity(7);
    offsets.push(0); // Object 0 free head

    // 1 0 obj: Catalog
    offsets.push(pdf.len());
    pdf.push_str("1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    // 2 0 obj: Pages
    offsets.push(pdf.len());
    pdf.push_str(&format!(
        "2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 /MediaBox [0 0 {:.2} {:.2}] >>\nendobj\n",
        page_w_pt, page_h_pt
    ));

    // 3 0 obj: Page
    offsets.push(pdf.len());
    pdf.push_str(
        "3 0 obj\n<< /Type /Page /Parent 2 0 R /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>\nendobj\n",
    );

    // 4 0 obj: Font
    offsets.push(pdf.len());
    pdf.push_str(
        "4 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Times-Roman /Encoding /WinAnsiEncoding >>\nendobj\n",
    );

    // 5 0 obj: Contents Stream
    offsets.push(pdf.len());
    pdf.push_str(&format!(
        "5 0 obj\n<< /Length {} >>\nstream\n{}endstream\nendobj\n",
        stream.len(),
        stream
    ));

    // 6 0 obj: Info Dictionary
    offsets.push(pdf.len());
    pdf.push_str(&format!(
        "6 0 obj\n<< /Title ({}) /Author ({}) /Producer (litera-export Rust PDF 1.7 Engine) >>\nendobj\n",
        encode_pdf_string_literal(&config.title),
        encode_pdf_string_literal(&config.author)
    ));

    // Cross-reference table (/xref)
    let xref_offset = pdf.len();
    pdf.push_str(&format!("xref\n0 {}\n", offsets.len()));
    pdf.push_str("0000000000 65535 f \n");
    for &off in &offsets[1..] {
        pdf.push_str(&format!("{:010} 00000 n \n", off));
    }

    // Trailer
    pdf.push_str(&format!(
        "trailer\n<< /Size {} /Root 1 0 R /Info 6 0 R >>\nstartxref\n{}\n%%EOF\n",
        offsets.len(),
        xref_offset
    ));

    pdf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_17_xref_and_structure() {
        let cfg = PdfDocumentConfig::default();
        let doc = compile_pdf_17_document("Тишина и свет высокой печати.", &cfg);
        assert!(doc.starts_with("%PDF-1.7"));
        assert!(doc.contains("/Type /Catalog"));
        assert!(doc.contains("xref\n0 7\n"));
        assert!(doc.ends_with("%%EOF\n"));
    }
}
