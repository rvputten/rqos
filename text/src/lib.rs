use glsl::{Vec2, Vec4};
use sfml::graphics::{
    glsl, Color, RenderStates, RenderTarget, RenderTexture, RenderWindow, Shader, ShaderType,
    Sprite, Transformable,
};
use sfml::system::{Vector2f, Vector2i};

use ansi::{Ansi, AnsiCode};
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

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum InsertMode {
    Overwrite,
    Insert,
}

pub struct TextBuilder {
    position: Option<Vector2i>,
    size: Option<Vector2i>,
    vertical_alignment: Option<VerticalAlignment>,
    font_scale: Option<i32>,
    fg_color: Option<Color>,
    bg_color: Option<Color>,
    cursor_insert_color: Option<Color>,
    cursor_normal_color: Option<Color>,
    bold: Option<bool>,
    cursor_state: Option<CursorState>,
    insert_mode: Option<InsertMode>,
}

impl TextBuilder {
    pub fn new() -> Self {
        Self {
            position: None,
            size: None,
            vertical_alignment: None,
            font_scale: None,
            fg_color: None,
            bg_color: None,
            cursor_insert_color: None,
            cursor_normal_color: None,
            bold: None,
            cursor_state: None,
            insert_mode: None,
        }
    }

    pub fn insert_mode(mut self, insert_mode: InsertMode) -> Self {
        self.insert_mode = Some(insert_mode);
        self
    }

    pub fn position(mut self, position: Vector2i) -> Self {
        self.position = Some(position);
        self
    }

    pub fn size(mut self, size: Vector2i) -> Self {
        self.size = Some(size);
        self
    }

    pub fn vertical_alignment(mut self, vertical_alignment: VerticalAlignment) -> Self {
        self.vertical_alignment = Some(vertical_alignment);
        self
    }

    pub fn font_scale(mut self, font_scale: i32) -> Self {
        self.font_scale = Some(font_scale);
        self
    }

    pub fn fg_color(mut self, fg_color: Color) -> Self {
        self.fg_color = Some(fg_color);
        self
    }

    pub fn bg_color(mut self, bg_color: Color) -> Self {
        self.bg_color = Some(bg_color);
        self
    }

    pub fn cursor_insert_color(mut self, cursor_insert_color: Color) -> Self {
        self.cursor_insert_color = Some(cursor_insert_color);
        self
    }

    pub fn cursor_normal_color(mut self, cursor_normal_color: Color) -> Self {
        self.cursor_normal_color = Some(cursor_normal_color);
        self
    }

    pub fn bold(mut self, bold: bool) -> Self {
        self.bold = Some(bold);
        self
    }

    pub fn cursor_state(mut self, cursor_state: CursorState) -> Self {
        self.cursor_state = Some(cursor_state);
        self
    }

    pub fn build(self) -> Text<'static> {
        let position = self.position.unwrap_or(Vector2i::new(0, 0));
        let size = self.size.unwrap_or(Vector2i::new(1, 1));
        let vertical_alignment = self
            .vertical_alignment
            .unwrap_or(VerticalAlignment::AlwaysTop);
        let font_scale = self.font_scale.unwrap_or(1);
        let fg_color = self.fg_color.unwrap_or(Color::BLACK);
        let bg_color = self.bg_color.unwrap_or(Color::WHITE);
        let cursor_insert_color = self.cursor_insert_color.unwrap_or(fg_color);
        let cursor_normal_color = self.cursor_normal_color.unwrap_or(Color::BLACK);
        let bold = self.bold.unwrap_or(false);
        let cursor_state = self.cursor_state.unwrap_or(CursorState::Hidden);

        let shader = Shader::from_file(
            &config::Config::get_resource_path("color_bold.frag"),
            ShaderType::Fragment,
        )
        .unwrap();

        let ansi_colors = color::AnsiColor::new();
        let insert_mode = self.insert_mode.unwrap_or(InsertMode::Overwrite);

        Text {
            text: vec![String::new()],
            position,
            vertical_alignment,
            texture: RenderTexture::new(size.x as u32, size.y as u32).unwrap(),
            font_scale,
            fg_color,
            bg_color,
            cursor_insert_color,
            cursor_normal_color,
            ansi_colors,
            bold,
            redraw: true,
            shader,
            cursor_state,
            cursor_position: Vector2i::new(0, 0),
            scroll_pos_y: 0,
            insert_mode,
        }
    }
}

