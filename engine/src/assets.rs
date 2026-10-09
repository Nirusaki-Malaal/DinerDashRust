//! Asset management system

use anyhow::{Context, Result};
use image::{ImageBuffer, Rgba};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

use crate::math::{Rect, Vec2, UVec2};

/// Texture ID handle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TextureId(pub u32);

/// Audio ID handle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AudioId(pub u32);

/// Font ID handle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FontId(pub u32);

/// Texture region within an atlas
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TextureRegion {
    pub texture: TextureId,
    pub uv_min: Vec2,
    pub uv_max: Vec2,
    pub size: UVec2,
}

impl TextureRegion {
    pub fn new(texture: TextureId, uv_min: Vec2, uv_max: Vec2, size: UVec2) -> Self {
        Self { texture, uv_min, uv_max, size }
    }
    
    pub fn uv_rect(&self) -> [Vec2; 4] {
        [
            self.uv_min,
            Vec2::new(self.uv_max.x, self.uv_min.y),
            self.uv_max,
            Vec2::new(self.uv_min.x, self.uv_max.y),
        ]
    }
}

/// Sprite frame data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteFrame {
    pub region: TextureRegion,
    pub pivot: Vec2, // 0-1 normalized
    pub duration_ms: u32,
}

/// Animation definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Animation {
    pub name: String,
    pub frames: Vec<SpriteFrame>,
    pub loop_mode: LoopMode,
    pub fps: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LoopMode {
    Once,
    Loop,
    PingPong,
}

/// Texture atlas entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasEntry {
    pub name: String,
    pub region: TextureRegion,
    pub frames: Vec<SpriteFrame>,
}

/// Complete texture atlas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureAtlas {
    pub texture_id: TextureId,
    pub width: u32,
    pub height: u32,
    pub entries: HashMap<String, AtlasEntry>,
}

impl TextureAtlas {
    pub fn get(&self, name: &str) -> Option<&AtlasEntry> {
        self.entries.get(name)
    }
    
    pub fn get_frame(&self, name: &str, frame: usize) -> Option<&SpriteFrame> {
        self.entries.get(name)?.frames.get(frame)
    }
}

