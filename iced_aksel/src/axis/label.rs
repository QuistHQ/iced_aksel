use iced_core::{Color, Font, Padding, Pixels, text::LineHeight};

use crate::style::LabelStyle;

/// Configuration for a label on an axis.
///
/// Labels display the numeric or textual value at each tick (or marker) position.
///
/// # Example
///
/// ```rust
/// use iced_aksel::axis::Label;
/// use iced::{Pixels, Padding};
///
/// // Create a custom label
/// let label = Label {
///     size: Pixels(14.0),
///     content: "100.0".to_string(),
///     padding: Padding::new(6.0),
/// };
///
/// // Or use the default (12px, empty content, 4px padding)
/// let default_label = Label::default();
/// ```
#[derive(Debug, Clone)]
pub struct Label {
    /// The font size of the label text.
    pub size: Pixels,
    /// The color of the label
    pub color: Color,
    /// The text content to display.
    pub content: String,
    /// Padding around the label.
    pub padding: Padding,
    /// Override the font used on the label - Defaults to the default font of the renderer
    pub font: Option<Font>,
    /// The lineheight of the label
    pub line_height: LineHeight,
}

impl Label {
    pub fn from_style(content: String, style: LabelStyle) -> Self {
        Self {
            content,
            font: None,
            size: style.size,
            color: style.color,
            padding: style.padding,
            line_height: style.line_height,
        }
    }
}

/// The spatial bounds of a label along the axis.
///
/// Used for overlap detection when rendering labels to prevent text collisions.
///
/// # Example
///
/// ```rust
/// use iced_aksel::axis::LabelBounds;
///
/// let label1 = LabelBounds::new(10.0, 50.0);
/// let label2 = LabelBounds::new(45.0, 85.0);
///
/// // Check if labels overlap with a 5-pixel minimum gap
/// if label1.overlaps_with_gap(&label2, 5.0) {
///     // Skip rendering one label to avoid collision
/// }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct LabelBounds {
    /// The starting position of the label along the axis.
    pub start: f32,
    /// The ending position of the label along the axis.
    pub end: f32,
}

impl LabelBounds {
    /// Creates new label bounds with the given start and end positions.
    pub const fn new(start: f32, end: f32) -> Self {
        Self { start, end }
    }

    /// Checks if this label overlaps with another label, accounting for a minimum gap.
    ///
    /// Returns `true` if the labels are too close together (within `min_gap` pixels).
    pub fn overlaps_with_gap(&self, other: &Self, min_gap: f32) -> bool {
        (self.start < other.end + min_gap) && (other.start < self.end + min_gap)
    }
}
