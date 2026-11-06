//! Donald Knuth & Michael Plass (1981) Dynamic Programming Paragraph Line-Breaking Algorithm.
//!
//! Formulates paragraph justification as a shortest-path problem on a directed acyclic graph
//! of legal breakpoints. Unlike greedy first-fit line-wrappers (used in basic word processors
//! and default browser layout), Knuth–Plass minimizes total cubic badness and inter-line
//! fitness jumps across the entire paragraph simultaneously.

use crate::metrics::{measure_string_em_units, FontCategory};

/// Fundamental box-and-glue primitive in a horizontal typesetting list.
#[derive(Debug, Clone, PartialEq)]
pub enum GalleyItem<'a> {
    /// Non-breakable typographic box (a word or syllable fragment) with width in 1/1000 em units.
    Box {
        slice: &'a str,
        width_units: i32,
        ink_density: u8,
    },
    /// Elastic inter-word space with ideal width, stretchability, and shrinkability in 1/1000 em.
    Glue {
        width_units: i32,
        stretch_units: i32,
        shrink_units: i32,
    },
    /// Potential breakpoint with aesthetic cost (`penalty`) and optional hyphen width if taken.
    Penalty {
        penalty_cost: i32,
        width_units: i32,
        flagged_hyphen: bool,
    },
}

/// Line tightness classification ($c \in \{0, 1, 2, 3\}$) used to penalize adjacent lines
/// with drastically different inter-word spacing (e.g., a very tight line next to a loose line).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FitnessClass {
    /// Adjustment ratio $r < -0.5$ (tightly compressed inter-word spaces).
    Tight = 0,
    /// Adjustment ratio $-0.5 \le r \le 0.5$ (ideal optical spacing).
    Normal = 1,
    /// Adjustment ratio $0.5 < r \le 1.0$ (moderately stretched spaces).
    Loose = 2,
    /// Adjustment ratio $r > 1.0$ (over-stretched "river-prone" spaces).
    VeryLoose = 3,
}

impl FitnessClass {
    #[inline]
    pub fn from_ratio(r: f32) -> Self {
        if r < -0.5 {
            FitnessClass::Tight
        } else if r <= 0.5 {
            FitnessClass::Normal
        } else if r <= 1.0 {
            FitnessClass::Loose
        } else {
            FitnessClass::VeryLoose
        }
    }

    #[inline]
    pub fn abs_diff(self, other: Self) -> i32 {
        ((self as i32) - (other as i32)).abs()
    }

    pub const fn russian_label(self) -> &'static str {
        match self {
            FitnessClass::Tight => "Плотная (Tight)",
            FitnessClass::Normal => "Оптимальная (Normal)",
            FitnessClass::Loose => "Разреженная (Loose)",
            FitnessClass::VeryLoose => "Слишком редкая (Over-loose)",
        }
    }
}

/// A single typeset line produced by either Knuth–Plass DP or Greedy First-Fit.
#[derive(Debug, Clone, PartialEq)]
pub struct TypesetLine {
    pub line_number: usize,
    pub text: String,
    pub word_count: usize,
    /// Natural unstretched width in 1/1000 em units.
    pub natural_width_units: i32,
    /// Target measure in 1/1000 em units.
    pub target_width_units: i32,
    /// Adjustment ratio $r \in [-1.0, +\infty)$. $r = 0.0$ is exact natural width.
    pub adjustment_ratio: f32,
    /// Knuth cubic badness: $b = \min(10000, \lfloor 100 \cdot |r|^3 + 0.5 \rfloor)$.
    pub badness: i32,
    /// Total demerits accumulated on this line break.
    pub demerits: i64,
    pub fitness: FitnessClass,
}

