//! Snapshot tests for all shape primitives
//!
//! These tests verify that all shape types (Rectangle, Ellipse, Arc, Triangle,
//! Polygon, Line, Polyline, Spline, Bezier, Area, Label) render correctly
//! with various styling options.

use iced::{
    Theme,
    alignment::{Horizontal, Vertical},
};
use iced_aksel::{
    Axis, Chart, Measure, PlotPoint, State, Stroke, axis,
    plot::{Plot, PlotData},
    scale::Linear,
    shape::{
        Arc, Area, Bezier, Ellipse, Label, Line, Polygon, Polyline, Rectangle, Spline, Triangle,
    },
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
// Axis IDs
// -----------------------------------------------------------------------------

const X: &str = "x";
const Y: &str = "y";

// -----------------------------------------------------------------------------
// Shape Test Helpers
// -----------------------------------------------------------------------------

/// Setup a simple coordinate system for shape testing
fn setup_shape_chart() -> State<&'static str, f64> {
    let mut state = State::new();

    state.set_axis(
        X,
        Axis::new(Linear::new(-20.0, 120.0), axis::Position::Bottom).invisible(),
    );

    state.set_axis(
        Y,
        Axis::new(Linear::new(-20.0, 120.0), axis::Position::Left).invisible(),
    );

    state
}

// -----------------------------------------------------------------------------
// Style Helper
// -----------------------------------------------------------------------------

fn style_default(theme: &Theme) -> Style {
    iced_aksel::style::default(theme)
}

// -----------------------------------------------------------------------------
// Rectangle Tests
// -----------------------------------------------------------------------------

struct RectangleShapes;

impl PlotData<f64> for RectangleShapes {
    fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {
        let palette = theme.palette();

        // Filled rectangle using corners
        plot.add_shape(
            Rectangle::corners(PlotPoint::new(10.0, 10.0), PlotPoint::new(30.0, 30.0))
                .fill(palette.primary),
        );

        // Stroked rectangle using centered
        plot.add_shape(
            Rectangle::centered(
                PlotPoint::new(50.0, 20.0),
                Measure::Plot(15.0),
                Measure::Plot(15.0),
            )
            .stroke(Stroke::new(palette.primary, Measure::Screen(2.0))),
        );

        // Screen-based rectangle (zoom independent)
        plot.add_shape(
            Rectangle::centered(
                PlotPoint::new(80.0, 20.0),
                Measure::Screen(30.0),
                Measure::Screen(20.0),
            )
            .fill(palette.success),
        );

        // Anisotropic (tall) rectangle
        plot.add_shape(
            Rectangle::centered(
                PlotPoint::new(50.0, 60.0),
                Measure::Plot(8.0),
                Measure::Plot(25.0),
            )
            .fill(palette.danger),
        );

        // Both fill and stroke
        plot.add_shape(
            Rectangle::centered(
                PlotPoint::new(50.0, 95.0),
                Measure::Plot(20.0),
                Measure::Plot(10.0),
            )
            .fill(palette.success)
            .stroke(Stroke::new(palette.text, Measure::Screen(2.0))),
        );
    }
}

#[test]
fn core_shapes_rectangles() {
    let state: &'static _ = Box::leak(Box::new(setup_shape_chart()));
    let shapes: &'static _ = Box::leak(Box::new(RectangleShapes));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(shapes, X, Y)
            .style(Box::new(style_default))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(&mut ui, "tests/snapshots/core_shapes_rectangles")
        .expect("Snapshot comparison failed for rectangles");
}

// -----------------------------------------------------------------------------
// Ellipse Tests
// -----------------------------------------------------------------------------

struct EllipseShapes;

