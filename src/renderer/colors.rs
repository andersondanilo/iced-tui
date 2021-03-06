use iced_native::Color;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AnsiColor {
    Black,
    Red,
    DarkRed,
    Green,
    DarkGreen,
    Yellow,
    DarkYellow,
    Blue,
    DarkBlue,
    Magenta,
    DarkMagenta,
    Cyan,
    DarkCyan,
    Grey,
    White,
}

impl AnsiColor {
    pub(crate) fn alpha_code(&self) -> u8 {
        match self {
            Self::Black => 110,
            Self::Red => 111,
            Self::DarkRed => 112,
            Self::Green => 113,
            Self::DarkGreen => 114,
            Self::Yellow => 115,
            Self::DarkYellow => 116,
            Self::Blue => 117,
            Self::DarkBlue => 118,
            Self::Magenta => 119,
            Self::DarkMagenta => 120,
            Self::Cyan => 121,
            Self::DarkCyan => 122,
            Self::Grey => 123,
            Self::White => 124,
        }
    }
}

impl From<AnsiColor> for Color {
    fn from(ansi_color: AnsiColor) -> Self {
        Color::from_rgba8(0, 0, 0, ansi_color.alpha_code() as f32)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TermColor {
    Rgb(u8, u8, u8),
    Ansi(AnsiColor),
}

impl From<Color> for TermColor {
    fn from(color: Color) -> Self {
        match color.a.round() as u16 {
            110 => Self::Ansi(AnsiColor::Black),
            111 => Self::Ansi(AnsiColor::Red),
            112 => Self::Ansi(AnsiColor::DarkRed),
            113 => Self::Ansi(AnsiColor::Green),
            114 => Self::Ansi(AnsiColor::DarkGreen),
            115 => Self::Ansi(AnsiColor::Yellow),
            116 => Self::Ansi(AnsiColor::DarkYellow),
            117 => Self::Ansi(AnsiColor::Blue),
            118 => Self::Ansi(AnsiColor::DarkBlue),
            119 => Self::Ansi(AnsiColor::Magenta),
            120 => Self::Ansi(AnsiColor::DarkMagenta),
            121 => Self::Ansi(AnsiColor::Cyan),
            122 => Self::Ansi(AnsiColor::DarkCyan),
            123 => Self::Ansi(AnsiColor::Grey),
            124 => Self::Ansi(AnsiColor::White),
            _ => Self::Rgb(
                to_term_color_channel(color.r),
                to_term_color_channel(color.g),
                to_term_color_channel(color.b),
            ),
        }
    }
}

pub(crate) fn get_crossterm_color(color: TermColor) -> crossterm::style::Color {
    match color {
        TermColor::Rgb(r, g, b) => crossterm::style::Color::Rgb { r, g, b },
        TermColor::Ansi(AnsiColor::Black) => crossterm::style::Color::Black,
        TermColor::Ansi(AnsiColor::Red) => crossterm::style::Color::Red,
        TermColor::Ansi(AnsiColor::DarkRed) => crossterm::style::Color::DarkRed,
        TermColor::Ansi(AnsiColor::Green) => crossterm::style::Color::Green,
        TermColor::Ansi(AnsiColor::DarkGreen) => crossterm::style::Color::DarkGreen,
        TermColor::Ansi(AnsiColor::Yellow) => crossterm::style::Color::Yellow,
        TermColor::Ansi(AnsiColor::DarkYellow) => crossterm::style::Color::DarkYellow,
        TermColor::Ansi(AnsiColor::Blue) => crossterm::style::Color::Blue,
        TermColor::Ansi(AnsiColor::DarkBlue) => crossterm::style::Color::DarkBlue,
        TermColor::Ansi(AnsiColor::Magenta) => crossterm::style::Color::Magenta,
        TermColor::Ansi(AnsiColor::DarkMagenta) => crossterm::style::Color::DarkMagenta,
        TermColor::Ansi(AnsiColor::Cyan) => crossterm::style::Color::Cyan,
        TermColor::Ansi(AnsiColor::DarkCyan) => crossterm::style::Color::DarkCyan,
        TermColor::Ansi(AnsiColor::Grey) => crossterm::style::Color::Grey,
        TermColor::Ansi(AnsiColor::White) => crossterm::style::Color::White,
    }
}

fn to_term_color_channel(color_channel: f32) -> u8 {
    (255.0 * color_channel).round() as u8
}
