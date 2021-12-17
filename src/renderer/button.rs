use super::primitives::{Cell, Primitive};
use super::tui_renderer::TuiRenderer;
use super::utils::round_individual_layout;
use crate::Style;
use iced_native::button;
use iced_native::Layout;

#[derive(Debug, Clone, Copy)]
pub struct ButtonStyle {
    pub(crate) normal: Style,
    pub(crate) hover: Style,
    pub(crate) pressed: Style,
    pub(crate) disabled: Style,
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self {
            normal: Style::default(),
            hover: Style::default(),
            pressed: Style::default(),
            disabled: Style::default(),
        }
    }
}

impl ButtonStyle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn normal(mut self, normal: Style) -> Self {
        self.normal = normal;
        self
    }

    pub fn hover(mut self, hover: Style) -> Self {
        self.hover = hover;
        self
    }

    pub fn pressed(mut self, pressed: Style) -> Self {
        self.pressed = pressed;
        self
    }

    pub fn disabled(mut self, disabled: Style) -> Self {
        self.disabled = disabled;
        self
    }
}

impl button::Renderer for TuiRenderer {
    const DEFAULT_PADDING: u16 = 0;

    type Style = ButtonStyle;

    fn draw<Message>(
        &mut self,
        defaults: &<Self as iced_native::Renderer>::Defaults,
        bounds: iced_core::Rectangle,
        cursor_position: iced_core::Point,
        is_disabled: bool,
        is_pressed: bool,
        button_style: &<Self as iced_native::button::Renderer>::Style,
        content: &iced_native::Element<'_, Message, Self>,
        content_layout: iced_native::Layout<'_>,
    ) -> <Self as iced_native::Renderer>::Output {
        let (layout_offset, node) = round_individual_layout(bounds, content_layout, content, self);
        let new_elem_layout = Layout::with_offset(layout_offset, &node);

        let content_primitive =
            content.draw(self, defaults, new_elem_layout, cursor_position, &bounds);

        let selected_style = button_style.normal.try_merge(if is_disabled {
            Some(button_style.disabled)
        } else if is_pressed {
            Some(button_style.pressed)
        } else if bounds.contains(cursor_position) {
            Some(button_style.hover)
        } else {
            None
        });

        let rectangle = Primitive::rectangle(
            bounds.x.round() as u16,
            bounds.y.round() as u16,
            bounds.width.round() as u16,
            bounds.height.round() as u16,
            Cell {
                style: selected_style,
                ..Cell::default()
            },
        );

        Primitive::merge(vec![rectangle, content_primitive])
    }
}
