//! `litera-phonetics`: Acoustic-Phonetic Classification, Sonority Sequencing,
//! Prosodic Meter Detection, Alliteration/Assonance Graph Analysis, Narrow IPA Transcription,
//! Andrey Zaliznyak Accentual Paradigms, and Pure-Rust DSP Synthesis.

pub mod acoustic_synth;
pub mod alliteration;
pub mod cyrillic;
pub mod formant_synthesizer;
pub mod ipa_transcriber;
pub mod latin;
pub mod prosody;
pub mod stress_lexicon;
pub mod syllable_sonority_fsm;

pub use acoustic_synth::{
    encode_riff_wav_pcm16, schedule_cadence_score, synthesize_pcm16_score, BiquadLowpassFilter,
    CadenceNoteEvent, DEFAULT_SAMPLE_RATE_HZ,
};
pub use alliteration::{
    analyze_sound_devices, AlliterationChain, AssonanceCluster, SoundDeviceReport,
};
pub use cyrillic::{
    classify_cyrillic_char, cyrillic_formant_brightness, cyrillic_palatalization_weight,
    CyrillicArticulation, CyrillicVowelHeight,
};
pub use formant_synthesizer::{
    lookup_russian_vowel_formants, synthesize_formant_vowel_pcm16, CascadeFormantTract,
    KlattFormantFrame, LiljencrantsFantGlottalPulse,
};
pub use ipa_transcriber::{
    transcribe_clause_to_ipa, transcribe_word_to_ipa, IpaWordTranscription,
};
pub use latin::{classify_latin_char, count_latin_syllables, LatinArticulation};
pub use prosody::{
    scan_prosodic_profile, MetricalFoot, ProsodicLineScan, ProsodicProfile, StressConfidence,
};
pub use stress_lexicon::{
    lookup_lexicon_stress, LexiconStressEntry, ZaliznyakParadigm, LITERARY_STRESS_LEXICON,
};
pub use syllable_sonority_fsm::{
    is_valid_russian_onset_cluster, parse_word_syllable_tree, StructuralSyllable, WordSyllableTree,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PhonemeClass {
    Vowel = 1,
    Sonorant = 2,
    Fricative = 3,
    Plosive = 4,
    Modifier = 5,
    NonPhonetic = 0,
}

impl PhonemeClass {
    #[inline]
    pub const fn sonority_index(self) -> u8 {
        match self {
            PhonemeClass::Vowel => 10,
            PhonemeClass::Sonorant => 7,
            PhonemeClass::Fricative => 4,
            PhonemeClass::Plosive => 2,
            PhonemeClass::Modifier => 1,
            PhonemeClass::NonPhonetic => 0,
        }
    }

    #[inline]
    pub const fn russian_label(self) -> &'static str {
        match self {
            PhonemeClass::Vowel => "Гласные (Вокализм)",
            PhonemeClass::Sonorant => "Сонорные (Плавные и носовые)",
            PhonemeClass::Fricative => "Фрикативные (Щелевые и шипящие)",
            PhonemeClass::Plosive => "Взрывные (Смычные и аффрикаты)",
            PhonemeClass::Modifier => "Модификаторы (ъ, ь)",
            PhonemeClass::NonPhonetic => "Нефонетические знаки",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcousticDescriptor {
    pub class: PhonemeClass,
    pub sonority_micro: u8,
    pub brightness_f2: u8,
    pub is_voiced: bool,
    pub is_palatal: bool,
}

impl AcousticDescriptor {
    pub const NON_PHONETIC: Self = Self {
        class: PhonemeClass::NonPhonetic,
        sonority_micro: 0,
        brightness_f2: 0,
        is_voiced: false,
        is_palatal: false,
    };
}

#[inline]
pub fn classify_unicode_char(ch: char) -> PhonemeClass {
    let code = ch as u32;
    if (0x0400..=0x04FF).contains(&code) {
        cyrillic::classify_cyrillic_char(ch).class
    } else if code <= 0x024F {
        latin::classify_latin_char(ch).class
    } else {
        PhonemeClass::NonPhonetic
    }
}

#[inline]
pub fn inspect_char_acoustics(ch: char) -> AcousticDescriptor {
    let code = ch as u32;
    if (0x0400..=0x04FF).contains(&code) {
        cyrillic::classify_cyrillic_char(ch)
    } else if code <= 0x024F {
        latin::classify_latin_char(ch)
    } else {
        AcousticDescriptor::NON_PHONETIC
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WordSonorityContour {
    pub char_len: usize,
    pub syllable_peaks: usize,
    pub mean_sonority: f32,
    pub mean_brightness_f2: f32,
    pub voiced_ratio: f32,
    pub cluster_turbulence: usize,
}

pub fn analyze_word_sonority(word: &str) -> WordSonorityContour {
    let mut char_len = 0usize;
    let mut syllable_peaks = 0usize;
    let mut sonority_sum = 0u32;
    let mut brightness_sum = 0u32;
    let mut voiced_count = 0usize;
    let mut cluster_turbulence = 0usize;

    let mut prev_sonority: u8 = 0;
    let mut prev_was_consonant = false;
    let mut consecutive_consonants = 0usize;
    let mut has_cyrillic = false;
    let mut prev_was_vowel = false;

    for ch in word.chars() {
        let code = ch as u32;
        if (0x0400..=0x04FF).contains(&code) {
            has_cyrillic = true;
        }
        let desc = inspect_char_acoustics(ch);
        if desc.class == PhonemeClass::NonPhonetic {
            continue;
        }

        char_len += 1;
        sonority_sum += u32::from(desc.sonority_micro);
        brightness_sum += u32::from(desc.brightness_f2);
        if desc.is_voiced {
            voiced_count += 1;
        }

        if desc.class == PhonemeClass::Vowel {
            if has_cyrillic || !prev_was_vowel {
                syllable_peaks += 1;
            }
            prev_was_vowel = true;
            prev_was_consonant = false;
            consecutive_consonants = 0;
        } else if desc.class != PhonemeClass::Modifier {
            prev_was_vowel = false;
            consecutive_consonants += 1;
            if prev_was_consonant && consecutive_consonants >= 2 {
                if desc.sonority_micro == prev_sonority || consecutive_consonants >= 3 {
                    cluster_turbulence += 1;
                }
            }
            prev_was_consonant = true;
        }

        prev_sonority = desc.sonority_micro;
    }

    if syllable_peaks == 0 && char_len > 0 {
        syllable_peaks = 1;
    }

    let denom = char_len.max(1) as f32;
    WordSonorityContour {
        char_len,
        syllable_peaks,
        mean_sonority: sonority_sum as f32 / denom,
        mean_brightness_f2: brightness_sum as f32 / denom,
        voiced_ratio: voiced_count as f32 / denom,
        cluster_turbulence,
    }
}
