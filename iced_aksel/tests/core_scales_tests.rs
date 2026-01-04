//! Snapshot tests for scale types
//!
//! These tests verify that Linear and Logarithmic scales render correctly
//! with various data distributions and ranges.

use iced::{Font, Theme};
use iced_aksel::{
    Axis, Chart, Measure, PlotPoint, State, Stroke,
    axis::{self, GridLine, TickLine, TickResult},
    plot::{Plot, PlotData},
    scale::{Linear, Logarithmic},
    shape::Polyline,
    style::Style,
};

#[derive(Debug, Clone)]
pub enum Message {}

// Import the common module with macro support
#[path = "common/mod.rs"]
#[macro_use]
mod common;

// Generate test helpers for the Message type
test_helpers!(Message);

// -----------------------------------------------------------------------------
// Test Data
// -----------------------------------------------------------------------------

/// Simple linear function: y = mx + b
struct LinearData {
    points: Vec<PlotPoint<f64>>,
}

impl LinearData {
    fn new(slope: f64, intercept: f64, x_min: f64, x_max: f64, count: usize) -> Self {
        let points = (0..=count)
            .map(|i| {
                let x = x_min + (i as f64 / count as f64) * (x_max - x_min);
                let y = slope * x + intercept;
                PlotPoint::new(x, y)
            })
            .collect();

        Self { points }
    }
}

impl PlotData<f64> for LinearData {
    fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {
        let palette = theme.extended_palette();

        plot.add_shape(Polyline::new(self.points.clone()).stroke(Stroke::new(
            palette.primary.base.color,
            Measure::Screen(2.5),
        )));
    }
}

/// Exponential function: y = 10^x
/// This demonstrates why log scales are useful
struct ExponentialData {
    points: Vec<PlotPoint<f64>>,
}

impl ExponentialData {
    fn new(x_min: f64, x_max: f64, count: usize) -> Self {
        let points = (0..=count)
            .map(|i| {
                let x = x_min + (i as f64 / count as f64) * (x_max - x_min);
                let y = 10.0_f64.powf(x);
                PlotPoint::new(x, y)
            })
            .collect();

        Self { points }
    }
}

impl PlotData<f64> for ExponentialData {
    fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {
        let palette = theme.extended_palette();

        plot.add_shape(Polyline::new(self.points.clone()).stroke(Stroke::new(
            palette.secondary.base.color,
            Measure::Screen(2.5),
        )));
    }
}

/// Quadratic function: y = x^2
struct QuadraticData {
    points: Vec<PlotPoint<f64>>,
}

impl QuadraticData {
    fn new(x_min: f64, x_max: f64, count: usize) -> Self {
        let points = (0..=count)
            .map(|i| {
                let x = x_min + (i as f64 / count as f64) * (x_max - x_min);
                let y = x * x;
                PlotPoint::new(x, y)
            })
            .collect();

        Self { points }
    }
}

impl PlotData<f64> for QuadraticData {
    fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {
        let palette = theme.extended_palette();

        plot.add_shape(Polyline::new(self.points.clone()).stroke(Stroke::new(
            palette.success.base.color,
            Measure::Screen(2.5),
        )));
    }
}

// -----------------------------------------------------------------------------
// Axis IDs
// -----------------------------------------------------------------------------

const X: &str = "x";
const Y: &str = "y";

// -----------------------------------------------------------------------------
// Scale Configuration Helpers
// -----------------------------------------------------------------------------

fn setup_linear_linear_scales() -> State<&'static str, f64> {
    let mut state = State::new();

    state.set_axis(
        X,
        Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom)
            .with_thickness(45.0)
            .with_cursor_formatter(|v| Some(format!("{:.1}", v))),
    );

    state.set_axis(
        Y,
        Axis::new(Linear::new(0.0, 100.0), axis::Position::Left)
            .with_thickness(55.0)
            .with_cursor_formatter(|v| Some(format!("{:.1}", v))),
    );

    state
}

fn setup_linear_log_scales() -> State<&'static str, f64> {
    let mut state = State::new();

    // X-Axis: Linear
    state.set_axis(
        X,
        Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom)
            .with_thickness(45.0)
            .with_cursor_formatter(|v| Some(format!("{:.0}", v))),
    );

    // Y-Axis: Logarithmic (base 10)
    state.set_axis(
        Y,
        Axis::new(Logarithmic::new(10.0, 1.0, 1000.0), axis::Position::Left)
            .with_thickness(65.0)
            .with_cursor_formatter(|v| Some(format!("{:.1e}", v))),
    );

    state
}

