//! Prosodic meter and rhythmic cadence scanner for Russian and English verse/prose.
//!
//! Models classical syllabo-tonic feet (Iamb, Trochee, Dactyl, Amphibrach, Anapaest)
//! as well as accentual verse (Dolnik / Tactovik) and rhythmic prose periods.
//! Inspired by the formalist statistical prosody of Andrey Bely ("Символизм", 1910),
//! Boris Tomashevsky ("О стихе", 1929), and Mikhail Gasparov ("Современный русский стих", 1974).

use crate::{classify_unicode_char, PhonemeClass};

/// Classical syllabo-tonic metrical foot or prose cadence pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricalFoot {
    /// Trochee (Хорей): `[ / u ]` — falling binary meter (stress on odd syllables: 1, 3, 5...).
    Trochee,
    /// Iamb (Ямб): `[ u / ]` — rising binary meter (stress on even syllables: 2, 4, 6...).
    Iamb,
    /// Dactyl (Дактиль): `[ / u u ]` — falling ternary meter (stress on 1, 4, 7...).
    Dactyl,
    /// Amphibrach (Амфибрахий): `[ u / u ]` — rocking ternary meter (stress on 2, 5, 8...).
    Amphibrach,
    /// Anapaest (Анапест): `[ u u / ]` — rising ternary meter (stress on 3, 6, 9...).
    Anapaest,
    /// Dolnik / Accentual (Дольник / Акцентный ритм): variable 1-2 unstressed intervals.
    Dolnik,
    /// Free Polymetric Prose (Свободная прозаическая каденция).
    FreeProse,
}

