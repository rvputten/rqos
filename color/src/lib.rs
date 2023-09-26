/*
# Regular Colors

| ANSI | Color  | Hex Code |
| ---- | ------ | -------- |
| 30   | Black  | 0x000000 |
| 31   | Red    | 0xB22222 |
| 32   | Green  | 0x228B22 |
| 33   | Yellow | 0xF0C700 |
| 34   | Blue   | 0x0000CD |
| 35   | Purple | 0x800080 |
| 36   | Cyan   | 0x00FFFF |
| 37   | White  | 0xFFFFFF |

# High Intensity

| ANSI | Color  | Hex Code |
| ---- | ------ | -------- |
| 90   | Black  | 0x696969 |
| 91   | Red    | 0xFF4500 |
| 92   | Green  | 0x32CD32 |
| 93   | Yellow | 0xFFFF00 |
| 94   | Blue   | 0x1E90FF |
| 95   | Purple | 0x9932CC |
| 96   | Cyan   | 0x00CED1 |
| 97   | White  | 0xF8F8FF |

*/

use sfml::graphics::Color;

#[derive(PartialEq, Debug)]
pub enum ColorType {
    Regular(usize),
    Background(usize),
    HighIntensity(usize),
    BackgroundHighIntensity(usize),
    RgbFg(u8, u8, u8),
    RgbBg(u8, u8, u8),
    ResetFg,
    ResetBg,
    Reset,
    Bold,
    Italic,
}

#[derive(Clone, Copy)]
enum ColorStarts {
    Regular = 30,
    Background = 40,
    HighIntensity = 90,
    BackgroundHighIntensity = 100,
}

pub struct AnsiColor {
    regular_colors: Vec<Color>,
    high_intensity_colors: Vec<Color>,
    regular_color_names: Vec<String>,
    high_intensity_color_names: Vec<String>,
    all_colors: Vec<Color>,
}

impl AnsiColor {
    pub fn new() -> AnsiColor {
        let count = 8;
        let start_regular = ColorStarts::Regular as usize;
        let start_background = ColorStarts::Background as usize;
        let start_high_intensity = ColorStarts::HighIntensity as usize;
        let start_background_high_intensity = ColorStarts::BackgroundHighIntensity as usize;

        let empty_color = Color::BLACK;

        let regular_colors = vec![
            Color::BLACK,
            Color::rgb(0xB2, 0x22, 0x22),
            Color::rgb(0x22, 0x8B, 0x22),
            Color::rgb(0xF0, 0xC7, 0x00),
            Color::rgb(0x00, 0x00, 0xCD),
            Color::rgb(0x80, 0x00, 0x80),
            Color::rgb(0x00, 0xFF, 0xFF),
            Color::WHITE,
        ];

        let high_intensity_colors = vec![
            Color::rgb(0x69, 0x69, 0x69),
            Color::rgb(0xFF, 0x45, 0x00),
            Color::rgb(0x32, 0xCD, 0x32),
            Color::rgb(0xFF, 0xFF, 0x00),
            Color::rgb(0x1E, 0x90, 0xFF),
            Color::rgb(0x99, 0x32, 0xCC),
            Color::rgb(0x00, 0xCE, 0xD1),
            Color::rgb(0xF8, 0xF8, 0xFF),
        ];

        let mut all_colors = vec![empty_color; start_regular];
        all_colors.extend(regular_colors.clone());
        all_colors.extend(vec![empty_color; start_background - start_regular - count]);
        all_colors.extend(regular_colors.clone());
        all_colors.extend(vec![
            empty_color;
            start_high_intensity - start_background - count
        ]);
        all_colors.extend(high_intensity_colors.clone());
        all_colors.extend(vec![
            empty_color;
            start_background_high_intensity
                - start_high_intensity
                - count
        ]);
        all_colors.extend(high_intensity_colors.clone());

        let names_regular: Vec<String> = [
            "Black", "Red", "Green", "Yellow", "Blue", "Purple", "Cyan", "White",
        ]
        .iter()
        .map(|x| x.to_string())
        .collect();
        let names_high_intensity: Vec<String> = names_regular
            .iter()
            .map(|x| format!("Light {}", x))
            .collect();

        AnsiColor {
            regular_colors,
            high_intensity_colors,
            regular_color_names: names_regular,
            high_intensity_color_names: names_high_intensity,
            all_colors,
        }
    }