/// Asset manifest for loading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetManifest {
    pub textures: Vec<TextureManifestEntry>,
    pub audio: Vec<AudioManifestEntry>,
    pub fonts: Vec<FontManifestEntry>,
    pub atlases: Vec<AtlasManifestEntry>,
    pub levels: Vec<LevelManifestEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureManifestEntry {
    pub name: String,
    pub path: PathBuf,
    pub atlas: Option<String>, // If part of an atlas
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioManifestEntry {
    pub name: String,
    pub path: PathBuf,
    pub loop_: bool,
    pub stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontManifestEntry {
    pub name: String,
    pub path: PathBuf,
    pub char_map: String,
    pub size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasManifestEntry {
    pub name: String,
    pub texture: String, // References texture manifest entry
    pub entries: Vec<AtlasSpriteEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasSpriteEntry {
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub pivot_x: f32,
    pub pivot_y: f32,
    pub frames: Vec<AtlasFrameEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasFrameEntry {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub duration_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelManifestEntry {
    pub id: String,
    pub path: PathBuf,
}

/// Asset manager - handles loading and caching
pub struct AssetManager {
    textures: HashMap<String, TextureId>,
    texture_data: Vec<TextureData>,
    audio: HashMap<String, AudioId>,
    audio_data: Vec<AudioData>,
    fonts: HashMap<String, FontId>,
    font_data: Vec<FontData>,
    atlases: HashMap<String, TextureAtlas>,
    next_texture_id: u32,
    next_audio_id: u32,
    next_font_id: u32,
}

#[derive(Debug, Clone)]
pub struct TextureData {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>, // RGBA8
    #[cfg(feature = "native")]
    pub gpu_texture: Option<wgpu::Texture>,
    #[cfg(not(feature = "native"))]
    pub gpu_texture: Option<()>,
}

#[derive(Debug, Clone)]
pub struct AudioData {
    pub sample_rate: u32,
    pub channels: u16,
    pub samples: Vec<i16>, // Interleaved
    #[cfg(feature = "native")]
    pub gpu_buffer: Option<()>,
    #[cfg(not(feature = "native"))]
    pub gpu_buffer: Option<()>,
}

#[derive(Debug, Clone)]
pub struct FontData {
    pub texture: TextureId,
    pub char_map: String,
    pub glyphs: HashMap<char, GlyphMetrics>,
    pub size: u32,
    pub line_height: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GlyphMetrics {
    pub uv_min: Vec2,
    pub uv_max: Vec2,
    pub size: UVec2,
    pub bearing: Vec2,
    pub advance: f32,
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            texture_data: Vec::new(),
            audio: HashMap::new(),
            audio_data: Vec::new(),
            fonts: HashMap::new(),
            font_data: Vec::new(),
            atlases: HashMap::new(),
            next_texture_id: 1,
            next_audio_id: 1,
            next_font_id: 1,
        }
    }
    
    /// Load manifest and all assets
    pub fn load_manifest(&mut self, manifest_path: &Path) -> Result<()> {
        let manifest_str = std::fs::read_to_string(manifest_path)
            .context("Failed to read manifest")?;
        let manifest: AssetManifest = ron::from_str(&manifest_str)
            .context("Failed to parse manifest")?;
        
        let base_dir = manifest_path.parent().unwrap_or(Path::new("."));
        
        // Load textures
        for entry in &manifest.textures {
            let path = base_dir.join(&entry.path);
            self.load_texture(&entry.name, &path)?;
        }
        
        // Build atlases
        for atlas_entry in &manifest.atlases {
            let texture_id = *self.textures.get(&atlas_entry.texture)
                .context(format!("Texture not found for atlas: {}", atlas_entry.texture))?;
            self.build_atlas(&atlas_entry.name, texture_id, &atlas_entry.entries)?;
        }
        
        // Load audio
        for entry in &manifest.audio {
            let path = base_dir.join(&entry.path);
            self.load_audio(&entry.name, &path, entry.loop_)?;
        }
        
        // Load fonts
        for entry in &manifest.fonts {
            let path = base_dir.join(&entry.path);
            self.load_font(&entry.name, &path, &entry.char_map, entry.size)?;
        }
        
        info!("Loaded {} textures, {} audio, {} fonts, {} atlases",
            self.textures.len(), self.audio.len(), self.fonts.len(), self.atlases.len());
        
        Ok(())
    }
    
    /// Load a single texture
    pub fn load_texture(&mut self, name: &str, path: &Path) -> Result<TextureId> {
        let img = image::open(path)
            .context(format!("Failed to load texture: {:?}", path))?
            .to_rgba8();
        
        let (width, height) = img.dimensions();
        let pixels = img.into_raw();
        
        let id = TextureId(self.next_texture_id);
        self.next_texture_id += 1;
        
        self.textures.insert(name.to_string(), id);
        self.texture_data.push(TextureData {
            width,
            height,
            pixels,
            #[cfg(not(target_arch = "wasm32"))]
            gpu_texture: None,
            #[cfg(target_arch = "wasm32")]
            gpu_texture: None,
        });
        
        info!("Loaded texture: {} ({}x{})", name, width, height);
        Ok(id)
    }
    
    /// Build texture atlas from individual sprites
    pub fn build_atlas(
        &mut self,
        name: &str,
        texture_id: TextureId,
        entries: &[AtlasSpriteEntry],
    ) -> Result<()> {
        let texture_idx = (texture_id.0 - 1) as usize;
        let texture = &self.texture_data[texture_idx];
        
        let mut atlas_entries = HashMap::new();
        
        for entry in entries {
            let uv_min = Vec2::new(
                entry.x as f32 / texture.width as f32,
                entry.y as f32 / texture.height as f32,
            );
            let uv_max = Vec2::new(
                (entry.x + entry.width) as f32 / texture.width as f32,
                (entry.y + entry.height) as f32 / texture.height as f32,
            );
            
            let region = TextureRegion::new(
                texture_id,
                uv_min,
                uv_max,
                UVec2::new(entry.width, entry.height),
            );
            
            let mut frames = vec![SpriteFrame {
                region,
                pivot: Vec2::new(entry.pivot_x, entry.pivot_y),
                duration_ms: 100,
            }];
            
            // Add additional frames if specified
            for frame in &entry.frames {
                let frame_uv_min = Vec2::new(
                    frame.x as f32 / texture.width as f32,
                    frame.y as f32 / texture.height as f32,
                );
                let frame_uv_max = Vec2::new(
                    (frame.x + frame.width) as f32 / texture.width as f32,
                    (frame.y + frame.height) as f32 / texture.height as f32,
                );
                
                frames.push(SpriteFrame {
                    region: TextureRegion::new(
                        texture_id,
                        frame_uv_min,
                        frame_uv_max,
                        UVec2::new(frame.width, frame.height),
                    ),
                    pivot: Vec2::new(entry.pivot_x, entry.pivot_y),
                    duration_ms: frame.duration_ms,
                });
            }
            
            atlas_entries.insert(entry.name.clone(), AtlasEntry {
                name: entry.name.clone(),
                region,
                frames,
            });
        }
        
        let atlas = TextureAtlas {
            texture_id,
            width: texture.width,
            height: texture.height,
            entries: atlas_entries,
        };
        
        self.atlases.insert(name.to_string(), atlas);
        info!("Built atlas: {} with {} sprites", name, entries.len());
        
        Ok(())
    }
    
    /// Load audio file
    pub fn load_audio(&mut self, name: &str, path: &Path, loop_: bool) -> Result<AudioId> {
        // For now, just register - actual decoding happens in WASM via Web Audio API
        let id = AudioId(self.next_audio_id);
        self.next_audio_id += 1;
        
        self.audio.insert(name.to_string(), id);
        self.audio_data.push(AudioData {
            sample_rate: 44100,
            channels: 2,
            samples: Vec::new(),
            gpu_buffer: None,
        });
        
        info!("Registered audio: {}", name);
        Ok(id)
    }
    
    /// Load bitmap font
    pub fn load_font(
        &mut self,
        name: &str,
        path: &Path,
        char_map: &str,
        size: u32,
    ) -> Result<FontId> {
        // Load the font texture
        let texture_id = self.load_texture(&format!("{}_font", name), path)?;
        
        let texture_idx = (texture_id.0 - 1) as usize;
        let texture = &self.texture_data[texture_idx];
        
        // Simple monospace font layout - chars in a grid
        let chars_per_row = 16;
        let char_width = texture.width / chars_per_row;
        let char_height = texture.height / (char_map.len() as u32 / chars_per_row + 1);
        
        let mut glyphs = HashMap::new();
        for (i, ch) in char_map.chars().enumerate() {
            let idx = i as u32;
            let row = idx / chars_per_row;
            let col = idx % chars_per_row;
            
            let uv_min = Vec2::new(
                col as f32 * char_width as f32 / texture.width as f32,
                row as f32 * char_height as f32 / texture.height as f32,
            );
            let uv_max = Vec2::new(
                (col + 1) as f32 * char_width as f32 / texture.width as f32,
                (row + 1) as f32 * char_height as f32 / texture.height as f32,
            );
            
            glyphs.insert(ch, GlyphMetrics {
                uv_min,
                uv_max,
                size: UVec2::new(char_width, char_height),
                bearing: Vec2::ZERO,
                advance: char_width as f32,
            });
        }
        
        let id = FontId(self.next_font_id);
        self.next_font_id += 1;
        let glyph_count = glyphs.len();
        
        self.fonts.insert(name.to_string(), id);
        self.font_data.push(FontData {
            texture: texture_id,
            char_map: char_map.to_string(),
            glyphs,
            size,
            line_height: char_height as f32 * 1.2,
        });
        
        info!("Loaded font: {} ({} glyphs)", name, glyph_count);
        Ok(id)
    }
    
    /// Get texture ID by name
    pub fn get_texture_id(&self, name: &str) -> Option<TextureId> {
        self.textures.get(name).copied()
    }
    
    /// Get texture data by ID
    pub fn get_texture_data(&self, id: TextureId) -> Option<&TextureData> {
        self.texture_data.get((id.0 - 1) as usize)
    }
    
    /// Get atlas by name
    pub fn get_atlas(&self, name: &str) -> Option<&TextureAtlas> {
        self.atlases.get(name)
    }
    
    /// Get mutable atlas by name
    pub fn get_atlas_mut(&mut self, name: &str) -> Option<&mut TextureAtlas> {
        self.atlases.get_mut(name)
    }
    
    /// Get audio ID by name
    pub fn get_audio_id(&self, name: &str) -> Option<AudioId> {
        self.audio.get(name).copied()
    }
    
    /// Get font ID by name
    pub fn get_font_id(&self, name: &str) -> Option<FontId> {
        self.fonts.get(name).copied()
    }
    
    /// Get font data by ID
    pub fn get_font_data(&self, id: FontId) -> Option<&FontData> {
        self.font_data.get((id.0 - 1) as usize)
    }
    
    /// Upload all textures to GPU
    #[cfg(feature = "native")]
    pub fn upload_to_gpu(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        for texture_data in &mut self.texture_data {
            if texture_data.gpu_texture.is_none() {
                let size = wgpu::Extent3d {
                    width: texture_data.width,
                    height: texture_data.height,
                    depth_or_array_layers: 1,
                };
                
                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: None,
                    size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                });
                
                queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture: &texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    &texture_data.pixels,
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * texture_data.width),
                        rows_per_image: Some(texture_data.height),
                    },
                    size,
                );
                
                texture_data.gpu_texture = Some(texture);
            }
        }
    }
}

impl Default for AssetManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_asset_manager_creation() {
        let manager = AssetManager::new();
        assert_eq!(manager.textures.len(), 0);
    }
    
    #[test]
    fn test_texture_region() {
        let region = TextureRegion::new(
            TextureId(1),
            Vec2::new(0.0, 0.0),
            Vec2::new(0.5, 0.5),
            UVec2::new(64, 64),
        );
        assert_eq!(region.size.x, 64);
    }
}