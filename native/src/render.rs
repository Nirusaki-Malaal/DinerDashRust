use crate::text::{draw_text, measure_text};
use diner_dash_engine::{
    assets::FontId,
    renderer::{CommandBufferRenderer, DrawCommand},
};
use macroquad::prelude::*;
use std::collections::HashMap;

pub struct NativeRenderer {
    pub commands: CommandBufferRenderer,
    textures: HashMap<String, Texture2D>,
}
impl NativeRenderer {
    pub fn new() -> Self {
        Self {
            commands: CommandBufferRenderer::new(),
            textures: HashMap::new(),
        }
    }
    pub fn image(&mut self, path: &str, rect: Rect, flip_x: bool, tint: Color) {
        if !self.textures.contains_key(path) {
            if let Some(bytes) = crate::asset(path) {
                let texture = Texture2D::from_file_with_format(bytes, Some(ImageFormat::Png));
                texture.set_filter(FilterMode::Linear);
                self.textures.insert(path.into(), texture);
            }
        }
        if let Some(texture) = self.textures.get(path) {
            draw_texture_ex(
                texture,
                rect.x,
                rect.y,
                tint,
                DrawTextureParams {
                    dest_size: Some(vec2(rect.w, rect.h)),
                    flip_x,
                    ..Default::default()
                },
            );
        }
    }
    pub fn flush(&mut self) {
        for command in self.commands.take_commands() {
            match command {
                DrawCommand::Clear(c) => draw_rectangle(0., 0., 800., 600., color(c)),
                DrawCommand::Image {
                    asset_path,
                    position,
                    reg_point,
                    flip_x,
                    ..
                } => {
                    if !self.textures.contains_key(&asset_path) {
                        if let Some(bytes) = crate::asset(&asset_path) {
                            let texture =
                                Texture2D::from_file_with_format(bytes, Some(ImageFormat::Png));
                            texture.set_filter(FilterMode::Linear);
                            self.textures.insert(asset_path.clone(), texture);
                        } else {
                            eprintln!("Missing image: {asset_path}");
                        }
                    }
                    if let Some(t) = self.textures.get(&asset_path) {
                        let x = if flip_x {
                            position.x + reg_point.x - t.width()
                        } else {
                            position.x - reg_point.x
                        };
                        draw_texture_ex(
                            t,
                            x,
                            position.y - reg_point.y,
                            WHITE,
                            DrawTextureParams {
                                flip_x,
                                ..Default::default()
                            },
                        );
                    }
                }
                DrawCommand::Rect {
                    rect,
                    color: c,
                    filled,
                    stroke_width,
                    ..
                } => {
                    if filled {
                        draw_rectangle(
                            rect.min.x,
                            rect.min.y,
                            rect.width(),
                            rect.height(),
                            color(c),
                        );
                    } else {
                        draw_rectangle_lines(
                            rect.min.x,
                            rect.min.y,
                            rect.width(),
                            rect.height(),
                            stroke_width,
                            color(c),
                        );
                    }
                }
                DrawCommand::Text {
                    position,
                    text,
                    font,
                    size,
                    color: c,
                    ..
                } => {
                    let x = if font == Some(FontId(1)) {
                        position.x - measure_text(&text, None, size as u16, 1.).width / 2.
                    } else {
                        position.x
                    };
                    draw_text(&text, x, position.y, size, color(c));
                }
                DrawCommand::Sprite {
                    transform,
                    dst_size,
                    color: c,
                    ..
                } => {
                    let p = transform.w_axis;
                    draw_rectangle(p.x, p.y, dst_size.x, dst_size.y, color(c));
                }
            }
        }
    }
}
fn color(c: diner_dash_engine::math::Color) -> Color {
    Color::new(c.x, c.y, c.z, c.w)
}
