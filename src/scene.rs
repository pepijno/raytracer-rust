use crate::material::Color;
use crate::material::Material;
use crate::ray::Ray;
use crate::intersect::Shape;
use crate::vector3::Vector3;
use crate::intersect::intersect_any;

const MAX_DEPTH: u8 = 5;

pub struct Scene {
    objects: Vec<Shape>,
    lights: Vec<Vector3>,
}

impl Scene {
    pub fn new(objects: Vec<Shape>, lights: Vec<Vector3>) -> Self {
        Self {
            objects: objects,
            lights: lights,
        }
    }

    pub fn trace_ray(&self, ray: &Ray, depth: u8) -> Color {
        if depth >= MAX_DEPTH {
            return Color::black();
        }

        let intersection = intersect_any(&self.objects, ray);

        match intersection {
            Some((object, int)) => {
                let Material { refractive_index, albedo, diffuse_color, specular_exponent } = object.material();
                let reflect_dir = ray.direction.reflect(&int.hit_normal).normalized();
                let refract_dir = ray.direction.refract(&int.hit_normal, refractive_index, 1.0).normalized();

                let reflect_color = self.trace_ray(&Ray::new(int.hit_point + reflect_dir * 0.0001, reflect_dir), depth + 1);
                let refract_color = self.trace_ray(&Ray::new(int.hit_point + refract_dir * 0.0001, refract_dir), depth + 1);

                let mut diffuse_light_intensity = 0.0;
                let mut specular_light_intensity = 0.0;
                for light in &self.lights {
                    let light_dir = (light - int.hit_point).normalized();
                    let i = intersect_any(&self.objects, &Ray::new(int.hit_point + light_dir * 0.0001, light_dir));
                    if let Some((_, ix)) = i {
                        if (ix.hit_point - int.hit_point).length_squared() < (light - int.hit_point).length_squared() {
                            continue;
                        }
                    }
                    diffuse_light_intensity += (1.0 * light_dir.inner_product(&int.hit_normal)).max(0.0);
                    specular_light_intensity += ((-(light_dir * -1.0).reflect(&int.hit_normal).inner_product(&ray.direction)).max(0.0).powf(specular_exponent)) * 1.0;
                }
                return diffuse_color * diffuse_light_intensity * albedo[0] + Color::singular(1.0) * specular_light_intensity * albedo[1] + reflect_color * albedo[2] + refract_color * albedo[3];
            },
            None => Color::black(),
        }
    }
}
