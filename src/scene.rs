use rand::prelude::*;
use rand::distributions::WeightedIndex;

use crate::material::*;
use crate::ray::*;
use crate::shape::*;
use crate::vector3::*;
use crate::photonmap::*;

const MAX_DEPTH: u8 = 6;
// const N_SHADOW_RAY: u8 = 5;

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
    objects: Vec<Object>,
    lights: Vec<Light>,
}

#[derive(PartialEq)]
pub enum BounceType {
    NONE,
    DIFFUSE,
    SPECULAR,
}

impl Scene {
    pub fn new(objects: Vec<Object>, lights: Vec<Light>) -> Self {
        Self {
            objects: objects,
            lights: lights,
        }
    }

    fn direct_illumination(&self, ray: &Ray, intersection: &Intersection, material: &Material) -> (Color, Color) {
        // ambient light
        let mut total_diffuse_color = Color::black();
        let mut total_specular_color = Color::black();

        for Light { position, color, intensity: _ } in &self.lights {
            let light_dir = (position - intersection.hit_point).normalized();
            let int = Object::intersect_any(&self.objects, &Ray::new(intersection.hit_point + light_dir * 0.0001, light_dir));
            if let Some((_, i)) = int {
                if (i.hit_point - intersection.hit_point).length_squared() < (position - intersection.hit_point).length_squared() {
                    continue;
                }
            }
            // do something with attenuation
            total_diffuse_color += color * material.diffuse_color * intersection.hit_normal.inner_product(&light_dir).max(0.0);
            total_specular_color += color * (-(light_dir * -1.0).reflect(&intersection.hit_normal).inner_product(&ray.direction)).max(0.0).powf(material.specular_exponent);
        }

        return (total_diffuse_color, total_specular_color);
    }

    fn global_illumination(&self, photon_map: &mut PhotonMap, hit_point: &Vector3, hit_normal: &Vector3) -> Color {
        return photon_map.irradiance_estimate(hit_point, hit_normal);
    }

    pub fn trace_ray(&self, photon_map_global: &mut PhotonMap, photon_map_caustic: &mut PhotonMap, ray: &Ray, depth: u8) -> Color {
        if depth >= MAX_DEPTH {
            return Color::black();
        }

        let intersection = Object::intersect_any(&self.objects, ray);

        match intersection {
            Some((material, int)) => {
                let Material { refractive_index, diffuse_color, reflect_color, specular_exponent: _ } = material;

                if refractive_index == 0.0 {
                    let (direct_color, specular_color) = self.direct_illumination(&ray, &int, &material);
                    let global_color = self.global_illumination(photon_map_global, &int.hit_point, &int.hit_normal);
                    let caustic_color = self.global_illumination(photon_map_caustic, &int.hit_point, &int.hit_normal);
                    let reflected_color = if reflect_color.max() > 0.0 {
                        let reflect_dir = ray.direction.reflect(&int.hit_normal).normalized();
                        let reflect_ray = Ray::new(&int.hit_point + 0.0001 * reflect_dir, reflect_dir);
                        reflect_color * self.trace_ray(photon_map_global, photon_map_caustic, &reflect_ray, depth + 1)
                    } else {
                        Color::black()
                    };
                    return direct_color + reflected_color + specular_color + diffuse_color * (global_color + caustic_color);
                } else {
                    let reflect_dir = ray.direction.reflect(&int.hit_normal).normalized();
                    let reflect_ray = Ray::new(&int.hit_point + 0.0001 * reflect_dir, reflect_dir);

                    let nt: f32;
                    let c: f32;
                    let mut t = Vector3::new(0.0, 0.0, 0.0);
                    if ray.direction.inner_product(&int.hit_normal) < 0.0 {
                        let n = 1.0;
                        nt = refractive_index;
                        if let Some(v) = refract(&ray.direction, &int.hit_normal, n, nt) {
                            t = v;
                        }
                        c = -1.0 * ray.direction.inner_product(&int.hit_normal);
                    } else {
                        let n = refractive_index;
                        nt = 1.0;
                        if let Some(v) = refract(&ray.direction, &(int.hit_normal * -1.0), n, nt) {
                            c = v.normalized().inner_product(&int.hit_normal);
                            t = v;
                        } else {
                            return Color::white() * self.trace_ray(photon_map_global, photon_map_caustic, &reflect_ray, depth + 1);
                        }
                    }

                    let r0 = ((nt - 1.0)/(nt + 1.0)).powf(2.0);
                    let r = r0 + (1.0 - r0) * (1.0 - c).powf(5.0);
                    let refract_ray = Ray::new(&int.hit_point + 0.0001 * t.normalized(), t.normalized());

                    return Color::white() * (r * self.trace_ray(photon_map_global, photon_map_caustic, &reflect_ray, depth + 1)
                                             + (1.0 - r) * self.trace_ray(photon_map_global, photon_map_caustic, &refract_ray, depth + 1));
                }
            },
            None => Color::black(),
        }
    }

     pub fn random_photon_ray(&self, n_photons: usize) -> (Ray, Color) {
        let weights: Vec<f32> = self.lights.iter()
            .map(|l| l.intensity)
            .collect();
        let dist = WeightedIndex::new(&weights).unwrap();

        let mut rng = rand::thread_rng();
        let index = dist.sample(&mut rng);
        let light = &self.lights[index];

        let dir = random_in_sphere();

        (Ray::new(light.position, dir), light.intensity * light.color / (n_photons as f32))
    }

