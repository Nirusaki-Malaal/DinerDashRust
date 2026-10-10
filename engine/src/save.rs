//! Save System
//! 
//! Handles game persistence (save/load) for both native and web

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::game::{GameMode, Stage, GameManager, LevelResult};
use crate::math::Vec2;

/// Save game data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub version: u32,
    pub timestamp: u64,
    
    // Career progress
    pub career: CareerSaveData,
    
    // Endless high scores
    pub endless_scores: HashMap<u32, u32>, // difficulty -> score
    
    // Settings
    pub settings: SettingsData,
    
    // Statistics
    pub stats: StatisticsData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CareerSaveData {
    #[serde(default = "first_level")]
    pub furthest_stage: u32,
    #[serde(default = "first_level")]
    pub furthest_level: u32,
    pub current_stage: u32,
    pub current_level: u32,
    pub total_score: u32,
    pub total_stars: u32,
    pub num_stars: u32,
    pub stage_stars: HashMap<u32, u32>, // stage -> stars earned
    pub level_expert: HashMap<String, bool>, // "stage_level" -> expert
    pub unlocked_stages: Vec<u32>,
}

impl Default for CareerSaveData {
    fn default() -> Self {
        Self {
            furthest_stage: 1,
            furthest_level: 1,
            current_stage: 1,
            current_level: 1,
            total_score: 0,
            total_stars: 3,
            num_stars: 3,
            stage_stars: HashMap::new(),
            level_expert: HashMap::new(),
            unlocked_stages: vec![1],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsData {
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub fullscreen: bool,
    pub vsync: bool,
    pub show_fps: bool,
    pub language: String,
}

impl Default for SettingsData {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 0.8,
            sfx_volume: 1.0,
            fullscreen: false,
            vsync: true,
            show_fps: false,
            language: "en".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsData {
    pub total_customers_served: u64,
    pub total_tips_earned: u64,
    pub total_levels_completed: u32,
    pub total_levels_failed: u32,
    pub expert_scores: u32,
    pub play_time_seconds: u64,
    pub games_started: u32,
    pub games_completed: u32,
}

impl Default for StatisticsData {
    fn default() -> Self {
        Self {
            total_customers_served: 0,
            total_tips_earned: 0,
            total_levels_completed: 0,
            total_levels_failed: 0,
            expert_scores: 0,
            play_time_seconds: 0,
            games_started: 0,
            games_completed: 0,
        }
    }
}

/// Save manager
pub struct SaveManager {
    save_data: SaveData,
    dirty: bool,
    auto_save_timer: f32,
    auto_save_interval: f32,
}

impl SaveManager {
    pub fn new() -> Self {
        Self {
            save_data: SaveData {
                version: 1,
                timestamp: 0,
                career: CareerSaveData::default(),
                endless_scores: HashMap::new(),
                settings: SettingsData::default(),
                stats: StatisticsData::default(),
            },
            dirty: false,
            auto_save_timer: 0.0,
            auto_save_interval: 30.0, // 30 seconds
        }
    }
    
    /// Load save data from storage
    pub fn load(&mut self) -> Result<()> {
        #[cfg(target_arch = "wasm32")]
        {
            self.load_web()?;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.load_native()?;
        }
        Ok(())
    }
    
    /// Save data to storage
    pub fn save(&mut self) -> Result<()> {
        self.save_data.timestamp = current_timestamp();
        
        #[cfg(target_arch = "wasm32")]
        {
            self.save_web()?;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.save_native()?;
        }
        
        self.dirty = false;
        Ok(())
    }
    
    /// Mark as dirty (needs save)
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    
    /// Update auto-save timer
    pub fn update(&mut self, dt: f32) {
        if self.dirty {
            self.auto_save_timer += dt;
            if self.auto_save_timer >= self.auto_save_interval {
                let _ = self.save();
                self.auto_save_timer = 0.0;
            }
        }
    }
    
    // ===== Career Progress =====
    
    pub fn get_career_progress(&self) -> &CareerSaveData {
        &self.save_data.career
    }
    
    pub fn get_mut_career_progress(&mut self) -> &mut CareerSaveData {
        self.dirty = true;
        &mut self.save_data.career
    }
    
    pub fn save_career_progress(
        &mut self,
        stage: u32,
        level: u32,
        total_score: u32,
        total_stars: u32,
        num_stars: u32,
    ) {
        let career = self.get_mut_career_progress();
        let furthest = (career.furthest_stage, career.furthest_level)
            .max((career.current_stage, career.current_level)).max((stage, level));
        career.furthest_stage = furthest.0;
        career.furthest_level = furthest.1;
        career.current_stage = stage;
        career.current_level = level;
        career.total_score = total_score;
        career.total_stars = total_stars;
        career.num_stars = num_stars;
        
        // Track stars per stage
        career.stage_stars.entry(stage).and_modify(|s| *s = (*s).max(num_stars)).or_insert(num_stars);
        
        // Unlock next stage/level
        if (level >= 10 || stage > 1) && stage <= 5 {
            if !career.unlocked_stages.contains(&stage) {
                career.unlocked_stages.push(stage);
            }
            if level >= 10 && stage < 5 && !career.unlocked_stages.contains(&(stage + 1)) {
                career.unlocked_stages.push(stage + 1);
            }
        }
    }
    
    pub fn record_expert(&mut self, stage: u32, level: u32) {
        let career = self.get_mut_career_progress();
        career.level_expert.insert(format!("{}_{}", stage, level), true);
    }
    
    pub fn is_expert(&self, stage: u32, level: u32) -> bool {
        self.save_data.career.level_expert.get(&format!("{}_{}", stage, level)).copied().unwrap_or(false)
    }
    
    pub fn is_stage_unlocked(&self, stage: u32) -> bool {
        self.save_data.career.unlocked_stages.contains(&stage)
    }
    
    pub fn get_stage_stars(&self, stage: u32) -> u32 {
        self.save_data.career.stage_stars.get(&stage).copied().unwrap_or(0)
    }
    
    // ===== Endless Scores =====
    
    pub fn submit_endless_score(&mut self, difficulty: u32, score: u32) -> u32 {
        let high_score = self.save_data.endless_scores.entry(difficulty).or_insert(0);
        if score > *high_score {
            *high_score = score;
            self.dirty = true;
            score
        } else {
            *high_score
        }
    }
    
    pub fn get_endless_high_score(&self, difficulty: u32) -> u32 {
        self.save_data.endless_scores.get(&difficulty).copied().unwrap_or(0)
    }
    
    // ===== Settings =====
    
    pub fn get_settings(&self) -> &SettingsData {
        &self.save_data.settings
    }
    
    pub fn get_mut_settings(&mut self) -> &mut SettingsData {
        self.dirty = true;
        &mut self.save_data.settings
    }
    
    // ===== Statistics =====
    
    pub fn get_stats(&self) -> &StatisticsData {
        &self.save_data.stats
    }
    
    pub fn record_level_complete(&mut self, result: &LevelResult) {
        self.dirty = true;
        let stats = &mut self.save_data.stats;
        stats.total_levels_completed += 1;
        stats.total_tips_earned += result.score as u64;
        stats.total_customers_served += result.customers_served as u64;
        if result.expert {
            stats.expert_scores += 1;
        }
    }
    
    pub fn record_level_failed(&mut self) {
        self.dirty = true;
        self.save_data.stats.total_levels_failed += 1;
    }
    
    pub fn record_game_started(&mut self) {
        self.dirty = true;
        self.save_data.stats.games_started += 1;
    }
    
    pub fn record_game_completed(&mut self) {
        self.dirty = true;
        self.save_data.stats.games_completed += 1;
    }
    
    pub fn add_play_time(&mut self, seconds: u64) {
        self.dirty = true;
        self.save_data.stats.play_time_seconds += seconds;
    }
    
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self.save_data)?)
    }

    pub fn from_json(&mut self, data: &str) -> Result<()> {
        self.save_data = serde_json::from_str(data)?;
        self.dirty = false;
        Ok(())
    }

    pub fn data(&self) -> &SaveData {
        &self.save_data
    }

    // ===== Platform-specific implementations =====
    
    #[cfg(target_arch = "wasm32")]
    fn load_web(&mut self) -> Result<()> {
        let window = web_sys::window().ok_or_else(|| anyhow::anyhow!("No window"))?;
        let storage = window
            .local_storage()
            .map_err(|e| anyhow::anyhow!("{:?}", e))?
            .ok_or_else(|| anyhow::anyhow!("No localStorage"))?;
        
        if let Some(data) = storage
            .get_item("diner_dash_save")
            .map_err(|e| anyhow::anyhow!("{:?}", e))?
        {
            self.save_data = serde_json::from_str(&data)?;
        }
        
        Ok(())
    }
    
    #[cfg(target_arch = "wasm32")]
    fn save_web(&self) -> Result<()> {
        let window = web_sys::window().ok_or_else(|| anyhow::anyhow!("No window"))?;
        let storage = window
            .local_storage()
            .map_err(|e| anyhow::anyhow!("{:?}", e))?
            .ok_or_else(|| anyhow::anyhow!("No localStorage"))?;
        
        let data = serde_json::to_string(&self.save_data)?;
        storage
            .set_item("diner_dash_save", &data)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        
        Ok(())
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    fn load_native(&mut self) -> Result<()> {
        use std::fs;
        
        let path = get_save_path()?;
        if path.exists() {
            let data = fs::read_to_string(path)?;
            self.save_data = serde_json::from_str(&data)?;
        }
        
        Ok(())
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    fn save_native(&self) -> Result<()> {
        use std::fs;
        
        let path = get_save_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let data = serde_json::to_string_pretty(&self.save_data)?;
        fs::write(path, data)?;
        
        Ok(())
    }
}

fn first_level() -> u32 { 1 }

#[cfg(not(target_arch = "wasm32"))]
fn get_save_path() -> Result<std::path::PathBuf> {
    let mut path = dirs::data_local_dir()
        .ok_or_else(|| anyhow::anyhow!("No data directory"))?;
    
    path.push("DinerDashRemastered");
    path.push("save.json");
    
    Ok(path)
}

fn current_timestamp() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        (js_sys::Date::now() / 1000.0) as u64
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
    }
}

impl Default for SaveManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{GameConfig, TableId};
    
