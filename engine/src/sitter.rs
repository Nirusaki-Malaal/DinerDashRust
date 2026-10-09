//! Sitter (Customer) System
//! 
//! Implements the 11-state customer state machine from the original game

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use crate::math::{Vec2, clamp, easing, rand};
use crate::ecs::{Entity, Component, World, System};
use crate::game::{CustomerColor, SitterState, SitterId, GroupId, GroupState, TimeMultiplier};
use crate::room::{TableId, Seat};

/// Sitter (individual customer) component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sitter {
    pub id: SitterId,
    pub group_id: GroupId,
    pub color: CustomerColor,
    pub state: SitterState,
    pub state_timer: f32,
    pub target_state: Option<SitterState>,
    
    // Position & movement
    pub position: Vec2,
    pub target_position: Vec2,
    pub move_speed: f32,
    
    // Table & seat assignment
    pub table_id: Option<TableId>,
    pub seat_index: Option<usize>,
    
    // Order & patience
    pub order: Option<String>,
    pub food_type: Option<String>,
    pub patience: f32,
    pub max_patience: f32,
    pub mood: f32, // 0.0 - 1.0
    pub tip_multiplier: f32,
    
    // Visual
    pub facing: Vec2,
    pub animation: String,
    
    // State data (durations per state)
    pub state_durations: HashMap<SitterState, f32>,
}

impl Sitter {
    pub fn new(id: SitterId, group_id: GroupId, color: CustomerColor, config: &SitterConfig) -> Self {
        let mut durations = HashMap::new();
        
        // Default state durations (in seconds)
        durations.insert(SitterState::New, 0.5);
        durations.insert(SitterState::Waiting, config.wait_patience);
        durations.insert(SitterState::Deciding, config.decide_time);
        durations.insert(SitterState::Ready, 1.0);
        durations.insert(SitterState::Ordered, config.order_time);
        durations.insert(SitterState::Eating, config.eat_time);
        durations.insert(SitterState::Snack, config.snack_time);
        durations.insert(SitterState::Check, config.check_time);
        durations.insert(SitterState::Scoring, 1.0);
        durations.insert(SitterState::Leaving, 0.05);
        durations.insert(SitterState::Ditching, 0.5);
        
        // Color affects patience
        let patience_mult = match color {
            CustomerColor::Green => 1.0,
            CustomerColor::Red => 0.8,
            CustomerColor::Blue => 1.2,
            CustomerColor::Yellow => 0.9,
            CustomerColor::Purple => 1.1,
            CustomerColor::None => 0.5, // Critic - very impatient
        };
        
        Self {
            id,
            group_id,
            color,
            state: SitterState::New,
            state_timer: 0.0,
            target_state: None,
            position: Vec2::ZERO,
            target_position: Vec2::ZERO,
            move_speed: 100.0, // pixels per second
            table_id: None,
            seat_index: None,
            order: None,
            food_type: None,
            patience: config.base_patience * patience_mult,
            max_patience: config.base_patience * patience_mult,
            mood: 1.0,
            tip_multiplier: 1.0,
            facing: Vec2::new(1.0, 0.0),
            animation: "idle".to_string(),
            state_durations: durations,
        }
    }
    
    /// Update sitter state machine
    pub fn update(&mut self, dt: f32, time_mult: f32, room: &crate::room::RoomManager) -> SitterEvent {
        let scaled_dt = dt * time_mult;
        self.state_timer += scaled_dt;
        
        // Decrease patience only while waiting for Flo (never while reading menu or eating)
        if matches!(
            self.state,
            SitterState::Waiting | SitterState::Ready | SitterState::Ordered | SitterState::Check
        ) {
            let drain_rate = (1.5 - 0.5 * self.mood).max(0.5);
            self.patience -= scaled_dt * drain_rate;
            if self.patience <= 0.0 {
                return SitterEvent::Ditch;
            }
        }
        
        // Movement
        if self.position != self.target_position {
            let dir = (self.target_position - self.position).normalize_or_zero();
            self.position += dir * self.move_speed * scaled_dt;
            
            // Check if arrived
            if self.position.distance(self.target_position) < 5.0 {
                self.position = self.target_position;
                self.facing = dir;
            }
        }
        
        // State machine
        match self.state {
            SitterState::New => {
                if self.state_timer >= self.get_state_duration(SitterState::New) {
                    self.transition(SitterState::Waiting);
                }
            }
            SitterState::Waiting => {
                // Seating, not a timer, starts menu reading.
            }
            SitterState::Deciding => {
                if self.state_timer >= self.get_state_duration(SitterState::Deciding) {
                    self.transition(SitterState::Ready);
                }
            }
            SitterState::Ready => {
                if self.state_timer >= self.get_state_duration(SitterState::Ready) {
                    // Wait for waiter to take order
                }
            }
            SitterState::Ordered => {
                // Only food delivery can begin eating.
            }
            SitterState::Eating => {
                if self.state_timer >= self.get_state_duration(SitterState::Eating) {
                    // Check for snack chance
                    if rand::bool(0.1) && room.has_snacks() {
                        self.transition(SitterState::Snack);
                    } else {
                        self.transition(SitterState::Check);
                    }
                }
            }
            SitterState::Snack => {
                if self.state_timer >= self.get_state_duration(SitterState::Snack) {
                    self.transition(SitterState::Check);
                }
            }
            SitterState::Check => {
                // Only Flo's check action starts payment.
            }
            SitterState::Scoring => {
                if self.state_timer >= self.get_state_duration(SitterState::Scoring) {
                    self.transition(SitterState::Leaving);
                    return SitterEvent::Left;
                }
            }
            SitterState::Leaving => {
                if self.state_timer >= self.get_state_duration(SitterState::Leaving) {
                    return SitterEvent::Left;
                }
            }
            SitterState::Ditching => {
                if self.state_timer >= self.get_state_duration(SitterState::Ditching) {
                    return SitterEvent::Ditched;
                }
            }
        }
        
        SitterEvent::None
    }
    
