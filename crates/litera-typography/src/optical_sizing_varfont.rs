//! OpenType 1.8 Variable Font (`fvar` / `avar` / `STAT`) Optical Sizing & Master Interpolation Engine (`optical_sizing_varfont.rs`).
//!
//! In 16th-century punchcutting (Claude Garamont, Robert Granjon, Miklós Kis), every point size
//! had a distinct physical steel punch: a `6 pt` *Nonpareille* punch had thicker hairlines,
//! wider internal counters, taller x-height, and looser letterfit than a `36 pt` *Canon* punch.
//!
//! This module implements OpenType 1.8 multi-axis piecewise-linear master interpolation across
//! four variation axes:
//! - `opsz` (Optical Size, $6.0\text{ pt} \dots 72.0\text{ pt}$)
//! - `wght` (Stroke Weight, $300 \dots 700$)
//! - `CONT` (Hairline-to-Stem Contrast, $0 \dots 100$)
//! - `ital` (Italic Cursive Angle & Aperture, $0.0 \dots 1.0$)

/// Registered OpenType 1.8 variation axis descriptor (`fvar` AxisValueRecord).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VariationAxisDescriptor {
    pub tag: [u8; 4],
    pub name_ru: &'static str,
    pub min_value: f32,
    pub default_value: f32,
    pub max_value: f32,
}

pub const CORMORANT_VARIATION_AXES: [VariationAxisDescriptor; 4] = [
    VariationAxisDescriptor {
        tag: *b"opsz",
        name_ru: "Оптический кегль (Optical Size)",
        min_value: 6.0,
        default_value: 12.0,
        max_value: 72.0,
    },
    VariationAxisDescriptor {
        tag: *b"wght",
        name_ru: "Насыщенность штриха (Weight)",
        min_value: 300.0,
        default_value: 400.0,
        max_value: 700.0,
    },
    VariationAxisDescriptor {
        tag: *b"CONT",
        name_ru: "Контрастность засечек (Contrast)",
        min_value: 0.0,
        default_value: 62.0,
        max_value: 100.0,
    },
    VariationAxisDescriptor {
        tag: *b"ital",
        name_ru: "Курсивная пластика (Italic)",
        min_value: 0.0,
        default_value: 0.0,
        max_value: 1.0,
    },
];

/// Historical 16th–18th century metal type body size classification (*Кегельная шкала Дидо / Гарамона*).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoricalBodySizeGrade {
    /// 6–7 pt: *Нонпарель / Миньон* (Marginalia, footnotes, apparatus criticus).
    NonpareilleCaption,
    /// 8–9 pt: *Петит / Боргес* (Commentary, parallel glosses, bibliography).
    PetitGloss,
    /// 10–12 pt: *Корпус / Цицеро* (Primary book body text).
    CiceroText,
    /// 14–18 pt: *Миттель / Терция* (Subheadings, epigraphs, opening lead).
    TertiaSubhead,
    /// 24–72 pt: *Канон / Империал* (Title pages, drop-cap initials, frontispiece).
    CanonDisplay,
}

impl HistoricalBodySizeGrade {
    pub const fn russian_title(self) -> &'static str {
        match self {
            HistoricalBodySizeGrade::NonpareilleCaption => "Нонпарель / Петит (6–7.5 pt · Маргиналии)",
            HistoricalBodySizeGrade::PetitGloss => "Боргес (8–9.5 pt · Глоссы и сноски)",
            HistoricalBodySizeGrade::CiceroText => "Корпус / Цицеро (10–13 pt · Основной набор)",
            HistoricalBodySizeGrade::TertiaSubhead => "Терция / Парагон (14–22 pt · Рубрики)",
            HistoricalBodySizeGrade::CanonDisplay => "Двойной Канон (24–72 pt · Титульный лист)",
        }
    }

    pub fn from_point_size(pt: f32) -> Self {
        if pt < 7.8 {
            HistoricalBodySizeGrade::NonpareilleCaption
        } else if pt < 9.8 {
            HistoricalBodySizeGrade::PetitGloss
        } else if pt < 13.8 {
            HistoricalBodySizeGrade::CiceroText
        } else if pt < 23.0 {
            HistoricalBodySizeGrade::TertiaSubhead
        } else {
            HistoricalBodySizeGrade::CanonDisplay
        }
    }
}

/// Interpolated optical master instance computed for a specific target point size and weight.
#[derive(Debug, Clone, PartialEq)]
pub struct OpticalMasterInstance {
    pub target_point_size_pt: f32,
    pub historical_grade: HistoricalBodySizeGrade,
    /// Normalized OpenType `fvar`/`avar` coordinate in $[-1.0, 1.0]$ along the `opsz` axis.
    pub normalized_opsz_coord: f32,
    /// Vertical stem width in UPEM units ($1000\text{ UPEM}$).
    pub main_stem_units: f32,
    /// Horizontal hairline / serif thickness in UPEM units.
    pub hairline_units: f32,
    /// Resulting optical stroke contrast ratio ($\text{Stem} : \text{Hairline}$).
    pub effective_contrast_ratio: f32,
    /// Normalized x-height in UPEM units (taller at small sizes for legibility, compact at display sizes).
    pub x_height_units: f32,
    /// Recommended tracking adjustment in $\text{em}/1000$ (positive for small sizes, negative for display).
    pub optical_tracking_milli_em: f32,
    /// Recommended CSS `font-variation-settings` declaration.
    pub css_variation_settings: String,
}

