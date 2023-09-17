use std::io::{BufRead, Write};

use sfml::system::Vector2i;

use crate::char::Char;

pub struct Font {
    pub name: String,
    pub char_size: Vector2i,
    pub chars: Vec<Char>,
}

impl Font {
    pub fn new(name: &str, char_size: Vector2i) -> Self {
        let name = name.to_string();
        let chars = Vec::new();

        Self {
            name,
            char_size,
            chars,
        }
    }

    #[allow(dead_code)]
    fn filename(name: &str, char_size: Vector2i) -> String {
        format!("resources/{}_{}x{}.bin", name, char_size.x, char_size.y)
    }

    #[allow(dead_code)]
    #[allow(unused_variables)]
    pub fn load(name: &str, char_size: Vector2i) -> std::io::Result<Font> {
        let mut font = Self::new(name, char_size);
        let filename = Self::filename(name, char_size);
        let file = std::fs::File::open(filename)?;
        let mut buf_reader = std::io::BufReader::new(file);

        let mut read_str = || {
            let mut s = String::new();
            let num_read = buf_reader.read_line(&mut s)?;
            if num_read == 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "Unexpected EOF",
                ));
            }
            Ok(s.trim().to_string())
        };

        let name = read_str()?.strip_prefix("Name: ").unwrap();
        let size = read_str()?.strip_prefix("Size: ").unwrap().to_string();
        let size: Vec<_> = size.split('x').collect();
        let size = Vector2i::new(size[0].parse().unwrap(), size[1].parse().unwrap());
        let mut current_char = None;
        let mut current_char_lines = String::new();

        let mut finish_char = |char_id: Option<usize>, char_lines: &str| -> bool {
            match char_id {
                Some(char_id) => {
                    font.extend_chars(char_id);
                    font.chars[char_id] = Char::from_ascii(char_lines);
                    true
                }
                None => false,
            }
        };

        loop {
            let mut line = String::new();
            let num_read = buf_reader.read_line(&mut line)?;
            if num_read == 0 {
                if let Some(char_id) = current_char {
                    if !finish_char(Some(char_id), &current_char_lines) {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Unexpected EOF",
                        ));
                    }
                    current_char_lines.clear();
                }
                break;
            }

            // header format: "Char: 0x41='A'"
            if let Some(line) = line.strip_prefix("Char: ") {
                let _ = finish_char(current_char, &current_char_lines);
                current_char_lines.clear();
                let mut parts = line.split('=');
                let index = parts.next().unwrap().strip_prefix("0x").unwrap();
                let index = usize::from_str_radix(index, 16).unwrap();
                current_char = Some(index);
            } else {
                current_char_lines.push_str(&line);
            }
        }

        Ok(font)
    }

    #[allow(dead_code)]
    pub fn save(&self) -> std::io::Result<()> {
        let filename = Self::filename(&self.name, self.char_size);
        let mut file = std::fs::File::create(filename)?;

        file.write_all(format!("Name: {}\n", self.name).as_bytes())?;
        file.write_all(format!("Size: {}x{}\n", self.char_size.x, self.char_size.y).as_bytes())?;

        for (index, ch) in self.chars.iter().enumerate() {
            let symbol = std::char::from_u32(index as u32).unwrap_or(' ');
            if !ch.pixels.is_empty() {
                // ISO-8859-15
                let repr = if index < 0x20 {
                    format!("0x{:02x}", index)
                } else if index < 0x7f {
                    format!("0x{:02x}='{}'", index, symbol)
                } else if index < 0xa0 {
                    format!("0x{:02x}", index)
                } else {
                    format!("0x{:02x}='{}'", index, symbol)
                };
                file.write_all(format!("Char: {}\n", repr).as_bytes())?;

                ch.save(&mut file)?;
            }
        }
        Ok(())
    }

    pub fn get_char(&self, index: usize) -> Char {
        if index >= self.chars.len() {
            return Char::new_filled(self.char_size);
        }
        if self.chars[index].pixels.is_empty() {
            return Char::new_filled(self.char_size);
        }
        self.chars[index].clone()
    }

    #[allow(dead_code)]
    pub fn set_char(&mut self, index: usize, ch: Char) {
        self.extend_chars(index);
        self.chars[index] = ch;
    }

    // make sure we can access chars[index]
    #[allow(dead_code)]
    pub fn extend_chars(&mut self, index: usize) {
        let len = self.chars.len();
        for _ in len..index + 1 {
            self.chars.push(Char::new_empty());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_font(name: &str) -> Font {
        let size = Vector2i::new(6, 8);
        let mut font = Font::new(name, size);
        let ch = Char::new_filled(size);
        font.set_char(0x20, ch); // ' '
        let ch = Char::from_ascii(
            r#" +++
+   +
+   +
+++++
+   +
+   +
+   +
     "#,
        );
        font.set_char(0x41, ch); // 'A'

        font
    }

    #[test]
    fn test_font() {
        let font = Font::new("test", Vector2i::new(16, 16));
        assert_eq!(font.name, "test");
        assert_eq!(font.char_size, Vector2i::new(16, 16));
        assert_eq!(font.chars.len(), 0);
    }

    #[test]
    fn test_font_save() {
        let font = setup_test_font("test_font_save");
        font.save().unwrap();

        let expected_output = r#"Name: test_font_save
Size: 6x8
Char: 0x20=' '








Char: 0x41='A'
 +++
+   +
+   +
+++++
+   +
+   +
+   +

"#;
        let filename = Font::filename("test_font_save", Vector2i::new(6, 8));
        let output = std::fs::read_to_string(filename).unwrap();
        let actual_lines = output.lines().collect::<Vec<_>>();
        let expected_lines = expected_output.lines().collect::<Vec<_>>();
        assert_eq!(actual_lines.len(), expected_lines.len());
        for i in 0..actual_lines.len() {
            eprintln!("{}: '{}' vs '{}'", i, actual_lines[i], expected_lines[i]);
            assert_eq!(actual_lines[i].trim(), expected_lines[i].trim());
        }
    }

    #[test]
    fn test_font_load() {
        let font = setup_test_font("test_font_load");
        font.save().unwrap();

        let font = Font::load("test_font_load", Vector2i::new(6, 8)).unwrap();
        assert_eq!(font.name, "test_font_load");
        assert_eq!(font.char_size, Vector2i::new(6, 8));
        assert_eq!(font.chars.len(), 66);
        assert_eq!(font.chars[0].pixels.len(), 0);
        assert_eq!(font.chars[5].pixels.len(), 0);
        eprintln!("' ':\n{}", font.chars[0x20]);
        assert_eq!(font.chars[0x20].pixels.len(), 8);
        eprintln!("'A':\n{}", font.chars[0x41]);
        assert_eq!(font.chars[0x41].pixels.len(), 8);
        let ch = Char::from_ascii(
            r#" +++
+   +
+   +
+++++
+   +
+   +
+   +
     "#,
        );
        for i in 0..ch.pixels.len() {
            assert_eq!(font.chars[0x41].pixels[i], ch.pixels[i]);
        }
    }
}
