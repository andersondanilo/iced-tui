use super::style::{CursorStyle, Style};
use core::fmt::Debug;
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct CursorPosition {
    pub(crate) x: u16,
    pub(crate) y: u16,
    pub(crate) style: CursorStyle,
}

impl CursorPosition {
    pub fn new(x: u16, y: u16, style: CursorStyle) -> Self {
        Self { x, y, style }
    }
}

#[derive(PartialEq, Debug)]
pub struct PrimitiveCell {
    pub(crate) x: u16,
    pub(crate) y: u16,
    pub(crate) cell: Cell,
}

impl PrimitiveCell {
    pub fn new(x: u16, y: u16, cell: Cell) -> Self {
        Self { x, y, cell }
    }

    pub fn from_char(x: u16, y: u16, content: char) -> Self {
        Self {
            x,
            y,
            cell: Cell::from_char(content),
        }
    }
}

pub struct Primitive {
    pub(crate) cells: BTreeMap<(u16, u16), Cell>,
    pub(crate) cursor_position: Option<CursorPosition>,
}

impl Default for Primitive {
    fn default() -> Self {
        Self {
            cells: BTreeMap::new(),
            cursor_position: None,
        }
    }
}

impl Primitive {
    pub fn merge(mut primitives: Vec<Primitive>) -> Self {
        let mut drain = primitives.drain(..);
        let mut main_primitive = drain.next().unwrap_or_default();

        for other_primitive in drain {
            for (key, new_value) in other_primitive.cells.into_iter() {
                if let Some(old_value) = main_primitive.cells.get_mut(&key) {
                    old_value.merge(new_value)
                } else {
                    main_primitive.cells.insert(key, new_value);
                }
            }

            if other_primitive.cursor_position.is_some() {
                main_primitive.cursor_position = other_primitive.cursor_position;
            }
        }

        main_primitive
    }

    pub fn rectangle(start_x: u16, start_y: u16, width: u16, height: u16, cell: Cell) -> Self {
        let mut cells = BTreeMap::new();

        for x in start_x..(start_x + width) {
            for y in start_y..(start_y + height) {
                cells.insert((x, y), cell.clone());
            }
        }

        Self {
            cells,
            cursor_position: None,
        }
    }

    pub fn from_cells(mut primitive_cells: Vec<PrimitiveCell>) -> Self {
        let mut cells = BTreeMap::new();

        for cell in primitive_cells.drain(..) {
            cells.insert((cell.x, cell.y), cell.cell);
        }

        Self {
            cells,
            cursor_position: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
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
