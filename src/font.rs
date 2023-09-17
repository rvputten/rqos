use std::io::Write;

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
    pub fn load(name: &str, char_size: Vector2i) -> Option<Self> {
        None
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

    fn setup_test_font() -> Font {
        let size = Vector2i::new(6, 8);
        let mut font = Font::new("test", size);
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
        let font = setup_test_font();
        font.save().unwrap();

        let expected_output = r#"Name: test
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
        let filename = Font::filename("test", Vector2i::new(6, 8));
        let output = std::fs::read_to_string(filename).unwrap();
        let actual_lines = output.lines().collect::<Vec<_>>();
        let expected_lines = expected_output.lines().collect::<Vec<_>>();
        assert_eq!(actual_lines.len(), expected_lines.len());
        for i in 0..actual_lines.len() {
            eprintln!("{}: '{}' vs '{}'", i, actual_lines[i], expected_lines[i]);
            assert_eq!(actual_lines[i].trim(), expected_lines[i].trim());
        }
    }
}
