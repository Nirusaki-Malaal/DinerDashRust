//! UI System
//! 
//! Handles HUD, buttons, dialogs, and UI animations

use anyhow::Result;
use std::collections::HashMap;
use crate::math::{Vec2, Rect, Color, easing};
use crate::ecs::{Entity, Component, World, System};
use crate::assets::{AssetManager, TextureRegion, SpriteFrame, Animation, LoopMode};
use crate::game::{GameMode, Stage};

/// UI Anchor points
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Anchor {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

/// UI Element base
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UIElement {
    pub id: String,
    pub anchor: Anchor,
    pub offset: Vec2,
    pub size: Vec2,
    pub visible: bool,
    pub z_index: i32,
    pub color: Color,
    pub texture_region: Option<TextureRegion>,
    pub text: Option<String>,
    pub font_id: Option<crate::assets::FontId>,
    pub font_size: f32,
}

impl Default for UIElement {
    fn default() -> Self {
        Self {
            id: String::new(),
            anchor: Anchor::TopLeft,
            offset: Vec2::ZERO,
            size: Vec2::new(100.0, 50.0),
            visible: true,
            z_index: 0,
            color: Color::new(1.0, 1.0, 1.0, 1.0),
            texture_region: None,
            text: None,
            font_id: None,
            font_size: 16.0,
        }
    }
}

impl UIElement {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            ..Default::default()
        }
    }
    
    pub fn with_anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }
    
    pub fn with_pos(mut self, offset: Vec2) -> Self {
        self.offset = offset;
        self
    }
    
    pub fn with_size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }
    
    pub fn with_texture(mut self, region: TextureRegion) -> Self {
        self.texture_region = Some(region);
        self
    }
    
    pub fn with_text(mut self, text: impl Into<String>, font_id: crate::assets::FontId, size: f32) -> Self {
        self.text = Some(text.into());
        self.font_id = Some(font_id);
        self.font_size = size;
        self
    }
    
    pub fn screen_rect(&self, screen_size: Vec2) -> Rect {
        let (anchor_x, anchor_y) = match self.anchor {
            Anchor::TopLeft => (0.0, 0.0),
            Anchor::TopCenter => (0.5, 0.0),
            Anchor::TopRight => (1.0, 0.0),
            Anchor::CenterLeft => (0.0, 0.5),
            Anchor::Center => (0.5, 0.5),
            Anchor::CenterRight => (1.0, 0.5),
            Anchor::BottomLeft => (0.0, 1.0),
            Anchor::BottomCenter => (0.5, 1.0),
            Anchor::BottomRight => (1.0, 1.0),
        };
        
        let x = screen_size.x * anchor_x + self.offset.x - self.size.x * anchor_x;
        let y = screen_size.y * anchor_y + self.offset.y - self.size.y * anchor_y;
        
        Rect::new(Vec2::new(x, y), Vec2::new(x + self.size.x, y + self.size.y))
    }
}

/// Button component
#[derive(Debug, Clone)]
pub struct Button {
    pub element: UIElement,
    pub normal_texture: Option<TextureRegion>,
    pub hover_texture: Option<TextureRegion>,
    pub pressed_texture: Option<TextureRegion>,
    pub disabled_texture: Option<TextureRegion>,
    pub callback: Option<String>,
    pub state: ButtonState,
    pub enabled: bool,
    pub toggle: bool,
    pub toggled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Normal,
    Hover,
    Pressed,
    Disabled,
}

