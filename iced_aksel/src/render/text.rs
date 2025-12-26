use ab_glyph::{Font, FontVec};
use iced_core::{
    Color, Pixels, Point,
    alignment::{Horizontal, Vertical},
};
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};
use ttf_parser::Face;

use crate::Quality;

static NEXT_FONT_ID: AtomicUsize = AtomicUsize::new(0);

/// A container for parsed font data, optimized for geometric rendering.
#[derive(Clone)]
pub struct GeometricFont<'a> {
    /// The high-level layout interface (used for calculating text width and kerning).
    pub(crate) layout: &'a FontVec,
    /// The low-level geometry interface (used for extracting Bezier curves).
    pub(crate) geometry: Face<'a>,
    /// Unique identifier for caching purposes.
    pub id: usize,
}

impl<'a> GeometricFont<'a> {
    /// Parses raw font bytes (TTF/OTF) into a `GeometricFont`.
    pub fn new(layout: &'a FontVec) -> Option<Self> {
        let geometry = Face::parse(layout.as_slice(), 0).ok()?;
        let id = NEXT_FONT_ID.fetch_add(1, Ordering::Relaxed);

        Some(Self {
            layout,
            geometry,
            id,
        })
    }

    /// Returns an iterator over all unique characters available in this font.
    pub fn characters(&self) -> impl Iterator<Item = char> + '_ {
        self.layout
            .codepoint_ids()
            .map(|(_glyph_id, character)| character)
    }
}

impl<'a> fmt::Debug for GeometricFont<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GeometricFont")
            .field("id", &self.id)
            .field("layout", &"ab_glyph::FontRef")
            .field("geometry", &"ttf_parser::Face")
            .finish()
    }
}

// A Text to draw on the screen
pub struct Text<'a> {
    pub content: &'a str,
    pub position: Point,
    pub size: Pixels,
    pub rotation: f32,
    pub horizontal_alignment: Horizontal,
    pub vertical_alignment: Vertical,
    pub fill: Color,
    pub quality: Quality,
    pub letter_spacing: f32,
}
