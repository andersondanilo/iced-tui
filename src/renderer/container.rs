use super::primitives::{Cell, Primitive, Style};
use super::tui_renderer::TuiRenderer;
use crate::renderer::utils::round_individual_layout;
use iced_native::{container, Element, Layout, Point, Rectangle};

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
        let (layout_offset, node) =
            round_individual_layout(container_bounds, original_content_layout, content, self);
        let new_elem_layout = Layout::with_offset(layout_offset, &node);

        let content_primitive =
            content.draw(self, defaults, new_elem_layout, cursor_position, viewport);

        let rectangle = Primitive::Rectangle(
            container_bounds.x.round() as u16,
            container_bounds.y.round() as u16,
            container_bounds.width.round() as u16,
            container_bounds.height.round() as u16,
            Cell {
                style: *style,
                ..Cell::default()
            },
        );

        Primitive::Group(vec![rectangle, content_primitive])
    }
}