impl Button {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            element: UIElement::new(id),
            normal_texture: None,
            hover_texture: None,
            pressed_texture: None,
            disabled_texture: None,
            callback: None,
            state: ButtonState::Normal,
            enabled: true,
            toggle: false,
            toggled: false,
        }
    }
    
    pub fn with_textures(
        mut self,
        normal: TextureRegion,
        hover: TextureRegion,
        pressed: TextureRegion,
    ) -> Self {
        self.normal_texture = Some(normal);
        self.hover_texture = Some(hover);
        self.pressed_texture = Some(pressed);
        self.element.texture_region = Some(normal);
        self
    }
    
    pub fn with_callback(mut self, callback: impl Into<String>) -> Self {
        self.callback = Some(callback.into());
        self
    }
    
    pub fn update(&mut self, mouse_pos: Vec2, mouse_pressed: bool, screen_size: Vec2) -> bool {
        if !self.enabled || !self.element.visible {
            self.state = ButtonState::Disabled;
            return false;
        }
        
        let rect = self.element.screen_rect(screen_size);
        let hovered = rect.contains(mouse_pos);
        
        let clicked = match self.state {
            ButtonState::Normal => {
                if hovered {
                    self.state = ButtonState::Hover;
                }
                false
            }
            ButtonState::Hover => {
                if !hovered {
                    self.state = ButtonState::Normal;
                    false
                } else if mouse_pressed {
                    self.state = ButtonState::Pressed;
                    false
                } else {
                    false
                }
            }
            ButtonState::Pressed => {
                if !mouse_pressed {
                    if hovered {
                        self.state = ButtonState::Hover;
                        true
                    } else {
                        self.state = ButtonState::Normal;
                        false
                    }
                } else {
                    false
                }
            }
            ButtonState::Disabled => false,
        };
        
        self.element.texture_region = match self.state {
            ButtonState::Normal => self.normal_texture,
            ButtonState::Hover => self.hover_texture.or(self.normal_texture),
            ButtonState::Pressed => self.pressed_texture.or(self.hover_texture).or(self.normal_texture),
            ButtonState::Disabled => self.disabled_texture.or(self.normal_texture),
        };
        
        clicked
    }
    
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.state = if enabled { ButtonState::Normal } else { ButtonState::Disabled };
    }
}

/// Progress bar
#[derive(Debug, Clone)]
pub struct ProgressBar {
    pub element: UIElement,
    pub fill_texture: Option<TextureRegion>,
    pub background_texture: Option<TextureRegion>,
    pub value: f32,
    pub direction: ProgressDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressDirection {
    LeftToRight,
    RightToLeft,
    BottomToTop,
    TopToBottom,
}

impl ProgressBar {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            element: UIElement::new(id),
            fill_texture: None,
            background_texture: None,
            value: 0.0,
            direction: ProgressDirection::LeftToRight,
        }
    }
    
    pub fn set_value(&mut self, value: f32) {
        self.value = value.clamp(0.0, 1.0);
    }
}

/// Star meter (lives display)
#[derive(Debug, Clone)]
pub struct StarMeter {
    pub element: UIElement,
    pub full_star: TextureRegion,
    pub empty_star: TextureRegion,
    pub half_star: Option<TextureRegion>,
    pub max_stars: u32,
    pub current_stars: u32,
    pub capacity: u32,
    pub animation_timer: f32,
    pub animating: bool,
}

impl StarMeter {
    pub fn new(id: impl Into<String>, max_stars: u32) -> Self {
        Self {
            element: UIElement::new(id),
            full_star: TextureRegion::new(
                crate::assets::TextureId(0),
                Vec2::ZERO,
                Vec2::ONE,
                crate::math::UVec2::ZERO,
            ),
            empty_star: TextureRegion::new(
                crate::assets::TextureId(0),
                Vec2::ZERO,
                Vec2::ONE,
                crate::math::UVec2::ZERO,
            ),
            half_star: None,
            max_stars,
            current_stars: 0,
            capacity: max_stars,
            animation_timer: 0.0,
            animating: false,
        }
    }
    
    pub fn set_full(&mut self, stars: u32) {
        self.current_stars = stars.min(self.capacity);
        self.animating = true;
        self.animation_timer = 0.0;
    }
    
    pub fn set_capacity(&mut self, capacity: u32) {
        self.capacity = capacity;
    }
    
    pub fn update(&mut self, dt: f32) {
        if self.animating {
            self.animation_timer += dt;
            if self.animation_timer > 1.0 {
                self.animating = false;
            }
        }
    }
}

/// Money jar (tip progress)
#[derive(Debug, Clone)]
pub struct MoneyJar {
    pub element: UIElement,
    pub frames: Vec<TextureRegion>,
    pub dollar_sign: Option<TextureRegion>,
    pub current_level: u32,
    pub target_level: u32,
    pub animation_timer: f32,
    pub blink_timer: f32,
    pub blinking: bool,
}