/// Comparative diagnostic between Greedy First-Fit and Knuth–Plass Optimal DP.
#[derive(Debug, Clone, PartialEq)]
pub struct LineBreakComparison {
    pub target_measure_em: f32,
    pub knuth_plass_lines: Vec<TypesetLine>,
    pub greedy_lines: Vec<TypesetLine>,
    pub knuth_plass_total_demerits: i64,
    pub greedy_total_demerits: i64,
    pub knuth_plass_max_badness: i32,
    pub greedy_max_badness: i32,
    pub demerit_reduction_pct: f32,
}

const INF_PENALTY: i32 = 10_000;
const LINE_PENALTY: i64 = 10;
const FITNESS_DEMERIT: i64 = 100;

/// Tokenizes a paragraph string into a `GalleyItem` stream of words, elastic inter-word glue,
/// and terminal finishing glue + forced break penalty.
pub fn build_galley_items<'a>(text: &'a str, font: FontCategory) -> Vec<GalleyItem<'a>> {
    let mut items = Vec::new();
    let words: Vec<&'a str> = text.split_whitespace().collect();

    // Standard Renaissance serif inter-word glue: 250 units (0.25 em)
    // Stretch: +125 units (1/8 em), Shrink: -83 units (1/12 em)
    let (space_w, space_stretch, space_shrink) = match font {
        FontCategory::RenaissanceSerif => (250, 130, 85),
        FontCategory::GeometricSans => (270, 135, 90),
        FontCategory::Monospace => (600, 0, 0),
    };

    for (idx, word) in words.iter().enumerate() {
        let w_units = measure_string_em_units(word, font);
        items.push(GalleyItem::Box {
            slice: word,
            width_units: w_units,
            ink_density: 58,
        });

        if idx + 1 < words.len() {
            items.push(GalleyItem::Glue {
                width_units: space_w,
                stretch_units: space_stretch,
                shrink_units: space_shrink,
            });
        }
    }

    // Add finishing glue (0 width, infinite stretch) and forced break penalty (-INF_PENALTY)
    if !items.is_empty() {
        items.push(GalleyItem::Glue {
            width_units: 0,
            stretch_units: 100_000,
            shrink_units: 0,
        });
        items.push(GalleyItem::Penalty {
            penalty_cost: -INF_PENALTY,
            width_units: 0,
            flagged_hyphen: false,
        });
    }

    items
}

/// Computes Knuth–Plass cubic badness $100 \cdot |r|^3$ and line demerits.
#[inline]
pub fn compute_badness_and_demerits(
    ratio: f32,
    penalty: i32,
    prev_fitness: FitnessClass,
    curr_fitness: FitnessClass,
    is_last_line: bool,
) -> (i32, i64) {
    if is_last_line {
        return (0, 0);
    }
    let abs_r = ratio.abs();
    let badness = if ratio < -1.0 {
        10_000
    } else {
        ((100.0 * abs_r * abs_r * abs_r).round() as i32).min(10_000)
    };

    let base = LINE_PENALTY + i64::from(badness);
    let p = i64::from(penalty);
    let mut demerits = if p >= 0 {
        (base + p) * (base + p)
    } else if p > -INF_PENALTY {
        base * base - p * p
    } else {
        base * base
    };

    if prev_fitness.abs_diff(curr_fitness) > 1 {
        demerits += FITNESS_DEMERIT;
    }

    (badness, demerits)
}

