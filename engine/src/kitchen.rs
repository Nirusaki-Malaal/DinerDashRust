//! Kitchen System
//! 
//! Handles food preparation, orders, and kitchen upgrades

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use crate::math::Vec2;
use crate::game::{TableId, SitterId};
use crate::room::{Kitchen, KitchenOrder, GridPos};

/// Kitchen station types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StationType {
    Grill,
    Fryer,
    Drink,
    Dessert,
    Prep,
}

/// Kitchen station
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Station {
    pub station_type: StationType,
    pub position: GridPos,
    pub current_order: Option<KitchenOrder>,
    pub progress: f32, // 0.0 - 1.0
    pub speed_multiplier: f32,
    pub upgrade_level: u32,
}

impl Station {
    pub fn new(station_type: StationType, position: GridPos) -> Self {
        Self {
            station_type,
            position,
            current_order: None,
            progress: 0.0,
            speed_multiplier: 1.0,
            upgrade_level: 0,
        }
    }
    
    pub fn can_handle(&self, food_type: &str) -> bool {
        match self.station_type {
            StationType::Grill => matches!(food_type, "burger" | "steak" | "chicken"),
            StationType::Fryer => matches!(food_type, "fries" | "onion_rings" | "fish"),
            StationType::Drink => matches!(food_type, "soda" | "coffee" | "shake"),
            StationType::Dessert => matches!(food_type, "pie" | "ice_cream" | "cake"),
            StationType::Prep => true, // Can handle anything
        }
    }
    
    pub fn start_order(&mut self, order: KitchenOrder) {
        self.current_order = Some(order);
        self.progress = 0.0;
    }
    
    pub fn update(&mut self, dt: f32) -> Option<KitchenOrder> {
        if let Some(order) = &mut self.current_order {
            self.progress += dt * self.speed_multiplier;
            if self.progress >= 1.0 {
                let completed = self.current_order.take().unwrap();
                self.progress = 0.0;
                return Some(completed);
            }
        }
        None
    }
    
    pub fn upgrade(&mut self, props: &std::collections::HashMap<String, f32>) {
        if let Some(level) = props.get("level") {
            self.upgrade_level = *level as u32;
            self.speed_multiplier = 1.0 + (self.upgrade_level as f32 * 0.2);
        }
    }
}

/// Enhanced Kitchen with multiple stations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedKitchen {
    pub stations: Vec<Station>,
    pub queue: VecDeque<KitchenOrder>,
    pub completed: VecDeque<KitchenOrder>,
    pub food_ready_ms: u32,
    pub auto_assign: bool,
}

impl AdvancedKitchen {
    pub fn new(grid_pos: GridPos) -> Self {
        let mut stations = Vec::new();
        
        // Default station layout
        stations.push(Station::new(StationType::Grill, GridPos::new(grid_pos.x, grid_pos.y)));
        stations.push(Station::new(StationType::Fryer, GridPos::new(grid_pos.x + 1, grid_pos.y)));
        stations.push(Station::new(StationType::Drink, GridPos::new(grid_pos.x + 2, grid_pos.y)));
        stations.push(Station::new(StationType::Prep, GridPos::new(grid_pos.x, grid_pos.y + 1)));
        
        Self {
            stations,
            queue: VecDeque::new(),
            completed: VecDeque::new(),
            food_ready_ms: 5000,
            auto_assign: true,
        }
    }
    
    pub fn add_order(&mut self, order: KitchenOrder) {
        if self.auto_assign {
            self.assign_to_station(order);
        } else {
            self.queue.push_back(order);
        }
    }
    
    fn assign_to_station(&mut self, order: KitchenOrder) {
        // Find best station
        for station in &mut self.stations {
            if station.current_order.is_none() && station.can_handle(&order.food_type) {
                station.start_order(order);
                return;
            }
        }
        
        // No available station, queue it
        self.queue.push_back(order);
    }
    
    pub fn update(&mut self, dt: f32) -> Vec<KitchenOrder> {
        let mut ready = Vec::new();
        
        // Update stations
        for station in &mut self.stations {
            if let Some(completed) = station.update(dt) {
                ready.push(completed);
            }
        }
        
        // Try to assign queued orders
        while let Some(order) = self.queue.pop_front() {
            let mut assigned = false;
            for station in &mut self.stations {
                if station.current_order.is_none() && station.can_handle(&order.food_type) {
                    station.start_order(order.clone());
                    assigned = true;
                    break;
                }
            }
            if !assigned {
                self.queue.push_front(order);
                break;
            }
        }
        
        ready
    }
    
    pub fn set_speed(&mut self, food_ready_ms: u32) {
        self.food_ready_ms = food_ready_ms;
        // Adjust station speeds proportionally
        let base_ms = 5000;
        let mult = base_ms as f32 / food_ready_ms as f32;
        for station in &mut self.stations {
            station.speed_multiplier = mult;
        }
    }
    
    pub fn upgrade_station(&mut self, station_type: StationType, props: &std::collections::HashMap<String, f32>) {
        for station in &mut self.stations {
            if station.station_type == station_type {
                station.upgrade(props);
            }
        }
    }
}

/// Kitchen System
pub struct KitchenSystem {
    pub kitchens: HashMap<Entity, AdvancedKitchen>,
}

impl KitchenSystem {
    pub fn new() -> Self {
        Self {
            kitchens: HashMap::new(),
        }
    }
    
    pub fn add_kitchen(&mut self, entity: Entity, kitchen: AdvancedKitchen) {
        self.kitchens.insert(entity, kitchen);
    }
    
    pub fn get_kitchen(&mut self, entity: Entity) -> Option<&mut AdvancedKitchen> {
        self.kitchens.get_mut(&entity)
    }
}

impl Default for KitchenSystem {
    fn default() -> Self {
        Self::new()
    }
}

use std::collections::HashMap;
use crate::ecs::{Entity, System, World};

impl System for KitchenSystem {
    fn name(&self) -> &str { "KitchenSystem" }
    fn priority(&self) -> i32 { 70 }
    
    fn run(&mut self, world: &mut World, dt: f32) {
        for kitchen in self.kitchens.values_mut() {
            let ready = kitchen.update(dt);
            for order in ready {
                // Would notify waiter or game manager
                // For now, just track
                kitchen.completed.push_back(order);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::room::GridPos;
    
    #[test]
    fn test_station() {
        let station = Station::new(StationType::Grill, GridPos::new(0, 0));
        assert!(station.can_handle("burger"));
        assert!(station.can_handle("steak"));
        assert!(!station.can_handle("fries"));
    }
    
    #[test]
    fn test_kitchen() {
        let mut kitchen = AdvancedKitchen::new(GridPos::new(0, 0));
        let order = KitchenOrder {
            table_id: TableId(1),
            seat_index: 0,
            food_type: "burger".to_string(),
            ready_time: 0.0,
        };
        
        kitchen.add_order(order);
        assert_eq!(kitchen.stations[0].current_order.is_some(), true);
    }
}