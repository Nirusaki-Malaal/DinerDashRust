//! Room Management System
//! 
//! Handles table layout, grid, pathfinding, and room objects

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque, BinaryHeap};
use std::cmp::Ordering;
use crate::math::{Vec2, Rect, UVec2, clamp};
use crate::ecs::{Entity, Component, World, System};
use crate::game::{GameMode, Stage, SitterState, GroupId};

/// Grid cell types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellType {
    Empty,
    Floor,
    Wall,
    Table,
    Chair,
    Kitchen,
    Podium,
    Decoration,
}

/// Grid position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GridPos {
    pub x: i32,
    pub y: i32,
}

impl GridPos {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
    
    pub fn to_world(&self, cell_size: f32) -> Vec2 {
        if (cell_size - 37.0).abs() < 0.5 {
            Vec2::new(
                148.0 + (self.x - 1) as f32 * cell_size,
                137.0 + (self.y - 1) as f32 * cell_size,
            )
        } else {
            Vec2::new(
                self.x as f32 * cell_size + cell_size * 0.5,
                self.y as f32 * cell_size + cell_size * 0.5,
            )
        }
    }
    
    pub fn from_world(pos: Vec2, cell_size: f32) -> Self {
        if (cell_size - 37.0).abs() < 0.5 {
            Self {
                x: ((pos.x - 130.0) / cell_size).floor() as i32 + 1,
                y: ((pos.y - 119.0) / cell_size).floor() as i32 + 1,
            }
        } else {
            Self {
                x: (pos.x / cell_size).floor() as i32,
                y: (pos.y / cell_size).floor() as i32,
            }
        }
    }
    
    pub fn distance_to(&self, other: &GridPos) -> u32 {
        ((self.x - other.x).abs() + (self.y - other.y).abs()) as u32
    }
    
    pub fn neighbors(&self) -> [GridPos; 4] {
        [
            GridPos::new(self.x + 1, self.y),
            GridPos::new(self.x - 1, self.y),
            GridPos::new(self.x, self.y + 1),
            GridPos::new(self.x, self.y - 1),
        ]
    }
}