fn setup_log_linear_scales() -> State<&'static str, f64> {
    let mut state = State::new();

    // X-Axis: Logarithmic (base 10)
    state.set_axis(
        X,
        Axis::new(Logarithmic::new(10.0, 1.0, 1000.0), axis::Position::Bottom)
            .with_thickness(50.0)
            .with_cursor_formatter(|v| {
                if v >= 100.0 {
                    Some(format!("{:.0}", v))
                } else if v >= 10.0 {
                    Some(format!("{:.1}", v))
                } else {
                    Some(format!("{:.2}", v))
                }
            }),
    );

    // Y-Axis: Linear
    state.set_axis(
        Y,
        Axis::new(Linear::new(0.0, 10.0), axis::Position::Left)
            .with_thickness(55.0)
            .with_cursor_formatter(|v| Some(format!("{:.1}", v))),
    );

    state
}

fn setup_log_log_scales() -> State<&'static str, f64> {
    let mut state = State::new();

    // X-Axis: Logarithmic (base 10)
    state.set_axis(
        X,
        Axis::new(Logarithmic::new(10.0, 1.0, 1000.0), axis::Position::Bottom)
            .with_thickness(50.0)
            .with_cursor_formatter(|v| Some(format!("{:.1e}", v))),
    );

    // Y-Axis: Logarithmic (base 2)
    state.set_axis(
        Y,
        Axis::new(Logarithmic::new(2.0, 1.0, 128.0), axis::Position::Left)
            .with_thickness(60.0)
            .with_cursor_formatter(|v| Some(format!("{:.1}", v))),
    );

    state
}

fn setup_negative_linear_scales() -> State<&'static str, f64> {
    let mut state = State::new();

    state.set_axis(
        X,
        Axis::new(Linear::new(-50.0, 50.0), axis::Position::Bottom)
            .with_thickness(45.0)
            .with_cursor_formatter(|v| Some(format!("{:.0}", v))),
    );

    state.set_axis(
        Y,
        Axis::new(Linear::new(-100.0, 100.0), axis::Position::Left)
            .with_thickness(55.0)
            .with_cursor_formatter(|v| Some(format!("{:.0}", v))),
    );

    state
}

// -----------------------------------------------------------------------------
// Style Helpers
// -----------------------------------------------------------------------------

