extern crate overload;
use overload::overload;
use std::ops; // <- don't forget this or you'll get nasty errors
use std::fmt;

#[derive(Copy, Clone)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
}

impl Color {
    pub fn black() -> Self {
        Self { r: 0.0, g: 0.0, b: 0.0 }
    }
    pub fn singular(c: f32) -> Self {
        Self { r: c, g: c, b: c }
    }
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r: r, g: g, b: b }
    }

    pub fn to_buffer(&self) -> [u8; 3] {
        [
            self.r_byte(),
            self.g_byte(),
            self.b_byte(),
        ]
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

#[derive(Copy, Clone)]
pub struct Material {
    pub refractive_index: f32,
    pub albedo: [f32; 4],
    pub diffuse_color: Color,
    pub specular_exponent: f32,
}
