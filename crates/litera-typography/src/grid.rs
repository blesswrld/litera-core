//! Classical Page Geometry Canons (Van de Graaf, Villard de Honnecourt, Tschichold Golden Canon,
//! and Bringhurst Modular Scales).
//!
//! Calculates exact book-spread margins, textblock dimensions, baseline grid increments,
//! and musical typographic scales ($r = 2^{1/n}$ or golden ratio $\varphi \approx 1.618034$).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanonPreset {
    /// Van de Graaf / Villard de Honnecourt Medieval Canon (2 : 3 : 4 : 6 margins, 1/9th grid).
    VanDeGraafMedieval,
    /// Jan Tschichold Golden Section Book Canon (Page 2:3, Textblock proportioned by $\varphi$).
    TschicholdGolden,
    /// Robert Bringhurst Modern Octave Editorial Grid (generous fore-edge & foot for marginalia).
    BringhurstEditorial,
    /// Swiss Neo-Brutalist / Josef Müller-Brockmann Modular Grid (tight inner, structured columns).
    SwissModular,
}

impl CanonPreset {
    pub const fn russian_title(self) -> &'static str {
        match self {
            CanonPreset::VanDeGraafMedieval => "Канон Ван де Граафа (2 : 3 : 4 : 6)",
            CanonPreset::TschicholdGolden => "Золотой канон Яна Чихольда (1 : 1.618)",
            CanonPreset::BringhurstEditorial => "Академическая полоса Брингхёрста (с маргиналиями)",
            CanonPreset::SwissModular => "Швейцарская модульная сетка Мюллера-Брокмана",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageDimensionsMm {
    pub width_mm: f32,
    pub height_mm: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MarginProportions {
    pub inner_spine_mm: f32,
    pub top_head_mm: f32,
    pub outer_fore_edge_mm: f32,
    pub bottom_foot_mm: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModularScaleStep {
    pub step_name: &'static str,
    pub point_size: f32,
    pub leading_pt: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageGeometryReport {
    pub canon: CanonPreset,
    pub page: PageDimensionsMm,
    pub margins: MarginProportions,
    pub textblock_width_mm: f32,
    pub textblock_height_mm: f32,
    pub textblock_area_ratio: f32,
    pub baseline_step_pt: f32,
    pub lines_per_page: usize,
    pub modular_scale: Vec<ModularScaleStep>,
}

/// Computes exact page geometry and modular typographic scale for a book trim size.
pub fn compute_page_geometry(
    canon: CanonPreset,
    page: PageDimensionsMm,
    body_size_pt: f32,
    leading_ratio: f32,
) -> PageGeometryReport {
    let w = page.width_mm.max(100.0);
    let h = page.height_mm.max(140.0);

    let margins = match canon {
        CanonPreset::VanDeGraafMedieval => {
            // 9x9 ninths construction: inner = 1/9 W, outer = 2/9 W, top = 1/9 H, bottom = 2/9 H
            MarginProportions {
                inner_spine_mm: w / 9.0,
                top_head_mm: h / 9.0,
                outer_fore_edge_mm: (w * 2.0) / 9.0,
                bottom_foot_mm: (h * 2.0) / 9.0,
            }
        }
        CanonPreset::TschicholdGolden => {
            // Proportions 2 : 3 : 5 : 8 (Fibonacci approximation to golden section)
            let unit_w = w / 21.0;
            let unit_h = h / 21.0;
            MarginProportions {
                inner_spine_mm: unit_w * 2.0,
                top_head_mm: unit_h * 2.5,
                outer_fore_edge_mm: unit_w * 4.5,
                bottom_foot_mm: unit_h * 5.0,
            }
        }
        CanonPreset::BringhurstEditorial => {
            // Wide outer fore-edge for scholarly side-notes (marginalia)
            MarginProportions {
                inner_spine_mm: w * 0.10,
                top_head_mm: h * 0.10,
                outer_fore_edge_mm: w * 0.24,
                bottom_foot_mm: h * 0.16,
            }
        }
        CanonPreset::SwissModular => MarginProportions {
            inner_spine_mm: w * 0.08,
            top_head_mm: h * 0.09,
            outer_fore_edge_mm: w * 0.10,
            bottom_foot_mm: h * 0.12,
        },
    };

    let tb_w = (w - margins.inner_spine_mm - margins.outer_fore_edge_mm).max(40.0);
    let tb_h = (h - margins.top_head_mm - margins.bottom_foot_mm).max(60.0);
    let area_ratio = (tb_w * tb_h) / (w * h);

    let baseline_step_pt = (body_size_pt * leading_ratio * 10.0).round() / 10.0;
    // 1 pt = 0.352778 mm
    let baseline_mm = baseline_step_pt * 0.352_778;
    let lines_per_page = (tb_h / baseline_mm.max(1.0)).floor() as usize;

    // Musical diatonic/golden typographic scale anchored at body_size_pt
    let scale_factors: [(&'static str, f32); 6] = [
        ("Маргиналии и колонтитул (Caption)", 0.75),
        ("Основной корпус (Body Antiqua)", 1.00),
        ("Подзаголовок главы (Subhead)", 1.25),
        ("Рубрика / Титул раздела (Section)", 1.618),
        ("Шмуцтитул (Chapter Display)", 2.25),
        ("Книжная буквица (Drop Cap Initial)", 3.50),
    ];

    let modular_scale = scale_factors
        .iter()
        .map(|&(name, factor)| {
            let pt = (body_size_pt * factor * 10.0).round() / 10.0;
            ModularScaleStep {
                step_name: name,
                point_size: pt,
                leading_pt: if factor >= 2.0 {
                    pt * 1.12
                } else {
                    baseline_step_pt
                },
            }
        })
        .collect();

    PageGeometryReport {
        canon,
        page,
        margins,
        textblock_width_mm: tb_w,
        textblock_height_mm: tb_h,
        textblock_area_ratio: area_ratio,
        baseline_step_pt,
        lines_per_page,
        modular_scale,
    }
}
