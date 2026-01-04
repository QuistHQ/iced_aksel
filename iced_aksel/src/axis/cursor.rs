use crate::style::TextStyle;
use iced_core::{Background, Border, Color, Pixels, Shadow};

/// The result returned from a cursor renderer function.
#[derive(Debug, Clone)]
pub struct CursorResult {
    /// The text to display in the cursor badge.
    pub label: String,
    /// Style of the cursor line.
    pub line: CursorLine,
    /// Style of the cursor badge (background, border, text).
    pub badge: CursorBadge,
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
    pub background: Option<Background>,
    pub border: Option<Border>,
    pub shadow: Option<Shadow>,
    pub padding: iced_core::Padding,
}
