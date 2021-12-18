use super::primitives::{Cell, CursorPosition, Primitive};

#[derive(Clone)]
pub struct VirtualBuffer {
    pub width: u16,
    pub height: u16,
    pub rows: Vec<Vec<Cell>>,
    pub hash: u64,
    pub cursor_position: Option<CursorPosition>,
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
        for ((x, y), cell) in primitive.cells.into_iter() {
            if x < self.width && y < self.height {
                self.rows[y as usize][x as usize] = cell;
            }
        }

        self.cursor_position = primitive.cursor_position
    }
}

#[cfg(test)]
mod tests {
    extern crate test;

    use super::super::primitives::{Cell, Primitive, PrimitiveCell};
    use super::super::style::Style;
    use super::VirtualBuffer;
    use iced_native::Color;
    use test::Bencher;

    #[bench]
    fn bench_merge_primitive(b: &mut Bencher) {
        let mut primitives = vec![];

        for x in 0_u8..100_u8 {
            let mut primitives_group = vec![];

            for y in 0_u8..25_u8 {
                primitives_group.push(PrimitiveCell::new(
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

            primitives.push(Primitive::from_cells(primitives_group));

            if x < 90_u8 {
                primitives.push(Primitive::rectangle(
                    x as u16,
                    (100 - x) as u16,
                    10 as u16,
                    10 as u16,
                    Cell::from_char('a'),
                ));
            }
        }

        let primitive = Primitive::merge(primitives);

        b.iter(|| {
            let mut virtual_buffer = VirtualBuffer::from_size(100, 100);
            virtual_buffer.merge_primitive(primitive.clone());
        });
    }
}
