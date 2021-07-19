use crate::vector3::Vector3;
use crate::vector3::random_in_unit_disk;
use crate::ray::Ray;
use std::f32::consts::PI;

#[derive(Copy, Clone)]
pub struct Camera {
    origin: Vector3,
    screen_dl: Vector3,
    horizontal: Vector3,
    vertical: Vector3,
    u: Vector3,
    v: Vector3,
    w: Vector3,
    lens_radius: f32,
}

impl Camera {
    pub fn new(look_from: &Vector3, look_at: Vector3, vup: Vector3, vofv: f32, aspect: f32, aperture: f32, focus_distance: f32) -> Self {
        let theta = vofv * PI / 180.0;
        let half_height = (theta / 2.0).tan();
        let half_width = aspect * half_height;
        let w = (look_from - look_at).normalized();
        let u = vup.outer_product(&w).normalized();
        let v = w.outer_product(&u);
        Camera {
            origin: *look_from,
            screen_dl: look_from - u * half_width * focus_distance- v * half_height * focus_distance - w * focus_distance,
            horizontal: u * 2.0 * half_width * focus_distance,
            vertical: v * 2.0 * half_height * focus_distance,
            u: u,
            v: v,
            w: w,
            lens_radius: aperture / 2.0,
        }
    }

    pub fn create_ray(&self, with_lens_focus: bool, x: f32, z: f32) -> Ray {
        let offset = if with_lens_focus {
            let rd = random_in_unit_disk() * self.lens_radius;
            self.u * rd.x + self.v * rd.y
        } else {
            Vector3 { x: 0.0, y: 0.0, z: 0.0 }
        };
        let direction = (self.screen_dl + self.horizontal * x + self.vertical * z - self.origin - offset).normalized();
        Ray::new(self.origin + offset, direction)
    }
}
