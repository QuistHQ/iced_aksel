use aksel::Float;
use iced_core::{Border, Color, Pixels, Rectangle, Shadow};

use crate::{
    axis::Orientation,
    style::{AxisStyle, BadgeStyle, MarkerLineStyle, MarkerStyle},
};

/// The position of a marker
///
/// Used when setting markers for an axis on a chart via. [`crate::Chart::marker`] or
/// [`crate::Chart::marker_maybe`]
///
/// ```rust,no_run,ignore
/// # use iced_aksel::{Chart, State, axis::{MarkerPosition, MarkerContext}};
/// # let state: State<&'static str, f64> = State::new();
/// Chart::new(&state)
///     // Set marker to follow cursor
///     .marker(&"axis_id", MarkerPosition::Cursor, |ctx: MarkerContext<f64>| {
///         Some(ctx.marker(ctx.value.to_string()))
///     })
///     // Set marker at domain value 100.0
///     .marker(&"axis_id", MarkerPosition::Value(100.0), |ctx: MarkerContext<f64>| {
///         Some(ctx.marker("100".to_string()))
///     })
///     // Set marker in the middle of the axis
///     .marker(&"axis_id", MarkerPosition::Normalized(0.5), |ctx: MarkerContext<f64>| {
///         Some(ctx.marker("Middle".to_string()))
///     });
/// ```
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum MarkerPosition<D> {
    /// A specific value of the domain. The marker will not render if the value is outside of the
    /// axis-bounds
    Value(D),
    /// A normalized value (percentage) of the axis
    Normalized(f32),
    /// Follow the cursor
    Cursor,
}

/// A boxed function that renders a marker given context, returning `Some(Marker)` if rendering should occur.
pub type MarkerRendererFn<D, Theme> = Box<dyn Fn(MarkerContext<D, Theme>) -> Option<Marker>>;

/// A request to render a marker on a specific axis at a given position.
///
/// Created via [`crate::Chart::marker`] or [`crate::Chart::marker_maybe`] and processed during rendering.
pub struct MarkerRequest<'a, AxisId, D, Theme = iced_core::Theme> {
    pub(crate) position: MarkerPosition<D>,
    pub(crate) axis_id: &'a AxisId,
    pub(crate) renderer: MarkerRendererFn<D, Theme>,
}

impl<AxisId: std::hash::Hash + Eq + Clone, D: Float, Theme> MarkerRequest<'_, AxisId, D, Theme> {
    pub(crate) fn create_marker(
        &self,
        axis: &super::Axis<D, Theme>,
        axis_bounds: &Rectangle,
        plot_bounds: &Rectangle,
        cursor: iced_core::mouse::Cursor,
        style: &AxisStyle,
        theme: &Theme,
    ) -> Option<(Marker, f32)> {
        let (&domain_min, &domain_max) = axis.domain();

        let mut style = *style;
        if let Some(style_override) = axis.style_override.as_ref() {
            style_override.borrow_mut()(&mut style)
        };

        let cursor_on_plot = cursor.position_over(*plot_bounds);
        let cursor_on_axis = cursor.position_over(*axis_bounds);

        let (value, normalized_position) = match self.position {
            MarkerPosition::Value(v) if (domain_min..=domain_max).contains(&v) => {
                (v, axis.normalize(&v))
            }
            MarkerPosition::Normalized(n) if ((0.)..=1.).contains(&n) => (axis.denormalize(n), n),
            MarkerPosition::Cursor => {
                let point = cursor_on_plot.or(cursor_on_axis)?;

                let pos = match axis.orientation() {
                    Orientation::Horizontal => point.x,
                    Orientation::Vertical => point.y,
                };

                let normalized = axis.screen_to_normalized(pos, axis_bounds);
                let value = axis.denormalize(normalized);
                (value, normalized)
            }
            _ => return None, // We can't render - Outside of bounds
        };

        let ctx = MarkerContext {
            value,
            normalized_position,
            scale_domain: (domain_min, domain_max),
            style: &style.marker,
            axis_bounds,
            cursor_on_plot: cursor_on_plot.is_some(),
            cursor_on_axis: cursor_on_axis.is_some(),
            theme,
        };

        (self.renderer)(ctx).zip(Some(normalized_position))
    }
}

/// Context provided to marker renderers for creating styled markers.
///
/// Contains the axis value at the marker position and the resolved marker style.
pub struct MarkerContext<'a, D, Theme = iced_core::Theme> {
    /// The axis value at the marker position.
    pub value: D,
    /// Normalized position (0.0-1.0) along the axis
    pub normalized_position: f32,
    /// The bounds of the axis in screen coordinates
    pub axis_bounds: &'a Rectangle,
    /// The domain (min, max) of the scale
    pub scale_domain: (D, D),
    /// Whether the cursor is within the chart bounds
    pub cursor_on_plot: bool,
    /// Whether the cursor is within the axis bounds
    pub cursor_on_axis: bool,
    /// The theme of the application
    pub theme: &'a Theme,
    /// The resolved style for the marker.
    pub(super) style: &'a MarkerStyle,
}

impl<D, Theme> MarkerContext<'_, D, Theme> {
    /// Creates a new [`Marker`] with applied styling.
    pub fn marker(&self, content: String) -> Marker {
        Marker {
            line: MarkerLine::from(self.style.line),
            label: super::Label::from_style(content, self.style.label),
            badge: MarkerBadge::from(self.style.badge),
        }
    }
}

/// A marker displayed on an axis, typically showing the current cursor position.
///
/// Combines a line extending into the plot area, a label showing the value,
/// and a badge background for the label.
pub struct Marker {
    /// The line extending from the axis into the plot.
    pub line: MarkerLine,
    /// The label displaying the marker value.
    pub label: super::Label,
    /// The badge background behind the label.
    pub badge: MarkerBadge,
}

/// Visual properties for the line extending from the marker badge into the plot.
pub struct MarkerLine {
    /// The color of the line
    pub color: Color,
    /// The width of the line
    pub width: Pixels,
    /// The gap between the markerline and the badge
    pub gap: Pixels,
}

impl From<MarkerLineStyle> for MarkerLine {
    fn from(value: MarkerLineStyle) -> Self {
        Self {
            color: value.color,
            width: value.width,
            gap: value.gap,
        }
    }
}

/// Visual properties for the badge background behind the marker label.
pub struct MarkerBadge {
    /// The background color of the badge.
    pub background: Color,
    /// The border styling of the badge.
    pub border: Border,
    /// The shadow styling of the badge.
    pub shadow: Shadow,
}

impl From<BadgeStyle> for MarkerBadge {
    fn from(value: BadgeStyle) -> Self {
        Self {
            background: value.background,
            border: value.border,
            shadow: value.shadow,
        }
    }
}
