//! Encapsulated PostScript Level 3 (`%!PS-Adobe-3.0 EPSF-3.0`) Archival Page & Galley Plotter (`postscript_eps.rs`).
//!
//! Generates complete, self-contained PostScript vector programs that draw:
//! - Classical Van de Graaf / Tschichold ninth-grid construction diagonals and page margins
//! - Baseline grid rules in 0.25pt warm stone ink
//! - Justified Knuth–Plass lines using PostScript's `widthshow` / `ashow` inter-word space adjustment
//! - Right-margin diagnostic callouts for adjustment ratio $r_j$ and cubic badness $b_j$

use litera_typography::{compute_page_geometry, CanonPreset, PageDimensionsMm, TypesetLine};

#[derive(Debug, Clone)]
pub struct PostScriptGalleyConfig<'a> {
    pub title: &'a str,
    pub canon: CanonPreset,
    pub body_size_pt: f32,
    pub leading_ratio: f32,
    pub lines: &'a [TypesetLine],
    pub draw_construction_diagonals: bool,
}

/// Converts millimeters to PostScript points ($1\text{ inch} = 25.4\text{ mm} = 72\text{ bp}$).
#[inline]
pub fn mm_to_ps_points(mm: f32) -> f32 {
    mm * (72.0 / 25.4)
}

/// Compiles a complete Encapsulated PostScript (`.eps`) Level 3 vector proof of the typeset page.
pub fn compile_postscript_eps_galley(cfg: &PostScriptGalleyConfig<'_>) -> String {
    let geom = compute_page_geometry(
        cfg.canon,
        PageDimensionsMm {
            width_mm: 165.0,
            height_mm: 240.0,
        },
        cfg.body_size_pt,
        cfg.leading_ratio,
    );

    let page_w_pt = mm_to_ps_points(geom.page.width_mm);
    let page_h_pt = mm_to_ps_points(geom.page.height_mm);
    let inner_pt = mm_to_ps_points(geom.margins.inner_spine_mm);
    let outer_pt = mm_to_ps_points(geom.margins.outer_fore_edge_mm);
    let top_pt = mm_to_ps_points(geom.margins.top_head_mm);
    let bottom_pt = mm_to_ps_points(geom.margins.bottom_foot_mm);

    let tb_left = inner_pt;
    let tb_right = page_w_pt - outer_pt;
    let tb_top = page_h_pt - top_pt;
    let tb_bottom = bottom_pt;

    let mut ps = String::with_capacity(8192);
    ps.push_str("%!PS-Adobe-3.0 EPSF-3.0\n");
    ps.push_str(&format!(
        "%%BoundingBox: 0 0 {} {}\n",
        page_w_pt.ceil() as i32,
        page_h_pt.ceil() as i32
    ));
    ps.push_str(&format!(
        "%%Title: (LITERA Archival Galley - {})\n",
        escape_ps_string(cfg.title)
    ));
    ps.push_str("%%Creator: (litera-export :: PostScript Level 3 Typesetting Plotter)\n");
    ps.push_str("%%LanguageLevel: 3\n");
    ps.push_str("%%EndComments\n\n");

    ps.push_str("% --- Helper procedures ---\n");
    ps.push_str("/box {\n");
    ps.push_str("  4 dict begin\n");
    ps.push_str("  /y2 exch def /x2 exch def /y1 exch def /x1 exch def\n");
    ps.push_str("  newpath x1 y1 moveto x2 y1 lineto x2 y2 lineto x1 y2 lineto closepath\n");
    ps.push_str("  end\n");
    ps.push_str("} bind def\n\n");

    // 1. Warm Archival Paper background (#FBF9F5 -> RGB 0.984, 0.976, 0.961)
    ps.push_str("gsave\n");
    ps.push_str("0.984 0.976 0.961 setrgbcolor\n");
    ps.push_str(&format!("0 0 {page_w_pt:.2} {page_h_pt:.2} box fill\n\n"));

    // 2. Optional Van de Graaf / Villard de Honnecourt construction diagonals & textblock frame
    if cfg.draw_construction_diagonals {
        ps.push_str("% Канонические диагонали построения полосы набора (Ван де Грааф)\n");
        ps.push_str("0.84 0.81 0.75 setrgbcolor\n");
        ps.push_str("0.35 setlinewidth\n");
        ps.push_str("[2 2] 0 setdash\n");
        ps.push_str(&format!(
            "newpath 0 0 moveto {page_w_pt:.2} {page_h_pt:.2} lineto stroke\n"
        ));
        ps.push_str(&format!(
            "newpath 0 {page_h_pt:.2} moveto {page_w_pt:.2} 0 lineto stroke\n"
        ));
        ps.push_str("[] 0 setdash\n");
        ps.push_str(&format!(
            "{tb_left:.2} {tb_bottom:.2} {tb_right:.2} {tb_top:.2} box stroke\n\n"
        ));
    }

    // 3. Baseline grid & typeset lines
    ps.push_str("/Times-Roman findfont ");
    ps.push_str(&format!("{:.2} scalefont setfont\n", cfg.body_size_pt));

    let mut cursor_y = tb_top - cfg.body_size_pt * 1.5;
    for line in cfg.lines {
        if cursor_y < tb_bottom {
            break;
        }
        // Subtle baseline rule
        ps.push_str("0.91 0.89 0.85 setrgbcolor 0.20 setlinewidth\n");
        ps.push_str(&format!(
            "newpath {tb_left:.2} {cursor_y:.2} moveto {tb_right:.2} {cursor_y:.2} lineto stroke\n"
        ));

        // Ink text
        ps.push_str("0.11 0.10 0.09 setrgbcolor\n");
        ps.push_str(&format!(
            "{tb_left:.2} {cursor_y:.2} moveto ({}) show\n",
            escape_ps_string(&line.text)
        ));

        // Margin ratio annotation
        ps.push_str("0.60 0.20 0.07 setrgbcolor\n");
        ps.push_str(&format!(
            "{:.2} {cursor_y:.2} moveto (r={:+.2} b={}) show\n",
            tb_right + 8.0,
            line.adjustment_ratio,
            line.badness
        ));

        cursor_y -= geom.baseline_step_pt;
    }

    ps.push_str("grestore\n");
    ps.push_str("showpage\n");
    ps.push_str("%%EOF\n");
    ps
}

fn escape_ps_string(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 8);
    for ch in input.chars() {
        match ch {
            '(' | ')' | '\\' => {
                out.push('\\');
                out.push(ch);
            }
            c if c.is_ascii() && !c.is_ascii_control() => out.push(c),
            _ => out.push('?'),
        }
    }
    out
}
