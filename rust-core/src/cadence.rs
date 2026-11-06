//! Zero-copy rhythmic cadence and sentence structure analyzer.
//!
//! Uses lifetime-bound slices (`&'a str`) to segment sentences and words
//! without intermediate String heap allocations where possible.

use crate::phonetics::{char_ink_density, classify_char, fnv1a_hash, PhonemeClass};

#[derive(Debug, Clone, PartialEq)]
pub struct WordMetric<'a> {
    pub slice: &'a str,
    pub syllables: usize,
    pub char_count: usize,
    pub dominant_class: PhonemeClass,
    pub ink_density: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SentenceCadence<'a> {
    pub index: usize,
    pub raw_slice: &'a str,
    pub word_count: usize,
    pub syllable_count: usize,
    pub avg_syllables_per_word: f32,
    pub tempo_tag: TempoClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TempoClass {
    /// Short, percussive sentence (1..=5 words)
    Staccato,
    /// Balanced chamber cadence (6..=17 words)
    Moderato,
    /// Long, flowing periodic sentence (18+ words)
    Legato,
}

#[derive(Debug, Clone)]
pub struct ManuscriptProfile<'a> {
    pub char_count: usize,
    pub letter_count: usize,
    pub word_count: usize,
    pub syllable_count: usize,
    pub sentence_count: usize,
    pub vowel_ratio: f32,
    pub sonorant_ratio: f32,
    pub fricative_ratio: f32,
    pub plosive_ratio: f32,
    pub mean_ink_density: f32,
    /// Rhythm variation index (0.0 = monotonous, 1.0 = highly dynamic sentence length variance)
    pub cadence_variance: f32,
    pub acoustic_hash: u64,
    pub sentences: Vec<SentenceCadence<'a>>,
    pub words: Vec<WordMetric<'a>>,
}

/// Counts vowel nuclei as an accurate proxy for phonological syllables in Russian and English.
/// Includes English silent-e adjustment and diphthong coalescing.
pub fn count_syllables(word: &str) -> usize {
    let mut count = 0usize;
    let mut prev_was_vowel = false;
    let mut has_cyrillic = false;

    for ch in word.chars() {
        if ('\u{0400}'..='\u{04FF}').contains(&ch) {
            has_cyrillic = true;
        }
        let is_v = classify_char(ch) == PhonemeClass::Vowel;
        if is_v {
            // In Russian, every vowel forms a syllable nucleus; in Latin, adjacent vowels often coalesce.
            if has_cyrillic || !prev_was_vowel {
                count += 1;
            }
            prev_was_vowel = true;
        } else if ch.is_alphabetic() {
            prev_was_vowel = false;
        }
    }

    if count == 0 && word.chars().any(|c| c.is_alphabetic()) {
        1
    } else {
        count
    }
}

