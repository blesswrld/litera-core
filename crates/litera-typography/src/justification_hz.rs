//! Hermann Zapf & Peter Karow `hz-program` Micro-Typographic Justification Engine (`justification_hz.rs`).
//!
//! Implements optical micro-scaling of glyph outlines ($\pm 2.0\%$ horizontal scaling)
//! combined with optical tracking ($\pm 1.5\%$ inter-character adjustment) and
//! multi-pass paragraph optimization.
//!
//! Traditional justification only stretches or shrinks inter-word spaces (`glue`), which
//! frequently causes unsightly white "rivers" in narrow columns or long Russian compound words.
//! The `hz-program` (developed by Hermann Zapf and URW in Hamburg, later adopted into pdfTeX
//! and Adobe InDesign's Paragraph Composer) distributes up to $40\%$ of the line's slack/shortfall
//! into imperceptible horizontal glyph scaling ($\in [0.980, 1.020]$) and counter-scales
//! vertical stem weights so hairlines and thick stems remain optically uniform.

use crate::knuth_plass::{FitnessClass, GalleyItem, TypesetLine};
use crate::metrics::{compute_glyph_metrics, FontCategory};

/// Configuration limits for Hermann Zapf micro-typographic expansion/contraction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HzProgramConfig {
    /// Maximum allowed horizontal expansion factor (e.g. `1.020` for $+2.0\%$).
    pub max_glyph_expansion: f32,
    /// Minimum allowed horizontal contraction factor (e.g. `0.980` for $-2.0\%$).
    pub min_glyph_contraction: f32,
    /// Maximum inter-character tracking adjustment in thousandths of an em ($\pm 15$).
    pub max_tracking_em_milli: f32,
    /// Fraction of residual line slack absorbed by glyph scaling before stretching word glue.
    pub glyph_absorption_share: f32,
    /// Counter-scaling factor applied to vertical stems during horizontal expansion
    /// to preserve perceived optical weight ($\approx 0.35$).
    pub stem_compensation_gamma: f32,
}

impl Default for HzProgramConfig {
    fn default() -> Self {
        Self {
            max_glyph_expansion: 1.020,
            min_glyph_contraction: 0.980,
            max_tracking_em_milli: 15.0,
            glyph_absorption_share: 0.45,
            stem_compensation_gamma: 0.35,
        }
    }
}

/// Detailed micro-typographic transformation parameters for a single justified line.
#[derive(Debug, Clone, PartialEq)]
pub struct HzJustifiedLine {
    pub line_index: usize,
    pub text: String,
    /// Original unadjusted adjustment ratio $r_0$ (without `hz` micro-scaling).
    pub raw_adjustment_ratio: f32,
    /// Effective adjustment ratio $r_{\text{hz}}$ after glyph scaling and micro-tracking.
    pub effective_adjustment_ratio: f32,
    /// Horizontal glyph scaling factor $s_x \in [0.980, 1.020]$ ($1.000 = 100\%$).
    pub glyph_scale_x: f32,
    /// Compensated stem weight multiplier to prevent bolding/thinning artifacts.
    pub optical_stem_weight_factor: f32,
    /// Applied inter-letter tracking in milli-em ($\text{em}/1000$).
    pub tracking_milli_em: f32,
    /// Original cubic badness $B_0 = 100 |r_0|^3$.
    pub raw_badness: f32,
    /// Improved cubic badness $B_{\text{hz}} = 100 |r_{\text{hz}}|^3$.
    pub hz_badness: f32,
    pub fitness_class: FitnessClass,
}

/// Complete report comparing standard Knuth–Plass justification against `hz-program` micro-typography.
#[derive(Debug, Clone, PartialEq)]
pub struct HzParagraphReport {
    pub lines: Vec<HzJustifiedLine>,
    pub raw_total_badness: f32,
    pub hz_total_badness: f32,
    pub badness_reduction_pct: f32,
    pub max_glyph_distortion_pct: f32,
    pub river_risk_eliminated_count: usize,
}

