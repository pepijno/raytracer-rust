use crate::material::Color;
use crate::material::Material;
use crate::ray::Ray;
use crate::shape::Shape;
use crate::shape::Intersection;
use crate::vector3::Vector3;
use crate::vector3::random_in_hemisphere;
use std::f32::consts::PI;

const MAX_DEPTH: u8 = 5;
const MONTE_CARLO_RAYS: u8 = 32;

#[derive(Clone)]
pub struct Light {
    position: Vector3,
    color: Color,
    intensity: f32,
}

impl Light {
    pub fn new(position: &Vector3, color: Color, intensity: f32) -> Self {
        Light {
            position: *position,
            color: color,
            intensity: intensity,
        }
    }
}

#[derive(Clone)]
pub struct Scene {
    objects: Vec<Shape>,
    lights: Vec<Light>,
}

impl Scene {
    pub fn new(objects: Vec<Shape>, lights: Vec<Light>) -> Self {
        Self {
            objects: objects,
            lights: lights,
        }
    }

    pub fn trace_ray(&self, ray: &Ray, depth: u8) -> Color {
        if depth >= MAX_DEPTH {
            return Color::black();
        }

        let intersection = self.intersect_any(ray);

        match intersection {
            Some((object, int)) => {
                let Material { refractive_index, albedo, diffuse_color, specular_exponent } = object.material();

                let reflect_color = if albedo[2] > 0.0001 {
                    let reflect_dir = ray.direction.reflect(&int.hit_normal).normalized();
                    let color = self.trace_ray(&Ray::new(int.hit_point + reflect_dir * 0.0001, reflect_dir), depth + 1);
                    if refractive_index != 1.0 {
                        let mut refract_color = Color::black();
                        let kr = fresnel(ray.direction, int.hit_normal, refractive_index);
                        let outside = ray.direction.inner_product(&int.hit_normal) < 0.0;
                        if kr < 1.0 {
                            let refract_dir = ray.direction.refract(&int.hit_normal, refractive_index, 1.0).normalized();
                            refract_color = self.trace_ray(&Ray::new(int.hit_point + refract_dir * 0.0001, refract_dir), depth + 1)
                        }
                        color * kr + refract_color * (1.0 - kr)
                    } else {
                        color
                    }
                } else {
                    Color::black()
                };

                let mut diffuse_light = Color::black();
                let mut specular_light = Color::black();
                for light in &self.lights {
                    let light_dir = (light.position - int.hit_point).normalized();
                    let i = self.intersect_any(&Ray::new(int.hit_point + light_dir * 0.0001, light_dir));
                    if let Some((_, ix)) = i {
                        if (ix.hit_point - int.hit_point).length_squared() < (light.position - int.hit_point).length_squared() {
                            continue;
                        }
                    }
                    diffuse_light = diffuse_light + (light.intensity * light_dir.inner_product(&int.hit_normal)).max(0.0) * light.color;
                    specular_light = specular_light + ((-(light_dir * -1.0).reflect(&int.hit_normal).inner_product(&ray.direction)).max(0.0).powf(specular_exponent)) * light.intensity * light.color;
                }

                let mut diffuse = diffuse_color * diffuse_light;

                if albedo[0] > 0.0001 {
                    let random_vector = random_in_hemisphere();
                    let (nx, ny, nz) = int.hit_normal.create_coord_system();
                    let adjusted_vector = Vector3 {
                        x: random_vector.x * nz.x + random_vector.y * nx.x + random_vector.z * ny.x,
                        y: random_vector.x * nz.y + random_vector.y * nx.y + random_vector.z * ny.y,
                        z: random_vector.x * nz.z + random_vector.y * nx.z + random_vector.z * ny.z,
                    };
                    diffuse = diffuse + self.trace_ray(&Ray::new(int.hit_point + adjusted_vector * 0.0001, adjusted_vector), depth + 1);
                }

                return diffuse * albedo[0] + Color::singular(1.0) * specular_light * albedo[1] + reflect_color * albedo[2];
            },
            None => Color::black(),
        }
    }

    fn intersect_any(&self, ray: &Ray) -> Option<(&Shape, Intersection)> {
        let mut min_t: f32 = std::f32::MAX;
        let mut intersection: Intersection = Default::default();
        let mut shape: Option<&Shape> = None;
        for object in &self.objects {
            let result = object.intersect(ray);
            if let Some(i) = result {
                if i.t < min_t {
                    intersection = i;
                    min_t = intersection.t;
                    shape = Some(&object);
                }
            }
        }
        match shape {
            Some(s) => Some((s, intersection)),
            None => None,
        }
    }
}

fn fresnel(direction: Vector3, normal: Vector3, ior: f32) -> f32 {
    let mut cosi = direction.inner_product(&normal).max(-1.0).min(1.0);
    let (mut eta_i, mut eta_t) = (1.0, ior);
    if cosi >= 0.0 {
        let x = eta_i;
        eta_i = eta_t;
        eta_t = x;
    }
    let sint = (eta_i / eta_t) * (1.0 - cosi * cosi).max(0.0).sqrt();
    if sint >= 1.0 {
        return 1.0;
    } else {
        let cost = (1.0 - sint * sint).max(0.0).sqrt();
        cosi = cosi.abs();
        let rs = ((eta_t * cosi) - (eta_i * cost)) / ((eta_t * cosi) + (eta_i * cost));
        let rp = ((eta_i * cosi) - (eta_t * cost)) / ((eta_i * cosi) + (eta_t * cost));
        return (rs * rs + rp * rp) / 2.0;
    }
}
