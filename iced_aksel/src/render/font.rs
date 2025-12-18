use ab_glyph::{Font, FontRef};
use std::fmt;
use std::sync::OnceLock;
use ttf_parser::Face;

// Embed the Roboto Regular font into the binary.
// This ensures the application always has a high-quality fallback font available.
const DEFAULT_FONT_DATA: &[u8] =
    include_bytes!("../../fonts/ibm_plex_mono/IBMPlexMono-Regular.ttf");

// A thread-safe cache for the parsed default font.
// We use OnceLock to lazily parse the font only when it is first requested.
static DEFAULT_FONT: OnceLock<GeometricFont<'static>> = OnceLock::new();

/// A container for parsed font data, optimized for geometric rendering.
///
/// This struct holds the parsed tables required to extract vector paths from a font file.
/// You should generally create this once and store it (e.g., in your application state),
/// rather than creating it every frame, as parsing headers can be expensive.
#[derive(Clone)]
pub struct GeometricFont<'a> {
    /// The high-level layout interface (used for calculating text width and kerning).
    pub(crate) layout: FontRef<'a>,
    /// The low-level geometry interface (used for extracting Bezier curves).
    pub(crate) geometry: Face<'a>,
}

impl<'a> GeometricFont<'a> {
    /// Parses raw font bytes (TTF/OTF) into a `GeometricFont`.
    ///
    /// Returns `None` if the font data is malformed or incompatible.
    pub fn new(data: &'a [u8]) -> Option<Self> {
        // Attempt to parse the font for layout purposes (advance widths, kerning)
        let layout = FontRef::try_from_slice(data).ok()?;

        // Attempt to parse the font for geometry extraction (curves, points)
        // We assume index 0 for font collections (.ttc).
        let geometry = Face::parse(data, 0).ok()?;

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

/// Returns a reference to the built-in default font (Roboto Regular).
///
/// This function is extremely fast; it parses the font only once on the first call
/// and returns a reference to the cached version thereafter.
///
/// # Panics
/// Panics if the embedded font file is corrupted (which should never happen in a release).
pub fn default() -> &'static GeometricFont<'static> {
    DEFAULT_FONT.get_or_init(|| {
        GeometricFont::new(DEFAULT_FONT_DATA)
            .expect("Critical Error: Failed to parse the embedded default font (Roboto-Regular).")
    })
}
