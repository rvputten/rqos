//use crate::color;
use sfml::graphics::Color;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AnsiCode {
    Char(char),
    ClearFromCursorDown,
    ClearFromCursorUp,
    ClearEntireScreen,
    ClearFromCursorToEndOfLine,
    ClearFromCursorToStartOfLine,
    ClearEntireLine,
    MoveCursorRelative(i32, i32),
    MoveCursorAbsoluteX(i32),
    MoveCursorAbsoluteY(i32),
    ScrollScreen(i32),
    ColorForeground(Color),
    ColorBackground(Color),
    Bold,
    Underline,
    Italic,
    Reverse,
    ResetColorForeground,
    ResetColorBackground,
    ResetAll, // reset color, bold, italic, underline, reverse etc.
    ClearScrollbackBuffer,
}

pub struct AnsiWrap {
    pub codes: Vec<AnsiCode>,
    pub char_count: usize,
}

pub struct Ansi {}

#[derive(PartialEq, Eq, Clone, Copy)]
enum State {
    Normal,
    Escape,
    Number,
    Code,
}

impl Ansi {
    /// Takes a string and returns a vector of AnsiCode
    /// and the number of characters that have been parsed as ansi codes.
    /// The string may contain chars beyond the ansi codes.
    /// AnsiWrap::char_count can be used to get the number of chars that have been parsed.
    ///
    /// | ANSI | Code                                            |
    /// | ---- | ----------------------------------------------- |
    /// | nA   | move cursor up                                  |
    /// | nB   | move cursor down                                |
    /// | nC   | move cursor forward                             |
    /// | nD   | move cursor backward                            |
    /// | nE   | move cursor down and to start of line           |
    /// | nF   | move cursor up and to start of line             |
    /// | nG   | move cursor to column n                         |
    /// | H    | home                                            |
    /// | n;mH | move cursor to row n, column m                  |
    /// | J    | clear to end of screen (also: 0J)               |
    /// | 1J   | clear to start of screen                        |
    /// | 2J   | clear entire screen                             |
    /// | 3J   | clear entire screen and scrollback buffer       |
    /// | K    | clear to end of line                            |
    /// | 2K   | clear entire line                               |
    /// | nS   | scroll up                                       |
    /// | nT   | scroll down                                     |
    /// | ---- | ----------------------------------------------- |
    pub fn parse(text: &str) -> AnsiWrap {
        let mut codes = Vec::new();
        let mut char_count = 0;
        let chars = text.chars();
        let mut state = State::Normal;
        let mut numbers: Vec<i32> = Vec::new();
        let color_codes = color::AnsiColor::new();
        for c in chars {
            let old_state = state;
            if old_state == State::Normal {
                if c == '\x1b' {
                    char_count += 1;
                    state = State::Escape;
                    numbers.clear();
                    continue;
                } else {
                    return AnsiWrap { codes, char_count };
                }
            }

            if state != State::Normal {
                char_count += 1;
            }

            if old_state == State::Escape {
                if c == '[' {
                    state = State::Number;
                    continue;
                } else {
                    state = State::Normal;
                    codes.push(AnsiCode::Char(c));
                    eprintln!("ansi: unknown escape sequence: {}", c);
                    continue;
                }
            }

            if old_state == State::Number {
                if c.is_ascii_digit() {
                    let n = numbers.pop().unwrap_or(0);
                    numbers.push(n * 10 + c.to_digit(10).unwrap() as i32);
                    continue;
                } else if c == ';' || c == ':' {
                    numbers.push(0);
                    continue;
                } else {
                    state = State::Code;
                }
            }

            if state == State::Code {
                let number1 = numbers.first().cloned();
                let number2 = numbers.get(1).cloned();
                match c {
                    'A' => codes.push(AnsiCode::MoveCursorRelative(0, -number1.unwrap_or(1))),
                    'B' => codes.push(AnsiCode::MoveCursorRelative(0, number1.unwrap_or(1))),
                    'C' => codes.push(AnsiCode::MoveCursorRelative(number1.unwrap_or(1), 0)),
                    'D' => codes.push(AnsiCode::MoveCursorRelative(-number1.unwrap_or(1), 0)),
                    'E' => codes.push(AnsiCode::MoveCursorRelative(0, number1.unwrap_or(1))),
                    'F' => codes.push(AnsiCode::MoveCursorRelative(0, -number1.unwrap_or(1))),
                    'G' => codes.push(AnsiCode::MoveCursorAbsoluteX(number1.unwrap_or(1))),
                    'H' => codes.push(AnsiCode::MoveCursorAbsoluteY(number1.unwrap_or(1))),
                    'J' => {
                        if number1 == Some(0) || number1.is_none() {
                            codes.push(AnsiCode::ClearFromCursorToEndOfLine);
                        } else if number1 == Some(1) {
                            codes.push(AnsiCode::ClearFromCursorToStartOfLine);
                        } else if number1 == Some(2) {
                            codes.push(AnsiCode::ClearEntireLine);
                        } else if number1 == Some(3) {
                            codes.push(AnsiCode::ClearScrollbackBuffer);
                        } else {
                            eprintln!("ansi: unknown escape sequence: {}J", number1.unwrap());
                        }
                    }
                    'K' => {
                        if number1 == Some(0) || number1.is_none() {
                            codes.push(AnsiCode::ClearFromCursorToEndOfLine);
                        } else if number1 == Some(1) {
                            codes.push(AnsiCode::ClearFromCursorToStartOfLine);
                        } else if number1 == Some(2) {
                            codes.push(AnsiCode::ClearEntireLine);
                        } else {
                            eprintln!("ansi: unknown escape sequence: {}K", number1.unwrap());
                        }
                    }
                    'S' => codes.push(AnsiCode::ScrollScreen(number1.unwrap_or(1))),
                    'T' => codes.push(AnsiCode::ScrollScreen(-number1.unwrap_or(1))),
                    'm' => {
                        if numbers.is_empty() {
                            codes.push(AnsiCode::ResetAll);
                            continue;
                        }
                        loop {
                            if numbers.is_empty() {
                                break;
                            }
                            let n = numbers.remove(0);
                            if n == 0 {
                                codes.push(AnsiCode::ResetAll);
                            } else {
                                match n {
                                    1 => codes.push(AnsiCode::Bold),
                                    3 => codes.push(AnsiCode::Italic),
                                    4 => codes.push(AnsiCode::Underline),
                                    7 => codes.push(AnsiCode::Reverse),
                                    22 => codes.push(AnsiCode::ResetAll),
                                    23 => codes.push(AnsiCode::ResetAll),
                                    24 => codes.push(AnsiCode::ResetAll),
                                    27 => codes.push(AnsiCode::ResetAll),
                                    30..=37 => {
                                        if let Some(n) = color_codes.get_color_from_ansi(n as usize)
                                        {
                                            codes.push(AnsiCode::ColorForeground(n));
                                        } else {
                                            eprintln!("ansi: unknown escape sequence 1: {}m", n);
                                        }
                                    }
                                    38 => {
                                        if number2 == Some(5) {
                                            let n = number1.unwrap();
                                            if let Some(n) =
                                                color_codes.get_color_from_ansi(n as usize)
                                            {
                                                codes.push(AnsiCode::ColorForeground(n));
                                            } else {
                                                eprintln!(
                                                    "ansi: unknown escape sequence 2: {}m",
                                                    n
                                                );
                                            }
                                        } else {
                                            eprintln!("ansi: unknown escape sequence 3: {}m", n);
                                        }
                                    }
                                    39 => codes.push(AnsiCode::ResetColorForeground),
                                    40..=47 => {
                                        if let Some(n) = color_codes.get_color_from_ansi(n as usize)
                                        {
                                            codes.push(AnsiCode::ColorBackground(n));
                                        } else {
                                            eprintln!("ansi: unknown escape sequence 4: {}m", n);
                                        }
                                    }
                                    48 => {
                                        if number2 == Some(5) {
                                            let n = number1.unwrap();
                                            if let Some(n) =
                                                color_codes.get_color_from_ansi(n as usize)
                                            {
                                                codes.push(AnsiCode::ColorBackground(n));
                                            } else {
                                                eprintln!(
                                                    "ansi: unknown escape sequence 5: {}m",
                                                    n
                                                );
                                            }
                                        } else {
                                            eprintln!("ansi: unknown escape sequence 6: {}m", n);
                                        }
                                    }
                                    49 => codes.push(AnsiCode::ResetColorBackground),
                                    _ => eprintln!("ansi: unknown escape sequence: {}m", n),
                                }
                            }
                        }
                    }
                    _ => eprintln!("ansi: unknown escape sequence: {:?}{}", numbers.first(), c),
                }
                state = State::Normal;
            }
        }
        AnsiWrap { codes, char_count }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansi() {
        let text = "\x1b[1;31mHello\x1b[0m";
        let color_codes = color::AnsiColor::new();
        let red = color_codes.get_color_from_ansi(31).unwrap();
        let codes = Ansi::parse(text);
        assert_eq!(codes.codes.len(), 2);
        assert_eq!(codes.codes[0], AnsiCode::Bold);
        assert_eq!(codes.codes[1], AnsiCode::ColorForeground(red));
        assert_eq!(codes.char_count, 7);
    }

    #[test]
    fn test_ansi2() {
        let text = "Hello\x1b[0m";
        let codes = Ansi::parse(text);
        assert_eq!(codes.codes.len(), 0);
        assert_eq!(codes.char_count, 0);
    }
}
