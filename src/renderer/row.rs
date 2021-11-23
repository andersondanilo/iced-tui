use super::primitives::Primitive;
use super::tui_renderer::TuiRenderer;
use iced_native::{row, Element, Layout, Point, Rectangle, Renderer};
use std::iter::zip;

impl row::Renderer for TuiRenderer {
    fn draw<Message>(
        &mut self,
        defaults: &<Self as Renderer>::Defaults,
        contents: &[Element<'_, Message, Self>],
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) -> <Self as Renderer>::Output {
        Primitive::Group(
            zip(layout.children(), contents)
                .map(|(layout, content)| {
                    content.draw(self, defaults, layout, cursor_position, viewport)
                })
                .collect::<Vec<Self::Output>>(),
        )
    }
}
