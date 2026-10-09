//! Waiter System
//! 
//! Handles waiter movement, task queue, and interactions

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use crate::math::{Vec2, clamp};
use crate::ecs::{Entity, Component, World, System};
use crate::game::{SitterState, TableId, SitterId};
use crate::room::RoomManager;

/// Waiter states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WaiterState {
    Idle,
    MovingToTable,
    TakingOrder,
    MovingToKitchen,
    PickingUpFood,
    DeliveringFood,
    DeliveringCheck,
    CleaningTable,
}

impl Default for WaiterState {
    fn default() -> Self {
        WaiterState::Idle
    }
}

/// Task types for waiter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WaiterTask {
    TakeOrder { table_id: TableId, seat_index: usize },
    DropTickets,
    PickupFood { table_id: TableId },
    DeliverFood { table_id: TableId, seat_index: usize, food_type: String },
    DeliverCheck { table_id: TableId },
    CleanTable { table_id: TableId },
    DropDishes,
    UsePodium,
}

impl WaiterTask {
    /// Table this action ends at (used for the destination checkmarks)
    pub fn target_table(&self) -> Option<TableId> {
        match self {
            WaiterTask::TakeOrder { table_id, .. }
            | WaiterTask::DeliverFood { table_id, .. }
            | WaiterTask::DeliverCheck { table_id }
            | WaiterTask::CleanTable { table_id } => Some(*table_id),
            WaiterTask::DropTickets
            | WaiterTask::PickupFood { .. }
            | WaiterTask::DropDishes
            | WaiterTask::UsePodium => None,
        }
    }
}

/// Waiter component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Waiter {
    pub state: WaiterState,
    pub position: Vec2,
    pub target_position: Vec2,
    pub speed: f32,
    pub current_task: Option<WaiterTask>,
    pub task_queue: VecDeque<WaiterTask>,
    pub carrying_food: Option<String>,
    pub carrying_check: Option<TableId>,
    pub animation: String,
    pub facing: Vec2,
    #[serde(default)]
    pub path: VecDeque<Vec2>,
    #[serde(default)]
    pub carried_tickets: Vec<TableId>,
    #[serde(default)]
    pub carried_food: Vec<TableId>,
    #[serde(default)]
    pub carried_dishes: u32,
    #[serde(default)]
    pub bus_dishes_count: u32,
    #[serde(default)]
    pub bus_empty_timer: Option<f32>,
    #[serde(default)]
    pub podium_end_timer: Option<f32>,
}

impl Default for Waiter {
    fn default() -> Self {
        Self {
            state: WaiterState::Idle,
            position: Vec2::ZERO,
            target_position: Vec2::ZERO,
            speed: 225.0, // pixels per second
            current_task: None,
            task_queue: VecDeque::new(),
            carrying_food: None,
            carrying_check: None,
            animation: "idle".to_string(),
            facing: Vec2::new(0.0, 1.0),
            path: VecDeque::new(),
            carried_tickets: Vec::new(),
            carried_food: Vec::new(),
            carried_dishes: 0,
            bus_dishes_count: 0,
            bus_empty_timer: None,
            podium_end_timer: None,
        }
    }
}

impl Waiter {
    pub fn new(start_pos: Vec2) -> Self {
        Self {
            position: start_pos,
            target_position: start_pos,
            ..Default::default()
        }
    }
    
    /// Add a task to the queue
    pub fn add_task(&mut self, task: WaiterTask) -> bool {
        if self.task_queue.len() >= 8
            || self.current_task.as_ref() == Some(&task)
            || self.task_queue.contains(&task)
        {
            return false;
        }
        self.task_queue.push_back(task);
        true
    }

    pub fn has_task_matching<F: Fn(&WaiterTask) -> bool>(&self, pred: F) -> bool {
        self.current_task.as_ref().is_some_and(&pred)
            || self.task_queue.iter().any(pred)
    }

    /// Number of queued+current actions targeting this table, capped at the
    /// three checkmarks the original table_view can display.
    pub fn pending_marks(&self, table_id: TableId) -> u8 {
        let current = self
            .current_task
            .as_ref()
            .and_then(|t| t.target_table())
            .filter(|&t| t == table_id);
        let queued = self
            .task_queue
            .iter()
            .filter(|t| t.target_table() == Some(table_id))
            .count();
        (queued + current.map(|_| 1).unwrap_or(0)).min(3) as u8
    }
    
    /// Add high-priority task to front
    pub fn add_urgent_task(&mut self, task: WaiterTask) {
        if self.add_task(task) {
            let last = self.task_queue.pop_back().unwrap();
            self.task_queue.push_front(last);
        }
    }
    