/// Layouts a paragraph using Donald Knuth & Michael Plass's Dynamic Programming algorithm.
///
/// Time complexity: $O(W \cdot K)$ where $W$ is word count and $K$ is maximum words per line.
pub fn layout_paragraph_knuth_plass(
    text: &str,
    target_width_units: i32,
    font: FontCategory,
) -> Vec<TypesetLine> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let n = words.len();
    if n == 0 {
        return Vec::new();
    }

    let word_widths: Vec<i32> = words
        .iter()
        .map(|w| measure_string_em_units(w, font))
        .collect();

    let (space_w, space_stretch, space_shrink) = match font {
        FontCategory::RenaissanceSerif => (250, 130, 85),
        FontCategory::GeometricSans => (270, 135, 90),
        FontCategory::Monospace => (600, 120, 60),
    };

    // Prefix sums of word widths for O(1) line span width lookup
    let mut prefix_w = vec![0i32; n + 1];
    for i in 0..n {
        prefix_w[i + 1] = prefix_w[i] + word_widths[i];
    }

    #[derive(Clone, Copy)]
    struct DpNode {
        total_demerits: i64,
        prev_break: usize,
        ratio: f32,
        badness: i32,
        fitness: FitnessClass,
        natural_width: i32,
    }

    let mut dp: Vec<Option<DpNode>> = vec![None; n + 1];
    dp[0] = Some(DpNode {
        total_demerits: 0,
        prev_break: 0,
        ratio: 0.0,
        badness: 0,
        fitness: FitnessClass::Normal,
        natural_width: 0,
    });

    for i in 0..n {
        let Some(curr_node) = dp[i] else {
            continue;
        };

        for j in (i + 1)..=n {
            let word_span = prefix_w[j] - prefix_w[i];
            let gaps = (j - i - 1) as i32;
            let natural_width = word_span + gaps * space_w;
            let is_last = j == n;

            let diff = target_width_units - natural_width;
            let ratio = if is_last {
                if diff >= 0 {
                    0.0
                } else {
                    let max_shrink = (gaps * space_shrink).max(1) as f32;
                    diff as f32 / max_shrink
                }
            } else if diff == 0 {
                0.0
            } else if diff > 0 {
                let max_stretch = (gaps * space_stretch).max(1) as f32;
                diff as f32 / max_stretch
            } else {
                let max_shrink = (gaps * space_shrink).max(1) as f32;
                diff as f32 / max_shrink
            };

            // If line is hopelessly overfull (ratio < -1.15) and has multiple words, stop extending
            if ratio < -1.15 && j > i + 1 {
                break;
            }

            // Skip absurdly loose lines unless no other option exists
            if ratio > 3.5 && !is_last && dp[j].is_some() {
                continue;
            }

            let fitness = FitnessClass::from_ratio(ratio);
            let (badness, line_demerits) =
                compute_badness_and_demerits(ratio, 0, curr_node.fitness, fitness, is_last);
            let candidate_total = curr_node.total_demerits.saturating_add(line_demerits);

            let should_update = match dp[j] {
                None => true,
                Some(existing) => candidate_total < existing.total_demerits,
            };

            if should_update {
                dp[j] = Some(DpNode {
                    total_demerits: candidate_total,
                    prev_break: i,
                    ratio,
                    badness,
                    fitness,
                    natural_width,
                });
            }
        }
    }

    // Reconstruct optimal breakpoint sequence backwards from n
    let mut breaks = Vec::new();
    let mut cursor = n;
    while cursor > 0 {
        if let Some(node) = dp[cursor] {
            breaks.push((node.prev_break, cursor, node));
            cursor = node.prev_break;
        } else {
            break;
        }
    }
    breaks.reverse();

    breaks
        .into_iter()
        .enumerate()
        .map(|(idx, (start, end, node))| {
            let prev_demerits = if start > 0 {
                dp[start].map(|d| d.total_demerits).unwrap_or(0)
            } else {
                0
            };
            TypesetLine {
                line_number: idx + 1,
                text: words[start..end].join(" "),
                word_count: end - start,
                natural_width_units: node.natural_width,
                target_width_units,
                adjustment_ratio: node.ratio,
                badness: node.badness,
                demerits: (node.total_demerits - prev_demerits).max(0),
                fitness: node.fitness,
            }
        })
        .collect()
}

