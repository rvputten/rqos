use sfml::graphics::{Color, RenderTarget, RenderWindow};
use sfml::system::Vector2i;
use sfml::window::Key;
use sfml::window::{Event, Style};

pub struct App {
    font: font::Font,
    font_scale: i32,
    text: text::Text,
    window: RenderWindow,
}

impl App {
    pub fn new() -> Self {
        let desktop_mode = sfml::window::VideoMode::desktop_mode();
        let screen_width = desktop_mode.width;
        let screen_height = desktop_mode.height;

        let font_name = "font";
        let font_size = Vector2i::new(10, 20);
        let font_scale = 2;
        let font = font::Font::load(font_name, font_size).expect("Failed to load font");

        let window_width = font_scale * font_size.x * 120;
        let window_height = font_scale * font_size.y * 80;
        let (window_pos_x, window_pos_y) = (
            ((screen_width as i32) - (window_width / 2)),
            ((screen_height as i32 / 2) - (window_height / 2)),
        );

        let mut window = RenderWindow::new(
            (window_width as u32, window_height as u32),
            "rqsh",
            Style::CLOSE,
            &Default::default(),
        );
        window.set_position(Vector2i::new(window_pos_x, window_pos_y));
        window.set_vertical_sync_enabled(true);

        let mut text = text::Text::new();
        text.write("Hello, world!");
        text.write("How's it going?");

        Self {
            font,
            font_scale,
            text,
            window,
        }
    }

    pub fn run(&mut self) {
        while self.window.is_open() {
            while let Some(event) = self.window.poll_event() {
                match event {
                    Event::Closed => self.window.close(),
                    Event::KeyPressed { code, .. } => self.key_pressed(code),
                    _ => {}
                }
            }

            self.window.clear(Color::BLACK);
            self.text
                .draw(&mut self.window, &self.font, self.font_scale);
            self.window.display();
        }
    }

    fn key_pressed(&mut self, code: Key) {
        match code {
            Key::Escape => self.window.close(),
            _ => {}
        }
    }
}
