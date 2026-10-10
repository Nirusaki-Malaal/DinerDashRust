//! Diner Dash Game Logic
//! 
//! Port of the original Lingo game systems to Rust

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use crate::math::{Vec2, Vec4, Rect, Color, clamp, rand};
use crate::ecs::{World, System};
use crate::renderer::Renderer;
use crate::room::{RoomManager, GridPos, TableState, KitchenOrder};
pub use crate::room::{TableId, SitterId};
use crate::sitter::{SitterSystem, SitterConfig, SitterEvent};
use crate::waiter::{Waiter, WaiterTask, WaiterEvent};
use crate::tutorial::TutorialManager;

/// Game modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMode {
    Career,
    Endless,
}

/// Game stages (restaurants)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Stage {
    Diner = 1,
    Chez = 2,
    Seafood = 3,
    Tiki = 4,
    Goddess = 5,
}

impl Stage {
    pub fn from_id(id: u32) -> Option<Self> {
        match id {
            1 => Some(Stage::Diner),
            2 => Some(Stage::Chez),
            3 => Some(Stage::Seafood),
            4 => Some(Stage::Tiki),
            5 => Some(Stage::Goddess),
            _ => None,
        }
    }
}

/// Customer colors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CustomerColor {
    Green,
    Red,
    Blue,
    Yellow,
    Purple,
    None, // Critic
}

impl CustomerColor {
    pub fn to_rgba(self) -> Color {
        match self {
            CustomerColor::Green => Vec4::new(0.20, 0.78, 0.35, 1.0),
            CustomerColor::Red => Vec4::new(0.90, 0.24, 0.24, 1.0),
            CustomerColor::Blue => Vec4::new(0.22, 0.52, 0.96, 1.0),
            CustomerColor::Yellow => Vec4::new(0.95, 0.80, 0.18, 1.0),
            CustomerColor::Purple => Vec4::new(0.68, 0.32, 0.88, 1.0),
            CustomerColor::None => Vec4::new(0.85, 0.85, 0.85, 1.0),
        }
    }
}

/// Sitter states (11 states from original)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SitterState {
    New,
    Waiting,
    Deciding,
    Ready,
    Ordered,
    Eating,
    Snack,
    Check,
    Scoring,
    Leaving,
    Ditching,
}

impl Default for SitterState {
    fn default() -> Self {
        SitterState::New
    }
}

/// Sitter group state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GroupState {
    Waiting,
    Seated,
    Eating,
    Done,
    Leaving,
}

/// Game configuration data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub starting_stars: u32,
    pub max_stars: u32,
    pub closing_time_ms: u32,
    pub career: CareerConfig,
    pub endless: EndlessConfig,
    pub powerups: HashMap<String, PowerupConfig>,
}

impl GameConfig {
    pub fn default_compiled() -> Self {
        const RAW_CONFIG: &str = include_str!("../../assets/data/game_config.json");
        serde_json::from_str(RAW_CONFIG).unwrap_or_else(|_| Self {
            starting_stars: 3,
            max_stars: 5,
            closing_time_ms: 10000,
            career: CareerConfig {
                num_stages: 5,
                levels_per_stage: 10,
                stages: Vec::new(),
            },
            endless: EndlessConfig {
                starting_time_multiplier: 1.0,
                time_multiplier_minimum: 0.4,
                time_multiplier_decrease: 0.05,
                difficulty_increase_ms: 60000,
                sublevels_per_level: 5,
            },
            powerups: HashMap::new(),
        })
    }
}

