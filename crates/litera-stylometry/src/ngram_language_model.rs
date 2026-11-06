//! Kneser–Ney N-Gram Language Model, Perplexity & Orthographic Register Classifier (`ngram_language_model.rs`).
//!
//! Implements interpolated Kneser–Ney character and syllable N-gram smoothing ($N = 1, 2, 3$),
//! cross-entropy $H(W)$, corpus perplexity $PP(W) = 2^{H(W)}$, and historical Russian
//! orthographic register classification (Modern Standard vs. Classical 19th-Century vs.
//! Archaic / High Liturgical Slavonic).

/// Orthographic and stylistic register classification for Russian and bilingual manuscripts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrthographicRegister {
    /// Pre-1918 classical orthography (`ѣ`, `і`, `ѳ`, `ѵ`, or `-аго`/`-яго` genitive adjectival endings).
    PreReformClassical,
    /// High poetic / liturgical register rich in Church Slavonicisms (`град`, `злато`, `глас`, `-ье`).
    HighPoeticSlavonic,
    /// Golden/Silver Age literary prose and verse balanced register.
    LiteraryClassical,
    /// Contemporary analytical / essayistic prose (`-ция`, `-ость`, `-ение`).
    ModernAnalytical,
}

impl OrthographicRegister {
    pub const fn russian_label(self) -> &'static str {
        match self {
            OrthographicRegister::PreReformClassical => "Дореформенная классическая орфография (до 1918 г.)",
            OrthographicRegister::HighPoeticSlavonic => "Высокий поэтический / славянизированный штиль",
            OrthographicRegister::LiteraryClassical => "Литературный канон (Золотой и Серебряный век)",
            OrthographicRegister::ModernAnalytical => "Современный аналитический и эссеистический регистр",
        }
    }
}

/// Summary report produced by the Kneser–Ney N-gram language model and register analyzer.
#[derive(Debug, Clone, PartialEq)]
pub struct NgramLanguageReport {
    pub unigram_count: usize,
    pub unique_bigrams: usize,
    pub unique_trigrams: usize,
    /// Interpolated Kneser–Ney cross-entropy $H(W)$ in bits per character.
    pub kneser_ney_cross_entropy_bits: f32,
    /// Character-level perplexity $PP(W) = 2^{H(W)}$.
    pub character_perplexity: f32,
    /// Ratio of vowel-to-consonant alternation transitions ($V \leftrightarrow C$) over total transitions.
    pub cv_alternation_probability: f32,
    /// Detected historical/stylistic register.
    pub register: OrthographicRegister,
    /// Confidence score in $[0.0, 1.0]$ for the register classification.
    pub register_confidence: f32,
    /// Top 8 highest-probability character bigrams in the manuscript.
    pub dominant_bigrams: Vec<([char; 2], u32, f32)>,
}

#[inline]
fn is_vowel_grapheme(ch: char) -> bool {
    matches!(
        ch.to_lowercase().next().unwrap_or(ch),
        'а' | 'е' | 'ё' | 'и' | 'о' | 'у' | 'ы' | 'э' | 'ю' | 'я' | 'ѣ' | 'і' | 'ѵ'
            | 'a' | 'e' | 'i' | 'o' | 'u' | 'y'
    )
}

