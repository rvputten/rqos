use sfml::graphics::{Color, RenderWindow};
use sfml::system::Vector2i;
use sfml::window::Key;

use text::Text;

// Definitions in: sfml/rust-sfml/src/ffi/window.rs
const KEYMAP_NOSHIFT: &str = "abcdefghijklmnopqrstuvwxyz01234567890.........[];,.'/\\`=- ";
const KEYMAP_SHIFT: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ)!@#$%^&*(..........{}:<>\"?|~+_ ";
// LBracket, RBracket, Semicolon, Comma, Period, Quote, Slash, Backslash, Tilde, Equal, Hyphen, Space,

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
}

pub struct Edit<'a> {
    text: Text<'a>,
    shift: bool,
    control: bool,
    pub mode: Mode,
}

impl Edit<'_> {
    pub fn new(
        position: Vector2i,
        size: Vector2i,
        vertical_alignment: text::VerticalAlignment,
        font_scale: i32,
        fg_color: Color,
        bg_color: Color,
        bold: bool,
    ) -> Self {
        let cursor_state = text::CursorState::InsertActive;
        Self {
            text: Text::new(
                position,
                size,
                vertical_alignment,
                font_scale,
                fg_color,
                bg_color,
                bold,
                cursor_state,
            ),
            shift: false,
            control: false,
            mode: Mode::Insert,
        }
    }

    pub fn set_cursor_state(&mut self, cursor_state: text::CursorState) {
        self.text.set_cursor_state(cursor_state);
    }

    pub fn get_cursor_state(&self) -> text::CursorState {
        self.text.get_cursor_state()
    }

    pub fn set_cursor_colors(&mut self, insert: Color, normal: Color) {
        self.text.set_cursor_colors(insert, normal);
    }

    pub fn write(&mut self, text: &str) {
        self.text.write(text);
    }

    pub fn replace(&mut self, text: Vec<String>) -> Vec<String> {
        self.text.replace(text)
    }

    pub fn get_text(&self) -> Vec<String> {
        self.text.get_text()
    }

    pub fn draw(&mut self, window: &mut RenderWindow, font: &font::Font) {
        self.text.draw(window, font);
    }

    pub fn set_position_size(&mut self, position: Vector2i, size: Vector2i) {
        self.text.set_position_size(position, size);
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        let old_state = self.text.get_cursor_state();
        let new_state = match (old_state, mode) {
            (text::CursorState::InsertActive, Mode::Normal) => text::CursorState::NormalActive,
            (text::CursorState::InsertInactive, Mode::Normal) => text::CursorState::NormalInactive,
            (text::CursorState::NormalActive, Mode::Insert) => text::CursorState::InsertActive,
            (text::CursorState::NormalInactive, Mode::Insert) => text::CursorState::InsertInactive,
            _ => old_state,
        };
        self.text.set_cursor_state(new_state);
    }

    pub fn key_pressed(&mut self, code: Key) {
        // Note: I know sf::Event::TextEntered exists, but so does xterm.
        let ucode = code as usize;
        let start_alphanum = Key::A as usize;
        let end_alphanum = Key::Num9 as usize;
        let start_special = Key::LBracket as usize;
        let end_special = Key::Space as usize;

        if (ucode >= start_alphanum && ucode <= end_alphanum)
            || (ucode >= start_special && ucode <= end_special)
        {
            let mode = self.mode;
            const S_DN: bool = true;
            const S_UP: bool = false;
            const C_DN: bool = true;
            const C_UP: bool = false;
            match (mode, self.shift, self.control) {
                (Mode::Insert, S_UP, C_UP) => self.write(&KEYMAP_NOSHIFT[ucode..ucode + 1]),
                (Mode::Insert, S_DN, C_UP) => self.write(&KEYMAP_SHIFT[ucode..ucode + 1]),
                (Mode::Insert, S_UP, C_DN) => match code {
                    Key::H => self.backspace(),
                    Key::U => self.text.clear(),
                    Key::LBracket | Key::J => {
                        self.set_mode(Mode::Normal);
                        println!("Mode: Normal");
                    }
                    _ => {}
                },
                (Mode::Insert, S_DN, C_DN) => {}
                // normal mode
                (Mode::Normal, S_UP, C_UP) => {
                    if ucode == Key::I as usize || ucode == Key::A as usize {
                        self.set_mode(Mode::Insert);
                        println!("Mode: Insert");
                    }
                }
                (Mode::Normal, S_DN, C_UP) => {}
                (Mode::Normal, S_UP, C_DN) => {}
                (Mode::Normal, S_DN, C_DN) => {}
            }
        } else {
            match code {
                Key::LShift | Key::RShift => self.shift_pressed(true),
                Key::LControl | Key::RControl => self.control_pressed(true),
                Key::Backspace => self.backspace(),
                _ => {}
            }
        }
    }

    pub fn key_released(&mut self, code: Key) {
        match code {
            Key::LShift | Key::RShift => self.shift_pressed(false),
            Key::LControl | Key::RControl => self.control_pressed(false),
            _ => {}
        }
    }

    fn backspace(&mut self) {
        self.text.redraw = true;
        let text = &mut self.text.text;
        let cursor_position = &mut self.text.cursor_position;
        if cursor_position.x > 0 {
            let line = &mut text[cursor_position.y as usize];
            let idx = cursor_position.x as usize;
            line.remove(idx - 1);
            cursor_position.x -= 1;
        } else if cursor_position.y > 0 {
            let line = text.remove(cursor_position.y as usize);
            cursor_position.y -= 1;
            cursor_position.x = text[cursor_position.y as usize].len() as i32;
            text[cursor_position.y as usize] += &line;
        }
    }

    fn shift_pressed(&mut self, is_down: bool) {
        self.shift = is_down;
    }

    fn control_pressed(&mut self, is_down: bool) {
        self.control = is_down;
    }
}
