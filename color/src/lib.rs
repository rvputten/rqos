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
    Reset,
    Bold,
}

#[derive(Clone, Copy)]
enum ColorStarts {
    Regular = 30,
    Background = 40,
    HighIntensity = 90,
    BackgroundHighIntensity = 100,
}

#[allow(dead_code)]
pub struct AnsiColor {
    regular_colors: Vec<Color>,
    high_intensity_colors: Vec<Color>,
    regular_color_names: Vec<String>,
    high_intensity_color_names: Vec<String>,
    all_colors: Vec<Color>,
    all_names: Vec<String>,
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
        let mut all_names = vec!["".to_string(); start_regular];
        all_names.extend(names_regular.clone());
        all_names.extend(vec![
            "".to_string();
            start_background - start_regular - count
        ]);
        all_names.extend(names_regular.clone());
        all_names.extend(vec![
            "".to_string();
            start_high_intensity - start_background - count
        ]);
        all_names.extend(names_high_intensity.clone());
        all_names.extend(vec![
            "".to_string();
            start_background_high_intensity
                - start_high_intensity
                - count
        ]);
        all_names.extend(names_high_intensity.clone());

        AnsiColor {
            regular_colors,
            high_intensity_colors,
            regular_color_names: names_regular,
            high_intensity_color_names: names_high_intensity,
            all_colors,
            all_names,
        }
    }

    pub fn get_color_from_ansi(&self, code: usize) -> Color {
        self.all_colors[code]
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

    pub fn parse_ansi_color_code(&self, text: &str) -> (usize, Option<ColorType>) {
        if text.starts_with("\x1b[0m") {
            return (4, Some(ColorType::Reset));
        } else if text.starts_with("\x1b[m") {
            return (3, Some(ColorType::Reset));
        } else if text.starts_with("\x1b[1m") {
            return (4, Some(ColorType::Bold));
        } else if !text.starts_with("\x1b[") {
            return (0, None);
        }
        let text = &text[2..];
        let digits = text
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect::<String>();
        let code = if let Ok(code) = digits.parse::<usize>() {
            code
        } else {
            return (0, None);
        };

        let len = digits.len();
        if len == 0 || !(30..=107).contains(&code) {
            return (0, None);
        }
        let match_code = |x: ColorStarts| code >= x as usize && code <= x as usize + 7;
        let text = &text[len..];
        if text.is_empty() || !text.starts_with('m') {
            eprintln!("Invalid ANSI color code: {}", text);
            return (0, None);
        }

        let color = match code {
            _ if match_code(ColorStarts::Regular) => Some(ColorType::Regular(code)),
            _ if match_code(ColorStarts::Background) => Some(ColorType::Background(code)),
            _ if match_code(ColorStarts::HighIntensity) => Some(ColorType::HighIntensity(code)),
            _ if match_code(ColorStarts::BackgroundHighIntensity) => {
                Some(ColorType::BackgroundHighIntensity(code))
            }
            _ => {
                eprintln!("Unrecognized ANSI color code: {}", code);
                None
            }
        };
        (2 + len + 1, color)
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
        assert_eq!(color.all_names.len(), 108);
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
            (4, Some(ColorType::Reset))
        );
        assert_eq!(
            color.parse_ansi_color_code("\x1b[30m"),
            (5, Some(ColorType::Regular(30)))
        );
        assert_eq!(color.parse_ansi_color_code("\x1b[38m"), (5, None));
        assert_eq!(
            color.parse_ansi_color_code("\x1b[90m"),
            (5, Some(ColorType::HighIntensity(90)))
        );
        assert_eq!(
            color.parse_ansi_color_code("\x1b[40m"),
            (5, Some(ColorType::Background(40)))
        );
        assert_eq!(
            color.parse_ansi_color_code("\x1b[100m"),
            (6, Some(ColorType::BackgroundHighIntensity(100)))
        );
    }
}
