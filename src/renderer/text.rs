use super::primitives::{Cell, Primitive, Style, TuiFont};
use super::tui_renderer::TuiRenderer;
use iced_native::{text, Color, HorizontalAlignment, Rectangle, Renderer, Size, VerticalAlignment};

impl text::Renderer for TuiRenderer {
    type Font = TuiFont;

    fn default_size(&self) -> u16 {
        1
    }

    fn measure(
        &self,
        content: &str,
        _size: u16,
        _font: <Self as text::Renderer>::Font,
        bounds: iced_native::Size,
    ) -> (f32, f32) {
        let (_, width, height) = crop_text_to_bounds(content, bounds, 0, 0, true, false);
        (width as f32, height as f32)
    }

    fn draw(
        &mut self,
        _defaults: &<Self as Renderer>::Defaults,
        bounds: Rectangle,
        content: &str,
        _size: u16,
        _font: <Self as text::Renderer>::Font,
        _color: std::option::Option<Color>,
        _horizontal_alignment: HorizontalAlignment,
        _vertical_alignment: VerticalAlignment,
    ) -> <Self as Renderer>::Output {
        let (primitive_cells, _width, _height) = crop_text_to_bounds(
            content,
            bounds.size(),
            bounds.x as u16,
            bounds.y as u16,
            true,
            true,
        );
        Primitive::Group(primitive_cells)
    }
}

fn crop_text_to_bounds<'a>(
    content: &str,
    size: Size,
    start_x: u16,
    start_y: u16,
    auto_wrap: bool,
    return_primitives: bool,
) -> (Vec<Primitive>, u16, u16) {
    let mut primitive_cells: Vec<Primitive> = Vec::with_capacity(content.len());
    let bounds_width_i = size.width as u16;
    let bounds_height_i = size.height as u16;
    let mut current_x: u16 = start_x;
    let mut current_y: u16 = start_y;
    let mut filled_width: u16 = 0;
    let mut filled_height: u16 = 1;

    // TODO: Handle non-printable chars and tab/space

    let mut row_width: u16 = 0;

    for c in content.chars() {
        if c == '\n' || (row_width >= bounds_width_i && auto_wrap) {
            // go to next row, or break if not possible
            if filled_height < bounds_height_i {
                current_x = start_x;
                row_width = 0;
                current_y += 1;
                filled_height += 1;
            } else {
                break;
            }
        }

        if row_width >= bounds_width_i || c == '\n' {
            continue;
        }

        // add char to current row, if inside width bounds
        if return_primitives {
            primitive_cells.push(Primitive::Cell(
                current_x,
                current_y,
                Cell {
                    content: Some(c),
                    style: Style::default(),
                },
            ));
        }

        current_x += 1;
        row_width += 1;

        if row_width > filled_width {
            filled_width = row_width;
        }
    }

    if row_width == 0 && filled_height > 0 {
        filled_height -= 1;
    }

    (primitive_cells, filled_width, filled_height)
}

#[cfg(test)]
mod tests {
    use super::super::primitives::Primitive;
    use super::crop_text_to_bounds;
    use iced_native::Size;

    #[test]
    fn it_get_primitives_from_crop_text() {
        let (primitives, width, height) =
            crop_text_to_bounds("Hello\nPan", Size::new(50., 50.), 10, 10, false, true);

        assert_eq!(width, 5);
        assert_eq!(height, 2);

        let expected_primitives = vec![
            Primitive::from_char(10, 10, 'H'),
            Primitive::from_char(11, 10, 'e'),
            Primitive::from_char(12, 10, 'l'),
            Primitive::from_char(13, 10, 'l'),
            Primitive::from_char(14, 10, 'o'),
            Primitive::from_char(10, 11, 'P'),
            Primitive::from_char(11, 11, 'a'),
            Primitive::from_char(12, 11, 'n'),
        ];

        for (i, expected_primitive) in expected_primitives.iter().enumerate() {
            assert!(
                primitives.len() > i,
                "should have primitive at index {}, ({})",
                primitives.len(),
                match expected_primitive {
                    Primitive::Cell(x, y, cell) => {
                        format!("x: {}, y: {}, char: {}", x, y, cell.content.unwrap())
                    }
                    _ => "".to_string(),
                }
            );
            assert_eq!(&primitives[i], expected_primitive);
        }

        assert_eq!(primitives.len(), expected_primitives.len());
    }

