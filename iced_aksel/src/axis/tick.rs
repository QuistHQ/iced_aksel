use super::Orientation;
use super::label::LabelBounds;
use crate::style::{GridStyle, TextStyle, TickStyle};
use aksel::{Float, Tick};
use derivative::Derivative;
use iced_core::{Color, Point, Rectangle, text, text::paragraph::Plain};

#[derive(Debug, Clone)]
pub struct TickResult {
    pub(crate) tick_style: TickStyle,
    pub(crate) grid_style: Option<GridStyle>,
    pub(crate) label: Option<String>,
    pub(crate) label_priority: Option<u8>,
}

impl TickResult {
    pub fn from_tick_style(tick_style: TickStyle) -> Self {
        Self {
            tick_style,
            grid_style: None,
            label: None,
            label_priority: None,
        }
    }

    pub fn label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }

    pub fn grid_style(mut self, style: GridStyle) -> Self {
        self.grid_style = Some(style);
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TickContext<'a, D> {
    pub tick: Tick<D>,
    // The "Perfect Default" passed from the Theme
    pub tick_style: &'a TickStyle,
    pub grid_style: &'a GridStyle,

    pub normalized_position: f32,
    pub axis_bounds: Rectangle,
    pub scale_domain: (D, D),
    pub orientation: Orientation,
}

impl<D: Float> TickContext<'_, D> {
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

// ... (Keep Label Logic structs: PlacedLabelInfo, LabelCandidate, etc. - unchanged) ...
// (I am omitting them here for brevity, but they must remain in the file)
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
    pub text: String,
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
