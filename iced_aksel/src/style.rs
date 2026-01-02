mod axis;
mod cursor;
mod grid;

use crate::style::axis::AxisStyle;
use crate::style::cursor::{AxisCursorStyle, BadgeStyle, CursorStyle};
use crate::style::grid::GridStyle;
use iced_core::text::LineHeight;
use iced_core::widget::text::Shaping;
use iced_core::{Background, Border, Color, Font, Padding, Pixels, Shadow, Theme};

/// Global style of a `Chart`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Style of the crosshair cursor on the plot area.
    pub cursor: CursorStyle,
    /// Style of the axes.
    pub axis: AxisStyle,
    /// Style of the grid lines.
    pub grid: GridStyle,
}

/// Style of the axis container.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContainerStyle {
    /// The background of the axis.
    pub background: Option<Background>,
    /// The border of the axis.
    pub border: Option<Border>,
    /// The shadow of the axis.
    pub shadow: Option<Shadow>,
}

/// Style of axis ticks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineStyle {
    /// The color of the tick lines.
    pub color: Color,
    /// The thickness of the tick lines.
    pub width: Pixels,
}

/// General text styling configuration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextStyle {
    /// The font size in pixels.
    pub size: Pixels,
    /// The font family to use.
    pub font: Font,
    /// The text color.
    pub color: Color,
    /// The line height.
    pub line_height: LineHeight,
    /// The text shaping strategy.
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

/// The default style function for a chart.
pub fn default(theme: &Theme) -> Style {
    let palette = theme.extended_palette();

    Style {
        cursor: CursorStyle {
            axis: AxisCursorStyle {
                badge: BadgeStyle {
                    text: TextStyle {
                        color: theme.palette().text,
                        line_height: LineHeight::default(),
                        size: 12.0.into(),
                        shaping: Shaping::Auto,

                        // TODO: Where should we get font from?
                        font: Font::default(),
                    },
                    container: ContainerStyle {
                        border: Some(Border {
                            width: 1.0.into(),
                            color: theme.palette().text,
                            radius: 2.0.into(),
                        }),
                        background: Some(theme.palette().background.into()),
                        shadow: Default::default(),
                    },
                    padding: 4.0.into(),
                },
                line: LineStyle {
                    color: theme.palette().text,
                    width: 1.0.into(),
                },
                line_gap: 4.0.into(),
            },
        },
        grid: GridStyle {
            color: palette.background.strong.color,
            width: 1.0.into(),
            dashed: false,
        },
        axis: AxisStyle {
            container: ContainerStyle {
                background: Some(theme.palette().background.into()),
                border: None,
                shadow: None,
            },
            line: LineStyle {
                color: palette.background.strong.color,
                width: 1.0.into(),
            },
            text_offset: 12.0.into(),
            text: TextStyle {
                color: palette.background.strong.text,
                ..Default::default()
            },
            ticks: LineStyle {
                color: palette.background.strong.color,
                width: 1.0.into(),
            },
        },
    }
}