impl PlotData<f64> for EllipseShapes {
    fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {
        let palette = theme.palette();

        // Perfect circle (filled)
        plot.add_shape(
            Ellipse::circle(PlotPoint::new(20.0, 20.0), Measure::Plot(10.0)).fill(palette.primary),
        );

        // Perfect circle (stroked)
        plot.add_shape(
            Ellipse::circle(PlotPoint::new(50.0, 20.0), Measure::Plot(10.0))
                .stroke(Stroke::new(palette.primary, Measure::Screen(2.0))),
        );

        // Wide ellipse
        plot.add_shape(
            Ellipse::new(
                PlotPoint::new(80.0, 20.0),
                Measure::Plot(15.0),
                Measure::Plot(8.0),
            )
            .fill(palette.success),
        );

        // Tall ellipse
        plot.add_shape(
            Ellipse::new(
                PlotPoint::new(50.0, 60.0),
                Measure::Plot(6.0),
                Measure::Plot(15.0),
            )
            .stroke(Stroke::new(palette.danger, Measure::Screen(2.0))),
        );

        // Both fill and stroke
        plot.add_shape(
            Ellipse::circle(PlotPoint::new(50.0, 95.0), Measure::Plot(12.0))
                .fill(palette.success)
                .stroke(Stroke::new(palette.text, Measure::Screen(3.0))),
        );
    }
}

#[test]
fn core_shapes_ellipses() {
    let state: &'static _ = Box::leak(Box::new(setup_shape_chart()));
    let shapes: &'static _ = Box::leak(Box::new(EllipseShapes));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(shapes, X, Y)
            .style(Box::new(style_default))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(&mut ui, "tests/snapshots/core_shapes_ellipses")
        .expect("Snapshot comparison failed for ellipses");
}

// -----------------------------------------------------------------------------
// Arc Tests
// -----------------------------------------------------------------------------

struct ArcShapes;

impl PlotData<f64> for ArcShapes {
    fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {
        let palette = theme.palette();

        // Sector (pie slice)
        plot.add_shape(
            Arc::new(PlotPoint::new(20.0, 20.0), Measure::Plot(12.0), 0.0, 4.0)
                .fill(palette.primary),
        );

        // Ring (donut)
        plot.add_shape(
            Arc::new(PlotPoint::new(50.0, 20.0), Measure::Plot(12.0), 0.0, 5.0)
                .inner_radius(Measure::Plot(7.0))
                .fill(palette.primary),
        );

        // Thin ring (stroked)
        plot.add_shape(
            Arc::new(PlotPoint::new(80.0, 20.0), Measure::Plot(12.0), 0.0, 3.14)
                .inner_radius(Measure::Plot(9.0))
                .stroke(Stroke::new(palette.success, Measure::Screen(2.0))),
        );

        // Full ring
        plot.add_shape(
            Arc::new(PlotPoint::new(50.0, 60.0), Measure::Plot(15.0), 0.0, 6.28)
                .inner_radius(Measure::Plot(8.0))
                .fill(palette.danger),
        );

        // Partial arc with stroke and fill
        plot.add_shape(
            Arc::new(PlotPoint::new(50.0, 95.0), Measure::Plot(15.0), 0.5, 5.0)
                .inner_radius(Measure::Plot(10.0))
                .fill(palette.success)
                .stroke(Stroke::new(palette.text, Measure::Screen(2.0))),
        );
    }
}

#[test]
fn core_shapes_arcs() {
    let state: &'static _ = Box::leak(Box::new(setup_shape_chart()));
    let shapes: &'static _ = Box::leak(Box::new(ArcShapes));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(shapes, X, Y)
            .style(Box::new(style_default))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(&mut ui, "tests/snapshots/core_shapes_arcs")
        .expect("Snapshot comparison failed for arcs");
}

// -----------------------------------------------------------------------------
// Triangle Tests
// -----------------------------------------------------------------------------

struct TriangleShapes;

