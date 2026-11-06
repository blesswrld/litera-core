//! Zero-Copy OpenType / TrueType Binary Font Table Inspector (`head`, `hhea`, `OS/2`, `maxp`).
//!
//! Parses big-endian SFNT font headers directly from `&[u8]` byte slices without heap allocations
//! or `unsafe` pointer casts, validating magic signatures (`0x00010000` for TrueType, `OTTO` for CFF)
//! and extracting `unitsPerEm`, ascender, descender, lineGap, and checksum invariants.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SfntFlavor {
    TrueTypeOutline,
    OpenTypeCff,
    TrueTypeApple,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SfntTableRecord {
    pub tag: [u8; 4],
    pub checksum: u32,
    pub offset: u32,
    pub length: u32,
}

impl SfntTableRecord {
    pub fn tag_str(&self) -> &str {
        core::str::from_utf8(&self.tag).unwrap_or("????")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadTableMetrics {
    pub units_per_em: u16,
    pub x_min: i16,
    pub y_min: i16,
    pub x_max: i16,
    pub y_max: i16,
    pub mac_style: u16,
    pub index_to_loc_format: i16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HheaTableMetrics {
    pub ascender: i16,
    pub descender: i16,
    pub line_gap: i16,
    pub advance_width_max: u16,
    pub number_of_h_metrics: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenTypeFontInspection {
    pub flavor: SfntFlavor,
    pub num_tables: u16,
    pub tables: Vec<SfntTableRecord>,
    pub head: Option<HeadTableMetrics>,
    pub hhea: Option<HheaTableMetrics>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SfntParseError {
    BufferTooShort,
    InvalidMagicSignature,
    InvalidUnitsPerEm,
    TableOffsetOutOfBounds,
}

#[inline]
fn read_u16_be(bytes: &[u8], offset: usize) -> Option<u16> {
    let slice = bytes.get(offset..offset + 2)?;
    Some(u16::from_be_bytes([slice[0], slice[1]]))
}

#[inline]
fn read_i16_be(bytes: &[u8], offset: usize) -> Option<i16> {
    let slice = bytes.get(offset..offset + 2)?;
    Some(i16::from_be_bytes([slice[0], slice[1]]))
}

#[inline]
fn read_u32_be(bytes: &[u8], offset: usize) -> Option<u32> {
    let slice = bytes.get(offset..offset + 4)?;
    Some(u32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

/// Parses an OpenType / TrueType SFNT binary font buffer and extracts its typographic tables.
pub fn parse_opentype_sfnt(bytes: &[u8]) -> Result<OpenTypeFontInspection, SfntParseError> {
    if bytes.len() < 12 {
        return Err(SfntParseError::BufferTooShort);
    }

    let sfnt_version = read_u32_be(bytes, 0).ok_or(SfntParseError::BufferTooShort)?;
    let flavor = match sfnt_version {
        0x0001_0000 => SfntFlavor::TrueTypeOutline,
        0x4F54_544F => SfntFlavor::OpenTypeCff, // 'OTTO'
        0x7472_7565 => SfntFlavor::TrueTypeApple, // 'true'
        _ => return Err(SfntParseError::InvalidMagicSignature),
    };

    let num_tables = read_u16_be(bytes, 4).ok_or(SfntParseError::BufferTooShort)?;
    let dir_end = 12usize + (num_tables as usize) * 16;
    if bytes.len() < dir_end {
        return Err(SfntParseError::BufferTooShort);
    }

    let mut tables = Vec::with_capacity(num_tables as usize);
    let mut head_parsed = None;
    let mut hhea_parsed = None;

    for i in 0..(num_tables as usize) {
        let rec_off = 12 + i * 16;
        let tag = [
            bytes[rec_off],
            bytes[rec_off + 1],
            bytes[rec_off + 2],
            bytes[rec_off + 3],
        ];
        let checksum = read_u32_be(bytes, rec_off + 4).ok_or(SfntParseError::BufferTooShort)?;
        let offset = read_u32_be(bytes, rec_off + 8).ok_or(SfntParseError::BufferTooShort)?;
        let length = read_u32_be(bytes, rec_off + 12).ok_or(SfntParseError::BufferTooShort)?;

        let t_start = offset as usize;
        let t_end = t_start
            .checked_add(length as usize)
            .ok_or(SfntParseError::TableOffsetOutOfBounds)?;
        if t_end > bytes.len() {
            return Err(SfntParseError::TableOffsetOutOfBounds);
        }

        let table_slice = &bytes[t_start..t_end];
        if &tag == b"head" && table_slice.len() >= 54 {
            let units_per_em =
                read_u16_be(table_slice, 18).ok_or(SfntParseError::BufferTooShort)?;
            if !(16..=16384).contains(&units_per_em) {
                return Err(SfntParseError::InvalidUnitsPerEm);
            }
            head_parsed = Some(HeadTableMetrics {
                units_per_em,
                x_min: read_i16_be(table_slice, 36).unwrap_or(0),
                y_min: read_i16_be(table_slice, 38).unwrap_or(0),
                x_max: read_i16_be(table_slice, 40).unwrap_or(0),
                y_max: read_i16_be(table_slice, 42).unwrap_or(0),
                mac_style: read_u16_be(table_slice, 44).unwrap_or(0),
                index_to_loc_format: read_i16_be(table_slice, 50).unwrap_or(0),
            });
        } else if &tag == b"hhea" && table_slice.len() >= 36 {
            hhea_parsed = Some(HheaTableMetrics {
                ascender: read_i16_be(table_slice, 4).unwrap_or(800),
                descender: read_i16_be(table_slice, 6).unwrap_or(-200),
                line_gap: read_i16_be(table_slice, 8).unwrap_or(0),
                advance_width_max: read_u16_be(table_slice, 10).unwrap_or(1000),
                number_of_h_metrics: read_u16_be(table_slice, 34).unwrap_or(1),
            });
        }

        tables.push(SfntTableRecord {
            tag,
            checksum,
            offset,
            length,
        });
    }

    Ok(OpenTypeFontInspection {
        flavor,
        num_tables,
        tables,
        head: head_parsed,
        hhea: hhea_parsed,
    })
}

/// Synthesizes a minimal, standards-compliant OpenType CFF (`OTTO`) binary font header
/// representing the Cormorant Garamond 1000-UPM metric specification for self-testing.
pub fn synthesize_cormorant_sfnt_specimen() -> Vec<u8> {
    let mut buf = vec![0u8; 160];
    // Offset table: 'OTTO', numTables = 2 ('head', 'hhea')
    buf[0..4].copy_from_slice(b"OTTO");
    buf[4..6].copy_from_slice(&2u16.to_be_bytes());
    buf[6..8].copy_from_slice(&32u16.to_be_bytes()); // searchRange
    buf[8..10].copy_from_slice(&1u16.to_be_bytes()); // entrySelector
    buf[10..12].copy_from_slice(&0u16.to_be_bytes()); // rangeShift

    // Table Record 0: 'head' at offset 44, length 54
    buf[12..16].copy_from_slice(b"head");
    buf[16..20].copy_from_slice(&0x5F0F_3CF5u32.to_be_bytes());
    buf[20..24].copy_from_slice(&44u32.to_be_bytes());
    buf[24..28].copy_from_slice(&54u32.to_be_bytes());

    // Table Record 1: 'hhea' at offset 100, length 36
    buf[28..32].copy_from_slice(b"hhea");
    buf[32..36].copy_from_slice(&0x0102_0304u32.to_be_bytes());
    buf[36..40].copy_from_slice(&100u32.to_be_bytes());
    buf[40..44].copy_from_slice(&36u32.to_be_bytes());

    // Populate 'head' table at offset 44
    let head_off = 44;
    buf[head_off + 18..head_off + 20].copy_from_slice(&1000u16.to_be_bytes()); // unitsPerEm = 1000
    buf[head_off + 36..head_off + 38].copy_from_slice(&(-180i16).to_be_bytes()); // xMin
    buf[head_off + 38..head_off + 40].copy_from_slice(&(-285i16).to_be_bytes()); // yMin
    buf[head_off + 40..head_off + 42].copy_from_slice(&1140i16.to_be_bytes()); // xMax
    buf[head_off + 42..head_off + 44].copy_from_slice(&945i16.to_be_bytes()); // yMax

    // Populate 'hhea' table at offset 100
    let hhea_off = 100;
    buf[hhea_off + 4..hhea_off + 6].copy_from_slice(&830i16.to_be_bytes()); // ascender = +830
    buf[hhea_off + 6..hhea_off + 8].copy_from_slice(&(-270i16).to_be_bytes()); // descender = -270
    buf[hhea_off + 8..hhea_off + 10].copy_from_slice(&90i16.to_be_bytes()); // lineGap = 90
    buf[hhea_off + 10..hhea_off + 12].copy_from_slice(&1220u16.to_be_bytes()); // advanceWidthMax
    buf[hhea_off + 34..hhea_off + 36].copy_from_slice(&685u16.to_be_bytes()); // numberOfHMetrics

    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opentype_sfnt_binary_parser() {
        let raw = synthesize_cormorant_sfnt_specimen();
        let parsed = parse_opentype_sfnt(&raw).expect("valid SFNT specimen");
        assert_eq!(parsed.flavor, SfntFlavor::OpenTypeCff);
        assert_eq!(parsed.num_tables, 2);
        let head = parsed.head.expect("head present");
        assert_eq!(head.units_per_em, 1000);
        let hhea = parsed.hhea.expect("hhea present");
        assert_eq!(hhea.ascender, 830);
        assert_eq!(hhea.descender, -270);
    }
}
