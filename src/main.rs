extern crate rand_distr;
extern crate rand;

use std::io::Write;
use rand_distr::{Distribution, Uniform};
use rayon::prelude::*;
use std::cmp::Ordering;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::AtomicU32;

use rust_raytracer::vector3::*;
use rust_raytracer::material::*;
use rust_raytracer::camera::*;
use rust_raytracer::scene::*;
use rust_raytracer::shape::*;
use rust_raytracer::photonmap::*;

// const THREAD_COUNT: i32 = 8;
const SAMPLING_AMOUNT: u32 = 1;
const N_PHOTONS: usize = 800000;

const SIZE: i32 = 600;
const ASPECT_RATIO: f32 = 1.66;
const HEIGHT: u32 = (2 * SIZE) as u32;
const WIDTH: u32 = (2.0 * ASPECT_RATIO * (SIZE as f32)) as u32;

fn create_scene() -> Scene {
    let ivory = Material {
        refractive_index: 0.0,
        diffuse_color: Color::new(0.1, 0.1, 0.15),
        reflect_color: Color::new(0.85, 0.85, 0.85),
        specular_exponent: 50.0,
    };
    let ivory2 = Material {
        refractive_index: 0.0,
        diffuse_color: Color::new(0.1, 0.8, 0.1),
        reflect_color: Color::black(),
        specular_exponent: 50.0,
    };
    let ivory3 = Material {
        refractive_index: 0.0,
        diffuse_color: Color::new(0.3, 0.7, 0.7),
        reflect_color: Color::new(0.3, 0.7, 0.7) * 0.2,
        specular_exponent: 50.0,
    };
    let glass = Material {
        refractive_index: 1.5,
        diffuse_color: Color::new(0.6, 0.7, 0.8),
        reflect_color: Color::black(),
        specular_exponent: 125.0,
    };
    let mirror = Material {
        refractive_index: 0.0,
        diffuse_color: Color::black(),
        reflect_color: Color::new(1.0, 1.0, 1.0) * 0.8,
        specular_exponent: 125.0,
    };
    let rubber = Material {
        refractive_index: 0.0,
        diffuse_color: Color::new(0.3, 0.1, 0.1),
        reflect_color: Color::black(),
        specular_exponent: 1000000.0,
    };

    let objects: Vec<Object> = vec![
        Object::new(Shape::Pyramid(Vector3::new(5.0, -0.999, -1.0), Vector3::new(3.0, -0.999, -4.0), Vector3::new(3.0, 2.0, -2.3), Vector3::new(1.0, -0.999, -1.0)), glass),
        // Object::new(Shape::Triangle(Vector3::new(4.0, 0.0, -2.0), Vector3::new(3.5, 0.0, -3.0), Vector3::new(3.5, 1.0, -2.5)), ivory),
        // Object::new(Shape::Triangle(Vector3::new(3.5, 0.0, -3.0), Vector3::new(3.5, 1.0, -2.5), Vector3::new(3.0, 0.0, -2.0)), ivory),
        // Object::new(Shape::Triangle(Vector3::new(4.0, 0.0, -2.0), Vector3::new(3.5, 0.0, -3.0), Vector3::new(3.0, 0.0, -2.0)), ivory),
        Object::new(Shape::Sphere(Vector3::new(-1.0, 0.0, -2.0), 1.0), ivory),
        // Object::new(Shape::Sphere(Vector3::new(1.0, 0.0, -2.0), 1.0), ivory2),
        // Object::new(Shape::Sphere(Vector3::new(2.0, 0.0, -6.0), 1.0), ivory2),
        Object::new(Shape::Sphere(Vector3::new(3.0, 0.0, 0.0), 1.0), glass),
        // Object::new(Shape::Sphere(Vector3::new(-2.0, 2.0, -6.0), 3.0), mirror),
        Object::new(Shape::Plane(Vector3::new(0.0, -1.0, 0.0), Vector3::new(0.0, 1.0, 0.0)), rubber),
        // Object::new(Shape::Plane(Vector3::new(-6.0, -1.0, 0.0), Vector3::new(1.0, 0.0, 0.0)), ivory3),
        Object::new(Shape::Pyramid(Vector3::new(10.0, -0.999, -8.0), Vector3::new(3.5, 12.0, -8.0), Vector3::new(-3.0, -0.999, -8.0), Vector3::new(3.5, -0.999, -15.0)), ivory),
        // Object::new(Shape::Triangle(Vector3::new(10.0, -1.0, -8.0), Vector3::new(3.5, 12.0, -8.0), Vector3::new(-3.0, -1.0, -8.0)), ivory),
        // Object::new(Shape::Plane(Vector3::new(6.0, -1.0, -8.0), Vector3::new(0.0, 0.0, 1.0)), ivory),
    ];

    let lights = vec![
        Light::new(&Vector3::new(5.0, 10.0, -4.0), Color::new(1.0, 1.0, 1.0), 5000.0),
        Light::new(&Vector3::new(-4.0, 12.0, 3.0), Color::new(1.0, 1.0, 1.0), 5000.0),
    ];

    return Scene::new(objects, lights);
}