impl PlotData<f64> for TriangleShapes {
    fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {
        let palette = theme.palette();

        // Vertex-based triangle
        plot.add_shape(
            Triangle::new(
                PlotPoint::new(15.0, 10.0),
                PlotPoint::new(25.0, 10.0),
                PlotPoint::new(20.0, 25.0),
            )
            .fill(palette.primary),
        );

        // Centered equilateral (stroked)
        plot.add_shape(
            Triangle::centered(
                PlotPoint::new(50.0, 20.0),
                Measure::Plot(15.0),
                Measure::Plot(15.0),
            )
            .stroke(Stroke::new(palette.primary, Measure::Screen(2.0))),
        );

        // Wide (directional marker)
        plot.add_shape(
            Triangle::centered(
                PlotPoint::new(80.0, 20.0),
                Measure::Plot(20.0),
                Measure::Plot(10.0),
            )
            .fill(palette.success),
        );

        // Tall (pointer)
        plot.add_shape(
            Triangle::centered(
                PlotPoint::new(50.0, 60.0),
                Measure::Plot(10.0),
                Measure::Plot(25.0),
            )
            .fill(palette.danger),
        );

        // Both fill and stroke
        plot.add_shape(
            Triangle::centered(
                PlotPoint::new(50.0, 95.0),
                Measure::Plot(18.0),
                Measure::Plot(15.0),
            )
            .fill(palette.success)
            .stroke(Stroke::new(palette.text, Measure::Screen(2.0))),
        );
    }
}

#[test]
fn core_shapes_triangles() {
    let state: &'static _ = Box::leak(Box::new(setup_shape_chart()));
    let shapes: &'static _ = Box::leak(Box::new(TriangleShapes));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(shapes, X, Y)
            .style(Box::new(style_default))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(&mut ui, "tests/snapshots/core_shapes_triangles")
        .expect("Snapshot comparison failed for triangles");
}

// -----------------------------------------------------------------------------
// Polygon Tests
// -----------------------------------------------------------------------------

struct PolygonShapes;

impl PlotData<f64> for PolygonShapes {
    fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {
        let palette = theme.palette();

        // Pentagon
        plot.add_shape(
            Polygon::new(PlotPoint::new(20.0, 20.0), Measure::Plot(12.0), 5).fill(palette.primary),
        );

        // Hexagon (stroked)
        plot.add_shape(
            Polygon::new(PlotPoint::new(50.0, 20.0), Measure::Plot(12.0), 6)
                .stroke(Stroke::new(palette.primary, Measure::Screen(2.0))),
        );

        // Octagon
        plot.add_shape(
            Polygon::new(PlotPoint::new(80.0, 20.0), Measure::Plot(12.0), 8).fill(palette.success),
        );

        // Diamond (rotated square)
        plot.add_shape(
            Polygon::new(PlotPoint::new(50.0, 60.0), Measure::Plot(12.0), 4)
                .rotation(45.0)
                .fill(palette.danger),
        );

        // Triangle (3-sided polygon) with rotation
        plot.add_shape(
            Polygon::new(PlotPoint::new(50.0, 95.0), Measure::Plot(15.0), 3)
                .rotation(30.0)
                .fill(palette.success)
                .stroke(Stroke::new(palette.text, Measure::Screen(2.0))),
        );
    }
}

#[test]
fn core_shapes_polygons() {
    let state: &'static _ = Box::leak(Box::new(setup_shape_chart()));
    let shapes: &'static _ = Box::leak(Box::new(PolygonShapes));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(shapes, X, Y)
            .style(Box::new(style_default))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(&mut ui, "tests/snapshots/core_shapes_polygons")
        .expect("Snapshot comparison failed for polygons");
}

// -----------------------------------------------------------------------------
// Line Tests
// -----------------------------------------------------------------------------

struct LineShapes;

