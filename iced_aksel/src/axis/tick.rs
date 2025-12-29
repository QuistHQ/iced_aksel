use crate::axis::GridLine;

use super::{Orientation, label::LabelBounds};

use aksel::{Float, Tick};
use derivative::Derivative;
use iced_core::{
    Pixels, Point, Rectangle,
    text::{self, paragraph::Plain},
};

/// The result returned from a tick renderer function.
///
/// Specifies which visual elements should be rendered for a given tick.
pub struct TickResult {
    /// Optional tick line mark on the axis
    pub tick_line: Option<TickLine>,
    /// Optional grid line extending into the plot area
    pub grid_line: Option<GridLine>,
    /// Optional text label for this tick
    pub label: Option<String>,
    /// Optional label rendering-priority
    pub label_priority: Option<u8>,
}

impl Default for TickResult {
    fn default() -> Self {
        Self {
            tick_line: Some(TickLine::default()),
            grid_line: Some(GridLine::default()),
            label: None,
            label_priority: None,
        }
    }
}

impl TickResult {
    /// Creates a new empty [TickResult]
    pub const fn new() -> Self {
        Self {
            tick_line: None,
            grid_line: None,
            label: None,
            label_priority: None,
        }
    }

    /// Creates a new empty [TickResult] with a label
    pub fn with_label<L: Into<String>>(label: L) -> Self {
        Self {
            label: Some(label.into()),
            ..Self::new()
        }
    }

    /// Creates a new [TickResult] with a tick-line
    pub fn with_tick_line(line: TickLine) -> Self {
        Self {
            tick_line: Some(line),
            ..Self::new()
        }
    }

    /// Creates a new [TickResult] with a grid-line
    pub fn with_grid_line(line: GridLine) -> Self {
        Self {
            grid_line: Some(line),
            ..Self::new()
        }
    }

    /// Sets the rendering-priority of the label
    ///
    /// 0 == Highest priority
    pub const fn label_priority(mut self, priority: u8) -> Self {
        self.label_priority = Some(priority);
        self
    }

    /// Adds a label to the [TickResult]
    pub fn label<L: Into<String>>(mut self, label: L) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Adds a tick-line to the [TickResult]
    pub const fn tick_line(mut self, line: TickLine) -> Self {
        self.tick_line = Some(line);
        self
    }

    /// Adds a grid-line to the [TickResult]
    pub const fn grid_line(mut self, line: GridLine) -> Self {
        self.grid_line = Some(line);
        self
    }
}

/// Defines the visual styling and content of a single tick mark on an Axis.
#[derive(Debug, Clone)]
pub struct TickLine {
    /// The visual thickness (stroke width) of the tick line.
    pub thickness: Pixels,

    /// The length of the tick line perpendicular to the axis.
    pub length: Pixels,
}

impl Default for TickLine {
    #[inline(always)]
    fn default() -> Self {
        Self {
            thickness: Pixels(1.0),
            length: Pixels(5.0),
        }
    }
}

/// Context provided to tick renderer functions.
///
/// Contains all the information needed to render a tick appropriately.
#[derive(Debug, Clone, Copy)]
pub struct TickContext<D> {
    /// The tick from the scale
    pub tick: Tick<D>,
    /// Normalized position (0.0-1.0) along the axis
    pub normalized_position: f32,
    /// The bounds of the axis in screen coordinates
    pub axis_bounds: Rectangle,
    /// The domain (min, max) of the scale
    pub scale_domain: (D, D),
    /// The orientation of the axis (horizontal or vertical)
    pub orientation: Orientation,
}

impl<D: Float> TickContext<D> {
    /// Returns the span of the axis in screen pixels.
    pub const fn axis_span(&self) -> f32 {
        match self.orientation {
            Orientation::Horizontal => self.axis_bounds.width,
            Orientation::Vertical => self.axis_bounds.height,
        }
    }

    /// Returns the span of the scale's domain.
    pub fn scale_span(&self) -> D {
        let (min, max) = self.scale_domain;
        min.abs_sub(max)
    }
}

/// Information about a label that has been placed on the axis.
///
/// Used internally for overlap detection and spacing calculations.
#[derive(Debug, Clone)]
pub struct PlacedLabelInfo<D> {
    /// The tick associated with this label
    pub tick: Tick<D>,
    /// Normalized position (0.0-1.0) along the axis
    pub normalized_position: f32,
    /// The spatial bounds of the label
    pub bounds: LabelBounds,
}

/// Decision on whether to render or skip a tick label.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelDecision {
    /// Render this label at its position.
    Render,
    /// Skip rendering this label (e.g., due to overlap or custom filtering).
    Skip,
}

/// A candidate label that may or may not be rendered.
///
/// Used internally during the label rendering process.
pub struct LabelCandidate<D> {
    /// The tick associated with the label
    pub tick: Tick<D>,
    /// Normalized position (0.0-1.0) along the axis
    pub normalized_position: f32,
    /// The label to be rendered
    pub label: String,
    /// Rendering-priority of the label
    pub priority: u8,
}

/// A label candidate that has been laid out and measured.
///
/// Used internally during the label rendering process.
pub struct ResolvedLabelCandidate<Renderer, D>
where
    Renderer: text::Renderer,
{
    /// The tick associated with the label
    pub tick: Tick<D>,
    /// Normalized position (0.0-1.0) along the axis
    pub normalized_position: f32,
    /// The bounds that have been laid out for the label
    pub bounds: LabelBounds,
    /// The Iced-paragraph to render
    pub paragraph: Plain<Renderer::Paragraph>,
    /// The screen-position to render at
    pub position: Point,
}

/// Context provided to custom label policy functions.
#[derive(Debug)]
pub struct LabelDecisionContext<'a, D> {
    /// The tick associated with this label
    pub tick: Tick<D>,
    /// Normalized position (0.0-1.0) along the axis
    pub normalized_position: f32,
    /// The spatial bounds of this label
    pub bounds: LabelBounds,
    /// The orientation of the axis
    pub orientation: Orientation,
    /// Labels that have already been accepted for rendering
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
        /// Minimum gap in pixels between labels
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