impl Default for GameConfig {
    fn default() -> Self {
        Self::default_compiled()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CareerConfig {
    pub num_stages: u32,
    pub levels_per_stage: u32,
    #[serde(default)]
    pub stages: Vec<StageData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndlessConfig {
    pub starting_time_multiplier: f32,
    pub time_multiplier_minimum: f32,
    pub time_multiplier_decrease: f32,
    pub difficulty_increase_ms: u32,
    pub sublevels_per_level: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerupConfig {
    pub levels: Vec<PowerupLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerupLevel {
    pub name: String,
    pub description: String,
    pub properties: HashMap<String, f32>,
    pub cost: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeatLayoutData {
    pub x: i32,
    pub y: i32,
    #[serde(default)]
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableLayoutData {
    pub x: i32,
    pub y: i32,
    pub capacity: u32,
    #[serde(default)]
    pub seats: Vec<SeatLayoutData>,
}

/// Level data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelData {
    pub id: String,
    pub minimum_tip: u32,
    pub bonus_goal: Option<u32>,
    pub num_stars: u32,
    #[serde(default)]
    pub bg_state: String,
    pub groups: Vec<GroupConfig>,
    pub text: LevelText,
    pub doodle: Option<String>,
    #[serde(default)]
    pub tables: Vec<TableLayoutData>,
    #[serde(default)]
    pub objects: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelText {
    pub intro: String,
    pub succeed: String,
    pub fail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupConfig {
    pub group_type: String,
    pub customers: u32,
    pub queue_time_ms: u32,
    pub frequency: u32,
    #[serde(default)]
    pub colors: Vec<CustomerColor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageData {
    pub id: String,
    pub name: String,
    pub num_levels: u32,
    pub background: String,
    pub music: String,
    pub num_stars: u32,
    pub levels: Vec<LevelData>,
}

/// Group data for spawning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupData {
    pub config: GroupConfig,
    pub state_data: HashMap<SitterState, f32>,
}

/// Main Game Manager - central controller
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameManager {
    pub mode: GameMode,
    pub stage: u32,
    pub level: u32,
    pub score: u32,
    pub total_score: u32,
    pub stars: u32,
    pub max_stars: u32,
    pub money_jar_level: u32,
    pub minimum_tip: u32,
    pub bonus_tip: u32,
    pub closing: bool,
    pub closing_timer: Option<f32>,
    #[serde(default)]
    pub door_closing_sign: bool,
    
    // Time management
    pub level_ms: u32,
    pub current_ms: u32,
    pub time_multiplier: TimeMultiplier,
    pub state_time_multipliers: HashMap<SitterState, f32>,
    pub ramp_ms: u32,
    pub accumulated_upgrades: u32,
    
    // Groups
    pub waiting_groups: VecDeque<GroupId>,
    pub new_groups: VecDeque<GroupId>,
    pub arrive_timer: Option<TimerId>,
    pub ramp_timer: Option<TimerId>,
    pub group_spawn_queue: VecDeque<GroupConfig>,
    pub next_arrival_timer_ms: f32,
    
    // Upgrades
    pub upgrades: UpgradeSet,
    pub available_snacks: Vec<String>,
    pub snack_percent: u32,
    
    // Tutorial
    pub tutorial_mode: bool,
    pub tutorial: Option<TutorialManager>,
    #[serde(default = "default_tutorial_step")]
    pub tutorial_step: u8,
    #[serde(default)]
    pub tutorial_step_timer: f32,
    #[serde(default)]
    pub tutorial_wave2_orders_sent: u32,
    
    // References
    pub config: GameConfig,
    pub level_data: Option<LevelData>,
    pub stage_data: Option<StageData>,
    pub sublevel_data: Option<LevelData>,
    
    // Active simulation subsystems
    pub room: RoomManager,
    pub sitters: SitterSystem,
    pub waiter: Waiter,
    pub selected_waiting_group: Option<GroupId>,
    /// Active customer drag (transient UI state, not persisted)
    #[serde(skip, default)]
    pub drag: Option<DragState>,
    /// Current pointer hover position in 800x600 stage space
    #[serde(skip, default)]
    pub hover_pos: Option<Vec2>,
    /// Table showing the "nothing to do" bubble from a failed Flo action
    #[serde(skip, default)]
    pub dest_error: Option<(TableId, f32)>,
    /// Transient coin-flip tip animations above tables (`tip_bonus.ls`: 400..600ms)
    #[serde(skip, default)]
    pub tip_flips: Vec<(TableId, u32, f32)>,
    pub ready_orders: Vec<KitchenOrder>,
    pub customers_served: u32,
    pub customers_left: u32,
    pub level_complete: Option<LevelResult>,

    // UI
    pub show_upgrade_button: bool,
}

fn default_tutorial_step() -> u8 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeMultiplier {
    pub starting: f32,
    pub current: f32,
    pub minimum: f32,
    pub decrease: f32,
}

impl Default for TimeMultiplier {
    fn default() -> Self {
        Self {
            starting: 1.0,
            current: 1.0,
            minimum: 1.0,
            decrease: 0.0,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpgradeSet {
    pub tables: Vec<PowerupLevel>,
    pub powerups: HashMap<String, Vec<PowerupLevel>>,
}

/// Timer ID for scheduled events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TimerId(pub u64);

/// Group ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GroupId(pub u64);

/// In-progress drag of a waiting party, mirroring the original
/// `waitgroup_drag` behavior: the group follows the mouse and a valid
/// table previews the seated party until the button is released.
#[derive(Debug, Clone)]
pub struct DragState {
    pub group: GroupId,
    pub pos: Vec2,
    pub down_pos: Vec2,
    pub moved: bool,
    pub hover_table: Option<TableId>,
}

impl GameManager {
    pub fn new(config: GameConfig) -> Self {
        let mut state_multipliers = HashMap::new();
        for state in [
            SitterState::Waiting,
            SitterState::Deciding,
            SitterState::Ready,
            SitterState::Ordered,
            SitterState::Eating,
            SitterState::Check,
            SitterState::Snack,
        ] {
            state_multipliers.insert(state, 1.0);
        }
        
        let mut room = RoomManager::new(18, 13, 37.0);
        let _ = room.load_stage(Stage::Diner);
        let sitters = SitterSystem::new(SitterConfig::default());
        let waiter = Waiter::new(Vec2::new(296.0, 211.0));

        Self {
            mode: GameMode::Career,
            stage: 1,
            level: 1,
            score: 0,
            total_score: 0,
            stars: config.starting_stars,
            max_stars: config.max_stars,
            money_jar_level: 0,
            minimum_tip: 100,
            bonus_tip: 500,
            closing: false,
            closing_timer: None,
            door_closing_sign: false,
            level_ms: 0,
            current_ms: 0,
            time_multiplier: TimeMultiplier::default(),
            state_time_multipliers: state_multipliers,
            ramp_ms: 0,
            accumulated_upgrades: 0,
            waiting_groups: VecDeque::new(),
            new_groups: VecDeque::new(),
            arrive_timer: None,
            ramp_timer: None,
            group_spawn_queue: VecDeque::new(),
            next_arrival_timer_ms: 0.0,
            upgrades: UpgradeSet::default(),
            available_snacks: Vec::new(),
            snack_percent: 0,
            tutorial_mode: false,
            tutorial: None,
            tutorial_step: 1,
            tutorial_step_timer: 0.0,
            tutorial_wave2_orders_sent: 0,
            config,
            level_data: None,
            stage_data: None,
            sublevel_data: None,
            room,
            sitters,
            waiter,
            selected_waiting_group: None,
            drag: None,
            hover_pos: None,
            dest_error: None,
            tip_flips: Vec::new(),
            ready_orders: Vec::new(),
            customers_served: 0,
            customers_left: 0,
            level_complete: None,
            show_upgrade_button: false,
        }
    }
    
    /// Initialize for new game
    pub fn init(&mut self, mode: GameMode, stage: u32, level: u32, difficulty: u32) -> Result<()> {
        self.mode = mode;
        self.stage = stage;
        self.level = level;
        
        if mode == GameMode::Career {
            self.stage_data = Some(self.config.career.stages.get((stage - 1) as usize).cloned()
                .ok_or_else(|| anyhow::anyhow!("Invalid stage"))?);
            self.level_data = Some(self.stage_data.as_ref().unwrap().levels.get((level - 1) as usize).cloned()
                .ok_or_else(|| anyhow::anyhow!("Invalid level"))?);
            let lvl = self.level_data.as_ref().unwrap();
            self.minimum_tip = lvl.minimum_tip;
            self.bonus_tip = lvl.bonus_goal.unwrap_or(self.minimum_tip * 2);
            self.stars = self.stage_data.as_ref().unwrap().num_stars;
        } else {
            self.setup_endless(difficulty);
        }
        
        self.score = 0;
        self.money_jar_level = 0;
        self.closing = false;
        self.closing_timer = None;
        self.door_closing_sign = false;
        self.waiting_groups.clear();
        self.new_groups.clear();
        self.group_spawn_queue.clear();
        self.ready_orders.clear();
        self.selected_waiting_group = None;
        self.drag = None;
        self.hover_pos = None;
        self.dest_error = None;
        self.tip_flips.clear();
        self.customers_served = 0;
        self.customers_left = 0;
        self.level_complete = None;
        
        Ok(())
    }
    
    fn setup_endless(&mut self, _difficulty: u32) {
        let endless = &self.config.endless;
        self.time_multiplier.starting = endless.starting_time_multiplier;
        self.time_multiplier.current = endless.starting_time_multiplier;
        self.time_multiplier.minimum = endless.time_multiplier_minimum;
        self.time_multiplier.decrease = endless.time_multiplier_decrease;
        self.ramp_ms = endless.difficulty_increase_ms;
        self.accumulated_upgrades = 0;
        self.show_upgrade_button = false;
        self.minimum_tip = 1000;
        self.bonus_tip = 2500;
    }
    
    /// Start a level
    pub fn start_level(&mut self) -> Result<()> {
        self.score = 0;
        self.money_jar_level = 0;
        self.closing = false;
        self.closing_timer = None;
        self.door_closing_sign = false;
        self.current_ms = 0;
        self.level_complete = None;
        self.ready_orders.clear();
        self.selected_waiting_group = None;
        self.drag = None;
        self.hover_pos = None;
        self.dest_error = None;
        self.tip_flips.clear();
        self.tutorial_mode = self.mode == GameMode::Career && self.stage == 1 && self.level == 1;
        self.tutorial_step = if self.tutorial_mode { 1 } else { 25 };
        self.tutorial_step_timer = 0.0;
        self.tutorial_wave2_orders_sent = 0;
        self.sitters = SitterSystem::new(SitterConfig::default());
        self.room = RoomManager::new(18, 13, 37.0);
        self.room.kitchen.grid_pos = GridPos::new(9, 3);
        // Per room_manager.ls line 544: `waiter.arriveAtTile(stations.ticketspot.getTile())`
        let start_pos = if self.stage == 3 {
            Vec2::new(333.0, 211.0)
        } else {
            Vec2::new(296.0, 211.0)
        };
        self.waiter = Waiter::new(start_pos);

        if self.mode == GameMode::Career {
            if let Some(level) = self.level_data.clone() {
                if !level.tables.is_empty() {
                    for t in &level.tables {
                        let seats: Vec<(GridPos, String)> = t
                            .seats
                            .iter()
                            .map(|s| (GridPos::new(s.x, s.y), s.color.clone()))
                            .collect();
                        self.room.add_table_with_colored_seats(GridPos::new(t.x, t.y), &seats);
                    }
                } else if let Some(stage_enum) = Stage::from_id(self.stage) {
                    let _ = self.room.load_stage(stage_enum);
                }

                let has_podium = level.objects.iter().any(|o| o == "podium")
                    || (self.stage >= 2 && self.level >= 3);
                if has_podium {
                    self.room.podium = Some(crate::room::Podium::new(GridPos::new(1, 12)));
                }

                self.group_spawn_queue.clear();
                self.new_groups.clear();
                for group_config in &level.groups {
                    let group_id = GroupId(rand::range_int(1, i32::MAX) as u64);
                    self.new_groups.push_back(group_id);
                    self.group_spawn_queue.push_back(group_config.clone());
                }
            }
            self.schedule_next_arrival(true)?;
        } else {
            let stage_enum = Stage::from_id(self.stage).unwrap_or(Stage::Diner);
            let _ = self.room.load_stage(stage_enum);
            self.stars = 5;
            self.max_stars = 5;
            self.group_spawn_queue.clear();
            self.new_groups.clear();
            self.spawn_random_group()?;
            self.schedule_next_arrival(true)?;
        }
        
        self.schedule_ramp()?;
        Ok(())
    }

    fn spawn_group_from_config(&mut self, cfg: GroupConfig) -> GroupId {
        let colors = if cfg.colors.is_empty() {
            let palette = [
                CustomerColor::Red,
                CustomerColor::Blue,
                CustomerColor::Green,
                CustomerColor::Yellow,
                CustomerColor::Purple,
            ];
            let c = palette[(rand::range_int(0, 4)) as usize];
            vec![c; cfg.customers.max(1) as usize]
        } else {
            cfg.colors
        };
        let party_size = cfg.customers.max(1);
        let group_id = self.sitters.spawn_group(
            cfg.group_type,
            party_size,
            colors,
            cfg.queue_time_ms,
            cfg.queue_time_ms,
        );
        self.waiting_groups.push_back(group_id);
        if self.selected_waiting_group.is_none() {
            self.selected_waiting_group = Some(group_id);
        }
        group_id
    }
    
    /// Schedule next group arrival (`game_manager.ls` line 662: when `newGroups.count = 0`, calls `room.closingTime()`, NOT `game_manager.closingTime()`)
    pub fn schedule_next_arrival(&mut self, now: bool) -> Result<()> {
        if self.mode == GameMode::Endless && self.group_spawn_queue.is_empty() {
            self.spawn_random_group()?;
        }
        
        if self.group_spawn_queue.is_empty() && self.new_groups.is_empty() {
            if self.mode == GameMode::Career {
                self.door_closing_sign = true;
            }
            return Ok(());
        }
        
        if let Some(next_cfg) = self.group_spawn_queue.front() {
            let delay_ms = if now { 0.0 } else { next_cfg.queue_time_ms.max(4000) as f32 };
            self.next_arrival_timer_ms = delay_ms;
            if delay_ms <= 0.0 {
                let cfg = self.group_spawn_queue.pop_front().unwrap();
                let _ = self.new_groups.pop_front();
                self.spawn_group_from_config(cfg);
                if self.mode == GameMode::Endless && self.group_spawn_queue.is_empty() {
                    self.spawn_random_group()?;
                }
                if let Some(after) = self.group_spawn_queue.front() {
                    self.next_arrival_timer_ms = after.queue_time_ms.max(5000) as f32;
                } else if self.mode == GameMode::Career {
                    self.door_closing_sign = true;
                }
            }
        } else if let Some(group_id) = self.new_groups.pop_front() {
            self.waiting_groups.push_back(group_id);
        }
        
        Ok(())
    }
    
    /// Add group to waiting queue
    pub fn add_waiting_group(&mut self, group_id: GroupId) {
        self.waiting_groups.push_back(group_id);
        self.schedule_next_arrival(false).ok();
    }
    
    /// Remove group from waiting
    pub fn remove_waiting_group(&mut self, group_id: GroupId) {
        self.waiting_groups.retain(|g| *g != group_id);
        if self.selected_waiting_group == Some(group_id) {
            self.selected_waiting_group = self.sitters.waiting_groups.front().copied();
        }
    }

    /// Advance the full game simulation by `dt` seconds
    pub fn update(&mut self, dt: f32) -> Option<LevelResult> {
        if let Some(done) = &self.level_complete {
            return Some(done.clone());
        }

        let dt_ms = (dt * 1000.0) as u32;
        self.current_ms = self.current_ms.saturating_add(dt_ms);

        // Fade the "nothing to do" bubble above a table
        if let Some((_, remaining)) = &mut self.dest_error {
            *remaining -= dt;
            if *remaining <= 0.0 {
                self.dest_error = None;
            }
        }

        // Update transient coin-flip animations (`tip_bonus.ls`)
        for (_, _, timer) in &mut self.tip_flips {
            *timer -= dt;
        }
        self.tip_flips.retain(|(_, _, timer)| *timer > 0.0);

        // Count down 1000ms "CLOSING TIME!" banner timer (`game_manager.ls` line 381)
        if let Some(rem) = &mut self.closing_timer {
            *rem = (*rem - dt).max(0.0);
        }

        // 1. Group spawn timer (in Career 1-1 tutorial, wait until the first group is served, table cleaned, and dishes bused)
        let is_tutorial_first_group_active = self.mode == GameMode::Career
            && self.stage == 1
            && self.level == 1
            && (self.customers_served == 0
                || self.has_dirty_tables()
                || self.waiter.carried_dishes > 0);
        if !self.group_spawn_queue.is_empty() && !is_tutorial_first_group_active {
            if self.mode == GameMode::Career
                && self.stage == 1
                && self.level == 1
                && self.customers_served >= 1
                && self.sitters.waiting_groups.is_empty()
                && self.sitters.seated_groups.is_empty()
            {
                self.next_arrival_timer_ms = 0.0;
            } else {
                self.next_arrival_timer_ms -= dt * 1000.0;
            }
            if self.next_arrival_timer_ms <= 0.0 {
                loop {
                    let Some(cfg) = self.group_spawn_queue.pop_front() else {
                        break;
                    };
                    let _ = self.new_groups.pop_front();
                    self.spawn_group_from_config(cfg);
                    // Spawn simultaneous 0ms groups together (e.g. tutorial wave 2)
                    if !matches!(self.group_spawn_queue.front(), Some(next) if next.queue_time_ms == 0) {
                        break;
                    }
                }
                if self.mode == GameMode::Endless && self.group_spawn_queue.is_empty() {
                    let _ = self.spawn_random_group();
                }
                if let Some(next_cfg) = self.group_spawn_queue.front() {
                    self.next_arrival_timer_ms = next_cfg.queue_time_ms.max(5000) as f32;
                } else if self.mode == GameMode::Career {
                    self.door_closing_sign = true;
                }
            }
        }

        // 2. Update kitchen orders
        let current_time_s = self.current_ms as f32 / 1000.0;
        let newly_ready = self.room.kitchen.update(current_time_s);
        self.ready_orders.extend(newly_ready);
        self.room.update_bg_state();

        // 3. Update sitters (both seated and waiting)
        let sitter_events = self.sitters.update(dt, &self.room);
        for (group_id, event) in sitter_events {
            match event {
                SitterEvent::Ditch | SitterEvent::Ditched => {
                    if self.sitters.get_group(group_id).is_none() {
                        continue;
                    }
                    self.customers_left += 1;
                    self.remove_waiting_group(group_id);
                    if let Some((&table_id, _)) = self
                        .room
                        .tables
                        .iter()
                        .find(|(_, t)| t.group_id == Some(group_id))
                    {
                        self.room.clear_table(table_id);
                        if let Some(t) = self.room.tables.get_mut(&table_id) {
                            t.state = TableState::Clean;
                            t.dirty_timer = None;
                        }
                        self.waiter.carried_tickets.retain(|&t| t != table_id);
                        self.waiter.carried_food.retain(|&t| t != table_id);
                        self.ready_orders.retain(|o| o.table_id != table_id);
                    }
                    self.sitters.remove_group(group_id);
                    self.angry_customer(500);
                }
                SitterEvent::Left => {
                    // Per `sgstate.scoring.ls` (`exitState`) & `sgstate.leave.ls`:
                    // After 1.0s of `#scoring`, the group vacates the table, leaves their tip (`showTip` coin flip),
                    // and the table becomes `#dirty`.
                    let Some(group) = self.sitters.get_group(group_id).cloned() else {
                        continue;
                    };
                    let timing = group.timing();
                    let customer_tip = (group.sitters.len() as u32 * timing.base_tip)
                        + (group.mood_points.max(0) as u32 * timing.mood_tip);
                    let total_tip = 50 + customer_tip;
                    if group.group_type.contains("critic") && group.mood_points >= 8 {
                        self.add_star();
                    }
                    if let Some((&table_id, _)) = self
                        .room
                        .tables
                        .iter()
                        .find(|(_, t)| t.group_id == Some(group_id))
                    {
                        self.room.clear_table(table_id);
                        self.tip_flips.push((table_id, total_tip, 0.6));
                    }
                    self.sitters.remove_group(group_id);
                    self.customers_served += 1;
                    self.score_tip(total_tip);
                }
                _ => {}
            }
        }

        // Keep waiting_groups synced with sitters.waiting_groups
        self.waiting_groups = self.sitters.waiting_groups.clone();
        if self.selected_waiting_group.is_none()
            || !self
                .waiting_groups
                .contains(&self.selected_waiting_group.unwrap())
        {
            self.selected_waiting_group = self.waiting_groups.front().copied();
        }

        // 4. Update waiter and handle completed tasks
        let waiter_events = self.waiter.update(dt, &mut self.room);
        for ev in waiter_events {
            match ev {
                WaiterEvent::OrderTaken { table_id } => {
                    if let Some(table) = self.room.tables.get(&table_id) {
                        if let Some(group_id) = table.group_id {
                            if let Some(group) = self.sitters.get_group_mut(group_id) {
                                if !group.sitters.iter().all(|s| s.state == SitterState::Ready) {
                                    continue;
                                }
                                group.on_order_taken("meal".to_string());
                            } else {
                                continue;
                            }
                            self.score_tip(20);
                            if !self.waiter.carried_tickets.contains(&table_id) {
                                self.waiter.carried_tickets.push(table_id);
                            }
                        }
                    }
                }
                WaiterEvent::TicketsDropped { tables } => {
                    for tid in tables {
                        if self.tutorial_mode && self.tutorial_step >= 15 {
                            self.tutorial_wave2_orders_sent += 1;
                        }
                        self.room.kitchen.add_order(
                            tid,
                            0,
                            "meal".to_string(),
                            current_time_s,
                        );
                    }
                }
                WaiterEvent::FoodPickedUp { table_id } => {
                    let pos_opt = self
                        .ready_orders
                        .iter()
                        .position(|o| o.table_id == table_id)
                        .or_else(|| (!self.ready_orders.is_empty()).then_some(0));
                    if let Some(pos) = pos_opt {
                        let order = self.ready_orders.remove(pos);
                        self.waiter.carried_food.push(order.table_id);
                        self.waiter.carrying_food = Some(order.food_type);
                    }
                }
                WaiterEvent::FoodDelivered { table_id, .. } => {
                    if let Some(table) = self.room.tables.get(&table_id) {
                        if let Some(group_id) = table.group_id {
                            if let Some(group) = self.sitters.get_group_mut(group_id) {
                                if !group.sitters.iter().all(|s| s.state == SitterState::Ordered) {
                                    continue;
                                }
                                group.on_food_delivered();
                            }
                            self.score_tip(30);
                        }
                    }
                }
                WaiterEvent::CheckDelivered { table_id } => {
                    // Per `sgstate.check.ls` line 43-44:
                    // `table.showCheck()`, `flo.dropCheck(table)`, `group.nextState()` -> `#scoring` (1000ms).
                    // The group stays at the table in `SitterState::Scoring` for 1.0s before `SitterEvent::Left` vacates it!
                    if let Some(table) = self.room.tables.get(&table_id) {
                        if let Some(group_id) = table.group_id {
                            if let Some(group) = self.sitters.get_group_mut(group_id) {
                                if !group.sitters.iter().all(|s| s.state == SitterState::Check) {
                                    continue;
                                }
                                let _ = group.on_check_delivered();
                            }
                        }
                    }
                }
                WaiterEvent::TableCleaned { table_id } => {
                    if let Some(table) = self.room.tables.get_mut(&table_id) {
                        if table.state != TableState::Dirty {
                            continue;
                        }
                        table.state = TableState::Clean;
                        table.dirty_timer = None;
                        table.group_id = None;
                        self.waiter.carried_dishes += 1;
                    }
                    self.score_tip(40);
                }
                WaiterEvent::DishesDropped { .. } => {}
                WaiterEvent::ArrivedAtPodium => {
                    if let Some(podium) = &mut self.room.podium {
                        podium.flo_here = true;
                        self.waiter.task_queue.clear();
                    }
                }
                WaiterEvent::TaskCompleted { .. } => {}
            }
        }

        // Update podium charging, activation, and cooldown (`podium.ls` lines 82-128)
        let mut podium_activated = false;
        if let Some(podium) = &mut self.room.podium {
            if self.waiter.current_task.is_some()
                || self.waiter.position.distance(Vec2::new(148.0, 507.0)) > 6.0
            {
                podium.stop();
            }
            if let Some(rem) = &mut podium.active_timer {
                *rem -= dt.max(0.0);
                if *rem <= 0.0 {
                    podium.active_timer = None;
                    podium.cooldown_timer = Some(6.0);
                }
            } else if let Some(rem) = &mut podium.cooldown_timer {
                *rem -= dt.max(0.0);
                if *rem <= 0.0 {
                    podium.cooldown_timer = None;
                    podium.ready = true;
                }
            } else if podium.ready && podium.flo_here {
                podium.charge_progress += dt.max(0.0) / 1.8;
                if podium.charge_progress >= 1.0 {
                    podium.charge_progress = 0.0;
                    podium.ready = false;
                    podium.active_timer = Some(2.0);
                    self.waiter.podium_end_timer = Some(0.6);
                    podium_activated = true;
                }
            }
        }
        if podium_activated {
            self.sitters.change_mood(0.6);
            self.score_tip(15);
        }

        // Advance 24-step tutorial state machine (`tutorial_manager.ls` + `tutorial_1..24.ls`)
        if self.tutorial_mode && self.tutorial_step <= 24 {
            self.update_tutorial_step(dt);
        }

        // 5. Check closingTime & level completion (`game_manager.ls` lines 342, 363-382 & 602)
        if self.mode == GameMode::Endless && self.stars == 0 {
            let res = self.end_level();
            self.level_complete = Some(res.clone());
            return Some(res);
        }
        if self.mode == GameMode::Career
            && self.group_spawn_queue.is_empty()
            && self.sitters.waiting_groups.is_empty()
            && self.sitters.seated_groups.is_empty()
            && (!self.tutorial_mode || self.tutorial_step > 24)
        {
            if !self.closing {
                let _ = self.start_closing();
            }
            if !self.has_dirty_tables() && self.waiter.carried_dishes == 0 {
                let res = self.end_level();
                self.level_complete = Some(res.clone());
                return Some(res);
            }
        }

        None
    }

    /// Select a waiting group in the queue
    pub fn click_waiting_group(&mut self, group_id: Option<GroupId>) -> Option<GroupId> {
        let target = group_id.or_else(|| self.sitters.waiting_groups.front().copied());
        if let Some(gid) = target {
            if self.sitters.waiting_groups.contains(&gid) {
                self.selected_waiting_group = Some(gid);
                return Some(gid);
            }
        }
        None
    }

    /// Click a table to dispatch Flo (seating is drag-and-drop only!)
    pub fn click_table(&mut self, table_id: TableId) -> bool {
        self.click_table_inner(table_id)
    }

    fn click_table_inner(&mut self, table_id: TableId) -> bool {
        let Some(table) = self.room.tables.get(&table_id).cloned() else {
            return false;
        };

        match table.state {
            TableState::Clean => {
                // Customers are seated ONLY by dragging and dropping onto a clean table.
                false
            }
            TableState::Dirty => {
                // If Flo's hands are full of dishes/trays, drop dishes at bus station first
                let hands_in_use = self.waiter.carried_food.len() + self.waiter.carried_dishes as usize;
                if hands_in_use >= 2 && self.waiter.carried_dishes > 0 {
                    let _ = self.waiter.add_task(WaiterTask::DropDishes);
                }
                self.waiter.add_task(WaiterTask::CleanTable { table_id })
            }
            TableState::Occupied => {
                if let Some(group_id) = table.group_id {
                    if let Some(group) = self.sitters.get_group(group_id) {
                        let all_ready = group.sitters.iter().all(|s| s.state == SitterState::Ready);
                        let all_ordered = group.sitters.iter().all(|s| s.state == SitterState::Ordered);
                        let all_check = group.sitters.iter().all(|s| s.state == SitterState::Check);

                        if all_ready {
                            return self.waiter.add_task(WaiterTask::TakeOrder {
                                table_id,
                                seat_index: 0,
                            });
                        } else if all_ordered {
                            let carrying_for_table = self.waiter.carried_food.contains(&table_id);
                            let pickup_queued = self.waiter.has_task_matching(
                                |t| matches!(t, WaiterTask::PickupFood { table_id: tid } if *tid == table_id),
                            );
                            if carrying_for_table || pickup_queued {
                                return self.waiter.add_task(WaiterTask::DeliverFood {
                                    table_id,
                                    seat_index: 0,
                                    food_type: "meal".to_string(),
                                });
                            }
                        } else if all_check {
                            return self.waiter.add_task(WaiterTask::DeliverCheck { table_id });
                        }
                    }
                }
                false
            }
            TableState::Reserved => false,
        }
    }

    /// Click ticket station at (301, 171) to drop off collected order tickets
    pub fn click_ticket_station(&mut self) -> bool {
        let has_tickets = !self.waiter.carried_tickets.is_empty();
        let taking_order = self
            .waiter
            .has_task_matching(|t| matches!(t, WaiterTask::TakeOrder { .. }));
        if has_tickets || taking_order {
            return self.waiter.add_task(WaiterTask::DropTickets);
        }
        false
    }

    /// Stage-specific coordinates for `[ticketspot, food_1..4, busspot, podium]`
    /// extracted from `bg_diner_layout.txt`, `bg_tiki_layout.txt`, `bg_seafood_layout.txt`,
    /// `bg_chez_layout.txt`, `bg_goddess_layout.txt`, and `room_layout.txt` (`branch.loc - branch.regPoint + child.loc`).
    pub fn stage_station_positions(&self) -> (Vec2, [Vec2; 4], Vec2, Vec2) {
        match self.stage {
            2 | 3 => (
                Vec2::new(301.0, 171.0),
                [
                    Vec2::new(373.0, 174.0),
                    Vec2::new(446.0, 174.0),
                    Vec2::new(518.0, 174.0),
                    Vec2::new(583.0, 174.0),
                ],
                Vec2::new(669.0, 203.0),
                Vec2::new(83.0, 515.0),
            ),
            4 => (
                Vec2::new(301.0, 165.0),
                [
                    Vec2::new(373.0, 168.0),
                    Vec2::new(446.0, 168.0),
                    Vec2::new(518.0, 168.0),
                    Vec2::new(583.0, 168.0),
                ],
                Vec2::new(668.0, 203.0),
                Vec2::new(83.0, 515.0),
            ),
            5 => (
                Vec2::new(301.0, 165.0),
                [
                    Vec2::new(373.0, 168.0),
                    Vec2::new(446.0, 168.0),
                    Vec2::new(518.0, 168.0),
                    Vec2::new(583.0, 168.0),
                ],
                Vec2::new(667.0, 203.0),
                Vec2::new(83.0, 515.0),
            ),
            _ => (
                Vec2::new(290.0, 151.0),
                [
                    Vec2::new(362.0, 154.0),
                    Vec2::new(435.0, 154.0),
                    Vec2::new(507.0, 154.0),
                    Vec2::new(582.0, 154.0),
                ],
                Vec2::new(663.0, 198.0),
                Vec2::new(83.0, 515.0),
            ),
        }
    }

    /// Click kitchen counter to pick up a ready food tray into Flo's hands
    pub fn click_kitchen(&mut self) -> bool {
        let (_, food_slots, _, _) = self.stage_station_positions();
        self.click_kitchen_at(food_slots[0])
    }

    pub fn click_kitchen_at(&mut self, pos: Vec2) -> bool {
        if self.ready_orders.is_empty() {
            return false;
        }
        let (_, food_slots, _, _) = self.stage_station_positions();
        // Find clicked tray slot (or first unqueued ready order)
        let mut chosen_tid = None;
        for (idx, order) in self.ready_orders.iter().take(4).enumerate() {
            let already_queued = self.waiter.has_task_matching(
                |t| matches!(t, WaiterTask::PickupFood { table_id } if *table_id == order.table_id),
            );
            if !already_queued && pos.distance(food_slots[idx]) < 42.0 {
                chosen_tid = Some(order.table_id);
                break;
            }
        }
        if chosen_tid.is_none() {
            chosen_tid = self
                .ready_orders
                .iter()
                .find(|o| {
                    !self.waiter.has_task_matching(
                        |t| matches!(t, WaiterTask::PickupFood { table_id } if *table_id == o.table_id),
                    )
                })
                .map(|o| o.table_id);
        }
        if let Some(table_id) = chosen_tid {
            let hands_in_use = self.waiter.carried_food.len() + self.waiter.carried_dishes as usize;
            if hands_in_use >= 2 && self.waiter.carried_dishes > 0 {
                let _ = self.waiter.add_task(WaiterTask::DropDishes);
            }
            return self.waiter.add_task(WaiterTask::PickupFood { table_id });
        }
        false
    }

    /// Click bus station to drop off dirty dishes
    pub fn click_bus_station(&mut self) -> bool {
        let has_dishes = self.waiter.carried_dishes > 0;
        let cleaning_queued = self
            .waiter
            .has_task_matching(|t| matches!(t, WaiterTask::CleanTable { .. }));
        if has_dishes || cleaning_queued {
            return self.waiter.add_task(WaiterTask::DropDishes);
        }
        false
    }

    /// Click podium to entertain waiting customers (`podium.ls` line 144: `if not floHere then room_mgr.selectDest(me)`)
    pub fn click_podium(&mut self) -> bool {
        let Some(podium) = &mut self.room.podium else {
            return false;
        };
        if !podium.flo_here {
            self.waiter.add_task(WaiterTask::UsePodium);
            return true;
        }
        false
    }

    /// Compute visual table center in 800x600 Director stage coordinates.
    /// `table.grid_pos` is `anchor_pos = (c1.x + 1, c1.y - 1)` per `room_manager.ls` line 438,
    /// so `base = grid_to_world(anchor_pos) = c1.world + (37, -37)`.
    /// Adding `(38, -4)` (2top), `(37, 9)` (4top), `(33, 24)` (6top) places `chair_1` at `c1.world`
    /// and `table_1` at `c1.world + (table_1.loc - chair_1.loc)` per `2top/4top/6top_layout.txt`.
    pub fn table_visual_center(&self, table: &crate::room::Table) -> Vec2 {
        let base = self.room.grid.grid_to_world(table.grid_pos);
        if (self.room.grid.cell_size - 37.0).abs() < 0.5 {
            match table.capacity {
                4 => base + Vec2::new(37.0, 9.0),
                6 => base + Vec2::new(33.0, 24.0),
                _ => base + Vec2::new(38.0, -4.0),
            }
        } else {
            base
        }
    }

    /// Queue-slot hit rectangle for the idx-th waiting party
    /// (Director waitqueue_layout coordinates: WaitGroup0 at y=483, step -45)
    fn waiting_group_rect(idx: usize) -> Rect {
        let group_y = 483.0 - (idx as f32) * 45.0;
        Rect::new(Vec2::new(5.0, group_y - 75.0), Vec2::new(145.0, group_y + 15.0))
    }

    /// Legacy waitgroup_layout rectangle for the idx-th waiting party
    fn legacy_waiting_group_rect(idx: usize) -> Rect {
        let legacy_y = 375.0 - (idx as f32) * 45.0;
        Rect::new(Vec2::new(10.0, legacy_y - 60.0), Vec2::new(155.0, legacy_y + 15.0))
    }

    /// Table whose artwork is under `pos`, if any
    fn table_hit(&self, pos: Vec2) -> Option<TableId> {
        let mut table_ids: Vec<TableId> = self.room.tables.keys().copied().collect();
        table_ids.sort_by_key(|t| t.0);
        table_ids.into_iter().find(|tid| {
            let Some(table) = self.room.tables.get(tid) else { return false; };
            let tc = self.table_visual_center(table);
            let half_y = match table.capacity {
                6 => 58.0,
                4 => 46.0,
                _ => 40.0,
            };
            let rect = Rect::new(tc - Vec2::new(58.0, half_y), tc + Vec2::new(58.0, half_y));
            rect.contains(pos)
        })
    }

    /// Original `table.canHold`: enough chairs, nobody seated, table not dirty
    pub fn can_seat_group_at(&self, table_id: TableId, group_id: GroupId) -> bool {
        let (Some(table), Some(group)) = (
            self.room.tables.get(&table_id),
            self.sitters.get_group(group_id),
        ) else {
            return false;
        };
        table.state == TableState::Clean
            && table.group_id.is_none()
            && group.sitters.len() <= table.capacity as usize
    }

    /// Press on a waiting party to pick it up (original `setDragging(1)`),
    /// otherwise fall through to the plain click handling.
    pub fn pointer_down(&mut self, pos: Vec2) -> bool {
        self.hover_pos = Some(pos);
        // Podium (`zOffset: 400000` in `room_manager.ls` line 104) sits in front of `waitqueue` at (83, 515)
        if self.room.podium.is_some() {
            let (_, _, _, podium_pos) = self.stage_station_positions();
            let podium_rect = Rect::new(podium_pos - Vec2::new(38.0, 40.0), podium_pos + Vec2::new(40.0, 65.0));
            if podium_rect.contains(pos) && self.click_podium() {
                return true;
            }
        }
        for (idx, &group_id) in self.sitters.waiting_groups.iter().enumerate() {
            if Self::waiting_group_rect(idx).contains(pos)
                || Self::legacy_waiting_group_rect(idx).contains(pos)
            {
                if let Some(g) = self.sitters.get_group_mut(group_id) {
                    g.picked_up = true;
                }
                self.selected_waiting_group = Some(group_id);
                self.drag = Some(DragState {
                    group: group_id,
                    pos,
                    down_pos: pos,
                    moved: false,
                    hover_table: None,
                });
                return true;
            }
        }
        self.handle_click(pos)
    }

    /// Move the dragged party; a valid table previews the seated group
    pub fn pointer_move(&mut self, pos: Vec2) {
        self.hover_pos = Some(pos);
        if self.drag.is_none() {
            return;
        }
        let group = self.drag.as_ref().unwrap().group;
        let hover = self.table_hit(pos).filter(|&tid| self.can_seat_group_at(tid, group));
        let drag = self.drag.as_mut().unwrap();
        drag.pos = pos;
        if pos.distance(drag.down_pos) > 6.0 {
            drag.moved = true;
        }
        drag.hover_table = hover;
    }

    /// Release the dragged party: seat it on a valid table, otherwise
    /// the party simply returns to the queue.
    pub fn pointer_up(&mut self, pos: Vec2) -> bool {
        self.hover_pos = Some(pos);
        let Some(drag) = self.drag.take() else { return false; };
        if let Some(g) = self.sitters.get_group_mut(drag.group) {
            g.picked_up = false;
        }
        let Some(table_id) = self
            .table_hit(pos)
            .filter(|&tid| self.can_seat_group_at(tid, drag.group))
        else {
            return false;
        };
        // Count matching chair colors BEFORE seating (`chair.ls` lines 40-60: baseChairBonus = 100 per matching seat)
        let mut color_match_points = 0u32;
        if let (Some(table), Some(group)) = (
            self.room.tables.get(&table_id),
            self.sitters.get_group(drag.group),
        ) {
            for (i, sitter) in group.sitters.iter().enumerate() {
                if let Some(seat) = table.seats.get(i) {
                    let scol_str = match sitter.color {
                        CustomerColor::Red => "red",
                        CustomerColor::Blue => "blue",
                        CustomerColor::Green => "green",
                        CustomerColor::Yellow => "yellow",
                        CustomerColor::Purple => "purple",
                        CustomerColor::None => "none",
                    };
                    if seat.color != "grey" && scol_str != "none" && seat.color == scol_str {
                        color_match_points += seat.bonus_multiplier * 100;
                    }
                }
            }
        }
        if self.sitters.seat_group_at_table(drag.group, table_id, &mut self.room) {
            self.waiting_groups = self.sitters.waiting_groups.clone();
            self.selected_waiting_group = self.waiting_groups.front().copied();
            if color_match_points > 0 {
                self.score_tip(color_match_points);
            }
            return true;
        }
        false
    }

    /// Handle a pointer click in canvas coordinates (800x600 space)
    pub fn handle_click(&mut self, pos: Vec2) -> bool {
        let (ticket_pos, food_slots, bus_pos, podium_pos) = self.stage_station_positions();

        // 0. Check Endless Mode Upgrade Button (`upgrade_interface_layout.txt`: `rect=(659, 12, 783, 81)`)
        if self.mode == GameMode::Endless && self.show_upgrade_button {
            let upg_rect = Rect::new(Vec2::new(655.0, 10.0), Vec2::new(788.0, 85.0));
            if upg_rect.contains(pos) {
                self.accumulated_upgrades = self.accumulated_upgrades.saturating_sub(1);
                self.show_upgrade_button = self.accumulated_upgrades > 0;
                if self.room.tables.len() == 4 {
                    self.room.add_table_with_seats(
                        GridPos::new(9, 8),
                        &[GridPos::new(8, 8), GridPos::new(10, 8), GridPos::new(10, 9), GridPos::new(8, 9)],
                    );
                } else if self.room.tables.len() == 5 {
                    self.room.add_table_with_seats(
                        GridPos::new(9, 5),
                        &[GridPos::new(8, 5), GridPos::new(10, 5), GridPos::new(10, 6), GridPos::new(8, 6)],
                    );
                } else {
                    self.waiter.set_speed(self.waiter.speed + 35.0);
                    let next_cook = self.room.kitchen.food_ready_ms.saturating_sub(600).max(2000);
                    self.room.kitchen.set_speed(next_cook);
                }
                return true;
            }
        }

        // 1. Check Ticket Station
        let ticket_rect = Rect::new(ticket_pos - Vec2::new(40.0, 50.0), ticket_pos + Vec2::new(38.0, 45.0));
        if ticket_rect.contains(pos) && self.click_ticket_station() {
            return true;
        }

        // 2. Check Kitchen Counter food pass
        let kitchen_rect = Rect::new(
            Vec2::new(food_slots[0].x - 36.0, food_slots[0].y - 50.0),
            Vec2::new(food_slots[3].x + 36.0, food_slots[0].y + 45.0),
        );
        if kitchen_rect.contains(pos) && self.click_kitchen_at(pos) {
            return true;
        }

        // 3. Check Bus Station
        let bus_rect = Rect::new(bus_pos - Vec2::new(55.0, 55.0), bus_pos + Vec2::new(55.0, 55.0));
        if bus_rect.contains(pos) && self.click_bus_station() {
            return true;
        }

        // 4. Check Podium area
        if self.room.podium.is_some() {
            let podium_rect = Rect::new(podium_pos - Vec2::new(42.0, 50.0), podium_pos + Vec2::new(42.0, 65.0));
            if podium_rect.contains(pos) && self.click_podium() {
                return true;
            }
        }

        // 5. Check Tables
        if let Some(tid) = self.table_hit(pos) {
            return self.click_table(tid);
        }

        false
    }

    /// Advance the 24-step sequential tutorial (`tutorial_1.ls` .. `tutorial_24.ls`)
    pub fn update_tutorial_step(&mut self, dt: f32) {
        if self.tutorial_step == 0 || self.tutorial_step > 24 {
            return;
        }
        self.tutorial_step_timer += dt;

        let seated_groups: Vec<&crate::sitter::SitterGroup> = self
            .room
            .tables
            .values()
            .filter_map(|t| t.group_id.and_then(|gid| self.sitters.get_group(gid)))
            .collect();

        let advance = match self.tutorial_step {
            // tutorial_1.ls: drag a group from queue (`group.pickedUp`) or already seated
            1 => {
                self.drag.is_some()
                    || self.sitters.waiting_groups.is_empty()
                    || !seated_groups.is_empty()
            }
            // tutorial_2.ls: release to seat the first group (`waitqueue.groups.count == 0`)
            2 => self.sitters.waiting_groups.is_empty() || !seated_groups.is_empty(),
            // tutorial_3.ls: wait until seated group finishes looking at menu (`Ready`)
            3 => seated_groups.iter().any(|g| {
                g.sitters
                    .first()
                    .map(|s| s.state != SitterState::Deciding)
                    .unwrap_or(false)
            }),
            // tutorial_4.ls: click table to pick up order (`waiter.tickets.count == 1`)
            4 => {
                !self.waiter.carried_tickets.is_empty()
                    || !self.room.kitchen.current_orders.is_empty()
                    || !self.ready_orders.is_empty()
            }
            // tutorial_5.ls: drop ticket at ticket station (`kitchen.timers.count == 1`)
            5 => {
                !self.room.kitchen.current_orders.is_empty()
                    || !self.ready_orders.is_empty()
                    || !self.waiter.carried_food.is_empty()
            }
            // tutorial_6.ls: chef finishes preparing food (`food_1.occupied`)
            6 => {
                !self.ready_orders.is_empty()
                    || !self.waiter.carried_food.is_empty()
                    || seated_groups.iter().any(|g| {
                        g.sitters
                            .first()
                            .map(|s| matches!(s.state, SitterState::Eating | SitterState::Check | SitterState::Scoring))
                            .unwrap_or(false)
                    })
            }
            // tutorial_7.ls: pick up food tray from pass (`not food_1.occupied`)
            7 => {
                !self.waiter.carried_food.is_empty()
                    || seated_groups.iter().any(|g| {
                        g.sitters
                            .first()
                            .map(|s| matches!(s.state, SitterState::Eating | SitterState::Check | SitterState::Scoring))
                            .unwrap_or(false)
                    })
            }
            // tutorial_8.ls: deliver food to table (`group.state == Eating`)
            8 => seated_groups.iter().any(|g| {
                g.sitters
                    .first()
                    .map(|s| matches!(s.state, SitterState::Eating | SitterState::Check | SitterState::Scoring))
                    .unwrap_or(false)
            }),
            // tutorial_9.ls: wait for customers to finish eating (`group.state == Check`)
            9 => {
                seated_groups.iter().any(|g| {
                    g.sitters
                        .first()
                        .map(|s| matches!(s.state, SitterState::Check | SitterState::Scoring))
                        .unwrap_or(false)
                }) || self.has_dirty_tables()
            }
            // tutorial_10.ls: deliver check (`group.state == Scoring` or table dirty)
            10 => {
                seated_groups.iter().any(|g| {
                    g.sitters
                        .first()
                        .map(|s| matches!(s.state, SitterState::Scoring | SitterState::Leaving))
                        .unwrap_or(false)
                }) || self.has_dirty_tables()
            }
            // tutorial_11.ls: customers leave tip & vacate table (`table.dirty`)
            11 => self.has_dirty_tables() || self.waiter.carried_dishes > 0,
            // tutorial_12.ls: pick up dirty dishes (`not table.dirty`)
            12 => {
                (!self.has_dirty_tables() && self.waiter.carried_dishes > 0)
                    || self.waiter.bus_dishes_count > 0
            }
            // tutorial_13.ls: drop dishes at bus station (`waiter.dishes == 0`) -> spawns 2 new groups!
            13 => self.waiter.carried_dishes == 0 && self.waiter.bus_dishes_count > 0,
            // tutorial_14.ls: seat both new groups (`waitqueue.groups.count == 0`)
            14 => self.sitters.waiting_groups.is_empty() && seated_groups.len() >= 2,
            // tutorial_15.ls: wait until both groups are ready to order (`Ready`)
            15 => {
                seated_groups.len() >= 2
                    && seated_groups.iter().all(|g| {
                        g.sitters
                            .first()
                            .map(|s| s.state != SitterState::Deciding)
                            .unwrap_or(false)
                    })
            }
            // tutorial_16.ls: queue both tables + ticket station, or tickets picked up/dropped
            16 => {
                self.waiter.carried_tickets.len() >= 2
                    || self.tutorial_wave2_orders_sent >= 2
                    || (self.waiter.task_queue.len() >= 2 && !self.waiter.carried_tickets.is_empty())
            }
            // tutorial_17.ls: wait until both orders dropped at ticket station
            17 => self.tutorial_wave2_orders_sent >= 2 || self.ready_orders.len() >= 2,
            // tutorial_18.ls: wait until both food trays are ready on the pass
            18 => {
                self.ready_orders.len() >= 2
                    || !self.waiter.carried_food.is_empty()
            }
            // tutorial_19.ls: pick up both trays and deliver to both tables (`both Eating`)
            19 => {
                seated_groups.len() >= 2
                    && seated_groups.iter().all(|g| {
                        g.sitters
                            .first()
                            .map(|s| matches!(s.state, SitterState::Eating | SitterState::Check | SitterState::Scoring))
                            .unwrap_or(false)
                    })
            }
            // tutorial_20..23.ls: timed HUD explanation bubbles (4.0s each)
            20 | 21 | 22 | 23 => self.tutorial_step_timer >= 4.0,
            // tutorial_24.ls: finish serving the 2 groups and clear all dirty tables
            24 => {
                self.tutorial_step_timer >= 4.0
                    && seated_groups.is_empty()
                    && !self.has_dirty_tables()
                    && self.waiter.carried_dishes == 0
            }
            _ => false,
        };

        if advance {
            let prev = self.tutorial_step;
            self.tutorial_step += 1;
            self.tutorial_step_timer = 0.0;
            // `tutorial_13.ls` lines 25-26: when step 13 completes, immediately spawn the 2 wave-2 groups!
            if prev == 13 && self.sitters.waiting_groups.is_empty() {
                for _ in 0..2 {
                    if let Some(cfg) = self.group_spawn_queue.pop_front() {
                        let _ = self.new_groups.pop_front();
                        self.spawn_group_from_config(cfg);
                    }
                }
                if self.group_spawn_queue.is_empty() && self.mode == GameMode::Career {
                    self.door_closing_sign = true;
                }
            }
        }
    }

    /// Render the entire restaurant state using extracted Macromedia Director cast members
    pub fn render(&self, renderer: &mut dyn Renderer) {
        renderer.begin_frame();
        renderer.clear(Vec4::new(0.08, 0.06, 0.10, 1.0));

        let anim_tick = (self.current_ms / 150) as usize;
        let bg_state = self
            .level_data
            .as_ref()
            .map(|l| l.bg_state.as_str())
            .unwrap_or("state_1");
        let has_drink_station = self.room.has_snacks() || !self.available_snacks.is_empty();

        let stage_cast = match self.stage {
            1 => "gfx.diner",
            2 => "gfx.tiki",
            3 => "gfx.seafood",
            4 => "gfx.chez",
            5 => "gfx.goddess",
            _ => "gfx.diner",
        };

        // Exact Chef animation from `chef.ls`, `kitchen.ls`, `bg_*_layout.txt`, and `chef_layout.txt`:
        // Note: `stove_chefanim` in `chef_layout.txt` is an invisible layout bounds rect (`312x78` solid brown),
        // NOT drawn in the live game. Only `chef_anim` (`1_stur..4stur_` / `1cut..5cut`) is rendered when cooking!
        let chef_origin = match self.stage {
            2 | 4 => Vec2::new(371.0, 76.0),
            3 | 5 => Vec2::new(371.0, 77.0),
            _ => Vec2::new(371.0, 75.0),
        };
        if !self.room.kitchen.current_orders.is_empty() {
            const CHEF_FRAMES: &[(&str, f32, f32)] = &[
                // "fry" loop (2x)
                ("gfx.people/1_stur", 95.0, 40.0),
                ("gfx.people/2stur", 92.0, 40.0),
                ("gfx.people/3stur", 89.0, 40.0),
                ("gfx.people/4stur_", 92.0, 40.0),
                ("gfx.people/1_stur", 95.0, 40.0),
                ("gfx.people/2stur", 92.0, 40.0),
                ("gfx.people/3stur", 89.0, 40.0),
                ("gfx.people/4stur_", 92.0, 40.0),
                // "chop" loop (2x)
                ("gfx.people/1cut", 129.0, 41.0),
                ("gfx.people/2cut", 132.0, 41.0),
                ("gfx.people/3cut", 134.0, 41.0),
                ("gfx.people/4cut", 134.0, 41.0),
                ("gfx.people/5cut", 138.0, 41.0),
                ("gfx.people/1cut", 129.0, 41.0),
                ("gfx.people/2cut", 132.0, 41.0),
                ("gfx.people/3cut", 134.0, 41.0),
                ("gfx.people/4cut", 134.0, 41.0),
                ("gfx.people/5cut", 138.0, 41.0),
            ];
            let chef_tick = ((self.current_ms / 120) as usize) % CHEF_FRAMES.len();
            let (chef_member, ox, oy) = CHEF_FRAMES[chef_tick];
            draw_member(
                renderer,
                chef_member,
                chef_origin + Vec2::new(ox, oy),
                false,
                2,
            );
        }

        // 1. Restaurant Stage Background, Floor, Counter, and Upgrades (Director scene_layouts)
        match self.stage {
            2 => {
                draw_member(renderer, "gfx.tiki/tiki_chef_background", Vec2::new(488.0, 109.0), false, 1);
                draw_member(renderer, "gfx.tiki/R2_bkg", Vec2::new(400.0, 300.0), false, 3);
                draw_member(renderer, "gfx.tiki/R2_counter", Vec2::new(457.0, 185.0), false, 6);
                if has_drink_station {
                    draw_member(renderer, "gfx.tiki/R2_drink_up", Vec2::new(617.0, 144.0), false, 8);
                }
                if matches!(bg_state, "state_2" | "state_3" | "state_4") {
                    draw_member(renderer, "gfx.tiki/R2_upgrade_1", Vec2::new(461.0, 154.0), false, 5);
                }
                if matches!(bg_state, "state_3" | "state_4") {
                    draw_member(renderer, "gfx.tiki/R2_upgrade_2", Vec2::new(114.0, 312.0), false, 5);
                }
                if bg_state == "state_4" {
                    draw_member(renderer, "gfx.tiki/R2_upgrade_3", Vec2::new(388.0, 185.0), false, 7);
                }
                if self.door_closing_sign {
                    draw_member(renderer, "gfx.tiki/closing_sign", Vec2::new(94.0, 188.0), false, 9);
                }
            }
            3 => {
                draw_member(renderer, "gfx.seafood/seafood_chef_background_r", Vec2::new(475.0, 128.0), false, 1);
                draw_member(renderer, "gfx.seafood/R3_floor", Vec2::new(400.0, 300.0), false, 3);
                draw_member(renderer, "gfx.seafood/R3_counter", Vec2::new(453.0, 186.0), false, 6);
                if has_drink_station {
                    draw_member(renderer, "gfx.seafood/R3_drink_up", Vec2::new(617.0, 141.0), false, 8);
                }
                if matches!(bg_state, "state_2" | "state_3" | "state_4") {
                    draw_member(renderer, "gfx.seafood/R3_upgrade_1", Vec2::new(459.0, 207.0), false, 5);
                }
                if matches!(bg_state, "state_3" | "state_4") {
                    draw_member(renderer, "gfx.seafood/R3_upgrade_2", Vec2::new(100.0, 236.0), false, 5);
                }
                if bg_state == "state_4" {
                    draw_member(renderer, "gfx.seafood/R3_upgrade_3", Vec2::new(400.0, 180.0), false, 7);
                }
                if self.door_closing_sign {
                    draw_member(renderer, "gfx.seafood/closing_sign", Vec2::new(90.0, 172.0), false, 9);
                }
            }
            4 => {
                draw_member(renderer, "gfx.chez/chez_chef_background_revi", Vec2::new(501.0, 131.0), false, 1);
                draw_member(renderer, "gfx.chez/R4_Floor", Vec2::new(400.0, 300.0), false, 3);
                draw_member(renderer, "gfx.chez/R4_counter", Vec2::new(400.0, 374.0), false, 6);
                if has_drink_station {
                    draw_member(renderer, "gfx.chez/R4_drink_on", Vec2::new(625.0, 136.0), false, 8);
                }
                if matches!(bg_state, "state_2" | "state_3" | "state_4") {
                    draw_member(renderer, "gfx.chez/R4_upgrade_1", Vec2::new(340.0, 166.0), false, 5);
                }
                if matches!(bg_state, "state_3" | "state_4") {
                    draw_member(renderer, "gfx.chez/R4_upgrade_2", Vec2::new(102.0, 306.0), false, 5);
                }
                if bg_state == "state_4" {
                    draw_member(renderer, "gfx.chez/R4_upgrade_3a", Vec2::new(137.0, 160.0), false, 5);
                    draw_member(renderer, "gfx.chez/R4_upgrade_3b", Vec2::new(707.0, 118.0), false, 7);
                }
                if self.door_closing_sign {
                    draw_member(renderer, "gfx.chez/closing_sign", Vec2::new(109.0, 167.0), false, 9);
                }
            }
            5 => {
                draw_member(renderer, "gfx.goddess/goddessbg_revised", Vec2::new(500.0, 118.0), false, 1);
                draw_member(renderer, "gfx.goddess/R5_floor", Vec2::new(400.0, 300.0), false, 3);
                draw_member(renderer, "gfx.goddess/R5_counter", Vec2::new(457.0, 188.0), false, 6);
                if has_drink_station {
                    draw_member(renderer, "gfx.goddess/R5_drink_on", Vec2::new(615.0, 132.0), false, 8);
                }
                if self.door_closing_sign {
                    draw_member(renderer, "gfx.goddess/closing_sign", Vec2::new(103.0, 157.0), false, 9);
                }
            }
            _ => {
                // Authentic z-layering from bg_diner_layout.txt:
                draw_member(renderer, "gfx.diner/flo_s_diner_revised", Vec2::new(479.0, 119.0), false, 1);
                draw_member(renderer, "gfx.diner/R1_floor", Vec2::new(400.0, 300.0), false, 3);
                if matches!(bg_state, "state_0" | "state_1" | "state_2" | "state_3") {
                    draw_member(renderer, "gfx.diner/R1_remove_3a", Vec2::new(28.0, 147.0), false, 4);
                    draw_member(renderer, "gfx.diner/R1_remove_3c", Vec2::new(743.0, 169.0), false, 4);
                }
                if matches!(bg_state, "state_0" | "state_1") {
                    draw_member(renderer, "gfx.diner/R1_remove_1a", Vec2::new(400.0, 413.0), false, 4);
                }
                if matches!(bg_state, "state_0" | "state_1" | "state_2") {
                    draw_member(renderer, "gfx.diner/R1_remove_2", Vec2::new(91.0, 305.0), false, 5);
                }
                draw_member(renderer, "gfx.diner/R1_counter", Vec2::new(526.0, 370.0), false, 6);
                if matches!(bg_state, "state_0" | "state_1" | "state_2" | "state_3") {
                    draw_member(renderer, "gfx.diner/R1_remove_3b", Vec2::new(402.0, 164.0), false, 7);
                }
                if matches!(bg_state, "state_0" | "state_1") {
                    draw_member(renderer, "gfx.diner/R1_remove_1b", Vec2::new(304.0, 150.0), false, 8);
                    draw_member(renderer, "gfx.diner/R1_remove_1c", Vec2::new(591.0, 141.0), false, 8);
                }
                if has_drink_station {
                    draw_member(renderer, "gfx.diner/R1_drink_on", Vec2::new(613.0, 132.0), false, 9);
                }
                if self.door_closing_sign {
                    draw_member(renderer, "gfx.diner/closing_sign", Vec2::new(87.0, 188.0), false, 9);
                }
            }
        }

        // 2. Interactive Room Stations (`bg_*_layout.txt`, `room_layout.txt` & `d_food_layout.txt` coordinates)
        let (ticket_pos, food_slots, bus_pos, podium_pos) = self.stage_station_positions();
        let ticket_sprite = if !self.room.kitchen.current_orders.is_empty() {
            "gfx.room/ticket_station__on"
        } else {
            "gfx.room/ticket_station__1"
        };
        draw_member(renderer, ticket_sprite, ticket_pos, false, 10);

        // Ready orders on kitchen pass: food_1..4
        // Per `d_food_layout.txt`, each food spot draws `<stage_cast>/food_tray` + `tray_number_flag` + `tablenum_<N>`!
        let pass_tray_key = format!("{}/food_tray", stage_cast);
        for (idx, order) in self.ready_orders.iter().take(4).enumerate() {
            let slot_pos = food_slots[idx];
            draw_member(renderer, &pass_tray_key, slot_pos, false, 11);
            draw_member(renderer, "gfx.room/tray_number_flag", slot_pos + Vec2::new(7.0, -18.0), false, 12);
            let tnum_key = format!("gfx.numbers/tablenum_{}", order.table_id.0.clamp(1, 16));
            draw_member(renderer, &tnum_key, slot_pos + Vec2::new(8.0, -20.0), false, 13);
        }

        // Bus station (`bus_station_layout.txt`: only `bus_0` / `bus_1` / `bus_2`, no `BUS_STATION_copy`)
        let bus_fill_key = match self.waiter.bus_dishes_count {
            0 => "gfx.room/bus_0",
            1 => "gfx.room/bus_1",
            _ => "gfx.room/bus_2",
        };
        draw_member(renderer, bus_fill_key, bus_pos, false, 11);

        // Podium (`podium_layout.txt`: `podium` at (83, 515), `podium_heart` at (86, 533))
        if let Some(podium) = &self.room.podium {
            // z = 44: black interior behind heart cut-out
            draw_member(renderer, "gfx.room/podium_black", podium_pos + Vec2::new(-20.0, -3.0), false, 44);

            // z = 45: heart window (charging frames 1..4 or full heart)
            if podium.charge_progress > 0.0 {
                let frame = ((podium.charge_progress * 4.0).floor() as u32 + 1).clamp(1, 4);
                draw_member(renderer, &format!("gfx.people/heart_{}", frame), podium_pos + Vec2::new(3.0, 18.0), false, 45);
            } else if podium.active_timer.is_some() || podium.ready {
                draw_member(renderer, "gfx.room/podium_heart", podium_pos + Vec2::new(3.0, 18.0), false, 45);
            }

            // z = 46: wood podium body
            draw_member(renderer, "gfx.room/podium", podium_pos, false, 46);

            // z = 47: 10 floating hearts around window when ready or active
            if podium.ready || podium.active_timer.is_some() {
                let heart_offsets = [
                    Vec2::new(-21.0, 17.0),
                    Vec2::new(-8.0, -7.0),
                    Vec2::new(-12.0, 29.0),
                    Vec2::new(23.0, 21.0),
                    Vec2::new(24.0, 3.0),
                    Vec2::new(-21.0, 0.0),
                    Vec2::new(4.0, 0.0),
                    Vec2::new(15.0, -7.0),
                    Vec2::new(3.0, 40.0),
                    Vec2::new(13.0, 32.0),
                ];
                for offset in heart_offsets {
                    draw_member(renderer, "gfx.people/heart", podium_pos + offset, false, 47);
                }
            }

            // z = 48: checkmark when Flo has a task to use podium
            let has_podium_task = matches!(self.waiter.current_task, Some(WaiterTask::UsePodium))
                || self.waiter.task_queue.iter().any(|t| matches!(t, WaiterTask::UsePodium));
            if has_podium_task {
                draw_member(renderer, "gfx.room/checkmark", podium_pos + Vec2::new(22.0, -48.0), false, 48);
            }
        }

        // 3. Waiting Customer Queue on Left (`waitqueue_layout.txt`, `waitgroup_layout.txt`, `stander_view.ls`, `heartrow_view.ls`)
        let dragged_group = self.drag.as_ref().map(|d| d.group);
        for (idx, &group_id) in self.sitters.waiting_groups.iter().take(8).enumerate() {
            if dragged_group == Some(group_id) {
                continue;
            }
            if let Some(group) = self.sitters.get_group(group_id) {
                let base_y = 483.0 - (idx as f32) * 45.0;
                let group_z = 36 - (idx as i32) * 2;
                let (stand_prefix, _, _) = customer_sprite_names(&group.group_type);
                let is_hovered = self.drag.is_none()
                    && self
                        .hover_pos
                        .map(|hp| Self::waiting_group_rect(idx).contains(hp))
                        .unwrap_or(false);
                let m = group.mood_points.clamp(1, 10) as f32;
                let frame_num = if is_hovered {
                    4
                } else {
                    (((m / 3.3).round() as i32) + 1).clamp(1, 3)
                };

                for (s_idx, sitter) in group.sitters.iter().enumerate() {
                    let cast = customer_color_cast(sitter.color, &group.group_type);
                    let member_key = format!("{}/{}_{}", cast, stand_prefix, frame_num);
                    let sx = 27.0 + (s_idx as f32) * 21.0;
                    let sy = base_y - (s_idx as f32) * 2.0;
                    draw_member(
                        renderer,
                        &member_key,
                        Vec2::new(sx, sy),
                        false,
                        group_z + (s_idx as i32),
                    );
                }

                // Heart meter above waiting group (`heartrow_view.ls`: 1..10 hearts = `sitterGroup.mood`)
                let hearts = group.mood_points.clamp(1, 10);
                let heart_key = format!("gfx.people/heartrow_{}", hearts);
                let heart_pos = Vec2::new(55.0, base_y - 68.0);
                draw_member(renderer, &heart_key, heart_pos, false, group_z + 1);
            }
        }

        // Party held by the cursor during a drag (`waitgroup_drag.ls`)
        if let Some(drag) = &self.drag {
            if drag.hover_table.is_none() {
                if let Some(group) = self.sitters.get_group(drag.group) {
                    let (stand_prefix, _, _) = customer_sprite_names(&group.group_type);
                    let m = group.mood_points.clamp(1, 10) as f32;
                    let frame_num = (((m / 3.3).round() as i32) + 1).clamp(1, 3);
                    for (s_idx, sitter) in group.sitters.iter().enumerate() {
                        let cast = customer_color_cast(sitter.color, &group.group_type);
                        let member_key = format!("{}/{}_{}", cast, stand_prefix, frame_num);
                        let sx = drag.pos.x - 24.0 + (s_idx as f32) * 21.0;
                        let sy = drag.pos.y - (s_idx as f32) * 2.0;
                        draw_member(renderer, &member_key, Vec2::new(sx, sy), false, 300 + s_idx as i32);
                    }
                    let hearts = group.mood_points.clamp(1, 10);
                    let heart_key = format!("gfx.people/heartrow_{}", hearts);
                    let heart_pos = Vec2::new(drag.pos.x, drag.pos.y - 68.0);
                    draw_member(renderer, &heart_key, heart_pos, false, 308);
                }
            }
        }

        // 4. Tables, Chairs, and Seated Customers (`2top_layout.txt`, `4top_layout.txt`, `6top_layout.txt`, `sitter_view.ls`, `table_view.ls`)
        let mut table_ids: Vec<TableId> = self.room.tables.keys().copied().collect();
        table_ids.sort_by_key(|t| t.0);
        for tid in table_ids {
            if let Some(table) = self.room.tables.get(&tid) {
                let tc = self.table_visual_center(table);
                let base_z = ((tc.y as i32) / 4).clamp(15, 120);

                let (table_member, dirty_member, card_off, num_off, shadow_off, heart_off, chair_slots, setting_slots): (
                    &str,
                    &str,
                    Vec2,
                    Vec2,
                    Vec2,
                    Vec2,
                    &[(Vec2, bool, i32)],
                    &[Vec2],
                ) = match table.capacity {
                    4 => (
                        "table_4top",
                        "gfx.room/dirty_4top",
                        Vec2::new(0.0, -13.0),
                        Vec2::new(-3.0, -16.0),
                        Vec2::new(-1.0, 35.0),
                        Vec2::new(1.0, 23.0),
                        &[
                            (Vec2::new(-37.0, -9.0), true, -2),
                            (Vec2::new(35.0, -10.0), false, -2),
                            (Vec2::new(38.0, 14.0), false, 1),
                            (Vec2::new(-36.0, 14.0), true, 1),
                        ],
                        &[
                            Vec2::new(-18.0, -19.0),
                            Vec2::new(18.0, -19.0),
                            Vec2::new(14.0, 5.0),
                            Vec2::new(-16.0, 5.0),
                        ],
                    ),
                    6 => (
                        "table_6top",
                        "gfx.room/dirty_6top",
                        Vec2::new(0.0, -11.0),
                        Vec2::new(-2.0, -14.0),
                        Vec2::new(0.0, 42.0),
                        Vec2::new(0.0, 29.0),
                        &[
                            (Vec2::new(-33.0, -24.0), true, -3),
                            (Vec2::new(34.0, -24.0), false, -3),
                            (Vec2::new(36.0, -3.0), false, -1),
                            (Vec2::new(39.0, 21.0), false, 1),
                            (Vec2::new(-37.0, 21.0), true, 1),
                            (Vec2::new(-35.0, -3.0), true, -1),
                        ],
                        &[
                            Vec2::new(-18.0, -33.0),
                            Vec2::new(17.0, -32.0),
                            Vec2::new(17.0, -10.0),
                            Vec2::new(17.0, 15.0),
                            Vec2::new(-17.0, 15.0),
                            Vec2::new(-18.0, -10.0),
                        ],
                    ),
                    _ => (
                        "table_2top",
                        "gfx.room/dirty_2top",
                        Vec2::new(0.0, -17.0),
                        Vec2::new(-3.0, -20.0),
                        Vec2::new(-2.0, 26.0),
                        Vec2::new(1.0, 14.0),
                        &[
                            (Vec2::new(-38.0, 4.0), true, 0),
                            (Vec2::new(38.0, 3.0), false, 0),
                        ],
                        &[
                            Vec2::new(-14.0, -11.0),
                            Vec2::new(13.0, -11.0),
                        ],
                    ),
                };

                // Table floor shadow (2top_layout.txt #z: 1)
                draw_member(renderer, "gfx.room/table_shadow", tc + shadow_off, false, base_z - 5);

                let seated_group = table.group_id.and_then(|gid| self.sitters.get_group(gid));
                let preview_drag = self.drag.as_ref().and_then(|d| d.hover_table) == Some(tid);
                let seated_group = seated_group.or_else(|| {
                    if preview_drag {
                        self.drag.as_ref().and_then(|d| self.sitters.get_group(d.group))
                    } else {
                        None
                    }
                });
                let carrying_for_this_table = self.waiter.carried_food.contains(&tid);

                // Chairs & seated customers (2top_layout.txt: chair z=2..3, legs z=7..8, table z=9, sitter z=26..27)
                for (s_idx, &(chair_off, flip_x, z_delta)) in chair_slots.iter().enumerate() {
                    let seat_color = table
                        .seats
                        .get(s_idx)
                        .map(|s| s.color.as_str())
                        .unwrap_or("none");
                    let chair_col = match seat_color {
                        "red" => "red",
                        "blue" => "blue",
                        "green" => "green",
                        "yellow" | "orange" => "yellow",
                        "purple" => "purple",
                        "grey" => "grey",
                        _ => "none",
                    };
                    let chair_key = format!("{}/chair_{}", stage_cast, chair_col);
                    let chair_pos = tc + chair_off;
                    draw_member(renderer, &chair_key, chair_pos, flip_x, base_z + z_delta - 3);

                    if let Some(group) = seated_group {
                        if let Some(sitter) = group.sitters.get(s_idx) {
                            let cast = customer_color_cast(sitter.color, &group.group_type);
                            let (_, legs_name, state_prefix) = customer_sprite_names(&group.group_type);
                            let legs_key = format!("{}/{}", cast, legs_name);
                            // Legs are drawn at chair_pos using authentic 2top_layout regPoints, behind tabletop!
                            draw_member(
                                renderer,
                                &legs_key,
                                chair_pos,
                                flip_x,
                                base_z + z_delta - 1,
                            );

                            // Exact `sitter_view.ls` state & mood animation selection:
                            let eat_tick = ((self.current_ms / 400) as usize) + s_idx;
                            let base_suffix = if preview_drag || sitter.state == SitterState::Deciding {
                                "state_reading"
                            } else {
                                match sitter.state {
                                    SitterState::Ready | SitterState::Snack => "state_callwaiter",
                                    SitterState::Ordered => {
                                        if carrying_for_this_table {
                                            if state_prefix == "critic" {
                                                "state_prefood_phas"
                                            } else if matches!(state_prefix, "boy" | "man") {
                                                "state_prefood_phase2"
                                            } else {
                                                "state_prefood"
                                            }
                                        } else if state_prefix == "girl" {
                                            "mood_normal_1"
                                        } else {
                                            "mood_normal"
                                        }
                                    }
                                    SitterState::Eating => {
                                        if eat_tick % 2 == 0 {
                                            "state_eating1"
                                        } else {
                                            "state_eating2"
                                        }
                                    }
                                    SitterState::Check => "state_callwaiter",
                                    SitterState::Scoring => {
                                        if group.mood_points > 7 || state_prefix == "critic" {
                                            "mood_happy"
                                        } else if group.mood_points < 4 {
                                            "mood_angry"
                                        } else {
                                            "state_scoring"
                                        }
                                    }
                                    SitterState::Ditching => "mood_angry",
                                    _ => "state_reading",
                                }
                            };

                            // `sitter_view.ls` lines 199-227 (`updateMood`):
                            // When `mood` changes, customer briefly plays `"happy"` + floating hearts (`heart_1..4`)
                            // or `"mad"` + annoyed lines (`annoyed_1..4`); if `mood == 1`, stays in `"mad"` + `"worse"`!
                            let state_suffix = if !preview_drag
                                && !matches!(sitter.state, SitterState::Eating | SitterState::Deciding)
                                && (group.mood_anim_timer > 0.0 || group.mood_points <= 1)
                            {
                                if group.last_mood_delta > 0 && group.mood_points > 1 {
                                    "mood_happy"
                                } else {
                                    "mood_angry"
                                }
                            } else {
                                base_suffix
                            };

                            // Held menu in customer's hands while reading menu (young_female_layout.txt #z: 3)
                            if sitter.state == SitterState::Deciding || preview_drag {
                                let menu_x = if flip_x { 22.0 } else { -22.0 };
                                draw_member(
                                    renderer,
                                    "gfx.people/menu_held",
                                    chair_pos + Vec2::new(menu_x, -23.0),
                                    flip_x,
                                    base_z + z_delta + 2,
                                );
                            }

                            // Upper body drawn at chair_pos using authentic composite sitter_regpoints, in front of tabletop!
                            let upper_key = format!("{}/{}_{}", cast, state_prefix, state_suffix);
                            draw_member(
                                renderer,
                                &upper_key,
                                chair_pos,
                                flip_x,
                                base_z + z_delta + 3,
                            );

                            // `facenode` status animation (`"better"` = `heart_1..4`, `"worse"` = `annoyed_1..4`)
                            if !preview_drag && (group.mood_anim_timer > 0.0 || group.mood_points <= 1) {
                                let status_frame = ((self.current_ms / 200) % 4) + 1;
                                let status_key = if group.last_mood_delta > 0 && group.mood_points > 1 {
                                    format!("gfx.people/heart_{}", status_frame)
                                } else {
                                    format!("gfx.people/annoyed_{}", status_frame)
                                };
                                draw_member(
                                    renderer,
                                    &status_key,
                                    chair_pos + Vec2::new(0.0, -58.0),
                                    flip_x,
                                    base_z + z_delta + 5,
                                );
                            }
                        }
                    }
                }

                // Tabletop (2top_layout.txt #z: 9 — in front of legs, behind customer upper bodies)
                let table_key = format!("{}/{}", stage_cast, table_member);
                draw_member(renderer, &table_key, tc, false, base_z);

                // Heart meter on front edge of occupied table (`heartrow_view.ls`: 1..10 hearts = `sitterGroup.mood`)
                if let Some(group) = seated_group {
                    let hearts = group.mood_points.clamp(1, 10);
                    draw_member(
                        renderer,
                        &format!("gfx.people/heartrow_{}", hearts),
                        tc + heart_off,
                        false,
                        base_z + 2,
                    );
                }

                // Table number card & number: shown once order is taken or food is ready/carried
                let show_number_tent = seated_group
                    .and_then(|g| g.sitters.first())
                    .map(|s| matches!(s.state, SitterState::Ordered))
                    .unwrap_or(false)
                    || self.ready_orders.iter().any(|o| o.table_id == tid)
                    || carrying_for_this_table;
                if show_number_tent {
                    draw_member(renderer, "gfx.room/number_card", tc + card_off, false, base_z + 4);
                    if carrying_for_this_table {
                        draw_member(renderer, "gfx.room/high_light_state", tc + card_off, false, base_z + 5);
                    }
                    let big_num_key = if tid.0 == 11 {
                        "gfx.numbers/tablenum_11".to_string()
                    } else {
                        format!("gfx.numbers/big_tablenum_{}", tid.0.clamp(1, 16))
                    };
                    draw_member(renderer, &big_num_key, tc + num_off, false, base_z + 6);
                }

                // Highlight table on drag hover
                if preview_drag {
                    draw_member(renderer, "gfx.room/high_light_state", tc + card_off, false, base_z + 5);
                }

                // Checkmarks for queued Flo tasks targeting this table
                let (ck1, ck2, ck3, err_off) = match table.capacity {
                    6 => (
                        Vec2::new(-11.0, -36.0),
                        Vec2::new(14.0, -36.0),
                        Vec2::new(-2.0, -27.0),
                        Vec2::new(1.0, -16.0),
                    ),
                    4 => (
                        Vec2::new(-10.0, -28.0),
                        Vec2::new(15.0, -28.0),
                        Vec2::new(-2.0, -21.0),
                        Vec2::new(1.0, -13.0),
                    ),
                    _ => (
                        Vec2::new(-10.0, -29.0),
                        Vec2::new(11.0, -29.0),
                        Vec2::new(0.0, -22.0),
                        Vec2::new(0.0, -16.0),
                    ),
                };
                let marks = self.waiter.pending_marks(tid);
                for (i, off) in [ck1, ck2, ck3].iter().enumerate().take(marks as usize) {
                    draw_member(renderer, "gfx.room/checkmark", tc + *off, false, base_z + 40 + i as i32);
                }

                // Tabletop settings & overlays (`table_view.ls` lines 410-433 & `food_5_layout.txt`)
                if table.state == TableState::Dirty {
                    draw_member(renderer, dirty_member, tc + Vec2::new(0.0, -11.0), false, base_z + 2);
                } else if let Some(group) = seated_group {
                    if let Some(first) = group.sitters.first() {
                        let num_seated = group.sitters.len().min(setting_slots.len());
                        match first.state {
                            SitterState::Ready => {
                                // Closed menus placed on the table when ready to order (`2top/4top/6top_layout.txt`)
                                for (i, &off) in setting_slots.iter().take(num_seated).enumerate() {
                                    let flip_m = chair_slots.get(i).map(|c| c.1).unwrap_or(false);
                                    draw_member(renderer, "gfx.people/menu_placed", tc + off, flip_m, base_z + 1);
                                }
                            }
                            SitterState::Eating | SitterState::Snack => {
                                // Per `table_view.ls` line 425 & `food_5_layout.txt`:
                                // Plated food animates through 4 frames over `eat_time`: `food_0X_4` (full) -> `_3` -> `_2` -> `_1`.
                                // Note: `food_0X_4..1` already includes the clean white plate! Do NOT draw `gfx.room/dishes` underneath.
                                let eat_dur = group.config.eat_time.max(1.0);
                                let eat_progress = (first.state_timer / eat_dur).clamp(0.0, 0.999);
                                let food_step = 4 - ((eat_progress * 4.0).floor() as i32).clamp(0, 3);
                                for (i, &off) in setting_slots.iter().take(num_seated).enumerate() {
                                    let f_key = format!("gfx.room/food_0{}_{}", (i % 2) + 1, food_step);
                                    draw_member(renderer, &f_key, tc + off, false, base_z + 2);
                                }
                            }
                            SitterState::Check => {
                                // `table_view.ls` line 432: `hideFood` sets plate animation to `"empty"` (`food_01_0` / `food_02_0`).
                                // Customers raise their hand (`state_callwaiter` in `sitter_view.ls`); NO thought bubble or check_gfx yet!
                                for (i, &off) in setting_slots.iter().take(num_seated).enumerate() {
                                    let f_key = format!("gfx.room/food_0{}_0", (i % 2) + 1);
                                    draw_member(renderer, &f_key, tc + off, false, base_z + 2);
                                }
                            }
                            SitterState::Scoring => {
                                // Check has been delivered (`check_gfx` on table while customers play `mood_happy` / `state_scoring` for 1.0s)
                                for (i, &off) in setting_slots.iter().take(num_seated).enumerate() {
                                    let f_key = format!("gfx.room/food_0{}_0", (i % 2) + 1);
                                    draw_member(renderer, &f_key, tc + off, false, base_z + 2);
                                }
                                draw_member(renderer, "gfx.room/check_gfx", tc + Vec2::new(0.0, -16.0), false, base_z + 4);
                            }
                            _ => {}
                        }
                    }
                }

                // Transient floating coin flip when a customer group leaves a tip (`tip_bonus.ls`)
                for &(flip_tid, _tip_amt, flip_timer) in &self.tip_flips {
                    if flip_tid == tid {
                        let progress = (1.0 - (flip_timer / 0.6)).clamp(0.0, 1.0);
                        let coin_frame = ((self.current_ms / 90) % 4) + 1;
                        draw_member(
                            renderer,
                            &format!("gfx.screens/coin_flip_{}", coin_frame),
                            tc + Vec2::new(0.0, -24.0 - progress * 28.0),
                            false,
                            base_z + 45,
                        );
                    }
                }
            }
        }

        // 5. Waiter (Flo) — animated legs + directional upper body + carried items (`waiter_view.ls` & `flo_layout.txt`)
        let wpos = self.waiter.position;
        let flo_z = ((wpos.y as i32) / 4 + 5).clamp(25, 135);
        let moving = self.waiter.position.distance(self.waiter.target_position) > 2.0;
        let fx = self.waiter.facing.x;
        let fy = self.waiter.facing.y;
        let flip_flo = fx < -0.25;
        let walk_frame = (anim_tick % 6) + 1;

        // Build list of carried items for Slot 1 and Slot 2
        enum CarriedItem {
            Food(u32),
            Dishes,
            Ticket,
        }
        let mut carried: Vec<CarriedItem> = Vec::new();
        for food in &self.waiter.carried_food {
            if carried.len() < 2 {
                carried.push(CarriedItem::Food(food.0 as u32));
            }
        }
        for _ in 0..self.waiter.carried_dishes {
            if carried.len() < 2 {
                carried.push(CarriedItem::Dishes);
            }
        }
        for _ in &self.waiter.carried_tickets {
            if carried.len() < 2 {
                carried.push(CarriedItem::Ticket);
            }
        }

        let item_count = carried.len();
        let flo_at_podium = self.room.podium.as_ref().map(|p| p.flo_here).unwrap_or(false);

        if flo_at_podium {
            // Flo at podium: legs hidden, dedicated podium full-body animation plays
            if self.waiter.podium_end_timer.is_some() {
                let frame = ((anim_tick % 6) + 1).clamp(1, 6);
                draw_member(
                    renderer,
                    &format!("gfx.people/podium_end_{:02}", frame),
                    wpos + Vec2::new(0.0, -24.0),
                    false,
                    flo_z,
                );
            } else if self.room.podium.as_ref().map(|p| p.active_timer.is_some()).unwrap_or(false) {
                let frame = ((anim_tick % 7) + 1).clamp(1, 7);
                draw_member(
                    renderer,
                    &format!("gfx.people/podium_loop_{:02}", frame),
                    wpos + Vec2::new(0.0, -24.0),
                    false,
                    flo_z,
                );
            } else {
                let charge = self.room.podium.as_ref().map(|p| p.charge_progress).unwrap_or(0.0);
                if charge < 0.33 {
                    draw_member(
                        renderer,
                        "gfx.people/podium_start_01",
                        wpos + Vec2::new(0.0, -24.0),
                        false,
                        flo_z,
                    );
                } else if charge < 0.66 {
                    draw_member(
                        renderer,
                        "gfx.people/podium_start_02",
                        wpos + Vec2::new(0.0, -24.0),
                        false,
                        flo_z,
                    );
                } else {
                    let frame = ((anim_tick % 7) + 1).clamp(1, 7);
                    draw_member(
                        renderer,
                        &format!("gfx.people/podium_loop_{:02}", frame),
                        wpos + Vec2::new(0.0, -24.0),
                        false,
                        flo_z,
                    );
                }
            }
        } else {
            // Flo normal movement and idle
            if !moving {
                if item_count == 0 {
                    let idle_frame = (anim_tick % 10) + 1;
                    let idle_key = format!("gfx.people/flo_idle_main_{:02}", idle_frame);
                    draw_member(renderer, &idle_key, wpos + Vec2::new(0.0, -24.0), false, flo_z);
                } else {
                    let body_key = if item_count >= 2 {
                        "gfx.people/flo_upper_body_forward_bo"
                    } else {
                        "gfx.people/flo_upper_body_forward_on"
                    };
                    draw_member(renderer, "gfx.people/legs_front_stopped", wpos + Vec2::new(0.0, 2.0), false, flo_z);
                    draw_member(renderer, body_key, wpos + Vec2::new(0.0, -30.0), false, flo_z + 1);
                }
            } else {
                let (legs_key, body_key) = if fy < -0.5 {
                    let b = if item_count >= 2 {
                        "gfx.people/flo_upper_body_backward_b"
                    } else if item_count == 1 {
                        "gfx.people/flo_upper_body_backward_o"
                    } else {
                        "gfx.people/flo_upper_body_backward_n"
                    };
                    (format!("gfx.people/legs_backwards{}", walk_frame), b)
                } else if fy > 0.5 {
                    let b = if item_count >= 2 {
                        "gfx.people/flo_upper_body_forward_bo"
                    } else if item_count == 1 {
                        "gfx.people/flo_upper_body_forward_on"
                    } else {
                        "gfx.people/flo_upper_body_forward_no"
                    };
                    (format!("gfx.people/legs_forward{}", walk_frame), b)
                } else {
                    let b = if item_count >= 2 {
                        "gfx.people/flo_upper_body_sideways_b"
                    } else if item_count == 1 {
                        "gfx.people/flo_upper_body_sideways_o"
                    } else {
                        "gfx.people/flo_upper_body_sideways_n"
                    };
                    (format!("gfx.people/legs_sideways{}", walk_frame), b)
                };
                draw_member(renderer, &legs_key, wpos + Vec2::new(0.0, 2.0), flip_flo, flo_z);
                draw_member(renderer, body_key, wpos + Vec2::new(0.0, -30.0), flip_flo, flo_z + 1);
            }

            // Render carried items in Flo's hands (`waiter_view.ls` & `flo_layout.txt`)
            // Slot 1 (screen-right hand in forward_on, or forward hand in sideways)
            // Slot 2 (screen-left hand in forward_bo)
            let tray_key = format!("{}/food_tray", stage_cast);
            for (slot_idx, item) in carried.iter().enumerate() {
                let (food_offset, payload_offset) = if slot_idx == 0 {
                    // Slot 1 (screen-right hand)
                    if fy < -0.5 || fy > 0.5 || !moving {
                        (Vec2::new(20.0, -30.0), Vec2::new(18.0, -28.0))
                    } else if fx >= 0.25 {
                        (Vec2::new(-20.0, -30.0), Vec2::new(-18.0, -28.0))
                    } else {
                        (Vec2::new(20.0, -30.0), Vec2::new(18.0, -28.0))
                    }
                } else {
                    // Slot 2 (screen-left hand)
                    if fy < -0.5 || fy > 0.5 || !moving {
                        (Vec2::new(-20.0, -30.0), Vec2::new(-18.0, -28.0))
                    } else if fx >= 0.25 {
                        (Vec2::new(20.0, -30.0), Vec2::new(18.0, -28.0))
                    } else {
                        (Vec2::new(-20.0, -30.0), Vec2::new(-18.0, -28.0))
                    }
                };

                match item {
                    CarriedItem::Food(target_table) => {
                        let pos = wpos + food_offset;
                        draw_member(renderer, &tray_key, pos, flip_flo, flo_z + 2);
                        draw_member(renderer, "gfx.room/tray_number_flag", pos + Vec2::new(0.0, -14.0), false, flo_z + 3);
                        draw_member(
                            renderer,
                            &format!("gfx.numbers/tablenum_{}", target_table),
                            pos + Vec2::new(0.0, -14.0),
                            false,
                            flo_z + 4,
                        );
                    }
                    CarriedItem::Dishes => {
                        let pos = wpos + payload_offset;
                        draw_member(renderer, "gfx.room/dishes_payload", pos, false, flo_z + 2);
                    }
                    CarriedItem::Ticket => {
                        let pos = wpos + payload_offset;
                        draw_member(renderer, "gfx.room/ticket_payload", pos, false, flo_z + 2);
                    }
                }
            }
        }

        // 6. Top HUD Interface (`game_interface_layout.txt` & `upgrade_interface_layout.txt`)
        draw_member(renderer, "gfx.room/panel_score", Vec2::new(148.0, 51.0), false, 200);
        let jar_idx = (self.money_jar_level + 1).clamp(1, 10);
        draw_member(
            renderer,
            &format!("gfx.screens/moneyjar_{}", jar_idx),
            Vec2::new(66.0, 49.0),
            false,
            202,
        );

        let num_stars = self.max_stars.clamp(1, 5);
        for i in 0..num_stars {
            let sx = 318.0 + (i as f32) * 40.0;
            draw_member(renderer, "gfx.room/star_off", Vec2::new(sx, 44.0), false, 200);
            if i < self.stars {
                draw_member(renderer, "gfx.room/star_on", Vec2::new(sx, 44.0), false, 201);
            }
        }

        if self.mode == GameMode::Endless {
            draw_member(renderer, "gfx.screens/upgrade_normal_state", Vec2::new(652.0, 48.0), false, 200);
            draw_member(renderer, "gfx.screens/upgrade_flash1", Vec2::new(573.0, 51.0), false, 201);
            draw_member(renderer, "gfx.screens/upgrade_level_lbl", Vec2::new(573.0, 37.0), false, 202);
            if self.show_upgrade_button {
                draw_member(renderer, "gfx.screens/upgrade_active_state", Vec2::new(721.0, 46.0), false, 203);
            }
        } else {
            draw_member(renderer, "gfx.room/level_bg", Vec2::new(593.0, 40.0), false, 200);
            draw_member(renderer, "gfx.room/panel_level_clock", Vec2::new(724.0, 68.0), false, 200);

            // `level_clock.ls` line 18: 12 clock hand frames (`fork_frame_1` .. `fork_frame_12`)
            let fork_idx = if self.door_closing_sign || self.closing {
                12
            } else {
                let total_groups = self
                    .level_data
                    .as_ref()
                    .map(|l| l.groups.len())
                    .unwrap_or(8)
                    .max(1);
                let spawned = total_groups.saturating_sub(self.group_spawn_queue.len());
                (((spawned * 11) / total_groups) + 1).clamp(1, 12) as u32
            };
            draw_member(
                renderer,
                &format!("gfx.room/fork_frame_{}", fork_idx),
                Vec2::new(771.0, 23.0),
                false,
                201,
            );
        }

        // `game_manager.ls` lines 363-382: `closing_time` banner only displays for 1.0s when closing begins with dirty tables!
        if self.closing_timer.unwrap_or(0.0) > 0.0 {
            draw_member(
                renderer,
                "gfx.screens/closing_time",
                Vec2::new(408.0, 253.0),
                false,
                190,
            );
        }

        // HUD text overlays matching original game_interface_layout.txt
        renderer.draw_text(
            Vec2::new(112.0, 38.0),
            &format!("GOAL: {}", self.minimum_tip),
            None,
            15.0,
            Vec4::new(0.45, 0.90, 1.0, 1.0),
            205,
        );
        let score_str = format!("{}", self.score);
        let mut sx = 172.0;
        for ch in score_str.chars() {
            if ch.is_ascii_digit() {
                draw_member(renderer, &format!("gfx.numbers/tip.{}", ch), Vec2::new(sx, 58.0), false, 205);
                sx += 10.0;
            }
        }
        if self.mode == GameMode::Endless {
            draw_member(
                renderer,
                &format!("gfx.numbers/big_tablenum_{}", self.level.clamp(1, 16)),
                Vec2::new(575.0, 68.0),
                false,
                205,
            );
        } else {
            draw_member(
                renderer,
                &format!("gfx.numbers/big_tablenum_{}", self.level.clamp(1, 16)),
                Vec2::new(593.0, 52.0),
                false,
                202,
            );
        }

        // 7. Exact 24-Step Stage 1 Level 1 Tutorial (`tutorial_layout.txt`, `tutorial_1..24.ls`, `Diner Dash.exe` XMED chunks)
        if self.mode == GameMode::Career && self.stage == 1 && self.level == 1 && (1..=24).contains(&self.tutorial_step) {
            let bounce = ((self.current_ms as f32 / 180.0).sin()) * 4.0;
            let active_tid = self
                .room
                .tables
                .iter()
                .find_map(|(&tid, t)| (t.group_id.is_some() || t.state == TableState::Dirty).then_some(tid))
                .unwrap_or(TableId(2));
            let active_arrow_x = if active_tid == TableId(1) { 334.0 } else { 594.0 };

            let (box_pos, lines, arrows): (Vec2, &[&str], Vec<(&str, Vec2)>) = match self.tutorial_step {
                1 => (
                    Vec2::new(296.0, 211.0),
                    &[
                        "DRAG A GROUP TO AN",
                        "EMPTY, CLEAN TABLE.",
                    ],
                    vec![
                        ("gfx.screens/tutorial_arrow", Vec2::new(44.0, 361.0 + bounce)),
                        ("gfx.screens/tutorial_arrow", Vec2::new(595.0, 217.0 + bounce)),
                    ],
                ),
                2 => (
                    Vec2::new(375.0, 197.0),
                    &[
                        "MOVE THEM AROUND THE",
                        "CHAIRS AND RELEASE",
                        "TO SEAT THEM.",
                    ],
                    vec![("gfx.screens/tutorial_arrow", Vec2::new(594.0, 219.0 + bounce))],
                ),
                3 => (
                    Vec2::new(359.0, 341.0),
                    &[
                        "THE CUSTOMERS ARE",
                        "LOOKING AT THE MENU.",
                        "LET'S WAIT UNTIL THEY'RE",
                        "READY TO ORDER!",
                    ],
                    vec![],
                ),
                4 => (
                    Vec2::new(497.0, 493.0),
                    &[
                        "NOW THEY'RE READY!",
                        "CLICK ON THE TABLE TO",
                        "PICK UP THEIR ORDER.",
                    ],
                    vec![("gfx.screens/tutorial_arrow", Vec2::new(active_arrow_x, 214.0 + bounce))],
                ),
                5 => (
                    Vec2::new(455.0, 475.0),
                    &[
                        "GOOD! YOU GOT THEIR",
                        "ORDER TICKET! NOW CLICK",
                        "HERE TO DROP IT OFF.",
                    ],
                    vec![("gfx.screens/tutorial_arrow", Vec2::new(297.0, 108.0 + bounce))],
                ),
                6 => (
                    Vec2::new(455.0, 475.0),
                    &[
                        "GOOD. THE CHEF IS NOW",
                        "PREPARING THE FOOD...",
                    ],
                    vec![],
                ),
                7 => (
                    Vec2::new(650.0, 151.0),
                    &[
                        "...AND IT'S READY.",
                        "CLICK ON THE TRAY",
                        "TO PICK IT UP.",
                    ],
                    vec![("gfx.screens/tutorial_arrow_left", Vec2::new(416.0 + bounce, 148.0))],
                ),
                8 => (
                    Vec2::new(455.0, 475.0),
                    &[
                        "SEE HOW THE TABLE GETS",
                        "HIGHLIGHTED! CLICK ON THE",
                        "TABLE TO SERVE THE FOOD.",
                    ],
                    vec![("gfx.screens/tutorial_arrow", Vec2::new(active_arrow_x, 214.0 + bounce))],
                ),
                9 => (
                    Vec2::new(451.0, 473.0),
                    &[
                        "GOOD!",
                        "NOW LET'S GIVE THEM",
                        "SOME TIME TO EAT.",
                    ],
                    vec![],
                ),
                10 => (
                    Vec2::new(459.0, 488.0),
                    &[
                        "NOW THAT THEY'RE DONE",
                        "EATING, CLICK ON THE TABLE",
                        "TO GIVE THEM THEIR",
                        "CHECK.",
                    ],
                    vec![("gfx.screens/tutorial_arrow", Vec2::new(active_arrow_x, 214.0 + bounce))],
                ),
                11 => (
                    Vec2::new(457.0, 485.0),
                    &[
                        "GREAT! LOOK AT THAT",
                        "JUICY TIP THEY LEFT!",
                    ],
                    vec![],
                ),
                12 => (
                    Vec2::new(312.0, 362.0),
                    &[
                        "NOW, YOU'VE GOT TO",
                        "CLEAN UP THE TABLE.",
                        "CLICK TO PICK UP",
                        "DIRTY DISHES.",
                    ],
                    vec![("gfx.screens/tutorial_arrow", Vec2::new(active_arrow_x, 214.0 + bounce))],
                ),
                13 => (
                    Vec2::new(425.0, 129.0),
                    &[
                        "YOU CAN DROP THE DISHES",
                        "OFF AT THE BUS STATION.",
                    ],
                    vec![("gfx.screens/tutorial_arrow_right", Vec2::new(604.0 + bounce, 189.0))],
                ),
                14 => (
                    Vec2::new(153.0, 166.0),
                    &[
                        "TWO MORE GROUPS OF",
                        "CUSTOMERS ARRIVED.",
                        "LET'S SEAT BOTH OF THEM.",
                    ],
                    vec![
                        ("gfx.screens/tutorial_arrow", Vec2::new(42.0, 293.0 + bounce)),
                        ("gfx.screens/tutorial_arrow", Vec2::new(334.0, 217.0 + bounce)),
                        ("gfx.screens/tutorial_arrow", Vec2::new(595.0, 217.0 + bounce)),
                    ],
                ),
                15 => (
                    Vec2::new(455.0, 475.0),
                    &[
                        "NOTICE HOW THEIR",
                        "CLOTHES MATCH THE",
                        "CHAIRS? THIS EARNS YOU",
                        "BONUS POINTS!",
                    ],
                    vec![],
                ),
                16 => (
                    Vec2::new(456.0, 479.0),
                    &[
                        "YOU CAN ALSO STRING",
                        "ACTIONS TOGETHER. CLICK",
                        "ON BOTH TABLES AND THEN",
                        "THE TICKET STATION.",
                    ],
                    vec![
                        ("gfx.screens/tutorial_arrow", Vec2::new(334.0, 214.0 + bounce)),
                        ("gfx.screens/tutorial_arrow", Vec2::new(594.0, 214.0 + bounce)),
                        ("gfx.screens/tutorial_arrow", Vec2::new(297.0, 108.0 + bounce)),
                    ],
                ),
                17 => (
                    Vec2::new(455.0, 475.0),
                    &[
                        "NOTICE YOU GET EXTRA",
                        "POINTS WHEN YOU REPEAT",
                        "SIMILAR ACTIONS IN A",
                        "ROW?",
                    ],
                    vec![],
                ),
                18 => (
                    Vec2::new(455.0, 475.0),
                    &[
                        "FLO CAN ALSO CARRY TWO",
                        "THINGS AT ONCE.",
                    ],
                    vec![],
                ),
                19 => (
                    Vec2::new(455.0, 475.0),
                    &[
                        "TRY PICKING UP BOTH",
                        "TRAYS AND DELIVERING TO",
                        "BOTH TABLES!",
                    ],
                    vec![
                        ("gfx.screens/tutorial_arrow", Vec2::new(362.0, 108.0 + bounce)),
                        ("gfx.screens/tutorial_arrow", Vec2::new(435.0, 108.0 + bounce)),
                    ],
                ),
                20 => (
                    Vec2::new(463.0, 476.0),
                    &[
                        "THESE HEARTS SHOW THE",
                        "MOOD OF THE CUSTOMERS.",
                        "MAKE SURE THEY DON'T",
                        "GET TOO LOW!",
                    ],
                    vec![
                        ("gfx.screens/tutorial_arrow_up", Vec2::new(344.0, 355.0 + bounce)),
                        ("gfx.screens/tutorial_arrow_up", Vec2::new(605.0, 354.0 + bounce)),
                    ],
                ),
                21 => (
                    Vec2::new(357.0, 169.0),
                    &[
                        "IF A GROUP LEAVES",
                        "ANGRY, YOU LOSE A STAR!",
                        "LOSE ALL YOUR STARS",
                        "AND THE GAME'S OVER.",
                    ],
                    vec![("gfx.screens/tutorial_arrow_up", Vec2::new(358.0, 88.0 + bounce))],
                ),
                22 => (
                    Vec2::new(186.0, 169.0),
                    &[
                        "YOU NEED TO EARN THIS",
                        "MUCH TO PASS THIS",
                        "LEVEL.",
                    ],
                    vec![("gfx.screens/tutorial_arrow_up", Vec2::new(150.0, 88.0 + bounce))],
                ),
                23 => (
                    Vec2::new(638.0, 182.0),
                    &[
                        "KEEP AN EYE ON THE",
                        "CLOCK! NO MORE",
                        "CUSTOMERS CAN ENTER",
                        "AFTER CLOSING TIME.",
                    ],
                    vec![("gfx.screens/tutorial_arrow_up", Vec2::new(724.0, 107.0 + bounce))],
                ),
                _ => (
                    Vec2::new(455.0, 475.0),
                    &[
                        "OKAY, NOW FINISH SERVING",
                        "THESE CUSTOMERS TO",
                        "COMPLETE THE LEVEL!",
                    ],
                    vec![],
                ),
            };

            draw_member(renderer, "gfx.screens/tutorial_box", box_pos, false, 210);
            let line_step = 22.0;
            let total_h = (lines.len().saturating_sub(1) as f32) * line_step;
            let start_y = box_pos.y - total_h * 0.5 + 6.0;
            for (i, line) in lines.iter().enumerate() {
                renderer.draw_text(
                    Vec2::new(box_pos.x, start_y + (i as f32) * line_step),
                    line,
                    Some(crate::assets::FontId(1)),
                    16.0,
                    Vec4::new(0.08, 0.08, 0.08, 1.0),
                    211,
                );
            }
            for (arrow_member, arrow_pos) in arrows {
                draw_member(renderer, arrow_member, arrow_pos, false, 212);
            }
        }

        renderer.end_frame();
    }
    
    /// Schedule difficulty ramp (endless mode)
    pub fn schedule_ramp(&mut self) -> Result<()> {
        if self.mode != GameMode::Endless { return Ok(()); }
        Ok(())
    }
    
    /// Increment level/difficulty
    pub fn increment_level(&mut self) -> bool {
        if self.mode == GameMode::Career {
            if let Some(stage) = &self.stage_data {
                if self.level < stage.num_levels {
                    self.level += 1;
                    return false;
                } else if self.stage < 5 {
                    self.stage += 1;
                    self.level = 1;
                    return false;
                } else {
                    return true;
                }
            }
        } else {
            self.time_multiplier.current = clamp(
                self.time_multiplier.current - self.time_multiplier.decrease,
                self.time_multiplier.minimum,
                self.time_multiplier.starting,
            );
            self.level += 1;
            self.accumulated_upgrades += 1;
            self.show_upgrade_button = self.accumulated_upgrades > 0;
        }
        false
    }
    
    /// Add score from tip
    pub fn score_tip(&mut self, tip: u32) {
        self.score = self.score.saturating_add(tip);
        if self.mode == GameMode::Endless {
            self.total_score = self.score;
        }
        self.update_money_jar();
        self.check_tip_goals();
    }
    
    /// Update money jar visual
    fn update_money_jar(&mut self) {
        if self.score == 0 {
            self.money_jar_level = 0;
            return;
        }
        
        if self.score < self.minimum_tip {
            let inc = ((self.minimum_tip as f32 * 0.2) as u32).max(1);
            let level = ((self.score as f32 / inc as f32) + 1.0).clamp(1.0, 4.0) as u32;
            self.money_jar_level = level;
        } else {
            let bonus = self.bonus_tip;
            let inc = ((bonus as f32 * 0.25) as u32).max(1);
            let level = ((self.score as f32 / inc as f32) + 4.0).clamp(5.0, 9.0) as u32;
            self.money_jar_level = level;
        }
    }
    
    /// Check if tip goals reached
    fn check_tip_goals(&mut self) {
        if self.mode == GameMode::Endless && self.score >= self.minimum_tip {
            self.end_sub_level();
        }
    }
    
    /// End sub-level (endless mode)
    pub fn end_sub_level(&mut self) {
        self.accumulated_upgrades += 1;
        if self.accumulated_upgrades == 1 {
            self.show_upgrade_button = true;
        }
        self.increment_level();
        self.minimum_tip = self.sublevel_data.as_ref().map(|d| d.minimum_tip).unwrap_or(self.minimum_tip + 1000);
    }
    
    /// Start closing sequence (`game_manager.ls` lines 363-382)
    pub fn start_closing(&mut self) -> Result<()> {
        if self.closing || self.mode == GameMode::Endless { return Ok(()); }
        
        self.closing = true;
        self.door_closing_sign = true;
        self.closing_timer = if self.has_dirty_tables() || self.waiter.carried_dishes > 0 { 
            Some((self.config.closing_time_ms as f32 / 1000.0).clamp(0.8, 1.5))
        } else { 
            None
        };
        Ok(())
    }
    
    fn has_dirty_tables(&self) -> bool {
        self.room.tables.values().any(|t| t.state == TableState::Dirty)
    }
    
    /// End level
    pub fn end_level(&mut self) -> LevelResult {
        let success = self.score >= self.minimum_tip;
        
        let mut result = LevelResult {
            success,
            score: self.score,
            required: self.minimum_tip,
            expert: self.score >= self.bonus_tip,
            stage: self.stage,
            level: self.level,
            customers_served: self.customers_served,
            customers_left: self.customers_left,
            text: if success {
                self.level_data
                    .as_ref()
                    .map(|l| l.text.succeed.clone())
                    .unwrap_or_else(|| "Great job!".to_string())
            } else {
                self.level_data
                    .as_ref()
                    .map(|l| l.text.fail.clone())
                    .unwrap_or_else(|| "Try again!".to_string())
            },
            end_game: false,
            hiscore: None,
            doodle: self.level_data.as_ref().and_then(|l| l.doodle.clone()),
        };
        
        if success {
            if self.mode == GameMode::Career {
                self.total_score += self.score;
                if self.increment_level() {
                    result.end_game = true;
                }
            }
        } else if self.mode == GameMode::Career {
            if self.stars > 0 {
                self.stars -= 1;
            }
            if self.stars == 0 {
                result.end_game = true;
            }
        }
        
        result
    }
    
    /// Win level (cheat or perfect)
    pub fn win_level(&mut self) {
        self.score = self.bonus_tip.max(self.minimum_tip);
        if self.mode == GameMode::Endless {
            self.end_sub_level();
        }
    }
    
    /// Lose level
    pub fn lose_level(&mut self) {
        self.score = 0;
    }
    
    /// Customer left angry
    pub fn angry_customer(&mut self, penalty: u32) {
        if self.mode == GameMode::Endless && self.stars > 0 {
            self.stars -= 1;
        }
        self.score = self.score.saturating_sub(penalty);
        self.update_money_jar();
    }
    
    /// Add star
    pub fn add_star(&mut self) {
        self.stars = (self.stars + 1).min(self.max_stars);
    }
    
    /// Purchase upgrade
    pub fn purchase_upgrade(&mut self, upgrade_type: &str) -> Result<()> {
        if upgrade_type == "table" {
            if let Some(_upgrade) = self.upgrades.tables.first().cloned() {
                self.upgrades.tables.remove(0);
                self.room.add_table_at(GridPos::new(8, 8), 4);
            }
        } else if let Some(powerups) = self.upgrades.powerups.get_mut(upgrade_type) {
            if let Some(upgrade) = powerups.first().cloned() {
                powerups.remove(0);
                self.apply_powerup(upgrade_type, &upgrade);
            }
        }
        self.accumulated_upgrades = self.accumulated_upgrades.saturating_sub(1);
        self.show_upgrade_button = self.accumulated_upgrades > 0;
        Ok(())
    }
    
    fn apply_powerup(&mut self, type_: &str, upgrade: &PowerupLevel) {
        match type_ {
            "drinks" | "stereo" => {
                let mult = upgrade.properties.get("patienceMultiplier").copied().unwrap_or(1.0);
                for state in [SitterState::Ordered, SitterState::Check, SitterState::Snack, SitterState::Ready] {
                    self.state_time_multipliers.insert(state, mult);
                }
            }
            "podium" => {
                if let Some(podium) = &mut self.room.podium {
                    podium.set_properties(&upgrade.properties);
                }
            }
            "oven" => {
                if let Some(speed) = upgrade.properties.get("foodReadyMS") {
                    self.room.kitchen.set_speed(*speed as u32);
                }
            }
            "shoes" => {
                if let Some(speed) = upgrade.properties.get("speed") {
                    self.waiter.set_speed(*speed);
                }
            }
            _ => {}
        }
    }
    
    /// Spawn a random group for endless mode
    pub fn spawn_random_group(&mut self) -> Result<()> {
        let group_id = GroupId(rand::range_int(1, i32::MAX) as u64);
        self.new_groups.push_back(group_id);
        let party_size = rand::range_int(1, 4) as u32;
        self.group_spawn_queue.push_back(GroupConfig {
            group_type: "default".to_string(),
            customers: party_size,
            queue_time_ms: 6000,
            frequency: 1,
            colors: Vec::new(),
        });
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpriteMeta {
    pub path: String,
    pub width: u32,
    pub height: u32,
    pub reg_x: f32,
    pub reg_y: f32,
}

fn sprite_manifest() -> &'static HashMap<String, SpriteMeta> {
    static MANIFEST: std::sync::OnceLock<HashMap<String, SpriteMeta>> = std::sync::OnceLock::new();
    MANIFEST.get_or_init(|| {
        const RAW_MANIFEST: &str = include_str!("../../assets/data/sprite_manifest.json");
        serde_json::from_str(RAW_MANIFEST).unwrap_or_default()
    })
}

fn draw_member(
    renderer: &mut dyn Renderer,
    key: &str,
    pos: Vec2,
    flip_x: bool,
    z_order: i32,
) {
    if let Some(meta) = sprite_manifest().get(key) {
        renderer.draw_image(
            &meta.path,
            pos,
            Vec2::new(meta.reg_x, meta.reg_y),
            flip_x,
            z_order,
        );
    }
}

fn customer_color_cast(color: CustomerColor, group_type: &str) -> &'static str {
    if group_type.starts_with("critic") {
        return "gfx.people.none";
    }
    match color {
        CustomerColor::Red => "gfx.people.red",
        CustomerColor::Blue => "gfx.people.blue",
        CustomerColor::Green => "gfx.people.green",
        CustomerColor::Yellow => "gfx.people.orange",
        CustomerColor::Purple => "gfx.people.purple",
        CustomerColor::None => "gfx.people.none",
    }
}

fn customer_sprite_names(group_type: &str) -> (&'static str, &'static str, &'static str) {
    if group_type.starts_with("business") {
        ("standing_business_female", "business_female_legs", "bizwoman")
    } else if group_type.starts_with("old_male") {
        ("standing_old_male", "old_male_legs", "oldman")
    } else if group_type.starts_with("boy") {
        ("standing_boy", "boy_legs", "boy")
    } else if group_type.starts_with("man") {
        ("standing_man", "man_legs", "man")
    } else if group_type.starts_with("critic") {
        ("standing_critic", "critic_legs", "critic")
    } else {
        ("standing_young_female", "young_female_legs", "girl")
    }
}

/// Level result data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelResult {
    pub success: bool,
    pub score: u32,
    pub required: u32,
    pub expert: bool,
    pub stage: u32,
    pub level: u32,
    pub customers_served: u32,
    pub customers_left: u32,
    pub text: String,
    pub end_game: bool,
    pub hiscore: Option<u32>,
    pub doodle: Option<String>,
}

/// Timer system for scheduling delayed actions
pub struct TimerSystem {
    timers: HashMap<TimerId, Timer>,
    next_id: u64,
}

pub struct Timer {
    pub id: TimerId,
    pub delay: f32,
    pub elapsed: f32,
    pub callback: Option<Box<dyn FnOnce(&mut World) + Send + Sync>>,
    pub repeating: bool,
}

impl TimerSystem {
    pub fn new() -> Self {
        Self {
            timers: HashMap::new(),
            next_id: 1,
        }
    }
    
    pub fn schedule<F>(&mut self, delay: f32, callback: F) -> TimerId 
    where 
        F: FnOnce(&mut World) + Send + Sync + 'static 
    {
        let id = TimerId(self.next_id);
        self.next_id += 1;
        
        self.timers.insert(id, Timer {
            id,
            delay,
            elapsed: 0.0,
            callback: Some(Box::new(callback)),
            repeating: false,
        });
        
        id
    }
    
    pub fn schedule_repeating<F>(&mut self, interval: f32, callback: F) -> TimerId
    where 
        F: Fn(&mut World) + Send + Sync + 'static 
    {
        let id = TimerId(self.next_id);
        self.next_id += 1;
        
        use std::sync::Arc;
        let callback = Arc::new(callback);
        let callback_clone = Arc::clone(&callback);
        
        self.timers.insert(id, Timer {
            id,
            delay: interval,
            elapsed: 0.0,
            callback: Some(Box::new(move |world| {
                callback_clone(world);
            })),
            repeating: true,
        });
        
        id
    }
    
    pub fn cancel(&mut self, id: TimerId) {
        self.timers.remove(&id);
    }
    
    pub fn update(&mut self, dt: f32, world: &mut World) {
        let mut to_fire = Vec::new();
        
        for (id, timer) in self.timers.iter_mut() {
            timer.elapsed += dt;
            if timer.elapsed >= timer.delay {
                to_fire.push(*id);
                if timer.repeating {
                    timer.elapsed = 0.0;
                }
            }
        }
        
        for id in to_fire {
            if let Some(mut timer) = self.timers.remove(&id) {
                if let Some(callback) = timer.callback.take() {
                    (callback)(world);
                }
                if timer.repeating {
                    timer.elapsed = 0.0;
                    self.timers.insert(id, timer);
                }
            }
        }
    }
}

impl System for TimerSystem {
    fn name(&self) -> &str { "TimerSystem" }
    fn priority(&self) -> i32 { 200 }
    
    fn run(&mut self, world: &mut World, dt: f32) {
        self.update(dt, world);
    }
}

impl Default for TimerSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::CommandBufferRenderer;
    
    #[test]
    fn test_game_manager_creation() {
        let config = GameConfig::default();
        assert_eq!(config.career.stages.len(), 5);
        assert_eq!(config.career.stages[0].levels.len(), 10);

        let mut manager = GameManager::new(config);
        assert_eq!(manager.mode, GameMode::Career);
        assert_eq!(manager.stage, 1);
        assert_eq!(manager.stars, 3);

        manager.init(GameMode::Career, 1, 1, 0).unwrap();
        manager.start_level().unwrap();
        assert!(!manager.room.tables.is_empty());
        assert!(!manager.sitters.waiting_groups.is_empty());

        // Plain click on clean table must NOT seat customers
        let first_table = *manager.room.tables.keys().next().unwrap();
        assert!(!manager.click_table(first_table));

        // Drag and drop first waiting group onto first table
        let tc = manager.table_visual_center(manager.room.tables.get(&first_table).unwrap());
        assert!(manager.pointer_down(Vec2::new(40.0, 450.0)));
        manager.pointer_move(tc);
        assert!(manager.pointer_up(tc));

        // Step simulation & render
        let _ = manager.update(0.5);
        let mut renderer = CommandBufferRenderer::new();
        manager.render(&mut renderer);
        assert!(!renderer.commands().is_empty());
    }
    
    #[test]
    fn test_sitter_states() {
        let state = SitterState::default();
        assert_eq!(state, SitterState::New);
    }

    #[test]
    fn test_decompiled_mechanics_and_24_step_tutorial() {
        let config = GameConfig::default();
        let mut manager = GameManager::new(config);
        manager.init(GameMode::Career, 1, 1, 0).unwrap();
        manager.start_level().unwrap();

        // 1. Verify Stage 1 station coordinates (`bg_diner_layout.txt` + `room_layout.txt`)
        let (ticket_pos, food_slots, bus_pos, podium_pos) = manager.stage_station_positions();
        assert_eq!(ticket_pos, Vec2::new(290.0, 151.0));
        assert_eq!(food_slots[0], Vec2::new(362.0, 154.0));
        assert_eq!(bus_pos, Vec2::new(663.0, 198.0));
        assert_eq!(podium_pos, Vec2::new(83.0, 515.0));

        // 2. Verify Stage 1 Level 1 table centers & service paths (`2top_layout.txt` & `room_manager.ls`)
        let t1 = manager.room.tables.get(&TableId(1)).unwrap();
        let t2 = manager.room.tables.get(&TableId(2)).unwrap();
        let tc1 = manager.table_visual_center(t1);
        let tc2 = manager.table_visual_center(t2);
        assert_eq!(tc1, Vec2::new(334.0, 281.0));
        assert_eq!(tc2, Vec2::new(593.0, 281.0));
        let path2 = manager.room.table_service_path(manager.waiter.position, TableId(2)).unwrap();
        assert_eq!(*path2.last().unwrap(), Vec2::new(592.0, 248.0));

        // 3. Step 1 -> Step 2 (pick up group) -> Step 3 (drop on Table 2)
        assert_eq!(manager.tutorial_step, 1);
        assert!(manager.pointer_down(Vec2::new(40.0, 450.0)));
        let _ = manager.update(0.05);
        assert_eq!(manager.tutorial_step, 2);
        manager.pointer_move(tc2);
        assert!(manager.pointer_up(tc2));
        let _ = manager.update(0.05);
        assert_eq!(manager.tutorial_step, 3);

        // 4. Wait until customers finish deciding (`Ready` -> Step 4)
        for _ in 0..50 {
            let _ = manager.update(0.1);
        }
        assert_eq!(manager.tutorial_step, 4);

        // 5. Click Table 2 to take order -> Step 5 -> Click Ticket Station -> Step 6
        assert!(manager.handle_click(tc2));
        for _ in 0..40 {
            let _ = manager.update(0.1);
        }
        assert_eq!(manager.tutorial_step, 5);
        assert!(manager.handle_click(ticket_pos));
        for _ in 0..40 {
            let _ = manager.update(0.1);
        }
        assert_eq!(manager.tutorial_step, 6);

        // Verify chef is cooking and `stove_chefanim` is NEVER drawn
        let mut renderer = CommandBufferRenderer::new();
        manager.render(&mut renderer);
        let drawn_images: Vec<String> = renderer
            .commands()
            .iter()
            .filter_map(|c| match c {
                crate::renderer::DrawCommand::Image { asset_path, .. } => Some(asset_path.clone()),
                _ => None,
            })
            .collect();
        assert!(!drawn_images.iter().any(|p| p.contains("stove_chefanim")));

        // 6. Wait for food to finish cooking -> Step 7 -> Click food tray -> Step 8 -> Click Table 2 -> Step 9 (Eating)
        for _ in 0..60 {
            let _ = manager.update(0.1);
        }
        assert_eq!(manager.tutorial_step, 7);
        assert!(manager.handle_click(food_slots[0]));
        for _ in 0..40 {
            let _ = manager.update(0.1);
        }
        assert_eq!(manager.tutorial_step, 8);
        assert!(manager.handle_click(tc2));
        for _ in 0..40 {
            let _ = manager.update(0.1);
        }
        assert_eq!(manager.tutorial_step, 9);

        // Verify Eating state does NOT draw corrupted `dishes_292.png` underneath `food_01_4..1`
        let mut renderer = CommandBufferRenderer::new();
        manager.render(&mut renderer);
        let drawn_images: Vec<String> = renderer
            .commands()
            .iter()
            .filter_map(|c| match c {
                crate::renderer::DrawCommand::Image { asset_path, .. } => Some(asset_path.clone()),
                _ => None,
            })
            .collect();
        assert!(!drawn_images.iter().any(|p| p.contains("dishes_292")));
        assert!(drawn_images.iter().any(|p| p.contains("food_01_")));

        // 7. Wait for Eating to finish -> Check state (Step 10)
        for _ in 0..70 {
            let _ = manager.update(0.1);
        }
        assert_eq!(manager.tutorial_step, 10);

        // Verify Check state has NO `thought_bubble1` or `ready_check`
        let mut renderer = CommandBufferRenderer::new();
        manager.render(&mut renderer);
        let drawn_images: Vec<String> = renderer
            .commands()
            .iter()
            .filter_map(|c| match c {
                crate::renderer::DrawCommand::Image { asset_path, .. } => Some(asset_path.clone()),
                _ => None,
            })
            .collect();
        assert!(!drawn_images.iter().any(|p| p.contains("thought_bubble")));
        assert!(!drawn_images.iter().any(|p| p.contains("ready_check")));

        // 8. Click Table 2 to deliver check -> Scoring (Step 11) -> Left + Dirty table + transient coin_flip (Step 12)
        assert!(manager.handle_click(tc2));
        for _ in 0..50 {
            let _ = manager.update(0.1);
        }
        assert_eq!(manager.tutorial_step, 12);
        assert_eq!(manager.room.tables.get(&TableId(2)).unwrap().state, TableState::Dirty);

        // 9. Clean Table 2 -> Step 13 -> Drop dishes at bus station -> Step 14 (2 wave-2 groups spawn!)
        assert!(manager.handle_click(tc2));
        for _ in 0..40 {
            let _ = manager.update(0.1);
        }
        assert_eq!(manager.tutorial_step, 13);
        assert!(manager.handle_click(bus_pos));
        for _ in 0..40 {
            let _ = manager.update(0.1);
        }
        assert_eq!(manager.tutorial_step, 14);
        assert_eq!(manager.sitters.waiting_groups.len(), 2);
    }
}