    #[test]
    fn test_save_manager_creation() {
        let manager = SaveManager::new();
        assert_eq!(manager.save_data.version, 1);
    }
    
    #[test]
    fn test_career_progress() {
        let mut manager = SaveManager::new();
        manager.save_career_progress(1, 5, 1000, 3, 3);
        
        let career = manager.get_career_progress();
        assert_eq!(career.current_stage, 1);
        assert_eq!(career.current_level, 5);
        assert_eq!(career.total_score, 1000);
    }
    
    #[test]
    fn test_endless_scores() {
        let mut manager = SaveManager::new();
        let high = manager.submit_endless_score(1, 5000);
        assert_eq!(high, 5000);
        
        let high2 = manager.submit_endless_score(1, 3000);
        assert_eq!(high2, 5000); // Should not update
        
        let high3 = manager.submit_endless_score(1, 7000);
        assert_eq!(high3, 7000); // Should update
    }

    #[test]
    fn test_save_json_round_trip() {
        let mut manager = SaveManager::new();
        manager.save_career_progress(2, 10, 14500, 4, 4);
        manager.record_expert(2, 10);
        manager.submit_endless_score(2, 9200);
        manager.add_play_time(120);

        let json = manager.to_json().expect("Failed to serialize SaveManager");
        let mut restored = SaveManager::new();
        restored.from_json(&json).expect("Failed to deserialize SaveManager");

        assert_eq!(restored.get_career_progress().current_stage, 2);
        assert_eq!(restored.get_career_progress().current_level, 10);
        assert_eq!(restored.get_career_progress().total_score, 14500);
        assert!(restored.is_expert(2, 10));
        assert!(restored.is_stage_unlocked(3));
        assert_eq!(restored.get_endless_high_score(2), 9200);
        assert_eq!(restored.get_stats().play_time_seconds, 120);
    }

    #[test]
    fn test_game_manager_session_snapshot_round_trip() {
        let mut game = GameManager::new(GameConfig::default());
        game.init(GameMode::Career, 1, 1, 1).unwrap();
        game.start_level().unwrap();
        game.update(1.5);
        assert!(!game.sitters.waiting_groups.is_empty());

        // Clicking a clean table without dragging must NOT seat customers
        assert!(!game.click_table(TableId(1)));
        assert_eq!(game.sitters.seated_groups.len(), 0);

        // Drag waiting group onto Table 1
        let t1_center = {
            let t1 = game.room.tables.get(&TableId(1)).unwrap();
            game.table_visual_center(t1)
        };
        assert!(game.pointer_down(Vec2::new(72.0, 480.0)));
        game.pointer_move(t1_center);
        assert!(game.pointer_up(t1_center));

        let snapshot = serde_json::to_string(&game).expect("Serialize GameManager");
        let restored: GameManager = serde_json::from_str(&snapshot).expect("Deserialize GameManager");

        assert_eq!(restored.stage, 1);
        assert_eq!(restored.level, 1);
        assert_eq!(restored.sitters.seated_groups.len(), 1);
        assert!(restored.room.tables.get(&TableId(1)).unwrap().group_id.is_some());
    }
}
