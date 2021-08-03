use crate::material::Color;
use std::io::Write;

pub struct PPM {
    file_name: String,
    height: usize,
    width: usize,
    data: Vec<u8>,
}

impl PPM {
    pub fn new(file_name: &String, width: usize, height: usize) -> Self {
        PPM {
            file_name: file_name.to_string(),
            height,
            width,
            data: vec![0; 3 * height * width],
        }
    }

    pub fn add_pixel(&mut self, x: usize, y: usize, color: Color) -> () {
        let location = 3 * (y * self.width + x);
        if location < 3 * self.height * self.width {
            self.data[location] = color.r_byte();
            self.data[location + 1] = color.g_byte();
            self.data[location + 2] = color.b_byte();
        }
    }

    pub fn write_file(&self) -> std::io::Result<()> {
        std::fs::File::create(self.file_name.to_string())
            .and_then(|mut f| {
                f.write_all(format!("P6\n{} {}\n255\n", self.width, self.height).as_bytes())
                    .map(|_| f)
            })
            .and_then(|mut f| f.write_all(&self.data))
    }
}
