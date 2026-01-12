//! Snapshot tests for the core_axes example
//!
//! These tests verify that the various axis configurations render correctly
//! by comparing against baseline snapshots.

use iced_aksel::{
    Axis, Chart, State,
    axis::{self, GridLine, TickLine, TickResult},
    scale::Linear,
};

#[derive(Debug, Clone)]
pub enum Message {}

// Import the common module with macro support
#[path = "common/mod.rs"]
#[macro_use]
mod common;

// Import shared code from the core_axes example
#[path = "../../examples/core_axes/src/shared.rs"]
mod shared;

use shared::*;

// Generate test helpers for the Message type
test_helpers!(Message);

// -----------------------------------------------------------------------------
// Snapshot Tests
// -----------------------------------------------------------------------------

#[test]
fn core_axes_minimal_axes_snapshot() {
    let state: &'static _ = Box::leak(Box::new(setup_minimal_axes()));
    let data: &'static _ = Box::leak(Box::new(SineWave::new(1.0, 0.8, 50)));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(data, X, Y)
            .style(Box::new(style_base))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(&mut ui, "tests/snapshots/core_axes_minimal_axes_snapshot")
        .expect("Snapshot comparison failed for minimal axes");
}

#[test]
fn core_axes_engineering_axes_snapshot() {
    let state: &'static _ = Box::leak(Box::new(setup_engineering_axes()));
    let data: &'static _ = Box::leak(Box::new(SineWave::new(2.5, 3.5, 100)));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(data, X, Y)
            .style(Box::new(style_engineering))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(
        &mut ui,
        "tests/snapshots/core_axes_engineering_axes_snapshot",
    )
    .expect("Snapshot comparison failed for engineering axes");
}

#[test]
fn core_axes_custom_placement_axes_snapshot() {
    let state: &'static _ = Box::leak(Box::new(setup_custom_axes()));
    let data: &'static _ = Box::leak(Box::new(SineWave::new(1.5, 0.8, 80)));

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
        "tests/snapshots/core_axes_custom_placement_axes_snapshot",
    )
    .expect("Snapshot comparison failed for custom placement axes");
}

// -----------------------------------------------------------------------------
// Additional Tests for Specific Axis Features
// -----------------------------------------------------------------------------

#[test]
fn core_axes_axis_without_grid() {
    let mut state = State::new();

    state.set_axis(
        X,
        Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom)
            .with_thickness(45.0)
            .without_grid(),
    );

    state.set_axis(
        Y,
        Axis::new(Linear::new(-1.0, 1.0), axis::Position::Left)
            .with_thickness(50.0)
            .without_grid(),
    );

    let state: &'static _ = Box::leak(Box::new(state));
    let data: &'static _ = Box::leak(Box::new(SineWave::new(1.0, 0.8, 50)));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(data, X, Y)
            .style(Box::new(style_base))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(&mut ui, "tests/snapshots/core_axes_axis_without_grid")
        .expect("Snapshot comparison failed for no grid axes");
}

#[test]
fn core_axes_invisible_axis() {
    let mut state = State::new();

    state.set_axis(
        X,
        Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom).with_thickness(45.0),
    );

    // Y-Axis is invisible but still provides scaling
    state.set_axis(
        Y,
        Axis::new(Linear::new(-1.0, 1.0), axis::Position::Left).invisible(),
    );

    let state: &'static _ = Box::leak(Box::new(state));
    let data: &'static _ = Box::leak(Box::new(SineWave::new(1.0, 0.8, 50)));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(data, X, Y)
            .style(Box::new(style_base))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(&mut ui, "tests/snapshots/core_axes_invisible_axis")
        .expect("Snapshot comparison failed for invisible Y axis");
}

#[test]
fn core_axes_top_and_right_axes() {
    let mut state = State::new();

    // X-Axis on top
    state.set_axis(
        X,
        Axis::new(Linear::new(0.0, 100.0), axis::Position::Top).with_thickness(45.0),
    );

    // Y-Axis on right
    state.set_axis(
        Y,
        Axis::new(Linear::new(-1.0, 1.0), axis::Position::Right).with_thickness(50.0),
    );

    let state: &'static _ = Box::leak(Box::new(state));
    let data: &'static _ = Box::leak(Box::new(SineWave::new(1.0, 0.8, 50)));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(data, X, Y)
            .style(Box::new(style_base))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(&mut ui, "tests/snapshots/core_axes_top_and_right_axes")
        .expect("Snapshot comparison failed for top and right axes");
}

#[test]
fn core_axes_custom_tick_renderer() {
    let mut state = State::new();

    // Custom renderer that only shows major ticks
    let major_only_renderer = |ctx: axis::TickContext<f64>| {
        if ctx.tick.level == 0 {
            TickResult {
                tick_line: Some(TickLine {
                    length: 10.0.into(),
                    thickness: 2.0.into(),
                }),
                grid_line: Some(GridLine {
                    thickness: 1.5.into(),
                }),
                label: Some(format!("{:.0}", ctx.tick.value)),
                ..Default::default()
            }
        } else {
            TickResult::default()
        }
    };

    state.set_axis(
        X,
        Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom)
            .with_thickness(45.0)
            .with_tick_renderer(major_only_renderer),
    );

    state.set_axis(
        Y,
        Axis::new(Linear::new(-1.0, 1.0), axis::Position::Left)
            .with_thickness(50.0)
            .with_tick_renderer(major_only_renderer),
    );

    let state: &'static _ = Box::leak(Box::new(state));
    let data: &'static _ = Box::leak(Box::new(SineWave::new(1.0, 0.8, 50)));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(data, X, Y)
            .style(Box::new(style_base))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(&mut ui, "tests/snapshots/core_axes_custom_tick_renderer")
        .expect("Snapshot comparison failed for custom tick renderer");
}
