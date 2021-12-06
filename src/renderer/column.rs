use super::tui_renderer::TuiRenderer;
use super::utils::{draw_list, RoundDirection};
use iced_native::column;

impl column::Renderer for TuiRenderer {
    fn draw<Message>(
        &mut self,
        defaults: &<Self as iced_native::Renderer>::Defaults,
        contents: &[iced_native::Element<'_, Message, Self>],
        layout: iced_native::Layout<'_>,
        cursor_position: iced_native::Point,
        viewport: &iced_native::Rectangle,
    ) -> <Self as iced_native::Renderer>::Output {
        draw_list(
            self,
            defaults,
            contents,
            layout,
            cursor_position,
            viewport,
            RoundDirection::Vertical,
        )
    }
}
