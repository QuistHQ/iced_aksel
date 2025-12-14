use aksel::Tick;
use iced_core::{
    Padding, Pixels, Point,
    text::{self, paragraph::Plain},
};

use crate::axis::Orientation;

/// Configuration for a tick label on an axis.
///
/// Labels display the numeric or textual value at each tick position.
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
    /// The text content to display.
    pub content: String,
    /// Padding around the label.
    pub padding: Padding,
    /// The priority of the label, if any.
    ///
    /// 0 == Highest priority
    pub priority: Option<u8>,
}

impl Default for Label {
    #[inline(always)]
    fn default() -> Self {
        Self {
            size: Pixels(12.0),
            content: String::default(),
            padding: Padding::new(4.0),
            priority: None,
        }
    }
}

impl From<String> for Label {
    fn from(content: String) -> Self {
        Self {
            content,
            ..Default::default()
        }
    }
}

impl<'a> From<&'a String> for Label {
    fn from(content: &'a String) -> Self {
        Self {
            content: content.clone(),
            ..Default::default()
        }
    }
}

impl From<&str> for Label {
    fn from(content: &str) -> Self {
        Self {
            content: content.to_string(),
            ..Default::default()
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

#[derive(Debug, Clone)]
pub struct PlacedLabelInfo<D> {
    pub tick: Tick<D>,
    pub normalized_position: f32,
    pub bounds: LabelBounds,
}

/// Decision on whether to render or skip a tick label.
///
/// Used by custom label policies provided to [`Axis::with_custom_label_policy`](crate::Axis::with_custom_label_policy).
///
/// # Example
///
/// ```rust
/// use iced_aksel::{Axis, axis::LabelDecision, scale::Linear};
///
/// // Only show labels for even values
/// let axis = Axis::new(Linear::new(0.0, 100.0), iced_aksel::axis::Position::Bottom)
///     .with_custom_label_policy(|ctx| {
///         if ctx.tick.value as i32 % 2 == 0 {
///             LabelDecision::Render
///         } else {
///             LabelDecision::Skip
///         }
///     });
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelDecision {
    /// Render this label at its position.
    Render,
    /// Skip rendering this label (e.g., due to overlap or custom filtering).
    Skip,
}

pub struct LabelCandidate<D> {
    pub(crate) tick: Tick<D>,
    pub(crate) normalized_position: f32,
    pub(crate) label: Label,
}

impl<D> LabelCandidate<D> {
    #[inline(always)]
    pub(crate) fn priority(&self) -> u8 {
        self.label.priority.unwrap_or(self.tick.level)
    }
}

pub struct ResolvedLabelCandidate<Renderer, D>
where
    Renderer: text::Renderer,
{
    pub(crate) tick: Tick<D>,
    pub(crate) normalized_position: f32,
    pub(crate) bounds: LabelBounds,
    pub(crate) paragraph: Plain<Renderer::Paragraph>,
    pub(crate) position: Point,
}

#[derive(Debug)]
pub struct LabelDecisionContext<'a, D> {
    pub tick: Tick<D>,
    pub normalized_position: f32,
    pub bounds: LabelBounds,
    pub orientation: Orientation,
    pub accepted: &'a [PlacedLabelInfo<D>],
}