    pub fn get_color_from_ansi(&self, code: usize) -> Option<Color> {
        if code >= self.all_colors.len() {
            None
        } else {
            Some(self.all_colors[code])
        }
    }

    pub fn get_ansi(&self, name: &str) -> Option<usize> {
        if let Some(color) = self
            .regular_color_names
            .iter()
            .position(|x| x == name)
            .map(|x| x + ColorStarts::Regular as usize)
        {
            Some(color)
        } else {
            self.high_intensity_color_names
                .iter()
                .position(|x| x == name)
                .map(|x| x + ColorStarts::HighIntensity as usize)
        }
    }

    pub fn get_ansi_background(&self, name: &str) -> Option<usize> {
        self.get_ansi(name)
            .map(|code| code - ColorStarts::Regular as usize + ColorStarts::Background as usize)
    }

    pub fn get_color(&self, name: &str) -> Option<Color> {
        self.get_ansi(name).map(|x| self.all_colors[x])
    }

    pub fn get_color_background(&self, name: &str) -> Option<Color> {
        self.get_ansi_background(name).map(|x| self.all_colors[x])
    }

    pub fn fg(&self, name: &str) -> String {
        let code = self.get_ansi(name).unwrap_or(0);
        format!("\x1b[{}m", code)
    }

    pub fn bg(&self, name: &str) -> String {
        let code = self.get_ansi_background(name).unwrap_or(0);
        format!("\x1b[{}m", code)
    }

    pub fn reset(&self) -> String {
        "\x1b[0m".to_string()
    }

