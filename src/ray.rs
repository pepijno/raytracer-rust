use std::fmt;
use crate::vector3::Vector3;

#[derive(Default, Copy, Clone)]
pub struct Ray {
    pub origin: Vector3,
    pub direction: Vector3,
}

impl Ray {
    pub fn new(origin: Vector3, direction: Vector3) -> Self {
        Ray { origin: origin, direction: direction }
    }
}

impl fmt::Display for Ray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ray ({} -> {})", self.origin, self.direction)
    }
}