fn create_camera() -> Camera {
    let origin = Vector3::new(7.0, 2.0, 5.0);
    let look_at = Vector3::new(-1.0, 0.0, -2.0);
    // let look_at = Vector3::new(3.0, 0.0, -2.0);
    let vup = Vector3::new(0.0, -1.0, 0.0);
    let focus_distance = (origin - look_at).length_squared().sqrt();
    let field_of_view = 90.0;
    let aperture = 0.8;
    return Camera::new(&origin, look_at, vup, field_of_view, ASPECT_RATIO, aperture, focus_distance);
}

fn main() {
    rayon::ThreadPoolBuilder::new().num_threads(8).build_global().unwrap();

    let mut file = std::fs::File::create("image.ppm").expect("create failed");

    file.write_all(format!("P6\n{} {}\n255\n", WIDTH, HEIGHT).as_bytes()).expect("write failed");

    let camera = create_camera();
    let scene = create_scene();

    println!("Calculating Photon map...");

    let mut photon_map_global = PhotonMap::new();
    let mut photon_map_caustic = PhotonMap::new();

    for _ in 0..N_PHOTONS {
        let (ray, color) = scene.random_photon_ray(N_PHOTONS);
        scene.trace_photon(&mut photon_map_global, &mut photon_map_caustic, &ray, color, 0, BounceType::NONE);
    }

    println!("Balancing...");
    photon_map_global.init_balance();
    photon_map_caustic.init_balance();
    // photon_map_global.scale_photon_power(10000.0 / (N_PHOTONS as f32));
    // photon_map_caustic.scale_photon_power(10000.0 / (N_PHOTONS as f32));
    println!("Photon map calculated! Size: global: {}, caustic: {}", photon_map_global.stored_photons, photon_map_caustic.stored_photons);

    let counter = AtomicU32::new(0);
    let ys = (0..HEIGHT).collect::<Vec<u32>>();
    let mut result = ys.par_iter()
        .map(|y| {
            let mut heap = Heap::new();
            let between = Uniform::new(-0.5, 0.5);
            let mut rng = rand::thread_rng();
            let xs = (0..WIDTH).collect::<Vec<u32>>();
            let colors = xs.into_iter()
                .map(|x| {
                    let samples = (0..SAMPLING_AMOUNT).collect::<Vec<u32>>();
                    samples.into_iter()
                        .map(|_| {
                            let ra = if SAMPLING_AMOUNT == 1 { 0.0 } else { between.sample(&mut rng) };
                            let rb = if SAMPLING_AMOUNT == 1 { 0.0 } else { between.sample(&mut rng) };
                            let a = (x as f32 + ra)/(WIDTH as f32);
                            let b = (*y as f32 + rb)/(HEIGHT as f32);
                            let ray = camera.create_ray(false, a, b);
                            scene.trace_ray(&mut heap, &photon_map_global, &photon_map_caustic, &ray, 0)
                        })
                        .reduce(|a, b| a + b).unwrap() * (1.0 / (SAMPLING_AMOUNT as f32))
                })
                .collect::<Vec<_>>();

            let c = counter.load(Relaxed) + 1;
            counter.store(c, Relaxed);
            println!("{:.2}%", 100.0 * (c as f32) / (HEIGHT as f32));

            (y, colors)
        })
        .collect::<Vec<_>>();

    result.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));

    for (_, colors) in result {
        for color in colors {
            let buf = color.to_buffer();
            file.write_all(&buf).expect("write failed");
        }
    }
}