    /// Set movement speed
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed.max(50.0).min(500.0);
    }
    
    /// Update waiter logic
    pub fn update(&mut self, dt: f32, room: &mut RoomManager) -> Vec<WaiterEvent> {
        let mut events = Vec::new();
        let dt_pos = dt.max(0.0);

        // Bus station 1500ms auto-empty timer (`busspot.ls` lines 97-102)
        if let Some(rem) = &mut self.bus_empty_timer {
            *rem -= dt_pos;
            if *rem <= 0.0 {
                self.bus_dishes_count = 0;
                self.bus_empty_timer = None;
            }
        }

        // Podium applause finish animation timer (`waiter_view.ls` line 288 `podium_3`)
        if let Some(rem) = &mut self.podium_end_timer {
            *rem -= dt_pos;
            if *rem <= 0.0 {
                self.podium_end_timer = None;
            }
        }
        
        // Consume an axis-aligned tile path without overshoot or teleportation.
        if self.current_task.is_some()
            && self.path.is_empty()
            && self.position.distance(self.target_position) > 1.0
        {
            let task = self.current_task.clone().unwrap();
            self.start_task(task, room);
        }
        let mut remaining = self.speed * dt_pos;
        while let Some(&next) = self.path.front() {
            let delta = next - self.position;
            let distance = delta.length();
            if distance > 0.001 {
                self.facing = delta / distance;
            }
            if distance <= remaining {
                self.position = next;
                remaining -= distance;
                self.path.pop_front();
            } else {
                self.position += self.facing * remaining;
                break;
            }
        }
        
        if self.state == WaiterState::Idle {
            self.animation = "idle".to_string();
            if let Some(task) = self.task_queue.pop_front() {
                if task != WaiterTask::UsePodium {
                    if let Some(podium) = &mut room.podium {
                        podium.stop();
                    }
                    self.podium_end_timer = None;
                }
                self.current_task = Some(task.clone());
                self.start_task(task, room);
            }
        }

        if let Some(task) = self.current_task.clone() {
            if self.path.is_empty() && self.position.distance(self.target_position) <= 1.5 {
                self.position = self.target_position;
                // Per waiter.ls line 99 & waiter_view.ls line 301: face "down" on arrival
                self.facing = Vec2::new(0.0, 1.0);
                match task {
                    WaiterTask::TakeOrder { table_id, .. } => {
                        events.push(WaiterEvent::OrderTaken { table_id });
                    }
                    WaiterTask::DropTickets => {
                        let tables = std::mem::take(&mut self.carried_tickets);
                        if !tables.is_empty() {
                            events.push(WaiterEvent::TicketsDropped { tables });
                        }
                    }
                    WaiterTask::PickupFood { table_id } => {
                        events.push(WaiterEvent::FoodPickedUp { table_id });
                    }
                    WaiterTask::DeliverFood {
                        table_id,
                        food_type,
                        ..
                    } => {
                        if let Some(pos) = self.carried_food.iter().position(|&t| t == table_id) {
                            self.carried_food.remove(pos);
                        }
                        if self.carried_food.is_empty() {
                            self.carrying_food = None;
                        }
                        events.push(WaiterEvent::FoodDelivered {
                            table_id,
                            food_type,
                        });
                    }
                    WaiterTask::DeliverCheck { table_id } => {
                        self.carrying_check = None;
                        events.push(WaiterEvent::CheckDelivered { table_id });
                    }
                    WaiterTask::CleanTable { table_id } => {
                        events.push(WaiterEvent::TableCleaned { table_id });
                    }
                    WaiterTask::DropDishes => {
                        let count = self.carried_dishes;
                        self.carried_dishes = 0;
                        if count > 0 {
                            self.bus_dishes_count = self.bus_dishes_count.saturating_add(count).min(2);
                            self.bus_empty_timer = Some(1.5);
                            events.push(WaiterEvent::DishesDropped { count });
                        }
                    }
                    WaiterTask::UsePodium => {
                        events.push(WaiterEvent::ArrivedAtPodium);
                    }
                }
                self.current_task = None;
                self.state = WaiterState::Idle;
            }
        }
        
        events
    }
    
    fn start_task(&mut self, task: WaiterTask, room: &RoomManager) {
        let path_opt = match &task {
            WaiterTask::TakeOrder { table_id, .. } => {
                self.state = WaiterState::MovingToTable;
                room.table_service_path(self.position, *table_id)
            }
            WaiterTask::DropTickets => {
                self.state = WaiterState::MovingToKitchen;
                room.station_service_path(self.position, Vec2::new(296.0, 211.0))
            }
            WaiterTask::PickupFood { .. } => {
                self.state = WaiterState::MovingToKitchen;
                room.station_service_path(self.position, Vec2::new(407.0, 211.0))
            }
            WaiterTask::DeliverFood { table_id, .. } => {
                self.state = WaiterState::DeliveringFood;
                room.table_service_path(self.position, *table_id)
            }
            WaiterTask::DeliverCheck { table_id } => {
                self.state = WaiterState::DeliveringCheck;
                self.carrying_check = Some(*table_id);
                room.table_service_path(self.position, *table_id)
            }
            WaiterTask::CleanTable { table_id } => {
                self.state = WaiterState::CleaningTable;
                room.table_service_path(self.position, *table_id)
            }
            WaiterTask::DropDishes => {
                self.state = WaiterState::CleaningTable;
                room.station_service_path(self.position, Vec2::new(666.0, 211.0))
            }
            WaiterTask::UsePodium => {
                self.state = WaiterState::MovingToTable;
                room.station_service_path(self.position, Vec2::new(148.0, 507.0))
            }
        };
        if let Some(path) = path_opt {
            self.target_position = path.last().copied().unwrap_or(self.position);
            self.path = path.into();
        } else {
            self.cancel_current();
        }
    }

    fn cancel_current(&mut self) {
        self.current_task = None;
        self.path.clear();
        self.target_position = self.position;
        self.carrying_check = None;
        self.state = WaiterState::Idle;
    }
}

