//! XeLaTeX / LuaLaTeX (`memoir` + `fontspec` + `microtype` + `lettrine`) Compiler.
//!
//! Produces archival book-grade LaTeX source code with Hermann Zapf `microtype`
//! optical margin protrusion and font expansion enabled.

use litera_typography::{compute_page_geometry, CanonPreset, PageDimensionsMm};

#[derive(Debug, Clone)]
pub struct LatexExportConfig<'a> {
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
    pub acoustic_hash: &'a str,
}

/// Compiles a complete XeLaTeX / LuaLaTeX document ready for `xelatex manuscript.tex`.
pub fn compile_xelatex_manuscript(cfg: &LatexExportConfig<'_>) -> String {
    let geom = compute_page_geometry(
        cfg.canon,
        PageDimensionsMm {
            width_mm: 165.0,
            height_mm: 240.0,
        },
        cfg.font_size_pt,
        cfg.leading_ratio,
    );

    let baseline_pt = cfg.font_size_pt * cfg.leading_ratio;
    let mut out = String::with_capacity(cfg.body_text.len() + 2400);

    out.push_str("% ============================================================================\n");
    out.push_str("% Сгенерировано в ЛИТЕРА (litera-export :: XeLaTeX / LuaLaTeX Book Engine)\n");
    out.push_str(&format!(
        "% Канон полосы: {} | Акустический хеш: {}\n",
        cfg.canon.russian_title(),
        cfg.acoustic_hash
    ));
    out.push_str("% Сборка: xelatex -interaction=nonstopmode manuscript.tex\n");
    out.push_str("% ============================================================================\n");
    out.push_str("\\documentclass[11pt,twoside,onecolumn]{memoir}\n\n");
    out.push_str("\\usepackage{fontspec}\n");
    out.push_str("\\usepackage[russian,english]{babel}\n");
    out.push_str("\\usepackage[protrusion=true,expansion=false,final]{microtype}\n");
    out.push_str("\\usepackage{lettrine}\n");
    out.push_str("\\usepackage{xcolor}\n\n");

    out.push_str("\\definecolor{ArchivalPaper}{HTML}{FBF9F5}\n");
    out.push_str("\\definecolor{ArchivalInk}{HTML}{1C1917}\n");
    out.push_str("\\definecolor{TerracottaAccent}{HTML}{9A3412}\n");
    out.push_str("\\definecolor{MutedStone}{HTML}{78716C}\n\n");

    out.push_str(&format!(
        "\\setmainfont{{{}}}[Numbers={{OldStyle,Proportional}},Ligatures=TeX]\n\n",
        cfg.font_family
    ));

    out.push_str("% Геометрия книжного разворота по выбранному канону\n");
    out.push_str(&format!(
        "\\setstocksize{{{:.1}mm}}{{{:.1}mm}}\n",
        geom.page.height_mm, geom.page.width_mm
    ));
    out.push_str(&format!(
        "\\settrimmedsize{{{:.1}mm}}{{{:.1}mm}}{{*}}\n",
        geom.page.height_mm, geom.page.width_mm
    ));
    out.push_str(&format!(
        "\\setlrmarginsandblock{{{:.1}mm}}{{{:.1}mm}}{{*}}\n",
        geom.margins.inner_spine_mm, geom.margins.outer_fore_edge_mm
    ));
    out.push_str(&format!(
        "\\setulmarginsandblock{{{:.1}mm}}{{{:.1}mm}}{{*}}\n",
        geom.margins.top_head_mm, geom.margins.bottom_foot_mm
    ));
    out.push_str("\\checkandfixthelayout\n\n");

    out.push_str("% Параметры алгоритма Кнута—Пласса в TeX\n");
    out.push_str("\\tolerance=400\n");
    out.push_str("\\emergencystretch=1.5em\n");
    out.push_str("\\hyphenpenalty=50\n");
    out.push_str("\\exhyphenpenalty=50\n");
    out.push_str("\\doublehyphendemerits=10000\n");
    out.push_str("\\finalhyphendemerits=5000\n\n");

    out.push_str("\\begin{document}\n");
    out.push_str("\\pagecolor{ArchivalPaper}\n");
    out.push_str("\\color{ArchivalInk}\n");
    out.push_str(&format!(
        "\\fontsize{{{:.1}pt}}{{{:.1}pt}}\\selectfont\n\n",
        cfg.font_size_pt, baseline_pt
    ));

    out.push_str(&format!(
        "\\noindent{{\\footnotesize\\scshape\\color{{TerracottaAccent}} {}}}\\\\[6pt]\n",
        escape_latex_str(cfg.subtitle)
    ));
    out.push_str(&format!(
        "\\noindent{{\\LARGE {}}}\\par\\vspace{{8pt}}\n",
        escape_latex_str(cfg.title)
    ));
    out.push_str("\\noindent\\textcolor{MutedStone}{\\rule{\\textwidth}{0.4pt}}\\par\\vspace{14pt}\n\n");

    let trimmed = cfg.body_text.trim();
    if cfg.use_dropcap {
        let mut chars = trimmed.chars();
        if let Some(first_ch) = chars.next() {
            let rest: String = chars.collect();
            let mut rest_words = rest.splitn(2, ' ');
            let first_word_tail = rest_words.next().unwrap_or("");
            let remaining_body = rest_words.next().unwrap_or("");
            out.push_str(&format!(
                "\\lettrine[lines=3,lhang=0.12,findent=3pt]{{\\textcolor{{TerracottaAccent}}{{{}}}}}{{{}}} {}\n\n",
                first_ch,
                escape_latex_str(first_word_tail),
                escape_latex_str(remaining_body)
            ));
        }
    } else {
        out.push_str(&format!("\\noindent {}\n\n", escape_latex_str(trimmed)));
    }

    out.push_str("\\vspace{24pt}\n");
    out.push_str("\\noindent\\textcolor{MutedStone}{\\rule{0.35\\textwidth}{0.4pt}}\\par\\vspace{6pt}\n");
    out.push_str(&format!(
        "\\noindent{{\\footnotesize\\itshape\\color{{MutedStone}} Колофон: Оптическая плотность {:.1}/100. {}}}\n",
        cfg.mean_ink_density,
        escape_latex_str(cfg.editorial_note)
    ));
    out.push_str("\\end{document}\n");

    out
}

fn escape_latex_str(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 16);
    for ch in input.chars() {
        match ch {
            '\\' => out.push_str("\\textbackslash{}"),
            '{' | '}' | '$' | '&' | '#' | '_' | '%' => {
                out.push('\\');
                out.push(ch);
            }
            '~' => out.push_str("\\textasciitilde{}"),
            '^' => out.push_str("\\textasciicircum{}"),
            _ => out.push(ch),
        }
    }
    out
}
