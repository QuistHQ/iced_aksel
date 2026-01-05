use crate::style::CursorStyle;
use iced_core::{Color, Font, Pixels};

/// The result returned from a cursor renderer function.
#[derive(Debug, Clone)]
pub struct CursorResult {
    pub(crate) label: Option<String>,
    pub(crate) style: CursorStyle,
}

impl CursorResult {
    /// Start with a base style (usually derived from the Theme).
    pub fn from_style(style: CursorStyle) -> Self {
        Self { label: None, style }
    }

    /// Set the label text.
    pub fn label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }

    // --- Badge Style ---

    /// Set the background color of the cursor badge.
    pub fn background(mut self, color: impl Into<Color>) -> Self {
        self.style.badge.background = Some(color.into().into());
        self
    }

    /// Set the text color inside the badge.
    pub fn text_color(mut self, color: Color) -> Self {
        self.style.text.color = color;
        self
    }

    /// Set the font size of the badge text.
    pub fn text_size(mut self, size: impl Into<Pixels>) -> Self {
        self.style.text.size = size.into();
        self
    }

    /// Set the font of the badge text.
    pub fn font(mut self, font: Font) -> Self {
        self.style.text.font = font;
        self
    }

    // --- Line Style ---

    /// Set the color of the cursor line.
    pub fn line_color(mut self, color: Color) -> Self {
        self.style.line.color = color;
        self
    }

    /// Set the width of the cursor line.
    pub fn line_width(mut self, width: impl Into<Pixels>) -> Self {
        self.style.line.width = width.into();
        self
    }

    /// Set the gap between the cursor line and the badge.
    pub fn line_gap(mut self, gap: impl Into<Pixels>) -> Self {
        self.style.line_gap = gap.into();
        self
    }
}
