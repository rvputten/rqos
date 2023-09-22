use sfml::graphics::{
    glsl, Color, RenderStates, RenderTarget, RenderTexture, RenderWindow, Shader, ShaderType,
    Sprite, Transformable,
};
use sfml::system::{Vector2f, Vector2i};

//use font;

pub struct Text {
    text: Vec<String>,
    position: Vector2i,
    texture: RenderTexture,
    font_scale: i32,
    fg_color: Color,
    bg_color: Color,
    bold: bool,
    redraw: bool,
}

impl Text {
    pub fn new(
        position: Vector2i,
        size: Vector2i,
        font_scale: i32,
        fg_color: Color,
        bg_color: Color,
        bold: bool,
    ) -> Self {
        Self {
            text: vec![String::new()],
            position,
            texture: RenderTexture::new(size.x as u32, size.y as u32).unwrap(),
            font_scale,
            fg_color,
            bg_color,
            bold,
            redraw: true,
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
        let shader_file = if self.bold {
            "resources/color_bold.frag"
        } else {
            "resources/color.frag"
        };
        let mut shader = Shader::from_file(shader_file, ShaderType::Fragment).unwrap();
        shader.set_uniform_vec4("bg_color", glsl::Vec4::from(self.bg_color));
        shader.set_uniform_vec4("fg_color", glsl::Vec4::from(self.fg_color));
        if self.bold {
            let texture_size = font.texture_size();
            shader.set_uniform_vec2(
                "texture_size",
                glsl::Vec2::new(texture_size.x as f32, texture_size.y as f32),
            );
        }

        let mut states = RenderStates::default();
        states.set_shader(Some(&shader));

        let col = |x: i32| -> f32 { (x * font.char_size.x * self.font_scale) as f32 };
        let row = |y: i32| -> f32 { (y * font.char_size.y * self.font_scale) as f32 };
        let pos = |x: i32, y: i32| -> Vector2f { Vector2f::new(col(x), row(y)) };

        let mut x = 0;

        if self.redraw {
            self.texture.clear(self.bg_color);
            for (y, line) in self.text.iter().enumerate() {
                for ch in line.chars() {
                    let mut sprite = font.get_sprite(ch as i32);
                    sprite.set_position(pos(x, y as i32));
                    sprite.set_scale(Vector2f::new(
                        self.font_scale as f32,
                        self.font_scale as f32,
                    ));
                    //self.texture.draw(&sprite);
                    self.texture.draw_with_renderstates(&sprite, &states);
                    x += 1;
                }
                x = 0;
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
