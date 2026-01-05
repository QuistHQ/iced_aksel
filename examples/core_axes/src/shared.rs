//! Shared code for the core_axes example and tests
//!
//! This module contains all the data structures, axis configurations,
//! and styling functions that are reused between the interactive example
//! and the snapshot tests.

use iced::{Font, Theme};
use iced_aksel::{
    Axis, Measure, PlotPoint, State, Stroke,
    axis::{self, GridLine, TickLine, TickResult},
    plot::{Plot, PlotData},
    scale::Linear,
    shape::Polyline,
    style::Style,
};

// -----------------------------------------------------------------------------
// Constants
// -----------------------------------------------------------------------------

pub const X: &str = "x";
pub const Y: &str = "y";

// -----------------------------------------------------------------------------
// Data Generation
// -----------------------------------------------------------------------------

pub struct SineWave {
    points: Vec<PlotPoint<f64>>,
}

impl SineWave {
    pub fn new(frequency: f64, amplitude: f64, count: usize) -> Self {
        let points = (0..=count)
            .map(|i| {
                let x = (i as f64 / count as f64) * 100.0;
                let y = (x * 0.1 * frequency).sin() * amplitude;
                PlotPoint::new(x, y)
            })
            .collect();

        Self { points }
    }
}

impl PlotData<f64> for SineWave {
    fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {
        let palette = theme.extended_palette();

        plot.add_shape(Polyline::new(self.points.clone()).stroke(Stroke::new(
            palette.primary.base.color,
            Measure::Screen(2.0),
        )));
    }
}

// -----------------------------------------------------------------------------
// Axis Configuration Helpers
// -----------------------------------------------------------------------------

pub fn setup_minimal_axes() -> State<&'static str, f64> {
    let mut state = State::new();

    // X-Axis: Standard look, no grid
    state.set_axis(
        X,
        Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom)
            .with_thickness(45.0)
            .without_grid()
            .with_cursor_formatter(|v| Some(format!("{:.0}", v))),
    );

    // Y-Axis: Invisible but active for scaling
    state.set_axis(
        Y,
        Axis::new(Linear::new(-1.2, 1.2), axis::Position::Left).invisible(),
    );

    state
}

pub fn setup_engineering_axes() -> State<&'static str, f64> {
    let mut state = State::new();

    // Define a custom renderer for a technical "ruler" look
    let ruler_renderer = |ctx: axis::TickContext<f64>| {
        if ctx.tick.level == 0 {
            TickResult {
                tick_line: Some(TickLine {
                    length: 8.0.into(),
                    thickness: 1.5.into(),
                }),
                grid_line: Some(GridLine {
                    thickness: 1.0.into(),
                }),
                label: Some(format!("{:.1}", ctx.tick.value)),
                ..Default::default()
            }
        } else {
            TickResult {
                tick_line: Some(TickLine {
                    length: 4.0.into(),
                    thickness: 1.0.into(),
                }),
                grid_line: Some(GridLine {
                    thickness: 0.5.into(),
                }),
                label: None,
                ..Default::default()
            }
        }
    };

    state.set_axis(
        Y,
        Axis::new(Linear::new(-4.0, 4.0), axis::Position::Left)
            .with_thickness(50.0)
            .with_tick_renderer(ruler_renderer),
    );

    state.set_axis(
        X,
        Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom)
            .with_thickness(35.0)
            .with_tick_renderer(ruler_renderer),
    );

    state
}

pub fn setup_custom_axes() -> State<&'static str, f64> {
    let mut state = State::new();

    // X-Axis on TOP
    state.set_axis(
        X,
        Axis::new(Linear::new(0.0, 100.0), axis::Position::Top)
            .with_thickness(45.0)
            .with_cursor_formatter(|v| Some(format!("T: {:.1}s", v))),
    );

    // Y-Axis on RIGHT
    state.set_axis(
        Y,
        Axis::new(Linear::new(-1.5, 1.5), axis::Position::Right)
            .with_thickness(55.0)
            .with_tick_renderer(|ctx| {
                // Show grid only for major ticks
                let is_major = ctx.tick.level == 0;
                TickResult {
                    grid_line: Some(GridLine {
                        thickness: if is_major { 1.0.into() } else { 0.0.into() },
                    }),
                    tick_line: Some(TickLine {
                        thickness: 1.0.into(),
                        length: 4.0.into(),
                    }),
                    label: if is_major {
                        Some(format!("{:.1}", ctx.tick.value))
                    } else {
                        None
                    },
                    ..Default::default()
                }
            })
            .with_cursor_formatter(|v| Some(format!("{:.3}", v))),
    );

    state
}

// -----------------------------------------------------------------------------
// Style Helpers
// -----------------------------------------------------------------------------

pub fn style_base(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    let mut style = iced_aksel::style::default(theme);

    style.axis.label.color = palette.background.strong.text;
    style.axis.ticks.color = palette.background.strong.text;

    style.axis.cursor.color = palette.primary.base.color;
    style.axis.cursor.badge.background = palette.primary.base.color;
    style.axis.cursor.text.color = palette.primary.base.text;

    style.plot_cursor.color = palette.primary.base.color;

    style
}

pub fn style_engineering(theme: &Theme) -> Style {
    let mut style = style_base(theme);

    style.axis.label.font = Font::MONOSPACE;
    style.axis.cursor.text.font = Font::MONOSPACE;

    style
}