impl MoneyJar {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            element: UIElement::new(id),
            frames: Vec::new(),
            dollar_sign: None,
            current_level: 0,
            target_level: 0,
            animation_timer: 0.0,
            blink_timer: 0.0,
            blinking: false,
        }
    }
    
    pub fn set_animation(&mut self, level: u32) {
        self.target_level = level.min(9);
    }
    
    pub fn blink(&mut self) {
        self.blinking = true;
        self.blink_timer = 0.0;
    }
    
    pub fn update(&mut self, dt: f32) {
        if self.current_level != self.target_level {
            self.animation_timer += dt * 5.0;
            let diff = (self.target_level as f32 - self.current_level as f32).abs();
            if self.animation_timer >= diff {
                self.current_level = self.target_level;
                self.animation_timer = 0.0;
            }
        }
        
        if self.blinking {
            self.blink_timer += dt;
            if self.blink_timer > 1.25 {
                self.blinking = false;
                self.blink_timer = 0.0;
            }
        }
    }
}

/// Clock hand (level timer)
#[derive(Debug, Clone)]
pub struct ClockHand {
    pub element: UIElement,
    pub hand_texture: TextureRegion,
    pub face_texture: Option<TextureRegion>,
    pub end_mark: f32,
    pub current_mark: f32,
    pub visible: bool,
}

impl ClockHand {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            element: UIElement::new(id),
            hand_texture: TextureRegion::new(
                crate::assets::TextureId(0),
                Vec2::ZERO,
                Vec2::ONE,
                crate::math::UVec2::ZERO,
            ),
            face_texture: None,
            end_mark: 1.0,
            current_mark: 0.0,
            visible: true,
        }
    }
    
    pub fn set_end_mark(&mut self, mark: f32) {
        self.end_mark = mark.clamp(0.0, 1.0);
    }
    
    pub fn set_mark(&mut self, mark: f32) {
        self.current_mark = mark.clamp(0.0, self.end_mark);
    }
}

/// Text display with animation
#[derive(Debug, Clone)]
pub struct AnimatedText {
    pub element: UIElement,
    pub target_text: String,
    pub current_text: String,
    pub char_timer: f32,
    pub char_index: usize,
    pub typewriter_speed: f32,
    pub color: Color,
    pub target_color: Color,
}

impl AnimatedText {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            element: UIElement::new(id),
            target_text: String::new(),
            current_text: String::new(),
            char_timer: 0.0,
            char_index: 0,
            typewriter_speed: 30.0,
            color: Color::new(1.0, 1.0, 1.0, 1.0),
            target_color: Color::new(1.0, 1.0, 1.0, 1.0),
        }
    }
    
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.target_text = text.into();
        self.current_text.clear();
        self.char_index = 0;
        self.char_timer = 0.0;
    }
    
    pub fn set_color(&mut self, color: Color) {
        self.target_color = color;
    }
    
    pub fn update(&mut self, dt: f32) {
        if self.char_index < self.target_text.len() {
            self.char_timer += dt;
            let chars_per_frame = self.typewriter_speed * dt;
            let new_index = (self.char_index as f32 + chars_per_frame) as usize;
            self.char_index = new_index.min(self.target_text.len());
            self.current_text = self.target_text[..self.char_index].to_string();
        }
        
        self.color = crate::math::lerp_color(self.color, self.target_color, dt * 5.0);
    }
}

/// Event text (popups for goal/bonus/new star)
#[derive(Debug, Clone)]
pub struct EventText {
    pub element: UIElement,
    pub home_position: Vec2,
    pub up_position: Vec2,
    pub state: EventTextState,
    pub timer: f32,
    pub flash_count: u32,
    pub flash_timer: f32,
    pub move_callback: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventTextState {
    Hidden,
    Showing,
    Flashing,
    MovingUp,
}

impl EventText {
    pub fn new(id: impl Into<String>, home_pos: Vec2) -> Self {
        Self {
            element: UIElement::new(id).with_pos(home_pos),
            home_position: home_pos,
            up_position: home_pos - Vec2::new(0.0, 400.0),
            state: EventTextState::Hidden,
            timer: 0.0,
            flash_count: 0,
            flash_timer: 0.0,
            move_callback: None,
        }
    }
    
