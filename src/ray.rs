use crate::material::Material;
use crate::objects::{Object, Plane, Pyramid, Shape, Sphere, Triangle};
use crate::vector3::Vector3;
use std::cmp::Ordering;

const BIAS: f32 = 0.0001;

#[derive(Default, Copy, Clone, Debug)]
pub struct Ray {
    pub origin: Vector3,
    pub direction: Vector3,
}

pub struct Intersection {
    pub t: f32,
    pub hit_point: Vector3,
    pub hit_normal: Vector3,
    pub material: Material,
}

impl Ray {
    pub fn new(origin: Vector3, direction: Vector3) -> Self {
        Self {
            origin: origin + direction * BIAS,
            direction,
        }
    }

    pub fn intersect_any(&self, objects: &Vec<Object>) -> Option<Intersection> {
        let mut intersections = objects
            .into_iter()
            .filter_map(|object| self.intersect(&object))
            .collect::<Vec<_>>();

        intersections.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap_or(Ordering::Equal));

        intersections.first().map(|i| Intersection {
            t: i.t,
            hit_point: i.hit_point,
            hit_normal: i.hit_normal,
            material: i.material,
        })
    }

    pub fn reflect(&self, point: Vector3, normal: Vector3) -> Self {
        let direction = self.direction.reflect(normal).normalized();
        Self::new(point, direction)
    }

    pub fn random_ray(origin: Vector3) -> Self {
        Self {
            origin,
            direction: Vector3::random_in_sphere(),
        }
    }

    pub fn random_ray_in_hemisphere(origin: Vector3, normal: Vector3) -> Self {
        let random_vector = Vector3::random_in_hemisphere();
        let (nx, ny, nz) = normal.create_coord_system();
        let adjusted_vector = Vector3 {
            x: random_vector.x * nz.x + random_vector.y * nx.x + random_vector.z * ny.x,
            y: random_vector.x * nz.y + random_vector.y * nx.y + random_vector.z * ny.y,
            z: random_vector.x * nz.z + random_vector.y * nx.z + random_vector.z * ny.z,
        };
        Self::new(origin, adjusted_vector)
    }

    fn intersect_plane(&self, plane: &Plane, material: &Material) -> Option<Intersection> {
        let Plane { position, normal } = *plane;

        let denominator = normal.inner_product(self.direction);
        // denominator is small, ray is parallel to plane
        if denominator.abs() < 1e-4 {
            return None;
        }

        let plane = position - self.origin;
        let t = plane.inner_product(normal) / denominator;

        // t is less than 0, intersection is behind the ray origin
        if t < 0.0 {
            return None;
        }

        let hit_point = self.origin + self.direction * t;
        let hit_normal = if denominator < 0.0 {
            normal
        } else {
            normal * -1.0
        };
        Some(Intersection {
            t,
            hit_point,
            hit_normal,
            material: *material,
        })
    }

    fn intersect_sphere(&self, sphere: &Sphere, material: &Material) -> Option<Intersection> {
        let Sphere { origin, radius } = sphere;

        let v = origin - self.origin;

        let tca = v.inner_product(self.direction);
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
        let hit_point = self.origin + self.direction * t;
        let normal = (hit_point - origin).normalized();
        Some(Intersection {
            t,
            hit_point,
            hit_normal: normal,
            material: *material,
        })
    }

    fn intersect_triangle(&self, triangle: &Triangle, material: &Material) -> Option<Intersection> {
        let Triangle {
            vertex1,
            vertex2,
            vertex3,
        } = triangle;

        let n1 = vertex2 - vertex1;
        let n2 = vertex3 - vertex1;
        let p_vec = self.direction.outer_product(n2);
        let determinant = n1.inner_product(p_vec);

        if determinant.abs() < 0.0 {
            return None;
        }

        let inverse_determinant = 1.0 / determinant;

        let t_vec = self.origin - vertex1;
        let u = t_vec.inner_product(p_vec) * inverse_determinant;
        if u < 0.0 || u > 1.0 {
            return None;
        }

        let q_vec = t_vec.outer_product(n1);
        let v = self.direction.inner_product(q_vec) * inverse_determinant;
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = n2.inner_product(q_vec) * inverse_determinant;
        if t < 0.0 {
            return None;
        }

        let hit_point = self.origin + self.direction * t;
        Some(Intersection {
            t,
            hit_point,
            hit_normal: n1.outer_product(n2).normalized(),
            material: *material,
        })
    }

    fn intersect_pyramid(&self, pyramid: &Pyramid, material: &Material) -> Option<Intersection> {
        let Pyramid {
            vertex1,
            vertex2,
            vertex3,
            vertex4,
        } = *pyramid;
        let intersections = vec![
            (
                vertex4,
                Triangle {
                    vertex1: vertex1,
                    vertex2: vertex2,
                    vertex3: vertex3,
                },
            ),
            (
                vertex2,
                Triangle {
                    vertex1: vertex1,
                    vertex2: vertex3,
                    vertex3: vertex4,
                },
            ),
            (
                vertex1,
                Triangle {
                    vertex1: vertex2,
                    vertex2: vertex4,
                    vertex3: vertex3,
                },
            ),
            (
                vertex3,
                Triangle {
                    vertex1: vertex1,
                    vertex2: vertex2,
                    vertex3: vertex4,
                },
            ),
        ];
        let mut hits: Vec<(Vector3, Intersection)> = intersections
            .into_iter()
            .map(|x| (x.0, self.intersect_triangle(&x.1, material)))
            .filter(|x| x.1.is_some())
            .map(|x| (x.0, x.1.unwrap()))
            .collect();

        hits.sort_by(|a, b| a.1.t.partial_cmp(&b.1.t).unwrap_or(Ordering::Equal));

        hits.first().map(|(p, intersection)| {
            let Intersection {
                t,
                hit_point,
                hit_normal,
                material: _,
            } = intersection;
            let inner_dir = p - hit_point;
            let normal = if self.direction.inner_product(inner_dir) < 0.0
                && hit_normal.inner_product(inner_dir) > 0.0
            {
                hit_normal * -1.0
            } else {
                *hit_normal
            };
            Intersection {
                t: *t,
                hit_point: *hit_point,
                hit_normal: normal,
                material: *material,
            }
        })
    }

    fn intersect(&self, object: &Object) -> Option<Intersection> {
        match &object.shape {
            Shape::Plane(plane) => self.intersect_plane(&plane, &object.material),
            Shape::Sphere(sphere) => self.intersect_sphere(&sphere, &object.material),
            Shape::Triangle(triangle) => self.intersect_triangle(&triangle, &object.material),
            Shape::Pyramid(pyramid) => self.intersect_pyramid(&pyramid, &object.material),
        }
    }
}