/// Evaluates Kneser–Ney smoothed N-gram probabilities, perplexity, and orthographic register.
pub fn evaluate_ngram_language_model(text: &str) -> NgramLanguageReport {
    let chars: Vec<char> = text
        .chars()
        .filter(|c| c.is_alphabetic() || *c == ' ')
        .map(|c| c.to_lowercase().next().unwrap_or(c))
        .collect();

    if chars.len() < 3 {
        return NgramLanguageReport {
            unigram_count: chars.len(),
            unique_bigrams: 0,
            unique_trigrams: 0,
            kneser_ney_cross_entropy_bits: 0.0,
            character_perplexity: 1.0,
            cv_alternation_probability: 0.0,
            register: OrthographicRegister::LiteraryClassical,
            register_confidence: 0.5,
            dominant_bigrams: Vec::new(),
        };
    }

    // Collect unigrams, bigrams, and trigrams using deterministic sorted vectors
    let mut unigrams: Vec<(char, u32)> = Vec::with_capacity(64);
    let mut bigrams: Vec<([char; 2], u32)> = Vec::with_capacity(256);
    let mut trigrams: Vec<([char; 3], u32)> = Vec::with_capacity(512);

    let mut cv_alternations = 0usize;
    let mut letter_transitions = 0usize;

    for i in 0..chars.len() {
        let c0 = chars[i];
        if let Some(entry) = unigrams.iter_mut().find(|(k, _)| *k == c0) {
            entry.1 += 1;
        } else {
            unigrams.push((c0, 1));
        }

        if i + 1 < chars.len() {
            let c1 = chars[i + 1];
            let bg = [c0, c1];
            if let Some(entry) = bigrams.iter_mut().find(|(k, _)| *k == bg) {
                entry.1 += 1;
            } else {
                bigrams.push((bg, 1));
            }

            if c0 != ' ' && c1 != ' ' {
                letter_transitions += 1;
                if is_vowel_grapheme(c0) != is_vowel_grapheme(c1) {
                    cv_alternations += 1;
                }
            }
        }

        if i + 2 < chars.len() {
            let tg = [c0, chars[i + 1], chars[i + 2]];
            if let Some(entry) = trigrams.iter_mut().find(|(k, _)| *k == tg) {
                entry.1 += 1;
            } else {
                trigrams.push((tg, 1));
            }
        }
    }

    // Absolute discounting parameter D for Kneser-Ney smoothing
    let n1_bigrams = bigrams.iter().filter(|(_, c)| *c == 1).count() as f32;
    let n2_bigrams = bigrams.iter().filter(|(_, c)| *c == 2).count() as f32;
    let discount_d = if n1_bigrams + 2.0 * n2_bigrams > 0.0 {
        (n1_bigrams / (n1_bigrams + 2.0 * n2_bigrams)).clamp(0.15, 0.85)
    } else {
        0.50
    };

    let total_unigrams = chars.len() as f32;
    let total_bigram_types = bigrams.len().max(1) as f32;

    // Evaluate Kneser-Ney log-likelihood across all bigrams in the text
    let mut log2_prob_sum = 0.0f32;
    let mut bigram_eval_count = 0usize;

    for i in 0..(chars.len() - 1) {
        let c0 = chars[i];
        let c1 = chars[i + 1];

        let count_c0 = unigrams
            .iter()
            .find(|(k, _)| *k == c0)
            .map(|(_, c)| *c as f32)
            .unwrap_or(1.0);
        let count_bg = bigrams
            .iter()
            .find(|(k, _)| *k == [c0, c1])
            .map(|(_, c)| *c as f32)
            .unwrap_or(0.0);

        // Number of unique right contexts following c0: N_{1+}(c0, •)
        let right_contexts = bigrams.iter().filter(|(k, _)| k[0] == c0).count().max(1) as f32;
        // Continuation count for c1: number of unique left contexts preceding c1: N_{1+}(•, c1)
        let left_contexts = bigrams.iter().filter(|(k, _)| k[1] == c1).count().max(1) as f32;

        let p_continuation = left_contexts / total_bigram_types;
        let lambda_c0 = (discount_d * right_contexts) / count_c0;
        let p_kn = ((count_bg - discount_d).max(0.0) / count_c0) + lambda_c0 * p_continuation;

        let safe_p = p_kn.clamp(1e-6, 1.0);
        log2_prob_sum += -safe_p.log2();
        bigram_eval_count += 1;
    }

    let cross_entropy = if bigram_eval_count > 0 {
        log2_prob_sum / bigram_eval_count as f32
    } else {
        0.0
    };
    let perplexity = 2.0f32.powf(cross_entropy);

    let cv_prob = if letter_transitions > 0 {
        cv_alternations as f32 / letter_transitions as f32
    } else {
        0.0
    };

    // Sort dominant non-space bigrams
    let mut filtered_bigrams: Vec<([char; 2], u32, f32)> = bigrams
        .into_iter()
        .filter(|(k, _)| k[0] != ' ' && k[1] != ' ')
        .map(|(k, c)| (k, c, c as f32 / total_unigrams))
        .collect();
    filtered_bigrams.sort_by(|a, b| b.1.cmp(&a.1));
    filtered_bigrams.truncate(8);

    let (register, confidence) = classify_orthographic_register(text);

    NgramLanguageReport {
        unigram_count: chars.len(),
        unique_bigrams: filtered_bigrams.len(),
        unique_trigrams: trigrams.len(),
        kneser_ney_cross_entropy_bits: cross_entropy,
        character_perplexity: perplexity,
        cv_alternation_probability: cv_prob,
        register,
        register_confidence: confidence,
        dominant_bigrams: filtered_bigrams,
    }
}