impl PlotData<f64> for LineShapes {
    fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {
        let palette = theme.palette();

        // Basic segment
        plot.add_shape(
            Line::new(PlotPoint::new(10.0, 10.0), PlotPoint::new(30.0, 30.0))
                .stroke(Stroke::new(palette.primary, Measure::Screen(2.0))),
        );

        // Arrow end
        plot.add_shape(
            Line::new(PlotPoint::new(40.0, 20.0), PlotPoint::new(60.0, 20.0))
                .stroke(Stroke::new(palette.primary, Measure::Screen(2.0)))
                .arrow_end(true),
        );

        // Double arrow
        plot.add_shape(
            Line::new(PlotPoint::new(70.0, 30.0), PlotPoint::new(90.0, 10.0))
                .stroke(Stroke::new(palette.success, Measure::Screen(2.0)))
                .arrows(true),
        );

        // Infinite line (trend line)
        plot.add_shape(
            Line::new(PlotPoint::new(45.0, 55.0), PlotPoint::new(55.0, 65.0))
                .stroke(Stroke::new(palette.danger, Measure::Screen(1.0)))
                .infinite(),
        );

        // Thick line
        plot.add_shape(
            Line::new(PlotPoint::new(30.0, 90.0), PlotPoint::new(70.0, 100.0))
                .stroke(Stroke::new(palette.text, Measure::Screen(5.0))),
        );
    }
}

#[test]
fn core_shapes_lines() {
    let state: &'static _ = Box::leak(Box::new(setup_shape_chart()));
    let shapes: &'static _ = Box::leak(Box::new(LineShapes));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(shapes, X, Y)
            .style(Box::new(style_default))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(&mut ui, "tests/snapshots/core_shapes_lines")
        .expect("Snapshot comparison failed for lines");
}

// -----------------------------------------------------------------------------
// Polyline Tests
// -----------------------------------------------------------------------------

struct PolylineShapes;

impl PlotData<f64> for PolylineShapes {
    fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {
        let palette = theme.palette();

        // Basic zig-zag
        plot.add_shape(
            Polyline::new(vec![
                PlotPoint::new(10.0, 10.0),
                PlotPoint::new(15.0, 25.0),
                PlotPoint::new(20.0, 10.0),
                PlotPoint::new(25.0, 25.0),
                PlotPoint::new(30.0, 10.0),
            ])
            .stroke(Stroke::new(palette.primary, Measure::Screen(2.0))),
        );

        // Thick stroke
        plot.add_shape(
            Polyline::new(vec![
                PlotPoint::new(40.0, 15.0),
                PlotPoint::new(50.0, 25.0),
                PlotPoint::new(60.0, 15.0),
            ])
            .stroke(Stroke::new(palette.primary, Measure::Screen(5.0))),
        );

        // Arrow start
        plot.add_shape(
            Polyline::new(vec![
                PlotPoint::new(70.0, 25.0),
                PlotPoint::new(80.0, 10.0),
                PlotPoint::new(90.0, 25.0),
            ])
            .stroke(Stroke::new(palette.success, Measure::Screen(2.0)))
            .arrow_start(true),
        );

        // Arrow end
        plot.add_shape(
            Polyline::new(vec![
                PlotPoint::new(35.0, 50.0),
                PlotPoint::new(50.0, 65.0),
                PlotPoint::new(65.0, 50.0),
            ])
            .stroke(Stroke::new(palette.danger, Measure::Screen(2.0)))
            .arrow_end(true),
        );

        // Complex path with both arrows
        plot.add_shape(
            Polyline::new(vec![
                PlotPoint::new(25.0, 85.0),
                PlotPoint::new(40.0, 95.0),
                PlotPoint::new(50.0, 85.0),
                PlotPoint::new(60.0, 100.0),
                PlotPoint::new(75.0, 90.0),
            ])
            .stroke(Stroke::new(palette.text, Measure::Screen(2.0)))
            .arrow_start(true)
            .arrow_end(true),
        );
    }
}

#[test]
fn core_shapes_polylines() {
    let state: &'static _ = Box::leak(Box::new(setup_shape_chart()));
    let shapes: &'static _ = Box::leak(Box::new(PolylineShapes));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(shapes, X, Y)
            .style(Box::new(style_default))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(&mut ui, "tests/snapshots/core_shapes_polylines")
        .expect("Snapshot comparison failed for polylines");
}

