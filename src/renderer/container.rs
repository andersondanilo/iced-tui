use super::primitives::{Cell, Primitive, Style};
use super::tui_renderer::TuiRenderer;
use iced_native::layout::Limits;
use iced_native::Vector;
use iced_native::{container, Element, Layout, Point, Rectangle, Size};

impl container::Renderer for TuiRenderer {
    type Style = Style;

    fn draw<Message>(
        &mut self,
        defaults: &Self::Defaults,
        container_bounds: Rectangle,
        cursor_position: Point,
        viewport: &Rectangle,
        style: &Self::Style,
        content: &Element<'_, Message, Self>,
        original_content_layout: Layout<'_>,
    ) -> <Self as iced_native::Renderer>::Output {
        let original_content_layout_bounds = original_content_layout.bounds();
        let limits = Limits::new(
            Size::ZERO,
            Size::new(
                container_bounds.width.round(),
                container_bounds.height.round(),
            ),
        );
        let mut node = content.layout(self, &limits);
        let node_position = Point::new(
            original_content_layout_bounds.x.round() - container_bounds.x.round(),
            original_content_layout_bounds.y.round() - container_bounds.y.round(),
        );
        node.move_to(node_position);

        let layout_offset = Vector::new(container_bounds.x, container_bounds.y);

        let new_elem_layout = Layout::with_offset(layout_offset, &node);

        let content = content.draw(self, defaults, new_elem_layout, cursor_position, viewport);

        let rectangle = Primitive::Rectangle(
            container_bounds.x.round() as u16,
            container_bounds.y.round() as u16,
            container_bounds.width.round() as u16,
            container_bounds.height.round() as u16,
            Cell {
                style: style.clone(),
                ..Cell::default()
            },
        );

        Primitive::Group(vec![rectangle, content])
    }
}