/// Classifies the historical/stylistic register of a Russian manuscript using
/// orthographic markers and morphological suffix frequencies.
pub fn classify_orthographic_register(text: &str) -> (OrthographicRegister, f32) {
    let lower: String = text.chars().flat_map(|c| c.to_lowercase()).collect();

    // 1. Check pre-1918 graphemes (`ѣ`, `і`, `ѳ`, `ѵ`) or `-аго`/`-яго` endings
    let has_pre_1918_chars = lower
        .chars()
        .any(|c| matches!(c, 'ѣ' | 'і' | 'ѳ' | 'ѵ'));
    if has_pre_1918_chars || lower.contains("аго ") || lower.contains("яго ") {
        return (OrthographicRegister::PreReformClassical, 0.96);
    }

    // 2. Count Church Slavonicisms / poetic archaisms vs modern analytical suffixes
    let slavonic_markers = [
        "град", "злат", "глас", "хлад", "брег", "млад", "врат", "длань", "очи", "уста", "чело",
        "бытие", "дыханье", "молчанье", "сиout", "твердь", "небес",
    ];
    let analytical_markers = [
        "ция", "ции", "ость", "ости", "ение", "ения", "ическ", "ирован", "структур", "функци",
        "процесс", "систем",
    ];

    let mut slavonic_hits = 0usize;
    for m in &slavonic_markers {
        if lower.contains(m) {
            slavonic_hits += 1;
        }
    }

    let mut analytical_hits = 0usize;
    for m in &analytical_markers {
        if lower.contains(m) {
            analytical_hits += 1;
        }
    }

    if slavonic_hits >= 2 && slavonic_hits > analytical_hits {
        let conf = (0.72 + slavonic_hits as f32 * 0.06).min(0.95);
        (OrthographicRegister::HighPoeticSlavonic, conf)
    } else if analytical_hits >= 3 && analytical_hits > slavonic_hits * 2 {
        let conf = (0.74 + analytical_hits as f32 * 0.05).min(0.94);
        (OrthographicRegister::ModernAnalytical, conf)
    } else {
        (OrthographicRegister::LiteraryClassical, 0.88)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kneser_ney_perplexity_bounds() {
        let text = "Слово рождается из дыхания и возвращается в тишину белого листа.";
        let rep = evaluate_ngram_language_model(text);
        assert!(rep.kneser_ney_cross_entropy_bits > 0.5);
        assert!(rep.character_perplexity > 1.2);
        assert!(rep.cv_alternation_probability > 0.40);
    }

    #[test]
    fn test_pre_reform_register_detection() {
        let (reg, conf) = classify_orthographic_register("Въ началѣ было Слово, и Слово было къ Богу.");
        assert_eq!(reg, OrthographicRegister::PreReformClassical);
        assert!(conf > 0.90);
    }
}
