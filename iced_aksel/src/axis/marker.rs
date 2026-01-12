use iced_core::{Border, Color, Pixels, Shadow};

use crate::style::{BadgeStyle, MarkerLineStyle, MarkerStyle};

/// Context provided to marker renderers for creating styled markers.
///
/// Contains the axis value at the marker position and the resolved marker style.
pub struct MarkerContext<'a, D> {
    /// The axis value at the marker position.
    pub value: D,
    /// The resolved style for the marker.
    pub style: &'a MarkerStyle,
}

impl<D> MarkerContext<'_, D> {
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

pub struct MarkerLine {
    pub color: Color,
    pub width: Pixels,
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

pub struct MarkerBadge {
    pub background: Color,
    pub border: Border,
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
