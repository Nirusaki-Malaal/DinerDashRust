use macroquad::audio::{load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams, Sound};
use std::collections::HashMap;

pub struct Audio {
    sounds: HashMap<&'static str, Sound>,
    current: &'static str,
    pub music_enabled: bool,
    pub sfx_enabled: bool,
}

impl Audio {
    pub fn new() -> Self {
        Self {
            sounds: HashMap::new(),
            current: "",
            music_enabled: true,
            sfx_enabled: true,
        }
    }

    async fn load(&mut self, path: &'static str) -> bool {
        if !self.sounds.contains_key(path) {
            let Some(bytes) = crate::asset(path) else {
                return false;
            };
            match load_sound_from_bytes(bytes).await {
                Ok(sound) => {
                    self.sounds.insert(path, sound);
                }
                Err(e) => {
                    eprintln!("Audio {path}: {e:?}");
                    return false;
                }
            }
        }
        true
    }

    pub async fn music(&mut self, stage: u32) {
        if !self.music_enabled {
            if let Some(old) = self.sounds.get(self.current) {
                stop_sound(old);
            }
            self.current = "";
            return;
        }

        let path = match stage {
            0 => "assets/audio/music_intro_full.wav",
            1 => "assets/audio/music_j_full.wav",
            2 => "assets/audio/music_h_full.wav",
            3 => "assets/audio/music_i_full.wav",
            4 => "assets/audio/music_c_full.wav",
            _ => "assets/audio/music_i_full.wav",
        };

        if self.current == path {
            return;
        }

        if let Some(old) = self.sounds.get(self.current) {
            stop_sound(old);
        }
        self.current = "";

        if self.load(path).await {
            play_sound(
                &self.sounds[path],
                PlaySoundParams {
                    looped: true,
                    volume: 0.3,
                },
            );
            self.current = path;
        }
    }

    pub async fn effect(&mut self, path: &'static str) {
        if self.sfx_enabled && self.load(path).await {
            play_sound(
                &self.sounds[path],
                PlaySoundParams {
                    looped: false,
                    volume: 0.7,
                },
            );
        }
    }

    pub fn set_music(&mut self, enabled: bool) {
        self.music_enabled = enabled;
        if !enabled {
            if let Some(old) = self.sounds.get(self.current) {
                stop_sound(old);
            }
            self.current = "";
        }
    }

    pub fn set_sfx(&mut self, enabled: bool) {
        self.sfx_enabled = enabled;
    }
}
