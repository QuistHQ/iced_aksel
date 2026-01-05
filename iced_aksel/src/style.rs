use iced_core::text::{LineHeight, Shaping};
use iced_core::{Background, Border, Color, Font, Pixels, Shadow, Theme, Vector};

// -----------------------------------------------------------------------------
// 1. PRIMITIVES
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineStyle {
    pub color: Color,
    pub width: Pixels,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextStyle {
    pub color: Color,
    pub size: Pixels,
    pub font: Font,
    pub line_height: LineHeight,
    pub shaping: Shaping,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContainerStyle {
    pub background: Option<Background>,
    pub border: Border,
    pub shadow: Shadow,
    pub padding: iced_core::Padding,
}

impl Default for ContainerStyle {
    fn default() -> Self {
        Self {
            background: None,
            border: Border::default(),
            shadow: Shadow::default(),
            padding: 4.0.into(),
        }
    }
}

// -----------------------------------------------------------------------------
// 2. COMPOSITES
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridStyle {
    pub line: LineStyle,
    pub dashed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TickStyle {
    pub line_style: LineStyle,
    pub length: Pixels,
    pub text_style: TextStyle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CursorStyle {
    pub line: LineStyle,
    pub line_gap: Pixels,
    pub badge: ContainerStyle,
    pub text: TextStyle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisStyle {
    pub spine: LineStyle,
    pub ticks: TickStyle,
    pub grid: GridStyle,
}

// -----------------------------------------------------------------------------
// 3. CATALOG IMPLEMENTATION
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub axis: AxisStyle,
    pub cursor: CursorStyle,
}

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

    // Extract Theme Colors
    let text_color = palette.background.weak.text;
    let axis_color = palette.background.strong.color;
    let grid_color = palette.background.weakest.color;
    let primary_color = palette.primary.base.color;
    let on_primary_text = palette.primary.base.text;

    // Define "Smart Defaults" using Theme Colors
    let default_text = TextStyle {
        color: text_color,
        size: 12.0.into(),
        font: Font::default(),
        line_height: LineHeight::Relative(1.2),
        shaping: Shaping::Auto,
    };

    let default_spine = LineStyle {
        color: axis_color,
        width: 1.0.into(),
    };

    let default_grid = LineStyle {
        color: grid_color,
        width: 1.0.into(),
    };

    Style {
        axis: AxisStyle {
            spine: default_spine,
            ticks: TickStyle {
                line_style: default_spine,
                length: 2.0.into(),
                text_style: default_text,
            },
            grid: GridStyle {
                line: default_grid,
                dashed: false,
            },
        },
        cursor: CursorStyle {
            line: LineStyle {
                color: primary_color,
                width: 1.0.into(),
            },
            line_gap: 2.0.into(),
            badge: ContainerStyle {
                background: Some(primary_color.into()),
                border: Border {
                    radius: 4.0.into(),
                    ..Border::default()
                },
                shadow: Shadow::default(),
                padding: 4.0.into(),
            },
            text: TextStyle {
                color: on_primary_text,
                ..default_text
            },
        },
    }
}
