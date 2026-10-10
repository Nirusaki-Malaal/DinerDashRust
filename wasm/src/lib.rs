//! Diner Dash WASM Entry Point & Browser Runtime

pub mod audio;
pub mod input;
pub mod renderer;
pub mod save;

use audio::WebAudioManager;
use diner_dash_engine::game::{GameConfig, GameManager, GameMode, TableId};
use diner_dash_engine::math::Vec2;
use diner_dash_engine::save::SaveManager;
use input::InputManager;
use renderer::WebRenderer;
use save::WebPersistence;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;

#[wasm_bindgen]
pub struct DinerDashApp {
    game: GameManager,
    save_mgr: SaveManager,
    renderer: WebRenderer,
    audio: WebAudioManager,
    input: InputManager,
    autosave_accum: f32,
    paused: bool,
    has_session: bool,
}

#[wasm_bindgen]
impl DinerDashApp {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> Result<DinerDashApp, JsValue> {
        console_error_panic_hook::set_once();
        WebPersistence::sync_from_idb_to_local_storage();

        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
        let document = window
            .document()
            .ok_or_else(|| JsValue::from_str("No document"))?;
        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or_else(|| JsValue::from_str(&format!("Canvas #{} not found", canvas_id)))?
            .dyn_into::<HtmlCanvasElement>()?;

        let mut renderer = WebRenderer::new(canvas)?;
        let audio = WebAudioManager::new();
        let input = InputManager::new();

        let mut save_mgr = SaveManager::new();
        let _ = WebPersistence::load_profile(&mut save_mgr);

        let config = GameConfig::default();
        let (game, has_session) = if let Ok(Some(restored)) = WebPersistence::load_session_snapshot() {
            (restored, true)
        } else {
            let progress = save_mgr.get_career_progress().clone();
            let mut gm = GameManager::new(config);
            let stg = progress.current_stage.clamp(1, 5);
            let lvl = progress.current_level.clamp(1, 10);
            gm.init(GameMode::Career, stg, lvl, 1)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            gm.total_score = progress.total_score;
            gm.start_level()
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            (gm, false)
        };

        game.render(&mut renderer);

        Ok(Self {
            game,
            save_mgr,
            renderer,
            audio,
            input,
            autosave_accum: 0.0,
            paused: true,
            has_session,
        })
    }

