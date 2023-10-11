use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;

use sfml::graphics::{Color, RenderTarget, RenderWindow};
use sfml::system::Vector2i;
use sfml::window::{Event, Key, Style};

use crate::args::Args;
use crate::builtin::Builtin;
use crate::execute::{BuiltinCommand, ExecMessage, Job};
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
    main_win: text::Text<'a>,
    status_win: text::Text<'a>,
    command_win: edit::Edit,
    info_win: text::Text<'a>,
    info_active: bool,
    info_command_tmp: String,
    info_text: Vec<String>,
    info_selection: usize,
    info_cursor_x: usize,
    command_bg_color_normal: Color,
    command_bg_color_running: Color,
    window: RenderWindow,
    dir_plain: Vec<String>,
    jobs: Vec<Job>,
    browse_job_history_idx: usize,
    tx: mpsc::Sender<ExecMessage>,
    rx: mpsc::Receiver<ExecMessage>,
    stdin_tx: Option<mpsc::Sender<String>>,
    stop_thread: Option<Arc<AtomicBool>>,
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

        let main_win = text::TextBuilder::new()
            .position(Vector2i::new(0, 0))
            .size(Vector2i::new(window_width, window_height - font_height * 2))
            .vertical_alignment(text::VerticalAlignment::AlwaysBottom)
            .build();

        let status_win = text::TextBuilder::new()
            .position(Vector2i::new(0, window_height - font_height * 2))
            .size(Vector2i::new(window_width, font_height))
            .fg_color(Color::BLACK)
            .bg_color(yellow)
            .build();

        let command_bg_color_normal = Color::WHITE;
        let command_bg_color_running = light_blue;
        let command_win = edit::EditBuilder::new()
            .cursor_colors(Color::BLACK, yellow)
            .build();

        let info_win = text::TextBuilder::new()
            .fg_color(Color::BLACK)
            .bg_color(Color::WHITE)
            .build();

        let (tx, rx) = mpsc::channel();
        let mut app = Self {
            font,
            font_scale,
            main_win,
            status_win,
            command_win,
            command_bg_color_normal,
            command_bg_color_running,
            info_win,
            info_command_tmp: String::new(),
            info_text: Vec::new(),
            info_selection: 0,
            info_cursor_x: 0,
            info_active: false,
            window,
            dir_plain: Vec::new(),
            jobs: Vec::new(),
            browse_job_history_idx: 0,
            tx,
            rx,
            stdin_tx: None,
            stop_thread: None,
            colors,
        };

        app.update_pwd_directory();
        app
    }

    pub fn run(&mut self) {
        let mut frame_counter = 0;
        while self.window.is_open() {
            let t = std::time::Instant::now();
            while let Some(event) = self.window.poll_event() {
                match event {
                    Event::Closed => self.window.close(),
                    Event::KeyPressed { code, .. } => self.key_pressed(code),
                    Event::KeyReleased { code, .. } => self.command_win.key_released(code),
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

            if self.main_win.must_draw()
                || self.status_win.must_draw()
                || self.command_win.must_draw()
                || self.info_win.must_draw()
                || frame_counter > 200
            {
                self.window.clear(Color::BLACK);
                self.main_win.draw(&mut self.window, &self.font);
                self.status_win.draw(&mut self.window, &self.font);
                self.command_win.draw(&mut self.window, &self.font);
                self.info_win.draw(&mut self.window, &self.font);

                self.window.display();
                frame_counter = 0;
            }
            frame_counter += 1;

            let elapsed = t.elapsed();
            let frame_diff = 16 - elapsed.as_millis() as i32;
            if frame_diff > 0 {
                std::thread::sleep(std::time::Duration::from_millis(frame_diff as u64));
            }
        }
    }

    fn set_active(&mut self, active: bool) {
        let old_state = self.command_win.get_cursor_state();
        self.command_win
            .set_cursor_state(match (old_state, active) {
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

        let status_win_height = font_height;
        let command_win_height = font_height;
        let info_win_height = font_height * 3;
        let main_win_height = height - status_win_height - command_win_height - info_win_height;

        let main_win_pos_y = 0;
        let status_win_pos_y = main_win_height;
        let command_win_pos_y = main_win_height + status_win_height;
        let info_win_pos_y = main_win_height + status_win_height + command_win_height;

        self.main_win.set_position_size(
            Vector2i::new(0, main_win_pos_y),
            Vector2i::new(width, main_win_height),
        );
        self.status_win.set_position_size(
            Vector2i::new(0, status_win_pos_y),
            Vector2i::new(width, status_win_height),
        );
        self.command_win.set_position_size(
            Vector2i::new(0, command_win_pos_y),
            Vector2i::new(width, command_win_height),
        );
        self.info_win.set_position_size(
            Vector2i::new(0, info_win_pos_y),
            Vector2i::new(width, info_win_height),
        );
    }

    fn key_pressed(&mut self, code: Key) {
        if self.command_win.mode == edit::Mode::Normal {
            self.normal_mode_key_pressed(code);
        } else {
            self.insert_mode_key_pressed(code);
        }
        self.update_info_win();
    }

    fn insert_mode_key_pressed(&mut self, code: Key) {
        if self.command_win.control {
            match code {
                Key::C => self.kill_job(),
                Key::D => self.send_eof(),
                Key::N => self.change_selection(1),
                Key::P => self.change_selection(-1),
                _ => self.command_win.key_pressed(code),
            }
        } else {
            let tab_dir = if self.command_win.shift { -1 } else { 1 };
            match code {
                Key::Tab => self.change_selection(tab_dir),
                Key::Enter => self.on_enter(),
                Key::Up => self.scroll(ScrollType::CursorUp),
                Key::Down => self.scroll(ScrollType::CursorDown),
                Key::PageUp => self.scroll(ScrollType::PageUp),
                Key::PageDown => self.scroll(ScrollType::PageDown),
                _ => self.command_win.key_pressed(code),
            }
        }
    }

    fn normal_mode_key_pressed(&mut self, code: Key) {
        let old_idx = self.browse_job_history_idx as i32;
        let old_command = self.command_win.get_text()[0].clone();
        let mut update_job_idx = |delta: i32| {
            if self.jobs.is_empty() {
                return;
            }
            if old_command.is_empty() {
                self.command_win
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
            self.command_win.replace(vec![new_command]);
        };

        match code {
            Key::Enter => {
                self.run_command();
                self.command_win.set_mode(edit::Mode::Insert);
            }
            Key::Escape => self.window.close(),
            Key::K => update_job_idx(-1),
            Key::J => update_job_idx(1),
            _ => self.command_win.key_pressed(code),
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

        self.main_win.scroll_pos_y += scroll_amount;
        let text_line_count = self.main_win.text.len() as i32;

        if self.main_win.scroll_pos_y > 0 || text_line_count <= main_window_line_count {
            self.main_win.scroll_pos_y = 0;
        } else if -self.main_win.scroll_pos_y > text_line_count - main_window_line_count {
            self.main_win.scroll_pos_y = -text_line_count + main_window_line_count;
        }

        self.main_win.redraw = true;
    }

    fn create_info_text(&self) -> Vec<String> {
        let command_text = self.command_win.get_text()[0].clone();
        let args = Args::new_notrim(&command_text).args;

        let len = args.len();
        match len {
            0 => self.jobs.iter().rev().map(|j| j.args_printable()).collect(),
            1 => {
                let arg = &args[0];
                let mut matches = vec![];
                for job in self.jobs.iter().rev() {
                    if job.args_printable().starts_with(arg) {
                        matches.push(job.args_printable());
                    }
                }
                matches
            }
            _ => {
                let last_arg = &format!("{}*", args[len - 1]);
                let glob = Glob::from_path(".").unwrap();
                glob.match_path_multiple(last_arg)
            }
        }
    }

    fn change_selection(&mut self, delta: i32) {
        if !self.info_active {
            self.info_active = true;
            self.info_command_tmp = self.command_win.get_text()[0].clone();
            self.info_selection = 0;
            self.info_cursor_x = self.info_command_tmp.len();
        } else {
            self.info_selection = (self.info_selection as i32 + delta)
                .max(0)
                .min(self.info_text.len() as i32 - 1) as usize;
            // remove padded lines
            while self.info_selection > 0 && self.info_text[self.info_selection].is_empty() {
                self.info_selection -= 1;
            }
        }
    }

    fn reset_info_win(&mut self) {
        self.info_text = vec![];
        self.info_win.redraw = true;
        self.info_command_tmp = self.command_win.get_text()[0].clone();
        self.info_active = false;
    }

    fn check_info_change(&mut self) {
        let command = if self.command_win.get_text().is_empty() {
            "".to_string()
        } else {
            self.command_win.get_text()[0].clone()
        };
        if command == self.info_command_tmp {
            return;
        }
        self.reset_info_win();
    }

    fn update_info_win(&mut self) {
        self.check_info_change();
        let len = self.info_text.len();
        if len == 0 {
            self.info_text = self.create_info_text();
        }
        let info_text_lines = 3;
        let add_empty_lines =
            (info_text_lines - self.info_text.len() % info_text_lines) % info_text_lines;
        self.info_text.extend(vec![String::new(); add_empty_lines]);

        let max_len: usize = 60;
        let many_spaces = " ".repeat(max_len);

        let len = self.info_text.len();
        let mut lines = vec![String::new(); info_text_lines];
        let highlight_color_bg = self.colors.bg("Light Blue");
        let highlight_color_fg = self.colors.fg("White");
        let reset = self.colors.reset();
        for idx in (0..len).step_by(info_text_lines) {
            let max_cnt = self.info_text[idx..idx + info_text_lines]
                .iter()
                .map(|s| s.len())
                .max()
                .unwrap()
                + 1;
            let max_cnt = max_cnt.min(max_len);
            for (i, line) in self.info_text[idx..idx + info_text_lines]
                .iter()
                .enumerate()
            {
                let line = if line.len() > max_len - 1 {
                    format!("{}...", &line[..max_len - 4])
                } else {
                    line.clone()
                };
                let extra_spaces = &many_spaces[..max_cnt - line.len()];
                if self.info_active && idx + i == self.info_selection {
                    lines[i] += &format!(
                        "{}{}{}{}{}",
                        highlight_color_bg, highlight_color_fg, line, extra_spaces, reset
                    );
                } else {
                    lines[i] += &line;
                    lines[i] += extra_spaces;
                }
            }
        }

        self.info_win.replace(lines);
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
        self.status_win.replace(vec![text]);

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

        self.update_info_win();
    }

    fn on_enter(&mut self) {
        if self.info_active {
            self.info_active = false;

            let mut args = Args::new_notrim(&self.info_command_tmp);
            let selection = self.info_text[self.info_selection].clone();
            let command = match args.args.len() {
                0 | 1 => selection,
                _ => {
                    args.args.pop();
                    args.args.push(selection);
                    args.printable()
                }
            };
            self.command_win.replace(vec![command]);
        } else {
            self.reset_info_win();
        }
        self.run_command();
    }

    fn run_command(&mut self) {
        let pwd = std::env::current_dir().unwrap();

        self.main_win.scroll_pos_y = 0;
        let command = self.command_win.replace(vec![]);
        let command = if command.is_empty() {
            "".to_string()
        } else {
            command[0].trim().to_string()
        };
        if self.stdin_tx.is_some() {
            let send_string = if !command.trim().is_empty() {
                format!("{}\n", command)
            } else {
                "\n".to_string()
            };
            self.stdin_tx.as_ref().unwrap().send(send_string).unwrap();
        } else {
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

            self.main_win.write(&format!(
                "{}{}{} {}> {}{}\n",
                self.colors.bg("Yellow"),
                self.colors.fg("Black"),
                job_id,
                pwd.display(),
                job.args_printable(),
                self.colors.reset()
            ));

            self.stop_thread = Builtin::run(self.tx.clone(), job);
        }
        self.update_info_win();
    }

    fn write_intermediate_status_win(&mut self) {
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

        let main_win_window_width =
            self.main_win.get_size().x / (self.font.char_size.x * self.font_scale);
        let spaces = if main_win_window_width as usize > command.len() {
            " ".repeat(main_win_window_width as usize - command.len())
        } else {
            "".to_string()
        };

        match (job.start_time, job.end_time) {
            (Some(start), Some(end)) => {
                let duration = end.duration_since(start).unwrap();
                let duration = format!("{}.{:03}s", duration.as_secs(), duration.subsec_millis());
                self.main_win.write(&format!(
                    "\n{}{}`{}` returned {} in {}{}{}\n",
                    bg, fg, command, return_code, duration, spaces, reset,
                ));
            }
            _ => self.main_win.write(&format!(
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
                self.command_win
                    .set_background_color(self.command_bg_color_running);
            }
            ExecMessage::StdOut(output) | ExecMessage::StdErr(output) => {
                self.main_win.write(&output);
                self.main_win.redraw = true;
            }
            ExecMessage::JobDone(job) => {
                self.end_job();
                self.jobs.push(job);
                self.update_pwd_directory();
                self.write_intermediate_status_win();
                self.reset_info_win();
                self.update_info_win();
            }
            ExecMessage::BuiltinCommand(cmd) => self.handle_builtin_command(cmd),
        }
    }

    fn handle_builtin_command(&mut self, cmd: BuiltinCommand) {
        match cmd {
            BuiltinCommand::Jobs => Builtin::jobs(self.tx.clone(), &self.jobs),
        };
    }

    fn send_eof(&mut self) {
        self.stdin_tx = None;
        self.command_win
            .set_background_color(self.command_bg_color_normal);
    }

    fn kill_job(&mut self) {
        if let Some(stop_thread) = self.stop_thread.take() {
            stop_thread.store(true, Ordering::SeqCst);
        }
    }

    fn end_job(&mut self) {
        self.send_eof();
    }
}
