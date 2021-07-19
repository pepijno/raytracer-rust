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
    Plane(Vector3, Vector3),
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
            Shape::Plane(origin, normal) => intersect_plane(origin, normal, ray),
        }
    }
}

pub fn intersect_any<'a>(shapes: &'a Vec<Shape>, ray: &Ray) -> Option<(&'a Shape, Intersection)> {
    let mut min_t: f32 = std::f32::MAX;
    let mut intersection: Intersection = Default::default();
    let mut shape: Option<&Shape> = None;
    for s in shapes {
        let result = s.intersect(ray);
        if let Some(i) = result {
            if i.t < min_t {
                intersection = i;
                min_t = intersection.t;
                shape = Some(s);
            }
        }
    }
    match shape {
        Some(s) => Some((s, intersection)),
        None => None,
    }
}
