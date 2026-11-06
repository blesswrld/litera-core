//! Phonetic classification and optical ink-density tables for Cyrillic and Latin scripts.
//!
//! Operates in O(1) time per Unicode scalar value (`char`) with zero heap allocations.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhonemeClass {
    /// Open and resonant vowels (а, е, ё, и, о, у, ы, э, ю, я / a, e, i, o, u, y)
    Vowel,
    /// Liquid and nasal sonorants (л, м, н, р, й / l, m, n, r, w)
    Sonorant,
    /// Sibilants and fricatives creating breath/air texture (с, з, ш, щ, ж, ф, в, х / s, z, f, v, h)
    Fricative,
    /// Plosives and affricates creating rhythmic percussion (п, б, т, д, к, г, ц, ч / p, b, t, d, k, g)
    Plosive,
    /// Modifier signs or silent graphemes (ъ, ь)
    Modifier,
    /// Punctuation or whitespace
    NonPhonetic,
}

/// Classifies a single Unicode `char` into its acoustic-phonetic family.
/// Complexity: O(1) time, O(1) space.
#[inline]
pub fn classify_char(ch: char) -> PhonemeClass {
    let lower = ch.to_lowercase().next().unwrap_or(ch);
    match lower {
        // Cyrillic + Latin vowels
        'а' | 'е' | 'ё' | 'и' | 'о' | 'у' | 'ы' | 'э' | 'ю' | 'я' |
        'a' | 'e' | 'i' | 'o' | 'u' | 'y' => PhonemeClass::Vowel,

        // Sonorants (liquids, nasals, glides)
        'л' | 'м' | 'н' | 'р' | 'й' |
        'l' | 'm' | 'n' | 'r' | 'w' => PhonemeClass::Sonorant,

        // Fricatives & sibilants
        'в' | 'ф' | 'с' | 'з' | 'ш' | 'щ' | 'ж' | 'х' |
        'v' | 'f' | 's' | 'z' | 'h' | 'j' | 'x' => PhonemeClass::Fricative,

        // Plosives & affricates
        'б' | 'п' | 'д' | 'т' | 'г' | 'к' | 'ц' | 'ч' |
        'b' | 'p' | 'd' | 't' | 'g' | 'k' | 'c' | 'q' => PhonemeClass::Plosive,

        // Cyrillic palatalization / separation signs
        'ъ' | 'ь' => PhonemeClass::Modifier,

        _ => PhonemeClass::NonPhonetic,
    }
}

/// Estimates relative typographic ink density (grey value on paper) of a character
/// in a high-contrast Renaissance serif face (e.g., Cormorant Garamond), scaled 0..=100.
///
/// Wide multi-stem glyphs (`ж`, `ш`, `щ`, `м`, `ы`, `w`, `m`) deposit more ink per em
/// than narrow single-stem or open-counter glyphs (`і`, `l`, `с`, `о`).
#[inline]
pub fn char_ink_density(ch: char) -> u8 {
    let lower = ch.to_lowercase().next().unwrap_or(ch);
    match lower {
        'ш' | 'щ' | 'ж' | 'м' | 'ы' | 'ю' | 'ф' | 'w' | 'm' => 84,
        'д' | 'б' | 'в' | 'п' | 'н' | 'и' | 'й' | 'я' | 'х' | 'b' | 'd' | 'h' | 'k' | 'n' | 'p' | 'q' | 'u' => 68,
        'а' | 'е' | 'ё' | 'о' | 'р' | 'т' | 'у' | 'к' | 'г' | 'з' | 'ч' | 'ц' | 'a' | 'e' | 'g' | 'o' | 'v' | 'x' | 'y' | 'z' => 54,
        'с' | 'л' | 'э' | 'ь' | 'ъ' | 'c' | 'f' | 'j' | 'l' | 'r' | 's' | 't' => 42,
        'i' | '.' | ',' | ':' | ';' | '!' | '?' | '-' | '—' => 24,
        _ if ch.is_alphabetic() => 55,
        _ => 0,
    }
}

/// Computes a deterministic 64-bit FNV-1a hash of a UTF-8 string slice.
/// Used to seed spontaneous stanza generation and acoustic signatures without external RNG crates.
#[inline]
pub fn fnv1a_hash(text: &str) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET_BASIS;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}
