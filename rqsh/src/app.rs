use sfml::graphics::{Color, RenderTarget, RenderWindow};
use sfml::system::Vector2i;
use sfml::window::{Event, Key, Style};

use crate::builtin::BuiltIn;
use crate::glob::Glob;

enum ScrollType {
    CursorUp,
    CursorDown,
    PageUp,
    PageDown,
    MouseWheelUp,
    MouseWheelDown,
}

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

        let font_name = "font_bold";
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

        let colors = color::AnsiColor::new();
        let yellow = colors.get_color("Yellow").unwrap();
        let light_blue = colors.get_color("Light Blue").unwrap();

        let main_text = text::Text::new(
            Vector2i::new(0, 0),
            Vector2i::new(
                window_width - dir_window_width,
                window_height - font_height * 2,
            ),
            text::VerticalAlignment::AlwaysBottom,
            font_scale,
            Color::WHITE,
            Color::BLACK,
            false,
            text::CursorState::Hidden,
        );

        let directory = text::Text::new(
            Vector2i::new(window_width - dir_window_width, 0),
            Vector2i::new(dir_window_width, window_height - font_height * 2),
            text::VerticalAlignment::AlwaysBottom,
            font_scale,
            light_blue,
            Color::BLACK,
            false,
            text::CursorState::Hidden,
        );

        let status_line = text::Text::new(
            Vector2i::new(0, window_height - font_height * 2),
            Vector2i::new(window_width, font_height),
            text::VerticalAlignment::AlwaysTop,
            font_scale,
            Color::BLACK,
            yellow,
            false,
            text::CursorState::Hidden,
        );

        let command = edit::Edit::new(
            Vector2i::new(0, window_height - font_height),
            Vector2i::new(window_width, font_height),
            text::VerticalAlignment::AlwaysTop,
            font_scale,
            Color::WHITE,
            Color::BLACK,
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

        app.update_pwd_directory("", 0);
        app
    }

    pub fn run(&mut self) {
        while self.window.is_open() {
            while let Some(event) = self.window.poll_event() {
                match event {
                    Event::Closed => self.window.close(),
                    Event::KeyPressed { code, .. } => self.key_pressed(code),
                    Event::KeyReleased { code, .. } => self.command.key_released(code),
                    Event::MouseWheelScrolled { delta, .. } => {
                        if delta > 0.0 {
                            self.scroll(ScrollType::MouseWheelUp);
                        } else {
                            self.scroll(ScrollType::MouseWheelDown);
                        }
                    }
                    Event::Resized { width, height } => {
                        self.resize_event(width as i32, height as i32)
                    }
                    Event::GainedFocus => self.set_active(true),
                    Event::LostFocus => self.set_active(false),
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

    fn set_active(&mut self, active: bool) {
        self.command.set_cursor_state(if active {
            text::CursorState::Active
        } else {
            text::CursorState::Inactive
        });
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
            Key::Up => self.scroll(ScrollType::CursorUp),
            Key::Down => self.scroll(ScrollType::CursorDown),
            Key::PageUp => self.scroll(ScrollType::PageUp),
            Key::PageDown => self.scroll(ScrollType::PageDown),
            _ => self.command.key_pressed(code),
        }
    }

    fn scroll(&mut self, scroll_type: ScrollType) {
        let font_height = self.font.char_size.y * self.font_scale;
        let window_height = self.window.size().y as i32;
        let main_window_line_count = (window_height - font_height * 2) / font_height;
        let scroll_amount = match scroll_type {
            ScrollType::CursorUp => -1,
            ScrollType::CursorDown => 1,
            ScrollType::PageUp => -main_window_line_count + 1,
            ScrollType::PageDown => main_window_line_count - 1,
            ScrollType::MouseWheelUp => -4,
            ScrollType::MouseWheelDown => 4,
        };

        self.main_text.scroll_pos_y += scroll_amount;
        let text_line_count = self.main_text.text.len() as i32;

        if self.main_text.scroll_pos_y > 0 || text_line_count <= main_window_line_count {
            self.main_text.scroll_pos_y = 0;
        } else if -self.main_text.scroll_pos_y > text_line_count - main_window_line_count {
            self.main_text.scroll_pos_y = -text_line_count + main_window_line_count;
        }

        self.main_text.redraw = true;
    }

    fn update_pwd_directory(&mut self, command: &str, return_code: usize) {
        let pwd = std::env::current_dir().unwrap();
        let text = if command.is_empty() {
            format!("{}", pwd.display())
        } else {
            format!("{} ({}) {}", pwd.display(), return_code, command)
        };
        self.status_line.replace(vec![text]);

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
        self.main_text.scroll_pos_y = 0;
        let command = self.command.replace(vec![]);
        if !command.is_empty() && !command[0].is_empty() {
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
            let (return_code, output) =
                if let Some((return_code, output)) = BuiltIn::run(&expanded_str) {
                    (return_code, output)
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
            let command = expanded_str.join(" ");
            let output = output.join("\n");
            let colors = color::AnsiColor::new();
            let red_bg = format!("\x1b[{}m", colors.get_ansi_background("Red").unwrap());
            let green_bg = format!("\x1b[{}m", colors.get_ansi_background("Green").unwrap());
            let bg = if return_code == 0 { green_bg } else { red_bg };
            let black_fg = format!("\x1b[{}m", colors.get_ansi("Black").unwrap());
            let reset = "\x1b[0m";

            let main_text_window_width =
                self.main_text.get_size().x / (self.font.char_size.x * self.font_scale);
            let spaces = if main_text_window_width as usize > command.len() {
                " ".repeat(main_text_window_width as usize - command.len())
            } else {
                "".to_string()
            };

            self.main_text.write(&output);
            self.main_text.write(&format!(
                "\n{}{}`{}` returned {}{}{}\n",
                bg, black_fg, command, return_code, spaces, reset,
            ));

            self.update_pwd_directory(&command, return_code as usize);
        }
    }
}
