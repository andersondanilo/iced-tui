#[derive(Debug)]
pub enum Primitive {
    Cell(u16, u16, Cell),
    Group(Vec<Primitive>),
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
        }
    }
}

#[derive(Clone, Copy)]
pub struct TuiFont {}

impl Default for TuiFont {
    fn default() -> Self {
        Self {}
    }
}

#[derive(Debug)]
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
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            content: Some(' '),
            style: Style::default(),
        }
    }
}

impl std::cmp::PartialEq for Cell {
    fn eq(&self, rhs: &Self) -> bool {
        self.content == rhs.content && self.style == rhs.style
    }
}

#[derive(Debug)]
pub struct Style {
    pub fg_color: Option<iced_native::Color>,
    pub bg_color: Option<iced_native::Color>,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            fg_color: None,
            bg_color: None,
        }
    }
}

impl std::cmp::PartialEq for Style {
    fn eq(&self, rhs: &Self) -> bool {
        self.fg_color == rhs.fg_color && self.bg_color == rhs.bg_color
    }
}

#[derive(Debug)]
pub struct VirtualBuffer {
    pub width: u16,
    pub height: u16,
    pub rows: Vec<Vec<Cell>>,
}

impl VirtualBuffer {
    pub fn from_size(width: u16, height: u16) -> Self {
        let mut rows: Vec<Vec<Cell>> = Vec::with_capacity(height.into());
        for y in 0..height {
            let mut row = Vec::with_capacity(width.into());
            for x in 0..width {
                row.push(Cell::default());
            }
            rows.push(row);
        }

        Self {
            rows,
            width,
            height,
        }
    }

    pub fn merge_primitive(&mut self, primitive: Primitive) {
        match primitive {
            Primitive::Group(primitives) => {
                for primitive in primitives {
                    self.merge_primitive(primitive);
                }
            }
            Primitive::Cell(x, y, cell) => {
                if x < self.width && y < self.height {
                    self.rows[y as usize][x as usize] = cell;
                }
            }
        };
    }
}
