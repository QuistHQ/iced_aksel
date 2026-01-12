//! Snapshot tests for interactive chart features
//!
//! These tests verify that interactive charts with hover, zoom, and pan capabilities
//! render correctly with various signal data and configurations.

use iced::Theme;
use iced_aksel::{
    Axis, Chart, PlotPoint, State, Stroke,
    axis::{self, TickLine, TickResult},
    plot::{Plot, PlotData},
    scale::Linear,
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
// Test Data - Signal Generators
// -----------------------------------------------------------------------------

/// Multi-frequency signal similar to the core_interactions example
struct ComplexSignal {
    points: Vec<PlotPoint<f64>>,
}

impl ComplexSignal {
    fn new(x_min: f64, x_max: f64, sample_count: usize) -> Self {
        let points = (0..=sample_count)
            .map(|i| {
                let x = x_min + (i as f64 / sample_count as f64) * (x_max - x_min);
                // Multiple frequencies combined: sin(50x) * 0.5 + sin(3x) * 5 + sin(12x) * 2
                let y = (x * 50.0)
                    .sin()
                    .mul_add(0.5, (x * 3.0).sin().mul_add(5.0, (x * 12.0).sin() * 2.0));
                PlotPoint::new(x, y)
            })
            .collect();

        Self { points }
    }
}

impl PlotData<f64> for ComplexSignal {
    fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {
        let palette = theme.extended_palette();
        plot.add_shape(Polyline::new(self.points.clone()).stroke(Stroke::new(
            palette.primary.base.color,
            iced_aksel::Measure::Screen(1.5),
        )));
    }
}

/// Simple sine wave for basic testing
struct SineWave {
    points: Vec<PlotPoint<f64>>,
}

impl SineWave {
    fn new(frequency: f64, amplitude: f64, x_min: f64, x_max: f64, sample_count: usize) -> Self {
        let points = (0..=sample_count)
            .map(|i| {
                let x = x_min + (i as f64 / sample_count as f64) * (x_max - x_min);
                let y = (x * frequency).sin() * amplitude;
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
            palette.secondary.base.color,
            iced_aksel::Measure::Screen(2.0),
        )));
    }
}

/// Noisy signal for testing dense data
struct NoisySignal {
    points: Vec<PlotPoint<f64>>,
}

impl NoisySignal {
    fn new(x_min: f64, x_max: f64, sample_count: usize) -> Self {
        let points = (0..=sample_count)
            .map(|i| {
                let x = x_min + (i as f64 / sample_count as f64) * (x_max - x_min);
                // Pseudo-random noise based on sin combinations
                let noise = (x * 123.456).sin() * 0.5 + (x * 789.012).sin() * 0.3;
                let signal = (x * 2.0).sin() * 5.0;
                let y = signal + noise;
                PlotPoint::new(x, y)
            })
            .collect();

        Self { points }
    }
}

impl PlotData<f64> for NoisySignal {
    fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {
        let palette = theme.extended_palette();
        plot.add_shape(Polyline::new(self.points.clone()).stroke(Stroke::new(
            palette.success.base.color,
            iced_aksel::Measure::Screen(1.0),
        )));
    }
}

// -----------------------------------------------------------------------------
// Axis IDs
// -----------------------------------------------------------------------------

const X: &str = "time";
const Y: &str = "amplitude";

// -----------------------------------------------------------------------------
// Chart Configuration Helpers
// -----------------------------------------------------------------------------

fn setup_signal_analyzer_chart() -> State<&'static str, f64> {
    let mut state = State::new();

    // X Axis: Time (showing only major ticks with labels)
    let axis_x =
        Axis::new(Linear::new(0.0, 10.0), axis::Position::Bottom).with_tick_renderer(|ctx| {
            if ctx.tick.level == 0 {
                return TickResult {
                    label: Some(format!("{:.1}s", ctx.tick.value)),
                    ..Default::default()
                };
            }
            TickResult::new()
        });

    // Y Axis: Amplitude (major ticks with labels, minor ticks without)
    let axis_y =
        Axis::new(Linear::new(-10.0, 10.0), axis::Position::Left).with_tick_renderer(|ctx| {
            if ctx.tick.level == 0 {
                return TickResult {
                    tick_line: Some(TickLine::default()),
                    label: Some(format!("{:.1}V", ctx.tick.value)),
                    ..Default::default()
                };
            }
            TickResult {
                tick_line: Some(TickLine {
                    thickness: 0.5.into(),
                    length: 2.5.into(),
                }),
                label: None,
                ..Default::default()
            }
        });

    state.set_axis(X, axis_x);
    state.set_axis(Y, axis_y);

    state
}