/// Events from waiter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WaiterEvent {
    OrderTaken { table_id: TableId },
    TicketsDropped { tables: Vec<TableId> },
    FoodPickedUp { table_id: TableId },
    FoodDelivered { table_id: TableId, food_type: String },
    CheckDelivered { table_id: TableId },
    TableCleaned { table_id: TableId },
    DishesDropped { count: u32 },
    ArrivedAtPodium,
    TaskCompleted { task: WaiterTask },
}

/// Waiter System
pub struct WaiterSystem {
    pub waiters: HashMap<Entity, Waiter>,
    pub default_speed: f32,
}

impl WaiterSystem {
    pub fn new() -> Self {
        Self {
            waiters: HashMap::new(),
            default_speed: 200.0,
        }
    }
    
    pub fn add_waiter(&mut self, entity: Entity, start_pos: Vec2) {
        self.waiters.insert(entity, Waiter::new(start_pos));
    }
    
    pub fn get_waiter(&self, entity: Entity) -> Option<&Waiter> {
        self.waiters.get(&entity)
    }
    
    pub fn get_waiter_mut(&mut self, entity: Entity) -> Option<&mut Waiter> {
        self.waiters.get_mut(&entity)
    }
    
    pub fn assign_task(&mut self, entity: Entity, task: WaiterTask) {
        if let Some(waiter) = self.waiters.get_mut(&entity) {
            waiter.add_task(task);
        }
    }
    
    pub fn assign_urgent_task(&mut self, entity: Entity, task: WaiterTask) {
        if let Some(waiter) = self.waiters.get_mut(&entity) {
            waiter.add_urgent_task(task);
        }
    }
    
    pub fn set_speed(&mut self, entity: Entity, speed: f32) {
        if let Some(waiter) = self.waiters.get_mut(&entity) {
            waiter.set_speed(speed);
        }
    }
}

impl System for WaiterSystem {
    fn name(&self) -> &str { "WaiterSystem" }
    fn priority(&self) -> i32 { 60 }
    
    fn run(&mut self, world: &mut World, dt: f32) {
        // Would need room reference - typically called from GameManager
    }
}

impl Default for WaiterSystem {
    fn default() -> Self {
        Self::new()
    }
}

use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::Entity;
    
    #[test]
    fn test_waiter_creation() {
        let waiter = Waiter::new(Vec2::new(100.0, 100.0));
        assert_eq!(waiter.state, WaiterState::Idle);
        assert_eq!(waiter.position, Vec2::new(100.0, 100.0));
    }
    
    #[test]
    fn test_waiter_task_queue() {
        let mut waiter = Waiter::new(Vec2::ZERO);
        waiter.add_task(WaiterTask::TakeOrder { table_id: TableId(1), seat_index: 0 });
        waiter.add_task(WaiterTask::DeliverCheck { table_id: TableId(1) });
        
        assert_eq!(waiter.task_queue.len(), 2);
    }
}
