//! Tutorial System
//! 
//! Implements the 24-step tutorial from the original game

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::math::{Vec2, Rect, Color, easing};
use crate::ecs::{Entity, Component, World, System};
use crate::assets::{AssetManager, TextureRegion};
use crate::game::GameMode;

/// Tutorial step types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TutorialStepType {
    /// Show text and wait for click
    Text,
    /// Highlight an area and wait for click
    HighlightClick,
    /// Force player to perform an action
    ForcedAction,
    /// Wait for a game event
    WaitForEvent,
    /// Show text with auto-advance
    AutoAdvance,
}

/// Wait condition for tutorial steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WaitCondition {
    Click { rect: Option<Rect> },
    Event { name: String },
    Time { seconds: f32 },
    Action { action_type: String, target: String },
    AnyClick,
}

/// Tutorial step definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorialStep {
    pub id: u32,
    pub step_type: TutorialStepType,
    pub text: String,
    pub highlight_rect: Option<Rect>,
    pub wait_condition: WaitCondition,
    pub on_complete: Option<String>, // Callback event
    pub forced_action: Option<ForcedAction>,
    pub skippable: bool,
    pub auto_advance_delay: Option<f32>,
}

/// Forced action definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForcedAction {
    pub action_type: String, // "click", "drag", "seat_customer", etc.
    pub target: String,      // Entity ID or UI element ID
    pub hint_text: String,
    pub highlight_rect: Option<Rect>,
}