fn style_base(theme: &Theme) -> Style {
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

fn style_scientific(theme: &Theme) -> Style {
    let mut style = style_base(theme);

    style.axis.label.font = Font::MONOSPACE;
    style.axis.cursor.text.font = Font::MONOSPACE;

    style
}

// -----------------------------------------------------------------------------
// Snapshot Tests
// -----------------------------------------------------------------------------

#[test]
fn core_scales_linear_linear_snapshot() {
    let state: &'static _ = Box::leak(Box::new(setup_linear_linear_scales()));
    // Simple linear relationship: y = 0.8x + 10
    let data: &'static _ = Box::leak(Box::new(LinearData::new(0.8, 10.0, 0.0, 100.0, 50)));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(data, X, Y)
            .style(Box::new(style_base))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(
        &mut ui,
        "tests/snapshots/core_scales_linear_linear_snapshot",
    )
    .expect("Snapshot comparison failed for linear-linear scales");
}

#[test]
fn core_scales_linear_log_snapshot() {
    let state: &'static _ = Box::leak(Box::new(setup_linear_log_scales()));
    // Exponential data: y = 10^(x/20) spans from 1 to 1000
    let data: &'static _ = Box::leak(Box::new(ExponentialData::new(0.0, 3.0, 100)));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(data, X, Y)
            .style(Box::new(style_scientific))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(&mut ui, "tests/snapshots/core_scales_linear_log_snapshot")
        .expect("Snapshot comparison failed for linear-log scales");
}

#[test]
fn core_scales_log_linear_snapshot() {
    let state: &'static _ = Box::leak(Box::new(setup_log_linear_scales()));

    // Logarithmic X data: y = log10(x)
    let points: Vec<PlotPoint<f64>> = (1..=100)
        .map(|i| {
            let x = 10.0_f64.powf((i as f64 / 100.0) * 3.0); // 1 to 1000
            let y = x.log10(); // 0 to 3
            PlotPoint::new(x, y)
        })
        .collect();

    struct LogXData {
        points: Vec<PlotPoint<f64>>,
    }

    impl PlotData<f64> for LogXData {
        fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {
            let palette = theme.extended_palette();
            plot.add_shape(Polyline::new(self.points.clone()).stroke(Stroke::new(
                palette.success.base.color,
                Measure::Screen(2.5),
            )));
        }
    }

    let data: &'static _ = Box::leak(Box::new(LogXData { points }));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(data, X, Y)
            .style(Box::new(style_base))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(&mut ui, "tests/snapshots/core_scales_log_linear_snapshot")
        .expect("Snapshot comparison failed for log-linear scales");
}

#[test]
fn core_scales_log_log_snapshot() {
    let state: &'static _ = Box::leak(Box::new(setup_log_log_scales()));

    // Create data that spans multiple orders of magnitude
    let points: Vec<PlotPoint<f64>> = (1..=100)
        .map(|i| {
            let x = 10.0_f64.powf((i as f64 / 100.0) * 3.0); // 1 to 1000
            let y = 2.0_f64.powf((i as f64 / 100.0) * 7.0); // 1 to 128
            PlotPoint::new(x, y)
        })
        .collect();

    struct LogLogData {
        points: Vec<PlotPoint<f64>>,
    }

    impl PlotData<f64> for LogLogData {
        fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {
            let palette = theme.extended_palette();
            plot.add_shape(
                Polyline::new(self.points.clone())
                    .stroke(Stroke::new(palette.danger.base.color, Measure::Screen(2.0))),
            );
        }
    }

    let data: &'static _ = Box::leak(Box::new(LogLogData { points }));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(data, X, Y)
            .style(Box::new(style_scientific))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(&mut ui, "tests/snapshots/core_scales_log_log_snapshot")
        .expect("Snapshot comparison failed for log-log scales");
}

#[test]
fn core_scales_negative_linear_snapshot() {
    let state: &'static _ = Box::leak(Box::new(setup_negative_linear_scales()));

    // Create data that crosses through zero
    let points: Vec<PlotPoint<f64>> = (0..=100)
        .map(|i| {
            let x = -50.0 + (i as f64);
            let y = 2.0 * x - 10.0;
            PlotPoint::new(x, y)
        })
        .collect();

    struct CrossingData {
        points: Vec<PlotPoint<f64>>,
    }

    impl PlotData<f64> for CrossingData {
        fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {
            let palette = theme.extended_palette();
            plot.add_shape(Polyline::new(self.points.clone()).stroke(Stroke::new(
                palette.primary.base.color,
                Measure::Screen(2.5),
            )));
        }
    }

    let data: &'static _ = Box::leak(Box::new(CrossingData { points }));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(data, X, Y)
            .style(Box::new(style_base))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(
        &mut ui,
        "tests/snapshots/core_scales_negative_linear_snapshot",
    )
    .expect("Snapshot comparison failed for negative linear scales");
}

// -----------------------------------------------------------------------------
// Additional Scale Feature Tests
// -----------------------------------------------------------------------------

#[test]
fn core_scales_wide_linear_range() {
    let mut state = State::new();

    state.set_axis(
        X,
        Axis::new(Linear::new(0.0, 10000.0), axis::Position::Bottom)
            .with_thickness(50.0)
            .with_cursor_formatter(|v| Some(format!("{:.0}", v))),
    );

    state.set_axis(
        Y,
        Axis::new(Linear::new(0.0, 1000.0), axis::Position::Left)
            .with_thickness(60.0)
            .with_cursor_formatter(|v| Some(format!("{:.0}", v))),
    );

    let state: &'static _ = Box::leak(Box::new(state));
    // Linear data: y = 0.1x spanning large range
    let data: &'static _ = Box::leak(Box::new(LinearData::new(0.1, 0.0, 0.0, 10000.0, 100)));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(data, X, Y)
            .style(Box::new(style_base))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(&mut ui, "tests/snapshots/core_scales_wide_linear_range")
        .expect("Snapshot comparison failed for wide linear range");
}

#[test]
fn core_scales_narrow_linear_range() {
    let mut state = State::new();

    state.set_axis(
        X,
        Axis::new(Linear::new(0.0, 1.0), axis::Position::Bottom)
            .with_thickness(50.0)
            .with_cursor_formatter(|v| Some(format!("{:.3}", v))),
    );

    state.set_axis(
        Y,
        Axis::new(Linear::new(0.0, 0.1), axis::Position::Left)
            .with_thickness(60.0)
            .with_cursor_formatter(|v| Some(format!("{:.4}", v))),
    );

    let state: &'static _ = Box::leak(Box::new(state));
    // Linear data with small values: y = 0.05x + 0.01
    let data: &'static _ = Box::leak(Box::new(LinearData::new(0.05, 0.01, 0.0, 1.0, 50)));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(data, X, Y)
            .style(Box::new(style_base))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(&mut ui, "tests/snapshots/core_scales_narrow_linear_range")
        .expect("Snapshot comparison failed for narrow linear range");
}

#[test]
fn core_scales_log_base_2() {
    let mut state = State::new();

    // Binary progression (powers of 2)
    state.set_axis(
        X,
        Axis::new(Logarithmic::new(2.0, 1.0, 256.0), axis::Position::Bottom)
            .with_thickness(50.0)
            .with_cursor_formatter(|v| Some(format!("{:.0}", v))),
    );

    state.set_axis(
        Y,
        Axis::new(Linear::new(0.0, 10.0), axis::Position::Left)
            .with_thickness(55.0)
            .with_cursor_formatter(|v| Some(format!("{:.1}", v))),
    );

    let state: &'static _ = Box::leak(Box::new(state));

    let points: Vec<PlotPoint<f64>> = (0..=100)
        .map(|i| {
            let x = 2.0_f64.powf((i as f64 / 100.0) * 8.0); // 1 to 256
            let y = (x.log2() / 8.0) * 10.0; // normalized
            PlotPoint::new(x, y)
        })
        .collect();

    struct Base2Data {
        points: Vec<PlotPoint<f64>>,
    }

    impl PlotData<f64> for Base2Data {
        fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {
            let palette = theme.extended_palette();
            plot.add_shape(Polyline::new(self.points.clone()).stroke(Stroke::new(
                palette.secondary.base.color,
                Measure::Screen(2.0),
            )));
        }
    }

    let data: &'static _ = Box::leak(Box::new(Base2Data { points }));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(data, X, Y)
            .style(Box::new(style_base))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(&mut ui, "tests/snapshots/core_scales_log_base_2")
        .expect("Snapshot comparison failed for logarithmic base-2 scale");
}

#[test]
fn core_scales_quadratic_on_linear() {
    let mut state = State::new();

    state.set_axis(
        X,
        Axis::new(Linear::new(0.0, 10.0), axis::Position::Bottom)
            .with_thickness(45.0)
            .with_cursor_formatter(|v| Some(format!("{:.1}", v))),
    );

    state.set_axis(
        Y,
        Axis::new(Linear::new(0.0, 100.0), axis::Position::Left)
            .with_thickness(55.0)
            .with_cursor_formatter(|v| Some(format!("{:.0}", v))),
    );

    let state: &'static _ = Box::leak(Box::new(state));
    // Quadratic data: y = x^2
    let data: &'static _ = Box::leak(Box::new(QuadraticData::new(0.0, 10.0, 100)));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(data, X, Y)
            .style(Box::new(style_base))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(&mut ui, "tests/snapshots/core_scales_quadratic_on_linear")
        .expect("Snapshot comparison failed for quadratic on linear scales");
}

#[test]
fn core_scales_simple_linear_progression() {
    let mut state = State::new();

    state.set_axis(
        X,
        Axis::new(Linear::new(0.0, 50.0), axis::Position::Bottom)
            .with_thickness(45.0)
            .with_cursor_formatter(|v| Some(format!("{:.0}", v))),
    );

    state.set_axis(
        Y,
        Axis::new(Linear::new(0.0, 50.0), axis::Position::Left)
            .with_thickness(55.0)
            .with_cursor_formatter(|v| Some(format!("{:.0}", v))),
    );

    let state: &'static _ = Box::leak(Box::new(state));
    // Perfect diagonal: y = x
    let data: &'static _ = Box::leak(Box::new(LinearData::new(1.0, 0.0, 0.0, 50.0, 50)));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(data, X, Y)
            .style(Box::new(style_base))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(
        &mut ui,
        "tests/snapshots/core_scales_simple_linear_progression",
    )
    .expect("Snapshot comparison failed for simple linear progression");
}

#[test]
fn core_scales_custom_tick_spacing() {
    let mut state = State::new();

    // Custom tick renderer for specific spacing
    let custom_renderer = |ctx: axis::TickContext<f64>| {
        if ctx.tick.level == 0 {
            TickResult {
                tick_line: Some(TickLine {
                    length: 10.0.into(),
                    thickness: 2.0.into(),
                }),
                grid_line: Some(GridLine {
                    thickness: 1.0.into(),
                }),
                label: Some(format!("{:.0}", ctx.tick.value)),
                ..Default::default()
            }
        } else {
            TickResult {
                tick_line: Some(TickLine {
                    length: 5.0.into(),
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
        X,
        Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom)
            .with_thickness(45.0)
            .with_tick_renderer(custom_renderer),
    );

    state.set_axis(
        Y,
        Axis::new(Linear::new(0.0, 100.0), axis::Position::Left)
            .with_thickness(55.0)
            .with_tick_renderer(custom_renderer),
    );

    let state: &'static _ = Box::leak(Box::new(state));
    // Linear data: y = 0.5x + 25
    let data: &'static _ = Box::leak(Box::new(LinearData::new(0.5, 25.0, 0.0, 100.0, 50)));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(data, X, Y)
            .style(Box::new(style_base))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(&mut ui, "tests/snapshots/core_scales_custom_tick_spacing")
        .expect("Snapshot comparison failed for custom tick spacing");
}
