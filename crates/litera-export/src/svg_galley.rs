//! Vector SVG Galley Proof Renderer (`svg_galley.rs`).
//!
//! Renders a high-precision vector galley diagram showing each Knuth–Plass broken line,
//! its horizontal justification adjustment ratio $r_j$, and its cubic badness indicator.

use litera_typography::TypesetLine;

#[derive(Debug, Clone)]
pub struct SvgGalleyConfig<'a> {
    pub title: &'a str,
    pub lines: &'a [TypesetLine],
    pub dark_mode: bool,
}

/// Generates a standalone, archival-quality SVG galley proof document.
pub fn render_svg_galley_proof(cfg: &SvgGalleyConfig<'_>) -> String {
    let width = 840;
    let row_h = 38;
    let header_h = 84;
    let footer_h = 48;
    let height = header_h + (cfg.lines.len().max(1) as i32) * row_h + footer_h;

    let (bg, fg, muted, border, accent) = if cfg.dark_mode {
        ("#0A0A0A", "#F5F2EB", "#A8A29E", "#262422", "#D97706")
    } else {
        ("#FBF9F5", "#1C1917", "#78716C", "#E6E1D6", "#9A3412")
    };

    let mut svg = String::with_capacity(4096);
    svg.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width} {height}\" width=\"{width}\" height=\"{height}\">\n"
    ));
    svg.push_str(&format!(
        "  <rect width=\"100%\" height=\"100%\" fill=\"{bg}\" />\n"
    ));
    svg.push_str(&format!(
        "  <text x=\"36\" y=\"38\" fill=\"{muted}\" font-family=\"Plus Jakarta Sans, sans-serif\" font-size=\"10\" letter-spacing=\"2\">ЛИТЕРА · ГРАНОЧНЫЙ ОТТИСК КНУТА—ПЛАССА</text>\n"
    ));
    svg.push_str(&format!(
        "  <text x=\"36\" y=\"62\" fill=\"{fg}\" font-family=\"Cormorant Garamond, Georgia, serif\" font-size=\"22\">{}</text>\n",
        escape_xml(cfg.title)
    ));
    svg.push_str(&format!(
        "  <line x1=\"36\" y1=\"74\" x2=\"{}\" y2=\"74\" stroke=\"{border}\" stroke-width=\"1\" />\n",
        width - 36
    ));

    for (idx, line) in cfg.lines.iter().enumerate() {
        let y = header_h + (idx as i32) * row_h + 24;
        svg.push_str(&format!(
            "  <text x=\"36\" y=\"{y}\" fill=\"{muted}\" font-family=\"JetBrains Mono, monospace\" font-size=\"11\">{:02}</text>\n",
            line.line_number
        ));
        svg.push_str(&format!(
            "  <text x=\"72\" y=\"{y}\" fill=\"{fg}\" font-family=\"Cormorant Garamond, Georgia, serif\" font-size=\"18\">{}</text>\n",
            escape_xml(&line.text)
        ));
        svg.push_str(&format!(
            "  <text x=\"{}\" y=\"{y}\" text-anchor=\"end\" fill=\"{accent}\" font-family=\"JetBrains Mono, monospace\" font-size=\"11\">r={:+.2} · b={}</text>\n",
            width - 36,
            line.adjustment_ratio,
            line.badness
        ));
    }

    svg.push_str("</svg>\n");
    svg
}

fn escape_xml(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 8);
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(ch),
        }
    }
    out
}
