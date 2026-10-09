//! 2D Scene Graph for the game engine

use anyhow::Result;
use std::collections::HashMap;
use crate::math::{Vec2, Mat4, Rect};
use crate::ecs::{Entity, Component, World, System};

/// Unique node identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct NodeId(pub u64);

impl NodeId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// 2D Scene Graph Node
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Node2D {
    pub id: NodeId,
    pub name: String,
    pub transform: crate::math::Transform2D,
    pub children: Vec<NodeId>,
    pub parent: Option<NodeId>,
    pub visible: bool,
    pub z_index: i32,
    pub entity: Option<Entity>, // Optional ECS entity link
}

impl Default for Node2D {
    fn default() -> Self {
        Self {
            id: NodeId(0),
            name: String::new(),
            transform: Default::default(),
            children: Vec::new(),
            parent: None,
            visible: true,
            z_index: 0,
            entity: None,
        }
    }
}

impl Node2D {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }
    
    pub fn with_transform(mut self, transform: crate::math::Transform2D) -> Self {
        self.transform = transform;
        self
    }
    
    pub fn with_entity(mut self, entity: Entity) -> Self {
        self.entity = Some(entity);
        self
    }
}

/// Scene Graph - manages hierarchy of 2D nodes
pub struct SceneGraph {
    nodes: HashMap<NodeId, Node2D>,
    root: NodeId,
    next_id: u64,
    // Dirty flags for transform updates
    dirty_nodes: Vec<NodeId>,
    world_transforms: HashMap<NodeId, Mat4>,
}

impl Default for SceneGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl SceneGraph {
    pub fn new() -> Self {
        let root_id = NodeId(0);
        let mut nodes = HashMap::new();
        nodes.insert(root_id, Node2D {
            id: root_id,
            name: "Root".to_string(),
            transform: Default::default(),
            children: Vec::new(),
            parent: None,
            visible: true,
            z_index: 0,
            entity: None,
        });
        
        Self {
            nodes,
            root: root_id,
            next_id: 1,
            dirty_nodes: Vec::new(),
            world_transforms: HashMap::new(),
        }
    }
    
    /// Create a new node and add to parent
    pub fn add_node(&mut self, name: impl Into<String>, parent: Option<NodeId>) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;
        
        let parent_id = parent.unwrap_or(self.root);
        
        let mut node = Node2D::new(name);
        node.id = id;
        node.parent = Some(parent_id);
        
        // Add to parent's children
        if let Some(parent_node) = self.nodes.get_mut(&parent_id) {
            parent_node.children.push(id);
        }
        
