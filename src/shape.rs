use crate::material::*;
use crate::ray::Ray;
use crate::vector3::Vector3;
use std::fmt;
use std::cmp::Ordering;

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

impl fmt::Display for Intersection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Intersection({}, {}, {})", self.t, self.hit_point, self.hit_normal)
    }
}

#[derive(Clone)]
pub struct Object {
    pub shape: Shape,
    pub material: Material,
}

impl Object {
    pub fn new(shape: Shape, material: Material) -> Self {
        Object {
            shape: shape,
            material: material
        }
    }

    pub fn intersect_any(objects: &Vec<Object>, ray: &Ray) -> Option<(Material, Intersection)> {
        let mut min_t: f32 = std::f32::MAX;
        let mut intersection: Intersection = Default::default();
        let mut material: Option<&Material> = None;
        for object in objects {
            let result = object.shape.intersect(ray);
            if let Some(i) = result {
                if i.t < min_t {
                    intersection = i;
                    min_t = intersection.t;
                    material = Some(&object.material);
                }
            }
        }
        match material {
            Some(m) => Some((*m, intersection)),
            None => None,
        }
    }
}

#[derive(Clone)]
pub enum Shape {
    Plane(Vector3, Vector3),
    Sphere(Vector3, f32),
    Triangle(Vector3, Vector3, Vector3),
    Pyramid(Vector3, Vector3, Vector3, Vector3),
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
        tca + thc
    } else {
        tca - thc
    };
    let hit_point = ray_origin + ray_direction * t;
    let normal = (hit_point - origin).normalized();
    return Some(Intersection::new(t, hit_point, normal));
}

fn intersect_triangle(p1: &Vector3, p2: &Vector3, p3: &Vector3, ray: &Ray) -> Option<Intersection> {
    let n1 = p2 - p1;
    let n2 = p3 - p1;
    let p_vec = ray.direction.outer_product(&n2);
    let determinant = n1.inner_product(&p_vec);

    if determinant.abs() < 0.0001 {
        return None;
    }

    let inverse_determinant = 1.0 / determinant;

    let t_vec = ray.origin - p1;
    let u = t_vec.inner_product(&p_vec) * inverse_determinant;
    if u < 0.0 || u > 1.0 {
        return None;
    }

    let q_vec = t_vec.outer_product(&n1);
    let v = ray.direction.inner_product(&q_vec) * inverse_determinant;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    let t = n2.inner_product(&q_vec) * inverse_determinant;
    if t < 0.0 {
        return None;
    }

    let hit_point = ray.origin + ray.direction * t;
    return Some(Intersection::new(t, hit_point, n1.outer_product(&n2).normalized()));
}

fn intersect_pyramid(p1: &Vector3, p2: &Vector3, p3: &Vector3, p4: &Vector3, ray: &Ray) -> Option<Intersection> {
    let intersections = vec!((p4, intersect_triangle(p1, p2, p3, ray)), (p2, intersect_triangle(p1, p3, p4, ray)), (p1, intersect_triangle(p2, p4, p3, ray)), (p3, intersect_triangle(p1, p2, p4, ray)));
    let mut hits: Vec<(&Vector3, Intersection)> = intersections.into_iter()
        .filter(|x| x.1.is_some())
        .map(|x| (x.0, x.1.unwrap()))
        .collect();

    hits.sort_by(|a, b| a.1.t.partial_cmp(&b.1.t).unwrap_or(Ordering::Equal));

    return hits.first()
        .map(|(&p, intersection)| {
            let Intersection { t, hit_point, hit_normal } = intersection;
            let inner_dir = p - hit_point;
            let normal = if ray.direction.inner_product(&inner_dir) < 0.0 && hit_normal.inner_product(&inner_dir) > 0.0 {
                hit_normal * -1.0
            } else {
                *hit_normal
            };
            Intersection::new(*t, *hit_point, normal)
        });
}

impl Shape {
    pub fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        match self {
            Shape::Plane(origin, normal) => intersect_plane(origin, normal, ray),
            Shape::Sphere(origin, radius) => intersect_sphere(origin, *radius, ray),
            Shape::Triangle(p1, p2, p3) => intersect_triangle(p1, p2, p3, ray),
            Shape::Pyramid(p1, p2, p3, p4) => intersect_pyramid(p1, p2, p3, p4, ray),
        }
    }
}

