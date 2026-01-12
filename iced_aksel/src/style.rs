use iced_core::text::LineHeight;
use iced_core::{Border, Color, Padding, Pixels, Shadow, Theme};

/// Global style of a `Chart`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Style of the axes.
    pub axis: AxisStyle,
}

/// Style of dashed lines
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DashStyle {
    /// Length of the gap between dashes
    pub gap_length: f32,
    /// Length of the dashes
    pub dash_length: f32,
}

/// Style of lines.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridLineStyle {
    /// The color of the grid lines.
    pub color: Color,
    /// The thickness of the grid lines in pixels.
    pub width: Pixels,
    /// Whether the gridline should have a dashed pattern.
    pub dashed: Option<DashStyle>,
}

/// Style of tick lines on an axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TickLineStyle {
    /// The color of the tick-lines.
    pub color: Color,
    /// The thickness of the tick-lines in pixels.
    pub width: Pixels,
}

/// Style of a `Chart`'s axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisStyle {
    /// Distance from the Axis Line to the text baseline (The "Rail").
    pub text_offset: Pixels,
    /// Style of the text labels.
    pub label: LabelStyle,
    /// Style of the ticks (lines).
    pub tick: TickLineStyle,
    /// Style of the marker badge and line on the axis.
    pub marker: MarkerStyle,
    /// Style of the grid lines.
    pub grid: GridLineStyle,
}

/// Style of a `Chart`'s interactive axis marker.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MarkerStyle {
    pub line: MarkerLineStyle,
    /// Style of the label inside the badge.
    pub label: LabelStyle,
    /// Style of the badge container (background, border, shadow).
    pub badge: BadgeStyle,
}

/// Style of the marker line above the badge
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MarkerLineStyle {
    /// Color of the marker line.
    pub color: Color,
    /// Width of the marker line.
    pub width: Pixels,
    /// Distance between the end of the marker line and the start of the badge.
    pub gap: Pixels,
}

/// Style of the badge container for the axis marker.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BadgeStyle {
    /// Background color of the badge.
    pub background: Color,
    /// Border style of the badge.
    pub border: Border,
    /// Shadow style of the badge.
    pub shadow: Shadow,
}

/// General text styling configuration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LabelStyle {
    /// The font size in pixels.
    pub size: Pixels,
    /// The text color.
    pub color: Color,
    /// The line height.
    pub line_height: LineHeight,
    /// Padding around the label
    pub padding: Padding,
}

/// A trait for theming the appearance of a [`Chart`](crate::Chart).
pub trait Catalog {
    type Class<'a>;
    fn default<'a>() -> <Self as Catalog>::Class<'a>;
    fn style(&self, class: &<Self as Catalog>::Class<'_>) -> Style;
}

pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> StyleFn<'a, Self> {
        Box::new(default)
    }

    fn style(&self, class: &StyleFn<'_, Self>) -> Style {
        class(self)
    }
}

/// The default style function for a chart.
pub fn default(theme: &Theme) -> Style {
    let palette = theme.extended_palette();

    Style {
        axis: AxisStyle {
            text_offset: 12.0.into(),
            label: LabelStyle {
                color: palette.background.strong.text,
                size: Pixels(12.0),
                line_height: LineHeight::Relative(1.2),
                padding: Padding::new(2.0),
            },
            grid: GridLineStyle {
                color: palette.background.strong.color,
                width: 1.0.into(),
                dashed: None,
            },
            tick: TickLineStyle {
                color: palette.background.strong.text,
                width: 1.0.into(),
            },
            marker: MarkerStyle {
                line: MarkerLineStyle {
                    color: palette.primary.base.color,
                    width: 1.0.into(),
                    gap: 4.0.into(),
                },
                label: LabelStyle {
                    color: palette.primary.base.text,
                    size: Pixels(12.0),
                    line_height: LineHeight::Relative(1.2),
                    padding: Padding::new(2.0),
                },
                badge: BadgeStyle {
                    background: palette.primary.base.color,
                    border: Border {
                        radius: 4.0.into(),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    shadow: Shadow::default(),
                },
            },
        },
    }
}
