use std::sync::mpsc;

use sfml::graphics::{Color, RenderTarget, RenderWindow};
use sfml::system::Vector2i;
use sfml::window::{Event, Key, Style};

use crate::args::Args;
use crate::builtin::BuiltIn;
use crate::execute::{ExecMessage, Job};
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
    status_line: text::Text<'a>,
    command: edit::Edit,
    command_bg_color_normal: Color,
    command_bg_color_running: Color,
    window: RenderWindow,
    dir_plain: Vec<String>,
    jobs: Vec<Job>,
    browse_job_history_idx: usize,
    tx: mpsc::Sender<ExecMessage>,
    rx: mpsc::Receiver<ExecMessage>,
    stdin_tx: Option<mpsc::Sender<String>>,
    colors: color::AnsiColor,
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

        let font_height = font_size.y * font_scale;
        let (cols, rows) = (160, 80);
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

        let main_text = text::TextBuilder::new()
            .position(Vector2i::new(0, 0))
            .size(Vector2i::new(window_width, window_height - font_height * 2))
            .vertical_alignment(text::VerticalAlignment::AlwaysBottom)
            .build();

        let status_line = text::TextBuilder::new()
            .position(Vector2i::new(0, window_height - font_height * 2))
            .size(Vector2i::new(window_width, font_height))
            .fg_color(Color::BLACK)
            .bg_color(yellow)
            .build();

        let command_bg_color_normal = Color::WHITE;
        let command_bg_color_running = light_blue;
        let command = edit::EditBuilder::new()
            .cursor_colors(Color::BLACK, yellow)
            .build();

        let (tx, rx) = mpsc::channel();
        let mut app = Self {
            font,
            font_scale,
            main_text,
            status_line,
            command,
            command_bg_color_normal,
            command_bg_color_running,
            window,
            dir_plain: Vec::new(),
            jobs: Vec::new(),
            browse_job_history_idx: 0,
            tx,
            rx,
            stdin_tx: None,
            colors,
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

            while let Ok(message) = self.rx.try_recv() {
                self.handle_exec_messages(message);
            }

            self.window.clear(Color::BLACK);
            self.main_text.draw(&mut self.window, &self.font);
            self.status_line.draw(&mut self.window, &self.font);
            self.command.draw(&mut self.window, &self.font);
            self.window.display();
        }
    }

    fn set_active(&mut self, active: bool) {
        let old_state = self.command.get_cursor_state();
        self.command.set_cursor_state(match (old_state, active) {
            (text::CursorState::NormalActive, false) => text::CursorState::NormalInactive,
            (text::CursorState::NormalInactive, true) => text::CursorState::NormalActive,
            (text::CursorState::InsertActive, false) => text::CursorState::InsertInactive,
            (text::CursorState::InsertInactive, true) => text::CursorState::InsertActive,
            (state, _) => state,
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
        if self.command.mode == edit::Mode::Normal {
            self.normal_mode_key_pressed(code);
        } else {
            self.insert_mode_key_pressed(code);
        }
    }

    fn insert_mode_key_pressed(&mut self, code: Key) {
        if self.command.control {
            if code == Key::D {
                self.end_job();
            } else {
                self.command.key_pressed(code);
            }
        } else {
            match code {
                Key::Enter => self.run_command(),
                Key::Up => self.scroll(ScrollType::CursorUp),
                Key::Down => self.scroll(ScrollType::CursorDown),
                Key::PageUp => self.scroll(ScrollType::PageUp),
                Key::PageDown => self.scroll(ScrollType::PageDown),
                _ => self.command.key_pressed(code),
            }
        }
    }

    fn normal_mode_key_pressed(&mut self, code: Key) {
        let old_idx = self.browse_job_history_idx as i32;
        let old_command = self.command.get_text()[0].clone();
        let mut update_job_idx = |delta: i32| {
            if self.jobs.is_empty() {
                return;
            }
            if old_command.is_empty() {
                self.command
                    .replace(vec![self.jobs[old_idx as usize].args_printable()]);
                return;
            }
            let job_count = self.jobs.len() as i32;
            let mut idx = old_idx + delta;
            if idx < 0 {
                idx = 0;
            } else if idx >= job_count {
                idx = job_count - 1;
            }
            let mut new_command;
            loop {
                self.browse_job_history_idx = idx.min(job_count - 1).max(0) as usize;
                new_command = self.jobs[self.browse_job_history_idx].args_printable();
                if new_command.as_str() != old_command.as_str()
                    || (idx == 0 || idx == job_count - 1)
                {
                    break;
                }
                idx += delta;
            }
            self.command.replace(vec![new_command]);
        };

        match code {
            Key::Enter => {
                self.run_command();
                self.command.set_mode(edit::Mode::Insert);
            }
            Key::Escape => self.window.close(),
            Key::K => update_job_idx(-1),
            Key::J => update_job_idx(1),
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

    fn update_pwd_directory(&mut self) {
        let pwd = std::env::current_dir().unwrap();
        let text = if let Some(job) = self.jobs.last() {
            let return_code = job.return_code.unwrap_or(0);
            let command = job.args_printable();
            format!("{} ({}) {}", pwd.display(), return_code, command)
        } else {
            format!("{}", pwd.display())
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
    }

    fn run_command(&mut self) {
        let pwd = std::env::current_dir().unwrap();

        self.main_text.scroll_pos_y = 0;
        let command = self.command.replace(vec![]);
        if self.stdin_tx.is_some() {
            let send_string = if !command.is_empty() && !command[0].trim().is_empty() {
                format!("{}\n", command[0])
            } else {
                "\n".to_string()
            };
            self.stdin_tx.as_ref().unwrap().send(send_string).unwrap();
        } else if !command.is_empty() && !command[0].trim().is_empty() {
            let command = command[0].trim().to_string();
            let args = Args::new(&command).args;
            let glob = Glob::from_path(".").unwrap();
            let mut expanded_args = glob.match_path_single(&args[0]);
            if expanded_args.is_empty() {
                expanded_args.push(args[0].to_string());
            }
            if args.len() > 1 {
                for arg in args[1..].iter() {
                    let g = glob.match_path_multiple(arg);
                    if g.is_empty() {
                        expanded_args.push(arg.to_string());
                    } else {
                        expanded_args.extend(g);
                    }
                }
            }

            let job = Job::new(expanded_args);

            let job_id = self.jobs.len();
            self.browse_job_history_idx = job_id;

            self.main_text.write(&format!(
                "{}{}{} {}> {}{}\n",
                self.colors.bg("Yellow"),
                self.colors.fg("Black"),
                job_id,
                pwd.display(),
                job.args_printable(),
                self.colors.reset()
            ));

            BuiltIn::run(self.tx.clone(), job);
        }
    }

    fn write_intermediate_status_line(&mut self) {
        let job = self.jobs.last().unwrap();
        let command = job.args_printable();
        let return_code = job.return_code.unwrap();
        let colors = color::AnsiColor::new();
        let bg = if return_code == 0 {
            colors.bg("Green")
        } else {
            colors.bg("Red")
        };
        let fg = colors.fg("Black");
        let reset = colors.reset();

        let main_text_window_width =
            self.main_text.get_size().x / (self.font.char_size.x * self.font_scale);
        let spaces = if main_text_window_width as usize > command.len() {
            " ".repeat(main_text_window_width as usize - command.len())
        } else {
            "".to_string()
        };

        match (job.start_time, job.end_time) {
            (Some(start), Some(end)) => {
                let duration = end.duration_since(start).unwrap();
                let duration = format!("{}.{:03}s", duration.as_secs(), duration.subsec_millis());
                self.main_text.write(&format!(
                    "\n{}{}`{}` returned {} in {}{}{}\n",
                    bg, fg, command, return_code, duration, spaces, reset,
                ));
            }
            _ => self.main_text.write(&format!(
                "\n{}{}`{}` returned {}{}{}\n",
                bg, fg, command, return_code, spaces, reset,
            )),
        };

        self.update_pwd_directory();
    }

    fn handle_exec_messages(&mut self, message: ExecMessage) {
        match message {
            ExecMessage::StdInQueue(tx) => {
                self.stdin_tx = Some(tx);
                self.command
                    .set_background_color(self.command_bg_color_running);
            }
            ExecMessage::StdOut(output) | ExecMessage::StdErr(output) => {
                self.main_text.write(&output);
                self.main_text.redraw = true;
            }
            ExecMessage::JobDone(job) => {
                self.stdin_tx = None;
                self.command
                    .set_background_color(self.command_bg_color_normal);
                self.jobs.push(job);
                self.update_pwd_directory();
                self.write_intermediate_status_line();
            }
        }
    }

    fn end_job(&mut self) {
        self.stdin_tx = None;
        self.command
            .set_background_color(self.command_bg_color_normal);
    }
}
