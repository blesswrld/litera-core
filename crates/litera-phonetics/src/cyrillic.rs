//! Detailed Cyrillic articulatory phonetics, formant brightness, and palatalization rules.
//!
//! Models Russian and pan-Cyrillic phonological traits including vowel height (open `а` vs
//! close `и`, `у`), place of articulation (bilabial, labiodental, dental/alveolar,
//! postalveolar, palatal, velar), and voicing assimilation vectors.

use crate::{AcousticDescriptor, PhonemeClass};

/// Articulatory place of a Cyrillic consonant or vowel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CyrillicArticulation {
    Bilabial,
    Labiodental,
    DentalAlveolar,
    PostalveolarHushing,
    PalatalGlide,
    Velar,
    VowelNucleus(CyrillicVowelHeight),
    OrthographicSign,
    Unknown,
}

/// Vertical tongue height of Cyrillic vowels, directly correlating with first formant (F1)
/// energy and perceived acoustic fullness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CyrillicVowelHeight {
    /// Maximum acoustic jaw opening (`а`, `я`) -> highest sonority (100).
    OpenLow,
    /// Mid-open / mid-close (`о`, `ё`, `е`, `э`) -> high sonority (92).
    Mid,
    /// Close / high vowels (`и`, `ы`, `у`, `ю`) -> focused sonority (84).
    CloseHigh,
}

