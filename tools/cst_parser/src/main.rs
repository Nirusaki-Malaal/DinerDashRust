//! CST Parser - Parses Director .cst files to extract bitmap metadata and assets

use anyhow::Result;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitmapMetadata {
    pub cast_name: String,
    pub member_id: u32,
    pub member_name: String,
    pub width: u16,
    pub height: u16,
    pub bits_per_pixel: u16,
    pub pitch: u16,
    pub palette_id: i32,
    pub flags: u16,
    pub color_depth: u16,
    pub chunk_id: String,
    pub bitd_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CSTManifest {
    pub cast_name: String,
    pub bitmaps: Vec<BitmapMetadata>,
    pub total_bitd_chunks: usize,
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: cst_parser <input_cst_dir> <output_dir> [--extract-bitd]");
        std::process::exit(1);
    }

    let input_dir = PathBuf::from(&args[1]);
    let output_dir = PathBuf::from(&args[2]);
    let extract_bitd = args.contains(&"--extract-bitd".to_string());

    std::fs::create_dir_all(&output_dir)?;

    let mut all_manifests = Vec::new();

    for entry in WalkDir::new(&input_dir) {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("cst") {
            match parse_cst_file(path, &output_dir, extract_bitd) {
                Ok(manifest) => {
                    all_manifests.push(manifest);
                }
                Err(e) => {
                    eprintln!("Error parsing {}: {}", path.display(), e);
                }
            }
        }
    }

    let manifest_path = output_dir.join("cst_manifest.ron");
    let ron_str = ron::to_string(&all_manifests)?;
    std::fs::write(&manifest_path, ron_str)?;

    Ok(())
}

fn parse_cst_file(cst_path: &Path, output_dir: &Path, _extract_bitd: bool) -> Result<CSTManifest> {
    let data = std::fs::read(cst_path)?;
    let mut cursor = Cursor::new(&data);

    let mut fourcc = [0u8; 4];
    cursor.read_exact(&mut fourcc)?;
    let is_little_endian = &fourcc == b"XFIR";

    let _container_size = if is_little_endian {
        cursor.read_u32::<LittleEndian>()?
    } else {
        cursor.read_u32::<BigEndian>()?
    };

    let mut codec = [0u8; 4];
    cursor.read_exact(&mut codec)?;
    let codec_str: String = if is_little_endian {
        codec.iter().rev().map(|&b| b as char).collect()
    } else {
        codec.iter().map(|&b| b as char).collect()
    };

    if codec_str != "MV93" && codec_str != "MV94" && codec_str != "MC95" {
        anyhow::bail!("Unknown codec: {}", codec_str);
    }

    let cast_name = cst_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let mut bitmaps = Vec::new();
    let mut bitd_chunks = HashMap::new();
    let mut member_counter = 0u32;

    while (cursor.position() as usize) + 8 <= data.len() {
        let mut chunk_fourcc = [0u8; 4];
        if cursor.read_exact(&mut chunk_fourcc).is_err() {
            break;
        }

        let chunk_size = if is_little_endian {
            cursor.read_u32::<LittleEndian>()?
        } else {
            cursor.read_u32::<BigEndian>()?
        } as usize;

        let chunk_type: String = if is_little_endian {
            chunk_fourcc.iter().rev().map(|&b| b as char).collect()
        } else {
            chunk_fourcc.iter().map(|&b| b as char).collect()
        };

        let chunk_data_start = cursor.position() as usize;
        let chunk_data_end = (chunk_data_start + chunk_size).min(data.len());
        let padded_end = (chunk_data_end + (chunk_size & 1)).min(data.len());

        match chunk_type.as_str() {
            "CASt" => {
                member_counter += 1;
                let slice = &data[chunk_data_start..chunk_data_end];
                if let Some(bm) = parse_cast_member(&cast_name, member_counter, slice) {
                    bitmaps.push(bm);
                }
            }
            "BITD" => {
                let chunk_id = format!("BITD-{}", bitd_chunks.len());
                let bitd_data = data[chunk_data_start..chunk_data_end].to_vec();
                bitd_chunks.insert(chunk_id, bitd_data);
            }
            _ => {}
        }

        cursor.seek(SeekFrom::Start(padded_end as u64))?;
    }

    let manifest = CSTManifest {
        cast_name: cast_name.clone(),
        bitmaps,
        total_bitd_chunks: bitd_chunks.len(),
    };

    let manifest_path = output_dir.join(format!("{}_manifest.ron", cast_name));
    let ron_str = ron::to_string(&manifest)?;
    std::fs::write(&manifest_path, ron_str)?;

    Ok(manifest)
}

fn parse_cast_member(cast_name: &str, member_id: u32, data: &[u8]) -> Option<BitmapMetadata> {
    if data.len() < 12 {
        return None;
    }
    let member_type = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
    if member_type != 1 {
        return None;
    }
    let info_len = u32::from_be_bytes([data[4], data[5], data[6], data[7]]) as usize;
    let spec_len = u32::from_be_bytes([data[8], data[9], data[10], data[11]]) as usize;
    if 12 + info_len + spec_len > data.len() || spec_len < 22 {
        return None;
    }

    let spec = &data[12 + info_len..12 + info_len + spec_len];
    let raw_pitch = u16::from_be_bytes([spec[0], spec[1]]);
    let pitch = raw_pitch & 0x0FFF;
    let flags = raw_pitch >> 12;
    let top = i16::from_be_bytes([spec[2], spec[3]]);
    let left = i16::from_be_bytes([spec[4], spec[5]]);
    let bottom = i16::from_be_bytes([spec[6], spec[7]]);
    let right = i16::from_be_bytes([spec[8], spec[9]]);
    let width = (right - left).max(0) as u16;
    let height = (bottom - top).max(0) as u16;
    let bpp = if spec_len >= 24 && spec[23] != 0 {
        spec[23] as u16
    } else {
        1
    };
    let palette_id = if spec_len >= 28 {
        i16::from_be_bytes([spec[26], spec[27]]) as i32
    } else {
        -1
    };

    Some(BitmapMetadata {
        cast_name: cast_name.to_string(),
        member_id,
        member_name: format!("member_{}", member_id),
        width,
        height,
        bits_per_pixel: bpp,
        pitch,
        palette_id,
        flags,
        color_depth: bpp,
        chunk_id: format!("CASt-{}", member_id),
        bitd_size: 0,
    })
}