    fn get_state_duration(&self, state: SitterState) -> f32 {
        self.state_durations.get(&state).copied().unwrap_or(1.0)
    }
    
    fn transition(&mut self, new_state: SitterState) {
        self.state = new_state;
        self.state_timer = 0.0;
        self.patience = self.max_patience;
        self.update_animation();
    }
    
    fn update_animation(&mut self) {
        self.animation = match self.state {
            SitterState::New => "spawn".to_string(),
            SitterState::Waiting => "wait".to_string(),
            SitterState::Deciding => "think".to_string(),
            SitterState::Ready => "ready".to_string(),
            SitterState::Ordered => "wait".to_string(),
            SitterState::Eating => "eat".to_string(),
            SitterState::Snack => "snack".to_string(),
            SitterState::Check => "check".to_string(),
            SitterState::Scoring => "happy".to_string(),
            SitterState::Leaving => "leave".to_string(),
            SitterState::Ditching => "angry".to_string(),
        };
    }
    
    /// Called when waiter takes order
    pub fn take_order(&mut self, food: String) {
        self.order = Some(food.clone());
        self.food_type = Some(food);
        self.transition(SitterState::Ordered);
    }
    
    /// Called when food is delivered
    pub fn serve_food(&mut self) {
        if self.state == SitterState::Ordered {
            self.transition(SitterState::Eating);
        }
    }
    
    /// Called when check is delivered
    pub fn deliver_check(&mut self) {
        if self.state == SitterState::Eating || self.state == SitterState::Snack {
            self.transition(SitterState::Check);
        }
    }
    
    /// Customer pays and leaves
    pub fn pay(&mut self, tip: f32) -> u32 {
        self.transition(SitterState::Scoring);
        (tip * self.tip_multiplier) as u32
    }
    
    /// Customer gets angry and leaves
    pub fn ditch(&mut self) {
        self.transition(SitterState::Ditching);
    }
    
    /// Set target position for movement
    pub fn move_to(&mut self, target: Vec2) {
        self.target_position = target;
    }
    
    /// Assign to table seat
    pub fn assign_seat(&mut self, table_id: TableId, seat_index: usize) {
        self.table_id = Some(table_id);
        self.seat_index = Some(seat_index);
    }
    
    /// Apply mood change (from podium, etc.)
    pub fn change_mood(&mut self, amount: f32) {
        self.mood = clamp(self.mood + amount, 0.0, 1.0);
    }
    
    /// Apply time multiplier to state durations
    pub fn set_time_multiplier(&mut self, multiplier: f32) {
        for (_, duration) in &mut self.state_durations {
            *duration /= multiplier;
        }
    }
}

/// Events that can come from sitter updates
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SitterEvent {
    None,
    Ditch,
    Ditched,
    Left,
    OrderReady,
    FoodServed,
    CheckReady,
    Paid(u32),
}

/// Per-type customer configuration extracted from `customer_dump.txt`
#[derive(Debug, Clone, Copy)]
pub struct CustomerTypeTiming {
    pub time_mood: i32,
    pub color_mood: i32,
    pub base_tip: u32,
    pub mood_tip: u32,
    pub leave_penalty: u32,
    pub wait_happy_s: f32,
    pub wait_mad_s: f32,
    pub decide_s: f32,
    pub ready_happy_s: f32,
    pub ready_mad_s: f32,
    pub order_happy_s: f32,
    pub order_mad_s: f32,
    pub eat_s: f32,
    pub check_happy_s: f32,
    pub check_mad_s: f32,
    pub snack_happy_s: f32,
    pub snack_mad_s: f32,
    pub leave_s: f32,
}

