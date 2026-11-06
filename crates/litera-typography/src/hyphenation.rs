//! Frank Liang's TeX Hyphenation Algorithm (1983) and Slavic Phonological Syllable Hyphenator.
//!
//! Combines two complementary hyphenation engines used in professional book composition:
//! 1. **Liang Pattern Automaton**: Assigns inter-letter weights $w_0, w_1, \dots, w_k \in \{0..9\}$
//!    from embedded prefix/infix/suffix patterns (e.g., `.ти3ши`, `2ст1в`, `1н2н`). Odd weights
//!    ($w_i \bmod 2 == 1$) denote legal hyphenation points, while even weights suppress breaks.
//! 2. **Christoffersen / Academy of Sciences Russian Phonological Splitter**: Splits words at
//!    rising-sonority boundaries while honoring non-detachable single-letter margins (`lefthyphenmin = 2`,
//!    `righthyphenmin = 2`), double-consonant splits (`ис-кус-ство`, `мед-лен-но`), and `й`/`ь`/`ъ`
//!    attachment rules (never detach `й`, `ь`, `ъ` from the preceding syllable).

use litera_phonetics_shim::{classify_char_sonority, is_vowel_char};

/// Internal lightweight phonological helper module so `litera-typography` remains self-contained.
mod litera_phonetics_shim {
    #[inline]
    pub fn is_vowel_char(ch: char) -> bool {
        let lower = ch.to_lowercase().next().unwrap_or(ch);
        matches!(
            lower,
            'а' | 'е'
                | 'ё'
                | 'и'
                | 'о'
                | 'у'
                | 'ы'
                | 'э'
                | 'ю'
                | 'я'
                | 'a'
                | 'e'
                | 'i'
                | 'o'
                | 'u'
                | 'y'
        )
    }

    #[inline]
    pub fn is_modifier_sign(ch: char) -> bool {
        matches!(ch, 'ь' | 'ъ' | 'Ь' | 'Ъ' | 'й' | 'Й')
    }

