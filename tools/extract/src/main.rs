use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{info, warn};
use walkdir::WalkDir;

mod bitd;
mod cast;
mod edim;

#[derive(Parser)]
#[command(name = "diner-dash-extract")]
#[command(about = "Extract assets from Diner Dash Director game files")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Extract all assets from a directory of decompiled .cst chunk directories
    Extract {
        /// Input directory containing decompiled cast subdirectories (with chunks/)
        #[arg(short, long)]
        input: PathBuf,
        /// Output directory for extracted assets
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Generate manifest.ron from extracted assets directory
    Manifest {
        /// Input assets directory
        #[arg(short, long)]
        input: PathBuf,
        /// Output manifest.ron path
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Decode a single BITD chunk to PNG
    Bitd {
        #[arg(short, long)]
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(short, long)]
        width: u32,
        #[arg(short, long)]
        height: u32,
        #[arg(short, long, default_value = "8")]
        bpp: u16,
        #[arg(short, long)]
        pitch: Option<u16>,
        #[arg(long)]
        palette: Option<PathBuf>,
    },
    /// Decode a single ediM chunk to WAV
    Edim {
        #[arg(short, long)]
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedTexture {
    pub cast: String,
    pub chunk_id: i32,
    pub name: String,
    pub path: String,
    pub width: u32,
    pub height: u32,
    pub reg_x: i16,
    pub reg_y: i16,
    pub bpp: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedAudio {
    pub cast: String,
    pub chunk_id: i32,
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExtractedManifest {
    pub textures: Vec<ExtractedTexture>,
    pub audio: Vec<ExtractedAudio>,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Extract { input, output } => extract_all(&input, &output),
        Commands::Manifest { input, output } => generate_manifest(&input, &output),
        Commands::Bitd {
            input,
            output,
            width,
            height,
            bpp,
            pitch,
            palette,
        } => decode_bitd_cmd(&input, &output, width, height, bpp, pitch, palette.as_deref()),
        Commands::Edim { input, output } => decode_edim_cmd(&input, &output),
    }
}

fn sanitize_name(name: &str, fallback_id: i32) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return format!("member_{}", fallback_id);
    }
    let clean: String = trimmed
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();
    format!("{}_{}", clean, fallback_id)
}

fn extract_all(input_dir: &Path, output_dir: &Path) -> Result<()> {
    info!(
        "Extracting assets from {} to {}",
        input_dir.display(),
        output_dir.display()
    );

    std::fs::create_dir_all(output_dir.join("images"))?;
    std::fs::create_dir_all(output_dir.join("audio"))?;
    std::fs::create_dir_all(output_dir.join("data"))?;

    let mut manifest = ExtractedManifest::default();

    // Find all `chunks` directories inside `input_dir`
    let mut chunk_dirs = Vec::new();
    for entry in WalkDir::new(input_dir).max_depth(3) {
        let entry = entry?;
        if entry.file_type().is_dir() && entry.file_name() == "chunks" {
            chunk_dirs.push(entry.path().to_path_buf());
        }
    }
    chunk_dirs.sort();

    // Pre-collect global CLUT palettes by 1-based cast slot across all casts
    let mut global_clut_by_slot: HashMap<i16, Vec<u8>> = HashMap::new();
    for chunks_dir in &chunk_dirs {
        let mut key_entries = Vec::new();
        let mut cas_slots: Vec<(u32, i32)> = Vec::new();
        let mut clut_chunks: HashMap<i32, Vec<u8>> = HashMap::new();
        if let Ok(rd) = std::fs::read_dir(chunks_dir) {
            for entry in rd.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("bin") {
                    continue;
                }
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
                if stem.starts_with("KEY_-") {
                    if let Ok(data) = std::fs::read(&path) {
                        if let Ok(entries) = cast::parse_key_table(&data) {
                            key_entries.extend(entries);
                        }
                    }
                } else if stem.starts_with("CAS_-") {
                    if let Ok(data) = std::fs::read(&path) {
                        cas_slots.extend(cast::parse_cas_table(&data));
                    }
                } else if let Some(id_str) = stem.strip_prefix("CLUT-") {
                    if let Ok(id) = id_str.parse::<i32>() {
                        if let Ok(data) = std::fs::read(&path) {
                            clut_chunks.insert(id, cast::parse_clut(&data));
                        }
                    }
                }
            }
        }
        let cast_to_slot: HashMap<i32, u32> = cas_slots.into_iter().map(|(s, c)| (c, s)).collect();
        for k in key_entries {
            if k.fourcc == "CLUT" {
                if let (Some(&slot), Some(pal)) = (cast_to_slot.get(&k.cast_id), clut_chunks.get(&k.section_id)) {
                    global_clut_by_slot.insert(slot as i16, pal.clone());
                }
            }
        }
    }

    for chunks_dir in chunk_dirs {
        let cast_name = chunks_dir
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        extract_cast_chunks(&cast_name, &chunks_dir, output_dir, &global_clut_by_slot, &mut manifest)?;
    }

    let manifest_path = output_dir.join("data").join("manifest.ron");
    let ron_pretty = ron::ser::PrettyConfig::default();
    let ron_str = ron::ser::to_string_pretty(&manifest, ron_pretty)?;
    std::fs::write(&manifest_path, ron_str)?;

    println!(
        "Extracted {} PNG images and {} WAV sounds. Manifest written to {}",
        manifest.textures.len(),
        manifest.audio.len(),
        manifest_path.display()
    );

    Ok(())
}

