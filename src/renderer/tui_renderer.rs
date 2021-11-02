use iced_native::Element;
use iced_native::Layout;
use iced_native::Point;
use iced_native::Rectangle;
use iced_native::{container, Renderer};
use tui::widgets::Widget as TuiWidget;

pub struct TuiRenderer {}

impl Default for TuiRenderer {
    fn default() -> Self {
        TuiRenderer {}
    }
}

impl Renderer for TuiRenderer {
    type Output = Box<dyn TuiWidget>;
    type Defaults = tui::style::Style;

    fn overlay(
        &mut self,
        base: <Self as iced_native::Renderer>::Output,
        _overlay: <Self as iced_native::Renderer>::Output,
        _overlay_bounds: iced_native::Rectangle,
    ) -> <Self as iced_native::Renderer>::Output {
        return base;
    }
}

impl container::Renderer for TuiRenderer {
    type Style = tui::style::Style;

    fn draw<Message>(
        &mut self,
        _defaults: &Self::Defaults,
        _bounds: Rectangle,
        _cursor_position: Point,
        _viewport: &Rectangle,
        _style: &Self::Style,
        _content: &Element<'_, Message, Self>,
        _content_layout: Layout<'_>,
    ) -> <Self as iced_native::Renderer>::Output {
        todo!()
    }
}
