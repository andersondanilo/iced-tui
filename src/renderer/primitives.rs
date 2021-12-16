use core::fmt::Debug;
use iced_native::Color;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

#[derive(Debug, Clone)]
pub enum Primitive {
    Cell(u16, u16, Cell),
    Rectangle(u16, u16, u16, u16, Cell),
    Group(Vec<Primitive>),
    CursorPosition(u16, u16, CursorStyle),
}

impl Primitive {
    pub fn from_char(x: u16, y: u16, content: char) -> Self {
        Self::Cell(x, y, Cell::from_char(content))
    }
}

impl std::cmp::PartialEq for Primitive {
    fn eq(&self, rhs: &Self) -> bool {
        match self {
            Self::Cell(x, y, cell) => match rhs {
                Self::Cell(rhs_x, rhs_y, rhs_cell) => x == rhs_x && y == rhs_y && cell == rhs_cell,
                _ => false,
            },
            Self::Rectangle(x, y, width, height, fill_cell) => match rhs {
                Self::Rectangle(rhs_x, rhs_y, rhs_width, rhs_height, rhs_fill_cell) => {
                    x == rhs_x
                        && y == rhs_y
                        && width == rhs_width
                        && height == rhs_height
                        && fill_cell == rhs_fill_cell
                }
                _ => false,
            },
            Self::Group(primitives) => match rhs {
                Self::Group(rhs_primitives) => {
                    if primitives.len() != rhs_primitives.len() {
                        return false;
                    }

                    for (i, primitive) in primitives.iter().enumerate() {
                        if primitive != &rhs_primitives[i] {
                            return false;
                        }
                    }

                    true
                }
                _ => false,
            },
            Self::CursorPosition(x, y, style) => match rhs {
                Self::CursorPosition(rhs_x, rhs_y, rhs_style) => {
                    x == rhs_x && y == rhs_y && style == rhs_style
                }
                _ => false,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cell {
    pub content: Option<char>,
    pub style: Style,
}

impl Cell {
    pub fn from_char(c: char) -> Self {
        Self {
            content: Some(c),
            style: Style::default(),
        }
    }

    pub fn merge(&mut self, other: &Self) {
        if other.content.is_some() {
            self.content = other.content
        }

        self.style = self.style.merge(&other.style);
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            content: None,
            style: Style::default(),
        }
    }
}

impl std::cmp::PartialEq for Cell {
    fn eq(&self, rhs: &Self) -> bool {
        self.content == rhs.content && self.style == rhs.style
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub(crate) fg_color: Option<Color>,
    pub(crate) bg_color: Option<Color>,
    pub(crate) is_bold: bool,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            fg_color: None,
            bg_color: None,
            is_bold: false,
        }
    }
}

impl std::cmp::PartialEq for Style {
    fn eq(&self, rhs: &Self) -> bool {
        self.fg_color == rhs.fg_color
            && self.bg_color == rhs.bg_color
            && self.is_bold == rhs.is_bold
    }
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn try_merge(self, other: Option<&Self>) -> Self {
        match other {
            Some(style) => self.merge(style),
            None => self,
        }
    }

    pub fn merge(mut self, other: &Self) -> Self {
        if other.fg_color.is_some() {
            self.fg_color = other.fg_color;
        }

        if other.bg_color.is_some() {
            self.bg_color = other.bg_color;
        }

        if other.is_bold {
            self.is_bold = other.is_bold;
        }

        self
    }

    pub fn bold(mut self) -> Self {
        self.is_bold = true;
        self
    }

    pub fn bg<C>(mut self, color: C) -> Self
    where
        C: Into<Color>,
    {
        self.bg_color = Some(color.into());
        self
    }

    pub fn fg<C>(mut self, color: C) -> Self
    where
        C: Into<Color>,
    {
        self.fg_color = Some(color.into());
        self
    }
}

#[derive(Clone)]
pub struct VirtualBuffer {
    pub width: u16,
    pub height: u16,
    pub rows: Vec<Vec<Cell>>,
    pub hash: u64,
    pub cursor_position: Option<(u16, u16, CursorStyle)>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CursorShape {
    UnderScore,
    Line,
    Block,
}

impl Default for CursorShape {
    fn default() -> Self {
        Self::Line
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CursorStyle {
    pub(crate) shape: CursorShape,
    pub(crate) blinking: bool,
}

impl Default for CursorStyle {
    fn default() -> Self {
        Self {
            shape: CursorShape::default(),
            blinking: true,
        }
    }
}

impl CursorStyle {
    pub fn blinking(mut self, enabled: bool) -> Self {
        self.blinking = enabled;
        self
    }
}

impl Debug for VirtualBuffer {
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::result::Result<(), std::fmt::Error> {
        formatter.write_fmt(format_args!(
            "VirtualBuffer(width: {}, height: {}, hash: {})",
            self.width, self.height, self.hash
        ))
    }
}

impl VirtualBuffer {
    pub fn from_size(width: u16, height: u16) -> Self {
        let mut rows: Vec<Vec<Cell>> = Vec::with_capacity(height.into());
        for _ in 0..height {
            let mut row = Vec::with_capacity(width.into());
            for _ in 0..width {
                row.push(Cell::default());
            }
            rows.push(row);
        }

        Self {
            rows,
            width,
            height,
            hash: 0,
            cursor_position: None,
        }
    }

    pub fn merge_primitive(&mut self, primitive: Primitive) {
        match primitive {
            Primitive::Group(primitives) => {
                for primitive in primitives {
                    self.merge_primitive(primitive);
                }
            }
            Primitive::Rectangle(start_x, start_y, width, height, fill_cell) => {
                for x in start_x..(start_x + width) {
                    for y in start_y..(start_y + height) {
                        if x < self.width && y < self.height {
                            self.rows[y as usize][x as usize].merge(&fill_cell);
                        }
                    }
                }
            }
            Primitive::Cell(x, y, cell) => {
                if x < self.width && y < self.height {
                    self.rows[y as usize][x as usize].merge(&cell);
                }
            }
            Primitive::CursorPosition(x, y, style) => self.cursor_position = Some((x, y, style)),
        };
    }

    pub fn calc_hash(&mut self) {
        let mut hasher = DefaultHasher::new();
        hasher.write_u16(self.width);
        hasher.write_u16(self.height);
        hasher.write_u8(self.cursor_position.map(|_| 1).unwrap_or(0));
        hasher.write_u16(self.cursor_position.map(|p| p.0).unwrap_or(0));
        hasher.write_u16(self.cursor_position.map(|p| p.1).unwrap_or(0));

        for row in &self.rows {
            for cell in row {
                hasher.write_u32(cell.content.unwrap_or(' ') as u32);

                if let Some(color) = cell.style.fg_color {
                    hasher.write_u16((256.0 * color.r).round() as u16);
                    hasher.write_u16((256.0 * color.g).round() as u16);
                    hasher.write_u16((256.0 * color.b).round() as u16);
                    hasher.write_u16((256.0 * color.a).round() as u16);
                } else {
                    hasher.write_u16(0);
                    hasher.write_u16(0);
                    hasher.write_u16(0);
                    hasher.write_u16(0);
                }

                if let Some(color) = cell.style.bg_color {
                    hasher.write_u16((256.0 * color.r).round() as u16);
                    hasher.write_u16((256.0 * color.g).round() as u16);
                    hasher.write_u16((256.0 * color.b).round() as u16);
                    hasher.write_u16((256.0 * color.a).round() as u16);
                } else {
                    hasher.write_u16(0);
                    hasher.write_u16(0);
                    hasher.write_u16(0);
                    hasher.write_u16(0);
                }

                hasher.write_u8(if cell.style.is_bold { 0 } else { 2 });
            }
        }

        self.hash = hasher.finish();
    }
}

#[cfg(test)]
mod tests {
    extern crate test;

    use super::{Cell, Primitive, Style, VirtualBuffer};
    use iced_native::Color;
    use test::{black_box, Bencher};

    #[bench]
    fn bench_merge_primitive_and_calc_hash(b: &mut Bencher) {
        let mut primitives = vec![];

        for x in 0_u8..100_u8 {
            let mut primitives_group = vec![];

            for y in 0_u8..25_u8 {
                primitives_group.push(Primitive::Cell(
                    x as u16,
                    (y * 4) as u16,
                    Cell {
                        content: Some('a'),
                        style: Style {
                            fg_color: Some(Color::from_rgb8(x, x + 10_u8, x + 5_u8)),
                            bg_color: Some(Color::from_rgb8(x, x + 8_u8, x + 7_u8)),
                            is_bold: true,
                        },
                    },
                ));
            }

            primitives.push(Primitive::Group(primitives_group));

            if x < 90_u8 {
                primitives.push(Primitive::Rectangle(
                    x as u16,
                    (100 - x) as u16,
                    10 as u16,
                    10 as u16,
                    Cell::from_char('a'),
                ));
            }
        }

        b.iter(|| {
            black_box({
                let mut virtual_buffer = VirtualBuffer::from_size(100, 100);
                let primitives = primitives.clone();

                for primitive in primitives {
                    virtual_buffer.merge_primitive(primitive);
                }
                virtual_buffer.calc_hash();
            });
        });
    }
}
