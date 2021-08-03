extern crate overload;
use overload::overload;
use rand::prelude::*;
use std::f32::consts::PI;
use std::ops;

#[derive(Debug, Default, Copy, Clone)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn inner_product(&self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn outer_product(&self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn length_squared(&self) -> f32 {
        self.inner_product(*self)
    }

    pub fn normalized(&self) -> Self {
        self * (1.0 / self.length_squared().sqrt())
    }

    pub fn reflect(&self, normal: Self) -> Self {
        self - normal * 2.0 * (self.inner_product(normal))
    }

    pub fn refract(&self, normal: Self, ior: f32) -> Self {
        let mut eta_t = ior;
        let mut eta_i = 1.0;
        let mut cosi = -self.inner_product(normal).min(1.0).max(-1.0);
        let n = if cosi < 0.0 {
            cosi *= -1.0;
            normal
        } else {
            let tmp = eta_t;
            eta_t = eta_i;
            eta_i = tmp;
            normal * -1.0
        };
        let eta = eta_i / eta_t;
        let k = 1.0 - eta * eta * (1.0 - cosi * cosi);
        if k < 0.0 {
            Self {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }
        } else {
            self * eta + n * (eta * cosi - k.sqrt())
        }
    }

    pub fn create_coord_system(&self) -> (Self, Self, Self) {
        let nt = if self.x.abs() > self.y.abs() {
            Vector3 {
                x: self.z,
                y: 0.0,
                z: -self.x,
            } / (self.x * self.x + self.z * self.z).sqrt()
        } else {
            Vector3 {
                x: 0.0,
                y: -self.z,
                z: self.y,
            } / (self.y * self.y + self.z * self.z).sqrt()
        };
        let nb = self.outer_product(nt);
        (*self, nt, nb)
    }

    pub fn coord(&self, index: usize) -> f32 {
        match index {
            0 => self.x,
            1 => self.y,
            _ => self.z,
        }
    }

    pub fn replace_coord(&self, index: usize, new_value: f32) -> Self {
        match index {
            0 => Vector3 {
                x: new_value,
                y: self.y,
                z: self.z,
            },
            1 => Vector3 {
                x: self.x,
                y: new_value,
                z: self.z,
            },
            _ => Vector3 {
                x: self.x,
                y: self.y,
                z: new_value,
            },
        }
    }

    pub fn min(&self, other: Self) -> Self {
        Vector3 {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
            z: self.z.min(other.z),
        }
    }

    pub fn max(&self, other: Self) -> Self {
        Vector3 {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
            z: self.z.max(other.z),
        }
    }

    pub fn to_array(&self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }

    pub fn random_in_unit_disk() -> Self {
        let mut rng = rand::thread_rng();
        let theta: f32 = (rng.gen::<f32>()) * 2.0 * PI;
        let r: f32 = rng.gen();
        Self {
            x: r * theta.cos(),
            y: r * theta.sin(),
            z: 0.0,
        }
    }

    pub fn random_in_hemisphere() -> Self {
        let mut rng = rand::thread_rng();
        let y = rng.gen::<f32>();
        let sin_theta = (1.0 - y * y).sqrt();
        let phi: f32 = (rng.gen::<f32>()) * 2.0 * PI;
        let x = sin_theta * phi.cos();
        let z = sin_theta * phi.sin();
        Self { x, y, z }
    }

    pub fn random_in_sphere() -> Self {
        let mut rng = rand::thread_rng();
        let y = -1.0 + 2.0 * rng.gen::<f32>();
        let sin_theta = (1.0 - y * y).sqrt();
        let phi: f32 = (rng.gen::<f32>()) * 2.0 * PI;
        let x = sin_theta * phi.cos();
        let z = sin_theta * phi.sin();
        Self { x, y, z }
    }
}

overload!((a: ?Vector3) + (b: ?Vector3) -> Vector3 { Vector3 { x: a.x + b.x, y: a.y + b.y, z: a.z + b.z } });
overload!((a: ?Vector3) - (b: ?Vector3) -> Vector3 { Vector3 { x: a.x - b.x, y: a.y - b.y, z: a.z - b.z } });
overload!((a: ?Vector3) * (b: ?Vector3) -> Vector3 { Vector3 { x: a.x * b.x, y: a.y * b.y, z: a.z * b.z } });
overload!((a: ?Vector3) * (b: f32) -> Vector3 { Vector3 { x: a.x * b, y: a.y * b, z: a.z * b } });
overload!((b: f32) * (a: ?Vector3) -> Vector3 { Vector3 { x: a.x * b, y: a.y * b, z: a.z * b } });
overload!((a: ?Vector3) / (b: f32) -> Vector3 { Vector3 { x: a.x / b, y: a.y / b, z: a.z / b } });
overload!((a: &mut Vector3) += (b: ?Vector3) { a.x += b.x; a.y += b.y; a.z += b.z; });
overload!((a: &mut Vector3) -= (b: ?Vector3) { a.x -= b.x; a.y -= b.y; a.z -= b.z; });
overload!((a: &mut Vector3) *= (b: f32) { a.x *= b; a.y *= b; a.z *= b; });
overload!((a: &mut Vector3) /= (b: f32) { a.x /= b; a.y /= b; a.z /= b; });
