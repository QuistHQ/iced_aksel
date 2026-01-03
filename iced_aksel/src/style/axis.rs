use crate::style::{ContainerStyle, LineStyle};
use iced_core::Pixels;

/// Style of a `Chart`'s axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisStyle {
    /// Style of the axis container (background, border, shadow).
    pub container: ContainerStyle,
    /// Style of the axis line (the spine).
    pub line: LineStyle,
    /// Distance from the Axis Line to the text baseline (The "Rail").
    pub text_offset: Pixels,
}

/// Style of the axis line (spine).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisLineStyle {
    /// The color of the axis line.
    pub color: iced_core::Color,
    /// The thickness of the axis line.
    pub width: Pixels,
}