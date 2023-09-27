use glsl::{Vec2, Vec4};
use sfml::graphics::{
    glsl, Color, RenderStates, RenderTarget, RenderTexture, RenderWindow, Shader, ShaderType,
    Sprite, Transformable,
};
use sfml::system::{Vector2f, Vector2i};

use color::ColorType;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum CursorState {
    Hidden,
    NormalActive,
    NormalInactive,
    InsertActive,
    InsertInactive,
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
    cursor_insert_color: Color,
    cursor_normal_color: Color,
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
        let shader_file = "resources/color_bold.frag";
        let shader = Shader::from_file(shader_file, ShaderType::Fragment).unwrap();

        Self {
            text: vec![String::new()],
            position,
            vertical_alignment,
            texture: RenderTexture::new(size.x as u32, size.y as u32).unwrap(),
            font_scale,
            fg_color,
            bg_color,
            cursor_insert_color: fg_color,
            cursor_normal_color: fg_color,
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

    pub fn get_size(&self) -> Vector2i {
        Vector2i::new(self.texture.size().x as i32, self.texture.size().y as i32)
    }

    pub fn set_cursor_state(&mut self, cursor_state: CursorState) {
        self.cursor_state = cursor_state;
        self.redraw = true;
    }

    pub fn get_cursor_state(&self) -> CursorState {
        self.cursor_state
    }

    pub fn set_cursor_colors(&mut self, insert: Color, normal: Color) {
        self.cursor_insert_color = insert;
        self.cursor_normal_color = normal;
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
        self.cursor_position.x = self.text[self.cursor_position.y as usize].chars().count() as i32;
        self.redraw = true;
        old_text
    }

    fn calculate_scroll_position(&self, font: &font::Font) -> (usize, i32, i32) {
        let scroll_up_lines = (-self.scroll_pos_y).max(0) as usize;
        let text_end = self.text.len() - scroll_up_lines;

        let font_height = font.char_size.y * self.font_scale;

        let fully_visible_lines = self.texture.size().y as i32 / font_height;
        let partially_visible_lines =
            (self.texture.size().y as i32 + font_height - 1) / font_height;

        let text_len = text_end as i32;
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
        (text_end, partially_skipped_lines, start_y)
    }

    fn draw_text(
        &mut self,
        font: &font::Font,
        text_end: usize,
        partially_skipped_lines: i32,
        start_y: i32,
    ) {
        let ansi_colors = color::AnsiColor::new();
        let font_width = font.char_size.x * self.font_scale;
        let font_height = font.char_size.y * self.font_scale;
        let bold_offset = if font.char_size.x > 12 { 2.0 } else { 1.0 }; // try to guess if font
                                                                         // stroke is 2 pixels

        let font_texture_size = font.texture.size();
        self.shader.set_uniform_vec2(
            "texture_size",
            Vec2::new(font_texture_size.x as f32, font_texture_size.y as f32),
        );

        macro_rules! set_fg {
            ($fg:expr) => {
                self.shader.set_uniform_vec4("fg_color", Vec4::from($fg))
            };
        }
        macro_rules! set_bg {
            ($bg:expr) => {
                self.shader.set_uniform_vec4("bg_color", Vec4::from($bg))
            };
        }
        macro_rules! set_bold {
            ($bold:expr) => {
                self.shader
                    .set_uniform_float("bold_offset", if $bold { bold_offset } else { 0.0 })
            };
        }
        set_fg!(self.fg_color);
        set_bg!(self.bg_color);
        set_bold!(self.bold);

        self.texture.clear(self.bg_color);

        for (y, line) in self.text[partially_skipped_lines as usize..text_end]
            .iter()
            .enumerate()
        {
            let mut skip_chars = 0;
            let mut skipped_chars = 0;
            for (x, ch) in line.chars().enumerate() {
                if skip_chars > 1 {
                    skip_chars -= 1;
                    skipped_chars += 1;
                } else if ch == 27 as char {
                    skipped_chars += 1;
                    let (to_skip, codes) = ansi_colors.parse_ansi_color_code(&line[x..]);
                    skip_chars = to_skip;
                    if codes.is_empty() {
                        eprintln!("Unrecognized ANSI escape code: {}", line);
                    }
                    for code in codes {
                        match code {
                            ColorType::Regular(color) => {
                                set_fg!(ansi_colors.get_color_from_ansi(color).unwrap())
                            }
                            ColorType::HighIntensity(color) => {
                                set_fg!(ansi_colors.get_color_from_ansi(color).unwrap())
                            }
                            ColorType::Background(color) => {
                                set_bg!(ansi_colors.get_color_from_ansi(color).unwrap())
                            }
                            ColorType::BackgroundHighIntensity(color) => {
                                set_bg!(ansi_colors.get_color_from_ansi(color).unwrap())
                            }
                            ColorType::ResetFg => set_fg!(self.fg_color),
                            ColorType::ResetBg => set_bg!(self.bg_color),
                            ColorType::RgbFg(r, g, b) => set_fg!(Color::rgb(r, g, b)),
                            ColorType::RgbBg(r, g, b) => set_bg!(Color::rgb(r, g, b)),
                            ColorType::Reset => {
                                set_fg!(self.fg_color);
                                set_bg!(self.bg_color);
                                set_bold!(false);
                            }
                            ColorType::Bold => {
                                set_bold!(true);
                            }
                            ColorType::Italic => {}
                        }
                    }
                } else {
                    let mut sprite = font.get_sprite(ch as i32);
                    sprite.set_position(Vector2f::new(
                        ((x - skipped_chars) as i32 * font_width) as f32,
                        (start_y + y as i32 * font_height) as f32,
                    ));
                    sprite.set_scale(Vector2f::new(
                        self.font_scale as f32,
                        self.font_scale as f32,
                    ));
                    let mut states_fg_bg = RenderStates::default();
                    states_fg_bg.set_shader(Some(&self.shader));
                    self.texture.draw_with_renderstates(&sprite, &states_fg_bg);
                }
            }
        }
    }

    fn draw_cursor(&mut self, font: &font::Font, start_y: i32) {
        let font_width = font.char_size.x * self.font_scale;
        let font_height = font.char_size.y * self.font_scale;
        //let stroke_width = if font_width > 10 { 2.0 } else { 1.0 };

        let mix = |fg: Vec4, bg: Vec4| -> Vec4 {
            Vec4 {
                x: (fg.x + bg.x) / 2.0,
                y: (fg.y + bg.y) / 2.0,
                z: (fg.z + bg.z) / 2.0,
                w: (fg.w + bg.w) / 2.0,
            }
        };

        let fg = match self.cursor_state {
            CursorState::NormalActive => Vec4::from(self.cursor_normal_color),
            CursorState::NormalInactive => mix(
                Vec4::from(self.cursor_normal_color),
                Vec4::from(self.bg_color),
            ),
            CursorState::InsertActive => Vec4::from(self.cursor_insert_color),
            CursorState::InsertInactive => {
                mix(Vec4::from(self.fg_color), Vec4::from(self.bg_color))
            }
            CursorState::Hidden => return,
        };

        self.shader.set_uniform_vec4("bg_color", fg);
        self.shader
            .set_uniform_vec4("fg_color", Vec4::from(self.bg_color));
        self.shader
            .set_uniform_float("bold_offset", if self.bold { 1.0 } else { 0.0 });

        let mut states_bg_fg = RenderStates::default();
        states_bg_fg.set_shader(Some(&self.shader));

        let ch = self.text[self.cursor_position.y as usize]
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

    pub fn draw(&mut self, window: &mut RenderWindow, font: &font::Font) {
        if self.redraw {
            self.redraw = false;

            let (text_end, partially_skipped_lines, start_y) = self.calculate_scroll_position(font);
            self.draw_text(font, text_end, partially_skipped_lines, start_y);
            self.draw_cursor(font, start_y);
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