    pub fn tick(&mut self, dt_sec: f32) -> String {
        if self.paused || self.game.level_complete.is_some() {
            self.game.render(&mut self.renderer);
            return self.get_status_json();
        }
        let clamped_dt = dt_sec.clamp(0.0, 0.1);

        // Process queued clicks
        for pos in self.input.take_clicks() {
            self.audio.ensure_stage_music(self.game.stage);
            if self.game.handle_click(pos) {
                self.audio.play_sfx("seat");
                let _ = WebPersistence::save_session_snapshot(&self.game);
            }
        }

        // Process keyboard shortcuts
        for key in self.input.take_keys() {
            self.audio.ensure_stage_music(self.game.stage);
            match key.as_str() {
                "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" => {
                    if let Ok(idx) = key.parse::<u64>() {
                        if self.game.click_table(TableId(idx)) {
                            self.audio.play_sfx("order");
                        }
                    }
                }
                " " | "Enter" => {
                    if self.game.click_kitchen() {
                        self.audio.play_sfx("serve");
                    }
                }
                "p" | "P" => {
                    if self.game.click_podium() {
                        self.audio.play_sfx("heart");
                    }
                }
                _ => {}
            }
        }

        let prev_score = self.game.score;
        let prev_stars = self.game.stars;
        let prev_ready = self.game.ready_orders.len();
        let prev_waiting = self.game.sitters.waiting_groups.len();
        let prev_kitchen_orders = self.game.room.kitchen.current_orders.len();
        let prev_served = self.game.customers_served;
        let prev_left = self.game.customers_left;
        let prev_eating = self
            .game
            .sitters
            .seated_groups
            .iter()
            .filter(|&&gid| {
                self.game
                    .sitters
                    .get_group(gid)
                    .map(|g| g.sitters.iter().any(|s| s.state == diner_dash_engine::game::SitterState::Eating))
                    .unwrap_or(false)
            })
            .count();
        let prev_check = self
            .game
            .sitters
            .seated_groups
            .iter()
            .filter(|&&gid| {
                self.game
                    .sitters
                    .get_group(gid)
                    .map(|g| g.sitters.iter().any(|s| s.state == diner_dash_engine::game::SitterState::Check))
                    .unwrap_or(false)
            })
            .count();
        let prev_dirty = self
            .game
            .room
            .tables
            .values()
            .filter(|t| t.state == diner_dash_engine::room::TableState::Dirty)
            .count();

        if let Some(level_res) = self.game.update(clamped_dt) {
            if level_res.success {
                self.audio.play_sfx("win");
                self.save_mgr.record_level_complete(&level_res);
                if level_res.expert {
                    self.save_mgr.record_expert(level_res.stage, level_res.level);
                }
                self.save_mgr.save_career_progress(
                    self.game.stage,
                    self.game.level,
                    self.game.total_score,
                    self.game.stars,
                    self.game.stars,
                );
            } else {
                self.audio.play_sfx("angry");
                self.save_mgr.record_level_failed();
            }
            let _ = WebPersistence::save_profile(&mut self.save_mgr);
            let _ = WebPersistence::save_session_snapshot(&self.game);
        } else {
            let now_eating = self
                .game
                .sitters
                .seated_groups
                .iter()
                .filter(|&&gid| {
                    self.game
                        .sitters
                        .get_group(gid)
                        .map(|g| g.sitters.iter().any(|s| s.state == diner_dash_engine::game::SitterState::Eating))
                        .unwrap_or(false)
                })
                .count();
            let now_check = self
                .game
                .sitters
                .seated_groups
                .iter()
                .filter(|&&gid| {
                    self.game
                        .sitters
                        .get_group(gid)
                        .map(|g| g.sitters.iter().any(|s| s.state == diner_dash_engine::game::SitterState::Check))
                        .unwrap_or(false)
                })
                .count();
            let now_dirty = self
                .game
                .room
                .tables
                .values()
                .filter(|t| t.state == diner_dash_engine::room::TableState::Dirty)
                .count();

            if self.game.customers_left > prev_left || self.game.stars < prev_stars {
                self.audio.play_sfx("angry");
            } else if self.game.stars > prev_stars {
                self.audio.play_sfx("heart");
            }
            if self.game.room.kitchen.current_orders.len() > prev_kitchen_orders {
                self.audio.play_sfx("order");
            }
            if self.game.ready_orders.len() > prev_ready {
                self.audio.play_sfx("ready");
            } else if self.game.ready_orders.len() < prev_ready && now_eating <= prev_eating {
                self.audio.play_sfx("pickup");
            }
            if now_eating > prev_eating {
                self.audio.play_sfx("serve");
            }
            if now_check > prev_check {
                self.audio.play_sfx("done_eating");
            }
            if self.game.customers_served > prev_served {
                self.audio.play_sfx("cash");
            } else if now_dirty < prev_dirty {
                self.audio.play_sfx("clean");
            } else if self.game.score > prev_score
                && self.game.room.kitchen.current_orders.len() == prev_kitchen_orders
                && now_eating == prev_eating
            {
                self.audio.play_sfx("cash");
            }
            if self.game.sitters.waiting_groups.len() > prev_waiting {
                self.audio.play_sfx("arrive");
            }
        }

        self.autosave_accum += clamped_dt;
        if self.autosave_accum >= 5.0 {
            self.autosave_accum = 0.0;
            self.save_mgr.add_play_time(5);
            let _ = WebPersistence::save_profile(&mut self.save_mgr);
            let _ = WebPersistence::save_session_snapshot(&self.game);
        }

        self.game.render(&mut self.renderer);
        self.get_status_json()
    }

    pub fn click(&mut self, x: f32, y: f32) -> bool {
        if self.paused || self.game.level_complete.is_some() { return false; }
        self.audio.ensure_stage_music(self.game.stage);
        let handled = self.game.handle_click(Vec2::new(x, y));
        if handled {
            self.audio.play_sfx("seat");
            let _ = WebPersistence::save_session_snapshot(&self.game);
        }
        self.game.render(&mut self.renderer);
        handled
    }