impl Default for TextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Text<'a> {
    pub text: Vec<String>,
    position: Vector2i,
    pub vertical_alignment: VerticalAlignment,
    texture: RenderTexture,
    font_scale: i32,
    ansi_colors: color::AnsiColor,
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
    pub insert_mode: InsertMode,
}

impl Default for Text<'_> {
    fn default() -> Text<'static> {
        TextBuilder::new().build()
    }
}

#[allow(clippy::too_many_arguments)]
impl<'a> Text<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn position_size(&mut self, position: Vector2i, size: Vector2i) -> &mut Self {
        let size_x = if size.x < 1 { 1 } else { size.x };
        let size_y = if size.y < 1 { 1 } else { size.y };
        self.position = position;
        self.texture = RenderTexture::new(size_x as u32, size_y as u32).unwrap();
        self
    }

    pub fn set_position_size(&mut self, position: Vector2i, size: Vector2i) {
        self.position_size(position, size);
        self.redraw = true;
    }

    pub fn cursor_state(&mut self, cursor_state: CursorState) -> &mut Self {
        self.cursor_state = cursor_state;
        self
    }

    pub fn set_cursor_state(&mut self, cursor_state: CursorState) {
        self.cursor_state(cursor_state);
        self.redraw = true;
    }

    pub fn cursor_colors(&mut self, insert: Color, normal: Color) -> &mut Self {
        self.cursor_insert_color = insert;
        self.cursor_normal_color = normal;
        self
    }

    pub fn set_cursor_colors(&mut self, insert: Color, normal: Color) {
        self.cursor_colors(insert, normal);
        self.redraw = true;
    }

    pub fn vertical_alignment(&mut self, vertical_alignment: VerticalAlignment) -> &mut Self {
        self.vertical_alignment = vertical_alignment;
        self
    }

    pub fn foreground_color(&mut self, color: Color) -> &mut Self {
        self.fg_color = color;
        self
    }

    pub fn background_color(&mut self, color: Color) -> &mut Self {
        self.bg_color = color;
        self
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.background_color(color);
        self.redraw = true;
    }

    pub fn get_cursor_state(&self) -> CursorState {
        self.cursor_state
    }

    pub fn get_size(&self) -> Vector2i {
        Vector2i::new(self.texture.size().x as i32, self.texture.size().y as i32)
    }

    /// write text to the internal buffer
    /// existing text is overwritten depending on the cursor position
    /// ansi escape codes are interpreted if they manipulate the buffer or the cursor
    /// ansi escape codes for color and style are stored in the buffer
    pub fn write(&mut self, input: &str) {
        if self.text.is_empty() {
            self.text.push(String::new());
        };

        let mut line: Vec<char>;

        macro_rules! text2line {
            () => {
                while self.text.len() <= self.cursor_position.y as usize {
                    self.text.push(String::new());
                }
                line = self.text[self.cursor_position.y as usize]
                    .chars()
                    .collect::<Vec<_>>()
            };
        }

        macro_rules! line2text {
            () => {
                self.text[self.cursor_position.y as usize] = line.into_iter().collect::<String>()
            };
        }

        macro_rules! line_set {
            ($c:expr) => {
                if self.insert_mode == InsertMode::Overwrite {
                    while line.len() <= self.cursor_position.x as usize {
                        line.push(' ');
                    }
                    line[self.cursor_position.x as usize] = $c;
                } else {
                    while line.len() < self.cursor_position.x as usize {
                        line.push(' ');
                    }
                    line.insert(self.cursor_position.x as usize, $c);
                }
            };
        }

        macro_rules! line_push_string {
            ($s:expr) => {
                for c in $s.chars() {
                    line_set!(c);
                    self.cursor_position.x += 1;
                }
            };
        }

        text2line!();

        let mut idx = 0;
        let chars = input.chars().collect::<Vec<_>>();
        while idx < chars.len() {
            let c = chars[idx];
            // test for Ansi sequence
            if c == '\x1b' {
                let codes = Ansi::parse(&input[idx..]);
                idx += codes.char_count;
                for code in codes.codes {
                    match code {
                        AnsiCode::CursorUp(n) => {
                            line2text!();
                            self.cursor_position.y -= n;
                            if self.cursor_position.y < 0 {
                                self.cursor_position.y = 0;
                            }
                            text2line!();
                        }
                        AnsiCode::CursorDown(n) => {
                            line2text!();
                            self.cursor_position.y += n;
                            if self.cursor_position.y >= self.text.len() as i32 {
                                self.cursor_position.y = self.text.len() as i32 - 1;
                            }
                            text2line!();
                        }
                        AnsiCode::ColorForeground(color) => {
                            let (r, g, b) = (color.r, color.g, color.b);
                            line_push_string!(format!("\x1b[38;2;{};{};{}m", r, g, b));
                        }
                        AnsiCode::ColorBackground(color) => {
                            let (r, g, b) = (color.r, color.g, color.b);
                            line_push_string!(format!("\x1b[48;2;{};{};{}m", r, g, b));
                        }
                        AnsiCode::Bold => {
                            line_push_string!("\x1b[1m");
                        }
                        AnsiCode::ResetAll => {
                            line_push_string!("\x1b[0m");
                        }
                        _ => {
                            eprintln!("Unrecognized ANSI escape code: {:?}", code);
                        }
                    }
                }
            } else if c == '\r' {
                self.cursor_position.x = 0;
                idx += 1;
            } else if c == '\n' {
                if self.insert_mode == InsertMode::Overwrite {
                    line2text!();
                    self.cursor_position.x = 0;
                    self.cursor_position.y += 1;
                    text2line!();
                } else {
                    let line_right: Vec<char> = line[self.cursor_position.x as usize..].to_vec();
                    line = line[..self.cursor_position.x as usize].to_vec();
                    line2text!();
                    line = line_right;
                    if self.cursor_position.y == self.text.len() as i32 - 1 {
                        self.text.push(String::new());
                    } else {
                        self.text
                            .insert(self.cursor_position.y as usize + 1, String::new());
                    }
                    self.cursor_position.x = 0;
                    self.cursor_position.y += 1;
                }
                idx += 1;
            } else {
                line_set!(c);
                self.cursor_position.x += 1;
                idx += 1;
            }
        }

        line2text!();

        self.redraw = true;
    }

    pub fn get_text(&self) -> Vec<String> {
        self.text.clone()
    }

    pub fn clear_to_here(&mut self) {
        let line = self.text[self.cursor_position.y as usize].clone()
            [self.cursor_position.x as usize..]
            .to_string();
        self.text[self.cursor_position.y as usize] = line;
        self.cursor_position.x = 0;
        self.redraw = true;
    }

    pub fn clear_from_here(&mut self) {
        let line = self.text[self.cursor_position.y as usize].clone()
            [..self.cursor_position.x as usize]
            .to_string();
        self.text[self.cursor_position.y as usize] = line;
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
        let font_width = font.char_size.x * self.font_scale;
        let font_height = font.char_size.y * self.font_scale;
        let bold_offset = if font.char_size.x > 12 { 2.0 } else { 1.0 }; // try to guess if font
                                                                         // stroke is 2 pixels

        let font_texture_size = font.texture.size();
        self.shader.set_uniform_vec2(
            "atlas_size",
            Vec2::new(font_texture_size.x as f32, font_texture_size.y as f32),
        );
        self.shader.set_uniform_vec2(
            "sprite_size",
            Vec2::new(font.char_size.x as f32, font.char_size.y as f32),
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
            let line = line.chars().collect::<Vec<_>>();
            let mut skip_chars = 0;
            let mut skipped_chars = 0;
            for (x, ch) in line.iter().enumerate() {
                if skip_chars > 1 {
                    skip_chars -= 1;
                    skipped_chars += 1;
                } else if *ch == 27 as char {
                    skipped_chars += 1;
                    let rest_of_line: String = line[x..].iter().collect();
                    let (to_skip, codes) = self.ansi_colors.parse_ansi_color_code(&rest_of_line);
                    skip_chars = to_skip;
                    if codes.is_empty() {
                        eprintln!(
                            "Unrecognized ANSI escape code: {}",
                            line.iter().collect::<String>()
                        );
                    }
                    for code in codes {
                        match code {
                            ColorType::Regular(color) => {
                                set_fg!(self.ansi_colors.get_color_from_ansi(color).unwrap())
                            }
                            ColorType::HighIntensity(color) => {
                                set_fg!(self.ansi_colors.get_color_from_ansi(color).unwrap())
                            }
                            ColorType::Background(color) => {
                                set_bg!(self.ansi_colors.get_color_from_ansi(color).unwrap())
                            }
                            ColorType::BackgroundHighIntensity(color) => {
                                set_bg!(self.ansi_colors.get_color_from_ansi(color).unwrap())
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
                    let mut sprite = font.get_sprite(*ch as i32);
                    sprite.set_position(Vector2f::new(
                        ((x - skipped_chars) as i32 * font_width) as f32,
                        (start_y + y as i32 * font_height) as f32,
                    ));
                    sprite.set_scale(Vector2f::new(
                        self.font_scale as f32,
                        self.font_scale as f32,
                    ));
                    self.shader.set_uniform_vec2(
                        "sprite_position",
                        Vec2::new(
                            sprite.texture_rect().left as f32,
                            sprite.texture_rect().top as f32,
                        ),
                    );
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
        let font_texture_size = font.texture.size();
        self.shader.set_uniform_vec2(
            "atlas_size",
            Vec2::new(font_texture_size.x as f32, font_texture_size.y as f32),
        );
        self.shader.set_uniform_vec2(
            "sprite_size",
            Vec2::new(font.char_size.x as f32, font.char_size.y as f32),
        );

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

    pub fn must_draw(&self) -> bool {
        self.redraw
    }

    pub fn move_cursor_horz(&mut self, dir: i32) {
        match dir {
            -1 => self.cursor_position.x -= 1,
            1 => self.cursor_position.x += 1,
            -2 => self.cursor_position.x = 0,
            2 => {
                self.cursor_position.x =
                    self.text[self.cursor_position.y as usize].chars().count() as i32
            }
            _ => panic!("Invalid direction"),
        }
        self.redraw = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write() {
        let mut text = Text::default();
        text.write("Hello World");
        assert_eq!(text.text, vec!["Hello World"]);
    }

    #[test]
    fn test_insert1() {
        let mut text = TextBuilder::new().insert_mode(InsertMode::Insert).build();
        text.write("World");
        text.move_cursor_horz(-2);
        text.write("Hello ");
        assert_eq!(text.text, vec!["Hello World"]); // extra space at end OK
        assert_eq!(text.cursor_position, Vector2i::new(6, 0));
    }

    #[test]
    fn test_insert2() {
        let mut text = Text::default();
        text.write("Hello\nWorld\n");
        assert_eq!(text.text, vec!["Hello", "World", ""]);
    }

    #[test]
    fn test_insert3() {
        let mut text = TextBuilder::new().insert_mode(InsertMode::Insert).build();
        text.write("Line 1\nLine 2\nLine 3");
        text.cursor_position = Vector2i::new(0, 1);
        text.write("Line 2.5\n");
        assert_eq!(text.text, vec!["Line 1", "Line 2.5", "Line 2", "Line 3"]);
        assert_eq!(text.cursor_position, Vector2i::new(0, 2));
    }

    #[test]
    fn test_insert4() {
        let mut text = TextBuilder::new().insert_mode(InsertMode::Insert).build();
        text.write("Line 1\nLin2\nLine 3\n");
        text.cursor_position = Vector2i::new(3, 1);
        text.write("e ");
        text.cursor_position = Vector2i::new(0, 3);
        text.write("Line 4\n");
        assert_eq!(text.text, vec!["Line 1", "Line 2", "Line 3", "Line 4", ""],);
        assert_eq!(text.cursor_position, Vector2i::new(0, 4));
    }

    #[test]
    fn test_overwrite() {
        let mut text = TextBuilder::new()
            .insert_mode(InsertMode::Overwrite)
            .build();
        text.write("LINE 1\nLine\nLine x");
        text.cursor_position = Vector2i::new(1, 0);
        text.write("ine");
        text.cursor_position = Vector2i::new(5, 1);
        text.write("2\nLine 3\nLine 4\n");
        assert_eq!(text.text, vec!["Line 1", "Line 2", "Line 3", "Line 4", ""],);
        assert_eq!(text.cursor_position, Vector2i::new(0, 4));
    }

    #[test]
    fn test_cursor_up_down() {
        let mut text = Text::default();
        // echo "Lines 1-4 must be in order without gaps."
        // echo "Line 1"
        // echo
        // echo "Line 3"
        // printf "\e[2ALine 2\n"
        // printf "\e[2BLine 4\n"
        text.write("Lines 1-4 must be in order without gaps.\n");
        text.write("Line 1\n");
        text.write("\n");
        text.write("Line 3\n");
        text.write("\x1b[2ALine 2\n");
        text.write("\x1b[2BLine 4\n");
        assert_eq!(
            text.text,
            vec![
                "Lines 1-4 must be in order without gaps.",
                "Line 1",
                "Line 2",
                "Line 3",
                "Line 4",
                ""
            ]
        );
        assert_eq!(text.cursor_position, Vector2i::new(0, 5));
    }
}