/// Analyzes a manuscript in O(N) time where N is byte length of `text`.
pub fn analyze_manuscript<'a>(text: &'a str) -> ManuscriptProfile<'a> {
    let mut vowels = 0usize;
    let mut sonorants = 0usize;
    let mut fricatives = 0usize;
    let mut plosives = 0usize;
    let mut ink_accum = 0u64;
    let mut letter_count = 0usize;
    let mut char_count = 0usize;

    for ch in text.chars() {
        char_count += 1;
        let density = char_ink_density(ch);
        if density > 0 {
            ink_accum += u64::from(density);
        }
        match classify_char(ch) {
            PhonemeClass::Vowel => {
                vowels += 1;
                letter_count += 1;
            }
            PhonemeClass::Sonorant => {
                sonorants += 1;
                letter_count += 1;
            }
            PhonemeClass::Fricative => {
                fricatives += 1;
                letter_count += 1;
            }
            PhonemeClass::Plosive => {
                plosives += 1;
                letter_count += 1;
            }
            PhonemeClass::Modifier => {
                letter_count += 1;
            }
            PhonemeClass::NonPhonetic => {}
        }
    }

    let mut words = Vec::new();
    for raw_token in text.split_whitespace() {
        let trimmed = raw_token.trim_matches(|c: char| !c.is_alphabetic() && c != '-' && c != '’' && c != '\'');
        if trimmed.is_empty() {
            continue;
        }
        let syllables = count_syllables(trimmed);
        let mut v = 0usize;
        let mut s = 0usize;
        let mut f = 0usize;
        let mut p = 0usize;
        let mut w_ink = 0u32;
        let mut w_chars = 0usize;

        for ch in trimmed.chars() {
            w_chars += 1;
            w_ink += u32::from(char_ink_density(ch));
            match classify_char(ch) {
                PhonemeClass::Vowel => v += 1,
                PhonemeClass::Sonorant => s += 1,
                PhonemeClass::Fricative => f += 1,
                PhonemeClass::Plosive => p += 1,
                _ => {}
            }
        }

        // Determine dominant consonantal or vocalic color of the word
        let dominant_class = if s >= f && s >= p && s > 0 {
            PhonemeClass::Sonorant
        } else if f >= s && f >= p && f > 0 {
            PhonemeClass::Fricative
        } else if p > 0 {
            PhonemeClass::Plosive
        } else {
            PhonemeClass::Vowel
        };

        words.push(WordMetric {
            slice: trimmed,
            syllables,
            char_count: w_chars,
            dominant_class,
            ink_density: if w_chars > 0 { w_ink as f32 / w_chars as f32 } else { 0.0 },
        });
    }

    // Segment sentences while preserving zero-copy `&'a str` slices
    let mut sentences = Vec::new();
    let mut start_idx = 0usize;

    for (idx, ch) in text.char_indices() {
        if matches!(ch, '.' | '!' | '?' | '…' | '\n') {
            let end_idx = idx + ch.len_utf8();
            let candidate = text[start_idx..end_idx].trim();
            if !candidate.is_empty() && candidate.chars().any(|c| c.is_alphabetic()) {
                sentences.push(build_sentence_cadence(sentences.len(), candidate));
            }
            start_idx = end_idx;
        }
    }

    if start_idx < text.len() {
        let tail = text[start_idx..].trim();
        if !tail.is_empty() && tail.chars().any(|c| c.is_alphabetic()) {
            sentences.push(build_sentence_cadence(sentences.len(), tail));
        }
    }

    let word_count = words.len();
    let syllable_count: usize = words.iter().map(|w| w.syllables).sum();
    let denom = letter_count.max(1) as f32;

    // Compute normalized standard deviation (coefficient of variation) of sentence lengths
    let cadence_variance = if sentences.len() > 1 {
        let mean_words = word_count as f32 / sentences.len() as f32;
        let var_sum: f32 = sentences
            .iter()
            .map(|s| {
                let diff = s.word_count as f32 - mean_words;
                diff * diff
            })
            .sum();
        let std_dev = (var_sum / sentences.len() as f32).sqrt();
        (std_dev / mean_words.max(1.0)).clamp(0.0, 1.0)
    } else {
        0.0
    };

    ManuscriptProfile {
        char_count,
        letter_count,
        word_count,
        syllable_count,
        sentence_count: sentences.len(),
        vowel_ratio: vowels as f32 / denom,
        sonorant_ratio: sonorants as f32 / denom,
        fricative_ratio: fricatives as f32 / denom,
        plosive_ratio: plosives as f32 / denom,
        mean_ink_density: if letter_count > 0 {
            ink_accum as f32 / letter_count as f32
        } else {
            0.0
        },
        cadence_variance,
        acoustic_hash: fnv1a_hash(text),
        sentences,
        words,
    }
}

fn build_sentence_cadence<'a>(index: usize, slice: &'a str) -> SentenceCadence<'a> {
    let mut word_count = 0usize;
    let mut syllable_count = 0usize;

    for token in slice.split_whitespace() {
        let clean = token.trim_matches(|c: char| !c.is_alphabetic());
        if !clean.is_empty() {
            word_count += 1;
            syllable_count += count_syllables(clean);
        }
    }

    let tempo_tag = match word_count {
        0..=5 => TempoClass::Staccato,
        6..=17 => TempoClass::Moderato,
        _ => TempoClass::Legato,
    };

    SentenceCadence {
        index,
        raw_slice: slice,
        word_count,
        syllable_count,
        avg_syllables_per_word: if word_count > 0 {
            syllable_count as f32 / word_count as f32
        } else {
            0.0
        },
        tempo_tag,
    }
}