    /// Press: picks up a waiting party (original waitgroup_drag) or acts
    pub fn pointer_down(&mut self, x: f32, y: f32) -> bool {
        if self.paused || self.game.level_complete.is_some() { return false; }
        self.audio.ensure_stage_music(self.game.stage);
        let prev_drag = self.game.drag.is_some();
        let handled = self.game.pointer_down(Vec2::new(x, y));
        if handled {
            if !prev_drag && self.game.drag.is_some() {
                self.audio.play_sfx("rollover2");
            } else {
                self.audio.play_sfx("select");
            }
        }
        self.game.render(&mut self.renderer);
        handled
    }

    /// Move: carries the party and updates the table drop preview
    pub fn pointer_move(&mut self, x: f32, y: f32) {
        if self.paused || self.game.level_complete.is_some() { return; }
        self.game.pointer_move(Vec2::new(x, y));
        self.game.render(&mut self.renderer);
    }

    /// Release: seats the party on the hovered table, else returns it
    pub fn pointer_up(&mut self, x: f32, y: f32) -> bool {
        if self.paused || self.game.level_complete.is_some() { return false; }
        let handled = self.game.pointer_up(Vec2::new(x, y));
        if handled {
            self.audio.play_sfx("seat");
            let _ = WebPersistence::save_session_snapshot(&self.game);
        }
        self.game.render(&mut self.renderer);
        handled
    }

    pub fn key_down(&mut self, key: &str) {
        if !self.paused && self.game.level_complete.is_none() {
            self.input.record_key_down(key.to_string());
        }
    }

    pub fn key_up(&mut self, key: &str) {
        self.input.record_key_up(key);
    }

    pub fn start_career_level(&mut self, stage: u32, level: u32) -> Result<(), JsValue> {
        let stg = stage.clamp(1, 5);
        let lvl = level.clamp(1, 10);
        self.game
            .init(GameMode::Career, stg, lvl, 1)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.game
            .start_level()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.audio.ensure_stage_music(stg);
        // Starting/replaying a shift is not completing/unlocking that shift.
        self.paused = false;
        self.has_session = true;
        self.autosave_accum = 0.0;
        let _ = WebPersistence::save_profile(&mut self.save_mgr);
        let _ = WebPersistence::save_session_snapshot(&self.game);
        self.game.render(&mut self.renderer);
        Ok(())
    }

    pub fn start_endless_stage(&mut self, stage: u32, difficulty: u32) -> Result<(), JsValue> {
        self.paused = false;
        self.has_session = true;
        self.autosave_accum = 0.0;
        let stg = stage.clamp(1, 5);
        self.game
            .init(GameMode::Endless, stg, 1, difficulty.clamp(1, 3))
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.game
            .start_level()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.audio.ensure_stage_music(stg);
        let _ = WebPersistence::save_session_snapshot(&self.game);
        self.game.render(&mut self.renderer);
        Ok(())
    }

    pub fn start_endless(&mut self, difficulty: u32) -> Result<(), JsValue> {
        self.start_endless_stage(1, difficulty)
    }

    pub fn next_shift(&mut self) -> Result<(), JsValue> {
        let stg = self.game.stage.clamp(1, 5);
        let lvl = self.game.level.clamp(1, 10);
        self.start_career_level(stg, lvl)
    }

    pub fn save_now(&mut self) -> Result<String, JsValue> {
        WebPersistence::save_profile(&mut self.save_mgr)?;
        WebPersistence::save_session_snapshot(&self.game)?;
        WebPersistence::export_save_json(&self.save_mgr)
    }

    pub fn load_now(&mut self) -> Result<String, JsValue> {
        WebPersistence::load_profile(&mut self.save_mgr)?;
        if let Some(session) = WebPersistence::load_session_snapshot()? {
            self.game = session;
        } else {
            let c = self.save_mgr.get_career_progress().clone();
            self.game
                .init(
                    GameMode::Career,
                    c.current_stage.clamp(1, 5),
                    c.current_level.clamp(1, 10),
                    1,
                )
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            self.game
                .start_level()
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
        }
        self.game.render(&mut self.renderer);
        self.has_session = true;
        WebPersistence::export_save_json(&self.save_mgr)
    }

