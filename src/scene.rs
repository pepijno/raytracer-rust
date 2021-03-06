use crate::material::{Color, Material};
use crate::objects::{Light, Object};
use crate::ray::{Intersection, Ray};
use crate::vector3::Vector3;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use kdtree::KdTree;
use kdtree::distance::squared_euclidean;
use core::f32::consts::PI;

const MAX_DEPTH: u8 = 6;
const N_PHOTON_RADIANCE: usize = 400;

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

#[derive(Debug)]
pub struct Photon {
    position: Vector3,
    direction: Vector3,
    power: Color,
}

impl Scene {
    pub fn new(objects: Vec<Object>, lights: Vec<Light>) -> Self {
        Self { objects, lights }
    }

    fn direct_illumination(
        &self,
        ray: &Ray,
        intersection: &Intersection,
        material: &Material,
    ) -> (Color, Color) {
        // ambient light
        let mut total_diffuse_color = Color::black();
        let mut total_specular_color = Color::black();

        for Light {
            position,
            color,
            intensity: _,
        } in &self.lights
        {
            let light_dir = (position - intersection.hit_point).normalized();
            let r = Ray::new(intersection.hit_point, light_dir);
            let int = r.intersect_any(&self.objects);
            if let Some(i) = int {
                if (i.hit_point - intersection.hit_point).length_squared()
                    < (position - intersection.hit_point).length_squared()
                {
                    continue;
                }
            }
            // do something with attenuation
            total_diffuse_color += color
                * material.diffuse_color
                * intersection.hit_normal.inner_product(light_dir).max(0.0);
            total_specular_color += color
                * (-(light_dir * -1.0)
                    .reflect(intersection.hit_normal)
                    .inner_product(ray.direction))
                .max(0.0)
                .powf(material.specular_exponent);
        }

        return (total_diffuse_color, total_specular_color);
    }

    fn global_illumination(
        &self,
        photon_map: &KdTree<f32, Photon, [f32; 3]>,
        hit_point: Vector3,
        hit_normal: Vector3,
    ) -> Color {
        let mut result = Color::black();
        let res = photon_map.nearest(&hit_point.to_array(), N_PHOTON_RADIANCE, &squared_euclidean).unwrap();

        if res.len() == 0 {
            return result;
        }
        let mut max_distance_squared = 0.0;
        for i in 0..res.len() {
            let photon = res[i].1;
            result += photon.power * hit_normal.inner_product(photon.direction).max(0.0);
            if res[i].0 > max_distance_squared {
                max_distance_squared = res[i].0;
            }
        }

        return result * (1.0 / (max_distance_squared * PI));
        // heap.reset();
        // return photon_map.irradiance_estimate(heap, hit_point, hit_normal);
    }

    pub fn trace_ray(
        &self,
        photon_map_global: &KdTree<f32, Photon, [f32; 3]>,
        photon_map_caustic: &KdTree<f32, Photon, [f32; 3]>,
        ray: &Ray,
        depth: u8,
    ) -> Color {
        if depth >= MAX_DEPTH {
            return Color::black();
        }

        let intersection = ray.intersect_any(&self.objects);

        match intersection {
            Some(int) => {
                let Material {
                    refractive_index,
                    diffuse_color,
                    reflect_color,
                    specular_exponent: _,
                } = int.material;

                if refractive_index == 0.0 {
                    let (direct_color, specular_color) =
                        self.direct_illumination(&ray, &int, &int.material);
                    let global_color = self.global_illumination(
                        photon_map_global,
                        int.hit_point,
                        int.hit_normal,
                    );
                    let caustic_color = self.global_illumination(
                        photon_map_caustic,
                        int.hit_point,
                        int.hit_normal,
                    );
                    let reflected_color = if reflect_color.max() > 0.0 {
                        let reflect_ray = ray.reflect(int.hit_point, int.hit_normal);
                        reflect_color
                            * self.trace_ray(
                                photon_map_global,
                                photon_map_caustic,
                                &reflect_ray,
                                depth + 1,
                            )
                    } else {
                        Color::black()
                    };
                    return direct_color
                        + reflected_color
                        + specular_color
                        + diffuse_color * (global_color + caustic_color);
                } else {
                    let reflect_ray = ray.reflect(int.hit_point, int.hit_normal);

                    let nt: f32;
                    let c: f32;
                    let mut t = Vector3::new(0.0, 0.0, 0.0);
                    if ray.direction.inner_product(int.hit_normal) < 0.0 {
                        let n = 1.0;
                        nt = refractive_index;
                        if let Some(v) = refract(ray.direction, int.hit_normal, n, nt) {
                            t = v;
                        }
                        c = -1.0 * ray.direction.inner_product(int.hit_normal);
                    } else {
                        let n = refractive_index;
                        nt = 1.0;
                        if let Some(v) = refract(ray.direction, int.hit_normal * -1.0, n, nt) {
                            c = v.normalized().inner_product(int.hit_normal);
                            t = v;
                        } else {
                            return Color::white()
                                * self.trace_ray(
                                    photon_map_global,
                                    photon_map_caustic,
                                    &reflect_ray,
                                    depth + 1,
                                );
                        }
                    }

                    let r0 = ((nt - 1.0) / (nt + 1.0)).powf(2.0);
                    let r = r0 + (1.0 - r0) * (1.0 - c).powf(5.0);
                    let refract_ray = Ray {
                        origin: &int.hit_point + 0.0001 * t.normalized(),
                        direction: t.normalized(),
                    };

                    return Color::white()
                        * (r * self.trace_ray(
                            photon_map_global,
                            photon_map_caustic,
                            &reflect_ray,
                            depth + 1,
                        ) + (1.0 - r)
                            * self.trace_ray(
                                photon_map_global,
                                photon_map_caustic,
                                &refract_ray,
                                depth + 1,
                            ));
                }
            }
            None => Color::black(),
        }
    }

