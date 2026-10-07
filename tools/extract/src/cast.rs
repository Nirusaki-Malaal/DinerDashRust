//! Director CASt, KEY_, CAS_, and CLUT chunk parsers
//!
//! Parses raw chunk `.bin` files dumped by ProjectorRays (`--dump-chunks`):
//! - `KEY_-*.bin`: Little-endian section ownership table mapping `CASt` chunk IDs to
//!   child media chunks (`BITD`, `ALFA`, `ediM`, `CLUT`, `STXT`, `XMED`).
//! - `CAS_-*.bin`: Big-endian cast member index table mapping member slot index -> `CASt` chunk ID.
//! - `CASt-*.bin`: Big-endian CastMemberChunk (Director v500+ format) containing
//!   `CastInfoChunk` (member name, script text) and `specificData` (bitmap dimensions,
//!   pitch, registration point, bit depth, palette reference).
//! - `CLUT-*.bin`: Big-endian Mac QuickDraw color lookup table (256 * 6 bytes -> 256 * 3 RGB).

use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemberType {
    Null,
    Bitmap,
    FilmLoop,
    Text,
    Palette,
    Picture,
    Sound,
    Button,
    Shape,
    Movie,
    DigitalVideo,
    Script,
    Rte,
    Unknown(u32),
}

