use super::primitives::{Cell, Primitive, Style};
use super::tui_renderer::TuiRenderer;
use iced_native::{
    layout::Layout, layout::Limits, Element, Point, Rectangle, Renderer, Size, Vector,
};
use std::cmp;
pub enum RoundDirection {
    Horizontal,
    Vertical,
}

pub fn draw_list<Message>(
    renderer: &mut TuiRenderer,
    defaults: &<TuiRenderer as Renderer>::Defaults,
    contents: &[Element<'_, Message, TuiRenderer>],
    layout: Layout<'_>,
    cursor_position: Point,
    viewport: &Rectangle,
    direction: RoundDirection,
) -> Primitive {
    let layout_position = layout.position();
    let planned_layouts = layout.children().collect::<Vec<Layout>>();
    let result_bounds = round_layout_list(&planned_layouts, layout_position, direction);
    let mut primitives: Vec<Primitive> = Vec::with_capacity(contents.len());

    for (bounds, element) in result_bounds.iter().zip(contents.iter()) {
        let position = bounds.position();
        let size = bounds.size();
        let limits = Limits::new(Size::ZERO, size);
        let mut node = element.layout(renderer, &limits);
        node.move_to(position);

        let elem_layout =
            Layout::with_offset(Vector::new(layout_position.x, layout_position.y), &node);

        primitives.push(element.draw(renderer, defaults, elem_layout, cursor_position, viewport));
    }

    Primitive::Group(primitives)
}

fn round_layout_list(
    planned_layouts: &[Layout<'_>],
    layout_position: Point,
    direction: RoundDirection,
) -> Vec<Rectangle> {
    let mut min_next_position: Option<i16> = None;
    let planned_sum_length_rounded: i16 = planned_layouts
        .iter()
        .map(|l| match direction {
            RoundDirection::Horizontal => l.bounds().width,
            RoundDirection::Vertical => l.bounds().height,
        })
        .sum::<f32>()
        .round() as i16;

    let mut current_sum_length: i16 = 0;
    let mut results: Vec<Rectangle> = Vec::with_capacity(planned_layouts.len());

    let mut last_planned_end_position: f32 = 0_f32;
    let mut last_result_end_position: i16 = 0;

    for layout in planned_layouts.iter() {
        let (planned_x, planned_y, planned_width, planned_height) = {
            let bounds = layout.bounds();

            (
                bounds.x - layout_position.x,
                bounds.y - layout_position.y,
                bounds.width,
                bounds.height,
            )
        };

        let planned_position = match direction {
            RoundDirection::Horizontal => planned_x,
            RoundDirection::Vertical => planned_y,
        };
        let planned_length = match direction {
            RoundDirection::Horizontal => planned_width,
            RoundDirection::Vertical => planned_height,
        };

        let mut result_position: i16 = match min_next_position {
            Some(min_position) => cmp::max(min_position as i16, planned_position.round() as i16),
            None => planned_position.round() as i16,
        };

        let had_padding = (last_planned_end_position - planned_position) > 0.6_f32;

        if had_padding && (result_position == last_result_end_position) {
            result_position += 1;
        }

        let mut result_length = planned_length.round() as i16;

        current_sum_length += result_length;

        if current_sum_length > planned_sum_length_rounded {
            result_length -= current_sum_length - planned_sum_length_rounded
        }

        min_next_position = Some(result_position + result_length);
        last_planned_end_position = planned_position + planned_length;
        last_result_end_position = result_position + result_length;

        let result_bounds = Rectangle::new(
            Point::new(
                match direction {
                    RoundDirection::Horizontal => result_position as f32,
                    RoundDirection::Vertical => planned_x.round(),
                },
                match direction {
                    RoundDirection::Horizontal => planned_y.round(),
                    RoundDirection::Vertical => result_position as f32,
                },
            ),
            Size::new(
                match direction {
                    RoundDirection::Horizontal => result_length as f32,
                    RoundDirection::Vertical => planned_width.round(),
                },
                match direction {
                    RoundDirection::Horizontal => planned_height.round(),
                    RoundDirection::Vertical => result_length as f32,
                },
            ),
        );

        results.push(result_bounds);
    }

    results
}

pub fn crop_text_to_bounds(
    content: &str,
    size: Option<Size>,
    start_x: u16,
    start_y: u16,
    auto_wrap: bool,
    return_primitives: bool,
    style: Style,
    allow_wrap: bool,
) -> (Vec<Primitive>, u16, u16) {
    let mut primitive_cells: Vec<Primitive> = Vec::with_capacity(content.len());
    let bounds_width_i = size.map(|s| s.width as u16).unwrap_or(u16::MAX);
    let bounds_height_i = size.map(|s| s.height as u16).unwrap_or(u16::MAX);
    let mut current_x: u16 = start_x;
    let mut current_y: u16 = start_y;
    let mut filled_width: u16 = 0;
    let mut filled_height: u16 = 1;

    let mut row_width: u16 = 0;

    for c in content.chars() {
        if c == '\n' || (row_width >= bounds_width_i && auto_wrap) {
            // go to next row, or break if not possible
            if allow_wrap && filled_height < bounds_height_i {
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

        if !is_printable(c) {
            continue;
        }

        // add char to current row, if inside width bounds
        if return_primitives {
            primitive_cells.push(Primitive::Cell(
                current_x,
                current_y,
                Cell {
                    content: Some(c),
                    style,
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

fn is_printable(c: char) -> bool {
    c as u32 >= 30
}

#[cfg(test)]
mod tests {
    use super::super::primitives::{Primitive, Style};
    use super::crop_text_to_bounds;
    use super::{round_layout_list, RoundDirection};
    use iced_native::{
        layout::{Layout, Node},
        Point, Rectangle, Size, Vector,
    };

    #[test]
    fn it_round_horizontally() {
        let planned_node1 = Node::new(Size::new(10.5, 20.3));
        let planned_node2 = Node::new(Size::new(10.3, 20.5));
        let planned_node3 = Node::new(Size::new(10.5, 20.5));

        let planned_layouts = vec![
            Layout::with_offset(Vector::new(100.5, 11.3), &planned_node1),
            Layout::with_offset(Vector::new(111_f32, 11.3), &planned_node2),
            Layout::with_offset(Vector::new(121.2, 11.3), &planned_node3),
        ];

        let expected_rectangles = vec![
            Rectangle {
                x: 101_f32,
                y: 11_f32,
                width: 11_f32,
                height: 20_f32,
            },
            Rectangle {
                x: 112_f32,
                y: 11_f32,
                width: 10_f32,
                height: 21_f32,
            },
            Rectangle {
                x: 122_f32,
                y: 11_f32,
                width: 10_f32,
                height: 21_f32,
            },
        ];

        let rectangles =
            round_layout_list(&planned_layouts, Point::ORIGIN, RoundDirection::Horizontal);

        assert_eq!(rectangles.len(), expected_rectangles.len());

        for (index, expected_bound) in expected_rectangles.iter().enumerate() {
            assert_eq!(&rectangles[index], expected_bound);
        }
    }

    #[test]
    fn it_round_vertically() {
        let planned_node1 = Node::new(Size::new(20.3, 10.5));
        let planned_node2 = Node::new(Size::new(20.5, 10.3));
        let planned_node3 = Node::new(Size::new(20.5, 10.5));

        let planned_layouts = vec![
            Layout::with_offset(Vector::new(11.3, 100.5), &planned_node1),
            Layout::with_offset(Vector::new(11.3, 111_f32), &planned_node2),
            Layout::with_offset(Vector::new(11.3, 121.2), &planned_node3),
        ];

        let expected_rectangles = vec![
            Rectangle {
                x: 11_f32,
                y: 101_f32,
                width: 20_f32,
                height: 11_f32,
            },
            Rectangle {
                x: 11_f32,
                y: 112_f32,
                width: 21_f32,
                height: 10_f32,
            },
            Rectangle {
                x: 11_f32,
                y: 122_f32,
                width: 21_f32,
                height: 10_f32,
            },
        ];

        let rectangles =
            round_layout_list(&planned_layouts, Point::ORIGIN, RoundDirection::Vertical);

        assert_eq!(rectangles.len(), expected_rectangles.len());

        for (index, expected_bound) in expected_rectangles.iter().enumerate() {
            assert_eq!(&rectangles[index], expected_bound);
        }
    }

    #[test]
    fn it_ignore_already_rounded() {
        // Example in diagonal
        let planned_node1 = Node::new(Size::new(10_f32, 20_f32));
        let planned_node2 = Node::new(Size::new(12_f32, 22_f32));
        let planned_node3 = Node::new(Size::new(13_f32, 24_f32));
        let planned_layouts = vec![
            Layout::with_offset(Vector::new(100_f32, 10_f32), &planned_node1),
            Layout::with_offset(Vector::new(111_f32, 30_f32), &planned_node2),
            Layout::with_offset(Vector::new(123_f32, 54_f32), &planned_node3),
        ];

        let expected_rectangles = vec![
            Rectangle {
                x: 100_f32,
                y: 10_f32,
                width: 10_f32,
                height: 20_f32,
            },
            Rectangle {
                x: 111_f32,
                y: 30_f32,
                width: 12_f32,
                height: 22_f32,
            },
            Rectangle {
                x: 123_f32,
                y: 54_f32,
                width: 13_f32,
                height: 24_f32,
            },
        ];

        let rectangles =
            round_layout_list(&planned_layouts, Point::ORIGIN, RoundDirection::Horizontal);
        assert_eq!(rectangles.len(), expected_rectangles.len());
        for (index, expected_bound) in expected_rectangles.iter().enumerate() {
            assert_eq!(&rectangles[index], expected_bound);
        }

        let rectangles =
            round_layout_list(&planned_layouts, Point::ORIGIN, RoundDirection::Vertical);
        assert_eq!(rectangles.len(), expected_rectangles.len());
        for (index, expected_bound) in expected_rectangles.iter().enumerate() {
            assert_eq!(&rectangles[index], expected_bound);
        }
    }

    #[test]
    fn it_get_primitives_from_crop_text() {
        let (primitives, width, height) = crop_text_to_bounds(
            "Hello\nPan",
            Some(Size::new(50., 50.)),
            10,
            10,
            false,
            true,
            Style::default(),
            true,
        );

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
            Some(Size::new(3., 10.)),
            10,
            10,
            false,
            true,
            Style::default(),
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
            Some(Size::new(10., 3.)),
            10,
            10,
            false,
            true,
            Style::default(),
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
            Some(Size::new(3., 10.)),
            10,
            10,
            true,
            true,
            Style::default(),
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

    #[test]
    fn it_ignore_non_printable() {
        let (primitives, width, height) = crop_text_to_bounds(
            "Hello\n\tPan\nLon\nI\ton\n",
            Some(Size::new(3., 10.)),
            10,
            10,
            true,
            true,
            Style::default(),
            true,
        );

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

        assert_eq!(width, 3);
        assert_eq!(height, 5);
        assert_eq!(primitives.len(), expected_primitives.len());
    }

    #[test]
    fn it_dont_ignore_space() {
        let (primitives, width, height) = crop_text_to_bounds(
            "Hello Pan",
            Some(Size::new(3., 10.)),
            10,
            10,
            true,
            true,
            Style::default(),
            true,
        );

        let expected_primitives = vec![
            Primitive::from_char(10, 10, 'H'),
            Primitive::from_char(11, 10, 'e'),
            Primitive::from_char(12, 10, 'l'),
            Primitive::from_char(10, 11, 'l'),
            Primitive::from_char(11, 11, 'o'),
            Primitive::from_char(12, 11, ' '),
            Primitive::from_char(10, 12, 'P'),
            Primitive::from_char(11, 12, 'a'),
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

        assert_eq!(width, 3);
        assert_eq!(height, 3);
        assert_eq!(primitives.len(), expected_primitives.len());
    }
}
