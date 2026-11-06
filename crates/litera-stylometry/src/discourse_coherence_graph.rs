//! Discourse Coherence & Lexical Cohesion Graph Analyzer (`discourse_coherence_graph.rs`).
//!
//! Implements the computational discourse models of Halliday & Hasan (*Cohesion in English*, 1976),
//! Marti Hearst (*TextTiling*, 1997), and Barzilay & Lapata (*Entity-Based Local Coherence*, 2008):
//! - **Stemmed Content-Word Extraction**: Strips Russian and English inflectional suffixes to extract
//!   invariant morphological stems ($\ge 4$ characters).
//! - **Adjacent & Long-Range Cosine Cohesion**: Measures lexical stem overlap between consecutive
//!   sentences ($S_i, S_{i+1}$) and across distant thematic echoes ($S_i, S_{i+k}$).
//! - **Thematic Shift & Paragraph Boundary Detection**: Identifies valleys in local cosine similarity
//!   where a new paragraph or strophe break is structurally warranted.

/// Lexical cohesion link between two sentences $(i, j)$ in a manuscript.
#[derive(Debug, Clone, PartialEq)]
pub struct SentenceCohesionEdge {
    pub sentence_a: usize,
    pub sentence_b: usize,
    pub distance_sentences: usize,
    pub shared_stems: Vec<String>,
    pub jaccard_similarity: f32,
}

/// Discourse coherence and thematic flow profile for a multi-sentence manuscript.
#[derive(Debug, Clone, PartialEq)]
pub struct DiscourseCoherenceReport {
    pub sentence_count: usize,
    /// Mean lexical similarity between adjacent sentences $S_i$ and $S_{i+1}$.
    pub local_adjacent_cohesion: f32,
    /// Mean lexical similarity between non-adjacent sentences ($|i - j| \ge 2$), measuring thematic return/ring composition.
    pub global_ring_cohesion: f32,
    /// True if the last sentence shares thematic stems with the first sentence (*Кольцевая композиция*).
    pub has_ring_composition_closure: bool,
    /// Sentence indices where a sharp drop in local cohesion suggests a natural paragraph or section break.
    pub recommended_paragraph_breaks: Vec<usize>,
    /// Strongest lexical cohesion edges across the manuscript.
    pub cohesion_edges: Vec<SentenceCohesionEdge>,
}

/// Lightweight Russian & English inflectional stemmer (Porter/Snowball-inspired suffix stripper)
/// operating without external dictionary allocations.
pub fn extract_morphological_stem(word: &str) -> String {
    let clean: String = word
        .chars()
        .filter(|c| c.is_alphabetic())
        .flat_map(|c| c.to_lowercase())
        .collect();

    let char_count = clean.chars().count();
    if char_count <= 3 {
        return clean;
    }

    // Common Russian nominal, adjectival, and verbal inflectional endings ordered longest-first
    let ru_endings = [
        "остями", "ениями", "аниями", "ыми", "ими", "ого", "его", "ому", "ему", "ами", "ями",
        "ость", "ости", "ение", "ание", "ать", "ять", "ить", "еть", "уть", "ться", "лся",
        "лась", "лись", "ый", "ий", "ой", "ая", "яя", "ое", "ее", "ые", "ие", "ов", "ев",
        "ей", "ам", "ям", "ах", "ях", "ом", "ем", "ой", "ей", "ую", "юю", "а", "я", "о",
        "е", "и", "ы", "у", "ю",
    ];

    for ending in &ru_endings {
        if clean.ends_with(ending) {
            let stem_bytes = clean.len() - ending.len();
            let stem = &clean[..stem_bytes];
            if stem.chars().count() >= 3 {
                return stem.to_string();
            }
        }
    }

    // English inflectional suffixes
    for en_end in &["ingly", "edly", "ness", "ment", "tion", "sion", "ing", "ed", "es", "s"] {
        if clean.ends_with(en_end) {
            let stem_bytes = clean.len() - en_end.len();
            let stem = &clean[..stem_bytes];
            if stem.chars().count() >= 3 {
                return stem.to_string();
            }
        }
    }

    clean
}

/// Checks if a stemmed token is a high-frequency stopword (preposition, conjunction, particle, pronoun).
fn is_discourse_stopword(stem: &str) -> bool {
    matches!(
        stem,
        "это" | "эт" | "как" | "так" | "что" | "чт" | "или" | "при" | "для" | "без" | "над"
            | "под" | "про" | "все" | "вс" | "они" | "она" | "оно" | "мой" | "твой" | "свой"
            | "наш" | "ваш" | "был" | "быт" | "ест" | "же" | "ли" | "бы" | "не" | "ни"
            | "the" | "and" | "for" | "that" | "with" | "from" | "this" | "are" | "was"
    )
}

