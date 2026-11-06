//! `litera-core`: Acoustic Typography & Phonetic Cadence Engine in Rust.
//!
//! Designed for both native CLI analysis and WebAssembly (`wasm32-unknown-unknown`) targets.

pub mod anthology;
pub mod bench_suite;
pub mod cadence;
pub mod phonetics;

pub use anthology::{find_specimen_by_id, AnthologySpecimen, SpecimenTradition, ANTHOLOGY_SPECIMENS};
pub use bench_suite::{run_full_architectural_verification, InvariantCheckResult};
pub use cadence::{
    analyze_manuscript, count_syllables, ManuscriptProfile, SentenceCadence, TempoClass, WordMetric,
};
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
