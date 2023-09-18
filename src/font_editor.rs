use sfml::graphics::{Color, RectangleShape, RenderTarget, RenderWindow, Shape, Transformable};
use sfml::system::{Vector2f, Vector2i};
use sfml::window::mouse;
use sfml::window::Event;

use crate::char::Char;
use crate::font::Font;

#[allow(dead_code)]
pub struct Editor {
    font_name: String,
    font_size: Vector2i,
    grid_size: Vector2i,
    grid_offset: Vector2i,
    scale: i32,
    window: RenderWindow,
    display_char: usize,
    font: Font,
    current_char: Char,
}

impl Editor {
    pub fn edit(
        font_name: &str,
        font_size: Vector2i,
        grid_size: Vector2i,
        scale: i32,
        window: RenderWindow,
    ) {
        let grid_offset = Vector2i::new(font_size.x, font_size.y) * scale;
        let font = if let Ok(font) = Font::load(font_name, font_size) {
            font
        } else {
            Font::new(font_name, font_size)
        };

        let display_char = 'A' as usize;
        let current_char = font.get_char(display_char);
        let mut editor = Self {
            font_name: font_name.to_string(),
            font_size,
            grid_size,
            grid_offset,
            scale,
            window,
            display_char,
            font,
            current_char,
        };

        editor.run();

        editor.font.save();
    }

    fn run(&mut self) {
        #[allow(unused_variables)]
        let char_data = self.font.get_char(self.display_char);
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

            self.draw_grid();
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
        let pixel_x = ((x - self.grid_offset.x) / self.grid_size.x) as usize;
        let pixel_y = ((y - self.grid_offset.y) / self.grid_size.y) as usize;
        if pixel_x < self.font_size.x as usize && pixel_y < self.font_size.y as usize {
            self.current_char.set_pixel(pixel_x, pixel_y, color);
            self.font
                .set_char(self.display_char, self.current_char.clone());
        }
    }

    fn draw_grid(&mut self) {
        let grid_pos = |x: i32, y: i32| {
            Vector2f::new(
                (x * self.grid_size.x + self.grid_offset.x) as f32,
                (y * self.grid_size.y + self.grid_offset.y) as f32,
            )
        };

        self.window.clear(Color::BLACK);

        let pixels = &self.current_char.pixels;
        for y in 0..self.font_size.y {
            for x in 0..self.font_size.x {
                let brightness = pixels[y as usize][x as usize];
                let color = Color::rgb(255 - brightness, 255 - brightness, 255 - brightness);

                let mut square = RectangleShape::new();
                square.set_size(Vector2f::new(
                    self.grid_size.x as f32,
                    self.grid_size.y as f32,
                ));
                square.set_fill_color(color);
                square.set_outline_thickness(2.0);
                square.set_outline_color(Color::rgb(128, 128, 128));
                square.set_position(grid_pos(x, y));

                self.window.draw(&square);
            }
        }

        self.window.display();
    }
}