    pub fn random_photon_ray(&self, n_photons: usize) -> (Ray, Color) {
        let weights: Vec<f32> = self.lights.iter().map(|l| l.intensity).collect();
        let dist = WeightedIndex::new(&weights).unwrap();

        let mut rng = rand::thread_rng();
        let index = dist.sample(&mut rng);
        let light = &self.lights[index];

        (
            Ray::random_ray(light.position),
            light.intensity * light.color / (n_photons as f32),
        )
    }

    pub fn trace_photon(
        &self,
        photon_map_global: &mut KdTree<f32, Photon, [f32; 3]>,
        photon_map_caustic: &mut KdTree<f32, Photon, [f32; 3]>,
        ray: &Ray,
        color: Color,
        depth: u8,
        bounce_type: BounceType,
    ) -> () {
        if depth >= MAX_DEPTH {
            return;
        }

        let intersection = ray.intersect_any(&self.objects);

        match intersection {
            Some(int) => {
                let Material {
                    refractive_index,
                    diffuse_color,
                    reflect_color,
                    specular_exponent: _,
                } = int.material;

                let mut bounce = BounceType::NONE;
                let mut rng = rand::thread_rng();
                if refractive_index == 0.0 {
                    let p_reflect = (diffuse_color + reflect_color).max();
                    let p_diffuse = diffuse_color.sum()
                        / (diffuse_color.sum() + reflect_color.sum())
                        * p_reflect;
                    let p_specular = reflect_color.sum()
                        / (diffuse_color.sum() + reflect_color.sum())
                        * p_reflect;

                    let mut absorb = false;
                    let mut reflected_photon_color = Color::black();
                    let mut reflect_ray = Ray {
                        origin: Vector3::new(0.0, 0.0, 0.0),
                        direction: Vector3::new(0.0, 0.0, 0.0),
                    };
                    let r = rng.gen::<f32>();

                    if r >= 0.0 && r < p_diffuse {
                        reflect_ray = Ray::random_ray_in_hemisphere(int.hit_point, int.hit_normal);
                        reflected_photon_color = color * diffuse_color / p_diffuse;
                        bounce = BounceType::DIFFUSE;
                    } else if r >= p_diffuse && r < (p_diffuse + p_specular) {
                        reflect_ray = ray.reflect(int.hit_point, int.hit_normal);
                        reflected_photon_color = color * reflect_color / p_specular;
                        bounce = BounceType::SPECULAR;
                    }

                    if r >= (p_specular + p_diffuse) {
                        absorb = true;
                    }

                    if depth != 0 {
                        if bounce_type == BounceType::DIFFUSE {
                            let photon = Photon {
                                position: int.hit_point,
                                direction: (ray.direction * -1.0).normalized(),
                                power: color,
                            };
                        } else if bounce_type == BounceType::SPECULAR {
                            let photon = Photon {
                                position: int.hit_point,
                                direction: (ray.direction * -1.0).normalized(),
                                power: color,
                            };
                            photon_map_caustic.add(int.hit_point.to_array(), photon).unwrap();
                        }
                    }

                    if absorb {
                        return;
                    } else {
                        return self.trace_photon(
                            photon_map_global,
                            photon_map_caustic,
                            &reflect_ray,
                            reflected_photon_color,
                            depth + 1,
                            bounce,
                        );
                    }
                } else {
                    let reflect_ray = ray.reflect(int.hit_point, int.hit_normal);

                    let n: f32;
                    let nt: f32;
                    let c: f32;
                    let mut t = Vector3::new(0.0, 0.0, 0.0);
                    if ray.direction.inner_product(int.hit_normal) < 0.0 {
                        n = 1.0;
                        nt = refractive_index;
                        if let Some(v) = refract(ray.direction, int.hit_normal, n, nt) {
                            t = v;
                        }
                        c = -1.0 * ray.direction.inner_product(int.hit_normal);
                    } else {
                        n = refractive_index;
                        nt = 1.0;
                        if let Some(v) = refract(ray.direction, int.hit_normal * -1.0, n, nt) {
                            c = v.normalized().inner_product(int.hit_normal);
                            t = v;
                        } else {
                            return self.trace_photon(
                                photon_map_global,
                                photon_map_caustic,
                                &reflect_ray,
                                color,
                                depth + 1,
                                BounceType::SPECULAR,
                            );
                        }
                    }

                    let r0 = ((nt - 1.0) / (nt + 1.0)).powf(2.0);
                    let r = r0 + (1.0 - r0) * (1.0 - c).powf(5.0);
                    let refract_ray = Ray {
                        origin: &int.hit_point + 0.0001 * t.normalized(),
                        direction: t.normalized(),
                    };

                    if rng.gen::<f32>() < r {
                        return self.trace_photon(
                            photon_map_global,
                            photon_map_caustic,
                            &reflect_ray,
                            color,
                            depth + 1,
                            BounceType::SPECULAR,
                        );
                    } else {
                        return self.trace_photon(
                            photon_map_global,
                            photon_map_caustic,
                            &refract_ray,
                            color,
                            depth + 1,
                            BounceType::SPECULAR,
                        );
                    }
                }
            }
            None => return,
        }
    }
}

fn refract(direction: Vector3, normal: Vector3, n: f32, nt: f32) -> Option<Vector3> {
    let dn = direction.inner_product(normal);
    let sq_rt = 1.0 - (n * n * (1.0 - (dn * dn))) / (nt * nt);

    if sq_rt < 0.0 {
        return None;
    } else {
        return Some((n * (direction - normal * dn)) / nt - normal * sq_rt.sqrt());
    }
}