/// Table in the restaurant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub id: TableId,
    pub grid_pos: GridPos,
    pub capacity: u32,
    pub seats: Vec<Seat>,
    pub state: TableState,
    pub dirty_timer: Option<f32>,
    pub group_id: Option<GroupId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TableId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TableState {
    Clean,
    Dirty,
    Occupied,
    Reserved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Seat {
    pub grid_pos: GridPos,
    pub occupied: bool,
    pub sitter_id: Option<SitterId>,
    #[serde(default)]
    pub color: String,
    #[serde(default = "default_multiplier")]
    pub bonus_multiplier: u32,
}

fn default_multiplier() -> u32 {
    1
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SitterId(pub u64);

impl Table {
    pub fn new(id: TableId, grid_pos: GridPos, capacity: u32) -> Self {
        let mut seats = Vec::new();
        
        // Generate seats around table (simplified)
        for i in 0..capacity {
            let angle = (i as f32 / capacity as f32) * std::f32::consts::TAU;
            let offset_x = (angle.cos() * 1.5).round() as i32;
            let offset_y = (angle.sin() * 1.5).round() as i32;
            
            seats.push(Seat {
                grid_pos: GridPos::new(grid_pos.x + offset_x, grid_pos.y + offset_y),
                occupied: false,
                sitter_id: None,
                color: "none".to_string(),
                bonus_multiplier: 1,
            });
        }
        
        Self {
            id,
            grid_pos,
            capacity,
            seats,
            state: TableState::Clean,
            dirty_timer: None,
            group_id: None,
        }
    }
    
    pub fn available_seats(&self) -> usize {
        self.seats.iter().filter(|s| !s.occupied).count()
    }
    
    pub fn occupy_seat(&mut self, seat_index: usize, sitter_id: SitterId) -> bool {
        if seat_index < self.seats.len() && !self.seats[seat_index].occupied {
            self.seats[seat_index].occupied = true;
            self.seats[seat_index].sitter_id = Some(sitter_id);
            self.state = TableState::Occupied;
            true
        } else {
            false
        }
    }
    
    pub fn free_seat(&mut self, seat_index: usize) {
        if seat_index < self.seats.len() {
            self.seats[seat_index].occupied = false;
            self.seats[seat_index].sitter_id = None;
            
            if self.seats.iter().all(|s| !s.occupied) {
                self.state = TableState::Dirty;
                self.dirty_timer = Some(0.0);
            }
        }
    }
}

/// Podium/Host station (`podium.ls`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Podium {
    pub grid_pos: GridPos,
    pub upgrade_level: u32,
    pub mood_bonus: f32,
    #[serde(default = "default_true")]
    pub ready: bool,
    #[serde(default)]
    pub flo_here: bool,
    #[serde(default)]
    pub charge_progress: f32,
    #[serde(default)]
    pub active_timer: Option<f32>,
    #[serde(default)]
    pub cooldown_timer: Option<f32>,
}

fn default_true() -> bool {
    true
}

impl Podium {
    pub fn new(grid_pos: GridPos) -> Self {
        Self {
            grid_pos,
            upgrade_level: 0,
            mood_bonus: 1.0,
            ready: true,
            flo_here: false,
            charge_progress: 0.0,
            active_timer: None,
            cooldown_timer: None,
        }
    }
    
    pub fn set_properties(&mut self, props: &HashMap<String, f32>) {
        if let Some(level) = props.get("level") {
            self.upgrade_level = *level as u32;
        }
        if let Some(mood) = props.get("moodBonus") {
            self.mood_bonus = *mood;
        }
    }

    pub fn stop(&mut self) {
        self.flo_here = false;
        self.charge_progress = 0.0;
    }
}

/// Kitchen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kitchen {
    pub grid_pos: GridPos,
    pub food_ready_ms: u32,
    pub current_orders: VecDeque<KitchenOrder>,
    pub upgrade_level: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KitchenOrder {
    pub table_id: TableId,
    pub seat_index: usize,
    pub food_type: String,
    pub ready_time: f32,
}

impl Kitchen {
    pub fn new(grid_pos: GridPos) -> Self {
        Self {
            grid_pos,
            food_ready_ms: 5000, // Default 5 seconds
            current_orders: VecDeque::new(),
            upgrade_level: 0,
        }
    }
    
    pub fn set_speed(&mut self, food_ready_ms: u32) {
        self.food_ready_ms = food_ready_ms;
    }
    
    pub fn add_order(&mut self, table_id: TableId, seat_index: usize, food_type: String, current_time: f32) {
        self.current_orders.push_back(KitchenOrder {
            table_id,
            seat_index,
            food_type,
            ready_time: current_time + (self.food_ready_ms as f32 / 1000.0),
        });
    }
    
    pub fn update(&mut self, current_time: f32) -> Vec<KitchenOrder> {
        let mut ready = Vec::new();
        
        while let Some(order) = self.current_orders.front() {
            if current_time >= order.ready_time {
                ready.push(self.current_orders.pop_front().unwrap());
            } else {
                break;
            }
        }
        
        ready
    }
}

mod vec_map_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::collections::HashMap;
    use std::hash::Hash;

    pub fn serialize<K, V, S>(map: &HashMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
    where
        K: Serialize + Eq + Hash,
        V: Serialize,
        S: Serializer,
    {
        let vec: Vec<(&K, &V)> = map.iter().collect();
        vec.serialize(serializer)
    }

    pub fn deserialize<'de, K, V, D>(deserializer: D) -> Result<HashMap<K, V>, D::Error>
    where
        K: Deserialize<'de> + Eq + Hash,
        V: Deserialize<'de>,
        D: Deserializer<'de>,
    {
        let vec: Vec<(K, V)> = Vec::deserialize(deserializer)?;
        Ok(vec.into_iter().collect())
    }
}

/// Grid Manager for room layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridManager {
    pub width: i32,
    pub height: i32,
    pub cell_size: f32,
    #[serde(with = "vec_map_serde")]
    pub cells: HashMap<GridPos, CellType>,
    #[serde(with = "vec_map_serde")]
    pub walkable: HashMap<GridPos, bool>,
}

impl GridManager {
    pub fn new(width: i32, height: i32, cell_size: f32) -> Self {
        let mut cells = HashMap::new();
        let mut walkable = HashMap::new();
        let is_director_grid = (cell_size - 37.0).abs() < 0.5;
        
        // Initialize grid
        for x in 0..=width {
            for y in 0..=height {
                let pos = GridPos::new(x, y);
                // Per grid_manager.ls lines 19-24 & 71-73:
                // Rows 1..2 (kitchen/counter) and cols 1..2 in rows 1..9 (waitqueue) are blocked.
                let blocked = is_director_grid
                    && (x < 1 || y < 3 || (x <= 2 && y <= 9));
                cells.insert(pos, if blocked { CellType::Wall } else { CellType::Empty });
                walkable.insert(pos, !blocked);
            }
        }
        
        Self {
            width,
            height,
            cell_size,
            cells,
            walkable,
        }
    }
    
