//! Pure-Rust WebAssembly Binary Format (`\0asm\x01\0\0\0`) Synthesizer and LEB128 Codec (`wasm_bytecode_gen.rs`).
//!
//! Encodes valid W3C WebAssembly 1.0 binary modules directly into byte vectors (`Vec<u8>`)
//! without external compiler toolchains, implementing:
//! - Unsigned and Signed DWARF LEB128 variable-length integer encoding (`encode_uleb128`, `encode_sleb128`)
//! - Section framing (`Type = 1`, `Function = 3`, `Memory = 5`, `Export = 7`, `Code = 10`)
//! - Stack-machine opcode generation for 64-bit FNV-1a hashing (`i64.const`, `i64.xor`, `i64.mul`, `i32.load8_u`)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WasmSectionId {
    Custom = 0,
    Type = 1,
    Import = 2,
    Function = 3,
    Table = 4,
    Memory = 5,
    Global = 6,
    Export = 7,
    Start = 8,
    Element = 9,
    Code = 10,
    Data = 11,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WasmValType {
    I32 = 0x7F,
    I64 = 0x7E,
    F32 = 0x7D,
    F64 = 0x7C,
}

/// Encodes an unsigned 64-bit integer in Unsigned LEB128 format.
#[inline]
pub fn encode_uleb128(mut value: u64, out: &mut Vec<u8>) {
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if value == 0 {
            break;
        }
    }
}

/// Encodes a signed 64-bit integer in Signed LEB128 format (two's complement).
#[inline]
pub fn encode_sleb128(mut value: i64, out: &mut Vec<u8>) {
    let mut more = true;
    while more {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        let sign_bit_set = (byte & 0x40) != 0;
        if (value == 0 && !sign_bit_set) || (value == -1 && sign_bit_set) {
            more = false;
        } else {
            byte |= 0x80;
        }
        out.push(byte);
    }
}

/// Appends a framed WebAssembly binary section `(section_id, uleb128(payload.len()), payload)`.
pub fn push_wasm_section(module: &mut Vec<u8>, id: WasmSectionId, payload: &[u8]) {
    module.push(id as u8);
    encode_uleb128(payload.len() as u64, module);
    module.extend_from_slice(payload);
}

