//! Renderer Abstraction
//!
//! Platform-agnostic 2D rendering trait and command buffer implementation.

use crate::assets::{FontId, TextureRegion};
use crate::ecs::Sprite;
use crate::math::{Color, Mat4, Rect, Vec2};

/// Draw command emitted by game/UI systems
#[derive(Debug, Clone)]
pub enum DrawCommand {
    Clear(Color),
    Image {
        asset_path: String,
        position: Vec2,
        reg_point: Vec2,
        flip_x: bool,
        z_order: i32,
    },
    Sprite {
        transform: Mat4,
        texture_region: TextureRegion,
        dst_size: Vec2,
        color: Color,
        z_order: i32,
    },
    Rect {
        rect: Rect,
        color: Color,
        filled: bool,
        stroke_width: f32,
        z_order: i32,
    },
    Text {
        position: Vec2,
        text: String,
        font: Option<FontId>,
        size: f32,
        color: Color,
        z_order: i32,
    },
}

impl DrawCommand {
    pub fn z_order(&self) -> i32 {
        match self {
            DrawCommand::Clear(_) => i32::MIN,
            DrawCommand::Image { z_order, .. } => *z_order,
            DrawCommand::Sprite { z_order, .. } => *z_order,
            DrawCommand::Rect { z_order, .. } => *z_order,
            DrawCommand::Text { z_order, .. } => *z_order,
        }
    }
}

/// Platform-independent renderer interface
pub trait Renderer {
    fn begin_frame(&mut self);
    fn clear(&mut self, color: Color);
    fn draw_image(
        &mut self,
        asset_path: &str,
        position: Vec2,
        reg_point: Vec2,
        flip_x: bool,
        z_order: i32,
    );
    fn draw_sprite(&mut self, transform: Mat4, sprite: &Sprite, dst_size: Vec2);
    fn draw_rect(&mut self, rect: Rect, color: Color, filled: bool, z_order: i32);
    fn draw_text(
        &mut self,
        position: Vec2,
        text: &str,
        font: Option<FontId>,
        size: f32,
        color: Color,
        z_order: i32,
    );
    fn end_frame(&mut self);
}

/// Command-buffer renderer that collects and sorts draw commands by z-order
#[derive(Debug, Default, Clone)]
pub struct CommandBufferRenderer {
    commands: Vec<DrawCommand>,
}

impl CommandBufferRenderer {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn commands(&self) -> &[DrawCommand] {
        &self.commands
    }

    pub fn take_commands(&mut self) -> Vec<DrawCommand> {
        std::mem::take(&mut self.commands)
    }
}

impl Renderer for CommandBufferRenderer {
    fn begin_frame(&mut self) {
        self.commands.clear();
    }

    fn clear(&mut self, color: Color) {
        self.commands.push(DrawCommand::Clear(color));
    }

    fn draw_image(
        &mut self,
        asset_path: &str,
        position: Vec2,
        reg_point: Vec2,
        flip_x: bool,
        z_order: i32,
    ) {
        self.commands.push(DrawCommand::Image {
            asset_path: asset_path.to_string(),
            position,
            reg_point,
            flip_x,
            z_order,
        });
    }

    fn draw_sprite(&mut self, transform: Mat4, sprite: &Sprite, dst_size: Vec2) {
        if !sprite.visible {
            return;
        }
        self.commands.push(DrawCommand::Sprite {
            transform,
            texture_region: sprite.texture_region.clone(),
            dst_size,
            color: sprite.color,
            z_order: sprite.z_index,
        });
    }

    fn draw_rect(&mut self, rect: Rect, color: Color, filled: bool, z_order: i32) {
        self.commands.push(DrawCommand::Rect {
            rect,
            color,
            filled,
            stroke_width: 1.0,
            z_order,
        });
    }

    fn draw_text(
        &mut self,
        position: Vec2,
        text: &str,
        font: Option<FontId>,
        size: f32,
        color: Color,
        z_order: i32,
    ) {
        self.commands.push(DrawCommand::Text {
            position,
            text: text.to_string(),
            font,
            size,
            color,
            z_order,
        });
    }

    fn end_frame(&mut self) {
        self.commands.sort_by_key(|cmd| cmd.z_order());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Vec4;

    #[test]
    fn test_command_buffer_sorting() {
        let mut renderer = CommandBufferRenderer::new();
        renderer.begin_frame();
        renderer.draw_rect(
            Rect::new(Vec2::ZERO, Vec2::new(10.0, 10.0)),
            Vec4::ONE,
            true,
            10,
        );
        renderer.draw_rect(
            Rect::new(Vec2::ZERO, Vec2::new(10.0, 10.0)),
            Vec4::ONE,
            true,
            1,
        );
        renderer.clear(Vec4::ZERO);
        renderer.end_frame();

        let cmds = renderer.commands();
        assert_eq!(cmds.len(), 3);
        assert_eq!(cmds[0].z_order(), i32::MIN);
        assert_eq!(cmds[1].z_order(), 1);
        assert_eq!(cmds[2].z_order(), 10);
    }
}