    pub fn set_cell(&mut self, pos: GridPos, cell_type: CellType) {
        let walkable = matches!(cell_type, CellType::Empty | CellType::Floor);
        self.cells.insert(pos, cell_type);
        self.walkable.insert(pos, walkable);
    }
    
    pub fn get_cell(&self, pos: GridPos) -> CellType {
        self.cells.get(&pos).copied().unwrap_or(CellType::Wall)
    }
    
    pub fn is_walkable(&self, pos: GridPos) -> bool {
        self.walkable.get(&pos).copied().unwrap_or(false)
    }
    
    pub fn in_bounds(&self, pos: GridPos) -> bool {
        if (self.cell_size - 37.0).abs() < 0.5 {
            pos.x >= 1 && pos.x <= self.width && pos.y >= 1 && pos.y <= self.height
        } else {
            pos.x >= 0 && pos.x < self.width && pos.y >= 0 && pos.y < self.height
        }
    }
    
    pub fn world_to_grid(&self, world: Vec2) -> GridPos {
        GridPos::from_world(world, self.cell_size)
    }
    
    pub fn grid_to_world(&self, grid: GridPos) -> Vec2 {
        grid.to_world(self.cell_size)
    }
    
    /// Find nearest walkable cell to target
    pub fn find_nearest_walkable(&self, target: GridPos, max_radius: i32) -> Option<GridPos> {
        if self.is_walkable(target) {
            return Some(target);
        }
        
        for radius in 1..=max_radius {
            for dx in -radius..=radius {
                for dy in -radius..=radius {
                    if dx.abs() == radius || dy.abs() == radius {
                        let pos = GridPos::new(target.x + dx, target.y + dy);
                        if self.in_bounds(pos) && self.is_walkable(pos) {
                            return Some(pos);
                        }
                    }
                }
            }
        }
        None
    }
}

/// A* Pathfinding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AStar {
    grid: GridManager,
}

impl AStar {
    pub fn new(grid: GridManager) -> Self {
        Self { grid }
    }
    
    pub fn find_path(&self, start: GridPos, goal: GridPos) -> Option<Vec<GridPos>> {
        if !self.grid.is_walkable(start) || !self.grid.is_walkable(goal) {
            return None;
        }
        
        let mut open_set = BinaryHeap::new();
        let mut came_from = HashMap::new();
        let mut g_score = HashMap::new();
        let mut f_score = HashMap::new();
        
        g_score.insert(start, 0);
        f_score.insert(start, Self::heuristic(start, goal));
        open_set.push(AStarNode {
            pos: start,
            f_score: Self::heuristic(start, goal),
        });
        
        while let Some(current_node) = open_set.pop() {
            let current = current_node.pos;
            
            if current == goal {
                return Some(Self::reconstruct_path(came_from, current));
            }
            
            for neighbor in current.neighbors() {
                if !self.grid.in_bounds(neighbor) || !self.grid.is_walkable(neighbor) {
                    continue;
                }
                
                let tentative_g = g_score.get(&current).unwrap_or(&u32::MAX) + 1;
                
                if tentative_g < *g_score.get(&neighbor).unwrap_or(&u32::MAX) {
                    came_from.insert(neighbor, current);
                    g_score.insert(neighbor, tentative_g);
                    let f = tentative_g + Self::heuristic(neighbor, goal);
                    f_score.insert(neighbor, f);
                    open_set.push(AStarNode { pos: neighbor, f_score: f });
                }
            }
        }
        
        None
    }
    
    fn heuristic(a: GridPos, b: GridPos) -> u32 {
        a.distance_to(&b)
    }
    
    fn reconstruct_path(came_from: HashMap<GridPos, GridPos>, mut current: GridPos) -> Vec<GridPos> {
        let mut path = vec![current];
        while let Some(&prev) = came_from.get(&current) {
            current = prev;
            path.push(current);
        }
        path.reverse();
        path
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AStarNode {
    pos: GridPos,
    f_score: u32,
}

impl Ord for AStarNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f_score.cmp(&self.f_score).then_with(|| self.pos.x.cmp(&other.pos.x))
    }
}

impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Room Manager - central room controller
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomManager {
    pub grid: GridManager,
    pub pathfinder: AStar,
    pub tables: HashMap<TableId, Table>,
    pub podium: Option<Podium>,
    pub kitchen: Kitchen,
    pub next_table_id: u64,
    pub time_multiplier: f32,
    pub bg_state: BackgroundState,
    pub snacks_enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackgroundState {
    Normal,
    Busy,
    Closing,
}

impl RoomManager {
    pub fn new(grid_width: i32, grid_height: i32, cell_size: f32) -> Self {
        let grid = GridManager::new(grid_width, grid_height, cell_size);
        let pathfinder = AStar::new(grid.clone());
        
        // Default kitchen position
        let kitchen = Kitchen::new(GridPos::new(grid_width - 3, 1));
        
        Self {
            grid,
            pathfinder,
            tables: HashMap::new(),
            podium: None,
            kitchen,
            next_table_id: 1,
            time_multiplier: 1.0,
            bg_state: BackgroundState::Normal,
            snacks_enabled: false,
        }
    }

    pub fn has_snacks(&self) -> bool {
        self.snacks_enabled
    }
    
    pub fn load_stage(&mut self, stage: Stage) -> Result<()> {
        // Load stage-specific layout
        match stage {
            Stage::Diner => self.load_diner_layout(),
            Stage::Chez => self.load_chez_layout(),
            Stage::Seafood => self.load_seafood_layout(),
            Stage::Tiki => self.load_tiki_layout(),
            Stage::Goddess => self.load_goddess_layout(),
        }
        Ok(())
    }
    
    fn load_diner_layout(&mut self) {
        // Exact Endless starting layout from conf_NOCHEAT.cst
        let tables: Vec<(GridPos, Vec<GridPos>)> = vec![
            (GridPos::new(5, 5), vec![GridPos::new(4, 5), GridPos::new(6, 5), GridPos::new(6, 6), GridPos::new(4, 6)]),
            (GridPos::new(13, 5), vec![GridPos::new(12, 5), GridPos::new(14, 5), GridPos::new(14, 6), GridPos::new(12, 6)]),
            (GridPos::new(5, 11), vec![GridPos::new(4, 11), GridPos::new(6, 11), GridPos::new(6, 12), GridPos::new(4, 12)]),
            (GridPos::new(13, 11), vec![GridPos::new(12, 11), GridPos::new(14, 11), GridPos::new(14, 12), GridPos::new(12, 12)]),
        ];
        for (pos, seats) in tables {
            self.add_table_with_seats(pos, &seats);
        }
        self.podium = None;
    }
    
    fn load_chez_layout(&mut self) {
        let tables: Vec<(GridPos, Vec<GridPos>)> = vec![
            (GridPos::new(5, 5), vec![GridPos::new(4, 5), GridPos::new(6, 5), GridPos::new(6, 6), GridPos::new(4, 6)]),
            (GridPos::new(13, 5), vec![GridPos::new(12, 5), GridPos::new(14, 5), GridPos::new(14, 6), GridPos::new(12, 6)]),
            (GridPos::new(5, 11), vec![GridPos::new(4, 11), GridPos::new(6, 11), GridPos::new(6, 12), GridPos::new(4, 12)]),
            (GridPos::new(13, 11), vec![GridPos::new(12, 11), GridPos::new(14, 11), GridPos::new(14, 12), GridPos::new(12, 12)]),
        ];
        for (pos, seats) in tables {
            self.add_table_with_seats(pos, &seats);
        }
        self.podium = None;
    }
    
    fn load_seafood_layout(&mut self) {
        let tables: Vec<(GridPos, Vec<GridPos>)> = vec![
            (GridPos::new(5, 5), vec![GridPos::new(4, 5), GridPos::new(6, 5), GridPos::new(6, 6), GridPos::new(4, 6)]),
            (GridPos::new(13, 5), vec![GridPos::new(12, 5), GridPos::new(14, 5), GridPos::new(14, 6), GridPos::new(12, 6)]),
            (GridPos::new(5, 11), vec![GridPos::new(4, 11), GridPos::new(6, 11), GridPos::new(6, 12), GridPos::new(4, 12)]),
            (GridPos::new(13, 11), vec![GridPos::new(12, 11), GridPos::new(14, 11), GridPos::new(14, 12), GridPos::new(12, 12)]),
        ];
        for (pos, seats) in tables {
            self.add_table_with_seats(pos, &seats);
        }
        self.podium = None;
    }
    
    fn load_tiki_layout(&mut self) {
        let tables: Vec<(GridPos, Vec<GridPos>)> = vec![
            (GridPos::new(5, 5), vec![GridPos::new(4, 5), GridPos::new(6, 5), GridPos::new(6, 6), GridPos::new(4, 6)]),
            (GridPos::new(13, 5), vec![GridPos::new(12, 5), GridPos::new(14, 5), GridPos::new(14, 6), GridPos::new(12, 6)]),
            (GridPos::new(5, 11), vec![GridPos::new(4, 11), GridPos::new(6, 11), GridPos::new(6, 12), GridPos::new(4, 12)]),
            (GridPos::new(13, 11), vec![GridPos::new(12, 11), GridPos::new(14, 11), GridPos::new(14, 12), GridPos::new(12, 12)]),
        ];
        for (pos, seats) in tables {
            self.add_table_with_seats(pos, &seats);
        }
        self.podium = None;
    }
    
    fn load_goddess_layout(&mut self) {
        let tables: Vec<(GridPos, Vec<GridPos>)> = vec![
            (GridPos::new(5, 5), vec![GridPos::new(4, 5), GridPos::new(6, 5), GridPos::new(6, 6), GridPos::new(4, 6)]),
            (GridPos::new(13, 5), vec![GridPos::new(12, 5), GridPos::new(14, 5), GridPos::new(14, 6), GridPos::new(12, 6)]),
            (GridPos::new(5, 11), vec![GridPos::new(4, 11), GridPos::new(6, 11), GridPos::new(6, 12), GridPos::new(4, 12)]),
            (GridPos::new(13, 11), vec![GridPos::new(12, 11), GridPos::new(14, 11), GridPos::new(14, 12), GridPos::new(12, 12)]),
        ];
        for (pos, seats) in tables {
            self.add_table_with_seats(pos, &seats);
        }
        self.podium = None;
    }
    
    pub fn mark_table_footprint(&mut self, pos: GridPos, capacity: u32) {
        let height_tiles = match capacity {
            6 => 3,
            4 => 2,
            _ => 1,
        };
        for dy in 0..height_tiles {
            let y = pos.y + dy;
            let left_chair = GridPos::new(pos.x, y);
            let center_table = GridPos::new(pos.x + 1, y);
            let right_chair = GridPos::new(pos.x + 2, y);
            if self.grid.in_bounds(left_chair) {
                self.grid.set_cell(left_chair, CellType::Chair);
            }
            if self.grid.in_bounds(center_table) {
                self.grid.set_cell(center_table, CellType::Table);
            }
            if self.grid.in_bounds(right_chair) {
                self.grid.set_cell(right_chair, CellType::Chair);
            }
        }
    }

    pub fn add_table_at(&mut self, pos: GridPos, capacity: u32) -> TableId {
        let id = TableId(self.next_table_id);
        self.next_table_id += 1;
        
        let table = Table::new(id, pos, capacity);
        self.mark_table_footprint(pos, capacity);
        
        self.tables.insert(id, table);
        id
    }

    pub fn add_table_with_seats(&mut self, pos: GridPos, seat_positions: &[GridPos]) -> TableId {
        let specs: Vec<(GridPos, String)> = seat_positions
            .iter()
            .map(|&p| (p, "none".to_string()))
            .collect();
        self.add_table_with_colored_seats(pos, &specs)
    }

    pub fn add_table_with_colored_seats(
        &mut self,
        pos: GridPos,
        seat_specs: &[(GridPos, String)],
    ) -> TableId {
        let id = TableId(self.next_table_id);
        self.next_table_id += 1;

        // Per room_manager.ls line 436-438 & grid_manager.ls line 144:
        // `placeObject(t, chairs[1].pos - [0, 1])` places `chairs[1].Tile` at `(c1.x + 1, c1.y - 1)`.
        let anchor_pos = if (self.grid.cell_size - 37.0).abs() < 0.5 {
            let c1 = seat_specs
                .first()
                .map(|(p, _)| *p)
                .unwrap_or(GridPos::new(pos.x - 1, pos.y));
            GridPos::new(c1.x + 1, c1.y - 1)
        } else {
            pos
        };

        let seats: Vec<Seat> = if seat_specs.is_empty() {
            Table::new(id, anchor_pos, 2).seats
        } else {
            seat_specs
                .iter()
                .map(|(grid_pos, color)| Seat {
                    grid_pos: *grid_pos,
                    occupied: false,
                    sitter_id: None,
                    color: if color.is_empty() {
                        "none".to_string()
                    } else {
                        color.clone()
                    },
                    bonus_multiplier: 1,
                })
                .collect()
        };
        let capacity = seats.len() as u32;
        self.mark_table_footprint(anchor_pos, capacity);
        self.tables.insert(
            id,
            Table {
                id,
                grid_pos: anchor_pos,
                capacity,
                seats,
                state: TableState::Clean,
                dirty_timer: None,
                group_id: None,
            },
        );
        id
    }
    
    pub fn reset_objects(&mut self) {
        self.tables.clear();
        self.podium = None;
        self.kitchen = Kitchen::new(GridPos::new(self.grid.width - 3, 1));
        self.next_table_id = 1;
        self.time_multiplier = 1.0;
        self.grid = GridManager::new(self.grid.width, self.grid.height, self.grid.cell_size);
        self.pathfinder = AStar::new(self.grid.clone());
    }
    
    pub fn find_path(&self, start: Vec2, goal: Vec2) -> Option<Vec<Vec2>> {
        let start_grid = self.grid.find_nearest_walkable(self.grid.world_to_grid(start), 4)?;
        let goal_grid = self.grid.find_nearest_walkable(self.grid.world_to_grid(goal), 4)?;
        
        AStar::new(self.grid.clone()).find_path(start_grid, goal_grid)
            .map(|path| path.into_iter().map(|p| self.grid.grid_to_world(p)).collect())
    }

    /// Route Flo to the table's exact `destTile` (`room_manager.ls` line 440:
    /// `destTile = grid.getTileAt(pos[1] + 2, pos[2] - 2)` = `GridPos::new(px + 1, py - 1)`)
    /// directly behind the top edge of the table so she never walks across or stands in front of a table.
    pub fn table_service_path(&self, start: Vec2, table_id: TableId) -> Option<Vec<Vec2>> {
        let table = self.tables.get(&table_id)?;
        let start_tile = self.grid.find_nearest_walkable(self.grid.world_to_grid(start), 6)?;
        let h = match table.capacity {
            6 => 3,
            4 => 2,
            _ => 1,
        };
        let px = table.grid_pos.x;
        let py = table.grid_pos.y;
        let astar = AStar::new(self.grid.clone());

        // Primary destTile from `room_manager.ls` line 440: directly above table center
        let primary_dest = GridPos::new(px + 1, py - 1);
        let path_opt = if self.grid.in_bounds(primary_dest) && self.grid.is_walkable(primary_dest) {
            astar.find_path(start_tile, primary_dest)
        } else {
            None
        }
        .or_else(|| {
            let fallbacks = [
                GridPos::new(px, py - 1),
                GridPos::new(px + 2, py - 1),
                GridPos::new(px + 1, py + h),
                GridPos::new(px, py + h),
                GridPos::new(px + 2, py + h),
                GridPos::new(px - 1, py),
                GridPos::new(px + 3, py),
            ];
            fallbacks
                .into_iter()
                .filter(|&p| self.grid.in_bounds(p) && self.grid.is_walkable(p))
                .filter_map(|p| astar.find_path(start_tile, p))
                .min_by_key(|p| p.len())
        });

        path_opt.map(|path| {
            let mut pts: Vec<Vec2> = path
                .into_iter()
                .skip(1)
                .map(|p| self.grid.grid_to_world(p))
                .collect();
            if pts.is_empty() {
                pts.push(self.grid.grid_to_world(start_tile));
            }
            pts
        })
    }

    /// Route Flo around tables to a counter/station `destTile` on Row 3 (`y = 211.0`).
    pub fn station_service_path(&self, start: Vec2, goal_world: Vec2) -> Option<Vec<Vec2>> {
        let start_tile = self.grid.find_nearest_walkable(self.grid.world_to_grid(start), 6)?;
        let goal_tile = self.grid.find_nearest_walkable(self.grid.world_to_grid(goal_world), 6)?;
        let astar = AStar::new(self.grid.clone());
        let path = astar.find_path(start_tile, goal_tile)?;
        let mut pts: Vec<Vec2> = path
            .into_iter()
            .skip(1)
            .map(|p| self.grid.grid_to_world(p))
            .collect();
        if pts.is_empty() {
            pts.push(self.grid.grid_to_world(goal_tile));
        }
        Some(pts)
    }

    /// Original waiter.setDest uses findPath(..., #nextto: 1): approach
    /// an occupied destination from a reachable neighboring floor tile.
    pub fn service_path(&self, start: Vec2, target: GridPos) -> Option<Vec<Vec2>> {
        let start_tile = self.grid.find_nearest_walkable(self.grid.world_to_grid(start), 4)?;
        let astar = AStar::new(self.grid.clone());
        target.neighbors().into_iter()
            .filter(|&p| self.grid.in_bounds(p) && self.grid.is_walkable(p))
            .filter_map(|p| astar.find_path(start_tile, p))
            .min_by_key(|p| p.len())
            .map(|path| {
                path.into_iter().skip(1).map(|p| self.grid.grid_to_world(p)).collect()
            })
    }
    
    pub fn set_time_multiplier(&mut self, multiplier: f32) {
        self.time_multiplier = multiplier.clamp(0.1, 5.0);
    }
    
    pub fn update_bg_state(&mut self) {
        // Update background visual state based on occupancy
        let occupied = self.tables.values().filter(|t| t.state == TableState::Occupied).count();
        let total = self.tables.len();
        
        self.bg_state = if total == 0 {
            BackgroundState::Normal
        } else if occupied as f32 / total as f32 > 0.8 {
            BackgroundState::Busy
        } else {
            BackgroundState::Normal
        };
    }
    
    pub fn add_group_to_table(&mut self, group_id: GroupId, table_id: TableId) -> bool {
        if let Some(table) = self.tables.get_mut(&table_id) {
            if table.available_seats() > 0 {
                table.group_id = Some(group_id);
                table.state = TableState::Occupied;
                return true;
            }
        }
        false
    }
    
    pub fn clear_table(&mut self, table_id: TableId) {
        if let Some(table) = self.tables.get_mut(&table_id) {
            table.group_id = None;
            table.state = TableState::Dirty;
            table.dirty_timer = Some(0.0);
            
            for seat in &mut table.seats {
                seat.occupied = false;
                seat.sitter_id = None;
            }
        }
    }
    
    pub fn get_available_table(&self, party_size: u32) -> Option<TableId> {
        self.tables.iter()
            .filter(|(_, t)| t.state == TableState::Clean && t.capacity >= party_size)
            .min_by_key(|(_, t)| t.capacity)
            .map(|(id, _)| *id)
    }
    
    pub fn update(&mut self, dt: f32) {
        // Update kitchen
        let current_time = 0.0; // Would come from game time
        let ready_orders = self.kitchen.update(current_time);
        
        for order in ready_orders {
            // Deliver food to table
            // Would notify sitter
        }
        
        // Update dirty tables
        for table in self.tables.values_mut() {
            if let Some(timer) = table.dirty_timer.as_mut() {
                *timer += dt;
                if *timer > 3.0 { // 3 seconds to clean
                    table.state = TableState::Clean;
                    table.dirty_timer = None;
                }
            }
        }
        
        self.update_bg_state();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_grid_pos() {
        let pos = GridPos::new(5, 10);
        let world = pos.to_world(32.0);
        assert_eq!(world.x, 5.0 * 32.0 + 16.0);
        
        let back = GridPos::from_world(world, 32.0);
        assert_eq!(back, pos);
    }
    
    #[test]
    fn test_astar() {
        let grid = GridManager::new(10, 10, 32.0);
        let astar = AStar::new(grid);
        
        let path = astar.find_path(GridPos::new(0, 0), GridPos::new(5, 5));
        assert!(path.is_some());
        assert_eq!(path.unwrap().len(), 11); // 0,0 to 5,5 = 10 steps + start
    }
    
    #[test]
    fn test_table() {
        let mut table = Table::new(TableId(1), GridPos::new(5, 5), 4);
        assert_eq!(table.capacity, 4);
        assert_eq!(table.available_seats(), 4);
        
        let sitter = SitterId(1);
        assert!(table.occupy_seat(0, sitter));
        assert_eq!(table.available_seats(), 3);
        assert!(table.seats[0].occupied);
    }
}