/// Extracts unique content-word morphological stems from a single sentence.
pub fn extract_sentence_content_stems(sentence: &str) -> Vec<String> {
    let mut stems: Vec<String> = Vec::new();
    for raw_token in sentence.split_whitespace() {
        let stem = extract_morphological_stem(raw_token);
        if stem.chars().count() >= 3 && !is_discourse_stopword(&stem) {
            if !stems.contains(&stem) {
                stems.push(stem);
            }
        }
    }
    stems
}

/// Computes fuzzy stem similarity between two stems (exact match or shared 4-character root prefix).
#[inline]
fn stems_match(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    let mut ca = a.chars();
    let mut cb = b.chars();
    let mut matched = 0usize;
    while let (Some(x), Some(y)) = (ca.next(), cb.next()) {
        if x == y {
            matched += 1;
        } else {
            break;
        }
    }
    matched >= 4
}

/// Analyzes the discourse cohesion graph across a sequence of sentences.
pub fn analyze_discourse_coherence(text: &str) -> DiscourseCoherenceReport {
    let sentences: Vec<&str> = text
        .split(|c| matches!(c, '.' | '!' | '?' | '…'))
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    let n = sentences.len();
    if n == 0 {
        return DiscourseCoherenceReport {
            sentence_count: 0,
            local_adjacent_cohesion: 0.0,
            global_ring_cohesion: 0.0,
            has_ring_composition_closure: false,
            recommended_paragraph_breaks: Vec::new(),
            cohesion_edges: Vec::new(),
        };
    }

    let sentence_stems: Vec<Vec<String>> = sentences
        .iter()
        .map(|s| extract_sentence_content_stems(s))
        .collect();

    let mut edges: Vec<SentenceCohesionEdge> = Vec::new();
    let mut adjacent_sum = 0.0f32;
    let mut adjacent_pairs = 0usize;
    let mut distant_sum = 0.0f32;
    let mut distant_pairs = 0usize;
    let mut adjacent_scores: Vec<f32> = Vec::with_capacity(n.saturating_sub(1));

    for i in 0..n {
        for j in (i + 1)..n {
            let stems_a = &sentence_stems[i];
            let stems_b = &sentence_stems[j];
            if stems_a.is_empty() || stems_b.is_empty() {
                if j == i + 1 {
                    adjacent_scores.push(0.0);
                }
                continue;
            }

            let mut shared: Vec<String> = Vec::new();
            for sa in stems_a {
                if stems_b.iter().any(|sb| stems_match(sa, sb)) && !shared.contains(sa) {
                    shared.push(sa.clone());
                }
            }

            let union_len = (stems_a.len() + stems_b.len()).saturating_sub(shared.len()).max(1);
            let jaccard = shared.len() as f32 / union_len as f32;

            if j == i + 1 {
                adjacent_sum += jaccard;
                adjacent_pairs += 1;
                adjacent_scores.push(jaccard);
            } else {
                distant_sum += jaccard;
                distant_pairs += 1;
            }

            if !shared.is_empty() {
                edges.push(SentenceCohesionEdge {
                    sentence_a: i,
                    sentence_b: j,
                    distance_sentences: j - i,
                    shared_stems: shared,
                    jaccard_similarity: jaccard,
                });
            }
        }
    }

    // Detect TextTiling valleys: if adjacent_scores[i] is significantly below neighbors, recommend break before sentence i+1
    let mut recommended_breaks = Vec::new();
    for i in 1..adjacent_scores.len().saturating_sub(1) {
        let prev = adjacent_scores[i - 1];
        let curr = adjacent_scores[i];
        let next = adjacent_scores[i + 1];
        if curr < 0.05 && (prev > 0.12 || next > 0.12) {
            recommended_breaks.push(i + 1);
        }
    }

    let has_ring_closure = if n >= 3 {
        edges
            .iter()
            .any(|e| e.sentence_a == 0 && e.sentence_b == n - 1 && !e.shared_stems.is_empty())
    } else {
        false
    };

    edges.sort_by(|a, b| {
        b.jaccard_similarity
            .partial_cmp(&a.jaccard_similarity)
            .unwrap_or(core::cmp::Ordering::Equal)
    });
    edges.truncate(12);

    DiscourseCoherenceReport {
        sentence_count: n,
        local_adjacent_cohesion: if adjacent_pairs > 0 {
            adjacent_sum / adjacent_pairs as f32
        } else {
            0.0
        },
        global_ring_cohesion: if distant_pairs > 0 {
            distant_sum / distant_pairs as f32
        } else {
            0.0
        },
        has_ring_composition_closure: has_ring_closure,
        recommended_paragraph_breaks: recommended_breaks,
        cohesion_edges: edges,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_composition_and_stem_cohesion() {
        let text = "Тишина старого дома хранит свет. В комнатах дома пахнет старым деревом. Вечерний свет снова уходит в тишину.";
        let rep = analyze_discourse_coherence(text);
        assert_eq!(rep.sentence_count, 3);
        assert!(rep.has_ring_composition_closure);
        assert!(!rep.cohesion_edges.is_empty());
    }
}
