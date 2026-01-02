use crate::style::{ContainerStyle, LineStyle, TextStyle};
use iced_core::{Padding, Pixels};

/// Style of the cursor both inside the plot, and on axes
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CursorStyle {
    pub axis: AxisCursorStyle,
}

/// Style of a `Chart`'s interactive axis cursor. This is the text shown at the cursor position on top of axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisCursorStyle {
    /// Style of the line from the plot to the text
    pub line: LineStyle,

    /// Distance between the end of the cursor line and the start of the badge.
    pub line_gap: Pixels,

    /// Style of the background and text of the value shown
    pub badge: BadgeStyle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BadgeStyle {
    /// Style of the text inside the badge.
    pub text: TextStyle,

    /// Style of the badge container (background, border, shadow).
    pub container: ContainerStyle,

    /// Padding around the text inside the badge.
    pub padding: Padding,
}