/// Tutorial manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorialManager {
    pub steps: Vec<TutorialStep>,
    pub current_step: usize,
    pub active: bool,
    pub completed_steps: HashMap<u32, bool>,
    pub forced_action_active: bool,
    pub highlight_entity: Option<Entity>,
    pub overlay_alpha: f32,
    pub text_box: TutorialTextBox,
    pub arrow: TutorialArrow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorialTextBox {
    pub element: crate::ui::UIElement,
    pub visible: bool,
    pub animation_timer: f32,
    pub target_alpha: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorialArrow {
    pub texture_region: TextureRegion,
    pub position: Vec2,
    pub target_position: Vec2,
    pub visible: bool,
    pub bounce_timer: f32,
    pub bounce_offset: f32,
}

impl TutorialManager {
    pub fn new() -> Self {
        Self {
            steps: Self::default_steps(),
            current_step: 0,
            active: false,
            completed_steps: HashMap::new(),
            forced_action_active: false,
            highlight_entity: None,
            overlay_alpha: 0.0,
            text_box: TutorialTextBox {
                element: crate::ui::UIElement::new("tutorial_text")
                    .with_anchor(crate::ui::Anchor::BottomCenter)
                    .with_pos(Vec2::new(0.0, -100.0))
                    .with_size(Vec2::new(600.0, 150.0)),
                visible: false,
                animation_timer: 0.0,
                target_alpha: 0.0,
            },
            arrow: TutorialArrow {
                texture_region: TextureRegion::new(
                    crate::assets::TextureId(0),
                    Vec2::ZERO,
                    Vec2::ONE,
                    crate::math::UVec2::ZERO,
                ),
                position: Vec2::ZERO,
                target_position: Vec2::ZERO,
                visible: false,
                bounce_timer: 0.0,
                bounce_offset: 0.0,
            },
        }
    }
    
    /// Default 24 tutorial steps from original game
    fn default_steps() -> Vec<TutorialStep> {
        vec![
            // Step 1: Welcome
            TutorialStep {
                id: 1,
                step_type: TutorialStepType::Text,
                text: "Welcome to Diner Dash! Click anywhere to continue.".to_string(),
                highlight_rect: None,
                wait_condition: WaitCondition::AnyClick,
                on_complete: None,
                forced_action: None,
                skippable: true,
                auto_advance_delay: None,
            },
            // Step 2: Seat customer
            TutorialStep {
                id: 2,
                step_type: TutorialStepType::ForcedAction,
                text: "Customers arrive at the podium. Drag them to a table to seat them.".to_string(),
                highlight_rect: Some(Rect::new(Vec2::new(400.0, 100.0), Vec2::new(600.0, 200.0))), // Podium area
                wait_condition: WaitCondition::Action {
                    action_type: "drag".to_string(),
                    target: "customer_group".to_string(),
                },
                on_complete: Some("customer_seated".to_string()),
                forced_action: Some(ForcedAction {
                    action_type: "drag".to_string(),
                    target: "customer_group".to_string(),
                    hint_text: "Drag the group to a table".to_string(),
                    highlight_rect: Some(Rect::new(Vec2::new(400.0, 100.0), Vec2::new(600.0, 200.0))),
                }),
                skippable: false,
                auto_advance_delay: None,
            },
            // Step 3: Take order
            TutorialStep {
                id: 3,
                step_type: TutorialStepType::ForcedAction,
                text: "When customers are ready, a bubble appears. Click them to take their order.".to_string(),
                highlight_rect: None,
                wait_condition: WaitCondition::Action {
                    action_type: "click".to_string(),
                    target: "ready_customer".to_string(),
                },
                on_complete: Some("order_taken".to_string()),
                forced_action: Some(ForcedAction {
                    action_type: "click".to_string(),
                    target: "ready_customer".to_string(),
                    hint_text: "Click the customer to take their order".to_string(),
                    highlight_rect: None,
                }),
                skippable: false,
                auto_advance_delay: None,
            },
            // Step 4: Deliver food
            TutorialStep {
                id: 4,
                step_type: TutorialStepType::ForcedAction,
                text: "Food appears at the kitchen window. Click it, then click the customer to deliver.".to_string(),
                highlight_rect: Some(Rect::new(Vec2::new(700.0, 400.0), Vec2::new(800.0, 500.0))), // Kitchen
                wait_condition: WaitCondition::Action {
                    action_type: "click".to_string(),
                    target: "food".to_string(),
                },
                on_complete: Some("food_picked_up".to_string()),
                forced_action: Some(ForcedAction {
                    action_type: "click".to_string(),
                    target: "food".to_string(),
                    hint_text: "Click the food at the kitchen".to_string(),
                    highlight_rect: Some(Rect::new(Vec2::new(700.0, 400.0), Vec2::new(800.0, 500.0))),
                }),
                skippable: false,
                auto_advance_delay: None,
            },
            // Step 5: Deliver to table
            TutorialStep {
                id: 5,
                step_type: TutorialStepType::ForcedAction,
                text: "Now click the customer at their table to deliver the food.".to_string(),
                highlight_rect: None,
                wait_condition: WaitCondition::Action {
                    action_type: "click".to_string(),
                    target: "customer_at_table".to_string(),
                },
                on_complete: Some("food_delivered".to_string()),
                forced_action: Some(ForcedAction {
                    action_type: "click".to_string(),
                    target: "customer_at_table".to_string(),
                    hint_text: "Click the customer to deliver food".to_string(),
                    highlight_rect: None,
                }),
                skippable: false,
                auto_advance_delay: None,
            },
            // Step 6: Check and pay
            TutorialStep {
                id: 6,
                step_type: TutorialStepType::ForcedAction,
                text: "When customers finish eating, they want the check. Click them to deliver it.".to_string(),
                highlight_rect: None,
                wait_condition: WaitCondition::Action {
                    action_type: "click".to_string(),
                    target: "check_customer".to_string(),
                },
                on_complete: Some("check_delivered".to_string()),
                forced_action: Some(ForcedAction {
                    action_type: "click".to_string(),
                    target: "check_customer".to_string(),
                    hint_text: "Click to deliver the check".to_string(),
                    highlight_rect: None,
                }),
                skippable: false,
                auto_advance_delay: None,
            },
            // Step 7: Clean table
            TutorialStep {
                id: 7,
                step_type: TutorialStepType::ForcedAction,
                text: "After they leave, click the dirty table to clean it for the next customers.".to_string(),
                highlight_rect: None,
                wait_condition: WaitCondition::Action {
                    action_type: "click".to_string(),
                    target: "dirty_table".to_string(),
                },
                on_complete: Some("table_cleaned".to_string()),
                forced_action: Some(ForcedAction {
                    action_type: "click".to_string(),
                    target: "dirty_table".to_string(),
                    hint_text: "Click the dirty table to clean it".to_string(),
                    highlight_rect: None,
                }),
                skippable: false,
                auto_advance_delay: None,
            },
            // Step 8: Goal and tips
            TutorialStep {
                id: 8,
                step_type: TutorialStepType::Text,
                text: "Earn tips to reach the goal! The money jar shows your progress. Expert goal earns bonus!".to_string(),
                highlight_rect: Some(Rect::new(Vec2::new(50.0, 50.0), Vec2::new(300.0, 150.0))), // Money jar area
                wait_condition: WaitCondition::AnyClick,
                on_complete: None,
                forced_action: None,
                skippable: true,
                auto_advance_delay: None,
            },
            // Step 9: Star meter
            TutorialStep {
                id: 9,
                step_type: TutorialStepType::Text,
                text: "Angry customers cost you stars. Lose all stars and it's game over!".to_string(),
                highlight_rect: Some(Rect::new(Vec2::new(600.0, 50.0), Vec2::new(750.0, 150.0))), // Star meter
                wait_condition: WaitCondition::AnyClick,
                on_complete: None,
                forced_action: None,
                skippable: true,
                auto_advance_delay: None,
            },
            // Step 10: Podium upgrades
            TutorialStep {
                id: 10,
                step_type: TutorialStepType::Text,
                text: "Buy a podium upgrade to improve customer mood while they wait.".to_string(),
                highlight_rect: Some(Rect::new(Vec2::new(400.0, 100.0), Vec2::new(600.0, 200.0))),
                wait_condition: WaitCondition::AnyClick,
                on_complete: None,
                forced_action: None,
                skippable: true,
                auto_advance_delay: None,
            },
            // Step 11: Drink upgrades
            TutorialStep {
                id: 11,
                step_type: TutorialStepType::Text,
                text: "Drinks make customers more patient while deciding and eating.".to_string(),
                highlight_rect: None,
                wait_condition: WaitCondition::AnyClick,
                on_complete: None,
                forced_action: None,
                skippable: true,
                auto_advance_delay: None,
            },
            // Step 12: Shoes upgrade
            TutorialStep {
                id: 12,
                step_type: TutorialStepType::Text,
                text: "Better shoes make you move faster between tables!".to_string(),
                highlight_rect: None,
                wait_condition: WaitCondition::AnyClick,
                on_complete: None,
                forced_action: None,
                skippable: true,
                auto_advance_delay: None,
            },
            // Step 13: Oven upgrade
            TutorialStep {
                id: 13,
                step_type: TutorialStepType::Text,
                text: "A faster oven means food is ready sooner!".to_string(),
                highlight_rect: Some(Rect::new(Vec2::new(700.0, 400.0), Vec2::new(800.0, 500.0))),
                wait_condition: WaitCondition::AnyClick,
                on_complete: None,
                forced_action: None,
                skippable: true,
                auto_advance_delay: None,
            },
            // Step 14: Endless mode intro
            TutorialStep {
                id: 14,
                step_type: TutorialStepType::Text,
                text: "In Endless mode, difficulty increases over time. Buy upgrades between levels!".to_string(),
                highlight_rect: None,
                wait_condition: WaitCondition::AnyClick,
                on_complete: None,
                forced_action: None,
                skippable: true,
                auto_advance_delay: None,
            },
            // Step 15: Upgrade shop
            TutorialStep {
                id: 15,
                step_type: TutorialStepType::ForcedAction,
                text: "When the upgrade button flashes, click it to buy upgrades for the next level.".to_string(),
                highlight_rect: Some(Rect::new(Vec2::new(350.0, 500.0), Vec2::new(450.0, 580.0))), // Upgrade button
                wait_condition: WaitCondition::Action {
                    action_type: "click".to_string(),
                    target: "upgrade_button".to_string(),
                },
                on_complete: Some("shop_opened".to_string()),
                forced_action: Some(ForcedAction {
                    action_type: "click".to_string(),
                    target: "upgrade_button".to_string(),
                    hint_text: "Click the upgrade button".to_string(),
                    highlight_rect: Some(Rect::new(Vec2::new(350.0, 500.0), Vec2::new(450.0, 580.0))),
                }),
                skippable: false,
                auto_advance_delay: None,
            },
            // Step 16: Buy table
            TutorialStep {
                id: 16,
                step_type: TutorialStepType::ForcedAction,
                text: "Buy a new table to seat more customers at once.".to_string(),
                highlight_rect: None,
                wait_condition: WaitCondition::Action {
                    action_type: "click".to_string(),
                    target: "table_upgrade".to_string(),
                },
                on_complete: Some("table_bought".to_string()),
                forced_action: Some(ForcedAction {
                    action_type: "click".to_string(),
                    target: "table_upgrade".to_string(),
                    hint_text: "Click the table upgrade".to_string(),
                    highlight_rect: None,
                }),
                skippable: false,
                auto_advance_delay: None,
            },
            // Step 17: Continue endless
            TutorialStep {
                id: 17,
                step_type: TutorialStepType::Text,
                text: "Click continue to play the next level with your new table!".to_string(),
                highlight_rect: None,
                wait_condition: WaitCondition::AnyClick,
                on_complete: None,
                forced_action: None,
                skippable: true,
                auto_advance_delay: None,
            },
            // Step 18: Career mode intro
            TutorialStep {
                id: 18,
                step_type: TutorialStepType::Text,
                text: "Career mode has 5 restaurants with 10 levels each. Complete goals to advance!".to_string(),
                highlight_rect: None,
                wait_condition: WaitCondition::AnyClick,
                on_complete: None,
                forced_action: None,
                skippable: true,
                auto_advance_delay: None,
            },
            // Step 19: Closing time
            TutorialStep {
                id: 19,
                step_type: TutorialStepType::Text,
                text: "When the clock runs out, closing time begins. Clean all tables to finish the level!".to_string(),
                highlight_rect: Some(Rect::new(Vec2::new(350.0, 50.0), Vec2::new(450.0, 120.0))), // Clock
                wait_condition: WaitCondition::AnyClick,
                on_complete: None,
                forced_action: None,
                skippable: true,
                auto_advance_delay: None,
            },
            // Step 20: Expert scores
            TutorialStep {
                id: 20,
                step_type: TutorialStepType::Text,
                text: "Reach the Expert goal on every level to earn a star for that restaurant!".to_string(),
                highlight_rect: None,
                wait_condition: WaitCondition::AnyClick,
                on_complete: None,
                forced_action: None,
                skippable: true,
                auto_advance_delay: None,
            },
            // Step 21: High scores
            TutorialStep {
                id: 21,
                step_type: TutorialStepType::Text,
                text: "Your best scores are saved. Try to beat your personal records!".to_string(),
                highlight_rect: None,
                wait_condition: WaitCondition::AnyClick,
                on_complete: None,
                forced_action: None,
                skippable: true,
                auto_advance_delay: None,
            },
            // Step 22: Pause menu
            TutorialStep {
                id: 22,
                step_type: TutorialStepType::Text,
                text: "Press the pause button anytime to pause the game or quit to the menu.".to_string(),
                highlight_rect: Some(Rect::new(Vec2::new(750.0, 50.0), Vec2::new(790.0, 90.0))), // Pause button
                wait_condition: WaitCondition::AnyClick,
                on_complete: None,
                forced_action: None,
                skippable: true,
                auto_advance_delay: None,
            },
            // Step 23: Final tips
            TutorialStep {
                id: 23,
                step_type: TutorialStepType::Text,
                text: "Remember: seat quickly, serve fast, clean tables! You're ready to play!".to_string(),
                highlight_rect: None,
                wait_condition: WaitCondition::AnyClick,
                on_complete: None,
                forced_action: None,
                skippable: true,
                auto_advance_delay: None,
            },
            // Step 24: Start game
            TutorialStep {
                id: 24,
                step_type: TutorialStepType::AutoAdvance,
                text: "Good luck! Click to start your first level.".to_string(),
                highlight_rect: None,
                wait_condition: WaitCondition::AnyClick,
                on_complete: Some("tutorial_complete".to_string()),
                forced_action: None,
                skippable: false,
                auto_advance_delay: Some(2.0),
            },
        ]
    }
    
    /// Start tutorial
    pub fn start(&mut self) {
        self.active = true;
        self.current_step = 0;
        self.forced_action_active = false;
        self.overlay_alpha = 0.0;
        self.show_step(0);
    }
    
    /// Show current step
    fn show_step(&mut self, index: usize) {
        if index >= self.steps.len() {
            self.complete();
            return;
        }
        
        self.current_step = index;
        let step = &self.steps[index];
        
        // Show text box
        self.text_box.visible = true;
        self.text_box.target_alpha = 1.0;
        self.text_box.animation_timer = 0.0;
        
        // Show highlight/arrow if needed
        if let Some(rect) = step.highlight_rect {
            self.arrow.visible = true;
            self.arrow.target_position = rect.center();
            self.arrow.position = self.arrow.target_position + Vec2::new(0.0, -50.0);
        } else {
            self.arrow.visible = false;
        }
        
        // Handle forced action
        if let Some(action) = &step.forced_action {
            self.forced_action_active = true;
            // Would highlight target entity
        }
    }
    
    /// Complete tutorial
    pub fn complete(&mut self) {
        self.active = false;
        self.text_box.visible = false;
        self.text_box.target_alpha = 0.0;
        self.arrow.visible = false;
        self.forced_action_active = false;
    }
    
    /// Advance to next step
    pub fn next_step(&mut self) {
        if self.current_step + 1 < self.steps.len() {
            self.show_step(self.current_step + 1);
        } else {
            self.complete();
        }
    }
    
    /// Handle game event
    pub fn handle_event(&mut self, event_name: &str) -> bool {
        if !self.active || self.current_step >= self.steps.len() {
            return false;
        }
        
        let step = &self.steps[self.current_step];
        
        match &step.wait_condition {
            WaitCondition::Event { name } if name == event_name => {
                self.next_step();
                true
            }
            WaitCondition::Action { action_type, target } => {
                // Would check if action matches
                false
            }
            WaitCondition::AnyClick => {
                self.next_step();
                true
            }
            _ => false,
        }
    }
    
    /// Handle click at position
    pub fn handle_click(&mut self, pos: Vec2) -> bool {
        if !self.active || self.current_step >= self.steps.len() {
            return false;
        }
        
        let step = &self.steps[self.current_step];
        
        match &step.wait_condition {
            WaitCondition::Click { rect } => {
                if let Some(r) = rect {
                    if r.contains(pos) {
                        self.next_step();
                        return true;
                    }
                } else {
                    self.next_step();
                    return true;
                }
            }
            WaitCondition::AnyClick => {
                self.next_step();
                return true;
            }
            _ => {}
        }
        false
    }
    
    /// Update animations
    pub fn update(&mut self, dt: f32) {
        if !self.active {
            return;
        }
        
        // Text box fade
        self.text_box.animation_timer += dt;
        let target = self.text_box.target_alpha;
        let current = self.text_box.element.color.w;
        self.text_box.element.color.w = current + (target - current) * dt * 5.0;
        
        // Arrow bounce
        if self.arrow.visible {
            self.arrow.bounce_timer += dt * 3.0;
            self.arrow.bounce_offset = self.arrow.bounce_timer.sin() * 10.0;
            self.arrow.position = self.arrow.target_position + Vec2::new(0.0, -50.0 + self.arrow.bounce_offset);
        }
        
        // Auto-advance
        if let Some(step) = self.steps.get(self.current_step) {
            if step.step_type == TutorialStepType::AutoAdvance {
                if let Some(delay) = step.auto_advance_delay {
                    // Would track time in step
                }
            }
        }
    }
    
    /// Check if step is completed
    pub fn is_step_completed(&self, step_id: u32) -> bool {
        self.completed_steps.get(&step_id).copied().unwrap_or(false)
    }
    
    /// Mark step as completed
    pub fn mark_completed(&mut self, step_id: u32) {
        self.completed_steps.insert(step_id, true);
    }
    
    /// Get current step text
    pub fn current_text(&self) -> Option<&str> {
        self.steps.get(self.current_step).map(|s| s.text.as_str())
    }
    
    /// Get progress (0.0 - 1.0)
    pub fn progress(&self) -> f32 {
        if self.steps.is_empty() { 1.0 } else {
            self.current_step as f32 / self.steps.len() as f32
        }
    }
}

impl Default for TutorialManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Tutorial System
pub struct TutorialSystem {
    pub manager: TutorialManager,
}

impl TutorialSystem {
    pub fn new() -> Self {
        Self {
            manager: TutorialManager::new(),
        }
    }
    
    pub fn start(&mut self) {
        self.manager.start();
    }
    
    pub fn handle_click(&mut self, pos: Vec2) {
        self.manager.handle_click(pos);
    }
    
    pub fn handle_event(&mut self, event: &str) {
        self.manager.handle_event(event);
    }
}

impl System for TutorialSystem {
    fn name(&self) -> &str { "TutorialSystem" }
    fn priority(&self) -> i32 { 5 }
    
    fn run(&mut self, world: &mut World, dt: f32) {
        self.manager.update(dt);
    }
}

impl Default for TutorialSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tutorial_steps() {
        let manager = TutorialManager::new();
        assert_eq!(manager.steps.len(), 24);
        assert_eq!(manager.steps[0].id, 1);
        assert_eq!(manager.steps[23].id, 24);
    }
    
    #[test]
    fn test_tutorial_start() {
        let mut manager = TutorialManager::new();
        manager.start();
        assert!(manager.active);
        assert_eq!(manager.current_step, 0);
    }
    
    #[test]
    fn test_wait_conditions() {
        let step = TutorialStep {
            id: 1,
            step_type: TutorialStepType::Text,
            text: "Test".to_string(),
            highlight_rect: None,
            wait_condition: WaitCondition::AnyClick,
            on_complete: None,
            forced_action: None,
            skippable: true,
            auto_advance_delay: None,
        };
        
        match step.wait_condition {
            WaitCondition::AnyClick => assert!(true),
            _ => assert!(false),
        }
    }
}