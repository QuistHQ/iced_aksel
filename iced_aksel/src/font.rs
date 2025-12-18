use ab_glyph::FontRef;
use std::{fmt, sync::OnceLock};
use ttf_parser::Face;

// Embed the default font data globally
const DEFAULT_FONT_DATA: &[u8] = include_bytes!("../fonts/ibm_plex_mono/IBMPlexMono-Regular.ttf");
// The cache for the parsed default font
static DEFAULT_FONT: OnceLock<GeometricFont<'static>> = OnceLock::new();

/// A container for parsed font data, optimized for geometric rendering.
///
/// You should create this once and store it (e.g., in your application state),
/// rather than creating it every frame.
#[derive(Clone)]
pub struct GeometricFont<'a> {
    pub(crate) layout: FontRef<'a>,
    pub(crate) geometry: Face<'a>,
}

// Manual implementation that skips printing the inner fields
impl<'a> fmt::Debug for GeometricFont<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GeometricFont")
            .field("layout", &"FontRef") // Placeholder string
            .field("geometry", &"Face") // Placeholder string
            .finish()
    }
}

impl<'a> GeometricFont<'a> {
    /// Parses raw font bytes (TTF/OTF) into a GeometricFont.
    ///
    /// Returns None if the font is malformed.
    pub fn new(data: &'a [u8]) -> Option<Self> {
        let layout = FontRef::try_from_slice(data).ok()?;
        // ttf-parser is strict; ensure index 0 exists
        let geometry = Face::parse(data, 0).ok()?;

        Some(Self { layout, geometry })
    }
}

/// Returns a reference to the built-in default font (Roboto).
///
/// This function is extremely fast; it parses the font only once on the first call
/// and returns a reference to the cached version thereafter.
pub fn default() -> &'static GeometricFont<'static> {
    DEFAULT_FONT.get_or_init(|| {
        GeometricFont::new(DEFAULT_FONT_DATA)
            .expect("Critical: Failed to parse embedded default font.")
    })
}
