#![feature(ptr_offset_from)]
use crate::vector3::Vector3;
use crate::material::Color;
use core::f32::consts::PI;
use std::cmp::Ordering;
use std::fmt;

const N_PHOTON_RADIANCE: usize = 200;

#[derive(Debug, Copy, Clone)]
enum Flag {
    XAxis,
    YAxis,
    ZAxis,
    LEAF,
}

#[derive(Debug, Clone, Copy)]
pub struct Neighbor {
    pub distance_squared: f32,
    pub index: usize,
}

#[derive(Debug)]
struct NearestPhotons {
    max: u32,
    found: u32,
    got_heap: bool,
    position: Vector3,
    distance: Vec<f32>,
    indices: Vec<usize>,
}

#[derive(Debug)]
pub struct Photon {
    position: Vector3,
    direction: Vector3,
    power: Color,
    plane: Flag,
}

#[derive(Debug)]
pub struct PhotonMap {
    photons: Vec<Photon>,
    pub stored_photons: usize,
    prev_scale: usize,
    heap: Vec<Neighbor>,
    max_distance: f32,
}

impl fmt::Display for PhotonMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PhotonMap (photons: [\n");
        for i in 0..self.stored_photons {
            let photon = &self.photons[i];
            write!(f, "\t{}: {:?}\n", i, photon);
        }
        write!(f, "])")
    }
}

impl PhotonMap {
    pub fn new() -> Self {
        PhotonMap {
            photons: Vec::new(),
            stored_photons: 0,
            prev_scale: 1,
            heap: vec![ Neighbor { distance_squared: 0.0, index: 0 }; N_PHOTON_RADIANCE],
            max_distance: 100.0,
        }
    }

    pub fn store(&mut self, position: &Vector3, direction: &Vector3, power: &Color) -> () {
        self.stored_photons += 1;
        let photon = Photon {
            position: *position,
            direction: *direction,
            power: *power,
            plane: Flag::LEAF,
        };
        self.photons.push(photon);
    }

    pub fn scale_photon_power(&mut self, scale: f32) -> () {
        for i in 0..self.stored_photons {
            self.photons[i].power *= scale;
        }
        self.prev_scale = self.stored_photons;
    }

    pub fn init_balance(&mut self) -> () {
        self.balance(0, self.stored_photons);
    }

    pub fn balance(&mut self, begin: usize, end: usize) -> () {
        if end - begin == 0 {
            return;
        }
        if end - begin == 1 {
            self.photons[begin].plane = Flag::LEAF;
            return;
        }

        let median = begin + (end - begin) / 2;
        let mut avg = Vector3::new(0.0, 0.0, 0.0);
        let mut var = Vector3::new(0.0, 0.0, 0.0);
        let n = end - begin;

        for i in begin..end {
            avg += self.photons[i].position;
        }
        for i in begin..end {
            var += (self.photons[i].position - avg) * (self.photons[i].position - avg);
        }
        var /= n as f32;

        let max_var = var.x.max(var.y).max(var.z);

        if max_var == var.x {
            let mut slice = &mut self.photons[begin..end];
            slice.sort_by(|a, b| a.position.x.partial_cmp(&b.position.x).unwrap_or(Ordering::Equal));
            self.photons[median].plane = Flag::XAxis;
        }
        if max_var == var.y {
            let mut slice = &mut self.photons[begin..end];
            slice.sort_by(|a, b| a.position.y.partial_cmp(&b.position.y).unwrap_or(Ordering::Equal));
            self.photons[median].plane = Flag::YAxis;
        }
        if max_var == var.z {
            let mut slice = &mut self.photons[begin..end];
            slice.sort_by(|a, b| a.position.z.partial_cmp(&b.position.z).unwrap_or(Ordering::Equal));
            self.photons[median].plane = Flag::ZAxis;
        }

        self.balance(begin, median);
        self.balance(median + 1, end);
        return;
    }

    pub fn irradiance_estimate(&mut self, position: &Vector3, normal: &Vector3) -> Color {
        self.max_distance = 100.0;
        let mut result = Color::black();
        let (distance_squared, size) = self.lookup(position, normal, 0, self.stored_photons, 100.0, 0);

        if size == 0 {
            return result;
        }
        let k = 0.0;
        for i in 0..size {
            let photon = &self.photons[self.heap[i].index];
            result += photon.power * normal.inner_product(&photon.direction).max(0.0);
        }

        return result * (1.0 / (distance_squared * PI));
    }

