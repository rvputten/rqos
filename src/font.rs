use sfml::graphics::{Color, Image, IntRect, Sprite, Texture};
use sfml::system::Vector2i;
use sfml::SfBox;

pub const NUM_CHARS: i32 = 256;
pub const NUM_COLS: i32 = 16;

pub struct Font {
    pub name: String,
    pub char_size: Vector2i,
    pub image: Image,
    pub texture: SfBox<Texture>,
}

impl Font {
    pub fn new(name: &str, char_size: Vector2i) -> Self {
        let image = Image::new(
            (char_size.x * NUM_COLS) as u32,
            (char_size.y * NUM_CHARS / NUM_COLS) as u32,
        );
        let mut texture = Texture::new().expect("Failed to create texture");
        texture
            .load_from_image(&image, IntRect::default())
            .expect("Failed to load texture");

        Self {
            name: name.to_string(),
            char_size,
            image,
            texture,
        }
    }

    fn filename(name: &str, char_size: Vector2i) -> String {
        format!("resources/{}_{}x{}.png", name, char_size.x, char_size.y)
    }

    pub fn load(name: &str, char_size: Vector2i) -> std::io::Result<Font> {
        let filename = Self::filename(name, char_size);
        let image = Image::from_file(&filename).ok_or(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("File not found: {}", filename),
        ))?;
        let mut texture = Texture::new().expect("Failed to create texture");
        texture
            .load_from_image(&image, IntRect::default())
            .expect("Failed to load texture");
        Ok(Self {
            name: name.to_string(),
            char_size,
            image,
            texture,
        })
    }

    pub fn get_sprite(&self, ch: i32) -> Sprite {
        if !(0..NUM_CHARS).contains(&ch) {
            panic!("Invalid character: {}", ch);
        }

        let x = (ch % NUM_COLS) * self.char_size.x;
        let y = (ch / NUM_COLS) * self.char_size.y;

        let mut sprite = Sprite::with_texture(&self.texture);
        sprite.set_texture_rect(IntRect::new(x, y, self.char_size.x, self.char_size.y));
        sprite
    }

    pub fn get_sprite_full(&self) -> Sprite {
        let mut sprite = Sprite::with_texture(&self.texture);
        let y_offset = (32 / NUM_COLS) * self.char_size.y; // ' '=32 is the first character
        sprite.set_texture_rect(IntRect::new(
            0,
            y_offset,
            self.image.size().x as i32,
            self.image.size().y as i32,
        ));
        sprite
    }

    pub fn set_pixel(&mut self, ch: i32, x: i32, y: i32, color: u8) {
        let x = x + (ch % NUM_COLS) * self.char_size.x;
        let y = y + (ch / NUM_COLS) * self.char_size.y;
        unsafe {
            self.image
                .set_pixel(x as u32, y as u32, Color::rgb(color, color, color));
            self.texture.update_from_image(&self.image, 0, 0)
        };
    }

    pub fn save(&self) -> std::io::Result<()> {
        let filename = Self::filename(&self.name, self.char_size);

        if self.image.save_to_file(&filename) {
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to save image",
            ))
        }
    }
}
