use super::primitives::Primitive;
use super::tui_renderer::TuiRenderer;
use iced_native::column;
use std::iter::zip;

impl column::Renderer for TuiRenderer {
    fn draw<Message>(
        &mut self,
        defaults: &<Self as iced_native::Renderer>::Defaults,
        contents: &[iced_native::Element<'_, Message, Self>],
        layout: iced_native::Layout<'_>,
        cursor_position: iced_native::Point,
        viewport: &iced_native::Rectangle,
    ) -> <Self as iced_native::Renderer>::Output {
        Primitive::Group(
            zip(layout.children(), contents)
                .map(|(layout, content)| {
                    content.draw(self, defaults, layout, cursor_position, viewport)
                })
                .collect::<Vec<Self::Output>>(),
        )
    }
}
