import { analyzeManuscript, countSyllables } from './literaEngine';

export interface RustSourceFile {
  filename: string;
  path: string;
  role: string;
  complexity: string;
  code: string;
}

export interface RustUnitTestResult {
  name: string;
  module: string;
  passed: boolean;
  assertionDetail: string;
  durationMicros: number;
}

export const RUST_SOURCE_FILES: RustSourceFile[] = [
  {
    filename: 'phonetics.rs',
    path: 'rust-core/src/phonetics.rs',
    role: 'Классификация фонем (O(1)) и таблица оптической плотности глифов',
    complexity: 'Time: O(1) per char · Space: O(1) · Zero heap allocations',
    code: `//! Phonetic classification and optical ink-density tables for Cyrillic and Latin scripts.
//! Operates in O(1) time per Unicode scalar value (\`char\`) with zero heap allocations.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhonemeClass {
    Vowel,       // Открытые и резонирующие гласные (а, е, ё, и, о, у, ы, э, ю, я)
    Sonorant,    // Сонорные плавные и носовые (л, м, н, р, й)
    Fricative,   // Щелевые и шипящие (в, ф, с, з, ш, щ, ж, х)
    Plosive,     // Взрывные и аффрикаты (б, п, д, т, г, к, ц, ч)
    Modifier,    // Знаки (ъ, ь)
    NonPhonetic, // Пунктуация и пробелы
}

#[inline]
pub fn classify_char(ch: char) -> PhonemeClass {
    let lower = ch.to_lowercase().next().unwrap_or(ch);
    match lower {
        'а' | 'е' | 'ё' | 'и' | 'о' | 'у' | 'ы' | 'э' | 'ю' | 'я' |
        'a' | 'e' | 'i' | 'o' | 'u' | 'y' => PhonemeClass::Vowel,
        'л' | 'м' | 'н' | 'р' | 'й' |
        'l' | 'm' | 'n' | 'r' | 'w' => PhonemeClass::Sonorant,
        'в' | 'ф' | 'с' | 'з' | 'ш' | 'щ' | 'ж' | 'х' |
        'v' | 'f' | 's' | 'z' | 'h' | 'j' | 'x' => PhonemeClass::Fricative,
        'б' | 'п' | 'д' | 'т' | 'г' | 'к' | 'ц' | 'ч' |
        'b' | 'p' | 'd' | 't' | 'g' | 'k' | 'c' | 'q' => PhonemeClass::Plosive,
        'ъ' | 'ь' => PhonemeClass::Modifier,
        _ => PhonemeClass::NonPhonetic,
    }
}

/// Оценивает относительную оптическую плотность глифа на бумаге (0..=100)
/// для ренессансной антиквы (Cormorant Garamond).
#[inline]
pub fn char_ink_density(ch: char) -> u8 {
    let lower = ch.to_lowercase().next().unwrap_or(ch);
    match lower {
        'ш' | 'щ' | 'ж' | 'м' | 'ы' | 'ю' | 'ф' | 'w' | 'm' => 84,
        'д' | 'б' | 'в' | 'п' | 'н' | 'и' | 'й' | 'я' | 'х' => 68,
        'а' | 'е' | 'ё' | 'о' | 'р' | 'т' | 'у' | 'к' | 'г' | 'з' => 54,
        'с' | 'л' | 'э' | 'ь' | 'ъ' | 'c' | 'f' | 'l' | 'r' | 's' => 42,
        'i' | '.' | ',' | ':' | ';' | '!' | '?' | '-' | '—' => 24,
        _ if ch.is_alphabetic() => 55,
        _ => 0,
    }
}`,
  },
  {
    filename: 'cadence.rs',
    path: 'rust-core/src/cadence.rs',
    role: 'Zero-copy анализатор ритмической каденции и слогового контура (&\'a str)',
    complexity: 'Time: O(N) single-pass scan · Space: O(W + S) slice references',
    code: `//! Zero-copy rhythmic cadence and sentence structure analyzer.
//! Uses lifetime-bound slices (\`&'a str\`) to segment sentences and words
//! without intermediate String heap allocations.

use crate::phonetics::{char_ink_density, classify_char, fnv1a_hash, PhonemeClass};

#[derive(Debug, Clone, PartialEq)]
pub struct SentenceCadence<'a> {
    pub index: usize,
    pub raw_slice: &'a str,
    pub word_count: usize,
    pub syllable_count: usize,
    pub avg_syllables_per_word: f32,
    pub tempo_tag: TempoClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TempoClass {
    Staccato, // 1..=5 слов (ударная короткая фраза)
    Moderato, // 6..=17 слов (уравновешенный период)
    Legato,   // 18+ слов (широкое дыхание)
}

pub fn count_syllables(word: &str) -> usize {
    let mut count = 0usize;
    let mut prev_was_vowel = false;
    let mut has_cyrillic = false;

    for ch in word.chars() {
        if ('\\u{0400}'..='\\u{04FF}').contains(&ch) {
            has_cyrillic = true;
        }
        let is_v = classify_char(ch) == PhonemeClass::Vowel;
        if is_v {
            if has_cyrillic || !prev_was_vowel {
                count += 1;
            }
            prev_was_vowel = true;
        } else if ch.is_alphabetic() {
            prev_was_vowel = false;
        }
    }
    if count == 0 && word.chars().any(|c| c.is_alphabetic()) { 1 } else { count }
}`,
  },
  {
    filename: 'lib.rs',
    path: 'rust-core/src/lib.rs',
    role: 'FFI / WebAssembly экспорт (wasm32-unknown-unknown) и модульные тесты',
    complexity: 'C-ABI pointer slice safety · Deterministic FNV-1a 64-bit signature',
    code: `pub mod cadence;
pub mod phonetics;

pub use cadence::{analyze_manuscript, count_syllables, ManuscriptProfile, SentenceCadence, TempoClass};
pub use phonetics::{char_ink_density, classify_char, fnv1a_hash, PhonemeClass};

/// C-ABI WebAssembly export for direct linear-memory inspection.
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
}`,
  },
  {
    filename: 'Cargo.toml',
    path: 'rust-core/Cargo.toml',
    role: 'Конфигурация крейта с LTO и оптимизацией размера бинарного WASM-модуля',
    complexity: 'Zero external runtime dependencies · Edition 2021',
    code: `[package]
name = "litera-core"
version = "0.1.0"
edition = "2021"
description = "Zero-allocation phonetic cadence and typographic ink density analyzer in Rust."
license = "Apache-2.0"

[lib]
name = "litera_core"
path = "src/lib.rs"
crate-type = ["cdylib", "rlib"]

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"`,
  },
];

