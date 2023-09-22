use sfml::graphics::{
    glsl, Color, RenderStates, RenderTarget, RenderTexture, RenderWindow, Shader, ShaderType,
    Sprite, Transformable,
};
use sfml::system::{Vector2f, Vector2i};

//use font;

pub struct Text<'a> {
    text: Vec<String>,
    position: Vector2i,
    texture: RenderTexture,
    font_scale: i32,
    fg_color: Color,
    bg_color: Color,
    bold: bool,
    redraw: bool,
    shader: Shader<'a>,
}

impl<'a> Text<'a> {
    pub fn new(
        position: Vector2i,
        size: Vector2i,
        font_scale: i32,
        fg_color: Color,
        bg_color: Color,
        bold: bool,
    ) -> Self {
        let shader_file = if bold {
            "resources/color_bold.frag"
        } else {
            "resources/color.frag"
        };
        let shader = Shader::from_file(shader_file, ShaderType::Fragment).unwrap();

        Self {
            text: vec![String::new()],
            position,
            texture: RenderTexture::new(size.x as u32, size.y as u32).unwrap(),
            font_scale,
            fg_color,
            bg_color,
            bold,
            redraw: true,
            shader,
        }
    }

    pub fn set_position_size(&mut self, position: Vector2i, size: Vector2i) {
        self.position = position;
        self.texture = RenderTexture::new(size.x as u32, size.y as u32).unwrap();
        self.redraw = true;
    }

    pub fn write(&mut self, text: &str) {
        let mut line = if self.text.is_empty() {
            String::new()
        } else {
            self.text.pop().unwrap()
        };
        for c in text.chars() {
            if c == '\n' {
                self.text.push(line);
                line = String::new();
            } else {
                line.push(c);
            }
        }
        self.text.push(line);
        self.redraw = true;
    }

    pub fn draw(&mut self, window: &mut RenderWindow, font: &font::Font) {
        if self.redraw {
            self.shader
                .set_uniform_vec4("bg_color", glsl::Vec4::from(self.bg_color));
            self.shader
                .set_uniform_vec4("fg_color", glsl::Vec4::from(self.fg_color));
            if self.bold {
                let font_texture_size = font.texture.size();
                self.shader.set_uniform_vec2(
                    "texture_size",
                    glsl::Vec2::new(font_texture_size.x as f32, font_texture_size.y as f32),
                );
            }

            let mut states = RenderStates::default();
            states.set_shader(Some(&self.shader));

            let font_height = font.char_size.y * self.font_scale;
            let font_width = font.char_size.x * self.font_scale;
            let fully_visible_lines = self.texture.size().y as i32 / font_height;
            let partially_visible_lines =
                (self.texture.size().y as i32 + font_height - 1) / font_height;
            let text_len = self.text.len() as i32;
            let fully_shown_lines = fully_visible_lines.min(text_len);
            let partially_shown_lines = partially_visible_lines.min(text_len);
            let partially_skipped_lines = text_len - partially_shown_lines;
            let fully_skipped_lines = text_len - fully_shown_lines;
            let start_y = if fully_skipped_lines > 0 {
                self.texture.size().y as i32 - partially_shown_lines * font_height
            } else {
                0
            };
            self.texture.clear(self.bg_color);
            for (y, line) in self.text[partially_skipped_lines as usize..]
                .iter()
                .enumerate()
            {
                for (x, ch) in line.chars().enumerate() {
                    let mut sprite = font.get_sprite(ch as i32);
                    sprite.set_position(Vector2f::new(
                        (x as i32 * font_width) as f32,
                        (start_y + y as i32 * font_height) as f32,
                    ));
                    sprite.set_scale(Vector2f::new(
                        self.font_scale as f32,
                        self.font_scale as f32,
                    ));
                    self.texture.draw_with_renderstates(&sprite, &states);
                }
            }
            self.redraw = false;
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