// -----------------------------------------------------------------------------
// Spline Tests (Catmull-Rom)
// -----------------------------------------------------------------------------

struct SplineShapes;

impl PlotData<f64> for SplineShapes {
    fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {
        let palette = theme.palette();

        let curve_points = vec![
            PlotPoint::new(5.0, 10.0),
            PlotPoint::new(12.0, 25.0),
            PlotPoint::new(20.0, 10.0),
            PlotPoint::new(28.0, 25.0),
            PlotPoint::new(35.0, 10.0),
        ];

        // Default tension (0.0 - smooth)
        plot.add_shape(
            Spline::new(curve_points.clone())
                .stroke(Stroke::new(palette.primary, Measure::Screen(2.0))),
        );

        // Tension 0.5 (tighter)
        plot.add_shape(
            Spline::new(vec![
                PlotPoint::new(40.0, 10.0),
                PlotPoint::new(47.0, 25.0),
                PlotPoint::new(55.0, 10.0),
                PlotPoint::new(63.0, 25.0),
                PlotPoint::new(70.0, 10.0),
            ])
            .tension(0.5)
            .stroke(Stroke::new(palette.primary, Measure::Screen(2.0))),
        );

        // Tension 1.0 (linear)
        plot.add_shape(
            Spline::new(vec![
                PlotPoint::new(75.0, 10.0),
                PlotPoint::new(82.0, 25.0),
                PlotPoint::new(90.0, 10.0),
                PlotPoint::new(98.0, 25.0),
                PlotPoint::new(105.0, 10.0),
            ])
            .tension(1.0)
            .stroke(Stroke::new(palette.success, Measure::Screen(2.0))),
        );

        // Closed loop
        plot.add_shape(
            Spline::new(vec![
                PlotPoint::new(50.0, 75.0),
                PlotPoint::new(65.0, 60.0),
                PlotPoint::new(50.0, 45.0),
                PlotPoint::new(35.0, 60.0),
                PlotPoint::new(50.0, 75.0),
            ])
            .stroke(Stroke::new(palette.danger, Measure::Screen(2.0))),
        );

        // Complex path with many points
        plot.add_shape(
            Spline::new(vec![
                PlotPoint::new(20.0, 85.0),
                PlotPoint::new(30.0, 95.0),
                PlotPoint::new(40.0, 90.0),
                PlotPoint::new(50.0, 100.0),
                PlotPoint::new(60.0, 92.0),
                PlotPoint::new(70.0, 98.0),
                PlotPoint::new(80.0, 88.0),
            ])
            .tension(0.3)
            .stroke(Stroke::new(palette.text, Measure::Screen(2.5))),
        );
    }
}

#[test]
fn core_shapes_splines() {
    let state: &'static _ = Box::leak(Box::new(setup_shape_chart()));
    let shapes: &'static _ = Box::leak(Box::new(SplineShapes));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(shapes, X, Y)
            .style(Box::new(style_default))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(&mut ui, "tests/snapshots/core_shapes_splines")
        .expect("Snapshot comparison failed for splines");
}

// -----------------------------------------------------------------------------
// Bezier Tests
// -----------------------------------------------------------------------------

struct BezierShapes;