    #[test]
    fn it_crop_text_horizontally() {
        let (primitives, width, height) = crop_text_to_bounds(
            "Hello\nPan\nLon\nIon\n",
            Size::new(3., 10.),
            10,
            10,
            false,
            true,
        );

        assert_eq!(width, 3);
        assert_eq!(height, 4);

        let expected_primitives = vec![
            Primitive::from_char(10, 10, 'H'),
            Primitive::from_char(11, 10, 'e'),
            Primitive::from_char(12, 10, 'l'),
            Primitive::from_char(10, 11, 'P'),
            Primitive::from_char(11, 11, 'a'),
            Primitive::from_char(12, 11, 'n'),
            Primitive::from_char(10, 12, 'L'),
            Primitive::from_char(11, 12, 'o'),
            Primitive::from_char(12, 12, 'n'),
            Primitive::from_char(10, 13, 'I'),
            Primitive::from_char(11, 13, 'o'),
            Primitive::from_char(12, 13, 'n'),
        ];

        for (i, expected_primitive) in expected_primitives.iter().enumerate() {
            assert!(
                primitives.len() > i,
                "should have primitive at index {}, ({})",
                primitives.len(),
                match expected_primitive {
                    Primitive::Cell(x, y, cell) => {
                        format!("x: {}, y: {}, char: {}", x, y, cell.content.unwrap())
                    }
                    _ => "".to_string(),
                }
            );
            assert_eq!(&primitives[i], expected_primitive);
        }

        assert_eq!(primitives.len(), expected_primitives.len());
    }

    #[test]
    fn it_crop_text_vertically() {
        let (primitives, width, height) = crop_text_to_bounds(
            "Hello\nPan\nLon\nIon\n",
            Size::new(10., 3.),
            10,
            10,
            false,
            true,
        );

        assert_eq!(width, 5);
        assert_eq!(height, 3);

        let expected_primitives = vec![
            Primitive::from_char(10, 10, 'H'),
            Primitive::from_char(11, 10, 'e'),
            Primitive::from_char(12, 10, 'l'),
            Primitive::from_char(13, 10, 'l'),
            Primitive::from_char(14, 10, 'o'),
            Primitive::from_char(10, 11, 'P'),
            Primitive::from_char(11, 11, 'a'),
            Primitive::from_char(12, 11, 'n'),
            Primitive::from_char(10, 12, 'L'),
            Primitive::from_char(11, 12, 'o'),
            Primitive::from_char(12, 12, 'n'),
        ];

        for (i, expected_primitive) in expected_primitives.iter().enumerate() {
            assert!(
                primitives.len() > i,
                "should have primitive at index {}, ({})",
                primitives.len(),
                match expected_primitive {
                    Primitive::Cell(x, y, cell) => {
                        format!("x: {}, y: {}, char: {}", x, y, cell.content.unwrap())
                    }
                    _ => "".to_string(),
                }
            );
            assert_eq!(&primitives[i], expected_primitive);
        }

        assert_eq!(primitives.len(), expected_primitives.len());
    }

    #[test]
    fn it_auto_wrap_text() {
        let (primitives, width, height) = crop_text_to_bounds(
            "Hello\nPan\nLon\nIon\n",
            Size::new(3., 10.),
            10,
            10,
            true,
            true,
        );

        assert_eq!(width, 3);
        assert_eq!(height, 5);

        let expected_primitives = vec![
            Primitive::from_char(10, 10, 'H'),
            Primitive::from_char(11, 10, 'e'),
            Primitive::from_char(12, 10, 'l'),
            Primitive::from_char(10, 11, 'l'),
            Primitive::from_char(11, 11, 'o'),
            Primitive::from_char(10, 12, 'P'),
            Primitive::from_char(11, 12, 'a'),
            Primitive::from_char(12, 12, 'n'),
            Primitive::from_char(10, 13, 'L'),
            Primitive::from_char(11, 13, 'o'),
            Primitive::from_char(12, 13, 'n'),
            Primitive::from_char(10, 14, 'I'),
            Primitive::from_char(11, 14, 'o'),
            Primitive::from_char(12, 14, 'n'),
        ];

        for (i, expected_primitive) in expected_primitives.iter().enumerate() {
            assert!(
                primitives.len() > i,
                "should have primitive at index {}, ({})",
                primitives.len(),
                match expected_primitive {
                    Primitive::Cell(x, y, cell) => {
                        format!("x: {}, y: {}, char: {}", x, y, cell.content.unwrap())
                    }
                    _ => "".to_string(),
                }
            );
            assert_eq!(&primitives[i], expected_primitive);
        }

        assert_eq!(primitives.len(), expected_primitives.len());
    }
}
