use crate::axis::GridLine;

use super::{
    Orientation,
    label::{Label, LabelBounds},
};

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
    pub label: Option<Label>,
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
    pub fn with_label<L: Into<Label>>(label: L) -> Self {
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
    pub fn label<L: Into<Label>>(mut self, label: L) -> Self {
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
///
/// # The Tick Rendering Pipeline
///
/// Understanding `TickLine` requires understanding how `aksel` processes axes.
/// The relationship flows as follows:
///
/// 1.  **Generation (`Scale`):** The [`Scale`](crate::Scale) (e.g., `Linear`, `Log`) calculates logical positions
///     and importance levels for potential ticks, creating [`Tick`] objects.
/// 2.  **Contextualization (`Axis`):** The Axis wraps each logical [`Tick`] into a [`TickLabelContext`](crate::axis::TickLabelContext).
///     This provides context like the axis bounds, domain, and orientation.
/// 3.  **Styling (User Logic):** The user provides a closure via [`Axis::with_tick_renderer`](crate::Axis::with_tick_renderer).
///     This closure receives the context and decides **if** and **how** that tick should be drawn
///     by returning an `Option<TickLine>`.
///
/// This separation allows you to have completely dynamic styling—for example, making every
/// major tick thick and red, while making minor ticks thin and grey, or hiding specific ticks entirely.
///
/// [Image of chart axis anatomy showing major and minor ticks]
///
/// # Example
///
/// This example demonstrates how to configure an axis to render major ticks with labels
/// and thick lines, while rendering minor ticks as smaller lines without labels.
///
/// ```rust
/// use iced_aksel::{Axis, axis::TickLine, scale::Linear};
/// use iced::Pixels;
///
/// let axis = Axis::new(Linear::new(0.0, 100.0), iced_aksel::axis::Position::Bottom)
///     .with_tick_renderer(|ctx| {
///         match ctx.tick.level {
///             // Major Tick (Level 0): Thicker, longer, and has a text label
///             0 => Some(TickLine {
///                 thickness: Pixels(2.0),
///                 length: Pixels(10.0),
///                 // We use the context's value to format the text
///                 label: Some(iced_aksel::axis::Label {
///                     content: format!("{:.1}", ctx.tick.value),
///                     ..Default::default()
///                 }),
///             }),
///             // Minor Tick (Level 1): Thinner, shorter, no label
///             1 => Some(TickLine {
///                 thickness: Pixels(1.0),
///                 length: Pixels(5.0),
///                 label: None,
///             }),
///             // Any other importance level: Do not draw (return None)
///             _ => None,
///         }
///     });
/// ```
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

/// A candidate label that may or may not be rendered.
///
/// Used internally during the label rendering process.
pub struct LabelCandidate<D> {
    /// The tick associated with the label
    pub tick: Tick<D>,
    /// Normalized position (0.0-1.0) along the axis
    pub normalized_position: f32,
    /// The label to be rendered
    pub label: Label,
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
///
/// Contains information needed to decide whether to render a label.
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
