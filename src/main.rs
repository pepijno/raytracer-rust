extern crate rand_distr;
extern crate rand;

use std::io::Write;
use std::thread;
use std::sync::mpsc;
use rand_distr::{Distribution, Uniform};
use std::sync::Arc;
use std::sync::Mutex;

use rust_raytracer::material::Material;
use rust_raytracer::vector3::Vector3;
use rust_raytracer::material::Color;
use rust_raytracer::camera::Camera;
use rust_raytracer::scene::*;
// use rust_raytracer::scene::Photon;
use rust_raytracer::shape::Shape;
use rust_raytracer::shape::Object;
use rust_raytracer::photonmap::*;

const THREAD_COUNT: i32 = 8;
const SAMPLING_AMOUNT: i32 = 10;
const PHOTON_MAP_N: usize = 400000;

const SIZE: i32 = 300;
const ASPECT_RATIO: f32 = 1.66;
const HEIGHT: u32 = (2 * SIZE) as u32;
const WIDTH: u32 = (2.0 * ASPECT_RATIO * (SIZE as f32)) as u32;

fn create_scene() -> Scene {
    let ivory = Material {
        refractive_index: 0.0,
        diffuse_color: Color::new(0.7, 0.7, 0.3),
        reflect_color: Color::new(0.7, 0.7, 0.3) * 0.3,
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
        reflect_color: Color::new(0.3, 0.7, 0.7) * 0.1,
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
        Object::new(Shape::Sphere(Vector3::new(-1.0, 0.0, -2.0), 1.0), ivory),
        // Object::new(Shape::Sphere(Vector3::new(1.0, 0.0, -2.0), 1.0), ivory2),
        Object::new(Shape::Sphere(Vector3::new(2.0, 0.0, -6.0), 1.0), ivory2),
        Object::new(Shape::Sphere(Vector3::new(3.0, 0.0, -2.0), 1.0), glass),
        Object::new(Shape::Sphere(Vector3::new(-2.0, 2.0, -6.0), 3.0), mirror),
        Object::new(Shape::Plane(Vector3::new(0.0, -1.0, 0.0), Vector3::new(0.0, 1.0, 0.0)), rubber),
        Object::new(Shape::Plane(Vector3::new(-6.0, -1.0, 0.0), Vector3::new(1.0, 0.0, 0.0)), ivory3),
    ];

    let lights = vec![
        Light::new(&Vector3::new(5.0, 10.0, -4.0), Color::new(1.0, 1.0, 1.0), 2500.0),
        // Light::new(&Vector3::new(-4.0, 12.0, -3.0), Color::new(1.0, 1.0, 1.0), 2500.0),
    ];

    return Scene::new(objects, lights);
}

fn create_camera() -> Camera {
    let origin = Vector3::new(4.0, 1.0, 2.0);
    let look_at = Vector3::new(3.0, 0.0, -2.0);
    let vup = Vector3::new(0.0, -1.0, 0.0);
    let focus_distance = (origin - look_at).length_squared().sqrt();
    let field_of_view = 90.0;
    let aperture = 0.8;
    return Camera::new(&origin, look_at, vup, field_of_view, ASPECT_RATIO, aperture, focus_distance);
}

fn main() {
    let mut file = std::fs::File::create("image.ppm").expect("create failed");

    file.write_all(format!("P6\n{} {}\n255\n", WIDTH, HEIGHT).as_bytes()).expect("write failed");

    let camera = create_camera();
    let scene = Arc::new(create_scene());

    println!("Calculating Photon map...");

    // let photon_map: Arc<Mutex<Vec<Photon>>> = Arc::new(Mutex::new(Vec::new()));
    // let mut handles = vec![];
    let mut photon_map_global = PhotonMap::new();
    let mut photon_map_caustic = PhotonMap::new();

    // for id in 0..THREAD_COUNT {
    //     let photon_map = photon_map.clone();
    //     let scene = scene.clone();
    //     let handle = thread::spawn(move || {

            for _ in 0..PHOTON_MAP_N {
                // let mut photons_m = &mut photon_map.lock().unwrap();
                // if photon_map.stored_photons >= PHOTON_MAP_N {
                //     break;
                // }
                let (ray, color) = scene.random_photon_ray(PHOTON_MAP_N);
                scene.trace_photon(&mut photon_map_global, &mut photon_map_caustic, &ray, color, 0, BounceType::NONE);
            }
    //     });
    //     handles.push(handle);
    // }
    // for i in handles {
    //     i.join().unwrap();
    // }
    // println!("Balancing...");
    //         photon_map.balance();
    photon_map_global.init_balance();
    photon_map_caustic.init_balance();
    // photon_map_global.scale_photon_power(10000.0 / (PHOTON_MAP_N as f32));
    // photon_map_caustic.scale_photon_power(10000.0 / (PHOTON_MAP_N as f32));
    println!("Photon map calculated! Size: global: {}, caustic: {}", photon_map_global.stored_photons, photon_map_caustic.stored_photons);
    // println!("{}", photon_map);
    // let mut heap = vec![ Neighbor { distance_squared: 0.0, index: 0 }; 50];
    // photon_map.lookup(&mut heap, &Vector3::new(1.0, -1.0, 2.0), &Vector3::new(0.0, 1.0, 0.0), 0, photon_map.stored_photons, 1000000.0, 0);
    // println!("{:?}", heap);
    // return;

    let mut colors = vec!(vec!(Color::black(); WIDTH as usize); HEIGHT as usize);

//     let (tx, rx) = mpsc::channel();
//     handles = vec![];

//     for id in 0..THREAD_COUNT {
//         let tx1 = mpsc::Sender::clone(&tx);
//         let scene = scene.clone();
//         let handle = thread::spawn(move || {
            let between = Uniform::new(-0.5, 0.5);
            let mut rng = rand::thread_rng();

            for y in 0..HEIGHT {
            // for y in (id..(HEIGHT as i32)).step_by(THREAD_COUNT as usize) {
                println!("{}", y);
                for x in 0..WIDTH {
                    let mut color = Color::black();
                    for _ in 0..SAMPLING_AMOUNT {
                        let ra = between.sample(&mut rng);
                        let rb = between.sample(&mut rng);
                        let a = (x as f32 + ra)/(WIDTH as f32);
                        let b = (y as f32 + rb)/(HEIGHT as f32);
                        // let a = (x as f32)/(WIDTH as f32);
                        // let b = (y as f32)/(HEIGHT as f32);
                        let ray = camera.create_ray(false, a, b);
                        color = color + &scene.trace_ray(&mut photon_map_global, &mut photon_map_caustic, &ray, 0);
                    }
                    color = color * (1.0 / (SAMPLING_AMOUNT as f32));
                    let buf = color.to_buffer();
                    file.write_all(&buf).expect("write failed");
                    // tx1.send((x, y, color));
                }
            }
        // });
        // handles.push(handle);
    // }

    // for i in handles {
        // i.join().unwrap();
    // }

    // let mut counter = 0;
    // for (x, z, color) in &rx {
    //     counter += 1;
    //     colors[z as usize][x as usize] = color;
    //     if counter >= WIDTH * HEIGHT {
    //         break;
    //     }
    // }

    // for z in 0..HEIGHT {
    //     for x in 0..WIDTH {
    //         let color = colors[z as usize][x as usize];
    //         let buf = color.to_buffer();
    //         file.write_all(&buf).expect("write failed");
    //     }
    // }
}
