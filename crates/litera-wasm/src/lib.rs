//! `litera-wasm`: WebAssembly Linear-Memory Bridge, Bump Arena Allocator,
//! SWAR 64-Bit Parallel Byte Scanner, Zero-Dependency JSON Wire Serializer,
//! and Pure-Rust W3C WebAssembly Binary Module Emitter.

pub mod arena;
pub mod json_wire;
pub mod ring_buffer_dsp;
pub mod simd_scan;
pub mod wasm_bytecode_gen;

pub use arena::{ArenaSliceHandle, BumpArena};
pub use json_wire::JsonWireWriter;
pub use ring_buffer_dsp::{
    BallisticAudioMeter, CacheAlignedIndex, WasmAudioRingBuffer, DSP_RING_CAPACITY,
    WEBAUDIO_RENDER_QUANTUM_FRAMES,
};
pub use simd_scan::{swar_count_ascii_spaces, swar_count_utf8_codepoints, SwarScanReport};
pub use wasm_bytecode_gen::{
    build_litera_hash_wasm_binary, encode_sleb128, encode_uleb128, WasmSectionId, WasmValType,
};

use litera_phonetics::{analyze_sound_devices, scan_prosodic_profile};
use litera_stylometry::compute_lexical_entropy;
use litera_typography::{compare_paragraph_breaking, FontCategory};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WasmBufferSlice {
    pub ptr: u32,
    pub len: u32,
}

impl WasmBufferSlice {
    #[inline]
    pub const fn pack_u64(self) -> u64 {
        ((self.ptr as u64) << 32) | (self.len as u64)
    }
}

pub fn analyze_full_pipeline_json(text: &str, measure_em: f32) -> String {
    let swar = simd_scan::swar_scan_buffer(text.as_bytes());
    let prosody = scan_prosodic_profile(text);
    let sound = analyze_sound_devices(text);
    let entropy = compute_lexical_entropy(text);
    let breaking = compare_paragraph_breaking(text, measure_em, FontCategory::RenaissanceSerif);

    let mut writer = JsonWireWriter::with_capacity(2048);
    writer.begin_object();
    writer.field_u64("utf8_codepoints", swar.utf8_codepoints as u64);
    writer.field_u64("ascii_whitespace", swar.ascii_spaces as u64);
    writer.field_str("dominant_meter", prosody.dominant_foot.russian_name());
    writer.field_f32("meter_regularity", prosody.mean_regularity);
    writer.field_f32("alliteration_density", sound.alliteration_density);
    writer.field_f32("char_entropy_bits", entropy.char_entropy_bits);
    writer.field_f32("word_entropy_bits", entropy.word_entropy_bits);
    writer.field_f32("yules_k", entropy.yules_characteristic_k);
    writer.field_f32("honores_r", entropy.honores_statistic_r);
    writer.field_i64("knuth_plass_demerits", breaking.knuth_plass_total_demerits);
    writer.field_i64("greedy_demerits", breaking.greedy_total_demerits);
    writer.field_f32("demerit_reduction_pct", breaking.demerit_reduction_pct);
    writer.end_object();

    writer.into_string()
}
