use sfml::graphics::{Color, RenderTarget, RenderTexture, RenderWindow, Sprite, Transformable};
use sfml::system::{Vector2f, Vector2i};

//use font;

pub struct Text {
    text: Vec<String>,
    position: Vector2i,
    texture: RenderTexture,
}

impl Text {
    pub fn new(position: Vector2i, size: Vector2i) -> Self {
        let texture = RenderTexture::new(size.x as u32, size.y as u32).unwrap();
        Self {
            text: Vec::new(),
            position,
            texture,
        }
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

    pub fn draw(&mut self, window: &mut RenderWindow, font: &font::Font, font_scale: i32) {
        let col = |x: i32| -> f32 { (x * font.char_size.x * font_scale) as f32 };
        let row = |y: i32| -> f32 { (y * font.char_size.y * font_scale) as f32 };
        let pos = |x: i32, y: i32| -> Vector2f { Vector2f::new(col(x), row(y)) };

        let mut x = 0;
        let color = Color::rgb(255, 255, 255);

        self.texture.clear(Color::BLACK);
        for (y, line) in self.text.iter().enumerate() {
            for ch in line.chars() {
                let mut sprite = font.get_sprite(ch as i32);
                sprite.set_position(pos(x, y as i32));
                sprite.set_color(color);
                sprite.set_scale(Vector2f::new(font_scale as f32, font_scale as f32));
                self.texture.draw(&sprite);
                x += 1;
            }
            x = 0;
        }
        self.texture.display();
        let mut sprite = Sprite::with_texture(self.texture.texture());
        sprite.set_position(Vector2f::new(
            self.position.x as f32,
            self.position.y as f32,
        ));
        window.draw(&sprite);
    }
}
