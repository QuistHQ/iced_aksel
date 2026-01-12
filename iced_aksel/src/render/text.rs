use crate::Quality;
use iced_core::{
    Color, Font, Pixels, Point, Size,
    alignment::{Horizontal, Vertical},
    text::Wrapping,
};

/// A Text to draw on the screen
pub struct Text<'a> {
    pub font: Font,
    pub content: &'a str,
    pub position: Point,
    pub size: Pixels,
    pub rotation: f32,
    pub horizontal_alignment: Horizontal,
    pub vertical_alignment: Vertical,
    pub fill: Color,
    pub quality: Quality,
    pub line_height: Pixels,
    pub bounds: Size,
    pub wrapping: Wrapping,
}