impl CustomerTypeTiming {
    pub fn for_group_type(group_type: &str) -> Self {
        let gt = group_type.to_ascii_lowercase();
        let leave_penalty = if gt == "critic" {
            4000
        } else if gt.ends_with('5') || gt.ends_with("_fast") {
            2000
        } else if gt.ends_with('4') || gt == "man" || gt == "endless_critic" {
            1000
        } else if gt.ends_with('3') || gt == "boy" {
            750
        } else if gt == "endless_boy" {
            200
        } else {
            500
        };

        if gt.contains("tutorial") {
            Self {
                time_mood: 3,
                color_mood: 1,
                base_tip: 10,
                mood_tip: 10,
                leave_penalty: 500,
                wait_happy_s: 4.0,
                wait_mad_s: 10.0,
                decide_s: 4.0,
                ready_happy_s: 4.0,
                ready_mad_s: 10.0,
                order_happy_s: 6.0,
                order_mad_s: 11.0,
                eat_s: 4.0,
                check_happy_s: 4.0,
                check_mad_s: 10.0,
                snack_happy_s: 8.0,
                snack_mad_s: 10.0,
                leave_s: 2.0,
            }
        } else if gt.contains("old_male") {
            let is_endless = gt.starts_with("endless");
            Self {
                time_mood: 3,
                color_mood: 1,
                base_tip: 5,
                mood_tip: 10,
                leave_penalty,
                wait_happy_s: 6.0,
                wait_mad_s: 15.0,
                decide_s: if is_endless { 16.5 } else { 21.0 },
                ready_happy_s: 6.0,
                ready_mad_s: 15.0,
                order_happy_s: 9.0,
                order_mad_s: 16.5,
                eat_s: if is_endless { 16.5 } else { 21.0 },
                check_happy_s: 6.0,
                check_mad_s: 15.0,
                snack_happy_s: 8.0,
                snack_mad_s: 10.0,
                leave_s: 3.0,
            }
        } else if gt.contains("business") {
            let is_endless = gt.starts_with("endless");
            let is_fast = gt.ends_with("_fast");
            Self {
                time_mood: 3,
                color_mood: 1,
                base_tip: if is_endless { 20 } else { 15 },
                mood_tip: if is_endless { 3 } else { 10 },
                leave_penalty,
                wait_happy_s: if is_endless { 2.8 } else if is_fast { 3.0 } else { 2.4 },
                wait_mad_s: if is_endless { 7.0 } else { 6.0 },
                decide_s: if is_fast { 1.0 } else if is_endless { 9.5 } else { 14.0 },
                ready_happy_s: if is_endless { 2.8 } else if is_fast { 3.0 } else { 2.4 },
                ready_mad_s: if is_endless { 7.0 } else { 6.0 },
                order_happy_s: if is_endless { 4.2 } else if is_fast { 4.0 } else { 3.6 },
                order_mad_s: if is_endless { 7.7 } else if is_fast { 6.5 } else { 6.6 },
                eat_s: if is_fast { 1.0 } else if is_endless { 9.5 } else { 14.0 },
                check_happy_s: if is_endless { 2.8 } else if is_fast { 3.0 } else { 2.4 },
                check_mad_s: if is_endless { 7.0 } else { 6.0 },
                snack_happy_s: 8.0,
                snack_mad_s: 10.0,
                leave_s: 2.0,
            }
        } else if gt.contains("boy") {
            let is_endless = gt.starts_with("endless");
            Self {
                time_mood: 3,
                color_mood: 1,
                base_tip: 10,
                mood_tip: if is_endless { 5 } else { 10 },
                leave_penalty,
                wait_happy_s: if is_endless { 6.0 } else { 4.0 },
                wait_mad_s: if is_endless { 15.0 } else { 10.0 },
                decide_s: if is_endless { 9.8 } else { 14.0 },
                ready_happy_s: if is_endless { 6.0 } else { 4.0 },
                ready_mad_s: if is_endless { 15.0 } else { 10.0 },
                order_happy_s: if is_endless { 9.0 } else { 6.0 },
                order_mad_s: if is_endless { 16.5 } else { 11.0 },
                eat_s: if is_endless { 9.8 } else { 14.0 },
                check_happy_s: if is_endless { 6.0 } else { 4.0 },
                check_mad_s: if is_endless { 15.0 } else { 10.0 },
                snack_happy_s: 8.0,
                snack_mad_s: 13.0,
                leave_s: if is_endless { 1.4 } else { 2.0 },
            }
        } else if gt.contains("man") {
            let is_endless = gt.starts_with("endless");
            let is_fast = gt.ends_with("_fast");
            Self {
                time_mood: 3,
                color_mood: 1,
                base_tip: if is_fast { 15 } else { 10 },
                mood_tip: 10,
                leave_penalty,
                wait_happy_s: 4.0,
                wait_mad_s: if is_endless { 10.0 } else { 10.0 },
                decide_s: if is_fast { 1.0 } else if is_endless { 10.0 } else { 14.0 },
                ready_happy_s: 4.0,
                ready_mad_s: if is_endless { 10.0 } else { 10.0 },
                order_happy_s: 6.0,
                order_mad_s: if is_endless { 11.0 } else { 11.0 },
                eat_s: if is_fast { 1.0 } else { 14.0 },
                check_happy_s: 4.0,
                check_mad_s: if is_endless { 10.0 } else { 10.0 },
                snack_happy_s: 6.0,
                snack_mad_s: 8.0,
                leave_s: 2.0,
            }
        } else if gt.contains("critic") {
            let is_endless = gt.starts_with("endless");
            Self {
                time_mood: 3,
                color_mood: 1,
                base_tip: 10,
                mood_tip: 10,
                leave_penalty,
                wait_happy_s: if is_endless { 2.4 } else { 4.0 },
                wait_mad_s: if is_endless { 6.0 } else { 10.0 },
                decide_s: if is_endless { 1.2 } else { 14.0 },
                ready_happy_s: if is_endless { 2.4 } else { 4.0 },
                ready_mad_s: if is_endless { 6.0 } else { 10.0 },
                order_happy_s: if is_endless { 3.6 } else { 6.0 },
                order_mad_s: if is_endless { 6.6 } else { 11.0 },
                eat_s: if is_endless { 12.0 } else { 14.0 },
                check_happy_s: if is_endless { 2.4 } else { 4.0 },
                check_mad_s: if is_endless { 6.0 } else { 10.0 },
                snack_happy_s: 8.0,
                snack_mad_s: 10.0,
                leave_s: 2.0,
            }
        } else {
            let is_endless = gt.starts_with("endless");
            let is_fast = gt.ends_with("_fast");
            Self {
                time_mood: 3,
                color_mood: 1,
                base_tip: if is_fast { 15 } else { 10 },
                mood_tip: 10,
                leave_penalty,
                wait_happy_s: 4.0,
                wait_mad_s: 10.0,
                decide_s: if is_fast { 1.0 } else if is_endless { 10.0 } else { 14.0 },
                ready_happy_s: 4.0,
                ready_mad_s: 10.0,
                order_happy_s: 6.0,
                order_mad_s: 11.0,
                eat_s: if is_fast { 1.0 } else if is_endless { 10.0 } else { 14.0 },
                check_happy_s: 4.0,
                check_mad_s: 10.0,
                snack_happy_s: 8.0,
                snack_mad_s: 10.0,
                leave_s: 2.0,
            }
        }
    }

