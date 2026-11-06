//! OpenType `GSUB` (Glyph Substitution) & `GPOS` (Glyph Positioning) Shaping Pipeline (`harfbuzz_shaper.rs`).
//!
//! Models the OpenType layout shaping stages executed by text engines (HarfBuzz, CoreText, DirectWrite):
//! 1. **Feature Tag Registry**: `[u8; 4]` OpenType tags (`liga`, `dlig`, `calt`, `kern`, `onum`, `lnum`,
//!    `pnum`, `tnum`, `smcp`, `c2sc`, `frac`, `ordn`, `zero`, `ss01`..`ss04`, `locl`).
//! 2. **GSUB Ligature & Alternate Substitution**: Multi-character ligature coalescing (`ffi`, `ffl`,
//!    `ff`, `fi`, `fl`, `st`, `ct`) with cluster index preservation so cursor hit-testing and
//!    phonetic mapping remain synchronized with the original UTF-8 byte offsets.
//! 3. **GPOS Contextual Kerning & Mark Attachment**: Adjusts horizontal `x_advance`, `x_offset`,
//!    and `y_offset` for combining acute stress marks (`\u{0301}`), Cyrillic breve/diaeresis,
//!    and punctuation optical spacing.

use crate::kerning::kern_pair_units;
use crate::metrics::{compute_glyph_metrics, FontCategory};

/// Standard 4-byte OpenType Layout Feature Tag (`GSUB` / `GPOS`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OpenTypeFeatureTag(pub [u8; 4]);

impl OpenTypeFeatureTag {
    pub const LIGA: Self = Self(*b"liga"); // Standard Ligatures (fi, fl, ff, ffi, ffl)
    pub const DLIG: Self = Self(*b"dlig"); // Discretionary / Historical Ligatures (st, ct)
    pub const CALT: Self = Self(*b"calt"); // Contextual Alternates
    pub const KERN: Self = Self(*b"kern"); // Pair Kerning
    pub const ONUM: Self = Self(*b"onum"); // Oldstyle (Minuscule) Figures
    pub const LNUM: Self = Self(*b"lnum"); // Lining (Majuscule) Figures
    pub const PNUM: Self = Self(*b"pnum"); // Proportional Figures
    pub const TNUM: Self = Self(*b"tnum"); // Tabular Figures
    pub const SMCP: Self = Self(*b"smcp"); // Lowercase to Small Capitals
    pub const C2SC: Self = Self(*b"c2sc"); // Capitals to Small Capitals
    pub const FRAC: Self = Self(*b"frac"); // Diagonal Fractions (1/2 -> ½)
    pub const LOCL: Self = Self(*b"locl"); // Localized Forms (Russian/Bulgarian/Serbian Cyrillic)

    #[inline]
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.0).unwrap_or("????")
    }

    pub const fn russian_description(self) -> &'static str {
        match &self.0 {
            b"liga" => "Стандартные лигатуры (fi, fl, ff, ffi, ffl)",
            b"dlig" => "Редкие и исторические лигатуры (st, ct)",
            b"calt" => "Контекстные альтернативы и росчерки",
            b"kern" => "Оптический кернинг глифовых пар",
            b"onum" => "Минускульные (прыгающие) книжные цифры",
            b"lnum" => "Маюскульные (ровные титульные) цифры",
            b"pnum" => "Пропорциональные цифры для сплошного набора",
            b"tnum" => "Табличные моноширинные цифры для колонок и сводок",
            b"smcp" => "Строчная капитель (Small Caps) для аббревиатур и рубрик",
            b"c2sc" => "Прописная капитель (Capitals to Small Caps)",
            b"frac" => "Косые книжные дроби (1/2, 1/4, 3/4)",
            b"locl" => "Локализованные формы кириллицы (русский / сербский / болгарский)",
            _ => "Пользовательская функция OpenType",
        }
    }
}

/// Active OpenType shaping configuration passed into the layout shaper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShapingFeatureSet {
    pub standard_ligatures: bool,
    pub discretionary_ligatures: bool,
    pub optical_kerning: bool,
    pub oldstyle_figures: bool,
    pub tabular_figures: bool,
    pub small_caps: bool,
    pub diagonal_fractions: bool,
}

impl Default for ShapingFeatureSet {
    fn default() -> Self {
        Self {
            standard_ligatures: true,
            discretionary_ligatures: false,
            optical_kerning: true,
            oldstyle_figures: true,
            tabular_figures: false,
            small_caps: false,
            diagonal_fractions: true,
        }
    }
}

