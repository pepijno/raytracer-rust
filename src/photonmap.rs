use crate::vector3::Vector3;
use crate::material::Color;
use core::f32::consts::PI;
use std::cmp::Ordering;

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
pub struct Heap {
    items: [Neighbor; N_PHOTON_RADIANCE],
    size: usize,
    max_distance_squared: f32,
}

impl Heap {
    pub fn new() -> Self {
        Heap {
            items: [Neighbor { distance_squared: 0.0, index: 0 }; N_PHOTON_RADIANCE],
            size: 0,
            max_distance_squared: 0.0,
        }
    }

    pub fn reset(&mut self) -> () {
        self.size = 0;
        self.max_distance_squared = 100.0;
    }

    fn add_photon(&mut self, index: usize, distance_squared: f32) -> () {
        if self.size == N_PHOTON_RADIANCE {
            self.heap_remove();
        }

        self.heap_add(index, distance_squared);

        self.max_distance_squared = self.items[0].distance_squared;
    }

    fn heap_add(&mut self, index: usize, distance_squared: f32) -> () {
        self.items[self.size].index = index;
        self.items[self.size].distance_squared = distance_squared;
        let mut i = self.size;
        self.size += 1;

        loop {
            if i == 0 {
                return;
            }
            let parent = (i - 1) / 2;
            let i_val = self.items[i].distance_squared;
            let parent_val = self.items[parent].distance_squared;
            if parent_val >= i_val {
                return;
            }

            self.items.swap(i, parent);
            i = parent;
        }
    }

    fn heap_remove(&mut self) -> () {
        self.items[0].index = self.items[self.size - 1].index;
        self.items[0].distance_squared = self.items[self.size - 1].distance_squared;
        self.size -= 1;
        let mut i = 0;

        loop {
            let left = 2 * i + 1;
            let right = 2 * i + 2;
            if left >= self.size && right >= self.size {
                return;
            }

            let i_val = self.items[i].distance_squared;
            let left_val = if left < self.size { self.items[left].distance_squared } else { -1.0 };
            let right_val = if right < self.size { self.items[right].distance_squared } else { -1.0 };

            if i_val >= left_val && i_val >= right_val {
                return;
            }

            if left_val == -1.0 && right_val != -1.0 {
                self.items.swap(i, right);
                i = right;
            } else if left_val != -1.0 && right_val == -1.0 {
                self.items.swap(i, left);
                i = right;
            } else {
                let bigger = if left_val > right_val { left } else { right };
                self.items.swap(i, bigger);
                i = bigger;
            }
        }
    }
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
}

unsafe impl Sync for PhotonMap {}

impl PhotonMap {
    pub fn new() -> Self {
        PhotonMap {
            photons: Vec::new(),
            stored_photons: 0,
            prev_scale: 1,
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
            let slice = &mut self.photons[begin..end];
            slice.sort_by(|a, b| a.position.x.partial_cmp(&b.position.x).unwrap_or(Ordering::Equal));
            self.photons[median].plane = Flag::XAxis;
        }
        if max_var == var.y {
            let slice = &mut self.photons[begin..end];
            slice.sort_by(|a, b| a.position.y.partial_cmp(&b.position.y).unwrap_or(Ordering::Equal));
            self.photons[median].plane = Flag::YAxis;
        }
        if max_var == var.z {
            let slice = &mut self.photons[begin..end];
            slice.sort_by(|a, b| a.position.z.partial_cmp(&b.position.z).unwrap_or(Ordering::Equal));
            self.photons[median].plane = Flag::ZAxis;
        }

        self.balance(begin, median);
        self.balance(median + 1, end);
        return;
    }

    pub fn irradiance_estimate(&self, heap: &mut Heap, position: &Vector3, normal: &Vector3) -> Color {
        let mut result = Color::black();
        self.lookup(heap, position, normal, 0, self.stored_photons);

        if heap.size == 0 {
            return result;
        }
        for i in 0..heap.size {
            let photon = &self.photons[heap.items[i].index];
            result += photon.power * normal.inner_product(&photon.direction).max(0.0);
        }

        return result * (1.0 / (heap.max_distance_squared * PI));
    }

    fn lookup(&self, heap: &mut Heap, position: &Vector3, normal: &Vector3, begin: usize, end: usize) -> () {
        if begin == end {
            return;
        }
        if begin + 1 == end {
            return self.add_neighbor(heap, position, normal, begin);
        }

        let median = begin + (end - begin) / 2;
        let flag = self.photons[median].plane;
        let split_value = self.split_value(median, flag);
        let position_value = position_value(position, flag);

        if position_value <= split_value {
            self.lookup(heap, position, normal, begin, median);
            self.add_neighbor(heap, position, normal, median);
            if heap.size >= N_PHOTON_RADIANCE && (position_value - split_value) * (position_value - split_value) > heap.max_distance_squared {
                return;
            }

            return self.lookup(heap, position, normal, median + 1, end);
        } else {
            self.lookup(heap, position, normal, median + 1, end);
            self.add_neighbor(heap, position, normal, median);
            if heap.size >= N_PHOTON_RADIANCE && (position_value - split_value) * (position_value - split_value) > heap.max_distance_squared {
                return;
            }

            return self.lookup(heap, position, normal, begin, median);
        }
    }

    fn add_neighbor(&self, heap: &mut Heap, position: &Vector3, normal: &Vector3, index: usize) -> () {
        if (position - self.photons[index].position).normalized().inner_product(normal).abs() > 0.033 {
            return;
        }

        let distance_squared = (self.photons[index].position - position).length_squared();
        if heap.size < N_PHOTON_RADIANCE || distance_squared < heap.max_distance_squared {
            return heap.add_photon(index, distance_squared);
        }
        return;
    }

    fn split_value(&self, index: usize, axis: Flag) -> f32 {
        position_value(&self.photons[index].position, axis)
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