fn setup_zoomed_chart() -> State<&'static str, f64> {
    let mut state = State::new();

    // Zoomed in view: 2.0 to 4.0 on X, -5.0 to 5.0 on Y
    state.set_axis(
        X,
        Axis::new(Linear::new(2.0, 4.0), axis::Position::Bottom).with_tick_renderer(|ctx| {
            if ctx.tick.level == 0 {
                TickResult::default().label(format!("{:.2}s", ctx.tick.value))
            } else {
                TickResult::new()
            }
        }),
    );

    state.set_axis(
        Y,
        Axis::new(Linear::new(-5.0, 5.0), axis::Position::Left).with_tick_renderer(|ctx| {
            if ctx.tick.level == 0 {
                TickResult::default().label(format!("{:.1}V", ctx.tick.value))
            } else {
                TickResult {
                    tick_line: Some(TickLine {
                        thickness: 0.5.into(),
                        length: 2.5.into(),
                    }),
                    label: None,
                    ..Default::default()
                }
            }
        }),
    );

    state
}

fn setup_panned_chart() -> State<&'static str, f64> {
    let mut state = State::new();

    // Panned view: offset X range
    state.set_axis(
        X,
        Axis::new(Linear::new(5.0, 15.0), axis::Position::Bottom).with_tick_renderer(|ctx| {
            if ctx.tick.level == 0 {
                TickResult::default().label(format!("{:.1}s", ctx.tick.value))
            } else {
                TickResult::new()
            }
        }),
    );

    state.set_axis(
        Y,
        Axis::new(Linear::new(-10.0, 10.0), axis::Position::Left).with_tick_renderer(|ctx| {
            if ctx.tick.level == 0 {
                TickResult::default().label(format!("{:.1}V", ctx.tick.value))
            } else {
                TickResult {
                    tick_line: Some(TickLine {
                        thickness: 0.5.into(),
                        length: 2.5.into(),
                    }),
                    label: None,
                    ..Default::default()
                }
            }
        }),
    );

    state
}

fn setup_high_resolution_chart() -> State<&'static str, f64> {
    let mut state = State::new();

    // High resolution view with more detailed ticks
    state.set_axis(
        X,
        Axis::new(Linear::new(0.0, 2.0), axis::Position::Bottom)
            .with_thickness(50.0)
            .with_tick_renderer(|ctx| {
                if ctx.tick.level == 0 {
                    TickResult::default().label(format!("{:.3}s", ctx.tick.value))
                } else {
                    TickResult {
                        tick_line: Some(TickLine {
                            thickness: 0.5.into(),
                            length: 3.0.into(),
                        }),
                        label: None,
                        ..Default::default()
                    }
                }
            }),
    );

    state.set_axis(
        Y,
        Axis::new(Linear::new(-2.0, 2.0), axis::Position::Left)
            .with_thickness(60.0)
            .with_tick_renderer(|ctx| {
                if ctx.tick.level == 0 {
                    TickResult::default().label(format!("{:.2}V", ctx.tick.value))
                } else {
                    TickResult {
                        tick_line: Some(TickLine {
                            thickness: 0.5.into(),
                            length: 2.5.into(),
                        }),
                        label: None,
                        ..Default::default()
                    }
                }
            }),
    );

    state
}

// -----------------------------------------------------------------------------
// Style Helpers
// -----------------------------------------------------------------------------

