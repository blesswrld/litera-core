//! Clements (1990) & Jespersen Sonority Dispersion Principle (SDP) Finite-State Syllabifier (`syllable_sonority_fsm.rs`).
//!
//! Segments words into explicit structural syllables (`Onset`, `Nucleus`, `Coda`) using
//! the Maximal Onset Principle (MOP) constrained by the Sonority Sequencing Principle (SSP)
//! and Russian phonotactic cluster legality rules.
//!
//! Computes G. N. Clements's **Sonority Dispersion Coefficient** $D$:
//! $$D = \frac{1}{m} \sum_{i=1}^{m} \frac{1}{(\Delta s_i)^2}$$
//! where $\Delta s_i = s(\text{Nucleus}) - s(\text{Margin})$. Core CV syllables with a steep
//! plosive-to-vowel rise (`та`, `па`, `ка`) have minimal dispersion $D$ (maximally unmarked),
//! whereas complex consonant clusters (`взгляд`, `всплеск`, `страсть`) raise acoustic complexity.

use crate::{inspect_char_acoustics, PhonemeClass};

/// Structural constituents of a single phonological syllable $\sigma = (\text{Onset}, \text{Nucleus}, \text{Coda})$.
#[derive(Debug, Clone, PartialEq)]
pub struct StructuralSyllable {
    pub index: usize,
    pub onset: String,
    pub nucleus: String,
    pub coda: String,
    /// Open syllable (`true` if `coda` is empty, characteristic of Russian vocalic tendency).
    pub is_open: bool,
    /// Rising sonority slope from onset minimum to vowel nucleus ($\Delta s_{\text{onset}}$).
    pub onset_sonority_rise: i16,
    /// Falling sonority slope from vowel nucleus to coda minimum ($\Delta s_{\text{coda}}$).
    pub coda_sonority_fall: i16,
    /// Clements Sonority Dispersion metric for the demisyllable ($D \in [0.01, 1.00]$).
    pub clements_dispersion: f32,
    /// True if the onset or coda violates strict monotonic Sonority Sequencing (e.g. Russian `ртуть`, `мгла`, `взгляд`).
    pub has_sonority_plateau_or_reversal: bool,
}

/// Complete structural syllabification report for a word or prosodic unit.
#[derive(Debug, Clone, PartialEq)]
pub struct WordSyllableTree {
    pub original_word: String,
    pub syllables: Vec<StructuralSyllable>,
    /// Proportion of syllables that are open ($\text{CV}$, lacking a coda), reflecting Slavic Law of Open Syllables.
    pub open_syllable_ratio: f32,
    /// Mean Clements Sonority Dispersion across all demisyllables.
    pub mean_clements_dispersion: f32,
    /// Count of initial/medial sonority reversals (sonorant-plosive onsets like `мгл-`, `лг-`, `рт-`).
    pub sonority_reversal_count: usize,
}

/// Checks if a two-consonant cluster is a legal Russian/Slavic syllable onset under the
/// Maximal Onset + Sonority Sequencing Principle.
#[inline]
pub fn is_valid_russian_onset_cluster(c1: char, c2: char) -> bool {
    let d1 = inspect_char_acoustics(c1);
    let d2 = inspect_char_acoustics(c2);
    if d1.class == PhonemeClass::Modifier || d2.class == PhonemeClass::Modifier {
        return false;
    }
    // Standard rising sonority onset: Plosive/Fricative + Sonorant (пр-, тр-, кл-, сн-,вл-)
    if d2.sonority_micro > d1.sonority_micro {
        return true;
    }
    // Sibilant/labiodental fricative + Plosive (`ст-`, `сп-`, `ск-`, `зд-`, `вз-`, `вс-`, `шп-`, `шт-`)
    let l1 = c1.to_lowercase().next().unwrap_or(c1);
    let l2 = c2.to_lowercase().next().unwrap_or(c2);
    matches!(
        (l1, l2),
        ('с', 'т')
            | ('с', 'п')
            | ('с', 'к')
            | ('с', 'х')
            | ('з', 'д')
            | ('з', 'г')
            | ('з', 'б')
            | ('в', 'з')
            | ('в', 'с')
            | ('в', 'т')
            | ('в', 'к')
            | ('ш', 'т')
            | ('ш', 'п')
            | ('ш', 'к')
    )
}

