//! Russian & Bilingual International Phonetic Alphabet (IPA) Broad/Narrow Transcriber (`ipa_transcriber.rs`).
//!
//! Implements classical Moscow/Petersburg phonological rules (Ruben Avanesov, Morris Halle,
//! and Lawrence Jones) to transcribe Cyrillic orthographic words into narrow IPA `[...]`:
//! 1. **Palatalization (Смягчение согласных)**: Consonants before `е, ё, и, ю, я, ь` acquire
//!    the palatal secondary articulation mark `[ʲ]` (except inherently hard `ж, ш, ц` and
//!    inherently soft `ч [tɕ], щ [ɕː], й [j]`).
//! 2. **Final Obstruent Devoicing (Оглушение на конце слова)**: Word-final voiced obstruents
//!    `б, в, г, д, ж, з` devoice to `[p, f, k, t, ʂ, s]`.
//! 3. **Regressive Voicing Assimilation (Регрессивная ассимиляция по глухости/звонкости)**:
//!    Obstruent clusters assimilate to the voicing of the rightmost obstruent (excluding `в` as trigger).
//! 4. **Vowel Reduction (Аканье и Иканье)**: Unstressed `о`/`а` reduce to `[ɐ]` (first pretonic)
//!    or `[ə]` (other unstressed positions); unstressed `е`/`я` after soft consonants reduce to `[ɪ]`.

use crate::prosody::estimate_word_stress;
use crate::{classify_unicode_char, PhonemeClass};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IpaWordTranscription {
    pub orthographic: String,
    pub ipa_narrow: String,
    pub stressed_syllable_1based: usize,
    pub total_syllables: usize,
    pub applied_rules: Vec<&'static str>,
}

/// Checks if a Cyrillic consonant is inherently unpaired hard (`ж`, `ш`, `ц`).
#[inline]
fn is_inherently_hard(ch: char) -> bool {
    matches!(ch, 'ж' | 'ш' | 'ц')
}

/// Checks if a Cyrillic character is an obstruent that participates in voicing assimilation.
#[inline]
fn is_obstruent(ch: char) -> bool {
    matches!(
        ch,
        'б' | 'п'
            | 'в'
            | 'ф'
            | 'г'
            | 'к'
            | 'д'
            | 'т'
            | 'ж'
            | 'ш'
            | 'з'
            | 'с'
            | 'х'
            | 'ц'
            | 'ч'
            | 'щ'
    )
}

#[inline]
fn is_voiced_obstruent(ch: char) -> bool {
    matches!(ch, 'б' | 'в' | 'г' | 'д' | 'ж' | 'з')
}

#[inline]
fn is_voiceless_obstruent(ch: char) -> bool {
    matches!(ch, 'п' | 'ф' | 'к' | 'т' | 'ш' | 'с' | 'х' | 'ц' | 'ч' | 'щ')
}

#[inline]
fn devoice_cyrillic(ch: char) -> char {
    match ch {
        'б' => 'п',
        'в' => 'ф',
        'г' => 'к',
        'д' => 'т',
        'ж' => 'ш',
        'з' => 'с',
        other => other,
    }
}

#[inline]
fn voice_cyrillic(ch: char) -> char {
    match ch {
        'п' => 'б',
        'ф' => 'в',
        'к' => 'г',
        'т' => 'д',
        'ш' => 'ж',
        'с' => 'з',
        other => other,
    }
}