    pub fn trace_photon(&self, photon_map_global: &mut PhotonMap, photon_map_caustic: &mut PhotonMap, ray: &Ray, color: Color, depth: u8, bounce_type: BounceType) -> () {
        if depth >= MAX_DEPTH {
            return;
        }

        let intersection = Object::intersect_any(&self.objects, ray);

        match intersection {
            Some((material, int)) => {
                let Material { refractive_index, diffuse_color, reflect_color, specular_exponent: _ } = material;

                let mut bounce = BounceType::NONE;
                let mut rng = rand::thread_rng();
                if refractive_index == 0.0 {
                    let (p_diffuse, p_specular) = (reflect_probability(&diffuse_color, &color), reflect_probability(&reflect_color, &color));

                    let mut absorb = false;
                    let mut reflected_photon_color = Color::black();
                    let mut reflect_ray = Ray::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 0.0));
                    let r = rng.gen::<f32>();

                    if r >= 0.0 && r < p_diffuse {
                        let random_vector = random_in_hemisphere();
                        let (nx, ny, nz) = int.hit_normal.create_coord_system();
                        let adjusted_vector = Vector3 {
                            x: random_vector.x * nz.x + random_vector.y * nx.x + random_vector.z * ny.x,
                            y: random_vector.x * nz.y + random_vector.y * nx.y + random_vector.z * ny.y,
                            z: random_vector.x * nz.z + random_vector.y * nx.z + random_vector.z * ny.z,
                        };
                        reflect_ray = Ray::new(&int.hit_point + 0.0001 * adjusted_vector, adjusted_vector);
                        reflected_photon_color = color * diffuse_color / p_diffuse;
                        bounce = BounceType::DIFFUSE;
                    } else if r >= p_diffuse && r < (p_diffuse + p_specular) {
                        let reflect_dir = ray.direction.reflect(&int.hit_normal).normalized();
                        reflect_ray = Ray::new(&int.hit_point + 0.0001 * reflect_dir, reflect_dir);
                        reflected_photon_color = color * reflect_color / p_specular;
                        bounce = BounceType::SPECULAR;
                    }

                    if r >= (p_specular + p_diffuse) {
                        absorb = true;
                    }

                    if depth != 0 {
                        if bounce_type == BounceType::DIFFUSE {
                            photon_map_global.store(&int.hit_point, &(ray.direction * -1.0).normalized(), &color);
                        } else if bounce_type == BounceType::SPECULAR {
                            photon_map_caustic.store(&int.hit_point, &(ray.direction * -1.0).normalized(), &color);
                        }
                    }

                    if absorb {
                        return;
                    } else {
                        return self.trace_photon(photon_map_global, photon_map_caustic, &reflect_ray, reflected_photon_color, depth + 1, bounce);
                    }
                } else {
                    let reflect_dir = ray.direction.reflect(&int.hit_normal).normalized();
                    let reflect_ray = Ray::new(&int.hit_point + 0.0001 * reflect_dir, reflect_dir);

                    let n: f32;
                    let nt: f32;
                    let c: f32;
                    let mut t = Vector3::new(0.0, 0.0, 0.0);
                    if ray.direction.inner_product(&int.hit_normal) < 0.0 {
                        n = 1.0;
                        nt = refractive_index;
                        if let Some(v) = refract(&ray.direction, &int.hit_normal, n, nt) {
                            t = v;
                        }
                        c = -1.0 * ray.direction.inner_product(&int.hit_normal);
                    } else {
                        n = refractive_index;
                        nt = 1.0;
                        if let Some(v) = refract(&ray.direction, &(int.hit_normal * -1.0), n, nt) {
                            c = v.normalized().inner_product(&int.hit_normal);
                            t = v;
                        } else {
                            return self.trace_photon(photon_map_global, photon_map_caustic, &reflect_ray, color, depth + 1, BounceType::SPECULAR);
                        }
                    }

                    let r0 = ((nt - 1.0)/(nt + 1.0)).powf(2.0);
                    let r = r0 + (1.0 - r0) * (1.0 - c).powf(5.0);
                    let refract_ray = Ray::new(&int.hit_point + 0.0001 * t.normalized(), t.normalized());

                    if rng.gen::<f32>() < r {
                        return self.trace_photon(photon_map_global, photon_map_caustic, &reflect_ray, color, depth + 1, BounceType::SPECULAR);
                    } else {
                        return self.trace_photon(photon_map_global, photon_map_caustic, &refract_ray, color, depth + 1, BounceType::SPECULAR);
                    }
                }
            },
            None => return,
        }
    }
}

fn refract(direction: &Vector3, normal: &Vector3, n: f32, nt: f32) -> Option<Vector3> {
    let dn = direction.inner_product(normal);
    let sq_rt = 1.0 - (n * n * (1.0 - (dn * dn))) / (nt * nt);

    if sq_rt < 0.0 {
        return None;
    } else {
        return Some((n * (direction - normal * dn)) / nt - normal * sq_rt.sqrt());
    }
}

fn reflect_probability(coefficient: &Color, power: &Color) -> f32 {
    let numerator = (coefficient.r * power.r).max(coefficient.g * power.g).max(coefficient.b * power.b);
    let denominator = power.r.max(power.g).max(power.b);
    numerator / denominator
}