    pub fn show(&mut self) {
        self.state = EventTextState::Showing;
        self.element.offset = self.home_position;
        self.timer = 0.0;
    }
    
    pub fn flash(&mut self, loops: u32, ms_per_flash: u32, callback: Option<String>) {
        self.state = EventTextState::Flashing;
        self.flash_count = loops;
        self.flash_timer = ms_per_flash as f32 / 1000.0;
        self.move_callback = callback;
    }
    
    pub fn move_up(&mut self, ms: f32, callback: Option<String>) {
        self.state = EventTextState::MovingUp;
        self.timer = ms / 1000.0;
        self.move_callback = callback;
    }
    
    pub fn update(&mut self, dt: f32) -> Option<String> {
        match self.state {
            EventTextState::Hidden => {}
            EventTextState::Showing => {
                self.timer += dt;
                if self.timer > 3.0 {
                    self.state = EventTextState::Hidden;
                    self.element.visible = false;
                }
            }
            EventTextState::Flashing => {
                self.flash_timer -= dt;
                if self.flash_timer <= 0.0 {
                    self.element.visible = !self.element.visible;
                    if self.flash_count > 0 {
                        self.flash_count -= 1;
                        self.flash_timer = 150.0 / 1000.0;
                    } else {
                        self.state = EventTextState::Hidden;
                        self.element.visible = false;
                        return self.move_callback.take();
                    }
                }
            }
            EventTextState::MovingUp => {
                self.timer -= dt;
                let t = 1.0 - (self.timer / (1000.0 * dt)).max(0.0);
                let eased = easing::ease_out_cubic(t);
                self.element.offset = crate::math::lerp_vec2(self.home_position, self.up_position, eased);
                
                if self.timer <= 0.0 {
                    self.state = EventTextState::Hidden;
                    self.element.visible = false;
                    return self.move_callback.take();
                }
            }
        }
        None
    }
}

/// Main HUD
#[derive(Debug, Clone)]
pub struct HUD {
    pub score_text: AnimatedText,
    pub goal_text: AnimatedText,
    pub level_text: AnimatedText,
    pub star_meter: StarMeter,
    pub money_jar: MoneyJar,
    pub clock: ClockHand,
    pub quit_button: Button,
    pub skip_button: Option<Button>,
    pub upgrade_button: Button,
    pub event_texts: HashMap<String, EventText>,
    pub notepad: Option<UIElement>,
    pub endless_interface: Option<EndlessInterface>,
}

#[derive(Debug, Clone)]
pub struct EndlessInterface {
    pub score_text: AnimatedText,
    pub level_text: AnimatedText,
    pub heart: UIElement,
    pub upgrade_button: Button,
    pub new_item: EventText,
}

impl HUD {
    pub fn new(_assets: &AssetManager) -> Result<Self> {
        Ok(Self {
            score_text: AnimatedText::new("score"),
            goal_text: AnimatedText::new("goal"),
            level_text: AnimatedText::new("level"),
            star_meter: StarMeter::new("stars", 5),
            money_jar: MoneyJar::new("money_jar"),
            clock: ClockHand::new("clock"),
            quit_button: Button::new("quit"),
            skip_button: None,
            upgrade_button: Button::new("upgrade"),
            event_texts: HashMap::new(),
            notepad: None,
            endless_interface: None,
        })
    }
    
    pub fn update(&mut self, dt: f32, screen_size: Vec2, mouse_pos: Vec2, mouse_pressed: bool) -> Vec<String> {
        let mut callbacks = Vec::new();
        
        self.score_text.update(dt);
        self.goal_text.update(dt);
        self.level_text.update(dt);
        self.star_meter.update(dt);
        self.money_jar.update(dt);
        
        if self.quit_button.update(mouse_pos, mouse_pressed, screen_size) {
            callbacks.push("pause".to_string());
        }
        
        if let Some(btn) = &mut self.skip_button {
            if btn.update(mouse_pos, mouse_pressed, screen_size) {
                callbacks.push("skip_level".to_string());
            }
        }
        
        if self.upgrade_button.update(mouse_pos, mouse_pressed, screen_size) {
            callbacks.push("click_upgrade".to_string());
        }
        
        for event_text in self.event_texts.values_mut() {
            if let Some(cb) = event_text.update(dt) {
                callbacks.push(cb);
            }
        }
        
        if let Some(ei) = &mut self.endless_interface {
            ei.score_text.update(dt);
            ei.level_text.update(dt);
            ei.upgrade_button.update(mouse_pos, mouse_pressed, screen_size);
            if let Some(cb) = ei.new_item.update(dt) {
                callbacks.push(cb);
            }
        }
        
        callbacks
    }
    