    pub fn parse_ansi_color_code(&self, text: &str) -> (usize, Vec<ColorType>) {
        if !text.starts_with("\x1b[") {
            return (0, vec![]);
        }

        let text = &text[2..];
        let mut idx = 0;
        let mut codes = vec![];
        let mut code = 0;

        loop {
            if idx >= text.len() {
                break;
            }
            let c = text.chars().nth(idx).unwrap();
            match c {
                '0'..='9' => {
                    code = code * 10 + c.to_digit(10).unwrap() as usize;
                }
                ';' => {
                    codes.push(code);
                    code = 0;
                }
                'm' => {
                    codes.push(code);
                    break;
                }
                _ => {
                    eprintln!("Invalid ANSI color code: {}", text);
                    return (0, vec![]);
                }
            }
            idx += 1;
        }

        let mut color_types = vec![];
        let len = codes.len();
        const NONE: usize = 256;
        const FAINT: usize = 2;
        const FG_S: usize = ColorStarts::Regular as usize;
        const FG_E: usize = FG_S + 7;
        const BG_S: usize = ColorStarts::Background as usize;
        const BG_E: usize = BG_S + 7;
        const HI_S: usize = ColorStarts::HighIntensity as usize;
        const HI_E: usize = HI_S + 7;
        const HIBG_S: usize = ColorStarts::BackgroundHighIntensity as usize;
        const HIBG_E: usize = HIBG_S + 7;
        codes.extend(vec![NONE; 8 - len]);
        let mut i = 0;
        loop {
            if i >= len {
                break;
            }
            match (codes[i], codes[i + 1]) {
                (0, _) => color_types.push(ColorType::Reset),
                (1, _) => color_types.push(ColorType::Bold),
                (FAINT, _) => eprintln!("ANSI color code 2 `Faint` not implemented"),
                (3, _) => color_types.push(ColorType::Italic),
                (4, _) => eprintln!("ANSI color code 4 `Underline` not implemented"),
                (9, _) => eprintln!("ANSI color code 9 `Strikethrough` not implemented"),
                (21, _) => eprintln!("ANSI color code 21 `BoldOff` not implemented"),
                (22, _) => eprintln!("ANSI color code 22 `BoldOff` not implemented"),
                (23, _) => eprintln!("ANSI color code 23 `ItalicOff` not implemented"),
                (24, _) => eprintln!("ANSI color code 24 `UnderlineOff` not implemented"),
                (25, _) => eprintln!("ANSI color code 25 `BlinkOff` not implemented"),
                (FG_S..=FG_E, _) => color_types.push(ColorType::Regular(codes[i])),
                (BG_S..=BG_E, _) => color_types.push(ColorType::Background(codes[i])),
                (HI_S..=HI_E, _) => color_types.push(ColorType::HighIntensity(codes[i])),
                (HIBG_S..=HIBG_E, _) => {
                    color_types.push(ColorType::BackgroundHighIntensity(codes[i]))
                }
                (38, 5) => {
                    if let Some(color) = self.decode_8_bit(codes[i + 2]) {
                        color_types.push(ColorType::RgbFg(color.0, color.1, color.2));
                        i += 2;
                    }
                }
                (48, 5) => {
                    if let Some(color) = self.decode_8_bit(codes[i + 2]) {
                        color_types.push(ColorType::RgbBg(color.0, color.1, color.2));
                        i += 2;
                    }
                }
                (38, 2) => {
                    if let Some(color) =
                        self.decode_24_bit(codes[i + 2], codes[i + 3], codes[i + 4])
                    {
                        color_types.push(ColorType::RgbFg(color.0, color.1, color.2));
                        i += 4;
                    }
                }
                (48, 2) => {
                    if let Some(color) =
                        self.decode_24_bit(codes[i + 2], codes[i + 3], codes[i + 4])
                    {
                        color_types.push(ColorType::RgbBg(color.0, color.1, color.2));
                        i += 4;
                    }
                }
                (39, _) => color_types.push(ColorType::ResetFg),
                (49, _) => color_types.push(ColorType::ResetBg),
                _ => {
                    eprintln!("Unrecognized ANSI color code: {}", codes[i]);
                    return (0, vec![]);
                }
            };
            i += 1;
        }
        (2 + idx + 1, color_types)
    }

    fn decode_8_bit(&self, code: usize) -> Option<(u8, u8, u8)> {
        if code > 255 {
            return None;
        }
        let decode = |code: usize| -> (u8, u8, u8) {
            let mut code = code;
            let b = code % 6;
            code /= 6;
            let g = code % 6;
            code /= 6;
            let r = code % 6;
            (r as u8, g as u8, b as u8)
        };
        if code <= 7 {
            let c = self.regular_colors[code];
            Some((c.r, c.g, c.b))
        } else if code <= 15 {
            let c = self.high_intensity_colors[code - 8];
            Some((c.r, c.g, c.b))
        } else if code < 232 {
            Some(decode(code - 16))
        } else {
            let code = code - 232;
            let r = code * 10 + code;
            let g = r;
            let b = r;
            Some((r as u8, g as u8, b as u8))
        }
    }

    fn decode_24_bit(&self, r: usize, g: usize, b: usize) -> Option<(u8, u8, u8)> {
        if r > 255 || g > 255 || b > 255 {
            return None;
        }
        Some((r as u8, g as u8, b as u8))
    }
}

impl Default for AnsiColor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_init() {
        let color = AnsiColor::new();

