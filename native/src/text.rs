//! Embedded font keeps text consistent on Windows, Linux, and Android.
use macroquad::prelude::*;
use std::cell::OnceCell;
thread_local! { static FONT: OnceCell<Font> = const { OnceCell::new() }; }
pub fn init() {
    FONT.with(|slot| {
        let mut font =
            load_ttf_font_from_bytes(include_bytes!("../../packaging/fonts/NotoSans-Bold.ttf"))
                .expect("embedded font");
        font.set_filter(FilterMode::Linear);
        let _ = slot.set(font);
    });
}
pub fn measure_text(text: &str, font: Option<&Font>, size: u16, scale: f32) -> TextDimensions {
    FONT.with(|slot| macroquad::text::measure_text(text, font.or(slot.get()), size, scale))
}
pub fn draw_text(text: &str, x: f32, y: f32, size: f32, color: Color) -> TextDimensions {
    FONT.with(|slot| {
        macroquad::text::draw_text_ex(
            text,
            x,
            y,
            TextParams {
                font: slot.get(),
                font_size: size as u16,
                color,
                ..Default::default()
            },
        )
    })
}
