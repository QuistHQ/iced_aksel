use crate::style::TickLineStyle;

use super::{Orientation, label::LabelBounds};

use aksel::Tick;
use derivative::Derivative;
use iced_core::{
    Color, Pixels, Point,
    text::{self, paragraph::Plain},
};

/// Defines the visual styling of a single tick mark on an Axis.
#[derive(Debug, Clone)]
pub struct TickLine {
    /// The visual thickness (stroke width) of the tick line.
    pub width: Pixels,

    /// The length of the tick line perpendicular to the axis.
    pub length: Pixels,

    /// The color of the tickline
    pub color: Color,
}

impl From<TickLineStyle> for TickLine {
    fn from(value: TickLineStyle) -> Self {
        Self {
            width: value.width,
            length: 5.0.into(),
            color: value.color,
        }
    }
}

/// Information about a label that has been accepted for rendering.
///
/// Used internally for overlap detection.
#[derive(Debug, Clone)]
pub struct PlacedLabelInfo<D> {
    /// The tick associated with this label.
    pub tick: Tick<D>,
    /// Normalized position (0.0-1.0) along the axis.
    pub normalized_position: f32,
    /// The spatial bounds of the label.
    pub bounds: LabelBounds,
}

pub(crate) struct PrioritizedTick<D> {
    pub tick: aksel::Tick<D>,
    /// 0.0 = Major Tick (Critical)
    /// 1.0 = Center of Interval (High Priority)
    /// 1.5 = Edge of Interval (Low Priority)
    pub score: f32,
}

/// A decision on whether to render or skip a tick label.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelDecision {
    /// Render this label at its position.
    Render,
    /// Skip rendering this label (e.g., due to overlap).
    Skip,
}

/// A candidate label that may or may not be rendered.
///
/// Used internally during the label layout process.
pub struct LabelCandidate<D> {
    pub tick: Tick<D>,
    pub normalized_position: f32,
    pub label: String,
    pub priority: u8,
}

/// A label candidate that has been laid out and measured.
///
/// Used internally during the label rendering process.
pub struct ResolvedLabelCandidate<Renderer, D>
where
    Renderer: text::Renderer,
{
    pub tick: Tick<D>,
    pub normalized_position: f32,
    pub bounds: LabelBounds,
    pub paragraph: Plain<Renderer::Paragraph>,
    pub position: Point,
}

/// Context provided to custom label policy functions.
#[derive(Debug)]
pub struct LabelDecisionContext<'a, D> {
    /// The tick associated with this label.
    pub tick: Tick<D>,
    /// Normalized position (0.0-1.0) along the axis.
    pub normalized_position: f32,
    /// The calculated screen bounds of this label.
    pub bounds: LabelBounds,
    /// The orientation of the axis.
    pub orientation: Orientation,
    /// Labels that have already been accepted for rendering in this pass.
    pub accepted: &'a [PlacedLabelInfo<D>],
}

type LabelPolicyFn<D> = dyn for<'a> Fn(LabelDecisionContext<'a, D>) -> LabelDecision + 'static;

/// Policy for determining which axis labels to render.
///
/// Controls label visibility and overlap detection to ensure readable axis labels.
#[derive(Derivative, Default)]
#[derivative(Debug)]
pub enum LabelPolicy<D> {
    /// Render all labels without any overlap detection.
    #[default]
    All,
    /// Skip labels that would overlap with already-placed labels.
    SkipOverlapping {
        /// Minimum gap in pixels between labels.
        min_gap: f32,
    },
    /// Use a custom function to decide which labels to render.
    Custom(#[derivative(Debug = "ignore")] Box<LabelPolicyFn<D>>),
}

impl<D> LabelPolicy<D> {
    /// Creates a policy that renders all labels.
    pub const fn all() -> Self {
        Self::All
    }

    /// Creates a policy that skips overlapping labels with the specified minimum gap.
    pub const fn skip_overlapping(min_gap: f32) -> Self {
        Self::SkipOverlapping { min_gap }
    }

    /// Creates a custom label policy using the provided function.
    pub fn custom<F>(policy: F) -> Self
    where
        F: for<'a> Fn(LabelDecisionContext<'a, D>) -> LabelDecision + 'static,
    {
        Self::Custom(Box::new(policy))
    }

    pub(crate) fn should_render(&self, context: LabelDecisionContext<'_, D>) -> bool {
        match self {
            Self::All => true,
            Self::SkipOverlapping { min_gap } => context
                .accepted
                .iter()
                .all(|placed| !context.bounds.overlaps_with_gap(&placed.bounds, *min_gap)),
            Self::Custom(policy) => matches!(policy(context), LabelDecision::Render),
        }
    }
}
