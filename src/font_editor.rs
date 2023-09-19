use sfml::graphics::{
    Color, PrimitiveType, RenderTarget, RenderWindow, Transformable, Vertex, VertexBuffer,
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
    edit_char_offset: Vector2i,
    font_table_offset: Vector2i,
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
        window: RenderWindow,
    ) {
        let font_width = font_size.x * font_scale;
        let font_height = font_size.y * font_scale;
        let edit_char_offset = Vector2i::new(font_width, font_height);
        let grid_size_y = edit_char_scale * font_size.y;
        let font_table_offset = Vector2i::new(
            edit_char_offset.x,
            edit_char_offset.y + grid_size_y + font_height,
        );
        let font = if let Ok(font) = Font::load(font_name, font_size) {
            font
        } else {
            Font::new(font_name, font_size)
        };

        let display_char = 'Q' as i32;
        let mut editor = Self {
            font_size,
            edit_char_scale,
            font_scale,
            edit_char_offset,
            font_table_offset,
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
                    // mouse moved
                    Event::MouseMoved { x, y } => self.mouse_moved(x, y),
                    _ => {}
                }
            }

            self.window.clear(Color::BLACK);
            self.draw_full_font_table();
            self.draw_grid();
            self.window.display();
        }
    }

    fn key_pressed(&mut self, code: Key) {
        match code {
            Key::Escape => self.window.close(),
            Key::C => self.copy_char(),
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

    fn draw_grid(&mut self) {
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
}
