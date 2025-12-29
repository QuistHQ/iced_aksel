use iced_core::{Border, Color, Font, Padding, Pixels, Shadow, Theme};
use iced_core::text::LineHeight;
use iced_core::widget::text::Shaping;

/// Global style of a `Chart`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Style of the crosshair cursor on the plot area.
    pub plot_cursor: PlotCursorStyle,
    /// Style of the axes.
    pub axis: AxisStyle,
    /// Style of the grid lines.
    pub grid: GridStyle,
}

/// Style of the grid lines.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridStyle {
    pub color: Color,
    pub width: Pixels,
}

/// Style of a `Chart`s axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisStyle {
    /// Distance from the Axis Line to the text baseline (The Rail).
    pub text_offset: Pixels,
    /// Style of the text labels.
    pub label: TextStyle,
    /// Style of the ticks (lines).
    pub ticks: TickStyle,
    /// Style of the cursor badge and line on the axis.
    pub cursor: AxisCursorStyle,
}

/// Style of axis ticks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TickStyle {
    pub color: Color,
    pub width: Pixels,
}

/// Style of a `Chart`s axis cursor. 
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisCursorStyle {
    /// Color of the cursor line.
    pub color: Color,
    /// Width of the cursor line.
    pub width: Pixels,
    /// Distance between the end of the cursor line and the start of the badge.
    pub line_gap: Pixels,

    /// Style of the text inside the badge.
    pub text: TextStyle,
    /// Style of the badge background and container.
    pub badge: AxisCursorBadgeStyle,
}

/// Style of the badge container for the axis cursor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisCursorBadgeStyle {
    pub padding: Padding,
    pub background: Color,
    pub border: Border,
    pub shadow: Shadow,
}

/// Style of the plot cursor that follows the mouse position over the chart.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlotCursorStyle {
    /// The color of the cursor crosshair.
    pub color: Color,
    /// The thickness of the cursor lines.
    pub width: Pixels,
}

/// General text styling.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextStyle {
    pub size: Pixels,
    pub font: Font,
    pub color: Color,
    pub line_height: LineHeight,
    pub shaping: Shaping,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            size: Pixels(12.0),
            font: Font::default(),
            color: Color::BLACK,
            line_height: LineHeight::Relative(1.2),
            shaping: Shaping::Basic,
        }
    }
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

pub fn default(theme: &Theme) -> Style {
    let palette = theme.extended_palette();

    Style {
        plot_cursor: PlotCursorStyle {
            color: palette.background.strong.color,
            width: 1.0.into(),
        },
        grid: GridStyle {
            color: palette.background.strong.color,
            width: 1.0.into(),
        },
        axis: AxisStyle {
            text_offset: 12.0.into(),
            label: TextStyle {
                color: palette.background.strong.text,
                ..Default::default()
            },
            ticks: TickStyle {
                color: palette.background.strong.color,
                width: 1.0.into(),
            },
            cursor: AxisCursorStyle {
                color: palette.primary.base.color,
                width: 1.0.into(),
                line_gap: 4.0.into(),
                text: TextStyle {
                    color: palette.primary.strong.text,
                    ..Default::default()
                },
                badge: AxisCursorBadgeStyle {
                    padding: Padding::new(4.0),
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