/// Transcribes a single Russian or Latin word into phonetic IPA notation `[...]`.
pub fn transcribe_word_to_ipa(word: &str) -> IpaWordTranscription {
    let clean: Vec<char> = word
        .chars()
        .filter(|c| c.is_alphabetic())
        .flat_map(|c| c.to_lowercase())
        .collect();

    if clean.is_empty() {
        return IpaWordTranscription {
            orthographic: String::from(word),
            ipa_narrow: String::from("[]"),
            stressed_syllable_1based: 0,
            total_syllables: 0,
            applied_rules: Vec::new(),
        };
    }

    let has_cyrillic = clean.iter().any(|&c| ('\u{0400}'..='\u{04FF}').contains(&c));
    let (stress_idx_0, total_syl, _) = estimate_word_stress(word);

    if !has_cyrillic {
        // Basic broad Latin IPA mapping
        let mut out = String::from("[");
        for (idx, &ch) in clean.iter().enumerate() {
            if idx == 0 && total_syl >= 1 {
                out.push('ˈ');
            }
            match ch {
                'a' => out.push('ɑ'),
                'e' => out.push('ɛ'),
                'i' | 'y' => out.push('ɪ'),
                'o' => out.push('ɔ'),
                'u' => out.push('ʊ'),
                'r' => out.push('ɹ'),
                other => out.push(other),
            }
        }
        out.push(']');
        return IpaWordTranscription {
            orthographic: String::from(word),
            ipa_narrow: out,
            stressed_syllable_1based: stress_idx_0 + 1,
            total_syllables: total_syl,
            applied_rules: vec!["Латинская широкая транскрипция"],
        };
    }

    let mut rules = Vec::new();
    let mut working = clean.clone();
    let n = working.len();

    // Rule 1: Word-final obstruent devoicing (allowing trailing 'ь'/'ъ')
    let last_cons_idx = if n >= 2 && matches!(working[n - 1], 'ь' | 'ъ') {
        Some(n - 2)
    } else if n >= 1 {
        Some(n - 1)
    } else {
        None
    };

    if let Some(idx) = last_cons_idx {
        if is_voiced_obstruent(working[idx]) {
            working[idx] = devoice_cyrillic(working[idx]);
            rules.push("Оглушение конечного звонкого шумного");
        }
    }

    // Rule 2: Regressive voicing assimilation right-to-left across obstruent clusters
    for i in (0..n.saturating_sub(1)).rev() {
        let right = if working[i + 1] == 'ь' && i + 2 < n {
            working[i + 2]
        } else {
            working[i + 1]
        };
        if is_obstruent(working[i]) && is_obstruent(right) {
            if is_voiceless_obstruent(right) && is_voiced_obstruent(working[i]) {
                working[i] = devoice_cyrillic(working[i]);
                if !rules.contains(&"Регрессивное оглушение в кластере") {
                    rules.push("Регрессивное оглушение в кластере");
                }
            } else if is_voiced_obstruent(right)
                && right != 'в'
                && is_voiceless_obstruent(working[i])
            {
                working[i] = voice_cyrillic(working[i]);
                if !rules.contains(&"Регрессивное озвончение в кластере") {
                    rules.push("Регрессивное озвончение в кластере");
                }
            }
        }
    }

    // Build IPA output with stress mark, palatalization, and vowel reduction (akan'ye / ikan'ye)
    let mut ipa = String::with_capacity(n * 3 + 4);
    ipa.push('[');

    let mut current_vowel_idx = 0usize;
    for i in 0..n {
        let ch = working[i];
        let next_ch = working.get(i + 1).copied();
        let prev_ch = if i > 0 { Some(working[i - 1]) } else { None };

        if matches!(ch, 'ь' | 'ъ') {
            continue;
        }

        let is_v = classify_unicode_char(ch) == PhonemeClass::Vowel;
        if is_v {
            let is_stressed = current_vowel_idx == stress_idx_0;
            let is_first_pretonic =
                stress_idx_0 > 0 && current_vowel_idx + 1 == stress_idx_0;

            if is_stressed && total_syl > 1 {
                ipa.push('ˈ');
            }

            // Determine if iotated vowel ('я', 'е', 'ё', 'ю') is in word-initial, post-vowel, or post-sign position
            let needs_j_glide = matches!(ch, 'я' | 'е' | 'ё' | 'ю')
                && match prev_ch {
                    None => true,
                    Some(p) => {
                        classify_unicode_char(p) == PhonemeClass::Vowel || matches!(p, 'ь' | 'ъ')
                    }
                };

            if needs_j_glide {
                ipa.push('j');
            }

            let prev_is_hard_sibilant = prev_ch.map(is_inherently_hard).unwrap_or(false);

            match ch {
                'а' => {
                    if is_stressed {
                        ipa.push('a');
                    } else if is_first_pretonic {
                        ipa.push('ɐ');
                        if !rules.contains(&"Аканье (редукция безударных гласных)") {
                            rules.push("Аканье (редукция безударных гласных)");
                        }
                    } else {
                        ipa.push('ə');
                    }
                }
                'о' => {
                    if is_stressed {
                        ipa.push('o');
                    } else if is_first_pretonic {
                        ipa.push('ɐ');
                        if !rules.contains(&"Аканье (редукция безударных гласных)") {
                            rules.push("Аканье (редукция безударных гласных)");
                        }
                    } else {
                        ipa.push('ə');
                    }
                }
                'я' => {
                    if is_stressed {
                        ipa.push_str("æ");
                    } else {
                        ipa.push('ɪ');
                        if !rules.contains(&"Иканье (редукция после мягких)") {
                            rules.push("Иканье (редукция после мягких)");
                        }
                    }
                }
                'е' => {
                    if is_stressed {
                        ipa.push('e');
                    } else if prev_is_hard_sibilant {
                        ipa.push('ɨ');
                    } else {
                        ipa.push('ɪ');
                        if !rules.contains(&"Иканье (редукция после мягких)") {
                            rules.push("Иканье (редукция после мягких)");
                        }
                    }
                }
                'ё' => ipa.push('ɵ'),
                'и' => {
                    if prev_is_hard_sibilant {
                        ipa.push('ɨ');
                    } else {
                        ipa.push('i');
                    }
                }
                'ы' => ipa.push('ɨ'),
                'э' => ipa.push('ɛ'),
                'у' => ipa.push('u'),
                'ю' => {
                    if is_stressed {
                        ipa.push('ʉ');
                    } else {
                        ipa.push('ʊ');
                    }
                }
                _ => ipa.push(ch),
            }

            current_vowel_idx += 1;
        } else {
            // Consonant transcription + palatalization check
            let softened_by_next = matches!(
                next_ch,
                Some('е' | 'ё' | 'и' | 'ю' | 'я' | 'ь')
            ) && !is_inherently_hard(ch);

            match ch {
                'ж' => ipa.push('ʐ'),
                'ш' => ipa.push('ʂ'),
                'щ' => ipa.push_str("ɕː"),
                'ч' => ipa.push_str("tɕ"),
                'ц' => ipa.push_str("ts"),
                'й' => ipa.push('j'),
                'х' => ipa.push('x'),
                'г' => ipa.push('ɡ'),
                'л' => {
                    if softened_by_next {
                        ipa.push_str("lʲ");
                    } else {
                        ipa.push('ɫ');
                    }
                }
                'р' => {
                    ipa.push('r');
                    if softened_by_next {
                        ipa.push('ʲ');
                    }
                }
                _ => {
                    let mapped = match ch {
                        'б' => 'b',
                        'п' => 'p',
                        'в' => 'v',
                        'ф' => 'f',
                        'д' => 'd',
                        'т' => 't',
                        'з' => 'z',
                        'с' => 's',
                        'к' => 'k',
                        'м' => 'm',
                        'н' => 'n',
                        other => other,
                    };
                    ipa.push(mapped);
                    if softened_by_next {
                        ipa.push('ʲ');
                        if !rules.contains(&"Палатализация согласного [ʲ]") {
                            rules.push("Палатализация согласного [ʲ]");
                        }
                    }
                }
            }
        }
    }

    ipa.push(']');

    IpaWordTranscription {
        orthographic: String::from(word),
        ipa_narrow: ipa,
        stressed_syllable_1based: stress_idx_0 + 1,
        total_syllables: total_syl,
        applied_rules: rules,
    }
}

/// Transcribes an entire clause or sentence into a space-separated narrow IPA stream.
pub fn transcribe_clause_to_ipa(text: &str) -> String {
    text.split_whitespace()
        .map(transcribe_word_to_ipa)
        .map(|t| t.ipa_narrow)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_russian_ipa_palatalization_and_devoicing() {
        let t1 = transcribe_word_to_ipa("тишина");
        assert!(t1.ipa_narrow.contains("tʲ"));
        assert!(t1.ipa_narrow.contains('ʂ'));

        let t2 = transcribe_word_to_ipa("сад");
        // Final 'д' must devoice to 't'
        assert_eq!(t2.ipa_narrow, "[sat]");
    }
}