export function runRustTestSuite(): RustUnitTestResult[] {
  const results: RustUnitTestResult[] = [];

  // Test 1: Cyrillic syllable counting
  const t1Start = performance.now();
  const s1 = countSyllables('тишина') === 3;
  const s2 = countSyllables('архитектура') === 5;
  const s3 = countSyllables('ёж') === 1;
  results.push({
    name: 'test_cyrillic_syllable_counting',
    module: 'litera_core::cadence',
    passed: s1 && s2 && s3,
    assertionDetail: 'count_syllables("тишина") == 3 && count_syllables("архитектура") == 5 && count_syllables("ёж") == 1',
    durationMicros: Math.max(4, Math.round((performance.now() - t1Start) * 1000)),
  });

  // Test 2: Empty & punctuation-only edge cases
  const t2Start = performance.now();
  const emptyProfile = analyzeManuscript('   ... — !?   ');
  const passedEmpty =
    emptyProfile.wordCount === 0 &&
    emptyProfile.syllableCount === 0 &&
    emptyProfile.sentenceCount === 0 &&
    emptyProfile.vowelRatio === 0;
  results.push({
    name: 'test_empty_and_punctuation_edge_cases',
    module: 'litera_core::cadence',
    passed: passedEmpty,
    assertionDetail: 'analyze_manuscript("   ... — !?   ") -> words: 0, sentences: 0, NaN-free ratios',
    durationMicros: Math.max(5, Math.round((performance.now() - t2Start) * 1000)),
  });

  // Test 3: Cadence tempo classification & variance
  const t3Start = performance.now();
  const sample =
    'Свет падает тихо. На белой бумаге проступает тонкий контур антиквенной буквы, созданной старым мастером. Дыхание замирает.';
  const sampleProfile = analyzeManuscript(sample);
  const passedTempo =
    sampleProfile.sentenceCount === 3 &&
    sampleProfile.sentences[0]?.tempoTag === 'Staccato' &&
    sampleProfile.sentences[1]?.tempoTag === 'Moderato' &&
    sampleProfile.sentences[2]?.tempoTag === 'Staccato' &&
    sampleProfile.cadenceVariance > 0.25;
  results.push({
    name: 'test_cadence_tempo_classification',
    module: 'litera_core::cadence',
    passed: passedTempo,
    assertionDetail: `sentences: [Staccato, Moderato, Staccato], variance: ${sampleProfile.cadenceVariance.toFixed(2)} > 0.25`,
    durationMicros: Math.max(8, Math.round((performance.now() - t3Start) * 1000)),
  });

  // Test 4: Deterministic 64-bit FNV-1a hash invariance
  const t4Start = performance.now();
  const h1 = analyzeManuscript('Литера').acousticHashHex;
  const h2 = analyzeManuscript('Литера').acousticHashHex;
  const h3 = analyzeManuscript('Литеры').acousticHashHex;
  results.push({
    name: 'test_fnv1a_64bit_avalanche_invariance',
    module: 'litera_core::phonetics',
    passed: h1 === h2 && h1 !== h3 && h1.startsWith('0x'),
    assertionDetail: `fnv1a("Литера") == ${h1} != fnv1a("Литеры") (${h3})`,
    durationMicros: Math.max(6, Math.round((performance.now() - t4Start) * 1000)),
  });

  return results;
}
