extern crate rand;

use std::io::Write;
use rust_raytracer::material::Material;
use rust_raytracer::vector3::Vector3;
use rust_raytracer::material::Color;
use rust_raytracer::camera::Camera;
use rust_raytracer::scene::Scene;
use rust_raytracer::shape::Shape;

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

    let ivory = Material {
        refractive_index: 1.0,
        albedo: [0.6, 0.3, 0.1, 0.0],
        diffuse_color: Color::new(0.7, 0.7, 0.3),
        specular_exponent: 50.0,
    };

    let objects: Vec<Shape> = vec![
        Shape::Plane(Vector3::new(0.0, -1.0, 0.0), Vector3::new(0.0, 1.0, 0.0), ivory),
        Shape::Sphere(Vector3::new(-1.0, 0.0, -2.0), 1.0, ivory),
    ];

    let lights = vec![
        Vector3::new(-2.0, 3.0, -1.0)
    ];

    let scene = Scene::new(objects, lights);

    for z in 0..HEIGHT {
        println!("{}", z);
        for x in 0..WIDTH {
            let a = (x as f32)/(WIDTH as f32);
            let b = (z as f32)/(HEIGHT as f32);
            let ray = camera.create_ray(false, a, b);
            let color = scene.trace_ray(&ray, 0);
            let buf = color.to_buffer();
            file.write_all(&buf).expect("write failed");
        }
    }
}