/// Layouts a paragraph using standard Greedy First-Fit line wrapping for comparison.
pub fn layout_paragraph_greedy(
    text: &str,
    target_width_units: i32,
    font: FontCategory,
) -> Vec<TypesetLine> {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return Vec::new();
    }

    let (space_w, space_stretch, space_shrink) = match font {
        FontCategory::RenaissanceSerif => (250, 130, 85),
        FontCategory::GeometricSans => (270, 135, 90),
        FontCategory::Monospace => (600, 120, 60),
    };

    let mut lines = Vec::new();
    let mut start = 0usize;
    let mut current_width = 0i32;
    let mut prev_fitness = FitnessClass::Normal;

    for i in 0..words.len() {
        let w = measure_string_em_units(words[i], font);
        let added = if i == start { w } else { space_w + w };

        if i > start && current_width + added > target_width_units {
            // Emit line [start..i]
            let gaps = (i - start - 1) as i32;
            let diff = target_width_units - current_width;
            let ratio = if diff >= 0 {
                diff as f32 / (gaps * space_stretch).max(1) as f32
            } else {
                diff as f32 / (gaps * space_shrink).max(1) as f32
            };
            let fitness = FitnessClass::from_ratio(ratio);
            let (badness, demerits) =
                compute_badness_and_demerits(ratio, 0, prev_fitness, fitness, false);
            prev_fitness = fitness;

            lines.push(TypesetLine {
                line_number: lines.len() + 1,
                text: words[start..i].join(" "),
                word_count: i - start,
                natural_width_units: current_width,
                target_width_units,
                adjustment_ratio: ratio,
                badness,
                demerits,
                fitness,
            });

            start = i;
            current_width = w;
        } else {
            current_width += added;
        }
    }

    if start < words.len() {
        lines.push(TypesetLine {
            line_number: lines.len() + 1,
            text: words[start..].join(" "),
            word_count: words.len() - start,
            natural_width_units: current_width,
            target_width_units,
            adjustment_ratio: 0.0,
            badness: 0,
            demerits: 0,
            fitness: FitnessClass::Normal,
        });
    }

    lines
}

/// Runs both Knuth–Plass DP and Greedy First-Fit on the same paragraph and quantifies
/// the reduction in typographic demerits and peak line badness.
pub fn compare_paragraph_breaking(
    text: &str,
    target_measure_em: f32,
    font: FontCategory,
) -> LineBreakComparison {
    let target_units = (target_measure_em * 1000.0).round() as i32;
    let kp = layout_paragraph_knuth_plass(text, target_units, font);
    let gr = layout_paragraph_greedy(text, target_units, font);

    let kp_demerits: i64 = kp.iter().map(|l| l.demerits).sum();
    let gr_demerits: i64 = gr.iter().map(|l| l.demerits).sum();
    let kp_max_bad = kp.iter().map(|l| l.badness).max().unwrap_or(0);
    let gr_max_bad = gr.iter().map(|l| l.badness).max().unwrap_or(0);

    let reduction = if gr_demerits > 0 && gr_demerits >= kp_demerits {
        ((gr_demerits - kp_demerits) as f32 / gr_demerits as f32) * 100.0
    } else {
        0.0
    };

    LineBreakComparison {
        target_measure_em,
        knuth_plass_lines: kp,
        greedy_lines: gr,
        knuth_plass_total_demerits: kp_demerits,
        greedy_total_demerits: gr_demerits,
        knuth_plass_max_badness: kp_max_bad,
        greedy_max_badness: gr_max_bad,
        demerit_reduction_pct: reduction,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knuth_plass_beats_or_equals_greedy_demerits() {
        let sample = "Когда взгляд скользит по широкому полю тёплой архивной бумаги, каждая тонкая засечка удерживает равновесие между светом и тенью. Хороший шрифт не кричит о себе и не требует внимания.";
        let cmp = compare_paragraph_breaking(sample, 24.0, FontCategory::RenaissanceSerif);
        assert!(!cmp.knuth_plass_lines.is_empty());
        assert!(cmp.knuth_plass_total_demerits <= cmp.greedy_total_demerits);
    }
}
