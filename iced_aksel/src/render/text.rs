use ab_glyph::{Font, FontVec};
use iced_core::{
    Color, Pixels, Point,
    alignment::{Horizontal, Vertical},
};
use std::fmt;
use ttf_parser::Face;

use crate::Quality;

// Embed the Roboto Regular font into the binary.
// This ensures the application always has a high-quality fallback font available.
const DEFAULT_FONT_DATA: &[u8] =
    include_bytes!("../../fonts/ibm_plex_mono/IBMPlexMono-Regular.ttf");

/// A container for parsed font data, optimized for geometric rendering.
///
/// This struct holds the parsed tables required to extract vector paths from a font file.
/// You should generally create this once and store it (e.g., in your application state),
/// rather than creating it every frame, as parsing headers can be expensive.
#[derive(Clone)]
pub struct GeometricFont<'a> {
    /// The high-level layout interface (used for calculating text width and kerning).
    pub(crate) layout: &'a FontVec,
    /// The low-level geometry interface (used for extracting Bezier curves).
    pub(crate) geometry: Face<'a>,
}

impl<'a> GeometricFont<'a> {
    /// Parses raw font bytes (TTF/OTF) into a `GeometricFont`.
    ///
    /// Returns `None` if the font data is malformed or incompatible.
    pub fn new(layout: &'a FontVec) -> Option<Self> {
        // Attempt to parse the font for geometry extraction (curves, points)
        // We assume index 0 for font collections (.ttc).
        let geometry = Face::parse(layout.as_slice(), 0).ok()?;

        Some(Self { layout, geometry })
    }

    /// Returns an iterator over all unique characters available in this font.
    ///
    /// This is useful for debugging or generating a catalogue of available glyphs.
    pub fn characters(&self) -> impl Iterator<Item = char> + '_ {
        self.layout
            .codepoint_ids()
            .map(|(_glyph_id, character)| character)
    }
}

// Manual Debug implementation to avoid printing massive internal font tables.
impl<'a> fmt::Debug for GeometricFont<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GeometricFont")
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
}
