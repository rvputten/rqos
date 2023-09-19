use sfml::graphics::{
    Color, PrimitiveType, RenderTarget, RenderWindow, Shape, Transformable, Vertex, VertexBuffer,
    VertexBufferUsage,
};
use sfml::system::{Vector2f, Vector2i};
use sfml::window::mouse;
use sfml::window::Event;
use sfml::window::Key;

use crate::font;
use crate::font::Font;

pub struct Editor {
    font_size: Vector2i,
    edit_char_scale: i32,
    font_scale: i32,
    sample_text_scale: i32,
    edit_char_offset: Vector2i,
    font_table_offset: Vector2i,
    sample_text_offset: Vector2i,
    window: RenderWindow,
    display_char: i32,
    font: Font,
}

impl Editor {
    pub fn edit(
        font_name: &str,
        font_size: Vector2i,
        edit_char_scale: i32,
        font_scale: i32,
        sample_text_scale: i32,
        window: RenderWindow,
    ) {
        let font_width = font_size.x * font_scale;
        let font_height = font_size.y * font_scale;
        let edit_char_offset = Vector2i::new(font_width, font_height);
        let edit_char_size =
            Vector2i::new(font_size.x * edit_char_scale, font_size.y * edit_char_scale);
        let sample_text_offset = Vector2i::new(
            edit_char_size.x + edit_char_offset.x + font_width,
            font_height,
        );
        let font_table_offset = Vector2i::new(
            edit_char_offset.x,
            edit_char_offset.y + edit_char_size.y + font_height,
        );
        let font = if let Ok(font) = Font::load(font_name, font_size) {
            font
        } else {
            Font::new(font_name, font_size)
        };

        let display_char = 'a' as i32;
        let mut editor = Self {
            font_size,
            edit_char_scale,
            font_scale,
            sample_text_scale,
            edit_char_offset,
            font_table_offset,
            sample_text_offset,
            window,
            display_char,
            font,
        };

        editor.run();

        let _ = editor.font.save();
    }

    fn run(&mut self) {
        while self.window.is_open() {
            while let Some(event) = self.window.poll_event() {
                match event {
                    Event::Closed => self.window.close(),
                    Event::KeyPressed { code, .. } => self.key_pressed(code),
                    Event::MouseButtonPressed { button, x, y } => self.mouse_pressed(button, x, y),
                    Event::MouseMoved { x, y } => self.mouse_moved(x, y),
                    _ => {}
                }
            }

            self.window.clear(Color::BLACK);
            self.draw_full_font_table();
            self.draw_active_char_border();
            self.draw_hover_char_border();
            self.draw_sample_text();
            self.draw_edit_char();
            self.window.display();
        }
    }

    fn key_pressed(&mut self, code: Key) {
        match code {
            Key::Escape => self.window.close(),
            Key::C => self.copy_char(),
            Key::H => self.font.flip_char_horizontal(self.display_char),
            Key::V => self.font.flip_char_vertical(self.display_char),
            Key::Left => self.font.shift_char(self.display_char, -1, 0),
            Key::Right => self.font.shift_char(self.display_char, 1, 0),
            Key::Up => self.font.shift_char(self.display_char, 0, -1),
            Key::Down => self.font.shift_char(self.display_char, 0, 1),
            _ => {}
        }
    }

    fn mouse_moved(&mut self, x: i32, y: i32) {
        let color = if mouse::Button::Left.is_pressed() {
            0xff
        } else if mouse::Button::Middle.is_pressed() {
            0x80
        } else if mouse::Button::Right.is_pressed() {
            0x00
        } else {
            return;
        };
        self.set_pixel(color, x, y);
    }

    fn mouse_pressed(&mut self, button: mouse::Button, x: i32, y: i32) {
        let color = match button {
            mouse::Button::Left => 0xff,
            mouse::Button::Middle => 0x80,
            mouse::Button::Right => 0x00,
            _ => return,
        };
        self.set_pixel(color, x, y);
        if button == mouse::Button::Left {
            self.pick_edit_char(x, y);
        }
    }

    fn copy_char(&mut self) {
        let mouse_pos = self.window.mouse_position();
        if let Some(ch) = self.pick_char(mouse_pos.x, mouse_pos.y) {
            self.font.copy_char(ch, self.display_char);
        }
    }

    fn set_pixel(&mut self, color: u8, x: i32, y: i32) {
        let pixel_x = (x - self.edit_char_offset.x) / self.edit_char_scale;
        let pixel_y = (y - self.edit_char_offset.y) / self.edit_char_scale;
        if pixel_x < self.font_size.x && pixel_y < self.font_size.y {
            self.font
                .set_pixel(self.display_char, pixel_x, pixel_y, color);
        }
    }

    fn pick_edit_char(&mut self, x: i32, y: i32) {
        if let Some(char) = self.pick_char(x, y) {
            self.display_char = char;
        }
    }

    fn pick_char(&self, x: i32, y: i32) -> Option<i32> {
        let font_grid_pos = Vector2i::new(
            (x - self.font_table_offset.x) / self.font_size.x / self.font_scale,
            (y - self.font_table_offset.y) / self.font_size.y / self.font_scale,
        );
        if font_grid_pos.x >= 0
            && font_grid_pos.x < font::NUM_COLS
            && font_grid_pos.y >= 0
            && font_grid_pos.y < font::NUM_ROWS
        {
            Some((font_grid_pos.y + font::NUM_ROWS_IGNORED) * font::NUM_COLS + font_grid_pos.x)
        } else {
            None
        }
    }