impl MetricalFoot {
    pub const fn russian_name(self) -> &'static str {
        match self {
            MetricalFoot::Trochee => "Хорей (— ◡)",
            MetricalFoot::Iamb => "Ямб (◡ —)",
            MetricalFoot::Dactyl => "Дактиль (— ◡ ◡)",
            MetricalFoot::Amphibrach => "Амфибрахий (◡ — ◡)",
            MetricalFoot::Anapaest => "Анапест (◡ ◡ —)",
            MetricalFoot::Dolnik => "Дольник (переменный шаг)",
            MetricalFoot::FreeProse => "Свободная каденция прозы",
        }
    }

    pub const fn schematic_pattern(self) -> &'static str {
        match self {
            MetricalFoot::Trochee => "— ◡ | — ◡ | — ◡",
            MetricalFoot::Iamb => "◡ — | ◡ — | ◡ —",
            MetricalFoot::Dactyl => "— ◡ ◡ | — ◡ ◡",
            MetricalFoot::Amphibrach => "◡ — ◡ | ◡ — ◡",
            MetricalFoot::Anapaest => "◡ ◡ — | ◡ ◡ —",
            MetricalFoot::Dolnik => "— ◡ (◡) — ◡ (◡) —",
            MetricalFoot::FreeProse => "◡ — ◡ ◡ — | — ◡ ◡",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StressConfidence {
    /// Explicit orthographic stress (`ё` or acute accent `\u{0301}`) or monosyllabic content word.
    Exact,
    /// Morphophonological suffix/prefix heuristic (high accuracy on Russian roots).
    MorphologicalHeuristic,
    /// Positional center-of-gravity fallback.
    StatisticalFallback,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProsodicLineScan {
    pub line_index: usize,
    pub total_syllables: usize,
    /// 1-indexed syllable positions carrying primary word stress within the clause/line.
    pub stressed_positions: Vec<usize>,
    pub detected_foot: MetricalFoot,
    pub regularity_score: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProsodicProfile {
    pub dominant_foot: MetricalFoot,
    pub binary_meter_affinity: f32,
    pub ternary_meter_affinity: f32,
    pub mean_regularity: f32,
    pub masculine_ending_ratio: f32,
    pub feminine_ending_ratio: f32,
    pub dactylic_ending_ratio: f32,
    pub lines: Vec<ProsodicLineScan>,
}

/// Estimates which syllable (0-indexed within the word) carries primary lexical stress,
/// returning `(stressed_syllable_index, total_syllables, StressConfidence)`.
///
/// Uses exact detection for `ё`, combining acute accents, monosyllables, and common
/// Russian morphological endings (`-ость`, `-ение`, `-ание`, `-ский`, `-ова`, `-ться`, etc.).
pub fn estimate_word_stress(word: &str) -> (usize, usize, StressConfidence) {
    let mut vowel_step = 0usize;
    let mut yo_syllable: Option<usize> = None;
    let mut acute_syllable: Option<usize> = None;
    let mut has_cyrillic = false;
    let mut prev_was_vowel = false;

    let chars: Vec<char> = word.chars().collect();
    for i in 0..chars.len() {
        let ch = chars[i];
        if ('\u{0400}'..='\u{04FF}').contains(&ch) {
            has_cyrillic = true;
        }
        if ch == '\u{0301}' && vowel_step > 0 {
            acute_syllable = Some(vowel_step - 1);
            continue;
        }
        if classify_unicode_char(ch) == PhonemeClass::Vowel {
            if has_cyrillic || !prev_was_vowel {
                if ch == 'ё' || ch == 'Ё' {
                    yo_syllable = Some(vowel_step);
                }
                vowel_step += 1;
            }
            prev_was_vowel = true;
        } else if ch.is_alphabetic() {
            prev_was_vowel = false;
        }
    }

    if vowel_step == 0 {
        return (0, 0, StressConfidence::Exact);
    }
    if vowel_step == 1 {
        return (0, 1, StressConfidence::Exact);
    }
    if let Some(acute) = acute_syllable {
        return (acute.min(vowel_step - 1), vowel_step, StressConfidence::Exact);
    }
    if let Some(yo) = yo_syllable {
        return (yo, vowel_step, StressConfidence::Exact);
    }

    let lower: String = word.chars().flat_map(|c| c.to_lowercase()).collect();

    // High-frequency Russian morphological stress anchors
    if lower.ends_with("ение")
        || lower.ends_with("ание")
        || lower.ends_with("ения")
        || lower.ends_with("ания")
        || lower.ends_with("овать")
        || lower.ends_with("евать")
    {
        return (
            vowel_step.saturating_sub(3),
            vowel_step,
            StressConfidence::MorphologicalHeuristic,
        );
    }

    if lower.ends_with("ость")
        || lower.ends_with("ости")
        || lower.ends_with("ый")
        || lower.ends_with("ий")
        || lower.ends_with("ая")
        || lower.ends_with("ое")
        || lower.ends_with("ые")
        || lower.ends_with("ого")
        || lower.ends_with("ому")
        || lower.ends_with("ыми")
    {
        return (
            vowel_step.saturating_sub(2),
            vowel_step,
            StressConfidence::MorphologicalHeuristic,
        );
    }

    if lower.ends_with("ок")
        || lower.ends_with("ек")
        || lower.ends_with("он")
        || lower.ends_with("ать")
        || lower.ends_with("ить")
        || lower.ends_with("уть")
        || lower.ends_with("на")
        || lower.ends_with("та")
    {
        return (
            vowel_step - 1,
            vowel_step,
            StressConfidence::MorphologicalHeuristic,
        );
    }

    // Default Russian root-penultimate tendency for 2-4 syllable words
    let fallback_idx = if vowel_step == 2 {
        0
    } else {
        (vowel_step / 2).saturating_sub(1).max(vowel_step.saturating_sub(2))
    };
    (fallback_idx, vowel_step, StressConfidence::StatisticalFallback)
}

/// Unstressed function words (clitics: prepositions, conjunctions, particles)
/// that attach prosodically to the adjacent phonological word in Russian and English.
fn is_unstressed_clitic(word: &str) -> bool {
    let lower: String = word.chars().flat_map(|c| c.to_lowercase()).collect();
    matches!(
        lower.as_str(),
        "в" | "во"
            | "на"
            | "с"
            | "со"
            | "к"
            | "ко"
            | "у"
            | "о"
            | "об"
            | "от"
            | "до"
            | "из"
            | "за"
            | "по"
            | "под"
            | "над"
            | "при"
            | "про"
            | "без"
            | "для"
            | "и"
            | "а"
            | "но"
            | "да"
            | "или"
            | "не"
            | "ни"
            | "же"
            | "бы"
            | "ли"
            | "как"
            | "что"
            | "the"
            | "a"
            | "an"
            | "in"
            | "on"
            | "at"
            | "to"
            | "of"
            | "and"
            | "or"
            | "by"
    )
}

/// Scans a full manuscript into clauses/lines and determines its prosodic meter profile.
pub fn scan_prosodic_profile(text: &str) -> ProsodicProfile {
    let mut lines = Vec::new();
    let mut masc_endings = 0usize;
    let mut fem_endings = 0usize;
    let mut dact_endings = 0usize;

    let mut binary_score_sum = 0.0f32;
    let mut ternary_score_sum = 0.0f32;
    let mut regularity_sum = 0.0f32;

    let mut foot_counts = [0usize; 7];

    for (line_idx, segment) in text
        .split(|c| matches!(c, '\n' | '.' | '!' | '?' | ';' | '—'))
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .enumerate()
    {
        let mut running_syllable = 0usize;
        let mut stressed_positions = Vec::new();
        let mut last_stress_pos = 0usize;

        for token in segment.split_whitespace() {
            let clean = token.trim_matches(|c: char| !c.is_alphabetic());
            if clean.is_empty() {
                continue;
            }
            let (stress_offset, syl_count, _) = estimate_word_stress(clean);
            if syl_count == 0 {
                continue;
            }
            if !is_unstressed_clitic(clean) {
                let pos_1based = running_syllable + stress_offset + 1;
                stressed_positions.push(pos_1based);
                last_stress_pos = pos_1based;
            }
            running_syllable += syl_count;
        }

        if running_syllable == 0 {
            continue;
        }

        // Classify clausula (ending after final stress)
        if last_stress_pos > 0 && running_syllable >= last_stress_pos {
            match running_syllable - last_stress_pos {
                0 => masc_endings += 1,
                1 => fem_endings += 1,
                _ => dact_endings += 1,
            }
        }

        let (foot, reg, bin_aff, tern_aff) =
            classify_line_meter(running_syllable, &stressed_positions);
        binary_score_sum += bin_aff;
        ternary_score_sum += tern_aff;
        regularity_sum += reg;

        let f_idx = match foot {
            MetricalFoot::Trochee => 0,
            MetricalFoot::Iamb => 1,
            MetricalFoot::Dactyl => 2,
            MetricalFoot::Amphibrach => 3,
            MetricalFoot::Anapaest => 4,
            MetricalFoot::Dolnik => 5,
            MetricalFoot::FreeProse => 6,
        };
        foot_counts[f_idx] += 1;

        lines.push(ProsodicLineScan {
            line_index: line_idx,
            total_syllables: running_syllable,
            stressed_positions,
            detected_foot: foot,
            regularity_score: reg,
        });
    }

    let n = lines.len().max(1) as f32;
    let dominant_foot = match foot_counts
        .iter()
        .enumerate()
        .max_by_key(|&(_, count)| count)
        .map(|(idx, _)| idx)
        .unwrap_or(6)
    {
        0 => MetricalFoot::Trochee,
        1 => MetricalFoot::Iamb,
        2 => MetricalFoot::Dactyl,
        3 => MetricalFoot::Amphibrach,
        4 => MetricalFoot::Anapaest,
        5 => MetricalFoot::Dolnik,
        _ => MetricalFoot::FreeProse,
    };

    ProsodicProfile {
        dominant_foot,
        binary_meter_affinity: binary_score_sum / n,
        ternary_meter_affinity: ternary_score_sum / n,
        mean_regularity: regularity_sum / n,
        masculine_ending_ratio: masc_endings as f32 / n,
        feminine_ending_ratio: fem_endings as f32 / n,
        dactylic_ending_ratio: dact_endings as f32 / n,
        lines,
    }
}

fn classify_line_meter(
    total_syllables: usize,
    stresses: &[usize],
) -> (MetricalFoot, f32, f32, f32) {
    if stresses.is_empty() || total_syllables < 3 {
        return (MetricalFoot::FreeProse, 0.5, 0.5, 0.5);
    }

    let count = stresses.len() as f32;
    // Trochee: odd positions (pos % 2 == 1)
    let trochee_hits = stresses.iter().filter(|&&p| p % 2 == 1).count() as f32 / count;
    // Iamb: even positions (pos % 2 == 0)
    let iamb_hits = stresses.iter().filter(|&&p| p % 2 == 0).count() as f32 / count;
    // Dactyl: pos % 3 == 1
    let dactyl_hits = stresses.iter().filter(|&&p| p % 3 == 1).count() as f32 / count;
    // Amphibrach: pos % 3 == 2
    let amphibrach_hits = stresses.iter().filter(|&&p| p % 3 == 2).count() as f32 / count;
    // Anapaest: pos % 3 == 0
    let anapaest_hits = stresses.iter().filter(|&&p| p % 3 == 0).count() as f32 / count;

    let binary_best = trochee_hits.max(iamb_hits);
    let ternary_best = dactyl_hits.max(amphibrach_hits).max(anapaest_hits);
    let overall_best = binary_best.max(ternary_best);

    if overall_best < 0.58 {
        return (MetricalFoot::FreeProse, overall_best, binary_best, ternary_best);
    }

    if ternary_best >= binary_best && ternary_best >= 0.65 {
        if dactyl_hits >= amphibrach_hits && dactyl_hits >= anapaest_hits {
            return (MetricalFoot::Dactyl, dactyl_hits, binary_best, ternary_best);
        } else if amphibrach_hits >= anapaest_hits {
            return (
                MetricalFoot::Amphibrach,
                amphibrach_hits,
                binary_best,
                ternary_best,
            );
        } else {
            return (
                MetricalFoot::Anapaest,
                anapaest_hits,
                binary_best,
                ternary_best,
            );
        }
    }

    if binary_best >= 0.70 {
        if iamb_hits >= trochee_hits {
            return (MetricalFoot::Iamb, iamb_hits, binary_best, ternary_best);
        } else {
            return (MetricalFoot::Trochee, trochee_hits, binary_best, ternary_best);
        }
    }

    (MetricalFoot::Dolnik, overall_best, binary_best, ternary_best)
}
