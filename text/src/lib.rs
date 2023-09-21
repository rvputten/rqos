use sfml::graphics::{Color, RenderTarget, RenderWindow, Transformable};
use sfml::system::Vector2f;

//use font;

#[derive(Default)]
pub struct Text {
    text: Vec<String>,
}

impl Text {
    pub fn new() -> Self {
        Self { text: Vec::new() }
    }

    pub fn write(&mut self, text: &str) {
        let mut line = String::new();
        for c in text.chars() {
            if c == '\n' {
                self.text.push(line);
                line = String::new();
            } else {
                line.push(c);
            }
        }
        self.text.push(line);
    }

    pub fn draw(&self, window: &mut RenderWindow, font: &font::Font, font_scale: i32) {
        let mut x = 0;
        let mut y = 0;
        let color = Color::rgb(255, 255, 255);

        for line in &self.text {
            for ch in line.chars() {
                let mut sprite = font.get_sprite(ch as i32);
                sprite.set_position(Vector2f::new(x as f32, y as f32));
                sprite.set_scale(Vector2f::new(font_scale as f32, font_scale as f32));
                sprite.set_color(color);
                window.draw(&sprite);
                x += font.char_size.x * font_scale;
            }
            x = 0;
            y += font.char_size.y * font_scale;
        }
    }
}
