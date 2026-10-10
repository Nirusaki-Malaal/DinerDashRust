//! Web Input Manager for Diner Dash WASM

use diner_dash_engine::math::Vec2;
use std::collections::HashSet;
use web_sys::HtmlCanvasElement;

#[derive(Debug, Default, Clone)]
pub struct InputManager {
    pub mouse_pos: Vec2,
    pub mouse_pressed: bool,
    pub pending_clicks: Vec<Vec2>,
    pub keys_pressed: HashSet<String>,
    pub keys_just_pressed: Vec<String>,
}

impl InputManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn map_client_to_canvas(
        canvas: &HtmlCanvasElement,
        client_x: f64,
        client_y: f64,
    ) -> Vec2 {
        let rect = canvas.get_bounding_client_rect();
        let rw = rect.width().max(1.0);
        let rh = rect.height().max(1.0);
        let cw = canvas.width() as f64;
        let ch = canvas.height() as f64;

        let x = ((client_x - rect.left()) / rw) * cw;
        let y = ((client_y - rect.top()) / rh) * ch;
        Vec2::new(x as f32, y as f32)
    }

    pub fn record_pointer_down(&mut self, pos: Vec2) {
        self.mouse_pos = pos;
        self.mouse_pressed = true;
        self.pending_clicks.push(pos);
    }

    pub fn record_pointer_move(&mut self, pos: Vec2) {
        self.mouse_pos = pos;
    }

    pub fn record_pointer_up(&mut self, pos: Vec2) {
        self.mouse_pos = pos;
        self.mouse_pressed = false;
    }

    pub fn record_key_down(&mut self, key: String) {
        if self.keys_pressed.insert(key.clone()) {
            self.keys_just_pressed.push(key);
        }
    }

    pub fn record_key_up(&mut self, key: &str) {
        self.keys_pressed.remove(key);
    }

    pub fn take_clicks(&mut self) -> Vec<Vec2> {
        std::mem::take(&mut self.pending_clicks)
    }

    pub fn take_keys(&mut self) -> Vec<String> {
        std::mem::take(&mut self.keys_just_pressed)
    }
}
