//! `litera-core`: Acoustic Typography & Phonetic Cadence Engine in Rust.
//!
//! Designed for both native CLI analysis and WebAssembly (`wasm32-unknown-unknown`) targets.

pub mod cadence;
pub mod phonetics;

pub use cadence::{analyze_manuscript, count_syllables, ManuscriptProfile, SentenceCadence, TempoClass, WordMetric};
pub use phonetics::{char_ink_density, classify_char, fnv1a_hash, PhonemeClass};

/// C-ABI compatible WebAssembly export for fast scalar metric computation on raw UTF-8 buffers.
#[no_mangle]
pub extern "C" fn wasm_compute_acoustic_signature(ptr: *const u8, len: usize) -> u64 {
    if ptr.is_null() || len == 0 {
        return 0;
    }
    let slice = unsafe { core::slice::from_raw_parts(ptr, len) };
    if let Ok(text) = core::str::from_utf8(slice) {
        fnv1a_hash(text)
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cyrillic_syllable_counting() {
        assert_eq!(count_syllables("тишина"), 3);
        assert_eq!(count_syllables("свет"), 1);
        assert_eq!(count_syllables("архитектура"), 5);
        assert_eq!(count_syllables("ёж"), 1);
    }

    #[test]
    fn test_empty_and_punctuation_edge_cases() {
        let profile = analyze_manuscript("   ... — !?   ");
        assert_eq!(profile.word_count, 0);
        assert_eq!(profile.syllable_count, 0);
        assert_eq!(profile.sentence_count, 0);
        assert_eq!(profile.vowel_ratio, 0.0);
    }

    #[test]
    fn test_cadence_tempo_classification() {
        let text = "Свет падает тихо. На белой бумаге проступает тонкий контур антиквенной буквы, созданной старым мастером. Дыхание замирает.";
        let profile = analyze_manuscript(text);
        assert_eq!(profile.sentence_count, 3);
        assert_eq!(profile.sentences[0].tempo_tag, TempoClass::Staccato);
        assert_eq!(profile.sentences[1].tempo_tag, TempoClass::Moderato);
        assert_eq!(profile.sentences[2].tempo_tag, TempoClass::Staccato);
        assert!(profile.cadence_variance > 0.25);
    }
}
