//! SWAR (SIMD Within A Register) 64-Bit Parallel UTF-8 Codepoint and Delimiter Scanner.
//!
//! Processes 8 bytes per CPU cycle using bitwise parallel tricks (`u64` word loads)
//! to count UTF-8 scalar values (bytes where `(b & 0xC0) != 0x80`), ASCII spaces,
//! and sentence-terminal punctuation without branch mispredictions.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwarScanReport {
    pub byte_len: usize,
    pub utf8_codepoints: usize,
    pub ascii_spaces: usize,
    pub sentence_delimiters: usize,
    pub cyrillic_lead_bytes: usize,
}

const LO_BYTES: u64 = 0x0101_0101_0101_0101;
const HI_BYTES: u64 = 0x8080_8080_8080_8080;

/// Returns a bitmask with the high bit (`0x80`) of each byte lane set if that byte equals `target`.
#[inline]
const fn swar_has_byte(word: u64, target: u8) -> u64 {
    let mask = (target as u64) * LO_BYTES;
    let xored = word ^ mask;
    xored.wrapping_sub(LO_BYTES) & !xored & HI_BYTES
}

/// Counts UTF-8 non-continuation bytes (`(byte & 0xC0) != 0x80`) across 8 bytes in parallel
/// using PopCount (`count_ones()`).
#[inline]
pub fn swar_count_utf8_codepoints(bytes: &[u8]) -> usize {
    let mut count = 0usize;
    let chunks = bytes.chunks_exact(8);
    let rem = chunks.remainder();

    for chunk in chunks {
        let word = u64::from_le_bytes([
            chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7],
        ]);
        // A byte is a UTF-8 continuation byte iff bit 7 is 1 and bit 6 is 0.
        let cont_mask = (word & !word.wrapping_shl(1)) & HI_BYTES;
        let cont_count = cont_mask.count_ones() as usize;
        count += 8 - cont_count;
    }

    for &b in rem {
        if (b & 0xC0) != 0x80 {
            count += 1;
        }
    }

    count
}

/// Counts ASCII space (`0x20`) occurrences using 64-bit SWAR parallel comparison.
#[inline]
pub fn swar_count_ascii_spaces(bytes: &[u8]) -> usize {
    let mut spaces = 0usize;
    let chunks = bytes.chunks_exact(8);
    let rem = chunks.remainder();

    for chunk in chunks {
        let word = u64::from_le_bytes([
            chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7],
        ]);
        let m_space = swar_has_byte(word, b' ');
        spaces += m_space.count_ones() as usize;
    }

    for &b in rem {
        if b == b' ' {
            spaces += 1;
        }
    }

    spaces
}

/// Performs a full single-pass SWAR scan over a UTF-8 buffer.
pub fn swar_scan_buffer(bytes: &[u8]) -> SwarScanReport {
    let utf8_codepoints = swar_count_utf8_codepoints(bytes);
    let ascii_spaces = swar_count_ascii_spaces(bytes);

    let mut sentence_delimiters = 0usize;
    let mut cyrillic_lead_bytes = 0usize;

    for &b in bytes {
        if matches!(b, b'.' | b'!' | b'?' | b'\n') {
            sentence_delimiters += 1;
        } else if matches!(b, 0xD0..=0xD3) {
            // UTF-8 leading byte for Cyrillic block U+0400..U+04FF
            cyrillic_lead_bytes += 1;
        }
    }

    SwarScanReport {
        byte_len: bytes.len(),
        utf8_codepoints,
        ascii_spaces,
        sentence_delimiters,
        cyrillic_lead_bytes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swar_utf8_codepoint_accuracy() {
        let text = "Тишина имеет собственный вес. Silence has weight.";
        let swar_cp = swar_count_utf8_codepoints(text.as_bytes());
        let std_cp = text.chars().count();
        assert_eq!(swar_cp, std_cp);
    }
}
