use super::primitives::Primitive;
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

#[cfg(test)]
mod tests {
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
}