    pub fn mood_intervals(&self, state: SitterState) -> Option<(f32, f32)> {
        match state {
            SitterState::New | SitterState::Waiting => Some((self.wait_happy_s, self.wait_mad_s)),
            SitterState::Ready => Some((self.ready_happy_s, self.ready_mad_s)),
            SitterState::Ordered => Some((self.order_happy_s, self.order_mad_s)),
            SitterState::Check => Some((self.check_happy_s, self.check_mad_s)),
            SitterState::Snack => Some((self.snack_happy_s, self.snack_mad_s)),
            _ => None,
        }
    }
}

/// Configuration for sitter behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitterConfig {
    pub base_patience: f32,
    pub wait_patience: f32,
    pub decide_time: f32,
    pub order_time: f32,
    pub eat_time: f32,
    pub snack_time: f32,
    pub check_time: f32,
}

impl Default for SitterConfig {
    fn default() -> Self {
        Self {
            base_patience: 60.0,
            wait_patience: 60.0,
            decide_time: 14.0,
            order_time: 66.0,
            eat_time: 14.0,
            snack_time: 10.0,
            check_time: 60.0,
        }
    }
}

/// Sitter Group - manages a group of sitters together (1:1 port of `sitterGroup.ls` + `mood_timer.ls`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitterGroup {
    pub id: GroupId,
    pub group_type: String,
    pub sitters: Vec<Sitter>,
    pub state: GroupState,
    pub state_timer: f32,
    pub queue_time_ms: u32,
    pub delay_ms: u32,
    pub config: SitterConfig,
    pub time_multiplier: f32,
    pub target_table: Option<TableId>,
    /// Exact Lingo `sitterGroup.mood` (0..=10, starts at 6)
    #[serde(default = "default_mood_points")]
    pub mood_points: i32,
    /// Elapsed seconds since last `madMS` heart loss in current passive state (`mood_timer.ms`)
    #[serde(default)]
    pub mood_elapsed_s: f32,
    /// Total elapsed seconds in current passive state (`mood_timer.totalms`)
    #[serde(default)]
    pub mood_total_s: f32,
    /// Last mood change direction (+1 for heart gain, -1 for heart loss) for `"better"` / `"worse"` anim
    #[serde(default)]
    pub last_mood_delta: i32,
    /// Remaining seconds for `"better"` (hearts) / `"worse"` (annoyed) status anim
    #[serde(default)]
    pub mood_anim_timer: f32,
    /// True while dragged by cursor (`sitterGroup.pickedUp` clamps `mood >= 1`)
    #[serde(default)]
    pub picked_up: bool,
}

