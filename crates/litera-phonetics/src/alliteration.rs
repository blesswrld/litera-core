//! Alliteration (consonant repetition) and Assonance (vowel harmony) proximity graphs.
//!
//! Identifies sonic echoes across adjacent words within a sliding window of $k = 3$ words.

use crate::{classify_unicode_char, PhonemeClass};

#[derive(Debug, Clone, PartialEq)]
pub struct AlliterationChain {
    pub consonant: char,
    pub word_indices: Vec<usize>,
    pub intensity: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssonanceCluster {
    pub vowel: char,
    pub occurrences: usize,
    pub share_of_vowels: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SoundDeviceReport {
    pub alliteration_density: f32,
    pub dominant_alliterations: Vec<AlliterationChain>,
    pub dominant_assonances: Vec<AssonanceCluster>,
}

/// Scans a text for alliterative consonant chains and dominant vowel assonance clusters.
pub fn analyze_sound_devices(text: &str) -> SoundDeviceReport {
    let words: Vec<&str> = text
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphabetic()))
        .filter(|w| !w.is_empty())
        .collect();

    if words.is_empty() {
        return SoundDeviceReport {
            alliteration_density: 0.0,
            dominant_alliterations: Vec::new(),
            dominant_assonances: Vec::new(),
        };
    }

    // Track onset / prominent consonants per word
    let mut consonant_hits: Vec<(char, Vec<usize>)> = Vec::new();
    let mut vowel_counts: Vec<(char, usize)> = Vec::new();
    let mut total_vowels = 0usize;

    for (w_idx, word) in words.iter().enumerate() {
        let mut seen_in_word = Vec::new();
        for ch in word.chars() {
            let lower = ch.to_lowercase().next().unwrap_or(ch);
            let cls = classify_unicode_char(lower);
            match cls {
                PhonemeClass::Vowel => {
                    total_vowels += 1;
                    // Normalize iotated vowels to their core acoustic vowel for assonance grouping
                    let canonical_v = match lower {
                        'я' => 'а',
                        'ё' | 'ю' => 'у',
                        'е' => 'э',
                        other => other,
                    };
                    if let Some(entry) = vowel_counts.iter_mut().find(|(v, _)| *v == canonical_v) {
                        entry.1 += 1;
                    } else {
                        vowel_counts.push((canonical_v, 1));
                    }
                }
                PhonemeClass::Sonorant | PhonemeClass::Fricative | PhonemeClass::Plosive => {
                    if !seen_in_word.contains(&lower) {
                        seen_in_word.push(lower);
                        if let Some(entry) = consonant_hits.iter_mut().find(|(c, _)| *c == lower) {
                            entry.1.push(w_idx);
                        } else {
                            consonant_hits.push((lower, vec![w_idx]));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Filter consonant chains where adjacent occurrences are within 3 words of each other
    let mut chains = Vec::new();
    let mut echo_links = 0usize;

    for (cons, indices) in consonant_hits {
        if indices.len() < 2 {
            continue;
        }
        let mut close_indices = Vec::new();
        for window in indices.windows(2) {
            if window[1] - window[0] <= 3 {
                echo_links += 1;
                if close_indices.last().copied() != Some(window[0]) {
                    close_indices.push(window[0]);
                }
                close_indices.push(window[1]);
            }
        }
        if close_indices.len() >= 2 {
            let intensity = close_indices.len() as f32 / words.len() as f32;
            chains.push(AlliterationChain {
                consonant: cons,
                word_indices: close_indices,
                intensity,
            });
        }
    }

    chains.sort_by(|a, b| {
        b.word_indices
            .len()
            .cmp(&a.word_indices.len())
            .then_with(|| a.consonant.cmp(&b.consonant))
    });
    chains.truncate(5);

    vowel_counts.sort_by(|a, b| b.1.cmp(&a.1));
    let dominant_assonances = vowel_counts
        .into_iter()
        .take(4)
        .map(|(vowel, occurrences)| AssonanceCluster {
            vowel,
            occurrences,
            share_of_vowels: occurrences as f32 / total_vowels.max(1) as f32,
        })
        .collect();

    SoundDeviceReport {
        alliteration_density: (echo_links as f32 / words.len().max(1) as f32).clamp(0.0, 1.0),
        dominant_alliterations: chains,
        dominant_assonances,
    }
}