        self.nodes.insert(id, node);
        self.mark_dirty(id);
        id
    }
    
    /// Remove a node and all its children
    pub fn remove_node(&mut self, id: NodeId) {
        // Remove from parent's children
        if let Some(node) = self.nodes.get(&id) {
            if let Some(parent_id) = node.parent {
                if let Some(parent) = self.nodes.get_mut(&parent_id) {
                    parent.children.retain(|c| *c != id);
                }
            }
        }
        
        // Recursively remove children
        let children: Vec<NodeId> = self.get_descendants(id);
        for child in children {
            self.nodes.remove(&child);
            self.world_transforms.remove(&child);
        }
        
        self.nodes.remove(&id);
        self.world_transforms.remove(&id);
    }
    
    /// Get a node by ID
    pub fn get(&self, id: NodeId) -> Option<&Node2D> {
        self.nodes.get(&id)
    }
    
    /// Get mutable node by ID
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Node2D> {
        if self.nodes.contains_key(&id) {
            self.mark_dirty(id);
            self.nodes.get_mut(&id)
        } else {
            None
        }
    }
    
    /// Set node transform
    pub fn set_transform(&mut self, id: NodeId, transform: crate::math::Transform2D) -> Result<()> {
        if let Some(node) = self.nodes.get_mut(&id) {
            node.transform = transform;
            self.mark_dirty(id);
            Ok(())
        } else {
            anyhow::bail!("Node not found: {:?}", id)
        }
    }
    
    /// Translate node
    pub fn translate(&mut self, id: NodeId, delta: Vec2) -> Result<()> {
        if let Some(node) = self.nodes.get_mut(&id) {
            node.transform.translate(delta);
            self.mark_dirty(id);
            Ok(())
        } else {
            anyhow::bail!("Node not found: {:?}", id)
        }
    }
    
    /// Get world transform matrix for a node
    pub fn get_world_transform(&mut self, id: NodeId) -> Mat4 {
        if let Some(transform) = self.world_transforms.get(&id) {
            return *transform;
        }
        
        let transform = self.compute_world_transform(id);
        self.world_transforms.insert(id, transform);
        transform
    }
    
    fn compute_world_transform(&self, id: NodeId) -> Mat4 {
        let node = &self.nodes[&id];
        let local = node.transform.matrix();
        
        if let Some(parent_id) = node.parent {
            let parent_transform = self.compute_world_transform(parent_id);
            parent_transform * local
        } else {
            local
        }
    }
    
    /// Mark node and all descendants as dirty
    fn mark_dirty(&mut self, id: NodeId) {
        if !self.dirty_nodes.contains(&id) {
            self.dirty_nodes.push(id);
        }
        let children = self.nodes.get(&id).map(|n| n.children.clone()).unwrap_or_default();
        for child in children {
            self.mark_dirty(child);
        }
    }
    
    /// Update all dirty world transforms
    pub fn update_transforms(&mut self) {
        let dirty: Vec<NodeId> = self.dirty_nodes.drain(..).collect();
        for id in dirty {
            if self.nodes.contains_key(&id) {
                let transform = self.compute_world_transform(id);
                self.world_transforms.insert(id, transform);
            }
        }
    }
    
    /// Get all descendants of a node
    pub fn get_descendants(&self, id: NodeId) -> Vec<NodeId> {
        let mut result = Vec::new();
        if let Some(node) = self.nodes.get(&id) {
            for child in &node.children {
                result.push(*child);
                result.extend(self.get_descendants(*child));
            }
        }
        result
    }
    
    /// Get children of a node
    pub fn get_children(&self, id: NodeId) -> Vec<NodeId> {
        self.nodes.get(&id).map(|n| n.children.clone()).unwrap_or_default()
    }
    
    /// Find node by name (breadth-first search)
    pub fn find_by_name(&self, name: &str) -> Option<NodeId> {
        let mut queue = vec![self.root];
        
        while let Some(id) = queue.pop() {
            if let Some(node) = self.nodes.get(&id) {
                if node.name == name {
                    return Some(id);
                }
                queue.extend(&node.children);
            }
        }
        None
    }
    
    /// Get all nodes sorted by Z-index for rendering
    pub fn get_render_order(&mut self) -> Vec<NodeId> {
        self.update_transforms();
        
        let mut nodes: Vec<_> = self.nodes.values()
            .filter(|n| n.visible && n.id != self.root)
            .collect();
        
        nodes.sort_by_key(|n| n.z_index);
        nodes.into_iter().map(|n| n.id).collect()
    }
    
    /// Get node count
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
    
    /// Set node visibility
    pub fn set_visible(&mut self, id: NodeId, visible: bool) -> Result<()> {
        if let Some(node) = self.nodes.get_mut(&id) {
            node.visible = visible;
            Ok(())
        } else {
            anyhow::bail!("Node not found: {:?}", id)
        }
    }
    
    /// Set Z-index
    pub fn set_z_index(&mut self, id: NodeId, z: i32) -> Result<()> {
        if let Some(node) = self.nodes.get_mut(&id) {
            node.z_index = z;
            Ok(())
        } else {
            anyhow::bail!("Node not found: {:?}", id)
        }
    }
    
    /// Move node to front (highest Z among siblings)
    pub fn move_to_front(&mut self, id: NodeId) -> Result<()> {
        if let Some(node) = self.nodes.get(&id) {
            if let Some(parent_id) = node.parent {
                if let Some(parent) = self.nodes.get(&parent_id) {
                    let max_z = parent.children.iter()
                        .filter_map(|c| self.nodes.get(c))
                        .map(|n| n.z_index)
                        .max()
                        .unwrap_or(0);
                    self.set_z_index(id, max_z + 1)?;
                }
            }
        }
        Ok(())
    }
    
    /// Traverse tree with a function
    pub fn traverse<F>(&self, mut f: F) where F: FnMut(&Node2D) {
        fn traverse_recursive(nodes: &HashMap<NodeId, Node2D>, id: NodeId, f: &mut dyn FnMut(&Node2D)) {
            if let Some(node) = nodes.get(&id) {
                f(node);
                for child in &node.children {
                    traverse_recursive(nodes, *child, f);
                }
            }
        }
        
        traverse_recursive(&self.nodes, self.root, &mut f);
    }
}

/// Scene - combines SceneGraph with ECS World
pub struct Scene {
    pub graph: SceneGraph,
    pub world: World,
    pub name: String,
}

impl Scene {
    pub fn new(name: impl Into<String>) -> Self {
        let mut world = World::new();
        world.add_system(Box::new(crate::ecs::PhysicsSystem));
        Self {
            graph: SceneGraph::new(),
            world,
            name: name.into(),
        }
    }
    
    /// Create a node with an associated ECS entity
    pub fn create_entity_node(&mut self, name: impl Into<String>, parent: Option<NodeId>) -> (NodeId, Entity) {
        let entity = self.world.spawn();
        let node_id = self.graph.add_node(name, parent);
        
        // Link node to entity
        if let Some(node) = self.graph.get_mut(node_id) {
            node.entity = Some(entity);
        }
        
        // Add Transform component matching node transform
        self.world.add_component(entity, crate::ecs::Transform::default());
        
        (node_id, entity)
    }
    