fn default_mood_points() -> i32 {
    6
}

impl SitterGroup {
    pub fn new(
        id: GroupId,
        group_type: String,
        party_size: u32,
        color_choices: Vec<CustomerColor>,
        mut config: SitterConfig,
        delay_ms: u32,
        queue_time_ms: u32,
    ) -> Self {
        let timing = CustomerTypeTiming::for_group_type(&group_type);
        config.decide_time = timing.decide_s;
        config.eat_time = timing.eat_s;
        config.wait_patience = timing.wait_mad_s * 6.0;
        config.order_time = timing.order_mad_s * 6.0;
        config.check_time = timing.check_mad_s * 6.0;

        let mut sitters = Vec::new();
        for i in 0..party_size {
            let color = color_choices.get(i as usize).copied().unwrap_or(CustomerColor::Green);
            let sitter_id = SitterId(rand::range_int(1, i32::MAX) as u64);
            let mut s = Sitter::new(sitter_id, id, color, &config);
            s.state = SitterState::Waiting;
            s.mood = 0.6;
            sitters.push(s);
        }

        Self {
            id,
            group_type,
            sitters,
            state: GroupState::Waiting,
            state_timer: 0.0,
            queue_time_ms,
            delay_ms,
            config,
            time_multiplier: 1.0,
            target_table: None,
            mood_points: 6,
            mood_elapsed_s: 0.0,
            mood_total_s: 0.0,
            last_mood_delta: 0,
            mood_anim_timer: 0.0,
            picked_up: false,
        }
    }

    pub fn timing(&self) -> CustomerTypeTiming {
        CustomerTypeTiming::for_group_type(&self.group_type)
    }

    /// Exact port of `sitterGroup.changeMood(num)` & `setMood(num)` (`sitterGroup.ls` lines 138-163)
    pub fn apply_mood_delta(&mut self, delta: i32) {
        if delta == 0 {
            return;
        }
        let is_tutorial = self.group_type.to_ascii_lowercase().contains("tutorial");
        let min_mood = if self.picked_up || is_tutorial { 1 } else { 0 };
        let new_mood = (self.mood_points + delta).clamp(min_mood, 10);
        if new_mood != self.mood_points {
            self.last_mood_delta = if new_mood > self.mood_points { 1 } else { -1 };
            self.mood_anim_timer = 1.2;
        } else if delta < 0 && new_mood == 1 {
            self.last_mood_delta = -1;
            self.mood_anim_timer = 1.2;
        }
        self.mood_points = new_mood;
        self.mood_elapsed_s = 0.0;
        self.sync_sitters_mood();
    }

    /// Exact port of `sitterGroup.stopMoodTimer(addresults)` (`sitterGroup.ls` lines 101-106 & `mood_timer.ls` lines 51-58)
    pub fn stop_mood_timer(&mut self, add_results: bool, state: SitterState) {
        let timing = self.timing();
        if add_results {
            if let Some((happy_s, _)) = timing.mood_intervals(state) {
                let scaled_happy_s = (happy_s * self.time_multiplier).max(0.5);
                if self.mood_total_s < scaled_happy_s {
                    self.apply_mood_delta(timing.time_mood);
                }
            }
        }
        self.mood_elapsed_s = 0.0;
        self.mood_total_s = 0.0;
    }

    fn sync_sitters_mood(&mut self) {
        let ratio = (self.mood_points as f32 / 10.0).clamp(0.0, 1.0);
        for sitter in &mut self.sitters {
            sitter.mood = ratio;
            sitter.patience = ratio * sitter.max_patience;
        }
    }

