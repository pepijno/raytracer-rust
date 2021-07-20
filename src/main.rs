extern crate rand;

use std::io::Write;
use std::thread;
use std::sync::mpsc;

use rust_raytracer::material::Material;
use rust_raytracer::vector3::Vector3;
use rust_raytracer::material::Color;
use rust_raytracer::camera::Camera;
use rust_raytracer::scene::Scene;
use rust_raytracer::shape::Shape;

const THREAD_COUNT: i32 = 8;

fn main() {
    let mut file = std::fs::File::create("image.ppm").expect("create failed");

    const SIZE: i32 = 1000;
    const ASPECT_RATIO: f32 = 1.66;
    const HEIGHT: u32 = (2 * SIZE) as u32;
    const WIDTH: u32 = (2.0 * ASPECT_RATIO * (SIZE as f32)) as u32;
    file.write_all(format!("P6\n{} {}\n255\n", WIDTH, HEIGHT).as_bytes()).expect("write failed");

    let origin = Vector3::new(4.0, 1.0, 2.0);
    let look_at = Vector3::new(0.0, 0.0, -1.0);
    let vup = Vector3::new(0.0, -1.0, 0.0);
    let focus_distance = (origin - look_at).length_squared().sqrt();
    let camera = Camera::new(&origin, look_at, vup, 90.0, ASPECT_RATIO, 0.4, focus_distance);

    let mut colors = vec!(vec!(Color::black(); WIDTH as usize); HEIGHT as usize);

    let (tx, rx) = mpsc::channel();
    let mut handles = vec![];

    for id in 0..THREAD_COUNT {
        let tx1 = mpsc::Sender::clone(&tx);
        let handle = thread::spawn(move || {
            let ivory = Material {
                refractive_index: 1.0,
                albedo: [0.6, 0.3, 0.1, 0.0],
                diffuse_color: Color::new(0.7, 0.7, 0.3),
                specular_exponent: 50.0,
            };
            let ivory2 = Material {
                refractive_index: 1.0,
                albedo: [0.7, 0.2, 0.1, 0.0],
                diffuse_color: Color::new(0.8, 0.1, 0.1),
                specular_exponent: 50.0,
            };
            let glass = Material {
                refractive_index: 1.3,
                albedo: [0.0, 0.5, 0.4, 0.8],
                diffuse_color: Color::new(0.6, 0.7, 0.8),
                specular_exponent: 125.0,
            };
            let mirror = Material {
                refractive_index: 1.0,
                albedo: [0.0, 10.0, 0.8, 0.0],
                diffuse_color: Color::new(1.0, 1.0, 1.0),
                specular_exponent: 1425.0,
            };
            let rubber = Material {
                refractive_index: 1.0,
                albedo: [0.9, 0.1, 0.0, 0.0],
                diffuse_color: Color::new(0.3, 0.1, 0.1),
                specular_exponent: 10.0,
            };

            let objects: Vec<Shape> = vec![
                Shape::Sphere(Vector3::new(-1.0, 0.0, -2.0), 1.0, ivory),
                Shape::Sphere(Vector3::new(1.0, 0.0, -2.0), 1.0, glass),
                Shape::Sphere(Vector3::new(3.0, 0.0, -2.0), 1.0, ivory2),
                Shape::Sphere(Vector3::new(-2.0, 2.0, -6.0), 3.0, mirror),
                Shape::Plane(Vector3::new(0.0, -1.0, 0.0), Vector3::new(0.0, 1.0, 0.0), ivory),
            ];

            let lights = vec![
                Vector3::new(5.0, 2.0, -4.0),
                Vector3::new(-2.0, 3.0, -1.0),
            ];

            let scene = Scene::new(objects, lights);

            for z in (id..(HEIGHT as i32)).step_by(THREAD_COUNT as usize) {
                println!("{}", z);
                for x in 0..WIDTH {
                    let a = (x as f32)/(WIDTH as f32);
                    let b = (z as f32)/(HEIGHT as f32);
                    let ray = camera.create_ray(false, a, b);
                    let color = scene.trace_ray(&ray, 0);
                    tx1.send((x, z, color));
                }
            }
        });
        handles.push(handle);
    }

    for i in handles {
        i.join().unwrap();
    }

    let mut counter = 0;
    for (x, z, color) in &rx {
        counter += 1;
        colors[z as usize][x as usize] = color;
        if counter >= WIDTH * HEIGHT {
            break;
        }
    }

    for z in 0..HEIGHT {
        for x in 0..WIDTH {
            let color = colors[z as usize][x as usize];
            let buf = color.to_buffer();
            file.write_all(&buf).expect("write failed");
        }
    }
}