    #[inline]
    pub fn classify_char_sonority(ch: char) -> u8 {
        let lower = ch.to_lowercase().next().unwrap_or(ch);
        if is_vowel_char(lower) {
            10
        } else if matches!(lower, 'р' | 'л' | 'м' | 'н' | 'й' | 'r' | 'l' | 'm' | 'n' | 'w') {
            7
        } else if matches!(
            lower,
            'в' | 'ф' | 'с' | 'з' | 'ш' | 'щ' | 'ж' | 'х' | 'v' | 'f' | 's' | 'z' | 'h'
        ) {
            4
        } else if lower.is_alphabetic() {
            2
        } else {
            0
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HyphenatedWord {
    pub original: String,
    /// Legal syllable fragments that can be joined by a soft hyphen (`\u{00AD}`) or discretionary `-`.
    pub fragments: Vec<String>,
    /// Character indices (in Unicode scalars) after which a line break with hyphen is valid.
    pub break_char_indices: Vec<usize>,
    /// Inter-character Liang weights $0..=9$ of length `chars.len() + 1`.
    pub liang_weights: Vec<u8>,
}

/// Embedded Liang hyphenation pattern rules for common Russian and English morphological sequences.
/// Format: `.` marks word boundary; digits `1..=5` set the minimum odd/even weight at that inter-letter slot.
pub const LIANG_PATTERNS_RU_EN: [&str; 64] = [
    ".а2", ".о2", ".у2", ".э2", ".и2", ".я2", ".ю2", ".е2",
    "2а.", "2о.", "2у.", "2э.", "2и.", "2я.", "2ю.", "2е.",
    "1б2л", "1б2р", "1в2л", "1в2р", "1г2л", "1г2р", "1д2р", "1к2л",
    "1к2р", "1п2л", "1п2р", "1с2т", "1с2к", "1с2л", "1с2м", "1с2н",
    "1с2п", "1т2р", "1ф2р", "1х2р", "2й1", "2ь1", "2ъ1", "н1н",
    "л1л", "м1м", "с1с", "т1т", "р1н", "л1н", "м1н", "р1к",
    "р1т", "л1к", "н1к", "н1т", "ст1в", "ств2о.", "ен1н", "ан1н",
    "ти1ши", "ар1хи", "тек1ту", "бу1ма", "за1сеч", "рав1но", "ве1си", "про1стран",
];

/// Parses a Liang pattern string (such as `"1с2т"` or `".ти1ши"`) into its pure letter sequence
/// and inter-character weight vector.
pub fn parse_liang_pattern(pattern: &str) -> (Vec<char>, Vec<u8>) {
    let mut letters = Vec::with_capacity(8);
    let mut weights = Vec::with_capacity(9);
    let mut pending_weight = 0u8;

    for ch in pattern.chars() {
        if let Some(digit) = ch.to_digit(10) {
            pending_weight = digit as u8;
        } else {
            weights.push(pending_weight);
            pending_weight = 0;
            letters.push(ch);
        }
    }
    weights.push(pending_weight);

    (letters, weights)
}

/// Hyphenates a single word using the combined Liang pattern matcher and Slavic phonological rules.
/// Enforces `left_hyphen_min = 2` and `right_hyphen_min = 2` so single orphan letters are never left behind.
pub fn hyphenate_word(word: &str) -> HyphenatedWord {
    let chars: Vec<char> = word.chars().collect();
    let n = chars.len();
    if n < 4 {
        return HyphenatedWord {
            original: String::from(word),
            fragments: vec![String::from(word)],
            break_char_indices: Vec::new(),
            liang_weights: vec![0; n + 1],
        };
    }

    // Build bordered lowercase character vector: ['.', c_0, c_1, ..., c_{n-1}, '.']
    let mut bordered = Vec::with_capacity(n + 2);
    bordered.push('.');
    for &ch in &chars {
        bordered.push(ch.to_lowercase().next().unwrap_or(ch));
    }
    bordered.push('.');

    // Weights between bordered characters: length = bordered.len() + 1 = n + 3
    let mut b_weights = vec![0u8; n + 3];

    // 1. Apply Liang patterns
    for &pat in &LIANG_PATTERNS_RU_EN {
        let (p_chars, p_weights) = parse_liang_pattern(pat);
        if p_chars.is_empty() || p_chars.len() > bordered.len() {
            continue;
        }
        let max_start = bordered.len() - p_chars.len();
        for start in 0..=max_start {
            if bordered[start..start + p_chars.len()] == p_chars[..] {
                for (offset, &w) in p_weights.iter().enumerate() {
                    if w > b_weights[start + offset] {
                        b_weights[start + offset] = w;
                    }
                }
            }
        }
    }

    // 2. Apply Slavic & General phonological syllable transition heuristics
    // In `chars`, slot `k` (1 <= k < n) represents the break between `chars[k - 1]` and `chars[k]`.
    // In `b_weights`, this corresponds to index `k + 1`.
    let mut vowel_positions = Vec::new();
    for (idx, &ch) in chars.iter().enumerate() {
        if is_vowel_char(ch) {
            vowel_positions.push(idx);
        }
    }

    if vowel_positions.len() >= 2 {
        for pair in vowel_positions.windows(2) {
            let v1 = pair[0];
            let v2 = pair[1];
            let span = v2 - v1;

            if span == 1 {
                // Two adjacent vowels (hiatus): break between them if not suppressed
                let slot = v1 + 1;
                if b_weights[slot + 1] == 0 {
                    b_weights[slot + 1] = 1;
                }
            } else if span == 2 {
                // V - C - V: open syllable rule puts break after V1 (before C), unless C is 'й'/'ь'/'ъ'
                let cons = chars[v1 + 1];
                let slot = if litera_phonetics_shim::is_modifier_sign(cons) {
                    v1 + 2
                } else {
                    v1 + 1
                };
                if slot < n && b_weights[slot + 1] == 0 {
                    b_weights[slot + 1] = 1;
                }
            } else {
                // V - C1 - C2 (...)- V: break after modifier sign, between identical consonants,
                // or after sonorant before obstruent
                let mut chosen_slot = v1 + 2;
                for k in (v1 + 1)..v2 {
                    if litera_phonetics_shim::is_modifier_sign(chars[k]) {
                        chosen_slot = k + 1;
                        break;
                    }
                    if k + 1 < v2 {
                        let s_left = classify_char_sonority(chars[k]);
                        let s_right = classify_char_sonority(chars[k + 1]);
                        if chars[k].eq_ignore_ascii_case(&chars[k + 1]) || s_left > s_right {
                            chosen_slot = k + 1;
                            break;
                        }
                    }
                }
                if chosen_slot < n && b_weights[chosen_slot + 1] == 0 {
                    b_weights[chosen_slot + 1] = 1;
                }
            }
        }
    }

    // Enforce left_hyphen_min = 2 and right_hyphen_min = 2, and never break before 'ь', 'ъ', 'й'
    let mut break_char_indices = Vec::new();
    let mut liang_weights = Vec::with_capacity(n + 1);
    for k in 0..=n {
        let mut w = b_weights[k + 1];
        if k < 2 || k > n - 2 {
            w = 0;
        } else if litera_phonetics_shim::is_modifier_sign(chars[k]) {
            w = 0;
        }
        liang_weights.push(w);
        if w % 2 == 1 {
            break_char_indices.push(k);
        }
    }

    let mut fragments = Vec::new();
    let mut prev = 0usize;
    for &brk in &break_char_indices {
        let frag: String = chars[prev..brk].iter().collect();
        fragments.push(frag);
        prev = brk;
    }
    let tail: String = chars[prev..].iter().collect();
    fragments.push(tail);

    HyphenatedWord {
        original: String::from(word),
        fragments,
        break_char_indices,
        liang_weights,
    }
}

/// Inserts visible hyphenation markers (`·` or `-`) into all words of a text for typographic preview.
pub fn hyphenate_text_preview(text: &str, separator: &str) -> String {
    let mut out = String::with_capacity(text.len() + text.len() / 3);
    for (idx, tok) in text.split_whitespace().enumerate() {
        if idx > 0 {
            out.push(' ');
        }
        let prefix_end = tok
            .char_indices()
            .find(|&(_, c)| c.is_alphabetic())
            .map(|(i, _)| i)
            .unwrap_or(tok.len());
        let suffix_start = tok
            .char_indices()
            .rev()
            .find(|&(_, c)| c.is_alphabetic())
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(prefix_end);

        out.push_str(&tok[..prefix_end]);
        if prefix_end < suffix_start {
            let core_word = &tok[prefix_end..suffix_start];
            let hyph = hyphenate_word(core_word);
            out.push_str(&hyph.fragments.join(separator));
        }
        out.push_str(&tok[suffix_start..]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_russian_hyphenation_rules() {
        let h1 = hyphenate_word("тишина");
        assert_eq!(h1.fragments, vec!["ти", "ши", "на"]);

        let h2 = hyphenate_word("медленно");
        assert!(h2.fragments.len() >= 2);

        // Modifier sign 'й' must never be detached onto the start of the next fragment
        let h3 = hyphenate_word("тайна");
        assert_eq!(h3.fragments, vec!["тай", "на"]);
    }
}
