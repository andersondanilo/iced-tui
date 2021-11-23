use super::primitives::{Primitive, Style};
use super::tui_renderer::TuiRenderer;
use iced_native::{container, Element, Layout, Point, Rectangle};

impl container::Renderer for TuiRenderer {
    type Style = Style;

    fn draw<Message>(
        &mut self,
        defaults: &Self::Defaults,
        bounds: Rectangle,
        cursor_position: Point,
        viewport: &Rectangle,
        _style: &Self::Style,
        content: &Element<'_, Message, Self>,
        content_layout: Layout<'_>,
    ) -> <Self as iced_native::Renderer>::Output {
        let content = content.draw(self, defaults, content_layout, cursor_position, viewport);

        Primitive::Group(vec![content])
    }
}