    fn font_char_pos(&self, ch: i32) -> Option<Vector2i> {
        if ch < font::NUM_CHARS_IGNORED {
            None
        } else {
            let ch = ch - font::NUM_CHARS_IGNORED;
            let char_x = ch % font::NUM_COLS;
            let char_y = ch / font::NUM_COLS;
            let x = char_x * self.font_size.x * self.font_scale + self.font_table_offset.x;
            let y = char_y * self.font_size.y * self.font_scale + self.font_table_offset.y;
            Some(Vector2i::new(x, y))
        }
    }

    fn draw_edit_char(&mut self) {
        let grid_pos = |x: i32, y: i32| {
            Vector2f::new(
                (x * self.edit_char_scale + self.edit_char_offset.x) as f32,
                (y * self.edit_char_scale + self.edit_char_offset.y) as f32,
            )
        };
        let grid_pos_color =
            |x: i32, y: i32, color: Color| Vertex::with_pos_color(grid_pos(x, y), color);

        // display char
        let mut sprite = self.font.get_sprite(self.display_char);
        sprite.set_position(grid_pos(0, 0));
        sprite.set_scale(Vector2f::new(
            self.edit_char_scale as f32,
            self.edit_char_scale as f32,
        ));
        self.window.draw(&sprite);

        // grid
        let mut vertex_buffer = VertexBuffer::new(
            PrimitiveType::LINES,
            ((self.font_size.x + 1) * 2 + (self.font_size.y + 1) * 2) as u32,
            VertexBufferUsage::STATIC,
        );
        let grid_color = Color::rgb(128, 128, 128);
        let mut vertices = Vec::new();

        // horizontal lines
        for x in 0..=self.font_size.x {
            vertices.push(grid_pos_color(x, 0, grid_color));
            vertices.push(grid_pos_color(x, self.font_size.y, grid_color));
        }
        // vertical lines
        for y in 0..=self.font_size.y {
            vertices.push(grid_pos_color(0, y, grid_color));
            vertices.push(grid_pos_color(self.font_size.x, y, grid_color));
        }

        vertex_buffer.update(&vertices, 0);

        self.window.draw(&vertex_buffer);
    }

    fn draw_sample_text(&mut self) {
        let mut text_pos = self.sample_text_offset;
        let mut draw_text = |s: &str, color: Color| {
            self.font
                .draw_text(s, text_pos, self.sample_text_scale, color, &mut self.window);
            text_pos.y += self.font_size.y * self.sample_text_scale;
        };

        let text = "The quick brown fox jumps over the lazy dog.";

        draw_text(text, Color::WHITE);
        draw_text(&text.to_uppercase(), Color::rgb(0xc0, 0xc0, 0xff));
        draw_text(&text.to_lowercase(), Color::rgb(0xff, 0xc0, 0xc0));
        let text2 = r#"
Indeed, the quick brown fox - agile, bold, and cunning - jumped over
the lazy dog; surprisingly, it didn't even break a sweat! However,
the dog, perplexed, thought: 'Why on earth would it do that?' Then,
the fox replied, "Why not?" and sent an email to its friend
@foxmail.com, writing: 'Had a great day/night, outsmarted a dog
again!'. The fox then looked at its reflection in the river, seeing
a victorious smile [or was it a smirk?] in the mirror-like surface."#;
        for line in text2.lines() {
            draw_text(line, Color::rgb(0xc0, 0xff, 0xc0));
        }

        for line in (r#"
Jack, the quick brown fox, exclaimed, 'I've outsmarted 10 dogs, earned
$100, and I'm still #1 in the forest!' before he dashed off into the
night."#)
            .lines()
        {
            draw_text(line, Color::rgb(0xff, 0xc0, 0xff));
        }

        for line in (r#"
0123456789!"\#$%&'()*+,-./:;<=>?@[\]^_`{|}~"#)
            .lines()
        {
            draw_text(line, Color::rgb(0xc0, 0xff, 0xff));
        }

        for line in (r#"
fn main() {
    let _a = 1+1; // 2
    let _b = 2*2; // 4
    let _c = 3/3; // 1
    let _d = 4-4; // 0
}"#)
        .lines()
        {
            draw_text(line, Color::rgb(0xff, 0xff, 0xc0));
        }
    }

    fn draw_full_font_table(&mut self) {
        let mut sprite = self.font.get_sprite_full();
        sprite.set_position(Vector2f::new(
            self.font_table_offset.x as f32,
            self.font_table_offset.y as f32,
        ));
        sprite.set_scale(Vector2f::new(
            self.font_scale as f32,
            self.font_scale as f32,
        ));
        self.window.draw(&sprite);
    }

    fn draw_active_char_border(&mut self) {
        self.draw_char_border(self.display_char, Color::rgb(0xC0, 0xC0, 0xC0));
    }

    fn draw_hover_char_border(&mut self) {
        let mouse_pos = self.window.mouse_position();
        if let Some(ch) = self.pick_char(mouse_pos.x, mouse_pos.y) {
            let color = Color::rgb(0xC0, 0xC0, 0x80);
            self.draw_char_border(ch, color);
        }
    }

    fn draw_char_border(&mut self, ch: i32, color: Color) {
        if let Some(pos) = self.font_char_pos(ch) {
            let x = pos.x;
            let y = pos.y;
            let size_x = self.font_size.x * self.font_scale;
            let size_y = self.font_size.y * self.font_scale;

            let mut rect = sfml::graphics::RectangleShape::new();
            rect.set_position((x as f32, y as f32));
            rect.set_size((size_x as f32, size_y as f32));
            rect.set_outline_thickness(2.0);
            rect.set_outline_color(color);
            rect.set_fill_color(Color::TRANSPARENT);
            self.window.draw(&rect);
        }
    }
}
