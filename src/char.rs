use std::fmt;
use std::io::Write;

use sfml::system::Vector2i;

#[derive(Clone)]
pub struct Char {
    pub pixels: Vec<Vec<u8>>,
}

impl fmt::Display for Char {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in &self.pixels {
            for pixel in row {
                let pixel_repr = match *pixel {
                    0..=63 => ' ',
                    64..=127 => '·',
                    128..=191 => '+',
                    192..=255 => '*',
                };
                write!(f, "{}", pixel_repr)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Char {
    pub fn new_empty() -> Self {
        Self { pixels: vec![] }
    }

    pub fn new_filled(size: Vector2i) -> Self {
        Self {
            pixels: vec![vec![0; size.x as usize]; size.y as usize],
        }
    }

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
        if y < self.pixels.len() && x < self.pixels[y].len() {
            self.pixels[y][x] = value;
        }
    }

    pub fn save(&self, file: &mut std::fs::File) -> std::io::Result<()> {
        let s = format!("{}", self);
        file.write_all(s.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_display() {
        let x = r#" +++
+   +
+   +
+++++
+   +
+   +
+   +
"#;
        let ch = Char::from_ascii(x);
        assert_eq!(format!("{}", ch), x);
    }
}