    pub fn update(&mut self, dt: f32, room: &crate::room::RoomManager) -> Vec<SitterEvent> {
        let scaled_dt = dt * self.time_multiplier;
        self.state_timer += scaled_dt;
        if self.mood_anim_timer > 0.0 {
            self.mood_anim_timer = (self.mood_anim_timer - dt).max(0.0);
            if self.mood_anim_timer <= 0.0 && self.mood_points > 1 {
                self.last_mood_delta = 0;
            }
        }

        let prev_state = self.sitters.first().map(|s| s.state).unwrap_or(SitterState::Waiting);
        let mut events = Vec::new();

        // Update individual sitters (movement & active timers: Deciding, Eating, Scoring, Leaving, Ditching)
        for sitter in &mut self.sitters {
            // Prevent old linear patience drain from prematurely ditching before mood_timer
            sitter.patience = sitter.max_patience;
            let event = sitter.update(dt, self.time_multiplier, room);
            if event != SitterEvent::None {
                events.push(event);
            }
        }

        let cur_state = self.sitters.first().map(|s| s.state).unwrap_or(SitterState::Waiting);
        if cur_state != prev_state {
            self.mood_elapsed_s = 0.0;
            self.mood_total_s = 0.0;
        }

        // Exact Lingo `mood_timer.exitFrame`: in passive states (Waiting, Ready, Ordered, Check, Snack),
        // decrement `mood` by 1 every `madMS`. When `mood == 0`, customer ditches!
        let timing = self.timing();
        if let Some((_, mad_s)) = timing.mood_intervals(cur_state) {
            let effective_mad_s = (mad_s * self.time_multiplier).max(1.0);
            self.mood_elapsed_s += dt;
            self.mood_total_s += dt;
            if self.mood_elapsed_s >= effective_mad_s {
                self.mood_elapsed_s = 0.0;
                self.apply_mood_delta(-1);
                if self.mood_points <= 0 {
                    for s in &mut self.sitters {
                        s.ditch();
                    }
                    events.push(SitterEvent::Ditch);
                }
            }
        } else {
            self.mood_elapsed_s = 0.0;
            self.mood_total_s = 0.0;
        }

        self.sync_sitters_mood();

        // Group state machine
        match self.state {
            GroupState::Waiting => {
                if self.target_table.is_some() {
                    self.state = GroupState::Seated;
                }
            }
            GroupState::Seated => {
                let all_ready = self.sitters.iter().all(|s| {
                    matches!(
                        s.state,
                        SitterState::Ready
                            | SitterState::Ordered
                            | SitterState::Eating
                            | SitterState::Snack
                            | SitterState::Check
                    )
                });
                if all_ready {
                    self.state = GroupState::Eating;
                }
            }
            GroupState::Eating => {
                let all_done = self.sitters.iter().all(|s| {
                    matches!(
                        s.state,
                        SitterState::Check | SitterState::Scoring | SitterState::Leaving
                    )
                });
                if all_done {
                    self.state = GroupState::Done;
                }
            }
            GroupState::Done => {
                let all_leaving = self.sitters.iter().all(|s| s.state == SitterState::Leaving);
                if all_leaving {
                    self.state = GroupState::Leaving;
                }
            }
            GroupState::Leaving => {}
        }

        events
    }

    pub fn seat_at_table(&mut self, table_id: TableId, seats: &[Seat]) {
        self.stop_mood_timer(true, SitterState::Waiting);
        self.target_table = Some(table_id);

        for (i, sitter) in self.sitters.iter_mut().enumerate() {
            if i < seats.len() {
                sitter.assign_seat(table_id, i);
                sitter.move_to(seats[i].grid_pos.to_world(37.0));
                sitter.transition(SitterState::Deciding);
            }
        }

        self.state = GroupState::Seated;
        self.sync_sitters_mood();
    }

    /// Called when Flo takes the group's order (`sgstate.ready.ls` line 50)
    pub fn on_order_taken(&mut self, food: String) {
        self.stop_mood_timer(true, SitterState::Ready);
        for sitter in &mut self.sitters {
            sitter.take_order(food.clone());
        }
        self.sync_sitters_mood();
    }

    /// Called when Flo delivers food to the table (`sgstate.ordered.ls` line 43)
    pub fn on_food_delivered(&mut self) {
        self.stop_mood_timer(true, SitterState::Ordered);
        for sitter in &mut self.sitters {
            sitter.serve_food();
        }
        self.sync_sitters_mood();
    }

    /// Called when Flo delivers the check (`sgstate.check.ls` line 44 & `sitterGroup.ls` line 247)
    pub fn on_check_delivered(&mut self) -> u32 {
        self.stop_mood_timer(true, SitterState::Check);
        let timing = self.timing();
        let total_tip = (self.sitters.len() as u32 * timing.base_tip)
            + (self.mood_points.max(0) as u32 * timing.mood_tip);
        for sitter in &mut self.sitters {
            sitter.transition(SitterState::Scoring);
        }
        self.sync_sitters_mood();
        total_tip
    }

