//! Glyph Advance Widths (in 1/1000 em units), Ascender/Descender Geometry,
//! and Optical Ink Density Profiling.

use crate::kerning::kern_pair_units;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontCategory {
    /// High-contrast Renaissance Antiqua (Cormorant Garamond).
    RenaissanceSerif,
    /// Contemporary Geometric Neo-Grotesque (Plus Jakarta Sans).
    GeometricSans,
    /// Fixed-width Engineering Monospace (JetBrains Mono, 600 units per glyph).
    Monospace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlyphMetric {
    /// Advance width in 1/1000 em units (1000 = 1 em quad).
    pub advance_width_units: u16,
    /// Optical ink coverage (0..=100) representing stroke mass relative to counter space.
    pub ink_density: u8,
    /// Number of vertical stems (`I` = 1, `П`/`Н` = 2, `Ш`/`Щ`/`М` = 3+).
    pub vertical_stems: u8,
    pub has_ascender: bool,
    pub has_descender: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParagraphInkProfile {
    pub total_em_width: f32,
    pub mean_ink_density: f32,
    pub stem_rhythm_per_em: f32,
    pub ascender_ratio: f32,
    pub descender_ratio: f32,
}

/// Computes precise glyph metrics for a character in the given `FontCategory`.
#[inline]
pub fn compute_glyph_metrics(ch: char, font: FontCategory) -> GlyphMetric {
    let lower = ch.to_lowercase().next().unwrap_or(ch);
    let is_upper = ch.is_uppercase();

    let (base_serif_w, ink, stems) = match lower {
        'ш' | 'щ' | 'ж' | 'ю' | 'ы' | 'м' | 'w' | 'm' => (820u16, 84u8, 3u8),
        'ф' | 'д' | 'б' | 'в' | 'п' | 'н' | 'и' | 'й' | 'я' | 'х' | 'ц' | 'h' | 'n' | 'u' | 'k' => {
            (610, 68, 2)
        }
        'а' | 'е' | 'ё' | 'о' | 'р' | 'т' | 'у' | 'к' | 'г' | 'з' | 'ч' | 'a' | 'e' | 'o' | 'b'
        | 'd' | 'p' | 'q' | 'g' | 'v' | 'x' | 'y' | 'z' => (510, 54, 1),
        'с' | 'л' | 'э' | 'ь' | 'ъ' | 'c' | 's' | 'r' | 'f' | 't' => (430, 42, 1),
        'i' | 'l' | 'j' | 'і' => (290, 36, 1),
        '—' => (920, 28, 0),
        '-' | '«' | '»' => (380, 26, 0),
        '.' | ',' | ':' | ';' | '!' | '?' => (260, 24, 0),
        ' ' => (250, 0, 0),
        _ if ch.is_alphabetic() => (520, 55, 1),
        _ => (300, 15, 0),
    };

    let width_units = match font {
        FontCategory::Monospace => 600,
        FontCategory::RenaissanceSerif => {
            if is_upper {
                (base_serif_w as f32 * 1.24) as u16
            } else {
                base_serif_w
            }
        }
        FontCategory::GeometricSans => {
            let sans_base = (base_serif_w as f32 * 1.07) as u16;
            if is_upper {
                (sans_base as f32 * 1.18) as u16
            } else {
                sans_base
            }
        }
    };

    let has_ascender = is_upper
        || matches!(
            lower,
            'б' | 'в' | 'ё' | 'й' | 'b' | 'd' | 'f' | 'h' | 'k' | 'l' | 't'
        );
    let has_descender = matches!(
        lower,
        'д' | 'р' | 'у' | 'ф' | 'ц' | 'щ' | 'g' | 'j' | 'p' | 'q' | 'y'
    );

    GlyphMetric {
        advance_width_units: width_units,
        ink_density: if font == FontCategory::GeometricSans && ink > 0 {
            (ink + 8).min(95)
        } else {
            ink
        },
        vertical_stems: stems,
        has_ascender,
        has_descender,
    }
}

/// Measures the exact horizontal advance width of a string in 1/1000 em units,
/// incorporating optical pair kerning for proportional fonts.
pub fn measure_string_em_units(text: &str, font: FontCategory) -> i32 {
    let mut total = 0i32;
    let mut prev: Option<char> = None;

    for ch in text.chars() {
        let gm = compute_glyph_metrics(ch, font);
        total += i32::from(gm.advance_width_units);
        if font != FontCategory::Monospace {
            if let Some(p) = prev {
                let (kern, _) = kern_pair_units(p, ch);
                total += i32::from(kern);
            }
        }
        prev = Some(ch);
    }

    total.max(0)
}

/// Computes an aggregate paragraph ink and stem-rhythm profile.
pub fn analyze_paragraph_ink(text: &str, font: FontCategory) -> ParagraphInkProfile {
    let mut total_units = 0i32;
    let mut ink_sum = 0u64;
    let mut letter_count = 0usize;
    let mut stem_sum = 0u64;
    let mut ascenders = 0usize;
    let mut descenders = 0usize;

    for ch in text.chars() {
        let gm = compute_glyph_metrics(ch, font);
        total_units += i32::from(gm.advance_width_units);
        if gm.ink_density > 0 && !ch.is_whitespace() {
            letter_count += 1;
            ink_sum += u64::from(gm.ink_density);
            stem_sum += u64::from(gm.vertical_stems);
            if gm.has_ascender {
                ascenders += 1;
            }
            if gm.has_descender {
                descenders += 1;
            }
        }
    }

    let total_em = total_units as f32 / 1000.0;
    let denom = letter_count.max(1) as f32;

    ParagraphInkProfile {
        total_em_width: total_em,
        mean_ink_density: ink_sum as f32 / denom,
        stem_rhythm_per_em: if total_em > 0.0 {
            stem_sum as f32 / total_em
        } else {
            0.0
        },
        ascender_ratio: ascenders as f32 / denom,
        descender_ratio: descenders as f32 / denom,
    }
}
