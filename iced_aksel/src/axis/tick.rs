use super::Orientation;
use super::label::LabelBounds;
use crate::style::{GridStyle, TextStyle, TickStyle};
use aksel::{Float, Tick};
use derivative::Derivative;
use iced_core::{Color, Font, Pixels, Point, Rectangle, text, text::paragraph::Plain};

/// The result returned from a tick renderer function.
#[derive(Debug, Clone)]
pub struct TickResult {
    pub(crate) tick_style: Option<TickStyle>,
    pub(crate) grid_style: Option<GridStyle>,
    pub(crate) label: Option<String>,
    pub(crate) label_priority: Option<u8>,
}

impl TickResult {
    /// Start with a base style (usually `*ctx.style`).
    pub fn empty() -> Self {
        Self {
            tick_style: None,
            grid_style: None,
            label: None,
            label_priority: None,
        }
    }

    /// Set the label text.
    pub fn label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }

    /// set the label priority
    pub fn label_priority(mut self, priority: u8) -> Self {
        self.label_priority = Some(priority);
        self
    }

    /// Explicitly set the grid style (or disable it with None).
    pub fn grid_style(mut self, style: GridStyle) -> Self {
        self.grid_style = Some(style);
        self
    }

    /// Explicitly set the tick style (or disable it with None).
    pub fn tick_style(mut self, style: TickStyle) -> Self {
        self.tick_style = Some(style);
        self
    }

    // --- Grid Helpers (Convenience) ---

    /// Enable the grid (if not already) and set its color.
    pub fn grid_color(mut self, color: Color) -> Self {
        if let Some(grid) = &mut self.grid_style {
            grid.line.color = color;
        } else {
            // Create a default grid if one doesn't exist, just to set the color
            self.grid_style = Some(GridStyle {
                line: crate::style::LineStyle {
                    color,
                    width: 1.0.into(),
                },
                dashed: false,
            });
        }
        self
    }

    /// Enable the grid (if not already) and set its width.
    pub fn grid_width(mut self, width: impl Into<Pixels>) -> Self {
        let w = width.into();
        if let Some(grid) = &mut self.grid_style {
            grid.line.width = w;
        } else {
            self.grid_style = Some(GridStyle {
                line: crate::style::LineStyle {
                    color: Color::from_rgb(0.8, 0.8, 0.8), // Placeholder default
                    width: w,
                },
                dashed: false,
            });
        }
        self
    }

    /// Enable the grid (if not already) and set whether it is dashed.
    pub fn grid_dashed(mut self, dashed: bool) -> Self {
        if let Some(grid) = &mut self.grid_style {
            grid.dashed = dashed;
        } else {
            self.grid_style = Some(GridStyle {
                line: crate::style::LineStyle {
                    color: Color::from_rgb(0.8, 0.8, 0.8),
                    width: 1.0.into(),
                },
                dashed,
            });
        }
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TickContext<'a, D> {
    pub tick: Tick<D>,
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
