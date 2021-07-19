use crate::material::Material;
use crate::material::Color;
use crate::ray::Ray;
use crate::vector3::Vector3;

#[derive(Default)]
pub struct Intersection {
    pub t: f32,
    pub hit_point: Vector3,
    pub hit_normal: Vector3,
}

impl Intersection {
    pub fn new(t: f32, hit_point: Vector3, hit_normal: Vector3) -> Self {
        Intersection {
            t: t,
            hit_point: hit_point,
            hit_normal: hit_normal,
        }
    }
}

pub enum Shape {
    Plane(Vector3, Vector3, Material),
}

fn intersect_plane(origin: &Vector3, normal: &Vector3, ray: &Ray) -> Option<Intersection> {
    let Ray { origin: ray_origin, direction: ray_direction } = ray;
    let denominator = normal.inner_product(&ray_direction);
    if denominator.abs() < 1e-4 {
        return None;
    }
    let plane = origin - ray_origin;
    let t = plane.inner_product(normal) / denominator;

    if t < 1e-4 {
        return None;
    }

    let hit_point = ray_origin + ray_direction * t;
    let hit_normal = if denominator < 0.0 {
        *normal
    } else {
        normal * -1.0
    };
    return Some(Intersection::new(t, hit_point, hit_normal));
}

impl Shape {
    pub fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        match self {
            Shape::Plane(origin, normal, _) => intersect_plane(origin, normal, ray),
        }
    }

    pub fn material(&self) -> Material {
        match self {
            Shape::Plane(_, _, material) => *material
        }
    }
}

