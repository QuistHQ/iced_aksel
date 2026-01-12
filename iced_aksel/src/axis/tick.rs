use crate::style::{AxisStyle, TickLineStyle};

use super::{Orientation, label::LabelBounds};

use aksel::{Float, Tick};
use derivative::Derivative;
use iced_core::{
    Color, Pixels, Point, Rectangle,
    text::{self, paragraph::Plain},
};

/// The result returned from a tick renderer function.
///
/// This struct specifies exactly what should be rendered for a specific tick mark.
/// It effectively decouples the tick logic from the rendering logic, allowing
/// for highly customizable axes.
///
/// # Example
///
/// ```rust
/// use iced_aksel::axis::{TickResult, TickLine};
///
/// // Create a tick that has a label and a short line
/// let result = TickResult::with_label("100")
///     .tick_line(TickLine::default());
/// ```
#[derive(Default)]
pub struct TickResult {
    /// Optional tick line mark on the axis.
    pub tick_line: Option<TickLine>,
    /// Optional grid line extending into the plot area.
    pub grid_line: Option<super::GridLine>,
    /// Optional text label for this tick.
    pub label: Option<super::Label>,
    /// Optional label rendering-priority (lower is higher priority).
    pub label_priority: Option<u8>,
}

impl TickResult {
    pub fn with_label(label: super::Label) -> Self {
        Self {
            label: Some(label),
            ..Self::default()
        }
    }

    pub fn label(mut self, label: super::Label) -> Self {
        self.label = Some(label);
        self
    }

    pub fn with_tick_line(tick_line: TickLine) -> Self {
        Self {
            tick_line: Some(tick_line),
            ..Self::default()
        }
    }

    pub const fn tick_line(mut self, tick_line: TickLine) -> Self {
        self.tick_line = Some(tick_line);
        self
    }

    pub fn with_grid_line(grid_line: super::GridLine) -> Self {
        Self {
            grid_line: Some(grid_line),
            ..Self::default()
        }
    }

    pub const fn grid_line(mut self, grid_line: super::GridLine) -> Self {
        self.grid_line = Some(grid_line);
        self
    }

    pub const fn label_priority(mut self, priority: u8) -> Self {
        self.label_priority = Some(priority);
        self
    }
}

/// Context provided to tick renderer functions.
///
/// Contains all the information needed to make decisions about how to render a tick.
#[derive(Debug, Clone, Copy)]
pub struct TickContext<'a, D, Theme = iced_core::Theme> {
    /// The tick value and metadata from the scale.
    pub tick: Tick<D>,
    /// Normalized position (0.0-1.0) along the axis.
    pub normalized_position: f32,
    /// The bounds of the axis in screen coordinates.
    pub axis_bounds: &'a Rectangle,
    /// The domain (min, max) of the scale.
    pub scale_domain: (D, D),
    /// The orientation of the axis (horizontal or vertical).
    pub orientation: &'a Orientation,
    /// The theme of the application
    pub theme: &'a Theme,
    /// The default styling for this context
    pub(super) style: &'a AxisStyle,
}

impl<D: Float, Theme> TickContext<'_, D, Theme> {
    /// Returns the total span of the axis in screen pixels.
    pub const fn axis_span(&self) -> f32 {
        match self.orientation {
            Orientation::Horizontal => self.axis_bounds.width,
            Orientation::Vertical => self.axis_bounds.height,
        }
    }

    /// Returns the total span of the scale's domain in data units.
    pub fn scale_span(&self) -> D {
        let (min, max) = self.scale_domain;
        min.abs_sub(max)
    }

    /// Creates a new [`TickLine`] with applied styling. Only one [`TickLine`] can be returned in the
    /// [`TickResult`]
    pub fn tickline(&self) -> TickLine {
        TickLine::from(self.style.tick)
    }

    /// Creates a new [`GridLine`] with applied styling. Only one [`GridLine`] can be returned in the
    /// [`TickResult`]
    pub fn gridline(&self) -> super::GridLine {
        super::GridLine::from(self.style.grid)
    }

    /// Creates a new [`Label`] with applied styling and supplied content. Only one [`Label`] can be returned in the
    /// [`TickResult`]
    pub fn label(&self, content: String) -> super::Label {
        super::Label::from_style(content, self.style.label)
    }

    /// Creates a new [`Label`] with applied styling. Only one [`Label`] can be returned in the
    /// [`TickResult`]
    pub fn label_empty(&self) -> super::Label {
        super::Label::from_style("".to_string(), self.style.label)
    }
}

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

pub struct PrioritizedTick<D> {
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
    pub label: super::Label,
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
    pub color: Color,
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
