use iced_core::{Color, Pixels, Background, Border, Shadow};
use crate::style::TextStyle;

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

impl Default for CursorResult {
    fn default() -> Self {
        Self {
            label: String::new(),
            line: CursorLine::default(),
            badge: CursorBadge::default(),
        }
    }
}

impl CursorResult {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            ..Self::default()
        }
    }

    pub fn line(mut self, line: CursorLine) -> Self {
        self.line = line;
        self
    }

    pub fn badge(mut self, badge: CursorBadge) -> Self {
        self.badge = badge;
        self
    }
}

/// Visual style of the cursor line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CursorLine {
    pub width: Pixels,
    pub color: Color,
    /// Gap between the cursor line and the badge.
    pub gap: Pixels,
}

impl Default for CursorLine {
    fn default() -> Self {
        Self {
            width: Pixels(1.0),
            color: Color::from_rgb(0.5, 0.5, 0.5),
            gap: Pixels(4.0),
        }
    }
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

impl Default for CursorBadge {
    fn default() -> Self {
        Self {
            text_style: TextStyle::default(),
            background: Some(Background::Color(Color::WHITE)),
            border: Some(Border {
                color: Color::BLACK,
                width: 1.0.into(),
                radius: 4.0.into(),
            }),
            shadow: None,
            padding: 4.0.into(),
        }
    }
}