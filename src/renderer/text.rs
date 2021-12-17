use super::primitives::Primitive;
use super::tui_renderer::TuiRenderer;
use super::utils::crop_text_to_bounds;
use crate::Style;
use iced_native::{text, Color, HorizontalAlignment, Rectangle, Renderer, VerticalAlignment};

impl text::Renderer for TuiRenderer {
    type Font = Style;

    fn default_size(&self) -> u16 {
        1
    }

    fn measure(
        &self,
        content: &str,
        _size: u16,
        _font: <Self as text::Renderer>::Font,
        bounds: iced_native::Size,
    ) -> (f32, f32) {
        let (_, width, height) = crop_text_to_bounds(
            content,
            Some(bounds),
            0,
            0,
            true,
            false,
            Style::default(),
            true,
        );
        (width as f32, height as f32)
    }

    fn draw(
        &mut self,
        _defaults: &<Self as Renderer>::Defaults,
        bounds: Rectangle,
        content: &str,
        _size: u16,
        font: <Self as text::Renderer>::Font,
        color: Option<Color>,
        _horizontal_alignment: HorizontalAlignment,
        _vertical_alignment: VerticalAlignment,
    ) -> <Self as Renderer>::Output {
        let style = Style {
            fg_color: color.or(font.fg_color),
            ..font
        };

        let (primitive_cells, _width, _height) = crop_text_to_bounds(
            content,
            Some(bounds.size()),
            bounds.x as u16,
            bounds.y as u16,
            true,
            true,
            style,
            true,
        );
        Primitive::from_cells(primitive_cells)
    }
}