    /// Update - run systems and update transforms
    pub fn update(&mut self, dt: f32) {
        // Run ECS systems
        self.world.run_systems(dt);
        
        // Sync ECS transforms to scene graph
        let ecs_transforms: Vec<(Entity, crate::math::Transform2D)> = self
            .world
            .query::<crate::ecs::Transform>()
            .into_iter()
            .map(|(entity, transform)| {
                (
                    entity,
                    crate::math::Transform2D {
                        position: transform.position,
                        rotation: transform.rotation,
                        scale: transform.scale,
                    },
                )
            })
            .collect();

        for (entity, node_transform) in ecs_transforms {
            if let Some(node_id) = self.find_node_for_entity(entity) {
                let _ = self.graph.set_transform(node_id, node_transform);
            }
        }
        
        // Update scene graph transforms
        self.graph.update_transforms();
    }
    
    /// Find node ID for an entity
    fn find_node_for_entity(&self, entity: Entity) -> Option<NodeId> {
        for (id, node) in &self.graph.nodes {
            if node.entity == Some(entity) {
                return Some(*id);
            }
        }
        None
    }
    
    /// Get renderable nodes with their world transforms
    pub fn get_renderables(&mut self) -> Vec<(NodeId, Mat4, &Node2D)> {
        let order = self.graph.get_render_order();
        for &id in &order {
            let _ = self.graph.get_world_transform(id);
        }
        let graph = &self.graph;
        order
            .into_iter()
            .filter_map(|id| {
                let node = graph.get(id)?;
                let transform = graph.world_transforms.get(&id).copied().unwrap_or(Mat4::IDENTITY);
                Some((id, transform, node))
            })
            .collect()
    }
}

/// Scene manager for handling multiple scenes
pub struct SceneManager {
    scenes: HashMap<String, Scene>,
    current: Option<String>,
    next_scene: Option<String>,
}

impl SceneManager {
    pub fn new() -> Self {
        Self {
            scenes: HashMap::new(),
            current: None,
            next_scene: None,
        }
    }
    
    pub fn add_scene(&mut self, scene: Scene) {
        let name = scene.name.clone();
        let is_first = self.scenes.is_empty();
        self.scenes.insert(name.clone(), scene);
        if is_first {
            self.current = Some(name);
        }
    }
    
    pub fn switch_to(&mut self, name: &str) -> Result<()> {
        if self.scenes.contains_key(name) {
            self.next_scene = Some(name.to_string());
            Ok(())
        } else {
            anyhow::bail!("Scene not found: {}", name)
        }
    }
    
    pub fn update(&mut self, dt: f32) {
        // Handle scene transition
        if let Some(next) = self.next_scene.take() {
            self.current = Some(next);
        }
        
        if let Some(current_name) = &self.current {
            if let Some(scene) = self.scenes.get_mut(current_name) {
                scene.update(dt);
            }
        }
    }
    
    pub fn current_scene(&self) -> Option<&Scene> {
        self.current.as_ref().and_then(|name| self.scenes.get(name))
    }
    
    pub fn current_scene_mut(&mut self) -> Option<&mut Scene> {
        self.current.as_ref().and_then(|name| self.scenes.get_mut(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_scene_graph_basic() {
        let mut graph = SceneGraph::new();
        
        let child1 = graph.add_node("Child1", None);
        let child2 = graph.add_node("Child2", Some(child1));
        
        assert_eq!(graph.node_count(), 3); // Root + 2 children
        
        let children = graph.get_children(child1);
        assert_eq!(children.len(), 1);
        assert_eq!(children[0], child2);
    }
    
    #[test]
    fn test_world_transform() {
        let mut graph = SceneGraph::new();
        
        let parent = graph.add_node("Parent", None);
        let child = graph.add_node("Child", Some(parent));
        
        graph.set_transform(parent, crate::math::Transform2D::new(Vec2::new(100.0, 50.0))).unwrap();
        graph.set_transform(child, crate::math::Transform2D::new(Vec2::new(10.0, 20.0))).unwrap();
        
        let parent_transform = graph.get_world_transform(parent);
        let child_transform = graph.get_world_transform(child);
        
        // Child world position should be parent + child local
        let child_pos = child_transform.transform_point3(Vec2::ZERO.extend(0.0)).truncate();
        assert!((child_pos.x - 110.0).abs() < 0.01);
        assert!((child_pos.y - 70.0).abs() < 0.01);
    }
    
    #[test]
    fn test_scene_with_ecs() {
        let mut scene = Scene::new("Test");
        
        let (node_id, entity) = scene.create_entity_node("Player", None);
        
        // Add velocity
        scene.world.add_component(entity, crate::ecs::Velocity {
            linear: Vec2::new(50.0, 0.0),
            angular: 0.0,
        });
        
        // Update
        scene.update(1.0); // 1 second
        
        // Check position synced
        let transform = scene.world.get_component::<crate::ecs::Transform>(entity).unwrap();
        assert!((transform.position.x - 50.0).abs() < 0.01);
    }
}