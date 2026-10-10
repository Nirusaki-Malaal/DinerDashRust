//! Web Audio & HTMLAudioElement Sound System for Diner Dash WASM

use std::collections::HashMap;
use wasm_bindgen::JsValue;
use web_sys::{AudioBuffer, AudioContext, GainNode, HtmlAudioElement, OscillatorType};

pub struct WebAudioManager {
    context: Option<AudioContext>,
    master_gain: Option<GainNode>,
    buffers: HashMap<String, AudioBuffer>,
    music_el: Option<HtmlAudioElement>,
    current_music_stage: u32,
    pub muted: bool,
    pub master_volume: f32,
}

impl WebAudioManager {
    pub fn new() -> Self {
        let context = AudioContext::new().ok();
        let master_gain = context.as_ref().and_then(|ctx| {
            let gain = ctx.create_gain().ok()?;
            let _ = gain.connect_with_audio_node(&ctx.destination());
            Some(gain)
        });

        Self {
            context,
            master_gain,
            buffers: HashMap::new(),
            music_el: None,
            current_music_stage: 0,
            muted: false,
            master_volume: 0.7,
        }
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
        if let Some(gain) = &self.master_gain {
            gain.gain().set_value(if self.muted { 0.0 } else { self.master_volume });
        }
        if let Some(music) = &self.music_el {
            music.set_volume(if self.muted { 0.0 } else { (self.master_volume * 0.45) as f64 });
        }
    }

    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
        if let Some(music) = &self.music_el {
            music.set_muted(muted);
        }
    }

    pub fn ensure_menu_music(&mut self) {
        self.ensure_music_track(99, "assets/audio/music_intro_full.wav?v=3");
    }

    pub fn ensure_stage_music(&mut self, stage: u32) {
        let track_url = match stage {
            1 => "assets/audio/music_j_full.wav?v=3",
            2 => "assets/audio/music_c_full.wav?v=3",
            3 => "assets/audio/music_i_full.wav?v=3",
            4 => "assets/audio/music_c_full.wav?v=3",
            5 => "assets/audio/music_j_full.wav?v=3",
            _ => "assets/audio/music_j_full.wav?v=3",
        };
        self.ensure_music_track(stage, track_url);
    }

    fn ensure_music_track(&mut self, track_id: u32, track_url: &str) {
        if let Some(ctx) = &self.context {
            let _ = ctx.resume();
        }
        if self.muted {
            return;
        }
        if self.current_music_stage == track_id {
            if let Some(music) = &self.music_el {
                if music.network_state() != 3 {
                    if music.paused() {
                        let _ = music.play();
                    }
                    return;
                }
            }
        }
        if let Some(old) = self.music_el.take() {
            let _ = old.pause();
        }
        if let Ok(audio) = HtmlAudioElement::new_with_src(track_url) {
            audio.set_loop(true);
            audio.set_volume((self.master_volume * 0.45) as f64);
            let _ = audio.play();
            self.music_el = Some(audio);
            self.current_music_stage = track_id;
        }
    }

    pub fn play_tone(&self, freq_hz: f32, duration_sec: f32, osc_type: OscillatorType) {
        if self.muted {
            return;
        }
        let (Some(ctx), Some(master)) = (&self.context, &self.master_gain) else {
            return;
        };
        let _ = ctx.resume();
        let Ok(osc) = ctx.create_oscillator() else {
            return;
        };
        let Ok(gain) = ctx.create_gain() else {
            return;
        };
        osc.set_type(osc_type);
        osc.frequency().set_value(freq_hz);

        let now = ctx.current_time();
        gain.gain().set_value(self.master_volume * 0.25);
        let _ = gain
            .gain()
            .exponential_ramp_to_value_at_time(0.001, now + duration_sec as f64);

        let _ = osc.connect_with_audio_node(&gain);
        let _ = gain.connect_with_audio_node(master);
        let _ = osc.start();
        let _ = osc.stop_with_when(now + duration_sec as f64);
    }

    pub fn play_sfx(&self, name: &str) {
        if self.muted {
            return;
        }
        if let (Some(ctx), Some(master), Some(buffer)) =
            (&self.context, &self.master_gain, self.buffers.get(name))
        {
            let _ = ctx.resume();
            if let Ok(source) = ctx.create_buffer_source() {
                source.set_buffer(Some(buffer));
                let _ = source.connect_with_audio_node(master);
                let _ = source.start();
                return;
            }
        }

        let wav_path = match name {
            "rollover" | "hover" => "assets/audio/sfx_rollover_1_316.wav?v=3",
            "rollover2" => "assets/audio/sfx_rollover_2_240.wav?v=3",
            "rollover3" => "assets/audio/sfx_rollover_3_239.wav?v=3",
            "select" | "click" => "assets/audio/sfx_food_ready_1_snd_252.wav?v=3",
            "menu_down" => "assets/audio/sfx_menu_down_2_246.wav?v=3",
            "seat" => "assets/audio/sfx_seat_people_snd_243.wav?v=3",
            "order" => "assets/audio/sfx_pencil_write_2_730.wav?v=3",
            "deliver_order" => "assets/audio/sfx_deliver_order_1_snd_263.wav?v=3",
            "serve" => "assets/audio/sfx_deliver_food_1_snd_265.wav?v=3",
            "pickup" => "assets/audio/sfx_pickup_food_1_snd_244.wav?v=3",
            "done_eating" => "assets/audio/sfx_done_eating_1_snd_338.wav?v=3",
            "cash" | "check" => "assets/audio/sfx_bring_check_1_snd_270.wav?v=3",
            "clean" => "assets/audio/sfx_clear_dishes_1_snd_269.wav?v=3",
            "dish_dropoff" => "assets/audio/sfx_dish_dropoff_1_snd_259.wav?v=3",
            "drink_pickup" => "assets/audio/sfx_get_drinks_1_snd_250.wav?v=3",
            "drink_dropoff" => "assets/audio/sfx_dropoff_drinks_1_snd_254.wav?v=3",
            "angry" | "lose" => "assets/audio/sfx_lose_heart_1_238.wav?v=3",
            "win" | "build" => "assets/audio/sfx_build_rest_331.wav?v=3",
            "goddess" => "assets/audio/sfx_goddess_appear_42.wav?v=3",
            "heart" => "assets/audio/sfx_gain_heart_1_247.wav?v=3",
            "arrive" => "assets/audio/sfx_party_arrive_1_snd_245.wav?v=3",
            "ready" => "assets/audio/sfx_food_ready_1_snd_252.wav?v=3",
            _ => "assets/audio/sfx_rollover_1_316.wav?v=3",
        };

        if let Ok(audio) = HtmlAudioElement::new_with_src(wav_path) {
            audio.set_volume(self.master_volume as f64);
            let _ = audio.play();
        }
    }

    pub fn register_buffer(&mut self, name: String, buffer: AudioBuffer) -> Result<(), JsValue> {
        self.buffers.insert(name, buffer);
        Ok(())
    }
}

impl Default for WebAudioManager {
    fn default() -> Self {
        Self::new()
    }
}