/// Classifies a Cyrillic character into a rich `AcousticDescriptor`.
#[inline]
pub fn classify_cyrillic_char(ch: char) -> AcousticDescriptor {
    let lower = match ch {
        'А'..='Я' => char::from_u32((ch as u32) + 32).unwrap_or(ch),
        'Ё' => 'ё',
        _ => ch,
    };

    match lower {
        // Open low vowels (maximum F1 energy)
        'а' => AcousticDescriptor {
            class: PhonemeClass::Vowel,
            sonority_micro: 100,
            brightness_f2: 58,
            is_voiced: true,
            is_palatal: false,
        },
        'я' => AcousticDescriptor {
            class: PhonemeClass::Vowel,
            sonority_micro: 98,
            brightness_f2: 76,
            is_voiced: true,
            is_palatal: true,
        },

        // Mid vowels
        'о' => AcousticDescriptor {
            class: PhonemeClass::Vowel,
            sonority_micro: 94,
            brightness_f2: 36,
            is_voiced: true,
            is_palatal: false,
        },
        'ё' => AcousticDescriptor {
            class: PhonemeClass::Vowel,
            sonority_micro: 94,
            brightness_f2: 64,
            is_voiced: true,
            is_palatal: true,
        },
        'э' => AcousticDescriptor {
            class: PhonemeClass::Vowel,
            sonority_micro: 92,
            brightness_f2: 72,
            is_voiced: true,
            is_palatal: false,
        },
        'е' => AcousticDescriptor {
            class: PhonemeClass::Vowel,
            sonority_micro: 92,
            brightness_f2: 84,
            is_voiced: true,
            is_palatal: true,
        },

        // Close high vowels
        'и' | 'і' | 'ї' => AcousticDescriptor {
            class: PhonemeClass::Vowel,
            sonority_micro: 86,
            brightness_f2: 96,
            is_voiced: true,
            is_palatal: true,
        },
        'ы' => AcousticDescriptor {
            class: PhonemeClass::Vowel,
            sonority_micro: 85,
            brightness_f2: 48,
            is_voiced: true,
            is_palatal: false,
        },
        'у' => AcousticDescriptor {
            class: PhonemeClass::Vowel,
            sonority_micro: 84,
            brightness_f2: 24,
            is_voiced: true,
            is_palatal: false,
        },
        'ю' => AcousticDescriptor {
            class: PhonemeClass::Vowel,
            sonority_micro: 84,
            brightness_f2: 52,
            is_voiced: true,
            is_palatal: true,
        },

        // Sonorants: glides, liquids, nasals
        'й' => AcousticDescriptor {
            class: PhonemeClass::Sonorant,
            sonority_micro: 78,
            brightness_f2: 92,
            is_voiced: true,
            is_palatal: true,
        },
        'р' => AcousticDescriptor {
            class: PhonemeClass::Sonorant,
            sonority_micro: 75,
            brightness_f2: 54,
            is_voiced: true,
            is_palatal: false,
        },
        'л' => AcousticDescriptor {
            class: PhonemeClass::Sonorant,
            sonority_micro: 73,
            brightness_f2: 46,
            is_voiced: true,
            is_palatal: false,
        },
        'м' => AcousticDescriptor {
            class: PhonemeClass::Sonorant,
            sonority_micro: 68,
            brightness_f2: 38,
            is_voiced: true,
            is_palatal: false,
        },
        'н' => AcousticDescriptor {
            class: PhonemeClass::Sonorant,
            sonority_micro: 67,
            brightness_f2: 56,
            is_voiced: true,
            is_palatal: false,
        },

        // Voiced fricatives & sibilants
        'в' => AcousticDescriptor {
            class: PhonemeClass::Fricative,
            sonority_micro: 52,
            brightness_f2: 40,
            is_voiced: true,
            is_palatal: false,
        },
        'з' => AcousticDescriptor {
            class: PhonemeClass::Fricative,
            sonority_micro: 48,
            brightness_f2: 68,
            is_voiced: true,
            is_palatal: false,
        },
        'ж' => AcousticDescriptor {
            class: PhonemeClass::Fricative,
            sonority_micro: 46,
            brightness_f2: 44,
            is_voiced: true,
            is_palatal: false,
        },

        // Voiceless fricatives & sibilants
        'ф' => AcousticDescriptor {
            class: PhonemeClass::Fricative,
            sonority_micro: 38,
            brightness_f2: 42,
            is_voiced: false,
            is_palatal: false,
        },
        'с' => AcousticDescriptor {
            class: PhonemeClass::Fricative,
            sonority_micro: 36,
            brightness_f2: 78,
            is_voiced: false,
            is_palatal: false,
        },
        'ш' => AcousticDescriptor {
            class: PhonemeClass::Fricative,
            sonority_micro: 35,
            brightness_f2: 46,
            is_voiced: false,
            is_palatal: false,
        },
        'щ' => AcousticDescriptor {
            class: PhonemeClass::Fricative,
            sonority_micro: 37,
            brightness_f2: 86,
            is_voiced: false,
            is_palatal: true,
        },
        'х' => AcousticDescriptor {
            class: PhonemeClass::Fricative,
            sonority_micro: 34,
            brightness_f2: 32,
            is_voiced: false,
            is_palatal: false,
        },

        // Voiced plosives
        'б' => AcousticDescriptor {
            class: PhonemeClass::Plosive,
            sonority_micro: 28,
            brightness_f2: 34,
            is_voiced: true,
            is_palatal: false,
        },
        'д' => AcousticDescriptor {
            class: PhonemeClass::Plosive,
            sonority_micro: 27,
            brightness_f2: 52,
            is_voiced: true,
            is_palatal: false,
        },
        'г' | 'ґ' => AcousticDescriptor {
            class: PhonemeClass::Plosive,
            sonority_micro: 26,
            brightness_f2: 36,
            is_voiced: true,
            is_palatal: false,
        },

        // Voiceless plosives & affricates
        'ч' => AcousticDescriptor {
            class: PhonemeClass::Plosive,
            sonority_micro: 22,
            brightness_f2: 82,
            is_voiced: false,
            is_palatal: true,
        },
        'ц' => AcousticDescriptor {
            class: PhonemeClass::Plosive,
            sonority_micro: 20,
            brightness_f2: 74,
            is_voiced: false,
            is_palatal: false,
        },
        'п' => AcousticDescriptor {
            class: PhonemeClass::Plosive,
            sonority_micro: 18,
            brightness_f2: 32,
            is_voiced: false,
            is_palatal: false,
        },
        'т' => AcousticDescriptor {
            class: PhonemeClass::Plosive,
            sonority_micro: 17,
            brightness_f2: 54,
            is_voiced: false,
            is_palatal: false,
        },
        'к' => AcousticDescriptor {
            class: PhonemeClass::Plosive,
            sonority_micro: 16,
            brightness_f2: 36,
            is_voiced: false,
            is_palatal: false,
        },

        // Palatalization / separation signs
        'ь' => AcousticDescriptor {
            class: PhonemeClass::Modifier,
            sonority_micro: 10,
            brightness_f2: 88,
            is_voiced: false,
            is_palatal: true,
        },
        'ъ' => AcousticDescriptor {
            class: PhonemeClass::Modifier,
            sonority_micro: 8,
            brightness_f2: 20,
            is_voiced: false,
            is_palatal: false,
        },

        _ => AcousticDescriptor::NON_PHONETIC,
    }
}