impl From<u32> for MemberType {
    fn from(v: u32) -> Self {
        match v {
            0 => MemberType::Null,
            1 => MemberType::Bitmap,
            2 => MemberType::FilmLoop,
            3 => MemberType::Text,
            4 => MemberType::Palette,
            5 => MemberType::Picture,
            6 => MemberType::Sound,
            7 => MemberType::Button,
            8 => MemberType::Shape,
            9 => MemberType::Movie,
            10 => MemberType::DigitalVideo,
            11 => MemberType::Script,
            12 => MemberType::Rte,
            other => MemberType::Unknown(other),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BitmapInfo {
    pub pitch: u16,
    pub initial_top: i16,
    pub initial_left: i16,
    pub initial_bottom: i16,
    pub initial_right: i16,
    pub width: u32,
    pub height: u32,
    pub reg_x: i16,
    pub reg_y: i16,
    pub bits_per_pixel: u16,
    pub palette_cast: i16,
    pub palette_id: i16,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CastMemberMeta {
    pub chunk_id: i32,
    pub member_slot: Option<u32>,
    pub member_type: u32,
    pub name: String,
    pub bitmap: Option<BitmapInfo>,
}

#[derive(Debug, Clone)]
pub struct KeyEntry {
    pub section_id: i32,
    pub cast_id: i32,
    pub fourcc: String,
}

/// Parse `KEY_-*.bin` chunk (little-endian table of 12-byte ownership records)
pub fn parse_key_table(data: &[u8]) -> Result<Vec<KeyEntry>> {
    if data.len() < 12 {
        anyhow::bail!("KEY_ chunk too small: {} bytes", data.len());
    }
    let entry_size = u16::from_le_bytes([data[0], data[1]]) as usize;
    let entry_count = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
    let used_count = u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize;

    let count = used_count.min(entry_count);
    let stride = if entry_size >= 12 { entry_size } else { 12 };
    let mut entries = Vec::with_capacity(count);

    let mut offset = 12;
    for _ in 0..count {
        if offset + 12 > data.len() {
            break;
        }
        let section_id = i32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        let cast_id = i32::from_le_bytes([
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]);
        // In XFIR (little-endian), FourCC bytes are reversed
        let fourcc_bytes = [
            data[offset + 11],
            data[offset + 10],
            data[offset + 9],
            data[offset + 8],
        ];
        let fourcc = String::from_utf8_lossy(&fourcc_bytes).to_string();
        entries.push(KeyEntry {
            section_id,
            cast_id,
            fourcc,
        });
        offset += stride;
    }

    Ok(entries)
}

/// Parse `CAS_-*.bin` chunk (big-endian array of i32 CASt chunk section IDs)
pub fn parse_cas_table(data: &[u8]) -> Vec<(u32, i32)> {
    let mut out = Vec::new();
    for (idx, chunk) in data.chunks_exact(4).enumerate() {
        let section_id = i32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        if section_id > 0 {
            out.push(((idx + 1) as u32, section_id));
        }
    }
    out
}

/// Parse `CASt-<id>.bin` chunk (Director v500+ big-endian layout)
pub fn parse_cast_chunk(chunk_id: i32, data: &[u8]) -> Result<CastMemberMeta> {
    if data.len() < 12 {
        anyhow::bail!("CASt-{} too small ({} bytes)", chunk_id, data.len());
    }

    let member_type = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
    let info_len = u32::from_be_bytes([data[4], data[5], data[6], data[7]]) as usize;
    let specific_len = u32::from_be_bytes([data[8], data[9], data[10], data[11]]) as usize;

    if 12 + info_len > data.len() {
        anyhow::bail!("CASt-{} info_len {} exceeds chunk size {}", chunk_id, info_len, data.len());
    }

    let info_slice = &data[12..12 + info_len];
    let name = if info_len >= 22 {
        parse_cast_info_name(info_slice).unwrap_or_default()
    } else {
        String::new()
    };

    let spec_start = 12 + info_len;
    let spec_end = (spec_start + specific_len).min(data.len());
    let spec_slice = &data[spec_start..spec_end];

    let bitmap = if member_type == 1 && spec_slice.len() >= 22 {
        Some(parse_bitmap_specific(spec_slice))
    } else {
        None
    };

    Ok(CastMemberMeta {
        chunk_id,
        member_slot: None,
        member_type,
        name,
        bitmap,
    })
}

fn parse_cast_info_name(info: &[u8]) -> Option<String> {
    if info.len() < 22 {
        return None;
    }
    let data_offset = u32::from_be_bytes([info[0], info[1], info[2], info[3]]) as usize;
    if data_offset + 2 > info.len() {
        return None;
    }
    let offset_table_len = u16::from_be_bytes([info[data_offset], info[data_offset + 1]]) as usize;
    if offset_table_len < 2 {
        return None;
    }
    let table_start = data_offset + 2;
    let table_bytes = offset_table_len * 4;
    if table_start + table_bytes + 4 > info.len() {
        return None;
    }

    let item1_offset = u32::from_be_bytes([
        info[table_start + 4],
        info[table_start + 5],
        info[table_start + 6],
        info[table_start + 7],
    ]) as usize;

    let items_len_pos = table_start + table_bytes;
    let items_len = u32::from_be_bytes([
        info[items_len_pos],
        info[items_len_pos + 1],
        info[items_len_pos + 2],
        info[items_len_pos + 3],
    ]) as usize;

    let item1_end = if offset_table_len > 2 {
        u32::from_be_bytes([
            info[table_start + 8],
            info[table_start + 9],
            info[table_start + 10],
            info[table_start + 11],
        ]) as usize
    } else {
        items_len
    };

    let list_start = items_len_pos + 4;
    if item1_end <= item1_offset || list_start + item1_end > info.len() {
        return None;
    }

    let item_slice = &info[list_start + item1_offset..list_start + item1_end];
    if item_slice.is_empty() {
        return None;
    }
    let str_len = (item_slice[0] as usize).min(item_slice.len() - 1);
    Some(String::from_utf8_lossy(&item_slice[1..1 + str_len]).to_string())
}

fn parse_bitmap_specific(spec: &[u8]) -> BitmapInfo {
    let raw_pitch = u16::from_be_bytes([spec[0], spec[1]]);
    let pitch = raw_pitch & 0x0FFF;
    let initial_top = i16::from_be_bytes([spec[2], spec[3]]);
    let initial_left = i16::from_be_bytes([spec[4], spec[5]]);
    let initial_bottom = i16::from_be_bytes([spec[6], spec[7]]);
    let initial_right = i16::from_be_bytes([spec[8], spec[9]]);

    let width = (initial_right as i32 - initial_left as i32).max(0) as u32;
    let height = (initial_bottom as i32 - initial_top as i32).max(0) as u32;

    let reg_y = i16::from_be_bytes([spec[18], spec[19]]);
    let reg_x = i16::from_be_bytes([spec[20], spec[21]]);

    let (bits_per_pixel, palette_cast, palette_id) = if spec.len() >= 28 {
        let bpp = if spec[23] == 0 { 1 } else { spec[23] as u16 };
        let p_cast = i16::from_be_bytes([spec[24], spec[25]]);
        let p_id = i16::from_be_bytes([spec[26], spec[27]]);
        (bpp, p_cast, p_id)
    } else if spec.len() >= 24 {
        let bpp = if spec[23] == 0 { 1 } else { spec[23] as u16 };
        (bpp, 0, -1)
    } else {
        (1, 0, -1)
    };

    BitmapInfo {
        pitch,
        initial_top,
        initial_left,
        initial_bottom,
        initial_right,
        width,
        height,
        reg_x,
        reg_y,
        bits_per_pixel,
        palette_cast,
        palette_id,
    }
}

/// Convert a 1536-byte Mac `CLUT` chunk (256 * 6 bytes BE u16 RGB) into 768-byte RGB888
pub fn parse_clut(data: &[u8]) -> Vec<u8> {
    if data.len() >= 1536 {
        let mut rgb = Vec::with_capacity(768);
        for i in 0..256 {
            let base = i * 6;
            rgb.push(data[base]);
            rgb.push(data[base + 2]);
            rgb.push(data[base + 4]);
        }
        rgb
    } else if data.len() >= 768 {
        data[..768].to_vec()
    } else {
        let mut rgb = vec![0u8; 768];
        for (i, chunk) in data.chunks(6).enumerate().take(256) {
            if chunk.len() >= 5 {
                rgb[i * 3] = chunk[0];
                rgb[i * 3 + 1] = chunk[2];
                rgb[i * 3 + 2] = chunk[4];
            }
        }
        rgb
    }
}

/// Standard Macintosh 256-color system palette fallback (RGB888, 768 bytes)
pub fn mac_system_palette() -> [u8; 768] {
    let mut pal = [0u8; 768];
    let mut idx = 0;
    // First 215 entries: 6x6x6 color cube (255 down to 0 in steps of 51)
    let levels = [0xFFu8, 0xCC, 0x99, 0x66, 0x33, 0x00];
    for &r in &levels {
        for &g in &levels {
            for &b in &levels {
                if idx < 215 {
                    pal[idx * 3] = r;
                    pal[idx * 3 + 1] = g;
                    pal[idx * 3 + 2] = b;
                    idx += 1;
                }
            }
        }
    }
    // Ramp entries (red, green, blue, gray ramps)
    let ramp = [0xEEu8, 0xDD, 0xBB, 0xAA, 0x88, 0x77, 0x55, 0x44, 0x22, 0x11];
    for &v in &ramp {
        if idx < 256 {
            pal[idx * 3] = v;
            idx += 1;
        }
    }
    for &v in &ramp {
        if idx < 256 {
            pal[idx * 3 + 1] = v;
            idx += 1;
        }
    }
    for &v in &ramp {
        if idx < 256 {
            pal[idx * 3 + 2] = v;
            idx += 1;
        }
    }
    for &v in &ramp {
        if idx < 256 {
            pal[idx * 3] = v;
            pal[idx * 3 + 1] = v;
            pal[idx * 3 + 2] = v;
            idx += 1;
        }
    }
    // Entry 255 is black (0, 0, 0)
    pal
}
