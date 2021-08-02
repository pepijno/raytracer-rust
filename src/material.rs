extern crate overload;
use overload::overload;
use std::fmt;
use std::ops; // <- don't forget this or you'll get nasty errors

#[derive(Debug, Copy, Clone)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub fn black() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        }
    }

    pub fn white() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
        }
    }

    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    pub fn max(&self) -> f32 {
        self.r.max(self.g).max(self.b)
    }

    pub fn sum(&self) -> f32 {
        self.r + self.g + self.b
    }

    pub fn capped(&self) -> Self {
        Color {
            r: self.r.min(1.0),
            g: self.g.min(1.0),
            b: self.b.min(1.0),
        }
    }

    pub fn to_buffer(&self) -> [u8; 3] {
        [self.r_byte(), self.g_byte(), self.b_byte()]
    }

    pub fn r_byte(&self) -> u8 {
        ((self.r * 255.0).min(255.0)).max(0.0) as u8
    }

    pub fn g_byte(&self) -> u8 {
        ((self.g * 255.0).min(255.0)).max(0.0) as u8
    }

    pub fn b_byte(&self) -> u8 {
        ((self.b * 255.0).min(255.0)).max(0.0) as u8
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(R:{}, G:{}, B:{})", self.r, self.g, self.b)
    }
}

overload!((a: ?Color) + (b: ?Color) -> Color { Color { r: a.r + b.r, g: a.g + b.g, b: a.b + b.b } });
overload!((a: ?Color) * (b: ?Color) -> Color { Color { r: a.r * b.r, g: a.g * b.g, b: a.b * b.b } });
overload!((a: ?Color) - (b: ?Color) -> Color { Color { r: a.r - b.r, g: a.g - b.g, b: a.b - b.b } });
overload!((a: ?Color) * (b: f32) -> Color { Color { r: a.r * b, g: a.g * b, b: a.b * b } });
overload!((b: f32) * (a: ?Color) -> Color { Color { r: a.r * b, g: a.g * b, b: a.b * b } });
overload!((a: ?Color) / (b: f32) -> Color { Color { r: a.r / b, g: a.g / b, b: a.b / b } });
overload!((a: &mut Color) += (b: ?Color) { a.r += b.r; a.g += b.g; a.b += b.b; });
overload!((a: &mut Color) -= (b: ?Color) { a.r -= b.r; a.g -= b.g; a.b -= b.b; });
overload!((a: &mut Color) *= (b: f32) { a.r *= b; a.g *= b; a.b *= b; });
overload!((a: &mut Color) /= (b: f32) { a.r /= b; a.g /= b; a.b /= b; });

#[derive(Copy, Clone)]
pub struct Material {
    pub refractive_index: f32,
    pub diffuse_color: Color,
    pub specular_exponent: f32,
    pub reflect_color: Color,
}