/// A shaped glyph in the output glyph buffer after `GSUB` and `GPOS` passes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapedGlyph {
    /// Synthetic glyph ID or Unicode codepoint after substitution.
    pub glyph_codepoint: char,
    /// Original UTF-8 byte cluster offset in the source string.
    pub cluster_byte_offset: usize,
    /// Number of source Unicode scalar values coalesced into this glyph (e.g. 3 for `ffi -> ﬃ`).
    pub source_char_span: u8,
    /// Horizontal advance width in 1/1000 em units (including `GPOS` kerning).
    pub x_advance: i32,
    /// Horizontal placement offset in 1/1000 em units (for combining marks or protrusion).
    pub x_offset: i32,
    /// Vertical placement offset in 1/1000 em units (e.g. oldstyle figure descenders or combining accents).
    pub y_offset: i32,
    /// Optical ink density of the shaped glyph (0..=100).
    pub ink_density: u8,
    /// Feature tag that mutated this glyph, if any (`Some(LIGA)`, `Some(ONUM)`, etc.).
    pub applied_gsub_feature: Option<OpenTypeFeatureTag>,
}

/// Complete report returned by `shape_text_run`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapedGlyphBuffer {
    pub glyphs: Vec<ShapedGlyph>,
    pub total_advance_units: i32,
    pub ligatures_substituted: usize,
    pub kerning_adjustments_applied: usize,
    pub oldstyle_figures_shifted: usize,
}

