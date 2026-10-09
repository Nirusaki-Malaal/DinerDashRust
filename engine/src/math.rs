//! Math utilities for the game engine

// Re-export glam types
pub use glam::{Vec2, Vec3, Vec4, Mat4, Quat, UVec2};

/// 4D Vector / Color alias
pub type Color = Vec4;

/// 2D Rectangle (min, max)
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
}

impl Rect {
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }
    
    pub fn from_center_size(center: Vec2, size: Vec2) -> Self {
        let half = size * 0.5;
        Self {
            min: center - half,
            max: center + half,
        }
    }
    
    pub fn width(&self) -> f32 { self.max.x - self.min.x }
    pub fn height(&self) -> f32 { self.max.y - self.min.y }
    pub fn size(&self) -> Vec2 { Vec2::new(self.width(), self.height()) }
    pub fn center(&self) -> Vec2 { (self.min + self.max) * 0.5 }
    
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y
    }
    
    pub fn intersects(&self, other: &Rect) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
        self.min.y <= other.max.y && self.max.y >= other.min.y
    }
    
    pub fn expand(&self, amount: f32) -> Self {
        Self {
            min: self.min - Vec2::splat(amount),
            max: self.max + Vec2::splat(amount),
        }
    }
}

/// 2D Transform (position, rotation, scale)
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Transform2D {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }
}

impl Transform2D {
    pub fn new(position: Vec2) -> Self {
        Self { position, ..Default::default() }
    }
    
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            self.scale.extend(1.0),
            Quat::from_rotation_z(self.rotation),
            self.position.extend(0.0),
        )
    }
    
    pub fn translate(&mut self, delta: Vec2) {
        self.position += delta;
    }
    
    pub fn rotate(&mut self, angle: f32) {
        self.rotation += angle;
    }
    
    pub fn scale_by(&mut self, factor: Vec2) {
        self.scale *= factor;
    }
}

/// Linear interpolation
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

/// Vector2 linear interpolation
pub fn lerp_vec2(a: Vec2, b: Vec2, t: f32) -> Vec2 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

/// Color linear interpolation
pub fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    a + (b - a) * t.clamp(0.0, 1.0)
}

/// Easing functions
pub mod easing {
    use super::*;
    
    pub fn linear(t: f32) -> f32 { t }
    
    pub fn ease_in_quad(t: f32) -> f32 { t * t }
    pub fn ease_out_quad(t: f32) -> f32 { 1.0 - (1.0 - t) * (1.0 - t) }
    pub fn ease_in_out_quad(t: f32) -> f32 {
        if t < 0.5 { 2.0 * t * t } else { 1.0 - 2.0 * (1.0 - t) * (1.0 - t) }
    }
    
    pub fn ease_in_cubic(t: f32) -> f32 { t * t * t }
    pub fn ease_out_cubic(t: f32) -> f32 { 1.0 - (1.0 - t).powi(3) }
    pub fn ease_in_out_cubic(t: f32) -> f32 {
        if t < 0.5 { 4.0 * t * t * t } else { 1.0 - 4.0 * (1.0 - t).powi(3) }
    }
    
    pub fn ease_in_quart(t: f32) -> f32 { t * t * t * t }
    pub fn ease_out_quart(t: f32) -> f32 { 1.0 - (1.0 - t).powi(4) }
    pub fn ease_in_out_quart(t: f32) -> f32 {
        if t < 0.5 { 8.0 * t * t * t * t } else { 1.0 - 8.0 * (1.0 - t).powi(4) }
    }
    
    pub fn ease_in_sine(t: f32) -> f32 { 1.0 - (t * std::f32::consts::FRAC_PI_2).cos() }
    pub fn ease_out_sine(t: f32) -> f32 { (t * std::f32::consts::FRAC_PI_2).sin() }
    pub fn ease_in_out_sine(t: f32) -> f32 { 0.5 * (1.0 - (t * std::f32::consts::PI).cos()) }
    
