use iced_core::{Border, Color, Pixels, Rectangle, Shadow};

use crate::style::{BadgeStyle, MarkerLineStyle, MarkerStyle};

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
