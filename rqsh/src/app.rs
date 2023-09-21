use sfml::graphics::{Color, RenderTarget, RenderWindow};
use sfml::system::Vector2i;
use sfml::window::Key;
use sfml::window::{Event, Style};

pub struct App {
    font: font::Font,
    main_text: text::Text,
    status_line: text::Text,
    command: text::Text,
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

        let rows = 80;
        let cols = 120;

        let row = |y: i32| -> i32 { y * font_size.y * font_scale };
        let col = |x: i32| -> i32 { x * font_size.x * font_scale };
        let p2t = |x: i32, y: i32| -> Vector2i { Vector2i::new(col(x), row(y)) };

        let window_width = col(cols);
        let window_height = row(rows);
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

        let mut main_text = text::Text::new(p2t(0, 0), p2t(cols, rows - 2));
        main_text.set_font_scale(font_scale);
        let mut status_line = text::Text::new(p2t(0, rows - 2), p2t(cols, 1));
        status_line.set_font_scale(font_scale);
        status_line.set_bg_color(Color::rgb(0xf0, 0xc7, 0x00));
        status_line.set_fg_color(Color::BLACK);
        status_line.set_bold(true);
        let mut command = text::Text::new(p2t(0, rows - 1), p2t(cols, 1));
        command.set_font_scale(font_scale);

        main_text.write("Hello,\nworld!");
        status_line.write(" willem@zen:/home/willem/rust/rqos ");
        command.write(": ");

        Self {
            font,
            main_text,
            status_line,
            command,
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
            self.main_text.draw(&mut self.window, &self.font);
            self.status_line.draw(&mut self.window, &self.font);
            self.command.draw(&mut self.window, &self.font);
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
