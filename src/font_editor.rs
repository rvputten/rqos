use sfml::graphics::{
    Color, PrimitiveType, RenderTarget, RenderWindow, Transformable, Vertex, VertexBuffer,
    VertexBufferUsage,
};
use sfml::system::{Vector2f, Vector2i};
use sfml::window::mouse;
use sfml::window::Event;

use crate::font::Font;

pub struct Editor {
    font_size: Vector2i,
    grid_size: i32,
    scale: i32,
    grid_offset: Vector2i,
    font_table_offset: Vector2i,
    window: RenderWindow,
    display_char: i32,
    font: Font,
}

impl Editor {
    pub fn edit(
        font_name: &str,
        font_size: Vector2i,
        grid_size: i32,
        scale: i32,
        window: RenderWindow,
    ) {
        let font_width = font_size.x * scale;
        let font_height = font_size.y * scale;
        let grid_offset = Vector2i::new(font_width, font_height);
        let grid_size_y = grid_size * font_size.y;
        let font_table_offset =
            Vector2i::new(grid_offset.x, grid_offset.y + grid_size_y + font_height);
        let font = if let Ok(font) = Font::load(font_name, font_size) {
            font
        } else {
            Font::new(font_name, font_size)
        };

        let display_char = 'C' as i32;
        let mut editor = Self {
            font_size,
            grid_size,
            scale,
            grid_offset,
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
                    Event::KeyPressed {
                        code: sfml::window::Key::Escape,
                        ..
                    } => self.window.close(),
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
    }

    fn set_pixel(&mut self, color: u8, x: i32, y: i32) {
        let pixel_x = (x - self.grid_offset.x) / self.grid_size;
        let pixel_y = (y - self.grid_offset.y) / self.grid_size;
        if pixel_x < self.font_size.x && pixel_y < self.font_size.y {
            self.font
                .set_pixel(self.display_char, pixel_x, pixel_y, color);
        }
    }

    fn draw_grid(&mut self) {
        let grid_pos = |x: i32, y: i32| {
            Vector2f::new(
                (x * self.grid_size + self.grid_offset.x) as f32,
                (y * self.grid_size + self.grid_offset.y) as f32,
            )
        };
        let grid_pos_color =
            |x: i32, y: i32, color: Color| Vertex::with_pos_color(grid_pos(x, y), color);

        // display char
        let mut sprite = self.font.get_sprite(self.display_char);
        sprite.set_position(grid_pos(0, 0));
        sprite.set_scale(Vector2f::new(self.grid_size as f32, self.grid_size as f32));
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
        sprite.set_scale(Vector2f::new(self.scale as f32, self.scale as f32));
        self.window.draw(&sprite);
    }
}
