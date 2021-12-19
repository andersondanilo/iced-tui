use super::primitives::{Cell, Primitive};
use super::tui_renderer::TuiRenderer;
use crate::Style;
use iced_native::progress_bar;
use iced_native::Color;

#[derive(Debug, Clone, Copy)]
pub struct ProgressBarStyle {
    pub(crate) loaded_style: Style,
    pub(crate) unloaded_style: Style,
}

impl Default for ProgressBarStyle {
    fn default() -> Self {
        Self {
            loaded_style: Style::default(),
            unloaded_style: Style::default(),
        }
    }
}

impl ProgressBarStyle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fg<C>(mut self, color: C) -> Self
    where
        C: Into<Color>,
    {
        self.loaded_style = self.loaded_style.bg(color);
        self
    }

    pub fn bg<C>(mut self, color: C) -> Self
    where
        C: Into<Color>,
    {
        self.unloaded_style = self.unloaded_style.bg(color);
        self
    }
}

impl progress_bar::Renderer for TuiRenderer {
    type Style = ProgressBarStyle;

    const DEFAULT_HEIGHT: u16 = 1;

    fn draw(
        &self,
        bounds: iced_core::Rectangle,
        range: std::ops::RangeInclusive<f32>,
        value: f32,
        progress_style: &<Self as iced_native::progress_bar::Renderer>::Style,
    ) -> <Self as iced_native::Renderer>::Output {
        let range_length = range.end() - range.start();
        let progress_ratio = value / range_length;
        let progress_width = (bounds.width * progress_ratio).round() as u16;

        Primitive::Group(vec![
            Primitive::Rectangle(
                bounds.x.round() as u16,
                bounds.y.round() as u16,
                progress_width,
                bounds.height as u16,
                Cell::from_char(' ').style(progress_style.loaded_style),
            ),
            Primitive::Rectangle(
                bounds.x.round() as u16 + progress_width,
                bounds.y.round() as u16,
                (bounds.width.round() as u16 - progress_width).max(0),
                bounds.height as u16,
                Cell::from_char(' ').style(progress_style.unloaded_style),
            ),
        ])
    }
}