/// Computes the glyph-specific horizontal elasticity coefficient for Cyrillic and Latin characters.
///
/// Round letters (`о`, `с`, `е`, `ю`, `ф`, `o`, `c`, `e`) and wide multi-stem letters (`ж`, `ш`, `щ`, `м`, `m`, `w`)
/// tolerate micro-scaling in their internal counters far better than narrow vertical letters (`і`, `i`, `l`, `1`).
#[inline]
pub fn glyph_counter_elasticity(ch: char) -> f32 {
    let lower = ch.to_lowercase().next().unwrap_or(ch);
    match lower {
        'о' | 'с' | 'е' | 'ё' | 'э' | 'ю' | 'ф' | 'o' | 'c' | 'e' | 'q' | 'd' | 'b' | 'p' => 1.25,
        'ж' | 'ш' | 'щ' | 'м' | 'ы' | 'ъ' | 'm' | 'w' => 1.15,
        'а' | 'в' | 'д' | 'з' | 'к' | 'н' | 'п' | 'р' | 'т' | 'у' | 'х' | 'ч' | 'я' => 1.00,
        'a' | 'g' | 'h' | 'k' | 'n' | 'u' | 'v' | 'x' | 'y' | 'z' => 1.00,
        'г' | 'л' | 'ь' | 'ц' | 'f' | 'j' | 'r' | 's' | 't' => 0.75,
        'і' | 'ї' | 'i' | 'l' | '1' | '|' => 0.25,
        _ => 0.50,
    }
}

