mod axis;

use crate::style::axis::AxisStyle;
use iced_core::text::LineHeight;
use iced_core::widget::text::Shaping;
use iced_core::{Background, Border, Color, Font, Pixels, Shadow, Theme};

pub mod grid;

use self::axis::AxisLineStyle;

/// Global style of a `Chart`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Style of the axes.
    pub axis: AxisStyle,
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
        },
    }
}
