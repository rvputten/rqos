use sfml::system::Vector2i;

use crate::char::Char;

pub struct Font {
    pub name: String,
    pub size: Vector2i,
    pub chars: Vec<Char>,
}

impl Font {
    pub fn new(name: &str, size: Vector2i) -> Self {
        let name = name.to_string();
        let mut chars = Vec::new();

        for _ in 0..(size.x * size.y) {
            chars.push(Char {
                pixels: vec![vec![0; 8]; 8],
            });
        }

        Self { name, size, chars }
    }

    #[allow(dead_code)]
    fn filename(name: &str, size: Vector2i) -> String {
        format!("resources/{}_{}x{}.bin", name, size.x, size.y)
    }

    #[allow(dead_code)]
    #[allow(unused_variables)]
    pub fn load(name: &str, size: Vector2i) -> Option<Self> {
        None
    }

    pub fn get_char(&self, index: usize) -> Char {
        if index >= self.chars.len() {
            return Char::new();
        }
        if self.chars[index].pixels.is_empty() {
            return Char::new();
        }
        self.chars[index].clone()
    }
}
