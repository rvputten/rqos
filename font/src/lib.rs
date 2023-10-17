use sfml::graphics::{
    Color, Image, IntRect, RenderTarget, RenderWindow, Sprite, Texture, Transformable,
};
use sfml::system::{Vector2f, Vector2i, Vector2u};
use sfml::SfBox;

pub const NUM_CHARS: i32 = 256;
pub const NUM_COLS: i32 = 16;
pub const NUM_ROWS: i32 = NUM_CHARS / NUM_COLS;
pub const NUM_CHARS_IGNORED: i32 = 32;
pub const NUM_ROWS_IGNORED: i32 = NUM_CHARS_IGNORED / NUM_COLS;

pub struct Font {
    pub name: String,
    pub char_size: Vector2i,
    pub image: Image,
    pub texture: SfBox<Texture>,
    pub char2idx: Vec<i32>,
    pub idx2char: Vec<i32>,
    pub max_char: i32,
}

impl Font {
    pub fn new(name: &str, char_size: Vector2i) -> Self {
        let image = Image::new(
            (char_size.x * NUM_COLS) as u32,
            (char_size.y * NUM_ROWS) as u32,
        );
        let mut texture = Texture::new().expect("Failed to create texture");
        texture
            .load_from_image(&image, IntRect::default())
            .expect("Failed to load texture");

        let mut char2idx = vec![];
        let mut idx2char = vec![];
        for ch in 0..128 {
            char2idx.push(ch);
            idx2char.push(ch);
        }
        let extended = "äöüÄÖÜß‘•〈〉\u{27e8}\u{27e9}";
        let max_char = extended.chars().map(|ch| ch as i32).max().unwrap();
        char2idx.resize(max_char as usize + 1, 0);
        for (i, ch) in extended.chars().enumerate() {
            let idx = i as i32 + 128;
            char2idx[ch as usize] = idx;
            idx2char.push(ch as i32);
        }

        Self {
            name: name.to_string(),
            char_size,
            image,
            texture,
            char2idx,
            idx2char,
            max_char,
        }
    }

    fn filename(name: &str, char_size: Vector2i) -> String {
        config::get_resource_path(&format!("{}_{}x{}.png", name, char_size.x, char_size.y))
    }

