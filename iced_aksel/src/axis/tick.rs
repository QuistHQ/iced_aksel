use super::{Orientation, label::LabelBounds};
use crate::axis::GridLine;
use crate::style::TextStyle;

use aksel::{Float, Tick};
use derivative::Derivative;
use iced_core::{
    Color, Pixels, Point, Rectangle,
    text::{self, paragraph::Plain},
};

/// The result returned from a tick renderer function.
pub struct TickResult {
    /// Optional tick line mark on the axis.
    pub tick_line: Option<TickLine>,
    /// Optional grid line extending into the plot area.
    pub grid_line: Option<GridLine>,
    /// Optional text label for this tick.
    pub label: Option<String>,
    /// Style for the label text (font, color, size).
    /// If None, a default style is used.
    pub label_style: Option<TextStyle>,
    /// Optional label rendering-priority (lower is higher priority).
    pub label_priority: Option<u8>,
}

impl TickResult {
    /// Creates a new empty `TickResult` (no lines, no label).
    pub const fn new() -> Self {
        Self {
            tick_line: None,
            grid_line: None,
            label: None,
            label_style: None,
            label_priority: None,
        }
    }

    /// Creates a new `TickResult` with a specific label.
    pub fn with_label<L: Into<String>>(label: L) -> Self {
        Self {
            label: Some(label.into()),
            ..Self::new()
        }
    }

    pub fn with_tick_line(line: TickLine) -> Self {
        Self {
            tick_line: Some(line),
            ..Self::new()
        }
    }

    pub fn with_grid_line(line: GridLine) -> Self {
        Self {
            grid_line: Some(line),
            ..Self::new()
        }
    }

    pub fn text_style(mut self, style: TextStyle) -> Self {
        self.label_style = Some(style);
        self
    }

    pub fn label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }

    pub const fn tick_line(mut self, line: TickLine) -> Self {
        self.tick_line = Some(line);
        self
    }

    pub const fn grid_line(mut self, line: GridLine) -> Self {
        self.grid_line = Some(line);
        self
    }
}

/// Defines the visual styling of a single tick mark on an Axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TickLine {
    /// The visual thickness (stroke width) of the tick line.
    pub thickness: Pixels,
    /// The length of the tick line perpendicular to the axis.
    pub length: Pixels,
    /// The color of the tick line.
    pub color: Color,
}

/// Context provided to tick renderer functions.
#[derive(Debug, Clone, Copy)]
pub struct TickContext<D> {
    pub tick: Tick<D>,
    pub normalized_position: f32,
    pub axis_bounds: Rectangle,
    pub scale_domain: (D, D),
    pub orientation: Orientation,
}

impl<D: Float> TickContext<D> {
    pub const fn axis_span(&self) -> f32 {
        match self.orientation {
            Orientation::Horizontal => self.axis_bounds.width,
            Orientation::Vertical => self.axis_bounds.height,
        }
    }

    pub fn scale_span(&self) -> D {
        let (min, max) = self.scale_domain;
        min.abs_sub(max)
    }
}

#[derive(Debug, Clone)]
pub struct PlacedLabelInfo<D> {
    pub tick: Tick<D>,
    pub normalized_position: f32,
    pub bounds: LabelBounds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelDecision {
    Render,
    Skip,
}

pub struct LabelCandidate<D> {
    pub tick: Tick<D>,
    pub normalized_position: f32,
    pub label: String,
    pub style: TextStyle,
    pub priority: u8,
}

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

#[derive(Debug)]
pub struct LabelDecisionContext<'a, D> {
    pub tick: Tick<D>,
    pub normalized_position: f32,
    pub bounds: LabelBounds,
    pub orientation: Orientation,
    pub accepted: &'a [PlacedLabelInfo<D>],
}

type LabelPolicyFn<D> = dyn for<'a> Fn(LabelDecisionContext<'a, D>) -> LabelDecision + 'static;

#[derive(Derivative, Default)]
#[derivative(Debug)]
pub enum LabelPolicy<D> {
    #[default]
    All,
    SkipOverlapping {
        min_gap: f32,
    },
    Custom(#[derivative(Debug = "ignore")] Box<LabelPolicyFn<D>>),
}

impl<D> LabelPolicy<D> {
    pub const fn all() -> Self {
        Self::All
    }

    pub const fn skip_overlapping(min_gap: f32) -> Self {
        Self::SkipOverlapping { min_gap }
    }

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
