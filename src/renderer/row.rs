use super::tui_renderer::TuiRenderer;
use super::utils::{draw_list, RoundDirection};
use iced_native::{row, Element, Layout, Point, Rectangle, Renderer};

impl row::Renderer for TuiRenderer {
    fn draw<Message>(
        &mut self,
        defaults: &<Self as Renderer>::Defaults,
        contents: &[Element<'_, Message, Self>],
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) -> <Self as Renderer>::Output {
        draw_list(
            self,
            defaults,
            contents,
            layout,
            cursor_position,
            viewport,
            RoundDirection::Horizontal,
        )
    }
}