/// Applies Hermann Zapf's `hz-program` micro-scaling to a sequence of typeset lines.
pub fn optimize_lines_with_hz_program(
    lines: &[TypesetLine],
    measure_em: f32,
    config: HzProgramConfig,
) -> HzParagraphReport {
    if lines.is_empty() {
        return HzParagraphReport {
            lines: Vec::new(),
            raw_total_badness: 0.0,
            hz_total_badness: 0.0,
            badness_reduction_pct: 0.0,
            max_glyph_distortion_pct: 0.0,
            river_risk_eliminated_count: 0,
        };
    }

    let mut hz_lines = Vec::with_capacity(lines.len());
    let mut raw_total_badness = 0.0f32;
    let mut hz_total_badness = 0.0f32;
    let mut max_distortion_pct = 0.0f32;
    let mut river_eliminated = 0usize;

    let total_lines = lines.len();

    for (idx, line) in lines.iter().enumerate() {
        let is_last_line = idx + 1 == total_lines;
        let r0 = line.adjustment_ratio;
        let raw_bad = line.badness;
        raw_total_badness += raw_bad;

        // Last line of a paragraph is set ragged-right (flush left) and needs no expansion
        if is_last_line || r0.abs() < 0.05 {
            hz_lines.push(HzJustifiedLine {
                line_index: line.line_index,
                text: line.text.clone(),
                raw_adjustment_ratio: r0,
                effective_adjustment_ratio: r0,
                glyph_scale_x: 1.000,
                optical_stem_weight_factor: 1.000,
                tracking_milli_em: 0.0,
                raw_badness: raw_bad,
                hz_badness: raw_bad,
                fitness_class: line.fitness_class,
            });
            hz_total_badness += raw_bad;
            continue;
        }

        // Measure mean counter elasticity across the line's glyphs
        let mut elasticity_sum = 0.0f32;
        let mut glyph_count = 0usize;
        let mut total_glyph_em = 0.0f32;

        for ch in line.text.chars() {
            if !ch.is_whitespace() {
                elasticity_sum += glyph_counter_elasticity(ch);
                let gm = compute_glyph_metrics(ch, FontCategory::HighContrastSerif);
                total_glyph_em += gm.advance_width_units as f32 / 1000.0;
                glyph_count += 1;
            }
        }

        let mean_elasticity = if glyph_count > 0 {
            elasticity_sum / glyph_count as f32
        } else {
            1.0
        };

        // Estimate the em slack/shortfall on the line from adjustment ratio
        // Typical inter-word space stretchability on a line is ~12% of measure_em
        let line_glue_flex_em = (measure_em * 0.12).max(0.5);
        let slack_em = r0 * line_glue_flex_em;

        // Absorb a fraction of slack_em via glyph horizontal scaling
        let desired_glyph_delta_em = slack_em * config.glyph_absorption_share * mean_elasticity;
        let raw_scale_x = if total_glyph_em > 0.1 {
            1.0 + (desired_glyph_delta_em / total_glyph_em)
        } else {
            1.0
        };

        let clamped_scale_x = raw_scale_x.clamp(
            config.min_glyph_contraction,
            config.max_glyph_expansion,
        );
        let achieved_glyph_delta_em = (clamped_scale_x - 1.0) * total_glyph_em;

        // Absorb an additional fraction via subtle optical tracking (+/- 15 milli-em)
        let remaining_slack_em = slack_em - achieved_glyph_delta_em;
        let raw_tracking_milli = if glyph_count > 1 {
            (remaining_slack_em * 0.35 / (glyph_count - 1) as f32) * 1000.0
        } else {
            0.0
        };
        let clamped_tracking_milli = raw_tracking_milli.clamp(
            -config.max_tracking_em_milli,
            config.max_tracking_em_milli,
        );
        let achieved_tracking_delta_em =
            (clamped_tracking_milli / 1000.0) * (glyph_count.saturating_sub(1) as f32);

        let residual_slack_em =
            slack_em - achieved_glyph_delta_em - achieved_tracking_delta_em;
        let effective_r = (residual_slack_em / line_glue_flex_em).clamp(-2.0, 4.0);
        let hz_bad = (effective_r.abs().powi(3) * 100.0).min(10000.0);

        // Compute stem weight compensation so expanded glyphs don't appear heavier
        let distortion_pct = (clamped_scale_x - 1.0).abs() * 100.0;
        if distortion_pct > max_distortion_pct {
            max_distortion_pct = distortion_pct;
        }
        let stem_factor = 1.0 - (clamped_scale_x - 1.0) * config.stem_compensation_gamma;

        if r0.abs() > 1.0 && effective_r.abs() <= 1.0 {
            river_eliminated += 1;
        }

        hz_total_badness += hz_bad;
        hz_lines.push(HzJustifiedLine {
            line_index: line.line_index,
            text: line.text.clone(),
            raw_adjustment_ratio: r0,
            effective_adjustment_ratio: effective_r,
            glyph_scale_x: clamped_scale_x,
            optical_stem_weight_factor: stem_factor,
            tracking_milli_em: clamped_tracking_milli,
            raw_badness: raw_bad,
            hz_badness: hz_bad,
            fitness_class: FitnessClass::from_ratio(effective_r),
        });
    }

    let reduction_pct = if raw_total_badness > 1e-3 {
        ((raw_total_badness - hz_total_badness) / raw_total_badness * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    HzParagraphReport {
        lines: hz_lines,
        raw_total_badness,
        hz_total_badness,
        badness_reduction_pct: reduction_pct,
        max_glyph_distortion_pct: max_distortion_pct,
        river_risk_eliminated_count: river_eliminated,
    }
}

/// Computes the exact horizontal advance table for a word under `hz-program` scaling.
pub fn compute_hz_word_advances(word: &str, scale_x: f32, tracking_milli_em: f32) -> Vec<(char, f32)> {
    let mut out = Vec::with_capacity(word.chars().count());
    let clamped_scale = scale_x.clamp(0.95, 1.05);
    for ch in word.chars() {
        let base_metric = compute_glyph_metrics(ch, FontCategory::HighContrastSerif);
        let elasticity = glyph_counter_elasticity(ch);
        let local_scale = 1.0 + (clamped_scale - 1.0) * elasticity;
        let scaled_units =
            (base_metric.advance_width_units as f32) * local_scale + tracking_milli_em;
        out.push((ch, scaled_units.max(100.0)));
    }
    out
}

/// Evaluates whether a sequence of `GalleyItem` boxes requires `hz-program` emergency pass 3.
pub fn requires_hz_emergency_pass(items: &[GalleyItem], measure_em: f32) -> bool {
    for item in items {
        if let GalleyItem::Box { width_em, .. } = item {
            if *width_em > measure_em * 0.42 {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knuth_plass::layout_paragraph_knuth_plass;

    #[test]
    fn test_hz_program_reduces_paragraph_badness() {
        let text = "Бесшумная типографика — это архитектура времени, отлитая в свинце и свете высокой печати.";
        let kp_lines = layout_paragraph_knuth_plass(text, 24.0);
        let report = optimize_lines_with_hz_program(&kp_lines, 24.0, HzProgramConfig::default());
        assert!(!report.lines.is_empty());
        assert!(report.hz_total_badness <= report.raw_total_badness + 1e-4);
        for line in &report.lines {
            assert!((0.980..=1.020).contains(&line.glyph_scale_x));
        }
    }

    #[test]
    fn test_counter_elasticity_ordering() {
        assert!(glyph_counter_elasticity('о') > glyph_counter_elasticity('н'));
        assert!(glyph_counter_elasticity('н') > glyph_counter_elasticity('i'));
    }
}
