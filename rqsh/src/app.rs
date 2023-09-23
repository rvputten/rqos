use sfml::graphics::{Color, RenderTarget, RenderWindow};
use sfml::system::Vector2i;
use sfml::window::Key;
use sfml::window::{Event, Style};

use crate::builtin::BuiltIn;
use crate::glob::Glob;

pub struct App<'a> {
    font: font::Font,
    font_scale: i32,
    main_text: text::Text<'a>,
    directory: text::Text<'a>,
    status_line: text::Text<'a>,
    command: edit::Edit<'a>,
    window: RenderWindow,
    dir_plain: Vec<String>,
}

impl App<'_> {
    pub fn new() -> Self {
        let desktop_mode = sfml::window::VideoMode::desktop_mode();
        let screen_width = desktop_mode.width;
        let screen_height = desktop_mode.height;

        let font_name = "font";
        let font_size = Vector2i::new(20, 40);
        let font_scale = 1;
        let font = font::Font::load(font_name, font_size).expect("Failed to load font");

        let font_width = font_size.x * font_scale;
        let font_height = font_size.y * font_scale;
        let dir_window_cols = 40;
        let dir_window_width = dir_window_cols * font_width;
        let (cols, rows) = (120 + dir_window_cols, 80);
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

        let main_text = text::Text::new(
            Vector2i::new(0, 0),
            Vector2i::new(
                window_width - dir_window_width,
                window_height - font_height * 2,
            ),
            text::VerticalAlignment::AlwaysBottom,
            font_scale,
            Color::rgb(0xad, 0xd8, 0xe6),
            Color::BLACK,
            false,
            text::CursorState::Hidden,
        );

        let directory = text::Text::new(
            Vector2i::new(window_width - dir_window_width, 0),
            Vector2i::new(dir_window_width, window_height - font_height * 2),
            text::VerticalAlignment::AlwaysBottom,
            font_scale,
            Color::rgb(0xad, 0xd8, 0xe6),
            Color::BLACK,
            true,
            text::CursorState::Hidden,
        );

        let status_line = text::Text::new(
            Vector2i::new(0, window_height - font_height * 2),
            Vector2i::new(window_width, font_height),
            text::VerticalAlignment::AlwaysTop,
            font_scale,
            Color::BLACK,
            Color::rgb(0xf0, 0xc7, 0x00),
            true,
            text::CursorState::Hidden,
        );

        let command = edit::Edit::new(
            Vector2i::new(0, window_height - font_height),
            Vector2i::new(window_width, font_height),
            text::VerticalAlignment::AlwaysTop,
            font_scale,
            Color::BLACK,
            Color::WHITE,
            false,
        );

        let mut app = Self {
            font,
            font_scale,
            main_text,
            directory,
            status_line,
            command,
            window,
            dir_plain: Vec::new(),
        };

        app.update_pwd_directory();
        app
    }

    pub fn run(&mut self) {
        while self.window.is_open() {
            while let Some(event) = self.window.poll_event() {
                match event {
                    Event::Closed => self.window.close(),
                    Event::KeyPressed { code, .. } => self.key_pressed(code),
                    Event::KeyReleased { code, .. } => self.command.key_released(code),
                    Event::Resized { width, height } => {
                        self.resize_event(width as i32, height as i32)
                    }
                    _ => {}
                }
            }

            self.window.clear(Color::BLACK);
            self.main_text.draw(&mut self.window, &self.font);
            self.directory.draw(&mut self.window, &self.font);
            self.status_line.draw(&mut self.window, &self.font);
            self.command.draw(&mut self.window, &self.font);
            self.window.display();
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
        let font_width = self.font.char_size.x * self.font_scale;
        let font_height = self.font.char_size.y * self.font_scale;

        let dir_window_width = if width < 40 * font_width {
            2 * font_width
        } else {
            40 * font_width
        };

        self.main_text.set_position_size(
            Vector2i::new(0, 0),
            Vector2i::new(width - dir_window_width, height - font_height * 2),
        );
        self.directory.set_position_size(
            Vector2i::new(width - dir_window_width, 0),
            Vector2i::new(dir_window_width, height - font_height * 2),
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
        match code {
            Key::Escape => self.window.close(),
            Key::Enter => self.run_command(),
            _ => self.command.key_pressed(code),
        }
    }

    fn update_pwd_directory(&mut self) {
        let pwd = std::env::current_dir().unwrap();
        self.status_line.replace(vec![format!("{}", pwd.display())]);

        let mut dir_adorned = String::new();
        let mut dir_plain = String::new();
        for entry in pwd.read_dir().unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let file_name = path.file_name().unwrap().to_str().unwrap();
            if path.is_dir() {
                dir_adorned.push_str(&format!("{}/\n", file_name));
            } else {
                dir_adorned.push_str(&format!("{}\n", file_name));
            }
            dir_plain.push_str(&format!("{}\n", file_name));
        }
        self.dir_plain = dir_plain.lines().map(|s| s.to_string()).collect();
        self.dir_plain.sort();

        let mut dir_adorned: Vec<String> = dir_adorned.lines().map(|s| s.to_string()).collect();
        dir_adorned.sort();

        self.directory.replace(dir_adorned);
    }

    fn run_command(&mut self) {
        let command = self.command.replace(vec![]);
        // system execute
        let args = command[0].split_whitespace().collect::<Vec<_>>();
        let glob = Glob::from_vec_string(self.dir_plain.clone());
        let mut expanded = glob.glob(args[0]);
        if expanded.is_empty() {
            expanded.push(args[0].to_string());
        }
        if args.len() > 1 {
            for arg in args[1..].iter() {
                let g = glob.glob(arg);
                if g.is_empty() {
                    expanded.push(arg.to_string());
                } else {
                    expanded.extend(g);
                }
            }
        }
        let expanded_str: Vec<&str> = expanded.iter().map(AsRef::as_ref).collect();
        let (ret, output) = if let Some((ret, output)) = BuiltIn::run(&expanded_str) {
            (ret, output)
        } else if let Ok(result) = std::process::Command::new(expanded_str[0])
            .args(&expanded_str[1..])
            .output()
        {
            let stdout = String::from_utf8_lossy(&result.stdout).into_owned();
            let stderr = String::from_utf8_lossy(&result.stderr).into_owned();
            let lines: Vec<String> = stdout
                .lines()
                .chain(stderr.lines())
                .map(|s| s.to_string())
                .collect();
            (result.status.code().unwrap_or(1), lines)
        } else {
            (1, vec!["Command failed to execute".to_string()])
        };
        let output = output.join("\n");
        self.main_text.write(&format!(
            "\n`{}` returned {}\n",
            expanded_str.join(" "),
            ret
        ));
        self.main_text.write(&output);

        self.update_pwd_directory();
    }
}