/// Normalizes a user-space axis value into OpenType's $[-1.0, 0.0, +1.0]$ coordinate space,
/// then applies a non-linear `avar` (Axis Variations Table)hyperbolic transfer curve so
/// optical compensation is steepest between `6 pt` and `14 pt`.
pub fn normalize_opsz_with_avar(point_size_pt: f32) -> f32 {
    let axis = &CORMORANT_VARIATION_AXES[0]; // opsz
    let clamped = point_size_pt.clamp(axis.min_value, axis.max_value);
    let linear_norm = if clamped < axis.default_value {
        (clamped - axis.default_value) / (axis.default_value - axis.min_value)
    } else if clamped > axis.default_value {
        (clamped - axis.default_value) / (axis.max_value - axis.default_value)
    } else {
        0.0
    };

    // Apply `avar` piecewise-logarithmic remapping for positive (display) coordinates
    if linear_norm > 0.0 {
        linear_norm.powf(0.62)
    } else {
        linear_norm
    }
}

/// Interpolates the exact glyph outline master parameters for a given point size and font weight.
pub fn interpolate_optical_master(point_size_pt: f32, weight: f32, is_italic: bool) -> OpticalMasterInstance {
    let opsz_coord = normalize_opsz_with_avar(point_size_pt);
    let wght_clamped = weight.clamp(300.0, 700.0);
    let wght_factor = (wght_clamped - 400.0) / 300.0; // [-0.33, +1.0]

    // At default 12pt (opsz_coord = 0.0): main stem = 86 units, hairline = 16 units
    // At 6pt (opsz_coord = -1.0): main stem = 96 units, hairline = 28 units (sturdy caption)
    // At 72pt (opsz_coord = +1.0): main stem = 82 units, hairline = 9.5 units (razor display)
    let base_stem = if opsz_coord < 0.0 {
        86.0 + (-opsz_coord) * 10.0
    } else {
        86.0 - opsz_coord * 4.0
    };
    let main_stem_units = (base_stem + wght_factor * 42.0).clamp(58.0, 155.0);

    let base_hairline = if opsz_coord < 0.0 {
        16.0 + (-opsz_coord) * 12.0
    } else {
        16.0 - opsz_coord * 6.5
    };
    let hairline_units = (base_hairline + wght_factor * 4.5).clamp(8.0, 38.0);

    // x-height expands from 425 UPEM at 12pt up to 468 UPEM at 6pt, and compresses to 398 UPEM at 72pt
    let x_height_units = if opsz_coord < 0.0 {
        425.0 + (-opsz_coord) * 43.0
    } else {
        425.0 - opsz_coord * 27.0
    };

    // Optical tracking: +24 milli-em at 6pt, 0 at 12pt, -18 milli-em at 72pt
    let tracking_milli_em = if opsz_coord < 0.0 {
        (-opsz_coord) * 24.0
    } else {
        -opsz_coord * 18.0
    };

    let contrast_ratio = main_stem_units / hairline_units.max(1.0);
    let cont_axis_val = ((contrast_ratio - 3.0) / 6.5 * 100.0).clamp(10.0, 100.0);
    let ital_val = if is_italic { 1 } else { 0 };

    let css = format!(
        "'opsz' {:.1}, 'wght' {:.0}, 'CONT' {:.0}, 'ital' {}",
        point_size_pt.clamp(6.0, 72.0),
        wght_clamped,
        cont_axis_val,
        ital_val
    );

    OpticalMasterInstance {
        target_point_size_pt: point_size_pt,
        historical_grade: HistoricalBodySizeGrade::from_point_size(point_size_pt),
        normalized_opsz_coord: opsz_coord,
        main_stem_units,
        hairline_units,
        effective_contrast_ratio: contrast_ratio,
        x_height_units,
        optical_tracking_milli_em: tracking_milli_em,
        css_variation_settings: css,
    }
}

/// Generates a full 5-grade specimen table from 6 pt Nonpareille to 48 pt Canon Display.
pub fn generate_optical_scale_specimen_table(weight: f32) -> Vec<OpticalMasterInstance> {
    let sizes = [6.5f32, 9.0, 11.5, 18.0, 48.0];
    sizes
        .iter()
        .map(|&pt| interpolate_optical_master(pt, weight, false))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_optical_size_thickens_hairlines_and_expands_xheight() {
        let caption_6pt = interpolate_optical_master(6.0, 400.0, false);
        let body_12pt = interpolate_optical_master(12.0, 400.0, false);
        let display_60pt = interpolate_optical_master(60.0, 400.0, false);

        // 6pt caption must have thicker hairlines and taller x-height than 12pt body
        assert!(caption_6pt.hairline_units > body_12pt.hairline_units);
        assert!(caption_6pt.x_height_units > body_12pt.x_height_units);
        // 60pt display must have higher stroke contrast than 12pt body
        assert!(display_60pt.effective_contrast_ratio > body_12pt.effective_contrast_ratio);
    }
}