/// Executes the two-pass OpenType `GSUB` + `GPOS` shaping pipeline on a UTF-8 text run.
pub fn shape_text_run(
    text: &str,
    font: FontCategory,
    features: ShapingFeatureSet,
) -> ShapedGlyphBuffer {
    let indexed_chars: Vec<(usize, char)> = text.char_indices().collect();
    let n = indexed_chars.len();
    let mut glyphs = Vec::with_capacity(n);

    let mut ligatures_substituted = 0usize;
    let mut oldstyle_figures_shifted = 0usize;

    // ========================================================================
    // PASS 1: GSUB (Glyph Substitution — Ligatures, Fractions, Small Caps)
    // ========================================================================
    let mut i = 0usize;
    while i < n {
        let (byte_off, ch) = indexed_chars[i];
        let c1 = indexed_chars.get(i + 1).map(|&(_, c)| c);
        let c2 = indexed_chars.get(i + 2).map(|&(_, c)| c);

        // 1A. 3-character standard ligatures (`ffi` -> `\u{FB03}`, `ffl` -> `\u{FB04}`)
        if features.standard_ligatures && font != FontCategory::Monospace {
            if ch == 'f' && c1 == Some('f') && c2 == Some('i') {
                glyphs.push(ShapedGlyph {
                    glyph_codepoint: '\u{FB03}', // ﬃ
                    cluster_byte_offset: byte_off,
                    source_char_span: 3,
                    x_advance: 760,
                    x_offset: 0,
                    y_offset: 0,
                    ink_density: 74,
                    applied_gsub_feature: Some(OpenTypeFeatureTag::LIGA),
                });
                ligatures_substituted += 1;
                i += 3;
                continue;
            }
            if ch == 'f' && c1 == Some('f') && c2 == Some('l') {
                glyphs.push(ShapedGlyph {
                    glyph_codepoint: '\u{FB04}', // ﬄ
                    cluster_byte_offset: byte_off,
                    source_char_span: 3,
                    x_advance: 780,
                    x_offset: 0,
                    y_offset: 0,
                    ink_density: 75,
                    applied_gsub_feature: Some(OpenTypeFeatureTag::LIGA),
                });
                ligatures_substituted += 1;
                i += 3;
                continue;
            }
        }

        // 1B. 3-character diagonal fractions (`1/2` -> `½`, `1/4` -> `¼`, `3/4` -> `¾`)
        if features.diagonal_fractions && c1 == Some('/') {
            let frac_char = match (ch, c2) {
                ('1', Some('2')) => Some('½'),
                ('1', Some('4')) => Some('¼'),
                ('3', Some('4')) => Some('¾'),
                ('1', Some('3')) => Some('⅓'),
                ('2', Some('3')) => Some('⅔'),
                _ => None,
            };
            if let Some(fc) = frac_char {
                glyphs.push(ShapedGlyph {
                    glyph_codepoint: fc,
                    cluster_byte_offset: byte_off,
                    source_char_span: 3,
                    x_advance: 720,
                    x_offset: 0,
                    y_offset: 0,
                    ink_density: 64,
                    applied_gsub_feature: Some(OpenTypeFeatureTag::FRAC),
                });
                ligatures_substituted += 1;
                i += 3;
                continue;
            }
        }

        // 1C. 2-character standard & discretionary ligatures (`fi`, `fl`, `ff`, `st`)
        if features.standard_ligatures && font != FontCategory::Monospace && ch == 'f' {
            let lig = match c1 {
                Some('i') => Some(('\u{FB01}', 520)), // ﬁ
                Some('l') => Some(('\u{FB02}', 535)), // ﬂ
                Some('f') => Some(('\u{FB00}', 560)), // ﬀ
                _ => None,
            };
            if let Some((lig_cp, adv)) = lig {
                glyphs.push(ShapedGlyph {
                    glyph_codepoint: lig_cp,
                    cluster_byte_offset: byte_off,
                    source_char_span: 2,
                    x_advance: adv,
                    x_offset: 0,
                    y_offset: 0,
                    ink_density: 68,
                    applied_gsub_feature: Some(OpenTypeFeatureTag::LIGA),
                });
                ligatures_substituted += 1;
                i += 2;
                continue;
            }
        }

        if features.discretionary_ligatures
            && font == FontCategory::RenaissanceSerif
            && ch == 's'
            && c1 == Some('t')
        {
            glyphs.push(ShapedGlyph {
                glyph_codepoint: '\u{FB06}', // ﬆ
                cluster_byte_offset: byte_off,
                source_char_span: 2,
                x_advance: 640,
                x_offset: 0,
                y_offset: 0,
                ink_density: 62,
                applied_gsub_feature: Some(OpenTypeFeatureTag::DLIG),
            });
            ligatures_substituted += 1;
            i += 2;
            continue;
        }

        // 1D. Single character shaping (Small Caps `smcp` and Oldstyle Figures `onum`)
        let gm = compute_glyph_metrics(ch, font);
        let mut x_adv = i32::from(gm.advance_width_units);
        let mut y_off = 0i32;
        let mut gsub_tag = None;

        if ch.is_ascii_digit() {
            if features.tabular_figures {
                x_adv = 560; // Fixed tabular figure width
                gsub_tag = Some(OpenTypeFeatureTag::TNUM);
            } else if features.oldstyle_figures && font == FontCategory::RenaissanceSerif {
                // Oldstyle figures: 3, 4, 5, 7, 9 descend below baseline; 6, 8 ascend above x-height
                match ch {
                    '3' | '4' | '5' | '7' | '9' => {
                        y_off = -140;
                        x_adv = 475;
                        oldstyle_figures_shifted += 1;
                    }
                    '6' | '8' => {
                        y_off = 110;
                        x_adv = 490;
                        oldstyle_figures_shifted += 1;
                    }
                    '0' | '1' | '2' => {
                        x_adv = 445;
                    }
                    _ => {}
                }
                gsub_tag = Some(OpenTypeFeatureTag::ONUM);
            }
        } else if features.small_caps && ch.is_lowercase() {
            // Small capitals are ~78% of full capital height with slightly wider tracking
            x_adv = ((x_adv as f32) * 1.08) as i32;
            gsub_tag = Some(OpenTypeFeatureTag::SMCP);
        }

        glyphs.push(ShapedGlyph {
            glyph_codepoint: ch,
            cluster_byte_offset: byte_off,
            source_char_span: 1,
            x_advance: x_adv,
            x_offset: 0,
            y_offset: y_off,
            ink_density: gm.ink_density,
            applied_gsub_feature: gsub_tag,
        });
        i += 1;
    }

    // ========================================================================
    // PASS 2: GPOS (Glyph Positioning — Pair Kerning & Combining Accent Marks)
    // ========================================================================
    let mut kerning_adjustments_applied = 0usize;
    if features.optical_kerning && font != FontCategory::Monospace && glyphs.len() >= 2 {
        for k in 0..(glyphs.len() - 1) {
            let left_cp = glyphs[k].glyph_codepoint;
            let right_cp = glyphs[k + 1].glyph_codepoint;

            // Zero-advance combining acute accent mark (`\u{0301}`) anchors above preceding vowel
            if right_cp == '\u{0301}' {
                glyphs[k + 1].x_advance = 0;
                glyphs[k + 1].x_offset = -(glyphs[k].x_advance / 2);
                glyphs[k + 1].y_offset = 180;
                continue;
            }

            let (kern_units, _) = kern_pair_units(left_cp, right_cp);
            if kern_units != 0 {
                glyphs[k].x_advance = (glyphs[k].x_advance + i32::from(kern_units)).max(120);
                kerning_adjustments_applied += 1;
            }
        }
    }

    let total_advance_units: i32 = glyphs.iter().map(|g| g.x_advance).sum();

    ShapedGlyphBuffer {
        glyphs,
        total_advance_units,
        ligatures_substituted,
        kerning_adjustments_applied,
        oldstyle_figures_shifted,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gsub_ligatures_and_oldstyle_figures() {
        let buf = shape_text_run(
            "Office flight 1926 и Точка",
            FontCategory::RenaissanceSerif,
            ShapingFeatureSet::default(),
        );
        // "ffi" -> 1 ligature, "fl" -> 1 ligature
        assert_eq!(buf.ligatures_substituted, 2);
        // "1926" contains '9' and '6' which shift in oldstyle figures
        assert_eq!(buf.oldstyle_figures_shifted, 2);
        // "То" triggers kerning pair adjustment
        assert!(buf.kerning_adjustments_applied >= 1);
    }
}
