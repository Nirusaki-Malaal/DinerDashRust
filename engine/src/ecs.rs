//! Entity Component System (ECS) for the game engine

use serde::{Deserialize, Serialize};
use std::any::{Any, TypeId};
use std::collections::HashMap;

/// Unique entity identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Entity(pub u64);

impl Entity {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Component trait - all components must implement this
pub trait Component: 'static + Send + Sync {}

/// System trait - all systems must implement this
pub trait System: Send + Sync {
    fn name(&self) -> &str;
    fn run(&mut self, world: &mut World, dt: f32);
    fn priority(&self) -> i32 {
        0
    }
}

/// Main ECS World
pub struct World {
    entities: Vec<Entity>,
    components: HashMap<TypeId, HashMap<Entity, Box<dyn Any + Send + Sync>>>,
    component_masks: HashMap<Entity, Vec<TypeId>>,
    systems: Vec<Box<dyn System>>,
    next_entity_id: u64,
    resources: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            components: HashMap::new(),
            component_masks: HashMap::new(),
            systems: Vec::new(),
            next_entity_id: 1,
            resources: HashMap::new(),
        }
    }

    /// Create a new entity
    pub fn spawn(&mut self) -> Entity {
        let entity = Entity(self.next_entity_id);
        self.next_entity_id += 1;
        self.entities.push(entity);
        self.component_masks.insert(entity, Vec::new());
        entity
    }

    /// Remove an entity and all its components
    pub fn despawn(&mut self, entity: Entity) {
        if let Some(mask) = self.component_masks.remove(&entity) {
            for type_id in mask {
                if let Some(map) = self.components.get_mut(&type_id) {
                    map.remove(&entity);
                }
            }
        }
        self.entities.retain(|e| *e != entity);
    }

    /// Add a component to an entity
    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) -> &mut T {
        let type_id = TypeId::of::<T>();
        let map = self.components.entry(type_id).or_default();
        map.insert(entity, Box::new(component));

        let mask = self.component_masks.entry(entity).or_default();
        if !mask.contains(&type_id) {
            mask.push(type_id);
        }

        map.get_mut(&entity).unwrap().downcast_mut::<T>().unwrap()
    }

    /// Get a component reference
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.components
            .get(&type_id)?
            .get(&entity)?
            .downcast_ref::<T>()
    }

    /// Get a mutable component reference
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.components
            .get_mut(&type_id)?
            .get_mut(&entity)?
            .downcast_mut::<T>()
    }

    /// Remove a component from an entity
    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> Option<T> {
        let type_id = TypeId::of::<T>();
        let comp = self.components.get_mut(&type_id)?.remove(&entity)?;

        if let Some(mask) = self.component_masks.get_mut(&entity) {
            mask.retain(|id| *id != type_id);
        }

        comp.downcast::<T>().ok().map(|b| *b)
    }

    /// Check if entity has component
    pub fn has_component<T: Component>(&self, entity: Entity) -> bool {
        let type_id = TypeId::of::<T>();
        self.component_masks
            .get(&entity)
            .map(|mask| mask.contains(&type_id))
            .unwrap_or(false)
    }

    /// Get all entities with a specific component
    pub fn entities_with_component<T: Component>(&self) -> Vec<Entity> {
        let type_id = TypeId::of::<T>();
        self.component_masks
            .iter()
            .filter(|(_, mask)| mask.contains(&type_id))
            .map(|(entity, _)| *entity)
            .collect()
    }

    /// Query all `(Entity, &T)` pairs for component `T`
    pub fn query<T: Component>(&self) -> Vec<(Entity, &T)> {
        let type_id = TypeId::of::<T>();
        let Some(map) = self.components.get(&type_id) else {
            return Vec::new();
        };
        map.iter()
            .filter_map(|(entity, comp)| Some((*entity, comp.downcast_ref::<T>()?)))
            .collect()
    }

    /// Get all entities
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    /// Register a system
    pub fn add_system(&mut self, system: Box<dyn System>) {
        self.systems.push(system);
        self.systems.sort_by_key(|s| -s.priority());
    }

    /// Run all systems
    pub fn run_systems(&mut self, dt: f32) {
        let mut systems = std::mem::take(&mut self.systems);
        for system in &mut systems {
            system.run(self, dt);
        }
        self.systems = systems;
    }

    /// Add a resource (singleton)
    pub fn add_resource<T: 'static + Send + Sync>(&mut self, resource: T) {
        self.resources.insert(TypeId::of::<T>(), Box::new(resource));
    }

    /// Get a resource
    pub fn get_resource<T: 'static + Send + Sync>(&self) -> Option<&T> {
        self.resources.get(&TypeId::of::<T>())?.downcast_ref::<T>()
    }

    /// Get a mutable resource
    pub fn get_resource_mut<T: 'static + Send + Sync>(&mut self) -> Option<&mut T> {
        self.resources
            .get_mut(&TypeId::of::<T>())?
            .downcast_mut::<T>()
    }
}