/// Returns the detailed articulatory place for a Cyrillic character.
#[inline]
pub fn cyrillic_articulation_place(ch: char) -> CyrillicArticulation {
    let lower = ch.to_lowercase().next().unwrap_or(ch);
    match lower {
        'б' | 'п' | 'м' => CyrillicArticulation::Bilabial,
        'в' | 'ф' => CyrillicArticulation::Labiodental,
        'д' | 'т' | 'з' | 'с' | 'ц' | 'н' | 'л' | 'р' => CyrillicArticulation::DentalAlveolar,
        'ж' | 'ш' | 'щ' | 'ч' => CyrillicArticulation::PostalveolarHushing,
        'й' => CyrillicArticulation::PalatalGlide,
        'г' | 'к' | 'х' => CyrillicArticulation::Velar,
        'а' | 'я' => CyrillicArticulation::VowelNucleus(CyrillicVowelHeight::OpenLow),
        'о' | 'ё' | 'е' | 'э' => CyrillicArticulation::VowelNucleus(CyrillicVowelHeight::Mid),
        'и' | 'ы' | 'у' | 'ю' => CyrillicArticulation::VowelNucleus(CyrillicVowelHeight::CloseHigh),
        'ъ' | 'ь' => CyrillicArticulation::OrthographicSign,
        _ => CyrillicArticulation::Unknown,
    }
}

/// Computes the acoustic F2 brightness ratio (0.0..=1.0) of a Cyrillic text slice.
/// High values indicate front/palatal vowel dominance (`и`, `е`, `я`, `щ`, `ч` — "светлый тембр");
/// low values indicate back/labial dominance (`у`, `о`, `ы`, `ш`, `х` — "тёмный/матовый тембр").
pub fn cyrillic_formant_brightness(text: &str) -> f32 {
    let mut sum = 0u64;
    let mut count = 0u64;
    for ch in text.chars() {
        let desc = classify_cyrillic_char(ch);
        if desc.class != PhonemeClass::NonPhonetic {
            sum += u64::from(desc.brightness_f2);
            count += 1;
        }
    }
    if count == 0 {
        0.5
    } else {
        (sum as f32 / (count as f32 * 100.0)).clamp(0.0, 1.0)
    }
}

/// Computes the palatalization density (proportion of soft-inducing vowels and soft consonants)
/// in a Cyrillic text slice.
pub fn cyrillic_palatalization_weight(text: &str) -> f32 {
    let mut palatal = 0usize;
    let mut total = 0usize;
    for ch in text.chars() {
        let desc = classify_cyrillic_char(ch);
        if desc.class != PhonemeClass::NonPhonetic {
            total += 1;
            if desc.is_palatal {
                palatal += 1;
            }
        }
    }
    if total == 0 {
        0.0
    } else {
        palatal as f32 / total as f32
    }
}