    pub fn ease_in_expo(t: f32) -> f32 { if t == 0.0 { 0.0 } else { 2.0_f32.powf(10.0 * (t - 1.0)) } }
    pub fn ease_out_expo(t: f32) -> f32 { if t == 1.0 { 1.0 } else { 1.0 - 2.0_f32.powf(-10.0 * t) } }
    pub fn ease_in_out_expo(t: f32) -> f32 {
        if t == 0.0 { 0.0 }
        else if t == 1.0 { 1.0 }
        else if t < 0.5 { 0.5 * 2.0_f32.powf(20.0 * t - 10.0) }
        else { 1.0 - 0.5 * 2.0_f32.powf(-20.0 * t + 10.0) }
    }
    
    pub fn ease_in_circ(t: f32) -> f32 { 1.0 - (1.0 - t * t).sqrt() }
    pub fn ease_out_circ(t: f32) -> f32 { (1.0 - (t - 1.0) * (t - 1.0)).sqrt() }
    pub fn ease_in_out_circ(t: f32) -> f32 {
        if t < 0.5 { 0.5 * (1.0 - (1.0 - 4.0 * t * t).sqrt()) }
        else { 0.5 * ((1.0 - (4.0 * t - 2.0) * (4.0 * t - 2.0)).sqrt() + 1.0) }
    }
    
    pub fn ease_in_back(t: f32) -> f32 {
        const C1: f32 = 1.70158;
        const C3: f32 = C1 + 1.0;
        C3 * t * t * t - C1 * t * t
    }
    
    pub fn ease_out_back(t: f32) -> f32 {
        const C1: f32 = 1.70158;
        const C3: f32 = C1 + 1.0;
        1.0 + C3 * (t - 1.0).powi(3) + C1 * (t - 1.0).powi(2)
    }
    
    pub fn ease_in_out_back(t: f32) -> f32 {
        const C1: f32 = 1.70158;
        const C2: f32 = C1 * 1.525;
        if t < 0.5 {
            (2.0 * t).powi(2) * ((C2 + 1.0) * 2.0 * t - C2) * 0.5
        } else {
            (2.0 * t - 2.0).powi(2) * ((C2 + 1.0) * (2.0 * t - 2.0) + C2) * 0.5 + 1.0
        }
    }
    
    pub fn ease_out_elastic(t: f32) -> f32 {
        const C4: f32 = 2.0 * std::f32::consts::PI / 3.0;
        if t == 0.0 { 0.0 }
        else if t == 1.0 { 1.0 }
        else { 2.0_f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * C4).sin() + 1.0 }
    }
    
    pub fn ease_out_bounce(t: f32) -> f32 {
        const N1: f32 = 7.5625;
        const D1: f32 = 2.75;
        if t < 1.0 / D1 {
            N1 * t * t
        } else if t < 2.0 / D1 {
            N1 * (t - 1.5 / D1).powi(2) + 0.75
        } else if t < 2.5 / D1 {
            N1 * (t - 2.25 / D1).powi(2) + 0.9375
        } else {
            N1 * (t - 2.625 / D1).powi(2) + 0.984375
        }
    }
}

/// Random utilities
pub mod rand {
    use rand::Rng;
    
    pub fn range(min: f32, max: f32) -> f32 {
        rand::thread_rng().gen_range(min..max)
    }
    
    pub fn range_int(min: i32, max: i32) -> i32 {
        rand::thread_rng().gen_range(min..=max)
    }
    
    pub fn bool(probability: f32) -> bool {
        rand::thread_rng().gen::<f32>() < probability
    }
    
    pub fn choose<T>(slice: &[T]) -> Option<&T> {
        if slice.is_empty() { None } else {
            let idx = rand::thread_rng().gen_range(0..slice.len());
            Some(&slice[idx])
        }
    }
}

/// Clamp value to range
pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
}

/// Smooth step interpolation
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Smoother step interpolation
pub fn smootherstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        assert!(rect.contains(Vec2::new(50.0, 50.0)));
        assert!(!rect.contains(Vec2::new(150.0, 50.0)));
    }
    
    #[test]
    fn test_lerp() {
        assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
        assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
        assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
    }
    
    #[test]
    fn test_easing() {
        assert!((easing::ease_out_quad(0.5) - 0.75).abs() < 0.01);
        assert!((easing::ease_in_quad(0.5) - 0.25).abs() < 0.01);
    }
}