fn extract_cast_chunks(
    cast_name: &str,
    chunks_dir: &Path,
    output_dir: &Path,
    global_clut_by_slot: &HashMap<i16, Vec<u8>>,
    manifest: &mut ExtractedManifest,
) -> Result<()> {
    // 1. Parse KEY_ and CAS_ tables if present
    let mut key_entries = Vec::new();
    let mut cas_slots: Vec<(u32, i32)> = Vec::new();
    let mut clut_chunks: HashMap<i32, Vec<u8>> = HashMap::new();
    let mut cast_files: Vec<(i32, PathBuf)> = Vec::new();

    for entry in std::fs::read_dir(chunks_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("bin") {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();

        if stem.starts_with("KEY_-") {
            let data = std::fs::read(&path)?;
            if let Ok(entries) = cast::parse_key_table(&data) {
                key_entries.extend(entries);
            }
        } else if stem.starts_with("CAS_-") {
            let data = std::fs::read(&path)?;
            cas_slots.extend(cast::parse_cas_table(&data));
        } else if let Some(id_str) = stem.strip_prefix("CLUT-") {
            if let Ok(id) = id_str.parse::<i32>() {
                let data = std::fs::read(&path)?;
                clut_chunks.insert(id, cast::parse_clut(&data));
            }
        } else if let Some(id_str) = stem.strip_prefix("CASt-") {
            if let Ok(id) = id_str.parse::<i32>() {
                cast_files.push((id, path));
            }
        }
    }

    cast_files.sort_by_key(|(id, _)| *id);

    let mut slot_to_cast: HashMap<u32, i32> = HashMap::new();
    let mut cast_to_slot: HashMap<i32, u32> = HashMap::new();
    for (slot, cid) in &cas_slots {
        slot_to_cast.insert(*slot, *cid);
        cast_to_slot.insert(*cid, *slot);
    }

    // Map cast_id -> list of (fourcc, section_id)
    let mut owner_map: HashMap<i32, Vec<(String, i32)>> = HashMap::new();
    for k in &key_entries {
        owner_map
            .entry(k.cast_id)
            .or_default()
            .push((k.fourcc.clone(), k.section_id));
    }

    // Map 1-based cast slot -> CLUT palette (starting with global CLUTs)
    let mut clut_by_slot: HashMap<i16, Vec<u8>> = global_clut_by_slot.clone();
    for (cid, children) in &owner_map {
        if let Some(&slot) = cast_to_slot.get(cid) {
            for (fourcc, sec_id) in children {
                if fourcc == "CLUT" {
                    if let Some(pal) = clut_chunks.get(sec_id) {
                        clut_by_slot.insert(slot as i16, pal.clone());
                    }
                }
            }
        }
    }

    // Default palette for this cast (first CLUT in cast if any)
    let default_cast_palette: Option<Vec<u8>> = clut_chunks
        .iter()
        .min_by_key(|(k, _)| *k)
        .map(|(_, v)| v.clone());

    let mut parsed_metas: HashMap<i32, cast::CastMemberMeta> = HashMap::new();
    for (cast_id, cast_path) in &cast_files {
        if let Ok(data) = std::fs::read(cast_path) {
            if let Ok(m) = cast::parse_cast_chunk(*cast_id, &data) {
                parsed_metas.insert(*cast_id, m);
            }
        }
    }

    let cast_img_dir = output_dir.join("images").join(cast_name);

    for (cast_id, _cast_path) in cast_files {
        let Some(meta) = parsed_metas.get(&cast_id).cloned() else {
            continue;
        };

        let children = owner_map.get(&cast_id);

        match cast::MemberType::from(meta.member_type) {
            cast::MemberType::Bitmap => {
                let Some(bm) = &meta.bitmap else { continue };
                if bm.width == 0 || bm.height == 0 {
                    continue;
                }

                let mut bitd_id = None;
                let mut alfa_id = None;
                let mut edim_id = None;

                if let Some(list) = children {
                    for (fourcc, sec_id) in list {
                        match fourcc.as_str() {
                            "BITD" => bitd_id = Some(*sec_id),
                            "ALFA" => alfa_id = Some(*sec_id),
                            "ediM" => edim_id = Some(*sec_id),
                            _ => {}
                        }
                    }
                }

                let alfa_bytes = alfa_id.and_then(|id| {
                    std::fs::read(chunks_dir.join(format!("ALFA-{}.bin", id))).ok()
                });

                let edim_bytes = edim_id.and_then(|id| {
                    std::fs::read(chunks_dir.join(format!("ediM-{}.bin", id))).ok()
                });

                let bitd_bytes = bitd_id.and_then(|id| {
                    std::fs::read(chunks_dir.join(format!("BITD-{}.bin", id))).ok()
                });

                // Resolve Director Mask ink (slot + 1 grayscale mask member) if no ALFA chunk
                let neighbor_mask = if alfa_bytes.is_none()
                    && !meta.name.to_ascii_lowercase().contains("mask")
                {
                    cast_to_slot
                        .get(&cast_id)
                        .and_then(|slot| slot_to_cast.get(&(slot + 1)))
                        .and_then(|next_cid| {
                            let next_meta = parsed_metas.get(next_cid)?;
                            let next_bm = next_meta.bitmap.as_ref()?;
                            if next_bm.bits_per_pixel != 8
                                || !next_meta.name.to_ascii_lowercase().contains("mask")
                            {
                                return None;
                            }
                            let next_bitd_id = owner_map
                                .get(next_cid)?
                                .iter()
                                .find(|(f, _)| f == "BITD")
                                .map(|(_, s)| *s)?;
                            let bytes = std::fs::read(
                                chunks_dir.join(format!("BITD-{}.bin", next_bitd_id)),
                            )
                            .ok()?;
                            Some((bytes, next_bm.width, next_bm.height, next_bm.pitch))
                        })
                } else {
                    None
                };

                let decoded_img = if let Some(ref jpeg) = edim_bytes {
                    let mask = alfa_bytes.as_deref().or_else(|| {
                        if bm.bits_per_pixel == 8 {
                            bitd_bytes.as_deref()
                        } else {
                            None
                        }
                    });
                    let mut img = bitd::decode_jpeg_with_alfa(jpeg, mask).ok();
                    if let (Some(ref mut im), Some((ref mbytes, mw, mh, mp))) =
                        (&mut img, &neighbor_mask)
                    {
                        let _ = bitd::apply_alfa_with_dims(im, mbytes, *mw, *mh, *mp);
                    }
                    img
                } else if let Some(ref bitd_raw) = bitd_bytes {
                    let pal = if bm.palette_id > 0 {
                        clut_by_slot
                            .get(&bm.palette_id)
                            .map(|v| v.as_slice())
                            .or(default_cast_palette.as_deref())
                    } else {
                        None
                    };
                    let mask_arg = if let Some(ref a) = alfa_bytes {
                        Some((a.as_slice(), bm.width, bm.height, ((bm.width + 1) & !1) as u16))
                    } else if let Some((ref mbytes, mw, mh, mp)) = neighbor_mask {
                        Some((mbytes.as_slice(), mw, mh, mp))
                    } else {
                        None
                    };
                    let mut img = bitd::decode_bitd_with_mask(
                        bitd_raw,
                        bm.width,
                        bm.height,
                        bm.bits_per_pixel,
                        bm.pitch,
                        pal,
                        mask_arg,
                    )
                    .ok();
                    if let (Some(im), Some((_, mw, mh, _))) = (&mut img, &neighbor_mask) {
                        let cw = im.width().min(*mw);
                        let ch = im.height().min(*mh);
                        if cw < im.width() || ch < im.height() {
                            *im = image::imageops::crop_imm(im, 0, 0, cw, ch).to_image();
                        }
                    }
                    img
                } else {
                    None
                };

                if let Some(img) = decoded_img {
                    std::fs::create_dir_all(&cast_img_dir)?;
                    let safe_name = sanitize_name(&meta.name, cast_id);
                    let rel_path = format!("images/{}/{}.png", cast_name, safe_name);
                    let out_path = output_dir.join(&rel_path);
                    if img.save(&out_path).is_ok() {
                        manifest.textures.push(ExtractedTexture {
                            cast: cast_name.to_string(),
                            chunk_id: cast_id,
                            name: if meta.name.is_empty() {
                                safe_name
                            } else {
                                meta.name.clone()
                            },
                            path: rel_path,
                            width: img.width(),
                            height: img.height(),
                            reg_x: bm.reg_x,
                            reg_y: bm.reg_y,
                            bpp: bm.bits_per_pixel,
                        });
                    }
                }
            }
            cast::MemberType::Sound => {
                let mut edim_id = None;
                if let Some(list) = children {
                    for (fourcc, sec_id) in list {
                        if fourcc == "ediM" {
                            edim_id = Some(*sec_id);
                        }
                    }
                }
                if let Some(sec_id) = edim_id {
                    let edim_path = chunks_dir.join(format!("ediM-{}.bin", sec_id));
                    if let Ok(edim_data) = std::fs::read(&edim_path) {
                        let safe_name = sanitize_name(&meta.name, cast_id);
                        let rel_path = format!("audio/{}.wav", safe_name);
                        let out_path = output_dir.join(&rel_path);
                        if edim::decode_edim_to_wav(&edim_data, &out_path).is_ok() {
                            manifest.audio.push(ExtractedAudio {
                                cast: cast_name.to_string(),
                                chunk_id: cast_id,
                                name: if meta.name.is_empty() {
                                    safe_name
                                } else {
                                    meta.name.clone()
                                },
                                path: rel_path,
                            });
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn generate_manifest(input_dir: &Path, output_path: &Path) -> Result<()> {
    if output_path.exists() {
        info!("Manifest already exists at {}", output_path.display());
        return Ok(());
    }
    let manifest = ExtractedManifest::default();
    let ron_str = ron::ser::to_string_pretty(&manifest, ron::ser::PrettyConfig::default())?;
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(output_path, ron_str)?;
    Ok(())
}

fn decode_bitd_cmd(
    input: &Path,
    output: &Path,
    width: u32,
    height: u32,
    bpp: u16,
    pitch: Option<u16>,
    palette_path: Option<&Path>,
) -> Result<()> {
    let data = std::fs::read(input)?;
    let pitch = pitch.unwrap_or_else(|| ((width * bpp as u32 + 7) / 8) as u16);

    let palette = if let Some(p) = palette_path {
        Some(cast::parse_clut(&std::fs::read(p)?))
    } else {
        None
    };

    let img = bitd::decode_bitd(&data, width, height, bpp, pitch, palette.as_deref())?;
    img.save(output)?;
    info!("Saved decoded bitmap: {}", output.display());
    Ok(())
}

fn decode_edim_cmd(input: &Path, output: &Path) -> Result<()> {
    let data = std::fs::read(input)?;
    edim::decode_edim_to_wav(&data, output)?;
    info!("Saved decoded audio: {}", output.display());
    Ok(())
}