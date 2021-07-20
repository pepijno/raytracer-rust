extern crate overload;
use overload::overload;
use std::ops;
use std::fmt;
use rand::prelude::*;
use std::f32::consts::PI;

#[derive(Default, Copy, Clone)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x: x, y: y, z: z }
    }

    pub fn inner_product(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn outer_product(&self, other: &Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn length_squared(&self) -> f32 {
        self.inner_product(self)
    }

    pub fn normalized(&self) -> Self {
        self * (1.0 / self.length_squared().sqrt())
    }

    pub fn reflect(&self, normal: &Self) -> Self {
        self - normal * 2.0 * (self.inner_product(normal))
    }

    pub fn refract(&self, normal: &Self, eta_t: f32, eta_i: f32) -> Self {
        let cosi = -self.inner_product(normal).min(1.0).max(-1.0);
        if cosi < 0.0 {
            return self.refract(&(normal * -1.0), eta_i, eta_t);
        }
        let eta = eta_i / eta_t;
        let k = 1.0 - eta * eta * (1.0 - cosi * cosi);
        if k < 0.0 {
            Self { x: 0.0, y: 0.0, z: 0.0 }
        } else {
            self * eta + normal * (eta * cosi - k.sqrt())
        }
    }
}

pub fn random_in_unit_disk() -> Vector3 {
    let mut rng = rand::thread_rng();
    let theta: f32 = (rng.gen::<f32>()) * 2.0 * PI;
    let r: f32 = rng.gen();
    Vector3 {
        x: r * theta.cos(),
        y: r * theta.sin(),
        z: 0.0,
    }
}

overload!((a: ?Vector3) + (b: ?Vector3) -> Vector3 { Vector3 { x: a.x + b.x, y: a.y + b.y, z: a.z + b.z } });
overload!((a: ?Vector3) - (b: ?Vector3) -> Vector3 { Vector3 { x: a.x - b.x, y: a.y - b.y, z: a.z - b.z } });
overload!((a: ?Vector3) * (b: f32) -> Vector3 { Vector3 { x: a.x * b, y: a.y * b, z: a.z * b } });
overload!((b: f32) * (a: ?Vector3) -> Vector3 { Vector3 { x: a.x * b, y: a.y * b, z: a.z * b } });

impl fmt::Display for Vector3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}
