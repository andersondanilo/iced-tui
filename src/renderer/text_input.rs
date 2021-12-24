use super::primitives::{Cell, Primitive};
use super::tui_renderer::TuiRenderer;
use super::utils::crop_text_to_bounds;
use crate::CursorStyle;
use crate::Style;
use iced_native::text_input;

#[derive(Debug, Clone, Copy, Default)]
pub struct TextInputStyle {
    pub(crate) normal: Style,
    pub(crate) focused: Style,
    pub(crate) placeholder: Style,
    pub(crate) hover: Style,
    pub(crate) cursor: CursorStyle,
}

impl TextInputStyle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn normal(mut self, normal: Style) -> Self {
        self.normal = normal;
        self
    }

    pub fn focused(mut self, focused: Style) -> Self {
        self.focused = focused;
        self
    }

    pub fn placeholder(mut self, placeholder: Style) -> Self {
        self.placeholder = placeholder;
        self
    }

    pub fn hover(mut self, hover: Style) -> Self {
        self.hover = hover;
        self
    }

    pub fn cursor(mut self, cursor: CursorStyle) -> Self {
        self.cursor = cursor;
        self
    }
}

impl text_input::Renderer for TuiRenderer {
    type Style = TextInputStyle;

    fn measure_value(
        &self,
        value: &str,
        _size: u16,
        _font: <Self as iced_native::text::Renderer>::Font,
    ) -> f32 {
        let (_, width, _) =
            crop_text_to_bounds(value, None, 0, 0, false, false, Style::default(), false);
        width as f32
    }

    fn offset(
        &self,
        text_bounds: iced_core::Rectangle,
        _font: <Self as iced_native::text::Renderer>::Font,
        _size: u16,
        value: &iced_native::text_input::Value,
        state: &iced_native::text_input::State,
    ) -> f32 {
        let cursor_state = state.cursor().state(value);
        let text_bounds_length = text_bounds.width as u16;

        let focused_index = match cursor_state {
            text_input::cursor::State::Index(cursor_index) => cursor_index as u16,
            text_input::cursor::State::Selection { start: _, end } => end as u16,
        };

        if focused_index > text_bounds_length {
            return (focused_index - text_bounds_length) as f32;
        }

        0_f32
    }

    fn draw(
        &mut self,
        bounds: iced_core::Rectangle,
        text_bounds: iced_core::Rectangle,
        cursor_position: iced_core::Point,
        font: <Self as iced_native::text::Renderer>::Font,
        size: u16,
        placeholder: &str,
        value: &iced_native::text_input::Value,
        state: &iced_native::text_input::State,
        style: &<Self as iced_native::text_input::Renderer>::Style,
    ) -> <Self as iced_native::Renderer>::Output {
        let offset = self.offset(text_bounds, font, size, value, state);

        let mut rendered_string = value.to_string();
        let mut rendered_is_placeholder = false;

        if rendered_string.is_empty() && !state.is_focused() {
            rendered_string = placeholder.to_string();
            rendered_is_placeholder = true;
        }

        let start_x = text_bounds.x.round() as u16;
        let start_y = text_bounds.y.round() as u16;
        let text_bounds_width = text_bounds.width.round() as u16;

        let main_style = style.normal.try_merge(if state.is_focused() {
            Some(style.focused)
        } else if bounds.contains(cursor_position) {
            Some(style.hover)
        } else {
            None
        });

        let text_style = if rendered_is_placeholder {
            main_style.merge(style.placeholder)
        } else {
            main_style
        };

        let (mut parsed_primitives, _, _) = crop_text_to_bounds(
            &rendered_string,
            None,
            start_x,
            start_y,
            false,
            true,
            text_style,
            false,
        );
        let mut parsed_primitives_drain = parsed_primitives.drain((offset as usize)..);
        let mut result_primitives = Vec::with_capacity(text_bounds.width as usize);

        for index in 0..text_bounds_width {
            let primitive = match parsed_primitives_drain.next() {
                Some(Primitive::Cell(_, _, cell)) => {
                    Primitive::Cell(start_x + index, start_y, cell)
                }
                _ => Primitive::Cell(
                    start_x + index,
                    start_y,
                    Cell {
                        content: None,
                        style: main_style,
                    },
                ),
            };

            result_primitives.push(primitive);
        }

        if state.is_focused() {
            let cursor_state = state.cursor().state(value);

            match cursor_state {
                text_input::cursor::State::Index(cursor_index) => {
                    result_primitives.push(Primitive::CursorPosition(
                        start_x + (cursor_index as u16) - offset as u16,
                        start_y,
                        style.cursor,
                    ))
                }
                text_input::cursor::State::Selection { start: _, end } => {
                    result_primitives.push(Primitive::CursorPosition(
                        start_x + (end as u16) - offset as u16,
                        start_y,
                        style.cursor,
                    ))
                }
            };
        }

        Primitive::Group(result_primitives)
    }
}