impl PlotData<f64> for BezierShapes {
    fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {
        let palette = theme.palette();

        // Quadratic (1 control point)
        plot.add_shape(
            Bezier::quadratic(
                PlotPoint::new(10.0, 10.0),
                PlotPoint::new(20.0, 30.0),
                PlotPoint::new(30.0, 10.0),
            )
            .stroke(Stroke::new(palette.primary, Measure::Screen(2.0))),
        );

        // Cubic (2 control points) - S-shape
        plot.add_shape(
            Bezier::cubic(
                PlotPoint::new(40.0, 10.0),
                PlotPoint::new(47.0, 30.0),
                PlotPoint::new(53.0, -5.0),
                PlotPoint::new(60.0, 20.0),
            )
            .stroke(Stroke::new(palette.primary, Measure::Screen(2.0))),
        );

        // Cubic loop
        plot.add_shape(
            Bezier::cubic(
                PlotPoint::new(80.0, 10.0),
                PlotPoint::new(65.0, 35.0),
                PlotPoint::new(95.0, 35.0),
                PlotPoint::new(80.0, 10.0),
            )
            .stroke(Stroke::new(palette.success, Measure::Screen(2.0))),
        );

        // Thick quadratic
        plot.add_shape(
            Bezier::quadratic(
                PlotPoint::new(35.0, 50.0),
                PlotPoint::new(50.0, 70.0),
                PlotPoint::new(65.0, 50.0),
            )
            .stroke(Stroke::new(palette.danger, Measure::Screen(6.0))),
        );

        // Complex cubic with extreme control points
        plot.add_shape(
            Bezier::cubic(
                PlotPoint::new(20.0, 85.0),
                PlotPoint::new(30.0, 110.0),
                PlotPoint::new(70.0, 75.0),
                PlotPoint::new(80.0, 95.0),
            )
            .stroke(Stroke::new(palette.text, Measure::Screen(2.5))),
        );
    }
}

#[test]
fn core_shapes_beziers() {
    let state: &'static _ = Box::leak(Box::new(setup_shape_chart()));
    let shapes: &'static _ = Box::leak(Box::new(BezierShapes));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(shapes, X, Y)
            .style(Box::new(style_default))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(&mut ui, "tests/snapshots/core_shapes_beziers")
        .expect("Snapshot comparison failed for beziers");
}

// -----------------------------------------------------------------------------
// Area Tests (Arbitrary Polygons)
// -----------------------------------------------------------------------------

struct AreaShapes;

impl PlotData<f64> for AreaShapes {
    fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {
        let palette = theme.palette();

        // Filled concave shape
        plot.add_shape(
            Area::new(vec![
                PlotPoint::new(20.0, 30.0),
                PlotPoint::new(30.0, 15.0),
                PlotPoint::new(20.0, 20.0), // Concavity
                PlotPoint::new(10.0, 15.0),
            ])
            .fill(palette.primary),
        );

        // Stroked concave
        plot.add_shape(
            Area::new(vec![
                PlotPoint::new(50.0, 30.0),
                PlotPoint::new(60.0, 15.0),
                PlotPoint::new(50.0, 20.0),
                PlotPoint::new(40.0, 15.0),
            ])
            .stroke(Stroke::new(palette.primary, Measure::Screen(2.0))),
        );

        // Fill + Stroke
        plot.add_shape(
            Area::new(vec![
                PlotPoint::new(80.0, 30.0),
                PlotPoint::new(90.0, 15.0),
                PlotPoint::new(80.0, 20.0),
                PlotPoint::new(70.0, 15.0),
            ])
            .fill(palette.success)
            .stroke(Stroke::new(palette.text, Measure::Screen(1.5))),
        );

        // Star shape
        plot.add_shape(
            Area::new(vec![
                PlotPoint::new(50.0, 75.0),
                PlotPoint::new(53.0, 63.0),
                PlotPoint::new(62.0, 63.0),
                PlotPoint::new(55.0, 56.0),
                PlotPoint::new(58.0, 45.0),
                PlotPoint::new(50.0, 51.0),
                PlotPoint::new(42.0, 45.0),
                PlotPoint::new(45.0, 56.0),
                PlotPoint::new(38.0, 63.0),
                PlotPoint::new(47.0, 63.0),
            ])
            .fill(palette.danger),
        );

        // Irregular polygon
        plot.add_shape(
            Area::new(vec![
                PlotPoint::new(25.0, 85.0),
                PlotPoint::new(40.0, 100.0),
                PlotPoint::new(55.0, 95.0),
                PlotPoint::new(70.0, 105.0),
                PlotPoint::new(75.0, 88.0),
                PlotPoint::new(60.0, 85.0),
                PlotPoint::new(45.0, 92.0),
                PlotPoint::new(30.0, 90.0),
            ])
            .fill(palette.success)
            .stroke(Stroke::new(palette.text, Measure::Screen(2.0))),
        );
    }
}