    pub fn set_time_multiplier(&mut self, multiplier: f32) {
        self.time_multiplier = multiplier;
        for sitter in &mut self.sitters {
            sitter.set_time_multiplier(multiplier);
        }
    }

    pub fn change_mood(&mut self, amount: f32) {
        let delta_hearts = (amount * 4.0).round() as i32;
        self.apply_mood_delta(delta_hearts.max(1));
    }

    pub fn is_done(&self) -> bool {
        matches!(self.state, GroupState::Leaving)
            && self.sitters.iter().all(|s| s.state == SitterState::Leaving)
    }

    pub fn all_left(&self) -> bool {
        self.sitters
            .iter()
            .all(|s| matches!(s.state, SitterState::Leaving | SitterState::Ditching))
    }
}

/// Sitter System - manages all sitter groups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitterSystem {
    pub groups: HashMap<GroupId, SitterGroup>,
    pub waiting_groups: VecDeque<GroupId>,
    pub seated_groups: VecDeque<GroupId>,
    pub config: SitterConfig,
    pub next_group_id: u64,
    pub next_sitter_id: u64,
}

impl SitterSystem {
    pub fn new(config: SitterConfig) -> Self {
        Self {
            groups: HashMap::new(),
            waiting_groups: VecDeque::new(),
            seated_groups: VecDeque::new(),
            config,
            next_group_id: 1,
            next_sitter_id: 1,
        }
    }

    pub fn spawn_group(
        &mut self,
        group_type: String,
        party_size: u32,
        colors: Vec<CustomerColor>,
        delay_ms: u32,
        queue_time_ms: u32,
    ) -> GroupId {
        let id = GroupId(self.next_group_id);
        self.next_group_id += 1;

        let mut group = SitterGroup::new(
            id,
            group_type,
            party_size,
            colors,
            self.config.clone(),
            delay_ms,
            queue_time_ms,
        );
        let queue_idx = self.waiting_groups.len() as f32;
        for (i, sitter) in group.sitters.iter_mut().enumerate() {
            let wait_pos = Vec2::new(40.0 + (i as f32) * 18.0, 140.0 + queue_idx * 56.0);
            sitter.position = wait_pos;
            sitter.target_position = wait_pos;
        }

        self.groups.insert(id, group);
        self.waiting_groups.push_back(id);

        id
    }

    pub fn update(&mut self, dt: f32, room: &crate::room::RoomManager) -> Vec<(GroupId, SitterEvent)> {
        let mut all_events = Vec::new();

        // Update seated groups
        let seated_ids: Vec<GroupId> = self.seated_groups.iter().cloned().collect();
        for group_id in seated_ids {
            if let Some(group) = self.groups.get_mut(&group_id) {
                let events = group.update(dt, room);
                for event in events {
                    if !all_events.contains(&(group_id, event)) {
                        all_events.push((group_id, event));
                    }
                }

                // Remove from active seated queue when done; `GameManager::update` removes from `self.groups` after scoring/clearing table
                if group.all_left() {
                    self.seated_groups.retain(|g| *g != group_id);
                }
            }
        }

        // Update waiting groups
        let waiting_ids: Vec<GroupId> = self.waiting_groups.iter().cloned().collect();
        for group_id in waiting_ids {
            if let Some(group) = self.groups.get_mut(&group_id) {
                let events = group.update(dt, room);
                let mut ditched = false;
                for event in events {
                    if matches!(event, SitterEvent::Ditch | SitterEvent::Ditched) {
                        ditched = true;
                    }
                    if !all_events.contains(&(group_id, event)) {
                        all_events.push((group_id, event));
                    }
                }
                if ditched || group.all_left() {
                    self.waiting_groups.retain(|g| *g != group_id);
                }
            }
        }

        all_events
    }

    pub fn seat_next_group(&mut self, room: &mut crate::room::RoomManager) -> Option<GroupId> {
        if let Some(group_id) = self.waiting_groups.front().copied() {
            if let Some(group) = self.groups.get(&group_id) {
                if let Some(table_id) = room.get_available_table(group.sitters.len() as u32) {
                    if self.seat_group_at_table(group_id, table_id, room) {
                        return Some(group_id);
                    }
                }
            }
        }
        None
    }