    fn lookup(&mut self, position: &Vector3, normal: &Vector3, begin: usize, end: usize, distance_squared: f32, size: usize) -> (f32, usize) {
        if begin == end {
            return (distance_squared, size);
        }
        if begin + 1 == end {
            return self.add_neighbor(position, normal, begin, distance_squared, size);
        }

        let median = begin + (end - begin) / 2;
        let flag = self.photons[median].plane;
        let split_value = self.split_value(median, flag);
        let position_value = position_value(position, flag);

        if position_value <= split_value {
            let (a, b) = self.lookup(position, normal, begin, median, distance_squared, size);
            let (c, d) = self.add_neighbor(position, normal, median, a, b);
            if d >= N_PHOTON_RADIANCE && (position_value - split_value) * (position_value - split_value) > c {
                return (c, d);
            }

            return self.lookup(position, normal, median + 1, end, c, d);
        } else {
            let (a, b) = self.lookup(position, normal, median + 1, end, distance_squared, size);
            let (c, d) = self.add_neighbor(position, normal, median, a, b);
            if d >= N_PHOTON_RADIANCE && (position_value - split_value) * (position_value - split_value) > c {
                return (c, d);
            }

            return self.lookup(position, normal, begin, median, c, d);
        }
    }

    fn add_neighbor(&mut self, position: &Vector3, normal: &Vector3, index: usize, distance: f32, size: usize) -> (f32, usize) {
        let dist = (position - self.photons[index].position).length_squared();
        if (position - self.photons[index].position).normalized().inner_product(normal).abs() > 0.033 || dist > self.max_distance * self.max_distance {
            return (distance, size);
        }

        let distance_squared = (self.photons[index].position - position).length_squared();
        if size < N_PHOTON_RADIANCE || distance_squared < distance {
            let mut new_size = size;
            if size == N_PHOTON_RADIANCE {
                new_size = self.heap_remove(size);
            }

            new_size = self.heap_add(new_size, index, distance_squared);

            return (self.heap[0].distance_squared, new_size);
        }
        return (distance, size);
    }

    fn split_value(&self, index: usize, axis: Flag) -> f32 {
        position_value(&self.photons[index].position, axis)
    }

    fn heap_add(&mut self, size: usize, index: usize, distance_squared: f32) -> usize {
        let mut i = size;
        self.heap[i].index = index;
        self.heap[i].distance_squared = distance_squared;
        let new_size = size + 1;

        loop {
            if i == 0 {
                return new_size;
            }
            let parent = (i - 1) / 2;
            let i_val = self.heap[i].distance_squared;
            let parent_val = self.heap[parent].distance_squared;
            if parent_val >= i_val {
                return new_size;
            }

            self.heap.swap(i, parent);
            i = parent;
        }
    }

    fn heap_remove(&mut self, size: usize) -> usize {
        self.heap[0].index = self.heap[size - 1].index;
        self.heap[0].distance_squared = self.heap[size - 1].distance_squared;
        let new_size = size - 1;

        let mut i = 0;
        let (mut i_val, mut left_val, mut right_val) = (0.0, 0.0, 0.0);

        loop {
            let left = 2 * i + 1;
            let right = 2 * i + 2;
            if left >= new_size && right >= new_size {
                return new_size;
            }

            i_val = self.heap[i].distance_squared;
            left_val = if left < new_size { self.heap[left].distance_squared } else { -1.0 };
            right_val = if right < new_size { self.heap[right].distance_squared } else { -1.0 };

            if i_val >= left_val && i_val >= right_val {
                return new_size;
            }

            if left_val == -1.0 && right_val != -1.0 {
                self.heap.swap(i, right);
                i = right;
            } else if left_val != -1.0 && right_val == -1.0 {
                self.heap.swap(i, left);
                i = right;
            } else {
                let bigger = if left_val > right_val { left } else { right };
                self.heap.swap(i, bigger);
                i = bigger;
            }
        }
    }
}

fn position_value(position: &Vector3, axis: Flag) -> f32 {
    match axis {
            Flag::XAxis => position.x,
            Flag::YAxis => position.y,
            Flag::ZAxis => position.z,
            _ => 0.0,
    }
}