fn style_dark(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    let mut style = iced_aksel::style::default(theme);

    style.axis.label.color = palette.background.base.text;
    style.axis.ticks.color = palette.background.base.text;

    style.axis.cursor.color = palette.primary.base.color;
    style.axis.cursor.badge.background = palette.primary.base.color;
    style.axis.cursor.text.color = palette.primary.base.text;

    style.plot_cursor.color = palette.primary.base.color;

    style
}

// -----------------------------------------------------------------------------
// Snapshot Tests
// -----------------------------------------------------------------------------

#[test]
fn core_interactions_initial_view() {
    let state: &'static _ = Box::leak(Box::new(setup_signal_analyzer_chart()));
    let data: &'static _ = Box::leak(Box::new(ComplexSignal::new(0.0, 20.0, 2000)));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(data, X, Y)
            .style(Box::new(style_dark))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(&mut ui, "tests/snapshots/core_interactions_initial_view")
        .expect("Snapshot comparison failed for initial interactions view");
}

#[test]
fn core_interactions_zoomed_view() {
    let state: &'static _ = Box::leak(Box::new(setup_zoomed_chart()));
    let data: &'static _ = Box::leak(Box::new(ComplexSignal::new(0.0, 20.0, 2000)));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(data, X, Y)
            .style(Box::new(style_dark))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(&mut ui, "tests/snapshots/core_interactions_zoomed_view")
        .expect("Snapshot comparison failed for zoomed view");
}

#[test]
fn core_interactions_panned_view() {
    let state: &'static _ = Box::leak(Box::new(setup_panned_chart()));
    let data: &'static _ = Box::leak(Box::new(ComplexSignal::new(0.0, 20.0, 2000)));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(data, X, Y)
            .style(Box::new(style_dark))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(&mut ui, "tests/snapshots/core_interactions_panned_view")
        .expect("Snapshot comparison failed for panned view");
}

#[test]
fn core_interactions_high_resolution() {
    let state: &'static _ = Box::leak(Box::new(setup_high_resolution_chart()));
    let data: &'static _ = Box::leak(Box::new(ComplexSignal::new(0.0, 20.0, 2000)));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(data, X, Y)
            .style(Box::new(style_dark))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(&mut ui, "tests/snapshots/core_interactions_high_resolution")
        .expect("Snapshot comparison failed for high resolution view");
}

#[test]
fn core_interactions_simple_sine() {
    let state: &'static _ = Box::leak(Box::new(setup_signal_analyzer_chart()));
    let data: &'static _ = Box::leak(Box::new(SineWave::new(2.0, 8.0, 0.0, 10.0, 200)));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(data, X, Y)
            .style(Box::new(style_dark))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(&mut ui, "tests/snapshots/core_interactions_simple_sine")
        .expect("Snapshot comparison failed for simple sine wave");
}

#[test]
fn core_interactions_noisy_signal() {
    let state: &'static _ = Box::leak(Box::new(setup_signal_analyzer_chart()));
    let data: &'static _ = Box::leak(Box::new(NoisySignal::new(0.0, 10.0, 1000)));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(data, X, Y)
            .style(Box::new(style_dark))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(&mut ui, "tests/snapshots/core_interactions_noisy_signal")
        .expect("Snapshot comparison failed for noisy signal");
}

#[test]
fn core_interactions_dense_sampling() {
    let state: &'static _ = Box::leak(Box::new(setup_signal_analyzer_chart()));
    // Very high sample rate
    let data: &'static _ = Box::leak(Box::new(SineWave::new(10.0, 5.0, 0.0, 10.0, 5000)));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(data, X, Y)
            .style(Box::new(style_dark))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(&mut ui, "tests/snapshots/core_interactions_dense_sampling")
        .expect("Snapshot comparison failed for dense sampling");
}

#[test]
fn core_interactions_sparse_sampling() {
    let state: &'static _ = Box::leak(Box::new(setup_signal_analyzer_chart()));
    // Very low sample rate
    let data: &'static _ = Box::leak(Box::new(SineWave::new(2.0, 8.0, 0.0, 10.0, 20)));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(data, X, Y)
            .style(Box::new(style_dark))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(&mut ui, "tests/snapshots/core_interactions_sparse_sampling")
        .expect("Snapshot comparison failed for sparse sampling");
}

