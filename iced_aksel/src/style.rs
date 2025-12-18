use iced_core::{Border, Color, Pixels, Shadow, Theme};

/// Style of a `Chart`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub cursor: PlotCursor,
    pub axis: AxisStyle,
    pub grid_color: Color,
}

/// Style of a `Chart`s axis. This defines how the axis is rendering with the Chart
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisStyle {
    pub cursor: AxisCursor,
    pub tick_color: Color,
    pub label_color: Color,
}

/// Style of a `Chart`s axis cursor. This defines how the position of your mouse is rendering on the Axis
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisCursor {
    pub color: Color,
    pub base_width: Pixels,
    pub border: Border,
    pub shadow: Shadow,
}

/// Style of the plot cursor that follows the mouse position over the chart.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlotCursor {
    /// The color of the cursor crosshair.
    pub color: Color,
    /// The thickness of the cursor lines.
    pub width: Pixels,
}

/// A trait for theming the appearance of a [`Chart`](crate::Chart).
///
/// This trait allows custom themes to define how charts should be styled.
/// Iced's [`Theme`] type implements this trait by default.
///
/// # Example
///
/// ```rust
/// use iced_aksel::{Chart, Catalog, State, Axis, axis::Position, scale::Linear};
/// use iced::Theme;
///
/// let state: State<&str, f64> = State::new()
///     .with_axis("x", Axis::new(Linear::new(0.0, 100.0), Position::Bottom));
///
/// // Use default theme styling
/// let chart: Chart<&str, f64, ()> = Chart::new(&state);
/// ```
pub trait Catalog {
    /// The item class of this catalog.
    type Class<'a>;

    /// Returns the default styling class.
    fn default<'a>() -> <Self as Catalog>::Class<'a>;

    /// Produces the style of a class with the given status.
    fn style(&self, class: &<Self as Catalog>::Class<'_>) -> Style;
}

/// A styling function for a [`Chart`](crate::Chart).
///
/// This is the default class type used by [`Theme`].
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

/// The default chart style for Iced's built-in [`Theme`].
///
/// Uses colors from the theme's extended palette for cursors, grid lines, ticks, and labels.
pub fn default(theme: &Theme) -> Style {
    let palette = theme.extended_palette();

    Style {
        cursor: PlotCursor {
            color: palette.background.weaker.color,
            width: 1.0.into(),
        },
        axis: AxisStyle {
            cursor: AxisCursor {
                color: palette.primary.base.color,
                base_width: 1.0.into(),
                border: Border::default().rounded(5.0),
                shadow: Shadow::default(),
            },
            tick_color: palette.background.weak.color,
            label_color: palette.background.weak.text,
        },
        grid_color: palette.background.weak.color,
    }
}
