use super::primitives::{Cell, Primitive};
use crate::CursorStyle;

#[derive(Clone, PartialEq)]
pub struct VirtualBuffer {
    pub width: u16,
    pub height: u16,
    pub rows: Vec<Vec<Cell>>,
    pub cursor_position: Option<(u16, u16, CursorStyle)>,
    pub primitive_hash: u64,
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
            primitive_hash: 0,
            cursor_position: None,
        }
    }

    pub fn merge_primitive(&mut self, primitive: &Primitive) {
        match primitive {
            Primitive::Group(primitives) => {
                for primitive in primitives {
                    self.merge_primitive(primitive);
                }
            }
            Primitive::Rectangle(start_x, start_y, width, height, fill_cell) => {
                if !fill_cell.is_empty() {
                    for x in *start_x..(*start_x + width) {
                        for y in *start_y..(*start_y + height) {
                            if x < self.width && y < self.height {
                                self.rows[y as usize][x as usize].merge(*fill_cell);
                            }
                        }
                    }
                }
            }
            Primitive::Cell(x, y, cell) => {
                if *x < self.width && *y < self.height {
                    self.rows[*y as usize][*x as usize].merge(*cell);
                }
            }
            Primitive::CursorPosition(x, y, style) => self.cursor_position = Some((*x, *y, *style)),
        };
    }
}

#[cfg(test)]
mod tests {
    extern crate test;

    use super::super::colors::TermColor;
    use super::super::primitives::{Cell, Primitive};
    use super::super::style::Style;
    use super::VirtualBuffer;
    use test::Bencher;

    fn make_example_primitive() -> Primitive {
        let mut primitive_cells = vec![];
        for x in 0_u8..100_u8 {
            for y in 0_u8..25_u8 {
                for add_y in 0_u8..5_u8 {
                    primitive_cells.push(Primitive::Cell(
                        x as u16,
                        (y + add_y) as u16,
                        Cell {
                            content: Some('a'),
                            style: Style {
                                fg_color: Some(TermColor::Rgb(x, x + 10_u8, y + 5_u8)),
                                bg_color: Some(TermColor::Rgb(x, x + 8_u8, y + 7_u8)),
                                is_bold: x % 2 == 0,
                            },
                        },
                    ));
                }
            }
        }

        Primitive::Group(primitive_cells)
    }

    #[bench]
    fn bench_merge_primitive(b: &mut Bencher) {
        let primitive = make_example_primitive();

        let mut vbuffer = VirtualBuffer::from_size(100, 100);

        b.iter(|| {
            vbuffer.merge_primitive(&primitive);
        });
    }

    #[bench]
    fn bench_cell_merge(b: &mut Bencher) {
        let cell_a = Cell::from_char('A');
        let cell_b = Cell::from_char('B');
        let cell_c = Cell::from_char('C');
        let cell_d = Cell::from_char('D');

        b.iter(|| {
            let mut cell_vec: Vec<Cell> = vec![
                cell_a,
                cell_b,
                cell_c,
                cell_d,
                cell_a,
                cell_b,
                cell_c,
                cell_d,
                cell_a,
                cell_b,
                cell_c,
                cell_d,
            ];

            for cell in cell_vec.iter_mut() {
                cell.merge(cell_a);
            }
        });
    }
}