/// Common components

#[derive(Debug, Clone)]
pub struct Transform {
    pub position: crate::math::Vec2,
    pub rotation: f32,
    pub scale: crate::math::Vec2,
}

impl Component for Transform {}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: crate::math::Vec2::ZERO,
            rotation: 0.0,
            scale: crate::math::Vec2::ONE,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Sprite {
    pub texture_region: crate::assets::TextureRegion,
    pub color: crate::math::Color,
    pub visible: bool,
    pub z_index: i32,
}

impl Component for Sprite {}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            texture_region: crate::assets::TextureRegion::new(
                crate::assets::TextureId(0),
                crate::math::Vec2::ZERO,
                crate::math::Vec2::ONE,
                crate::math::UVec2::ZERO,
            ),
            color: crate::math::Color::ONE,
            visible: true,
            z_index: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Animation {
    pub current_animation: String,
    pub current_frame: usize,
    pub frame_timer: f32,
    pub playing: bool,
    pub speed: f32,
    pub loop_mode: crate::assets::LoopMode,
}

impl Component for Animation {}

impl Default for Animation {
    fn default() -> Self {
        Self {
            current_animation: String::new(),
            current_frame: 0,
            frame_timer: 0.0,
            playing: false,
            speed: 1.0,
            loop_mode: crate::assets::LoopMode::Loop,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Velocity {
    pub linear: crate::math::Vec2,
    pub angular: f32,
}

impl Component for Velocity {}

impl Default for Velocity {
    fn default() -> Self {
        Self {
            linear: crate::math::Vec2::ZERO,
            angular: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Collider {
    pub bounds: crate::math::Rect,
    pub solid: bool,
    pub trigger: bool,
}

impl Component for Collider {}

impl Collider {
    pub fn new(bounds: crate::math::Rect) -> Self {
        Self {
            bounds,
            solid: true,
            trigger: false,
        }
    }

    pub fn trigger(bounds: crate::math::Rect) -> Self {
        Self {
            bounds,
            solid: false,
            trigger: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Name {
    pub value: String,
}

impl Component for Name {}

#[derive(Debug, Clone)]
pub struct Parent {
    pub entity: Entity,
}

impl Component for Parent {}

#[derive(Debug, Clone, Default)]
pub struct Children {
    pub entities: Vec<Entity>,
}

impl Component for Children {}

#[derive(Debug, Clone, Default)]
pub struct StateMachine {
    pub current_state: String,
    pub previous_state: String,
    pub state_timer: f32,
    pub data: HashMap<String, f32>,
}

impl Component for StateMachine {}

impl StateMachine {
    pub fn transition(&mut self, new_state: String) {
        self.previous_state = self.current_state.clone();
        self.current_state = new_state;
        self.state_timer = 0.0;
    }

    pub fn set_data(&mut self, key: String, value: f32) {
        self.data.insert(key, value);
    }

    pub fn get_data(&self, key: &str) -> Option<f32> {
        self.data.get(key).copied()
    }
}

/// Timer component for delayed actions
pub struct EcsTimer {
    pub duration: f32,
    pub elapsed: f32,
    pub repeating: bool,
    pub callback: Option<Box<dyn FnOnce(&mut World, Entity) + Send + Sync>>,
}

impl std::fmt::Debug for EcsTimer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EcsTimer")
            .field("duration", &self.duration)
            .field("elapsed", &self.elapsed)
            .field("repeating", &self.repeating)
            .finish()
    }
}

impl Component for EcsTimer {}

impl EcsTimer {
    pub fn new(duration: f32) -> Self {
        Self {
            duration,
            elapsed: 0.0,
            repeating: false,
            callback: None,
        }
    }

    pub fn repeating(duration: f32) -> Self {
        Self {
            duration,
            elapsed: 0.0,
            repeating: true,
            callback: None,
        }
    }

    pub fn with_callback<F>(mut self, callback: F) -> Self
    where
        F: FnOnce(&mut World, Entity) + Send + Sync + 'static,
    {
        self.callback = Some(Box::new(callback));
        self
    }
}

/// System for updating entity timers
pub struct EcsTimerSystem;

impl System for EcsTimerSystem {
    fn name(&self) -> &str {
        "EcsTimerSystem"
    }

    fn run(&mut self, world: &mut World, dt: f32) {
        let mut to_fire = Vec::new();

        for entity in world.entities_with_component::<EcsTimer>() {
            if let Some(timer) = world.get_component_mut::<EcsTimer>(entity) {
                timer.elapsed += dt;
                if timer.elapsed >= timer.duration {
                    if timer.repeating {
                        timer.elapsed = 0.0;
                    }
                    if let Some(callback) = timer.callback.take() {
                        to_fire.push((entity, callback));
                    }
                }
            }
        }

        for (entity, callback) in to_fire {
            callback(world, entity);
        }
    }
}

/// System for updating animations
pub struct AnimationSystem;

impl System for AnimationSystem {
    fn name(&self) -> &str {
        "AnimationSystem"
    }

    fn run(&mut self, world: &mut World, dt: f32) {
        for entity in world.entities_with_component::<Animation>() {
            if let Some(animation) = world.get_component_mut::<Animation>(entity) {
                if !animation.playing || animation.current_animation.is_empty() {
                    continue;
                }

                animation.frame_timer += dt * animation.speed;

                if animation.frame_timer >= 0.1 {
                    animation.frame_timer = 0.0;
                    animation.current_frame += 1;
                }
            }
        }
    }
}

/// System for updating transforms from velocity
pub struct PhysicsSystem;

impl System for PhysicsSystem {
    fn name(&self) -> &str {
        "PhysicsSystem"
    }
    fn priority(&self) -> i32 {
        100
    }

    fn run(&mut self, world: &mut World, dt: f32) {
        let entities = world.entities_with_component::<Transform>();
        for entity in entities {
            let velocity = world.get_component::<Velocity>(entity).cloned();
            if let (Some(transform), Some(vel)) =
                (world.get_component_mut::<Transform>(entity), velocity)
            {
                transform.position += vel.linear * dt;
                transform.rotation += vel.angular * dt;
            }
        }
    }
}

/// System for parent-child transform hierarchy
pub struct HierarchySystem;

impl System for HierarchySystem {
    fn name(&self) -> &str {
        "HierarchySystem"
    }
    fn priority(&self) -> i32 {
        90
    }

    fn run(&mut self, _world: &mut World, _dt: f32) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecs_basic() {
        let mut world = World::new();

        let entity = world.spawn();
        world.add_component(entity, Transform::default());
        world.add_component(entity, Velocity::default());

        assert!(world.has_component::<Transform>(entity));
        assert!(world.has_component::<Velocity>(entity));

        let transform = world.get_component::<Transform>(entity).unwrap();
        assert_eq!(transform.position, crate::math::Vec2::ZERO);
    }

    #[test]
    fn test_query() {
        let mut world = World::new();

        for _ in 0..5 {
            let e = world.spawn();
            world.add_component(e, Transform::default());
            world.add_component(e, Velocity::default());
        }

        let count = world.entities_with_component::<Transform>().len();
        assert_eq!(count, 5);
        assert_eq!(world.query::<Transform>().len(), 5);
    }
}