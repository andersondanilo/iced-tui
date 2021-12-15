use super::primitives::{Cell, Primitive};
use super::tui_renderer::TuiRenderer;
use iced_native::space;

impl space::Renderer for TuiRenderer {
    fn draw(&mut self, bounds: iced_core::Rectangle) -> <Self as iced_native::Renderer>::Output {
        Primitive::Rectangle(
            bounds.x.round() as u16,
            bounds.y.round() as u16,
            bounds.width.round() as u16,
            bounds.height.round() as u16,
            Cell::default(),
        )
    }
}