#[test]
fn core_shapes_areas() {
    let state: &'static _ = Box::leak(Box::new(setup_shape_chart()));
    let shapes: &'static _ = Box::leak(Box::new(AreaShapes));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(shapes, X, Y)
            .style(Box::new(style_default))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(&mut ui, "tests/snapshots/core_shapes_areas")
        .expect("Snapshot comparison failed for areas");
}

// -----------------------------------------------------------------------------
// Label Tests
// -----------------------------------------------------------------------------

struct LabelShapes;

impl PlotData<f64> for LabelShapes {
    fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {
        let palette = theme.palette();

        // Basic label (default alignment)
        plot.add_shape(
            Label::new("Basic", PlotPoint::new(20.0, 20.0))
                .size(14.0)
                .fill(palette.text),
        );

        // Left aligned
        plot.add_shape(
            Label::new("Left", PlotPoint::new(50.0, 20.0))
                .size(16.0)
                .fill(palette.primary)
                .align(Horizontal::Left, Vertical::Center),
        );

        // Right aligned
        plot.add_shape(
            Label::new("Right", PlotPoint::new(80.0, 20.0))
                .size(16.0)
                .fill(palette.success)
                .align(Horizontal::Right, Vertical::Center),
        );

        // Top aligned
        plot.add_shape(
            Label::new("Top", PlotPoint::new(50.0, 50.0))
                .size(18.0)
                .fill(palette.danger)
                .align(Horizontal::Center, Vertical::Top),
        );

        // Bottom aligned
        plot.add_shape(
            Label::new("Bottom", PlotPoint::new(50.0, 70.0))
                .size(18.0)
                .fill(palette.danger)
                .align(Horizontal::Center, Vertical::Bottom),
        );

        // Large text
        plot.add_shape(
            Label::new("LARGE", PlotPoint::new(50.0, 95.0))
                .size(24.0)
                .fill(palette.text)
                .align(Horizontal::Center, Vertical::Center),
        );
    }
}

#[test]
fn core_shapes_labels() {
    let state: &'static _ = Box::leak(Box::new(setup_shape_chart()));
    let shapes: &'static _ = Box::leak(Box::new(LabelShapes));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(shapes, X, Y)
            .style(Box::new(style_default))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(&mut ui, "tests/snapshots/core_shapes_labels")
        .expect("Snapshot comparison failed for labels");
}

// -----------------------------------------------------------------------------
// Combined Shape Gallery Test
// -----------------------------------------------------------------------------

struct ShapeGallery;