/// Synthesizes a complete, standalone WebAssembly 1.0 binary module (`Vec<u8>`) that exports:
/// - `memory`: 1 linear memory page (64 KiB)
/// - `fnv1a_hash_wasm(ptr: i32, len: i32) -> i64`: native WASM 64-bit FNV-1a loop
pub fn build_litera_hash_wasm_binary() -> Vec<u8> {
    let mut wasm = Vec::with_capacity(256);

    // 8-byte WASM Magic Header (`\0asm`) + Version 1 (`\x01\0\0\0`)
    wasm.extend_from_slice(&[0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00]);

    // 1. Type Section: 1 function type `(i32, i32) -> (i64)`
    let mut type_sec = Vec::new();
    encode_uleb128(1, &mut type_sec); // 1 type
    type_sec.push(0x60); // func type constructor
    encode_uleb128(2, &mut type_sec); // 2 params
    type_sec.push(WasmValType::I32 as u8);
    type_sec.push(WasmValType::I32 as u8);
    encode_uleb128(1, &mut type_sec); // 1 result
    type_sec.push(WasmValType::I64 as u8);
    push_wasm_section(&mut wasm, WasmSectionId::Type, &type_sec);

    // 2. Function Section: 1 function referencing type index 0
    let mut func_sec = Vec::new();
    encode_uleb128(1, &mut func_sec);
    encode_uleb128(0, &mut func_sec);
    push_wasm_section(&mut wasm, WasmSectionId::Function, &func_sec);

    // 3. Memory Section: 1 memory with min = 1 page (64 KiB), max = 16 pages (1 MiB)
    let mut mem_sec = Vec::new();
    encode_uleb128(1, &mut mem_sec);
    mem_sec.push(0x01); // flags: has maximum
    encode_uleb128(1, &mut mem_sec);
    encode_uleb128(16, &mut mem_sec);
    push_wasm_section(&mut wasm, WasmSectionId::Memory, &mem_sec);

    // 4. Export Section: export "memory" (mem 0) and "fnv1a_hash_wasm" (func 0)
    let mut exp_sec = Vec::new();
    encode_uleb128(2, &mut exp_sec);
    // Export 0: "memory"
    let mem_name = b"memory";
    encode_uleb128(mem_name.len() as u64, &mut exp_sec);
    exp_sec.extend_from_slice(mem_name);
    exp_sec.push(0x02); // memory export kind
    encode_uleb128(0, &mut exp_sec);
    // Export 1: "fnv1a_hash_wasm"
    let fn_name = b"fnv1a_hash_wasm";
    encode_uleb128(fn_name.len() as u64, &mut exp_sec);
    exp_sec.extend_from_slice(fn_name);
    exp_sec.push(0x00); // func export kind
    encode_uleb128(0, &mut exp_sec);
    push_wasm_section(&mut wasm, WasmSectionId::Export, &exp_sec);

    // 5. Code Section: function 0 body
    // Locals: param 0 = ptr (i32), param 1 = len (i32), local 2 = hash (i64), local 3 = i (i32)
    let mut body = Vec::new();
    encode_uleb128(2, &mut body); // 2 local declaration groups
    encode_uleb128(1, &mut body);
    body.push(WasmValType::I64 as u8); // local 2: i64
    encode_uleb128(1, &mut body);
    body.push(WasmValType::I32 as u8); // local 3: i32

    // hash = 0xcbf29ce484222325 (as signed i64 = -3750763034362895579)
    body.push(0x42); // i64.const
    encode_sleb128(-3_750_763_034_362_895_579i64, &mut body);
    body.push(0x21); // local.set 2
    encode_uleb128(2, &mut body);

    // block $exit (void)
    body.push(0x02);
    body.push(0x40);
    // loop $top (void)
    body.push(0x03);
    body.push(0x40);

    // br_if $exit (i >= len)
    body.push(0x20); // local.get 3 (i)
    encode_uleb128(3, &mut body);
    body.push(0x20); // local.get 1 (len)
    encode_uleb128(1, &mut body);
    body.push(0x4E); // i32.ge_u
    body.push(0x0D); // br_if 1 ($exit)
    encode_uleb128(1, &mut body);

    // hash = (hash ^ i64.load8_u(ptr + i)) * 0x100000001b3
    body.push(0x20); // local.get 2 (hash)
    encode_uleb128(2, &mut body);
    body.push(0x20); // local.get 0 (ptr)
    encode_uleb128(0, &mut body);
    body.push(0x20); // local.get 3 (i)
    encode_uleb128(3, &mut body);
    body.push(0x6A); // i32.add
    body.push(0x30); // i64.load8_u align=0 offset=0
    body.push(0x00);
    body.push(0x00);
    body.push(0x85); // i64.xor
    body.push(0x42); // i64.const 0x100000001b3 (1099511628211)
    encode_sleb128(1_099_511_628_211i64, &mut body);
    body.push(0x7E); // i64.mul
    body.push(0x21); // local.set 2 (hash)
    encode_uleb128(2, &mut body);

    // i = i + 1
    body.push(0x20); // local.get 3
    encode_uleb128(3, &mut body);
    body.push(0x41); // i32.const 1
    encode_sleb128(1, &mut body);
    body.push(0x6A); // i32.add
    body.push(0x21); // local.set 3
    encode_uleb128(3, &mut body);

    // br 0 ($top)
    body.push(0x0C);
    encode_uleb128(0, &mut body);

    body.push(0x0B); // end loop
    body.push(0x0B); // end block

    // return hash (local.get 2)
    body.push(0x20);
    encode_uleb128(2, &mut body);
    body.push(0x0B); // end function

    let mut code_sec = Vec::new();
    encode_uleb128(1, &mut code_sec); // 1 function body
    encode_uleb128(body.len() as u64, &mut code_sec);
    code_sec.extend_from_slice(&body);
    push_wasm_section(&mut wasm, WasmSectionId::Code, &code_sec);

    wasm
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_binary_header_and_sections() {
        let bin = build_litera_hash_wasm_binary();
        assert_eq!(&bin[0..4], b"\0asm");
        assert_eq!(&bin[4..8], &[1, 0, 0, 0]);
        assert!(bin.len() > 80);
    }
}
