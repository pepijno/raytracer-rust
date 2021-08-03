use crate::material::{Color, Material};
use crate::vector3::Vector3;

pub struct Light {
    pub position: Vector3,
    pub color: Color,
    pub intensity: f32,
}

pub struct Object {
    pub shape: Shape,
    pub material: Material,
}

pub struct Plane {
    pub position: Vector3,
    pub normal: Vector3,
}

pub struct Sphere {
    pub origin: Vector3,
    pub radius: f32,
}

pub struct Triangle {
    pub vertex1: Vector3,
    pub vertex2: Vector3,
    pub vertex3: Vector3,
}

pub struct Pyramid {
    pub vertex1: Vector3,
    pub vertex2: Vector3,
    pub vertex3: Vector3,
    pub vertex4: Vector3,
}

pub enum Shape {
    Plane(Plane),
    Sphere(Sphere),
    Triangle(Triangle),
    Pyramid(Pyramid),
}

impl Shape {
    pub fn plane(position: Vector3, normal: Vector3) -> Shape {
        Shape::Plane(Plane {
            position,
            normal,
        })
    }

    pub fn sphere(origin: Vector3, radius: f32) -> Shape {
        Shape::Sphere(Sphere {
            origin,
            radius,
        })
    }

    pub fn triangle(vertex1: Vector3, vertex2: Vector3, vertex3: Vector3) -> Shape {
        Shape::Triangle(Triangle {
            vertex1,
            vertex2,
            vertex3,
        })
    }

    pub fn pyramid(vertex1: Vector3, vertex2: Vector3, vertex3: Vector3, vertex4: Vector3) -> Shape {
        Shape::Pyramid(Pyramid {
            vertex1,
            vertex2,
            vertex3,
            vertex4,
        })
    }
}