/// Segments a single word into `StructuralSyllable` trees using Clements's Sonority Dispersion
/// and Russian/Slavic open-syllable preference.
pub fn parse_word_syllable_tree(word: &str) -> WordSyllableTree {
    let clean: Vec<char> = word
        .chars()
        .filter(|c| inspect_char_acoustics(*c).class != PhonemeClass::NonPhonetic)
        .collect();

    if clean.is_empty() {
        return WordSyllableTree {
            original_word: word.to_string(),
            syllables: Vec::new(),
            open_syllable_ratio: 0.0,
            mean_clements_dispersion: 0.0,
            sonority_reversal_count: 0,
        };
    }

    // Locate all vocalic nuclei indices
    let has_cyrillic = clean
        .iter()
        .any(|c| ('\u{0400}'..='\u{04FF}').contains(c));
    let mut nuclei_ranges: Vec<(usize, usize)> = Vec::new();
    let mut i = 0usize;
    while i < clean.len() {
        if inspect_char_acoustics(clean[i]).class == PhonemeClass::Vowel {
            let start = i;
            let mut end = i + 1;
            // In Latin/English, coalesce adjacent vowels into a single diphthong nucleus
            if !has_cyrillic {
                while end < clean.len()
                    && inspect_char_acoustics(clean[end]).class == PhonemeClass::Vowel
                {
                    end += 1;
                }
            }
            nuclei_ranges.push((start, end));
            i = end;
        } else {
            i += 1;
        }
    }

    // If a consonant-only word (e.g. preposition `в`, `с`, `к`), treat whole cluster as degenerate syllable
    if nuclei_ranges.is_empty() {
        let onset: String = clean.iter().collect();
        return WordSyllableTree {
            original_word: word.to_string(),
            syllables: vec![StructuralSyllable {
                index: 0,
                onset,
                nucleus: String::new(),
                coda: String::new(),
                is_open: true,
                onset_sonority_rise: 0,
                coda_sonority_fall: 0,
                clements_dispersion: 1.0,
                has_sonority_plateau_or_reversal: false,
            }],
            open_syllable_ratio: 1.0,
            mean_clements_dispersion: 1.0,
            sonority_reversal_count: 0,
        };
    }

    let num_syl = nuclei_ranges.len();
    let mut onset_bounds: Vec<(usize, usize)> = vec![(0, 0); num_syl];
    let mut coda_bounds: Vec<(usize, usize)> = vec![(0, 0); num_syl];

    // Word-initial consonants belong to the first syllable's onset
    onset_bounds[0] = (0, nuclei_ranges[0].0);
    // Word-final consonants belong to the last syllable's coda
    coda_bounds[num_syl - 1] = (nuclei_ranges[num_syl - 1].1, clean.len());

    // Partition intervocalic consonant clusters between coda[k] and onset[k+1]
    for k in 0..(num_syl - 1) {
        let cluster_start = nuclei_ranges[k].1;
        let cluster_end = nuclei_ranges[k + 1].0;
        let cluster_len = cluster_end.saturating_sub(cluster_start);

        if cluster_len == 0 {
            coda_bounds[k] = (cluster_start, cluster_start);
            onset_bounds[k + 1] = (cluster_end, cluster_end);
        } else if cluster_len == 1 {
            // Single intervocalic consonant: unless it is `й`, `ъ`, or `ь`, Maximal Onset attaches it to onset[k+1]
            let ch = clean[cluster_start];
            let lower = ch.to_lowercase().next().unwrap_or(ch);
            if matches!(lower, 'й' | 'ъ' | 'ь') {
                coda_bounds[k] = (cluster_start, cluster_end);
                onset_bounds[k + 1] = (cluster_end, cluster_end);
            } else {
                coda_bounds[k] = (cluster_start, cluster_start);
                onset_bounds[k + 1] = (cluster_start, cluster_end);
            }
        } else {
            // Multi-consonant cluster: check if first consonant is a sonorant (`р`, `л`, `м`, `н`, `й`) or modifier (`ь`, `ъ`)
            // In Russian (Avanesov / Bondarko theory), sonorants following a vowel close the preceding syllable (`вар-ка`, `пол-ка`)
            // whereas obstruent+sonorant (`о-кно`, `ве-сна`, `зе-мля`) open the preceding syllable!
            let mut split_idx = cluster_end - 1;
            if split_idx > cluster_start {
                let c_prev = clean[split_idx - 1];
                let c_last = clean[split_idx];
                if is_valid_russian_onset_cluster(c_prev, c_last) {
                    split_idx -= 1;
                }
            }
            // Ensure `ь`/`ъ`/`й` always remain in the coda of syllable `k`
            while split_idx < cluster_end {
                let ch = clean[split_idx].to_lowercase().next().unwrap_or(clean[split_idx]);
                if matches!(ch, 'ь' | 'ъ' | 'й') {
                    split_idx += 1;
                } else {
                    break;
                }
            }

            coda_bounds[k] = (cluster_start, split_idx);
            onset_bounds[k + 1] = (split_idx, cluster_end);
        }
    }

    let mut syllables = Vec::with_capacity(num_syl);
    let mut open_count = 0usize;
    let mut dispersion_sum = 0.0f32;
    let mut reversal_count = 0usize;

    for k in 0..num_syl {
        let onset_slice = &clean[onset_bounds[k].0..onset_bounds[k].1];
        let nucleus_slice = &clean[nuclei_ranges[k].0..nuclei_ranges[k].1];
        let coda_slice = &clean[coda_bounds[k].0..coda_bounds[k].1];

        let onset_str: String = onset_slice.iter().collect();
        let nucleus_str: String = nucleus_slice.iter().collect();
        let coda_str: String = coda_slice.iter().collect();

        let is_open = coda_slice.is_empty();
        if is_open {
            open_count += 1;
        }

        let nuc_sonority = nucleus_slice
            .first()
            .map(|c| i16::from(inspect_char_acoustics(*c).sonority_micro))
            .unwrap_or(95);

        let onset_min_sonority = onset_slice
            .iter()
            .map(|c| i16::from(inspect_char_acoustics(*c).sonority_micro))
            .min()
            .unwrap_or(nuc_sonority);
        let coda_min_sonority = coda_slice
            .iter()
            .map(|c| i16::from(inspect_char_acoustics(*c).sonority_micro))
            .min()
            .unwrap_or(nuc_sonority);

        let onset_rise = (nuc_sonority - onset_min_sonority).max(0);
        let coda_fall = (nuc_sonority - coda_min_sonority).max(0);

        // Check for sonority reversals within the onset (where sonority drops before the vowel, e.g. `мгла`, `ртуть`)
        let mut has_reversal = false;
        if onset_slice.len() >= 2 {
            for w in onset_slice.windows(2) {
                let s1 = inspect_char_acoustics(w[0]).sonority_micro;
                let s2 = inspect_char_acoustics(w[1]).sonority_micro;
                if s1 > s2 + 5 {
                    has_reversal = true;
                }
            }
        }
        if has_reversal {
            reversal_count += 1;
        }

        // Compute normalized Clements dispersion D: steeper rise (larger onset_rise) -> lower dispersion D
        let norm_delta = (onset_rise as f32 / 20.0).max(0.8);
        let clements_d = (1.0 / (norm_delta * norm_delta)).clamp(0.04, 1.0);
        dispersion_sum += clements_d;

        syllables.push(StructuralSyllable {
            index: k,
            onset: onset_str,
            nucleus: nucleus_str,
            coda: coda_str,
            is_open,
            onset_sonority_rise: onset_rise,
            coda_sonority_fall: coda_fall,
            clements_dispersion: clements_d,
            has_sonority_plateau_or_reversal: has_reversal,
        });
    }

    WordSyllableTree {
        original_word: word.to_string(),
        open_syllable_ratio: open_count as f32 / num_syl as f32,
        mean_clements_dispersion: dispersion_sum / num_syl as f32,
        sonority_reversal_count: reversal_count,
        syllables,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_russian_open_syllable_tendency() {
        // "весна" -> ве-сна (both syllables open under Avanesov's rising sonority rule)
        let tree = parse_word_syllable_tree("весна");
        assert_eq!(tree.syllables.len(), 2);
        assert_eq!(tree.syllables[0].onset, "в");
        assert_eq!(tree.syllables[0].nucleus, "е");
        assert_eq!(tree.syllables[0].coda, "");
        assert_eq!(tree.syllables[1].onset, "сн");
        assert_eq!(tree.syllables[1].nucleus, "а");
        assert!((tree.open_syllable_ratio - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_sonority_reversal_detection_in_mgla() {
        let tree = parse_word_syllable_tree("мгла");
        assert_eq!(tree.syllables.len(), 1);
        assert!(tree.syllables[0].has_sonority_plateau_or_reversal);
        assert_eq!(tree.sonority_reversal_count, 1);
    }
}
