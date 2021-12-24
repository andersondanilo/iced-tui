use super::colors::TermColor;
use iced_native::Color;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Style {
    pub(crate) fg_color: Option<TermColor>,
    pub(crate) bg_color: Option<TermColor>,
    pub(crate) is_bold: bool,
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.fg_color.is_none() && self.bg_color.is_none() && !self.is_bold
    }

    pub fn try_merge(self, other: Option<Self>) -> Self {
        match other {
            Some(style) => self.merge(style),
            None => self,
        }
    }

    pub fn merge(mut self, other: Self) -> Self {
        if other.fg_color.is_some() {
            self.fg_color = other.fg_color;
        }

        if other.bg_color.is_some() {
            self.bg_color = other.bg_color;
        }

        if other.is_bold {
            self.is_bold = other.is_bold;
        }

        self
    }

    pub fn bold(mut self) -> Self {
        self.is_bold = true;
        self
    }

    pub fn bg<C>(mut self, color: C) -> Self
    where
        C: Into<Color>,
    {
        self.bg_color = Some(color.into().into());
        self
    }

    pub fn fg<C>(mut self, color: C) -> Self
    where
        C: Into<Color>,
    {
        self.fg_color = Some(color.into().into());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CursorShape {
    UnderScore,
    Line,
    Block,
}

impl Default for CursorShape {
    fn default() -> Self {
        Self::Line
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CursorStyle {
    pub(crate) shape: CursorShape,
    pub(crate) blinking: bool,
}

impl Default for CursorStyle {
    fn default() -> Self {
        Self {
            shape: CursorShape::default(),
            blinking: true,
        }
    }
}

impl CursorStyle {
    pub fn blinking(mut self, enabled: bool) -> Self {
        self.blinking = enabled;
        self
    }
}