    pub fn load(name: &str, char_size: Vector2i) -> std::io::Result<Font> {
        let mut font = Self::new(name, char_size);
        let filename = Self::filename(name, char_size);
        font.image = Image::from_file(&filename).ok_or(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("File not found: {}", filename),
        ))?;
        font.texture = Texture::new().expect("Failed to create texture");
        font.texture
            .load_from_image(&font.image, IntRect::default())
            .expect("Failed to load texture");
        Ok(font)
    }

    pub fn get_sprite(&self, ch: i32) -> Sprite {
        //let ch = if !(0..NUM_CHARS).contains(&ch) {
        let ch = if ch > self.max_char || self.char2idx[ch as usize] == 0 {
            println!("Invalid character: 0x{:x}='{}'", ch, ch as u8 as char);
            '?' as i32
        } else {
            self.char2idx[ch as usize]
        };

        let x = (ch % NUM_COLS) * self.char_size.x;
        let y = (ch / NUM_COLS) * self.char_size.y;

        let mut sprite = Sprite::with_texture(&self.texture);
        sprite.set_texture_rect(IntRect::new(x, y, self.char_size.x, self.char_size.y));
        sprite
    }

    pub fn get_sprite_full(&self) -> Sprite {
        let mut sprite = Sprite::with_texture(&self.texture);
        let y_offset = NUM_ROWS_IGNORED * self.char_size.y; // ' '=32 is the first character
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

    pub fn copy_char(&mut self, source_char: i32, destination_char: i32) {
        let source_x = (source_char % NUM_COLS) * self.char_size.x;
        let source_y = (source_char / NUM_COLS) * self.char_size.y;
        let destination_x = (destination_char % NUM_COLS) * self.char_size.x;
        let destination_y = (destination_char / NUM_COLS) * self.char_size.y;

        let rect = IntRect::new(source_x, source_y, self.char_size.x, self.char_size.y);
        let copy = self.image.clone();
        unsafe {
            self.image.copy_image(
                &copy,
                destination_x as u32,
                destination_y as u32,
                rect,
                false,
            );

            self.texture.update_from_image(&self.image, 0, 0);
        }
    }

    pub fn make_all_bold(&mut self) {
        for ch in 0..NUM_CHARS {
            self.make_bold(ch);
        }
    }

    pub fn make_bold(&mut self, ch: i32) {
        let x = (ch % NUM_COLS) * self.char_size.x;
        let y = (ch / NUM_COLS) * self.char_size.y;
        let width = self.char_size.x;
        let height = self.char_size.y;

        let copy = self.image.clone();
        unsafe {
            for j in 0..height {
                for i in 0..(width - 1) {
                    let i = width - 1 - 1 - i;
                    let source_x = x + i + 1;
                    let source_y = y + j;
                    let dest_x = x + i;
                    let dest_y = y + j;
                    if (source_x < 0)
                        || (source_x >= self.image.size().x as i32)
                        || (source_y < 0)
                        || (source_y >= self.image.size().y as i32)
                    {
                        println!(
                            "Invalid source: ch={}, x={}, y={}, i={}, j={}",
                            ch, x, y, i, j
                        );
                    }
                    if (dest_x < 0)
                        || (dest_x >= self.image.size().x as i32)
                        || (dest_y < 0)
                        || (dest_y >= self.image.size().y as i32)
                    {
                        println!("Invalid dest: {}, {}", dest_x, dest_y);
                    }
                    let color = copy.pixel_at((x + i + 1) as u32, (y + j) as u32);
                    if color.r > 0 {
                        self.image.set_pixel(
                            (x + i) as u32,
                            (y + j) as u32,
                            Color::rgb(color.r, color.g, color.b),
                        );
                    }
                }
            }
            self.texture.update_from_image(&self.image, 0, 0);
        }
    }

    pub fn flip_char_horizontal(&mut self, ch: i32) {
        let x = (ch % NUM_COLS) * self.char_size.x;
        let y = (ch / NUM_COLS) * self.char_size.y;
        let width = self.char_size.x;
        let height = self.char_size.y;

        let copy = self.image.clone();
        unsafe {
            for i in 0..width {
                for j in 0..height {
                    let color = copy.pixel_at((x + i) as u32, (y + j) as u32);
                    self.image.set_pixel(
                        (x + width - i - 1) as u32,
                        (y + j) as u32,
                        Color::rgb(color.r, color.g, color.b),
                    );
                }
            }
            self.texture.update_from_image(&self.image, 0, 0);
        }
    }

    pub fn flip_char_vertical(&mut self, ch: i32) {
        let x = (ch % NUM_COLS) * self.char_size.x;
        let y = (ch / NUM_COLS) * self.char_size.y;
        let width = self.char_size.x;
        let height = self.char_size.y;

        let copy = self.image.clone();
        unsafe {
            for i in 0..width {
                for j in 0..height {
                    let color = copy.pixel_at((x + i) as u32, (y + j) as u32);
                    self.image.set_pixel(
                        (x + i) as u32,
                        (y + height - j - 1) as u32,
                        Color::rgb(color.r, color.g, color.b),
                    );
                }
            }
            self.texture.update_from_image(&self.image, 0, 0);
        }
    }

    pub fn shift_char(&mut self, ch: i32, dx: i32, dy: i32) {
        let x = (ch % NUM_COLS) * self.char_size.x;
        let y = (ch / NUM_COLS) * self.char_size.y;
        let mut source = IntRect::new(
            x - dx,
            y - dy,
            self.char_size.x - dx.abs(),
            self.char_size.y - dy.abs(),
        );
        let mut dest = Vector2u::new(x as u32, y as u32);
        if source.left < 0 {
            dest.x += (-source.left) as u32;
            source.left = 0;
        }
        let copy = self.image.clone();
        unsafe {
            self.image.copy_image(&copy, dest.x, dest.y, source, false);
            self.texture.update_from_image(&self.image, 0, 0);
        }
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

    pub fn draw_text(
        &self,
        text: &str,
        pos: Vector2i,
        scale: i32,
        color: Color,
        window: &mut RenderWindow,
    ) {
        let mut x = pos.x;
        let mut y = pos.y;
        for ch in text.chars() {
            if ch == '\n' {
                x = pos.x;
                y += self.char_size.y * scale;
            } else {
                let mut sprite = self.get_sprite(ch as i32);
                sprite.set_position(Vector2f::new(x as f32, y as f32));
                sprite.set_scale(Vector2f::new(scale as f32, scale as f32));
                sprite.set_color(color);
                window.draw(&sprite);
                x += self.char_size.x * scale;
            }
        }
    }
}
