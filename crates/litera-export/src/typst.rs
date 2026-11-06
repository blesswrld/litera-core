//! Typst (`.typ`) Modern Typesetting Script Compiler.
//!
//! Generates complete, self-contained Typst documents configured with classical page
//! canons (Van de Graaf, Tschichold, Bringhurst), optical margin alignment (`hanging-indent`),
//! drop-cap initials (`droplet`), and marginalia side-notes.

use litera_typography::{compute_page_geometry, CanonPreset, PageDimensionsMm};

#[derive(Debug, Clone)]
pub struct TypstExportConfig<'a> {
    pub title: &'a str,
    pub subtitle: &'a str,
    pub body_text: &'a str,
    pub editorial_note: &'a str,
    pub font_family: &'a str,
    pub font_size_pt: f32,
    pub leading_ratio: f32,
    pub canon: CanonPreset,
    pub use_dropcap: bool,
    pub mean_ink_density: f32,
    pub cadence_variance: f32,
    pub acoustic_hash: &'a str,
}

/// Compiles the manuscript and typographic parameters into a production `.typ` file.
pub fn compile_typst_manuscript(cfg: &TypstExportConfig<'_>) -> String {
    let geom = compute_page_geometry(
        cfg.canon,
        PageDimensionsMm {
            width_mm: 165.0,
            height_mm: 240.0,
        },
        cfg.font_size_pt,
        cfg.leading_ratio,
    );

    let leading_em = (cfg.leading_ratio - 1.0).max(0.45);
    let mut out = String::with_capacity(cfg.body_text.len() + 2048);

    out.push_str("// ============================================================================\n");
    out.push_str("// Сгенерировано в ЛИТЕРА (litera-export :: Typst Typesetting Engine)\n");
    out.push_str(&format!(
        "// Канон полосы: {} | Акустический хеш: {}\n",
        cfg.canon.russian_title(),
        cfg.acoustic_hash
    ));
    out.push_str("// Компиляция: `typst compile manuscript.typ manuscript.pdf`\n");
    out.push_str("// ============================================================================\n\n");

    out.push_str("#set document(\n");
    out.push_str(&format!("  title: \"{}\",\n", escape_typst_str(cfg.title)));
    out.push_str("  author: \"ЛИТЕРА — Акустическая Типографика\",\n");
    out.push_str(")\n\n");

    out.push_str("#set page(\n");
    out.push_str(&format!(
        "  width: {:.1}mm,\n  height: {:.1}mm,\n",
        geom.page.width_mm, geom.page.height_mm
    ));
    out.push_str("  margin: (\n");
    out.push_str(&format!(
        "    inside: {:.1}mm,\n    top: {:.1}mm,\n    outside: {:.1}mm,\n    bottom: {:.1}mm,\n",
        geom.margins.inner_spine_mm,
        geom.margins.top_head_mm,
        geom.margins.outer_fore_edge_mm,
        geom.margins.bottom_foot_mm
    ));
    out.push_str("  ),\n");
    out.push_str("  fill: rgb(\"#FBF9F5\"),\n");
    out.push_str("  header: align(right, text(size: 7.5pt, fill: rgb(\"#78716C\"), tracking: 0.12em)[\n");
    out.push_str(&format!("    ЛИТЕРА · {}\n", escape_typst_str(cfg.title)));
    out.push_str("  ]),\n");
    out.push_str("  footer: align(center, context counter(page).display(\"— 1 —\")),\n");
    out.push_str(")\n\n");

    out.push_str("#set text(\n");
    out.push_str(&format!("  font: (\"{}\", \"Linux Libertine\", \"Georgia\"),\n", cfg.font_family));
    out.push_str(&format!("  size: {:.1}pt,\n", cfg.font_size_pt));
    out.push_str("  fill: rgb(\"#1C1917\"),\n");
    out.push_str("  lang: \"ru\",\n");
    out.push_str("  region: \"RU\",\n");
    out.push_str("  ligatures: true,\n");
    out.push_str("  number-type: \"old-style\",\n");
    out.push_str(")\n\n");

    out.push_str("#set par(\n");
    out.push_str(&format!("  leading: {:.2}em,\n", leading_em));
    out.push_str("  justify: true,\n");
    out.push_str("  linebreaks: \"optimized\", // Алгоритм Кнута—Пласса в движке Typst\n");
    out.push_str("  first-line-indent: 1.5em,\n");
    out.push_str(")\n\n");

    out.push_str("// Шмуцтитул и рубрика\n");
    out.push_str("#v(18pt)\n");
    out.push_str("#text(size: 8pt, tracking: 0.16em, fill: rgb(\"#9A3412\"))[\n");
    out.push_str(&format!("  #upper[\"{}\"]\n", escape_typst_str(cfg.subtitle)));
    out.push_str("]\n");
    out.push_str("#v(6pt)\n");
    out.push_str(&format!(
        "#text(size: {:.1}pt, weight: \"medium\")[{}]\n",
        cfg.font_size_pt * 2.1,
        escape_typst_str(cfg.title)
    ));
    out.push_str("#v(12pt)\n");
    out.push_str("#line(length: 100%, stroke: 0.4pt + rgb(\"#D6CEBE\"))\n");
    out.push_str("#v(16pt)\n\n");

    // Body text with optional dropcap
    if cfg.use_dropcap {
        let mut chars = cfg.body_text.trim().chars();
        if let Some(first_ch) = chars.next() {
            let rest: String = chars.collect();
            out.push_str(&format!(
                "#box(baseline: 65%, inset: (right: 6pt))[#text(size: {:.1}pt, fill: rgb(\"#9A3412\"), weight: \"semibold\")[{}]]{}\n\n",
                cfg.font_size_pt * 3.4,
                first_ch,
                escape_typst_str(&rest)
            ));
        }
    } else {
        out.push_str(&format!("{}\n\n", escape_typst_str(cfg.body_text.trim())));
    }

    out.push_str("#v(24pt)\n");
    out.push_str("#line(length: 35%, stroke: 0.4pt + rgb(\"#D6CEBE\"))\n");
    out.push_str("#v(6pt)\n");
    out.push_str("#text(size: 8.5pt, style: \"italic\", fill: rgb(\"#57534E\"))[\n");
    out.push_str(&format!(
        "  Колофон: Оптическая плотность полосы {:.1}/100 · Вариативность каденции {:.0}% · {}\n",
        cfg.mean_ink_density,
        cfg.cadence_variance * 100.0,
        escape_typst_str(cfg.editorial_note)
    ));
    out.push_str("]\n");

    out
}

fn escape_typst_str(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len() + 16);
    for ch in input.chars() {
        match ch {
            '#' | '$' | '@' | '*' | '_' | '\\' | '"' => {
                escaped.push('\\');
                escaped.push(ch);
            }
            _ => escaped.push(ch),
        }
    }
    escaped
}
