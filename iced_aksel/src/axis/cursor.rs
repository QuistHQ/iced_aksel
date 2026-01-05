use crate::style::CursorStyle;
use iced_core::Color;

#[derive(Debug, Clone)]
pub struct CursorResult {
    pub(crate) label: Option<String>,
    pub(crate) style: CursorStyle,
}

impl CursorResult {
    pub fn from_style(style: CursorStyle) -> Self {
        Self { label: None, style }
    }

    pub fn label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }

    // Fluent helper to override background color
    pub fn background(mut self, color: Color) -> Self {
        self.style.badge.background = Some(color.into());
        self
    }
}
