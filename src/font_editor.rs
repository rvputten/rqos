use sfml::graphics::{Color, PrimitiveType, RenderTarget, RenderWindow, Vertex};
use sfml::system::{Vector2f, Vector2i};
use sfml::window::Event;

use crate::char::Char;
use crate::font::Font;

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
    current_color: u8,
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
        let font = if let Some(font) = Font::load(font_name, font_size) {
            font
        } else {
            Font::new(&font_name, font_size)
        };

        let display_char = 'A' as usize;
        let current_char = font.get_char(display_char);
        let current_color = 0xff;
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
            current_color,
        };
        editor.run();
    }

    fn run(&mut self) {
        #[allow(unused_variables)]
        let char_data = self.font.get_char(self.display_char);

        let grid_pos = |x: i32, y: i32, color: Color| {
            Vertex::with_pos_color(
                Vector2f::new(
                    (x * self.grid_size.x + self.grid_offset.x) as f32,
                    (y * self.grid_size.y + self.grid_offset.y) as f32,
                ),
                color,
            )
        };

        let mut grid = Vec::new();

        for x in 0..=self.font_size.x {
            grid.push(vec![
                grid_pos(x, 0, Color::BLACK),
                grid_pos(x, self.font_size.y, Color::BLACK),
            ]);
        }

        for y in 0..=self.font_size.y {
            grid.push(vec![
                grid_pos(0, y, Color::BLACK),
                grid_pos(self.font_size.x, y, Color::BLACK),
            ]);
        }

        while self.window.is_open() {
            while let Some(event) = self.window.poll_event() {
                match event {
                    Event::Closed => self.window.close(),
                    Event::KeyPressed {
                        code: sfml::window::Key::Escape,
                        ..
                    } => self.window.close(),
                    Event::MouseButtonPressed {
                        button: sfml::window::mouse::Button::Left,
                        x,
                        y,
                    } => {
                        self.current_char.set_pixel(
                            ((x as i32 - self.grid_offset.x) / self.grid_size.x) as usize,
                            ((y as i32 - self.grid_offset.y) / self.grid_size.y) as usize,
                            self.current_color,
                        );
                    }
                    _ => {}
                }
            }

            self.window.clear(Color::WHITE);
            for v in &grid {
                self.window
                    .draw_primitives(v, PrimitiveType::LINES, &Default::default());
            }
            self.window.display();
        }
    }
}
