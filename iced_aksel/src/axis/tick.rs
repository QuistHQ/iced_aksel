use crate::style::{AxisStyle, BadgeStyle, TickLineStyle};

use super::{Orientation, label::LabelBounds};

use aksel::{Float, Tick};
use derivative::Derivative;
use iced_core::{
    Border, Color, Pixels, Point, Rectangle, Shadow,
    text::{self, paragraph::Plain},
};

/// The result returned from a tick renderer function.
///
/// This struct specifies exactly what should be rendered for a specific tick mark.
/// It effectively decouples the tick logic from the rendering logic, allowing
/// for highly customizable axes.
///
/// See [`crate::Axis`] for more info
#[derive(Default)]
pub struct TickResult {
    /// Optional tick line mark on the axis.
    pub tick_line: Option<TickLine>,
    /// Optional grid line extending into the plot area.
    pub grid_line: Option<super::GridLine>,
    /// Optional text label for this tick.
    pub label: Option<super::Label>,
    /// Optional badge to render behind the label.
    pub label_badge: Option<LabelBadge>,
    /// Optional label rendering-priority (lower is higher priority).
    pub label_priority: Option<u8>,
}

impl TickResult {
    /// Creates a new result containing only the specified text label.
    pub fn with_label(label: super::Label) -> Self {
        Self {
            label: Some(label),
            ..Self::default()
        }
    }

    /// Attaches a text label to this result.
    pub fn label(mut self, label: super::Label) -> Self {
        self.label = Some(label);
        self
    }

    /// Creates a new result containing only the specified axis tick mark.
    pub fn with_tick_line(tick_line: TickLine) -> Self {
        Self {
            tick_line: Some(tick_line),
            ..Self::default()
        }
    }

    /// Adds an axis tick mark to this result.
    pub const fn tick_line(mut self, tick_line: TickLine) -> Self {
        self.tick_line = Some(tick_line);
        self
    }

    /// Creates a new result containing only the specified grid line.
    pub fn with_grid_line(grid_line: super::GridLine) -> Self {
        Self {
            grid_line: Some(grid_line),
            ..Self::default()
        }
    }

    /// Adds a background grid line extending from this tick.
    pub const fn grid_line(mut self, grid_line: super::GridLine) -> Self {
        self.grid_line = Some(grid_line);
        self
    }

    /// Creates a new result containing only the specified badge.
    pub fn with_badge(badge: LabelBadge) -> Self {
        Self {
            label_badge: Some(badge),
            ..Self::default()
        }
    }

    /// Adds a badge to this result.
    pub const fn badge(mut self, badge: LabelBadge) -> Self {
        self.label_badge = Some(badge);
        self
    }

    /// Sets the collision priority for the label (lower values are less likely to be hidden).
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
    /// The index of the current tick
    pub index: usize,
    /// The total amount of ticks to be rendered
    pub total_ticks: usize,
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

    /// Creates a new [`super::GridLine`] with applied styling. Only one [`super::GridLine`] can be returned in the
    /// [`TickResult`]
    pub fn gridline(&self) -> super::GridLine {
        super::GridLine::from(self.style.grid)
    }

    /// Creates a new [`super::Label`] with applied styling and supplied content. Only one [`super::Label`] can be returned in the
    /// [`TickResult`]
    pub const fn label(&self, content: String) -> super::Label {
        super::Label::from_style(content, self.style.label)
    }

    /// Creates a new [`super::Label`] with applied styling. Only one [`super::Label`] can be returned in the
    /// [`TickResult`]
    pub fn label_empty(&self) -> super::Label {
        super::Label::from_style("".to_string(), self.style.label)
    }
    /// Creates a new [`LabelBadge`] with applied styling. Only one [`LabelBadge`] can be returned in the
    /// [`TickResult`]
    pub const fn label_badge(&self) -> super::LabelBadge {
        self.style.label_badge
    }
}

/// Visual properties for the badge background behind a tick label.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LabelBadge {
    /// The background color of the badge.
    pub background: Color,
    /// The border styling of the badge.
    pub border: Border,
    /// The shadow styling of the badge.
    pub shadow: Shadow,
}

