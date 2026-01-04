use ab_glyph::{Font, FontVec};
use iced_core::{
    Color, Pixels, Point,
    alignment::{Horizontal, Vertical},
};
use iced_graphics::text::cosmic_text::fontdb::ID;
use std::fmt;
use ttf_parser::Face;

use crate::{Quality, memory::CachedFont};

/// A container for parsed font data, optimized for geometric rendering.
#[derive(Clone)]
pub struct GeometricFont<'a> {
    /// The high-level layout interface (used for calculating text width and kerning).
    pub(crate) layout: &'a FontVec,
    /// The low-level geometry interface (used for extracting Bezier curves).
    pub(crate) geometry: Face<'a>,
    /// Unique identifier for caching purposes.
    pub id: &'a ID,
}

impl<'a> GeometricFont<'a> {
    /// Parses raw font bytes (TTF/OTF) into a `GeometricFont`.
    pub fn new(font: &'a CachedFont) -> Option<Self> {
        let geometry = Face::parse(font.bytes.as_slice(), 0).ok()?;

        Some(Self {
            layout: &font.bytes,
            geometry,
            id: &font.id,
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
