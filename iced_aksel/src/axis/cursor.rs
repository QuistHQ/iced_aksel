use crate::style::TextStyle;
use iced_core::{Background, Border, Color, Pixels, Shadow};

/// The result returned from a cursor renderer function.
#[derive(Debug, Clone)]
pub struct CursorResult {
    /// The text to display in the cursor badge.
    pub(crate) label: Option<String>,
    /// Style of the cursor line.
    pub(crate) cursor_line: Option<CursorLine>,
    /// Style of the cursor badge (background, border, text).
    pub(crate) cursor_badge: Option<CursorBadge>,
}

impl CursorResult {
    pub fn empty() -> Self {
        Self {
            label: None,
            cursor_line: None,
            cursor_badge: None,
        }
    }

    pub fn label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }

    pub fn cursor_line(mut self, cursor_line: CursorLine) -> Self {
        self.cursor_line = Some(cursor_line);
        self
    }

    pub fn cursor_badge(mut self, cursor_badge: CursorBadge) -> Self {
        self.cursor_badge = Some(cursor_badge);
        self
    }
}

/// Visual style of the cursor line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CursorLine {
    pub width: Pixels,
    pub color: Color,
    pub gap: Pixels,
}

/// Visual style of the cursor badge.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CursorBadge {
    pub text_style: TextStyle,
    pub background: Option<Color>,
    pub border: Option<Border>,
    pub shadow: Option<Shadow>,
    pub padding: iced_core::Padding,
}
