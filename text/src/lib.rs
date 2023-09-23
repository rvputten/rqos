use std::fmt;
use std::fmt::{Display, Formatter};

use sfml::graphics::{
    glsl, Color, RenderStates, RenderTarget, RenderTexture, RenderWindow, Shader, ShaderType,
    Sprite, Transformable,
};
use sfml::system::{Vector2f, Vector2i};

#[derive(Debug, PartialEq)]
pub enum CursorState {
    Hidden,
    Active,
    Inactive,
}

impl Display for CursorState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CursorState::Hidden => write!(f, "Hidden"),
            CursorState::Active => write!(f, "Active"),
            CursorState::Inactive => write!(f, "Inactive"),
        }
    }
}

pub enum VerticalAlignment {
    AlwaysTop,
    AlwaysBottom,
    BottomOnOverflow,
}

pub struct Text<'a> {
    pub text: Vec<String>,
    position: Vector2i,
    pub vertical_alignment: VerticalAlignment,
    texture: RenderTexture,
    font_scale: i32,
    fg_color: Color,
    bg_color: Color,
    bold: bool,
    pub redraw: bool,
    shader: Shader<'a>,
    pub cursor_state: CursorState,
    pub cursor_position: Vector2i,
    pub scroll_pos_y: i32,
}

#[allow(clippy::too_many_arguments)]
impl<'a> Text<'a> {
    pub fn new(
        position: Vector2i,
        size: Vector2i,
        vertical_alignment: VerticalAlignment,
        font_scale: i32,
        fg_color: Color,
        bg_color: Color,
        bold: bool,
        cursor_state: CursorState,
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
            vertical_alignment,
            texture: RenderTexture::new(size.x as u32, size.y as u32).unwrap(),
            font_scale,
            fg_color,
            bg_color,
            bold,
            redraw: true,
            shader,
            cursor_state,
            cursor_position: Vector2i::new(0, 0),
            scroll_pos_y: 0,
        }
    }

    pub fn set_position_size(&mut self, position: Vector2i, size: Vector2i) {
        let size_x = if size.x < 1 { 1 } else { size.x };
        let size_y = if size.y < 1 { 1 } else { size.y };
        self.position = position;
        self.texture = RenderTexture::new(size_x as u32, size_y as u32).unwrap();
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
                self.cursor_position.x = 0;
                self.cursor_position.y += 1;
            } else {
                line.push(c);
                self.cursor_position.x += 1;
            }
        }
        self.text.push(line);
        self.redraw = true;
    }

    pub fn get_text(&self) -> Vec<String> {
        self.text.clone()
    }

    pub fn clear(&mut self) {
        self.text = vec![String::new()];
        self.cursor_position = Vector2i::new(0, 0);
        self.redraw = true;
    }

    pub fn replace(&mut self, text: Vec<String>) -> Vec<String> {
        let old_text = std::mem::replace(&mut self.text, text);
        if self.text.is_empty() {
            self.text.push(String::new());
        }
        self.cursor_position.y = self.text.len() as i32 - 1;
        self.cursor_position.x = self.text[self.cursor_position.y as usize].len() as i32;
        self.redraw = true;
        old_text
    }

    fn set_shader_parameters(&mut self, font: &font::Font, fg_color: Color, bg_color: Color) {
        self.shader
            .set_uniform_vec4("fg_color", glsl::Vec4::from(fg_color));
        self.shader
            .set_uniform_vec4("bg_color", glsl::Vec4::from(bg_color));
        if self.bold {
            let font_texture_size = font.texture.size();
            self.shader.set_uniform_vec2(
                "texture_size",
                glsl::Vec2::new(font_texture_size.x as f32, font_texture_size.y as f32),
            );
        }
    }

    pub fn draw(&mut self, window: &mut RenderWindow, font: &font::Font) {
        if self.redraw {
            let text = if self.scroll_pos_y < 0 {
                let m = (-self.scroll_pos_y) as usize;
                let m = self.text.len() - m;
                self.text[..m].to_vec()
            } else {
                self.text.clone()
            };
            let font_height = font.char_size.y * self.font_scale;
            let font_width = font.char_size.x * self.font_scale;

            let fully_visible_lines = self.texture.size().y as i32 / font_height;
            let partially_visible_lines =
                (self.texture.size().y as i32 + font_height - 1) / font_height;

            let text_len = text.len() as i32;
            let fully_shown_lines = fully_visible_lines.min(text_len);
            let partially_shown_lines = partially_visible_lines.min(text_len);

            let partially_skipped_lines = text_len - partially_shown_lines;
            let fully_skipped_lines = text_len - fully_shown_lines;
            let start_y = if fully_skipped_lines > 0 {
                self.texture.size().y as i32 - partially_shown_lines * font_height
            } else {
                0
            };

            let (partially_skipped_lines, start_y) = match self.vertical_alignment {
                VerticalAlignment::AlwaysTop => (0, 0),
                VerticalAlignment::AlwaysBottom => {
                    if text_len * font_height > self.texture.size().y as i32 {
                        (partially_skipped_lines, start_y)
                    } else {
                        (0, self.texture.size().y as i32 - text_len * font_height)
                    }
                }
                VerticalAlignment::BottomOnOverflow => {
                    if partially_skipped_lines > 0 {
                        (partially_skipped_lines, start_y)
                    } else {
                        (0, 0)
                    }
                }
            };

            {
                self.set_shader_parameters(font, self.fg_color, self.bg_color);
                let mut states_fg_bg = RenderStates::default();
                states_fg_bg.set_shader(Some(&self.shader));
                self.texture.clear(self.bg_color);
                for (y, line) in text[partially_skipped_lines as usize..].iter().enumerate() {
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
                        self.texture.draw_with_renderstates(&sprite, &states_fg_bg);
                    }
                }
                self.redraw = false;
            }

            if self.cursor_state != CursorState::Hidden {
                self.set_shader_parameters(font, self.bg_color, self.fg_color);
                let mut states_bg_fg = RenderStates::default();
                states_bg_fg.set_shader(Some(&self.shader));

                let ch = text[self.cursor_position.y as usize]
                    .chars()
                    .nth(self.cursor_position.x as usize)
                    .unwrap_or(' ');
                let mut sprite = font.get_sprite(ch as i32);
                sprite.set_position(Vector2f::new(
                    (self.cursor_position.x * font_width) as f32,
                    (start_y + self.cursor_position.y * font_height) as f32,
                ));
                sprite.set_scale(Vector2f::new(
                    self.font_scale as f32,
                    self.font_scale as f32,
                ));
                self.texture.draw_with_renderstates(&sprite, &states_bg_fg);
            }
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
