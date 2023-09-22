use sfml::graphics::{Color, RenderTarget, RenderWindow};
use sfml::system::Vector2i;
use sfml::window::Key;
use sfml::window::{Event, Style};

pub struct App {
    font: font::Font,
    font_scale: i32,
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

        let font_height = font_size.y * font_scale;
        let (cols, rows) = (120, 80);
        let window_width = cols * font_size.x * font_scale;
        let window_height = rows * font_height;
        let (window_pos_x, window_pos_y) = (
            ((screen_width as i32) - (window_width / 2)),
            ((screen_height as i32 / 2) - (window_height / 2)),
        );

        let mut window = RenderWindow::new(
            (window_width as u32, window_height as u32),
            "rqsh",
            Style::CLOSE | Style::RESIZE,
            &Default::default(),
        );
        window.set_position(Vector2i::new(window_pos_x, window_pos_y));
        window.set_vertical_sync_enabled(true);

        let mut main_text = text::Text::new(
            Vector2i::new(0, 0),
            Vector2i::new(window_width, window_height - font_height * 2),
            font_scale,
            Color::BLACK,
            Color::WHITE,
            false,
        );
        let mut status_line = text::Text::new(
            Vector2i::new(0, window_height - font_height * 2),
            Vector2i::new(window_width, font_height),
            font_scale,
            Color::BLACK,
            Color::rgb(0xf0, 0xc7, 0x00),
            true,
        );
        let mut command = text::Text::new(
            Vector2i::new(0, window_height - font_height),
            Vector2i::new(window_width, font_height),
            font_scale,
            Color::WHITE,
            Color::BLACK,
            false,
        );

        main_text.write("Hello, ");
        main_text.write("world!\n");
        for y in 1..rows {
            for x in (0..cols).step_by(5) {
                main_text.write(&format!(" {:4}", 100 * y + x));
            }
            main_text.write("\n");
        }
        status_line.write(" willem@zen:/home/willem/rust/rqos ");
        command.write(": ");

        Self {
            font,
            font_scale,
            main_text,
            status_line,
            command,
            window,
        }
    }

    pub fn run(&mut self) {
        while self.window.is_open() {
            let frame_start_time = std::time::Instant::now();
            while let Some(event) = self.window.poll_event() {
                match event {
                    Event::Closed => self.window.close(),
                    Event::KeyPressed { code, .. } => self.key_pressed(code),
                    Event::Resized { width, height } => {
                        self.resize_event(width as i32, height as i32)
                    }
                    _ => {}
                }
            }

            self.window.clear(Color::BLACK);
            self.main_text.draw(&mut self.window, &self.font);
            self.status_line.draw(&mut self.window, &self.font);
            self.command.draw(&mut self.window, &self.font);
            self.window.display();

            let frame_end_time = std::time::Instant::now();
            let frame_duration = frame_end_time - frame_start_time;
            print!("\rduration: {:?}    ", frame_duration);
            let frame_time_target = 33;
            if frame_duration.as_millis() < frame_time_target {
                let time_to_sleep = (frame_time_target - frame_duration.as_millis()) as u64;
                std::thread::sleep(std::time::Duration::from_millis(time_to_sleep));
            }
        }
    }

    fn resize_event(&mut self, width: i32, height: i32) {
        self.window.set_view(&sfml::graphics::View::new(
            sfml::system::Vector2f::new(width as f32 / 2.0, height as f32 / 2.0),
            sfml::system::Vector2f::new(width as f32, height as f32),
        ));
        self.set_window_sizes(width, height);
    }

    fn set_window_sizes(&mut self, width: i32, height: i32) {
        let font_height = self.font.char_size.y * self.font_scale;

        self.main_text.set_position_size(
            Vector2i::new(0, 0),
            Vector2i::new(width, height - font_height * 2),
        );
        self.status_line.set_position_size(
            Vector2i::new(0, height - font_height * 2),
            Vector2i::new(width, font_height),
        );
        self.command.set_position_size(
            Vector2i::new(0, height - font_height),
            Vector2i::new(width, font_height),
        );
    }

    fn key_pressed(&mut self, code: Key) {
        #[allow(clippy::single_match)]
        match code {
            Key::Escape => self.window.close(),
            _ => {}
        }
    }
}