impl From<BadgeStyle> for LabelBadge {
    fn from(value: BadgeStyle) -> Self {
        Self {
            background: value.background,
            border: value.border,
            shadow: value.shadow,
        }
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

/// A tick wrapper used to filter ticks based on visual importance.
///
/// This is used during "Level of Detail" calculations to ensure that critical
/// ticks (like zero-lines or round numbers) are preserved when zooming out,
/// while less important intermediate ticks are dropped.
pub struct PrioritizedTick<D> {
    /// The underlying tick data containing the value and hierarchy level.
    ///
    /// The internal `level` field defines the hierarchical depth of this tick.
    /// This is used to style ticks differently (e.g., longer lines for major ticks)
    /// or to filter them out when the chart is zoomed out.
    ///
    /// # Level Examples
    /// * `0`: **Major Tick** (Critical).
    ///   * *Example:* "10", "20"
    /// * `1`: **Minor Tick** (Intermediate).
    ///   * *Example:* "15", "25"
    /// * `2+`: **Sub-minor Tick** (Detail).
    ///   * *Example:* ""
    ///
    /// This will vary based on the [`aksel::Scale`] you use.
    pub tick: aksel::Tick<D>,

    /// The index of the tick
    pub index: usize,

    /// The importance of this tick (lower values are higher priority).
    ///
    /// * `0.0` - Major Tick (Critical, e.g., Axis Zero)
    /// * `1.0` - Center of Interval (High Priority)
    /// * `1.5` - Edge of Interval (Low Priority)
    pub score: f32,
}

/// A decision on whether to render or skip a tick label.
///
/// This is the output of the collision detection pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelDecision {
    /// The label does not overlap others and should be drawn.
    Render,
    /// The label overlaps a higher-priority label and should be hidden.
    Skip,
}

/// A label that intends to be drawn, but hasn't been measured yet.
///
/// This struct holds the *logical* data for a label (the text, the data position,
/// the priority) before the text engine has calculated how many pixels wide/tall
/// it actually is.
pub struct LabelCandidate<D> {
    /// The underlying tick data containing the value and hierarchy level.
    ///
    /// The internal `level` field defines the hierarchical depth of this tick.
    /// This is used to style ticks differently (e.g., longer lines for major ticks)
    /// or to filter them out when the chart is zoomed out.
    ///
    /// # Level Examples
    /// * `0`: **Major Tick** (Critical).
    ///   * *Example:* "10", "20"
    /// * `1`: **Minor Tick** (Intermediate).
    ///   * *Example:* "15", "25"
    /// * `2+`: **Sub-minor Tick** (Detail).
    ///   * *Example:* ""
    ///
    /// This will vary based on the [`aksel::Scale`] you use.
    pub tick: Tick<D>,
    /// The position on the axis (0.0 to 1.0).
    pub normalized_position: f32,
    /// The style and content of the label.
    pub label: super::Label,
    /// Optional badge
    pub badge: Option<LabelBadge>,
    /// Collision priority (lower is better).
    pub priority: u8,
}

/// A label that has been fully measured and positioned by the text engine.
///
/// Unlike `LabelCandidate`, this struct contains specific physical dimensions (`bounds`)
/// and backend-specific resources (`paragraph`). It represents the final state
/// ready for rendering to the screen.
pub struct ResolvedLabelCandidate<Renderer, D>
where
    Renderer: text::Renderer,
{
    /// The original source tick.
    pub tick: Tick<D>,
    /// The position on the axis (0.0 to 1.0).
    pub normalized_position: f32,
    /// The physical bounding box of the text in screen coordinates.
    pub bounds: LabelBounds,
    /// The backend-specific text object (ready to draw).
    pub paragraph: Plain<Renderer::Paragraph>,
    /// The final screen coordinates for the top-left of the text.
    pub position: Point,
    /// The resolved color.
    pub color: Color,
    /// Badge for the label
    pub badge: Option<LabelBadge>,
    /// The bounds of the whole badge
    pub badge_bounds: Rectangle,
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