impl PlotData<f64> for ShapeGallery {
    fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {
        let palette = theme.palette();

        // Create a small gallery with one of each shape type
        let row_1 = 90.0;
        let row_2 = 60.0;
        let row_3 = 30.0;

        let col_1 = 15.0;
        let col_2 = 35.0;
        let col_3 = 55.0;
        let col_4 = 75.0;
        let col_5 = 95.0;

        // Row 1
        plot.add_shape(
            Rectangle::centered(
                PlotPoint::new(col_1, row_1),
                Measure::Plot(8.0),
                Measure::Plot(8.0),
            )
            .fill(palette.primary),
        );
        plot.add_shape(
            Ellipse::circle(PlotPoint::new(col_2, row_1), Measure::Plot(5.0)).fill(palette.success),
        );
        plot.add_shape(
            Triangle::centered(
                PlotPoint::new(col_3, row_1),
                Measure::Plot(10.0),
                Measure::Plot(10.0),
            )
            .fill(palette.danger),
        );
        plot.add_shape(
            Polygon::new(PlotPoint::new(col_4, row_1), Measure::Plot(6.0), 6).fill(palette.primary),
        );
        plot.add_shape(
            Arc::new(PlotPoint::new(col_5, row_1), Measure::Plot(6.0), 0.0, 4.5)
                .fill(palette.success),
        );

        // Row 2
        plot.add_shape(
            Line::new(
                PlotPoint::new(col_1 - 5.0, row_2 - 5.0),
                PlotPoint::new(col_1 + 5.0, row_2 + 5.0),
            )
            .stroke(Stroke::new(palette.primary, Measure::Screen(2.0)))
            .arrows(true),
        );
        plot.add_shape(
            Polyline::new(vec![
                PlotPoint::new(col_2 - 5.0, row_2 - 3.0),
                PlotPoint::new(col_2, row_2 + 5.0),
                PlotPoint::new(col_2 + 5.0, row_2 - 3.0),
            ])
            .stroke(Stroke::new(palette.success, Measure::Screen(2.0))),
        );
        plot.add_shape(
            Spline::new(vec![
                PlotPoint::new(col_3 - 7.0, row_2),
                PlotPoint::new(col_3 - 2.0, row_2 + 5.0),
                PlotPoint::new(col_3 + 2.0, row_2 - 5.0),
                PlotPoint::new(col_3 + 7.0, row_2),
            ])
            .stroke(Stroke::new(palette.danger, Measure::Screen(2.0))),
        );
        plot.add_shape(
            Bezier::quadratic(
                PlotPoint::new(col_4 - 7.0, row_2),
                PlotPoint::new(col_4, row_2 + 7.0),
                PlotPoint::new(col_4 + 7.0, row_2),
            )
            .stroke(Stroke::new(palette.primary, Measure::Screen(2.0))),
        );
        plot.add_shape(
            Area::new(vec![
                PlotPoint::new(col_5, row_2 + 5.0),
                PlotPoint::new(col_5 + 5.0, row_2 - 2.0),
                PlotPoint::new(col_5, row_2 + 1.0),
                PlotPoint::new(col_5 - 5.0, row_2 - 2.0),
            ])
            .fill(palette.success),
        );

        // Row 3: Labels
        plot.add_shape(
            Label::new("Rect", PlotPoint::new(col_1, row_3))
                .size(10.0)
                .fill(palette.text)
                .align(Horizontal::Center, Vertical::Center),
        );
        plot.add_shape(
            Label::new("Ellipse", PlotPoint::new(col_2, row_3))
                .size(10.0)
                .fill(palette.text)
                .align(Horizontal::Center, Vertical::Center),
        );
        plot.add_shape(
            Label::new("Triangle", PlotPoint::new(col_3, row_3))
                .size(10.0)
                .fill(palette.text)
                .align(Horizontal::Center, Vertical::Center),
        );
        plot.add_shape(
            Label::new("Polygon", PlotPoint::new(col_4, row_3))
                .size(10.0)
                .fill(palette.text)
                .align(Horizontal::Center, Vertical::Center),
        );
        plot.add_shape(
            Label::new("Arc", PlotPoint::new(col_5, row_3))
                .size(10.0)
                .fill(palette.text)
                .align(Horizontal::Center, Vertical::Center),
        );
    }
}

#[test]
fn core_shapes_gallery() {
    let state: &'static _ = Box::leak(Box::new(setup_shape_chart()));
    let shapes: &'static _ = Box::leak(Box::new(ShapeGallery));

    let view_fn = move || {
        Chart::new(state)
            .plot_data(shapes, X, Y)
            .style(Box::new(style_default))
            .into()
    };

    let (app, _) = App::new(view_fn);
    let mut ui = simulator(&app);

    assert_snapshot_matches(&mut ui, "tests/snapshots/core_shapes_gallery")
        .expect("Snapshot comparison failed for shape gallery");
}