    pub fn seat_group_at_table(
        &mut self,
        group_id: GroupId,
        table_id: TableId,
        room: &mut crate::room::RoomManager,
    ) -> bool {
        let Some(group) = self.groups.get_mut(&group_id) else {
            return false;
        };
        let Some(table) = room.tables.get(&table_id) else {
            return false;
        };
        if table.state != crate::room::TableState::Clean
            || table.available_seats() < group.sitters.len()
        {
            return false;
        }
        let seats: Vec<Seat> = table
            .seats
            .iter()
            .filter(|s| !s.occupied)
            .cloned()
            .collect();
        group.picked_up = false;
        group.seat_at_table(table_id, &seats);

        // Exact `table.ls` lines 303-318 & `chair.ls` lines 40-60:
        // Compare each sitter's color with the chair color; award +1 heart per match (`getColorBonus`),
        // or recolor the chair to the sitter's color when different (unless grey chair or critic).
        let mut match_bonus_hearts = 0i32;
        let sitter_info: Vec<(SitterId, CustomerColor)> =
            group.sitters.iter().map(|s| (s.id, s.color)).collect();
        if let Some(t) = room.tables.get_mut(&table_id) {
            t.group_id = Some(group_id);
            t.state = crate::room::TableState::Occupied;
            for (i, (sid, scol)) in sitter_info.into_iter().enumerate() {
                t.occupy_seat(i, sid);
                if let Some(seat) = t.seats.get_mut(i) {
                    let scol_str = match scol {
                        CustomerColor::Red => "red",
                        CustomerColor::Blue => "blue",
                        CustomerColor::Green => "green",
                        CustomerColor::Yellow => "yellow",
                        CustomerColor::Purple => "purple",
                        CustomerColor::None => "none",
                    };
                    if seat.color != "grey" && scol_str != "none" {
                        if seat.color == scol_str {
                            seat.bonus_multiplier = (seat.bonus_multiplier + 1).min(4);
                            match_bonus_hearts += 1;
                        } else {
                            seat.color = scol_str.to_string();
                            seat.bonus_multiplier = 1;
                        }
                    } else if (seat.color == "none" || seat.color == "grey") && scol_str != "none" {
                        seat.color = scol_str.to_string();
                        seat.bonus_multiplier = 1;
                    }
                }
            }
        }
        if match_bonus_hearts > 0 {
            let color_mood = group.timing().color_mood;
            group.apply_mood_delta(match_bonus_hearts * color_mood);
        }

        self.waiting_groups.retain(|&g| g != group_id);
        self.seated_groups.push_back(group_id);
        true
    }

    pub fn get_group(&self, id: GroupId) -> Option<&SitterGroup> {
        self.groups.get(&id)
    }

    pub fn get_group_mut(&mut self, id: GroupId) -> Option<&mut SitterGroup> {
        self.groups.get_mut(&id)
    }

    pub fn remove_group(&mut self, id: GroupId) {
        self.groups.remove(&id);
        self.waiting_groups.retain(|g| *g != id);
        self.seated_groups.retain(|g| *g != id);
    }

    pub fn set_time_multiplier(&mut self, multiplier: f32) {
        for group in self.groups.values_mut() {
            group.set_time_multiplier(multiplier);
        }
    }

    pub fn change_mood(&mut self, amount: f32) {
        for group in self.groups.values_mut() {
            group.change_mood(amount);
        }
    }
}

impl System for SitterSystem {
    fn name(&self) -> &str { "SitterSystem" }
    fn priority(&self) -> i32 { 50 }

    fn run(&mut self, _world: &mut World, _dt: f32) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sitter_creation() {
        let config = SitterConfig::default();
        let sitter = Sitter::new(
            SitterId(1),
            GroupId(1),
            CustomerColor::Green,
            &config,
        );
        
        assert_eq!(sitter.state, SitterState::New);
        assert_eq!(sitter.color, CustomerColor::Green);
        assert!(sitter.patience > 0.0);
    }
    
    #[test]
    fn test_sitter_group() {
        let config = SitterConfig::default();
        let group = SitterGroup::new(
            GroupId(1),
            "family".to_string(),
            4,
            vec![CustomerColor::Green, CustomerColor::Red, CustomerColor::Blue, CustomerColor::Yellow],
            config,
            3000,
            5000,
        );
        
        assert_eq!(group.sitters.len(), 4);
        assert_eq!(group.state, GroupState::Waiting);
    }
    
    #[test]
    fn test_sitter_transitions() {
        let config = SitterConfig::default();
        let mut sitter = Sitter::new(
            SitterId(1),
            GroupId(1),
            CustomerColor::Green,
            &config,
        );
        
        // Simulate time passing through states
        sitter.state_timer = 1.0;
        let _ = sitter.update(1.0, 1.0, &crate::room::RoomManager::new(10, 10, 32.0));
        // Would transition through states with enough time
    }
}