        assert_eq!(color.regular_colors.len(), 8);
        assert_eq!(color.high_intensity_colors.len(), 8);
        assert_eq!(color.regular_color_names.len(), 8);
        assert_eq!(color.high_intensity_color_names.len(), 8);
        assert_eq!(color.all_colors.len(), 108);
    }

    #[test]
    fn test_get_ansi() {
        let color = AnsiColor::new();

        assert_eq!(color.get_ansi("Black"), Some(30));
        assert_eq!(color.get_ansi("Red"), Some(31));
        assert_eq!(color.get_ansi("Green"), Some(32));
        assert_eq!(color.get_ansi("Yellow"), Some(33));
        assert_eq!(color.get_ansi("Blue"), Some(34));
        assert_eq!(color.get_ansi("Purple"), Some(35));
        assert_eq!(color.get_ansi("Cyan"), Some(36));
        assert_eq!(color.get_ansi("White"), Some(37));

        assert_eq!(color.get_ansi_background("Black"), Some(40));
        assert_eq!(color.get_ansi_background("Red"), Some(41));
        assert_eq!(color.get_ansi_background("Green"), Some(42));
        assert_eq!(color.get_ansi_background("Yellow"), Some(43));
        assert_eq!(color.get_ansi_background("Blue"), Some(44));
        assert_eq!(color.get_ansi_background("Purple"), Some(45));
        assert_eq!(color.get_ansi_background("Cyan"), Some(46));
        assert_eq!(color.get_ansi_background("White"), Some(47));

        assert_eq!(color.get_ansi("Light Black"), Some(90));
    }

    #[test]
    fn test_get_color() {
        let color = AnsiColor::new();

        // sample regular colors
        assert_eq!(color.get_color("Black"), Some(Color::rgb(0x00, 0x00, 0x00)));
        assert_eq!(color.get_color("Red"), Some(Color::rgb(0xB2, 0x22, 0x22)));
        assert_eq!(color.get_color("White"), Some(Color::rgb(0xFF, 0xFF, 0xFF)));

        /*
        # High Intensity

        | ANSI | Color  | Hex Code |
        | ---- | ------ | -------- |
        | 90   | Black  | 0x696969 |
        | 91   | Red    | 0xFF4500 |
        | 92   | Green  | 0x32CD32 |
        | 93   | Yellow | 0xFFFF00 |
        | 94   | Blue   | 0x1E90FF |
        | 95   | Purple | 0x9932CC |
        | 96   | Cyan   | 0x00CED1 |
        | 97   | White  | 0xF8F8FF |
        */
        // sample bright colors
        assert_eq!(
            color.get_color("Light Black"),
            Some(Color::rgb(0x69, 0x69, 0x69))
        );
        assert_eq!(
            color.get_color("Light Red"),
            Some(Color::rgb(0xFF, 0x45, 0x00))
        );

        // sample background color
        assert_eq!(
            color.get_color_background("Red"),
            Some(Color::rgb(0xB2, 0x22, 0x22))
        );

        // sample background bright color
        assert_eq!(
            color.get_color_background("Light Red"),
            Some(Color::rgb(0xFF, 0x45, 0x00))
        );
    }

    #[test]
    fn test_parse_ansi_color_code() {
        let color = AnsiColor::new();

        assert_eq!(
            color.parse_ansi_color_code("\x1b[0m"),
            (4, vec![ColorType::Reset])
        );
        assert_eq!(
            color.parse_ansi_color_code("\x1b[30m"),
            (5, vec![ColorType::Regular(30)])
        );
        assert_eq!(color.parse_ansi_color_code("\x1b[38m"), (0, vec![]));
        assert_eq!(
            color.parse_ansi_color_code("\x1b[90m"),
            (5, vec![ColorType::HighIntensity(90)])
        );
        assert_eq!(
            color.parse_ansi_color_code("\x1b[40m"),
            (5, vec![ColorType::Background(40)])
        );
        assert_eq!(
            color.parse_ansi_color_code("\x1b[100m"),
            (6, vec![ColorType::BackgroundHighIntensity(100)])
        );
        assert_eq!(
            color.parse_ansi_color_code("\x1b[1;32m"),
            (7, vec![ColorType::Bold, ColorType::Regular(32)])
        );
        assert_eq!(color.parse_ansi_color_code("\x1b[186m"), (0, vec![]));
    }
}
