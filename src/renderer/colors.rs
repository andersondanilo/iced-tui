use iced_native::Color;

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

impl Into<Color> for AnsiColor {
    fn into(self) -> Color {
        Color::from_rgba8(
            0,
            0,
            0,
            match self {
                Self::Black => 110.0,
                Self::Red => 111.0,
                Self::DarkRed => 112.0,
                Self::Green => 113.0,
                Self::DarkGreen => 114.0,
                Self::Yellow => 115.0,
                Self::DarkYellow => 116.0,
                Self::Blue => 117.0,
                Self::DarkBlue => 118.0,
                Self::Magenta => 119.0,
                Self::DarkMagenta => 120.0,
                Self::Cyan => 121.0,
                Self::DarkCyan => 122.0,
                Self::Grey => 123.0,
                Self::White => 124.0,
            },
        )
    }
}

pub(crate) fn get_crossterm_color(color: Color) -> crossterm::style::Color {
    match color.a.round() as u16 {
        110 => crossterm::style::Color::Black,
        111 => crossterm::style::Color::Red,
        112 => crossterm::style::Color::DarkRed,
        113 => crossterm::style::Color::Green,
        114 => crossterm::style::Color::DarkGreen,
        115 => crossterm::style::Color::Yellow,
        116 => crossterm::style::Color::DarkYellow,
        117 => crossterm::style::Color::Blue,
        118 => crossterm::style::Color::DarkBlue,
        119 => crossterm::style::Color::Magenta,
        120 => crossterm::style::Color::DarkMagenta,
        121 => crossterm::style::Color::Cyan,
        122 => crossterm::style::Color::DarkCyan,
        123 => crossterm::style::Color::Grey,
        124 => crossterm::style::Color::White,
        _ => to_term_color(color),
    }
}

fn to_term_color(color: Color) -> crossterm::style::Color {
    crossterm::style::Color::Rgb {
        r: to_term_color_channel(color.r),
        g: to_term_color_channel(color.g),
        b: to_term_color_channel(color.b),
    }
}

fn to_term_color_channel(color_channel: f32) -> u8 {
    (255.0 * color_channel).round() as u8
}
