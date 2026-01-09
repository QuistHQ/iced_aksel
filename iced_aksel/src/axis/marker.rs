use aksel::Tick;
use iced_core::{Border, Color, Pixels, Shadow};

use crate::style::{BadgeStyle, MarkerLineStyle, MarkerStyle};

pub struct MarkerContext<'a, D> {
    pub tick: Tick<D>,
    style: &'a MarkerStyle,
}

impl<D> MarkerContext<'_, D> {
    pub fn marker(&self, content: String) -> Marker {
        Marker {
            line: MarkerLine::from(self.style.line),
            label: super::Label::from_style(content, self.style.label),
            badge: MarkerBadge::from(self.style.badge),
        }
    }
}

pub struct Marker {
    pub line: MarkerLine,
    pub label: super::Label,
    pub badge: MarkerBadge,
}

pub struct MarkerLine {
    pub color: Color,
    pub width: Pixels,
    pub line_gap: Pixels,
}

impl From<MarkerLineStyle> for MarkerLine {
    fn from(value: MarkerLineStyle) -> Self {
        Self {
            color: value.color,
            width: value.width,
            line_gap: value.gap,
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
