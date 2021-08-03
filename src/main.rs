extern crate rand;
extern crate rand_distr;

use rand_distr::{Distribution, Uniform};
use rayon::prelude::*;
use rust_raytracer::camera::Camera;
use rust_raytracer::material::{Color, Material};
use rust_raytracer::photonmap::{Heap, PhotonMap};
use rust_raytracer::ppm::PPM;
use rust_raytracer::scene::{BounceType, Scene};
use rust_raytracer::objects::{Object, Shape, Light};
use rust_raytracer::vector3::Vector3;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;

const THREAD_COUNT: usize = 8;
const SAMPLING_AMOUNT: usize = 1;
const NUMBER_OF_PHOTONS: usize = 800000;

const SIZE: usize = 150;
const ASPECT_RATIO: f32 = 1.66;
const HEIGHT: usize = 2 * SIZE;
const WIDTH: usize = (2.0 * ASPECT_RATIO * (SIZE as f32)) as usize;

fn create_scene() -> Scene {
    let ivory = Material {
        refractive_index: 0.0,
        diffuse_color: Color::new(0.1, 0.1, 0.15),
        reflect_color: Color::new(0.85, 0.85, 0.85),
        specular_exponent: 50.0,
    };
    // let ivory2 = Material {
    //     refractive_index: 0.0,
    //     diffuse_color: Color::new(0.1, 0.8, 0.1),
    //     reflect_color: Color::black(),
    //     specular_exponent: 50.0,
    // };
    let ivory3 = Material {
        refractive_index: 0.0,
        diffuse_color: Color::new(0.3, 0.7, 0.7),
        reflect_color: Color::new(0.2, 0.1, 0.3),
        specular_exponent: 50.0,
    };
    let glass = Material {
        refractive_index: 1.5,
        diffuse_color: Color::new(0.6, 0.7, 0.8),
        reflect_color: Color::black(),
        specular_exponent: 125.0,
    };
    // let mirror = Material {
    //     refractive_index: 0.0,
    //     diffuse_color: Color::black(),
    //     reflect_color: Color::new(1.0, 1.0, 1.0) * 0.8,
    //     specular_exponent: 125.0,
    // };
    let rubber = Material {
        refractive_index: 0.0,
        diffuse_color: Color::new(0.3, 0.1, 0.1),
        reflect_color: Color::black(),
        specular_exponent: 1000000.0,
    };

    let objects: Vec<Object> = vec![
        Object {
            shape: Shape::pyramid(
                Vector3::new(5.0, -0.999, -1.0),
                Vector3::new(3.0, -0.999, -4.0),
                Vector3::new(3.0, 2.0, -2.3),
                Vector3::new(1.0, -0.999, -1.0),
            ),
            material: glass,
        },
        Object {
            shape: Shape::sphere(Vector3::new(-1.0, 0.0, -2.0), 1.0),
            material: ivory,
        },
        Object {
            shape: Shape::sphere(Vector3::new(3.0, 0.0, 0.0), 1.0),
            material: glass,
        },
        Object {
            shape: Shape::plane(Vector3::new(0.0, -1.0, 0.0), Vector3::new(0.0, 1.0, 0.0)),
            material: rubber,
        },
        Object {
            shape: Shape::pyramid(
                Vector3::new(10.0, -0.999, -8.0),
                Vector3::new(3.5, 12.0, -8.0),
                Vector3::new(-3.0, -0.999, -8.0),
                Vector3::new(3.5, -0.999, -15.0),
            ),
            material: ivory,
        },
        Object {
            shape: Shape::sphere(Vector3::new(-5.0, 1.0, 3.0), 2.0),
            material: ivory3,
        },
    ];

    let lights = vec![
        Light {
            position: Vector3::new(5.0, 10.0, -4.0),
            color: Color::new(1.0, 1.0, 1.0),
            intensity: 5000.0,
        },
        Light {
            position: Vector3::new(-4.0, 12.0, 3.0),
            color: Color::new(1.0, 1.0, 1.0),
            intensity: 5000.0,
        },
    ];

    Scene::new(objects, lights)
}

fn create_camera() -> Camera {
    let origin = &Vector3::new(7.0, 2.0, 5.0);
    let look_at = &Vector3::new(3.0, 0.0, 0.0);
    let vup = &Vector3::new(0.0, -1.0, 0.0);
    let focus_distance = (origin - look_at).length_squared().sqrt();
    let field_of_view = 90.0;
    let aperture = 0.8;
    Camera::new(
        origin,
        look_at,
        vup,
        field_of_view,
        ASPECT_RATIO,
        aperture,
        focus_distance,
    )
}

fn main() {
    rayon::ThreadPoolBuilder::new()
        .num_threads(THREAD_COUNT)
        .build_global()
        .unwrap();

    let camera = create_camera();
    let scene = create_scene();

    println!("Calculating Photon map...");

    let mut photon_map_global = PhotonMap::new();
    let mut photon_map_caustic = PhotonMap::new();

    for _ in 0..NUMBER_OF_PHOTONS {
        let (ray, color) = scene.random_photon_ray(NUMBER_OF_PHOTONS);
        scene.trace_photon(
            &mut photon_map_global,
            &mut photon_map_caustic,
            &ray,
            color,
            0,
            BounceType::NONE,
        );
    }

    println!("Balancing...");
    photon_map_global.init_balance();
    photon_map_caustic.init_balance();
    println!(
        "Photon map calculated! Size: global: {}, caustic: {}",
        photon_map_global.stored_photons, photon_map_caustic.stored_photons
    );

    let mut ppm = PPM::new(&String::from("image.ppm"), WIDTH, HEIGHT);

    let counter = AtomicU32::new(0);
    let ys = (0..HEIGHT).collect::<Vec<usize>>();
    let result = ys
        .par_iter()
        .map(|y| {
            let mut heap = Heap::new();
            let between = Uniform::new(-0.5, 0.5);
            let mut rng = rand::thread_rng();
            let xs = (0..WIDTH).collect::<Vec<usize>>();
            let colors = xs
                .into_iter()
                .map(|x| {
                    let samples = (0..SAMPLING_AMOUNT).collect::<Vec<usize>>();
                    samples
                        .into_iter()
                        .map(|_| {
                            let ra = if SAMPLING_AMOUNT == 1 {
                                0.0
                            } else {
                                between.sample(&mut rng)
                            };
                            let rb = if SAMPLING_AMOUNT == 1 {
                                0.0
                            } else {
                                between.sample(&mut rng)
                            };
                            let a = (x as f32 + ra) / (WIDTH as f32);
                            let b = (*y as f32 + rb) / (HEIGHT as f32);
                            let ray = camera.create_ray(false, a, b);
                            scene.trace_ray(
                                &mut heap,
                                &photon_map_global,
                                &photon_map_caustic,
                                &ray,
                                0,
                            )
                        })
                        .reduce(|a, b| a + b)
                        .unwrap()
                        * (1.0 / (SAMPLING_AMOUNT as f32))
                })
                .collect::<Vec<_>>();

            let c = counter.load(Relaxed) + 1;
            counter.store(c, Relaxed);
            println!("{:.2}%", 100.0 * (c as f32) / (HEIGHT as f32));

            (y, colors)
        })
        .collect::<Vec<_>>();

    for (y, colors) in result {
        for (x, color) in colors.iter().enumerate() {
            ppm.add_pixel(x, *y, *color)
        }
    }

    ppm.write_file().expect("Writing to ppm file failed!");
}
