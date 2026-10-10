//! Web Canvas2D Renderer for Diner Dash WASM
//!
//! Implements `diner_dash_engine::renderer::Renderer` using `CanvasRenderingContext2d`.

use diner_dash_engine::assets::FontId;
use diner_dash_engine::ecs::Sprite;
use diner_dash_engine::math::{Color, Mat4, Rect, Vec2};
use diner_dash_engine::renderer::{CommandBufferRenderer, DrawCommand, Renderer};
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};

pub struct WebRenderer {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    cmd_buffer: CommandBufferRenderer,
    images: HashMap<String, HtmlImageElement>,
}

impl WebRenderer {
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, JsValue> {
        let ctx = canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("Failed to acquire 2d context"))?
            .dyn_into::<CanvasRenderingContext2d>()?;

        Ok(Self {
            canvas,
            ctx,
            cmd_buffer: CommandBufferRenderer::new(),
            images: HashMap::new(),
        })
    }

    pub fn width(&self) -> f32 {
        self.canvas.width() as f32
    }

    pub fn height(&self) -> f32 {
        self.canvas.height() as f32
    }

    fn color_to_css(c: Color) -> String {
        let r = (c.x.clamp(0.0, 1.0) * 255.0) as u8;
        let g = (c.y.clamp(0.0, 1.0) * 255.0) as u8;
        let b = (c.z.clamp(0.0, 1.0) * 255.0) as u8;
        let a = c.w.clamp(0.0, 1.0);
        format!("rgba({},{},{},{:.3})", r, g, b, a)
    }

    fn get_or_load_image(&mut self, asset_path: &str) -> Option<&HtmlImageElement> {
        if !self.images.contains_key(asset_path) {
            if let Ok(img) = HtmlImageElement::new() {
                img.set_src(asset_path);
                self.images.insert(asset_path.to_string(), img);
            }
        }
        self.images.get(asset_path)
    }

    pub fn flush(&mut self) {
        let width = self.canvas.width() as f64;
        let height = self.canvas.height() as f64;
        let commands = self.cmd_buffer.take_commands();

        for cmd in commands {
            match cmd {
                DrawCommand::Clear(color) => {
                    self.ctx.set_fill_style_str(&Self::color_to_css(color));
                    self.ctx.fill_rect(0.0, 0.0, width, height);
                }
                DrawCommand::Rect {
                    rect,
                    color,
                    filled,
                    stroke_width,
                    ..
                } => {
                    let css = Self::color_to_css(color);
                    let x = rect.min.x as f64;
                    let y = rect.min.y as f64;
                    let w = rect.width() as f64;
                    let h = rect.height() as f64;
                    if filled {
                        self.ctx.set_fill_style_str(&css);
                        self.ctx.fill_rect(x, y, w, h);
                    } else {
                        self.ctx.set_stroke_style_str(&css);
                        self.ctx.set_line_width(stroke_width as f64);
                        self.ctx.stroke_rect(x, y, w, h);
                    }
                }
                DrawCommand::Text {
                    position,
                    text,
                    font,
                    size,
                    color,
                    ..
                } => {
                    let css = Self::color_to_css(color);
                    self.ctx.set_fill_style_str(&css);
                    if font == Some(FontId(1)) {
                        self.ctx.set_text_align("center");
                        self.ctx.set_font(&format!(
                            "900 {}px 'Comic Sans MS', 'Chalkboard SE', 'Trebuchet MS', sans-serif",
                            size as u32
                        ));
                        let _ = self
                            .ctx
                            .fill_text(&text, position.x as f64, position.y as f64);
                        self.ctx.set_text_align("left");
                    } else {
                        self.ctx.set_text_align("left");
                        self.ctx
                            .set_font(&format!("bold {}px 'Trebuchet MS', sans-serif", size as u32));
                        let _ = self
                            .ctx
                            .fill_text(&text, position.x as f64, position.y as f64);
                    }
                }
                DrawCommand::Sprite {
                    transform,
                    dst_size,
                    color,
                    ..
                } => {
                    let pos = transform.transform_point3(glam::Vec3::ZERO);
                    let css = Self::color_to_css(color);
                    self.ctx.set_fill_style_str(&css);
                    self.ctx.fill_rect(
                        pos.x as f64,
                        pos.y as f64,
                        dst_size.x as f64,
                        dst_size.y as f64,
                    );
                }
                DrawCommand::Image {
                    asset_path,
                    position,
                    reg_point,
                    flip_x,
                    ..
                } => {
                    let _ = self.get_or_load_image(&asset_path);
                    if let Some(img) = self.images.get(&asset_path) {
                        if img.complete() && img.natural_width() > 0 {
                            if flip_x {
                                self.ctx.save();
                                let _ = self.ctx.translate(position.x as f64, position.y as f64);
                                let _ = self.ctx.scale(-1.0, 1.0);
                                let _ = self.ctx.draw_image_with_html_image_element(
                                    img,
                                    -reg_point.x as f64,
                                    -reg_point.y as f64,
                                );
                                self.ctx.restore();
                            } else {
                                let _ = self.ctx.draw_image_with_html_image_element(
                                    img,
                                    (position.x - reg_point.x) as f64,
                                    (position.y - reg_point.y) as f64,
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Renderer for WebRenderer {
    fn begin_frame(&mut self) {
        self.cmd_buffer.begin_frame();
    }

    fn clear(&mut self, color: Color) {
        self.cmd_buffer.clear(color);
    }

    fn draw_sprite(&mut self, transform: Mat4, sprite: &Sprite, dst_size: Vec2) {
        self.cmd_buffer.draw_sprite(transform, sprite, dst_size);
    }

    fn draw_image(
        &mut self,
        asset_path: &str,
        position: Vec2,
        reg_point: Vec2,
        flip_x: bool,
        z_order: i32,
    ) {
        self.cmd_buffer
            .draw_image(asset_path, position, reg_point, flip_x, z_order);
    }

    fn draw_rect(&mut self, rect: Rect, color: Color, filled: bool, z_order: i32) {
        self.cmd_buffer.draw_rect(rect, color, filled, z_order);
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
        self.cmd_buffer
            .draw_text(position, text, font, size, color, z_order);
    }

    fn end_frame(&mut self) {
        self.cmd_buffer.end_frame();
        self.flush();
    }
}
