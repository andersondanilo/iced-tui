use crate::CursorStyle;
use crate::Style;
use core::fmt::Debug;

#[derive(Debug, PartialEq)]
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

#[derive(Debug, Clone, Copy, PartialEq)]
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

    pub fn is_empty(&self) -> bool {
        self.content.is_none() && self.style.is_empty()
    }

    pub fn merge(&mut self, other: Self) {
        if other.content.is_some() {
            self.content = other.content
        }

        self.style = self.style.merge(other.style);
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