// -----------------------------------------------------------------------------
// Tick Configuration Tests
// -----------------------------------------------------------------------------

#[test]
fn core_interactions_major_ticks_only() {
    let mut state = State::new();

    // Only show major ticks (level 0), no minor ticks at all
    state.set_axis(
        X,
        Axis::new(Linear::new(0.0, 10.0), axis::Position::Bottom).with_tick_renderer(|ctx| {
            if ctx.tick.level == 0 {
                TickResult::default().label(format!("{:.0}s", ctx.tick.value))
            } else {
                TickResult::new() // Hide everything else
            }
        }),
    );

    state.set_axis(
        Y,
        Axis::new(Linear::new(-10.0, 10.0), axis::Position::Left).with_tick_renderer(|ctx| {
            if ctx.tick.level == 0 {
                TickResult::default().label(format!("{:.0}V", ctx.tick.value))
            } else {
                TickResult::new()
            }
        }),
    );

    let state: &'static _ = Box::leak(Box::new(state));
    let data: &'static _ = Box::leak(Box::new(SineWave::new(3.0, 7.0, 0.0, 10.0, 200)));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(data, X, Y)
            .style(Box::new(style_dark))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(
        &mut ui,
        "tests/snapshots/core_interactions_major_ticks_only",
    )
    .expect("Snapshot comparison failed for major ticks only");
}

#[test]
fn core_interactions_all_ticks_with_labels() {
    let mut state = State::new();

    // Show all tick levels with labels
    state.set_axis(
        X,
        Axis::new(Linear::new(0.0, 10.0), axis::Position::Bottom).with_tick_renderer(|ctx| {
            TickResult::default().label(format!("{:.2}", ctx.tick.value))
        }),
    );

    state.set_axis(
        Y,
        Axis::new(Linear::new(-10.0, 10.0), axis::Position::Left).with_tick_renderer(|ctx| {
            TickResult::default().label(format!("{:.2}", ctx.tick.value))
        }),
    );

    let state: &'static _ = Box::leak(Box::new(state));
    let data: &'static _ = Box::leak(Box::new(SineWave::new(3.0, 7.0, 0.0, 10.0, 200)));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(data, X, Y)
            .style(Box::new(style_dark))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(
        &mut ui,
        "tests/snapshots/core_interactions_all_ticks_with_labels",
    )
    .expect("Snapshot comparison failed for all ticks with labels");
}

#[test]
fn core_interactions_custom_tick_lengths() {
    let mut state = State::new();

    // Graduated tick lengths based on level
    state.set_axis(
        X,
        Axis::new(Linear::new(0.0, 10.0), axis::Position::Bottom).with_tick_renderer(|ctx| {
            let (length, thickness, label) = match ctx.tick.level {
                0 => (12.0, 2.0, Some(format!("{:.1}s", ctx.tick.value))),
                1 => (6.0, 1.5, None),
                _ => (3.0, 1.0, None),
            };

            TickResult {
                tick_line: Some(TickLine {
                    length: length.into(),
                    thickness: thickness.into(),
                }),
                label,
                ..Default::default()
            }
        }),
    );

    state.set_axis(
        Y,
        Axis::new(Linear::new(-10.0, 10.0), axis::Position::Left).with_tick_renderer(|ctx| {
            let (length, thickness, label) = match ctx.tick.level {
                0 => (12.0, 2.0, Some(format!("{:.1}V", ctx.tick.value))),
                1 => (6.0, 1.5, None),
                _ => (3.0, 1.0, None),
            };

            TickResult {
                tick_line: Some(TickLine {
                    length: length.into(),
                    thickness: thickness.into(),
                }),
                label,
                ..Default::default()
            }
        }),
    );

    let state: &'static _ = Box::leak(Box::new(state));
    let data: &'static _ = Box::leak(Box::new(SineWave::new(3.0, 7.0, 0.0, 10.0, 200)));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(data, X, Y)
            .style(Box::new(style_dark))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(
        &mut ui,
        "tests/snapshots/core_interactions_custom_tick_lengths",
    )
    .expect("Snapshot comparison failed for custom tick lengths");
}
