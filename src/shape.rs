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

#[derive(Clone)]
pub enum Shape {
    Plane(Vector3, Vector3, Material),
    Sphere(Vector3, f32, Material),
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

fn intersect_sphere(origin: &Vector3, radius: f32, ray: &Ray) -> Option<Intersection> {
    let Ray { origin: ray_origin, direction: ray_direction } = ray;
    let v = origin - ray_origin;

    let tca = v.inner_product(&ray_direction);
    if tca < 0.0 {
        return None;
    }

    let d2 = v.length_squared() - tca * tca;
    if d2 > radius * radius {
        return None;
    }

    let thc = (radius * radius - d2).sqrt();
    let t = if tca - thc < 0.0 {
        tca - thc
    } else {
        tca + thc
    };
    let hit_point = ray_origin + ray_direction * t;
    let normal = (hit_point - origin).normalized();
    return Some(Intersection::new(t, hit_point, normal));
}

impl Shape {
    pub fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        match self {
            Shape::Plane(origin, normal, _) => intersect_plane(origin, normal, ray),
            Shape::Sphere(origin, radius, _) => intersect_sphere(origin, *radius, ray),
        }
    }

    pub fn material(&self) -> Material {
        match self {
            Shape::Plane(_, _, material) => *material,
            Shape::Sphere(_, _, material) => *material,
        }
    }
}

