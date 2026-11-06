//! Latin script acoustic-phonetic classification and English/Romance syllable heuristics.

use crate::{AcousticDescriptor, PhonemeClass};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LatinArticulation {
    Labial,
    Coronal,
    Dorsal,
    Glottal,
    Vowel,
    Other,
}

#[inline]
pub fn classify_latin_char(ch: char) -> AcousticDescriptor {
    let lower = ch.to_ascii_lowercase();
    match lower {
        'a' => AcousticDescriptor {
            class: PhonemeClass::Vowel,
            sonority_micro: 100,
            brightness_f2: 60,
            is_voiced: true,
            is_palatal: false,
        },
        'e' => AcousticDescriptor {
            class: PhonemeClass::Vowel,
            sonority_micro: 92,
            brightness_f2: 82,
            is_voiced: true,
            is_palatal: true,
        },
        'i' | 'y' => AcousticDescriptor {
            class: PhonemeClass::Vowel,
            sonority_micro: 86,
            brightness_f2: 94,
            is_voiced: true,
            is_palatal: true,
        },
        'o' => AcousticDescriptor {
            class: PhonemeClass::Vowel,
            sonority_micro: 94,
            brightness_f2: 36,
            is_voiced: true,
            is_palatal: false,
        },
        'u' => AcousticDescriptor {
            class: PhonemeClass::Vowel,
            sonority_micro: 84,
            brightness_f2: 28,
            is_voiced: true,
            is_palatal: false,
        },
        'l' | 'r' | 'w' => AcousticDescriptor {
            class: PhonemeClass::Sonorant,
            sonority_micro: 74,
            brightness_f2: 50,
            is_voiced: true,
            is_palatal: false,
        },
        'm' | 'n' => AcousticDescriptor {
            class: PhonemeClass::Sonorant,
            sonority_micro: 68,
            brightness_f2: 46,
            is_voiced: true,
            is_palatal: false,
        },
        'v' | 'z' | 'j' => AcousticDescriptor {
            class: PhonemeClass::Fricative,
            sonority_micro: 48,
            brightness_f2: 62,
            is_voiced: true,
            is_palatal: false,
        },
        'f' | 's' | 'h' | 'x' => AcousticDescriptor {
            class: PhonemeClass::Fricative,
            sonority_micro: 36,
            brightness_f2: 70,
            is_voiced: false,
            is_palatal: false,
        },
        'b' | 'd' | 'g' => AcousticDescriptor {
            class: PhonemeClass::Plosive,
            sonority_micro: 27,
            brightness_f2: 44,
            is_voiced: true,
            is_palatal: false,
        },
        'p' | 't' | 'k' | 'c' | 'q' => AcousticDescriptor {
            class: PhonemeClass::Plosive,
            sonority_micro: 18,
            brightness_f2: 46,
            is_voiced: false,
            is_palatal: false,
        },
        _ => AcousticDescriptor::NON_PHONETIC,
    }
}

/// Counts syllables in a Latin-script word using vowel-run coalescing,
/// silent trailing `-e` adjustment, and syllabic `-le` restoration.
pub fn count_latin_syllables(word: &str) -> usize {
    let bytes = word.as_bytes();
    if bytes.is_empty() {
        return 0;
    }

    let mut count = 0usize;
    let mut prev_vowel = false;
    let mut alpha_len = 0usize;

    for &b in bytes {
        let lower = b.to_ascii_lowercase();
        if lower.is_ascii_alphabetic() {
            alpha_len += 1;
            let is_v = matches!(lower, b'a' | b'e' | b'i' | b'o' | b'u' | b'y');
            if is_v {
                if !prev_vowel {
                    count += 1;
                }
                prev_vowel = true;
            } else {
                prev_vowel = false;
            }
        }
    }

    if alpha_len == 0 {
        return 0;
    }

    // Adjust for silent trailing 'e' in words longer than 3 letters (unless ending in consonant + "le")
    if alpha_len > 3 && count > 1 {
        let last = bytes[bytes.len() - 1].to_ascii_lowercase();
        let penult = bytes[bytes.len() - 2].to_ascii_lowercase();
        if last == b'e' && !matches!(penult, b'a' | b'e' | b'i' | b'o' | b'u' | b'y' | b'l') {
            count -= 1;
        }
    }

    count.max(1)
}
