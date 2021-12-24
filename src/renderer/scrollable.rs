use super::tui_renderer::TuiRenderer;
use crate::Style;
use iced_native::scrollable;

impl scrollable::Renderer for TuiRenderer {
    type Style = Style;

    fn scrollbar(
        &self,
        _bounds: iced_core::Rectangle,
        _content_bounds: iced_core::Rectangle,
        _offset: u32,
        _scrollbar_width: u16,
        _scrollbar_margin: u16,
        _scroller_width: u16,
    ) -> std::option::Option<iced_native::scrollable::Scrollbar> {
        None
    }

    fn draw(
        &mut self,
        _scrollable: &iced_native::scrollable::State,
        _bounds: iced_core::Rectangle,
        _content_bounds: iced_core::Rectangle,
        _is_mouse_over: bool,
        _is_mouse_over_scrollbar: bool,
        _scrollbar: std::option::Option<iced_native::scrollable::Scrollbar>,
        _offset: u32,
        _style: &<Self as iced_native::scrollable::Renderer>::Style,
        content: <Self as iced_native::Renderer>::Output,
    ) -> <Self as iced_native::Renderer>::Output {
        content
        //content.cut_to_offset(offset as u16, bounds.height.round() as u16)
    }
}
