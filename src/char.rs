use std::io::Write;

use sfml::system::Vector2i;

#[derive(Clone)]
pub struct Char {
    pub pixels: Vec<Vec<u8>>,
}

impl Char {
    #[allow(dead_code)]
    pub fn new_empty() -> Self {
        Self { pixels: vec![] }
    }

    pub fn new_filled(size: Vector2i) -> Self {
        Self {
            pixels: vec![vec![0; size.x as usize]; size.y as usize],
        }
    }

    #[allow(dead_code)]
    pub fn from_ascii(s: &str) -> Self {
        let mut pixels = Vec::new();
        for line in s.lines() {
            let mut row = Vec::new();
            for c in line.chars() {
                row.push(match c {
                    ' ' => 0,
                    '·' => 85,
                    '+' => 170,
                    '*' => 255,
                    _ => 0,
                });
            }
            pixels.push(row);
        }
        Self { pixels }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, value: u8) {
        self.pixels[y][x] = value;
    }

    pub fn save(&self, file: &mut std::fs::File) -> std::io::Result<()> {
        let mut line = String::new();
        for row in &self.pixels {
            eprintln!("row: {:?}", row);
            for pixel in row {
                let pixel_repr = match *pixel {
                    0..=63 => ' ',
                    64..=127 => '·',
                    128..=191 => '+',
                    192..=255 => '*',
                };
                line.push(pixel_repr);
            }
            line.push('\n');
        }
        file.write_all(line.as_bytes())?;

        Ok(())
    }
}