    pub fn export_save_json(&self) -> Result<String, JsValue> {
        WebPersistence::export_save_json(&self.save_mgr)
    }

    pub fn import_save_json(&mut self, json: &str) -> Result<(), JsValue> {
        WebPersistence::import_save_json(&mut self.save_mgr, json)?;
        let c = self.save_mgr.get_career_progress().clone();
        self.start_career_level(c.current_stage, c.current_level)?;
        Ok(())
    }

    pub fn reset_save(&mut self) -> Result<(), JsValue> {
        self.save_mgr = SaveManager::new();
        WebPersistence::clear_session_snapshot()?;
        WebPersistence::save_profile(&mut self.save_mgr)?;
        self.start_career_level(1, 1)?;
        self.has_session = false;
        Ok(())
    }

    pub fn set_muted(&mut self, muted: bool) {
        self.audio.set_muted(muted);
    }

    pub fn play_sfx(&self, name: &str) {
        self.audio.play_sfx(name);
    }

    pub fn ensure_menu_music(&mut self) {
        self.audio.ensure_menu_music();
    }

    pub fn ensure_stage_music(&mut self, stage: u32) {
        self.audio.ensure_stage_music(stage);
    }

    pub fn register_audio_buffer(&mut self, name: &str, buffer: web_sys::AudioBuffer) {
        let _ = self.audio.register_buffer(name.to_string(), buffer);
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
        // Do not replay keyboard actions accumulated before a menu transition.
        self.input.take_keys();
        self.input.take_clicks();
    }

    /// Table geometry + states in 800x600 stage coordinates.
    /// Used by the automated UI/browser tests to target drag & drop.
    pub fn table_info_json(&self) -> String {
        let mut ids: Vec<_> = self.game.room.tables.keys().copied().collect();
        ids.sort_by_key(|t| t.0);
        let tables: Vec<_> = ids
            .iter()
            .filter_map(|tid| self.game.room.tables.get(tid).map(|t| (tid, t)))
            .map(|(tid, t)| {
                let c = self.game.table_visual_center(t);
                serde_json::json!({
                    "id": tid.0,
                    "x": c.x,
                    "y": c.y,
                    "state": format!("{:?}", t.state),
                    "capacity": t.capacity,
                    "seated": t.group_id.is_some(),
                    "marks": self.game.waiter.pending_marks(*tid) as u32,
                    "error": matches!(self.game.dest_error, Some((eid, _)) if eid == *tid),
                })
            })
            .collect();
        serde_json::to_string(&tables).unwrap_or_else(|_| "[]".to_string())
    }

    pub fn get_status_json(&self) -> String {
        let intro = self
            .game
            .level_data
            .as_ref()
            .map(|l| l.text.intro.as_str())
            .unwrap_or("");
        serde_json::json!({
            "mode": format!("{:?}", self.game.mode),
            "paused": self.paused,
            "has_session": self.has_session,
            "stages": self.game.config.career.stages.iter().map(|s| {
                serde_json::json!({"id": s.id, "name": s.name, "num_levels": s.num_levels})
            }).collect::<Vec<_>>(),
            "stage": self.game.stage,
            "level": self.game.level,
            "score": self.game.score,
            "total_score": self.game.total_score,
            "minimum_tip": self.game.minimum_tip,
            "bonus_tip": self.game.bonus_tip,
            "stars": self.game.stars,
            "waiting_groups": self.game.sitters.waiting_groups.len(),
            "seated_groups": self.game.sitters.seated_groups.len(),
            "remaining_spawns": self.game.group_spawn_queue.len(),
            "ready_orders": self.game.ready_orders.len(),
            "customers_served": self.game.customers_served,
            "customers_left": self.game.customers_left,
            "intro": intro,
            "level_complete": self.game.level_complete,
            "career_save": self.save_mgr.get_career_progress(),
            "stats": self.save_mgr.get_stats(),
        })
        .to_string()
    }
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    WebPersistence::sync_from_idb_to_local_storage();
    Ok(())
}