    pub fn set_score(&mut self, score: u32) {
        self.score_text.set_text(format!("{}", score));
    }
    
    pub fn set_goal(&mut self, goal: u32, bonus: u32, expert: bool) {
        if expert {
            self.goal_text.set_text(format!("EXPERT: {}", bonus));
            self.goal_text.set_color(Color::new(1.0, 0.0, 1.0, 1.0));
        } else {
            self.goal_text.set_text(format!("GOAL: {}", goal));
            self.goal_text.set_color(Color::new(0.0, 1.0, 1.0, 1.0));
        }
    }
    
    pub fn set_level(&mut self, level: u32, mode: GameMode) {
        if mode == GameMode::Career {
            self.level_text.set_text(format!("LEVEL {}", level));
        } else {
            self.endless_interface.as_mut().map(|ei| ei.level_text.set_text(format!("{}", level)));
        }
    }
    
    pub fn show_event(&mut self, name: &str, home_pos: Vec2) {
        let event = EventText::new(name, home_pos);
        self.event_texts.insert(name.to_string(), event);
    }
    
    pub fn trigger_event(&mut self, name: &str, flash: bool, move_up: bool) {
        if let Some(event) = self.event_texts.get_mut(name) {
            event.show();
            if flash {
                event.flash(4, 150, None);
            }
            if move_up {
                event.move_up(750.0, None);
            }
        }
    }
}

/// UI System
pub struct UISystem {
    pub hud: Option<HUD>,
    pub screen_size: Vec2,
    pub mouse_pos: Vec2,
    pub mouse_pressed: bool,
}

impl UISystem {
    pub fn new(screen_size: Vec2) -> Self {
        Self {
            hud: None,
            screen_size,
            mouse_pos: Vec2::ZERO,
            mouse_pressed: false,
        }
    }
    
    pub fn set_hud(&mut self, hud: HUD) {
        self.hud = Some(hud);
    }
    
    pub fn set_mouse(&mut self, pos: Vec2, pressed: bool) {
        self.mouse_pos = pos;
        self.mouse_pressed = pressed;
    }
    
    pub fn update(&mut self, dt: f32) -> Vec<String> {
        if let Some(hud) = &mut self.hud {
            hud.update(dt, self.screen_size, self.mouse_pos, self.mouse_pressed)
        } else {
            Vec::new()
        }
    }
}

impl System for UISystem {
    fn name(&self) -> &str { "UISystem" }
    fn priority(&self) -> i32 { 10 }
    
    fn run(&mut self, _world: &mut World, dt: f32) {
        let _callbacks = self.update(dt);
    }
}

impl Default for UISystem {
    fn default() -> Self {
        Self::new(Vec2::new(800.0, 600.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_button() {
        let mut button = Button::new("test");
        button.element.anchor = Anchor::Center;
        let screen = Vec2::new(800.0, 600.0);
        
        button.update(Vec2::new(400.0, 300.0), false, screen);
        assert_eq!(button.state, ButtonState::Hover);
        
        button.update(Vec2::new(400.0, 300.0), true, screen);
        assert_eq!(button.state, ButtonState::Pressed);
        
        let clicked = button.update(Vec2::new(400.0, 300.0), false, screen);
        assert!(clicked);
    }
    
    #[test]
    fn test_progress_bar() {
        let mut bar = ProgressBar::new("test");
        bar.set_value(0.5);
        assert_eq!(bar.value, 0.5);
        
        bar.set_value(1.5);
        assert_eq!(bar.value, 1.0);
    }
}