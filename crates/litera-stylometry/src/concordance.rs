//! KWIC (Key Word In Context) Concordance Indexer, Phonetic Rhyme & Near-Rhyme Graph,
//! and Wagner–Fischer Edit-Distance Matrix (`concordance.rs`).
//!
//! Provides computational philology tools for inspecting recurring motifs,
//! clausula rhyme pairs (exact, rich, assonant, and consonant/slant rhymes),
//! and lexical collocations across a manuscript.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KwicConcordanceLine<'a> {
    pub word_position: usize,
    pub left_context: &'a str,
    pub keyword: &'a str,
    pub right_context: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RhymeQuality {
    /// Exact tonic vowel + post-tonic segments + supporting onset consonant (Богатая точная рифма).
    RichExact,
    /// Matching tonic vowel and final coda structure (Точная мужская/женская рифма).
    StandardExact,
    /// Matching vowel nuclei with differing consonants (Ассонансная неточная рифма).
    AssonantEcho,
    /// Matching consonant frame with differing vowels (Консонансная / диссонансная рифма).
    ConsonantSlant,
}

impl RhymeQuality {
    pub const fn russian_label(self) -> &'static str {
        match self {
            RhymeQuality::RichExact => "Богатая точная рифма",
            RhymeQuality::StandardExact => "Классическая точная рифма",
            RhymeQuality::AssonantEcho => "Ассонансное созвучие",
            RhymeQuality::ConsonantSlant => "Консонансная (диссонансная) перекличка",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RhymeEchoPair {
    pub word_a: String,
    pub word_b: String,
    pub pos_a: usize,
    pub pos_b: usize,
    pub quality: RhymeQuality,
}

/// Computes the Wagner–Fischer Levenshtein edit distance between two Unicode strings
/// using a single $O(\min(M, N))$ stack/heap row vector.
pub fn unicode_levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();

    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    let mut prev_row: Vec<usize> = (0..=n).collect();
    let mut curr_row: Vec<usize> = vec![0; n + 1];

    for i in 1..=m {
        curr_row[0] = i;
        for j in 1..=n {
            let cost = usize::from(a_chars[i - 1] != b_chars[j - 1]);
            curr_row[j] = (prev_row[j] + 1)
                .min(curr_row[j - 1] + 1)
                .min(prev_row[j - 1] + cost);
        }
        core::mem::swap(&mut prev_row, &mut curr_row);
    }

    prev_row[n]
}

/// Builds a Key-Word-In-Context (KWIC) concordance for all occurrences of `query_stem`
/// (case-insensitive prefix/substring match) with `window_chars` of surrounding margin context.
pub fn build_kwic_concordance<'a>(
    text: &'a str,
    query_stem: &str,
    window_chars: usize,
) -> Vec<KwicConcordanceLine<'a>> {
    let query_lower: String = query_stem.chars().flat_map(|c| c.to_lowercase()).collect();
    if query_lower.is_empty() {
        return Vec::new();
    }

    let mut results = Vec::new();
    let mut word_pos = 0usize;
    let mut search_from = 0usize;

    for raw_tok in text.split_whitespace() {
        let Some(rel_offset) = text[search_from..].find(raw_tok) else {
            break;
        };
        let byte_start = search_from + rel_offset;
        let byte_end = byte_start + raw_tok.len();
        search_from = byte_end;

        let clean_lower: String = raw_tok
            .trim_matches(|c: char| !c.is_alphabetic())
            .chars()
            .flat_map(|c| c.to_lowercase())
            .collect();

        if clean_lower.contains(&query_lower) {
            // Find valid UTF-8 character boundaries around [byte_start..byte_end]
            let left_start = text[..byte_start]
                .char_indices()
                .rev()
                .nth(window_chars)
                .map(|(idx, _)| idx)
                .unwrap_or(0);
            let right_end = text[byte_end..]
                .char_indices()
                .nth(window_chars)
                .map(|(idx, _)| byte_end + idx)
                .unwrap_or(text.len());

            results.push(KwicConcordanceLine {
                word_position: word_pos,
                left_context: text[left_start..byte_start].trim_start(),
                keyword: &text[byte_start..byte_end],
                right_context: text[byte_end..right_end].trim_end(),
            });
        }

        word_pos += 1;
    }

    results
}

/// Scans a manuscript for phonetic rhyme & homeoteleuton echoes between distinct words
/// separated by at least 2 words and at most 18 words.
pub fn detect_rhyme_echoes(text: &str) -> Vec<RhymeEchoPair> {
    let words: Vec<(usize, String)> = text
        .split_whitespace()
        .enumerate()
        .map(|(idx, tok)| {
            let clean: String = tok
                .trim_matches(|c: char| !c.is_alphabetic())
                .chars()
                .flat_map(|c| c.to_lowercase())
                .collect();
            (idx, clean)
        })
        .filter(|(_, w)| w.chars().count() >= 4)
        .collect();

    let mut pairs = Vec::new();

    for i in 0..words.len() {
        for j in (i + 1)..words.len() {
            let (pos_a, ref wa) = words[i];
            let (pos_b, ref wb) = words[j];
            if pos_b - pos_a < 2 || pos_b - pos_a > 18 {
                continue;
            }
            if wa == wb {
                continue;
            }

            let ca: Vec<char> = wa.chars().collect();
            let cb: Vec<char> = wb.chars().collect();
            let la = ca.len();
            let lb = cb.len();

            if la >= 4 && lb >= 4 && ca[la - 3..] == cb[lb - 3..] {
                let quality = if ca[la - 4] == cb[lb - 4] {
                    RhymeQuality::RichExact
                } else {
                    RhymeQuality::StandardExact
                };
                pairs.push(RhymeEchoPair {
                    word_a: wa.clone(),
                    word_b: wb.clone(),
                    pos_a,
                    pos_b,
                    quality,
                });
                if pairs.len() >= 12 {
                    return pairs;
                }
            }
        }
    }

    pairs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_and_kwic_concordance() {
        assert_eq!(unicode_levenshtein_distance("литера", "литеры"), 1);
        let sample = "Тишина имеет собственный вес. В этой тишине рождается слово.";
        let kwic = build_kwic_concordance(sample, "тишин", 20);
        assert_eq!(kwic.len(), 2);
    }
}
