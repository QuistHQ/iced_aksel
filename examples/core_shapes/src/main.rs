use iced::{
    Color, Theme,
    alignment::{Horizontal, Vertical},
};
use iced_aksel::{
    Axis, Chart, Measure, PlotPoint, State, Stroke, axis,
    plot::{Plot, PlotData},
    scale::Linear,
    shape::{
        Arc, Area, Bezier, Ellipse, Label, Line, Polygon, Polyline, Rectangle, Spline, Triangle,
    },
};

type AxisId = &'static str;

pub fn main() -> iced::Result {
    iced::application(ShapeGallery::new, ShapeGallery::update, ShapeGallery::view)
        .title("Aksel Shape Matrix")
        .antialiasing(true)
        .run()
}

pub struct ShapeGallery {
    state: State<AxisId, f64>,
}

#[derive(Debug, Clone)]
pub enum Message {}

impl ShapeGallery {
    const X: &'static str = "X";
    const Y: &'static str = "Y";

    pub fn new() -> (Self, iced::Task<Message>) {
        let mut state = State::new();

        // Set up a coordinate system large enough for 10 rows of shapes
        // Y-axis: 0 to 300 (Top to Bottom layout logic)
        // X-axis: 0 to 140 (4 Columns)
        state.set_axis(
            Self::X,
            Axis::new(Linear::new(-20.0, 150.0), axis::Position::Bottom).invisible(),
        );

        state.set_axis(
            Self::Y,
            Axis::new(Linear::new(-20.0, 320.0), axis::Position::Left).invisible(),
        );

        (Self { state }, iced::Task::none())
    }

    pub fn update(&mut self, _message: Message) -> iced::Task<Message> {
        iced::Task::none()
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        Chart::new(&self.state)
            .plot_data(self, Self::X, Self::Y)
            .into()
    }
}

impl PlotData<f64> for ShapeGallery {
    fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {
        let palette = theme.palette();
        let text_main = palette.text;
        let text_dim = Color::from_rgba(palette.text.r, palette.text.g, palette.text.b, 0.6);

        // --- Layout Constants ---
        let col_1 = 20.0; // Basic / Fill
        let col_2 = 55.0; // Stroked / Styled
        let col_3 = 90.0; // Variation 1 (Rotation, Tension, Arrows)
        let col_4 = 125.0; // Variation 2 (Complex, Infinite)

        let row_height = 30.0;
        let mut y_cursor = 290.0; // Start from top

        // --- Helper: Draw Grid Headers ---
        // Pass 'plot' as an argument to avoid capturing it mutably for the whole scope
        let draw_header = |plot: &mut Plot<f64>, x: f64, text: &str| {
            plot.add_shape(
                Label::new(text, PlotPoint::new(x, 310.0))
                    .size(Measure::Screen(14.0))
                    .fill(text_main)
                    .align(Horizontal::Center, Vertical::Bottom),
            );
        };

        draw_header(plot, col_1, "Basic / Fill");
        draw_header(plot, col_2, "Stroked");
        draw_header(plot, col_3, "Variation A");
        draw_header(plot, col_4, "Variation B");

        // --- Helper: Draw Row Title ---
        // Pass 'plot' as an argument here as well
        let mut next_row = |plot: &mut Plot<f64>, title: &str| -> f64 {
            let current_y = y_cursor;
            plot.add_shape(
                Label::new(title, PlotPoint::new(-15.0, current_y))
                    .size(Measure::Screen(16.0))
                    .fill(text_main)
                    .align(Horizontal::Left, Vertical::Center),
            );

            // Draw subtle separator line
            plot.add_shape(
                Line::new(
                    PlotPoint::new(-15.0, current_y - 15.0),
                    PlotPoint::new(145.0, current_y - 15.0),
                )
                .stroke(Stroke::new(
                    Color::from_rgba(0.5, 0.5, 0.5, 0.1),
                    Measure::Screen(1.0),
                )),
            );

            y_cursor -= row_height;
            current_y
        };

        // =====================================================================
        //  1. RECTANGLE
        // =====================================================================
        let y = next_row(plot, "Rectangle");

        // 1. Basic Fill (Corners)
        plot.add_shape(
            Rectangle::corners(
                PlotPoint::new(col_1 - 5.0, y - 5.0),
                PlotPoint::new(col_1 + 5.0, y + 5.0),
            )
            .fill(palette.primary),
        );

        // 2. Centered Stroke
        plot.add_shape(
            Rectangle::centered(
                PlotPoint::new(col_2, y),
                Measure::Plot(10.0),
                Measure::Plot(10.0),
            )
            .stroke(Stroke::new(palette.primary, Measure::Screen(2.0))),
        );

        // 3. Fixed Screen Size (Zoom Independent)
        plot.add_shape(
            Rectangle::centered(
                PlotPoint::new(col_3, y),
                Measure::Screen(20.0),
                Measure::Screen(10.0), // Wide aspect
            )
            .fill(palette.success),
        );
        plot.add_shape(
            Label::new("Screen Px", PlotPoint::new(col_3, y - 8.0))
                .size(Measure::Screen(8.0))
                .fill(text_dim),
        );

        // 4. Tall (Anisotropic)
        plot.add_shape(
            Rectangle::centered(
                PlotPoint::new(col_4, y),
                Measure::Plot(4.0),
                Measure::Plot(12.0),
            )
            .fill(palette.danger),
        );

        // =====================================================================
        //  2. ELLIPSE (CIRCLE)
        // =====================================================================
        let y = next_row(plot, "Ellipse");

        // 1. Perfect Circle (Fill)
        plot.add_shape(
            Ellipse::circle(PlotPoint::new(col_1, y), Measure::Plot(6.0)).fill(palette.primary),
        );

        // 2. Perfect Circle (Stroke)
        plot.add_shape(
            Ellipse::circle(PlotPoint::new(col_2, y), Measure::Plot(6.0))
                .stroke(Stroke::new(palette.primary, Measure::Screen(2.0))),
        );

        // 3. Ellipse (Wide)
        plot.add_shape(
            Ellipse::new(
                PlotPoint::new(col_3, y),
                Measure::Plot(8.0),
                Measure::Plot(4.0),
            )
            .fill(palette.success),
        );

        // 4. Ellipse (Tall)
        plot.add_shape(
            Ellipse::new(
                PlotPoint::new(col_4, y),
                Measure::Plot(3.0),
                Measure::Plot(8.0),
            )
            .stroke(Stroke::new(palette.danger, Measure::Screen(2.0))),
        );

        // =====================================================================
        //  3. ARC
        // =====================================================================
        let y = next_row(plot, "Arc");

        // 1. Sector (Pie Slice)
        plot.add_shape(
            Arc::new(
                PlotPoint::new(col_1, y),
                Measure::Plot(8.0),
                0.0,
                4.0, // Radians
            )
            .fill(palette.primary),
        );

        // 2. Ring (Donut)
        plot.add_shape(
            Arc::new(PlotPoint::new(col_2, y), Measure::Plot(8.0), 0.0, 5.0)
                .inner_radius(Measure::Plot(5.0))
                .fill(palette.primary),
        );

        // 3. Thin Ring (Stroked)
        plot.add_shape(
            Arc::new(PlotPoint::new(col_3, y), Measure::Plot(8.0), 0.0, 3.14)
                .inner_radius(Measure::Plot(6.0))
                .stroke(Stroke::new(palette.success, Measure::Screen(2.0))),
        );

        // 4. Full Ring
        plot.add_shape(
            Arc::new(PlotPoint::new(col_4, y), Measure::Plot(8.0), 0.0, 6.28)
                .inner_radius(Measure::Plot(4.0))
                .fill(palette.danger),
        );

        // =====================================================================
        //  4. TRIANGLE
        // =====================================================================
        let y = next_row(plot, "Triangle");

        // 1. Vertex Based
        plot.add_shape(
            Triangle::new(
                PlotPoint::new(col_1 - 5.0, y - 5.0),
                PlotPoint::new(col_1 + 5.0, y - 5.0),
                PlotPoint::new(col_1, y + 5.0),
            )
            .fill(palette.primary),
        );

        // 2. Centered (Equilateral-ish)
        plot.add_shape(
            Triangle::centered(
                PlotPoint::new(col_2, y),
                Measure::Plot(10.0),
                Measure::Plot(10.0),
            )
            .stroke(Stroke::new(palette.primary, Measure::Screen(2.0))),
        );

        // 3. Wide (Directional Marker)
        plot.add_shape(
            Triangle::centered(
                PlotPoint::new(col_3, y),
                Measure::Plot(15.0),
                Measure::Plot(6.0),
            )
            .fill(palette.success),
        );

        // 4. Tall (Pointer)
        plot.add_shape(
            Triangle::centered(
                PlotPoint::new(col_4, y),
                Measure::Plot(6.0),
                Measure::Plot(15.0),
            )
            .fill(palette.danger),
        );

        // =====================================================================
        //  5. POLYGON (REGULAR)
        // =====================================================================
        let y = next_row(plot, "Polygon");

        // 1. Pentagon
        plot.add_shape(
            Polygon::new(PlotPoint::new(col_1, y), Measure::Plot(7.0), 5).fill(palette.primary),
        );

        // 2. Hexagon (Stroked)
        plot.add_shape(
            Polygon::new(PlotPoint::new(col_2, y), Measure::Plot(7.0), 6)
                .stroke(Stroke::new(palette.primary, Measure::Screen(2.0))),
        );

        // 3. Octagon
        plot.add_shape(
            Polygon::new(PlotPoint::new(col_3, y), Measure::Plot(7.0), 8).fill(palette.success),
        );

        // 4. Diamond (Square Rotated)
        plot.add_shape(
            Polygon::new(PlotPoint::new(col_4, y), Measure::Plot(7.0), 4)
                .rotation(45.0)
                .fill(palette.danger),
        );
        plot.add_shape(
            Label::new("Rotated", PlotPoint::new(col_4, y - 8.0))
                .size(Measure::Screen(8.0))
                .fill(text_dim),
        );

        // =====================================================================
        //  6. LINE
        // =====================================================================
        let y = next_row(plot, "Line");

        // 1. Basic Segment
        plot.add_shape(
            Line::new(
                PlotPoint::new(col_1 - 8.0, y - 5.0),
                PlotPoint::new(col_1 + 8.0, y + 5.0),
            )
            .stroke(Stroke::new(palette.primary, Measure::Screen(2.0))),
        );

        // 2. Arrow End
        plot.add_shape(
            Line::new(
                PlotPoint::new(col_2 - 8.0, y),
                PlotPoint::new(col_2 + 8.0, y),
            )
            .stroke(Stroke::new(palette.primary, Measure::Screen(2.0)))
            .arrow_end(true),
        );

        // 3. Double Arrow
        plot.add_shape(
            Line::new(
                PlotPoint::new(col_3 - 8.0, y + 5.0),
                PlotPoint::new(col_3 + 8.0, y - 5.0),
            )
            .stroke(Stroke::new(palette.success, Measure::Screen(2.0)))
            .arrows(true),
        );

        // 4. Infinite Extension (Trend line)
        plot.add_shape(
            Line::new(
                PlotPoint::new(col_4 - 2.0, y - 2.0),
                PlotPoint::new(col_4 + 2.0, y + 2.0),
            )
            .stroke(Stroke::new(palette.danger, Measure::Screen(1.0)))
            .infinite(),
        );
        plot.add_shape(
            Label::new("Infinite", PlotPoint::new(col_4, y - 8.0))
                .size(Measure::Screen(8.0))
                .fill(text_dim),
        );

        // =====================================================================
        //  7. POLYLINE
        // =====================================================================
        let y = next_row(plot, "Polyline");

        let zig_zag = |cx: f64| {
            vec![
                PlotPoint::new(cx - 8.0, y - 5.0),
                PlotPoint::new(cx - 3.0, y + 5.0),
                PlotPoint::new(cx + 3.0, y - 5.0),
                PlotPoint::new(cx + 8.0, y + 5.0),
            ]
        };

        // 1. Basic
        plot.add_shape(
            Polyline::new(zig_zag(col_1))
                .stroke(Stroke::new(palette.primary, Measure::Screen(2.0))),
        );

        // 2. Thick
        plot.add_shape(
            Polyline::new(zig_zag(col_2))
                .stroke(Stroke::new(palette.primary, Measure::Screen(4.0))),
        );

        // 3. Arrow Start
        plot.add_shape(
            Polyline::new(zig_zag(col_3))
                .stroke(Stroke::new(palette.success, Measure::Screen(2.0)))
                .arrow_start(true),
        );

        // 4. Arrow End
        plot.add_shape(
            Polyline::new(zig_zag(col_4))
                .stroke(Stroke::new(palette.danger, Measure::Screen(2.0)))
                .arrow_end(true),
        );

        // =====================================================================
        //  8. SPLINE (Catmull-Rom)
        // =====================================================================
        let y = next_row(plot, "Spline");

        let curve_pts = |cx: f64| {
            vec![
                PlotPoint::new(cx - 10.0, y - 5.0),
                PlotPoint::new(cx - 5.0, y + 5.0),
                PlotPoint::new(cx, y - 5.0),
                PlotPoint::new(cx + 5.0, y + 5.0),
                PlotPoint::new(cx + 10.0, y - 5.0),
            ]
        };

        // 1. Default (Tension 0.0 - Smooth)
        plot.add_shape(
            Spline::new(curve_pts(col_1))
                .stroke(Stroke::new(palette.primary, Measure::Screen(2.0))),
        );
        plot.add_shape(
            Label::new("Default (0.0)", PlotPoint::new(col_1, y - 8.0))
                .size(Measure::Screen(8.0))
                .fill(text_dim),
        );

        // 2. Tension 0.5 (Tighter)
        plot.add_shape(
            Spline::new(curve_pts(col_2))
                .tension(0.5)
                .stroke(Stroke::new(palette.primary, Measure::Screen(2.0))),
        );
        plot.add_shape(
            Label::new("Tension 0.5", PlotPoint::new(col_2, y - 8.0))
                .size(Measure::Screen(8.0))
                .fill(text_dim),
        );

        // 3. Tension 1.0 (Linear)
        plot.add_shape(
            Spline::new(curve_pts(col_3))
                .tension(1.0)
                .stroke(Stroke::new(palette.success, Measure::Screen(2.0))),
        );
        plot.add_shape(
            Label::new("Tension 1.0", PlotPoint::new(col_3, y - 8.0))
                .size(Measure::Screen(8.0))
                .fill(text_dim),
        );

        // 4. Closed Loop (Manual) - demonstrating flexibility
        let loop_pts = vec![
            PlotPoint::new(col_4, y + 8.0),
            PlotPoint::new(col_4 + 8.0, y),
            PlotPoint::new(col_4, y - 8.0),
            PlotPoint::new(col_4 - 8.0, y),
            PlotPoint::new(col_4, y + 8.0), // Close loop
        ];
        plot.add_shape(
            Spline::new(loop_pts).stroke(Stroke::new(palette.danger, Measure::Screen(2.0))),
        );
        plot.add_shape(
            Label::new("Loop", PlotPoint::new(col_4, y - 10.0))
                .size(Measure::Screen(8.0))
                .fill(text_dim),
        );

        // =====================================================================
        //  9. BEZIER
        // =====================================================================
        let y = next_row(plot, "Bezier");

        // 1. Quadratic (1 Control)
        plot.add_shape(
            Bezier::quadratic(
                PlotPoint::new(col_1 - 10.0, y - 5.0),
                PlotPoint::new(col_1, y + 10.0), // Control
                PlotPoint::new(col_1 + 10.0, y - 5.0),
            )
            .stroke(Stroke::new(palette.primary, Measure::Screen(2.0))),
        );
        plot.add_shape(
            Label::new("Quadratic", PlotPoint::new(col_1, y - 8.0))
                .size(Measure::Screen(8.0))
                .fill(text_dim),
        );

        // 2. Cubic (2 Controls) - S-shape
        plot.add_shape(
            Bezier::cubic(
                PlotPoint::new(col_2 - 10.0, y - 5.0),
                PlotPoint::new(col_2 - 3.0, y + 8.0), // C1
                PlotPoint::new(col_2 + 3.0, y - 8.0), // C2
                PlotPoint::new(col_2 + 10.0, y + 5.0),
            )
            .stroke(Stroke::new(palette.primary, Measure::Screen(2.0))),
        );
        plot.add_shape(
            Label::new("Cubic", PlotPoint::new(col_2, y - 8.0))
                .size(Measure::Screen(8.0))
                .fill(text_dim),
        );

        // 3. Cubic Loop
        plot.add_shape(
            Bezier::cubic(
                PlotPoint::new(col_3, y - 5.0),
                PlotPoint::new(col_3 - 15.0, y + 10.0),
                PlotPoint::new(col_3 + 15.0, y + 10.0),
                PlotPoint::new(col_3, y - 5.0),
            )
            .stroke(Stroke::new(palette.success, Measure::Screen(2.0))),
        );
        plot.add_shape(
            Label::new("Loop", PlotPoint::new(col_3, y - 8.0))
                .size(Measure::Screen(8.0))
                .fill(text_dim),
        );

        // 4. Thick Stroke
        plot.add_shape(
            Bezier::quadratic(
                PlotPoint::new(col_4 - 10.0, y),
                PlotPoint::new(col_4, y + 10.0),
                PlotPoint::new(col_4 + 10.0, y),
            )
            .stroke(Stroke::new(palette.danger, Measure::Screen(5.0))),
        );

        // =====================================================================
        //  10. AREA (Arbitrary)
        // =====================================================================
        let y = next_row(plot, "Area");

        let shape_pts = |cx: f64| {
            vec![
                PlotPoint::new(cx, y + 8.0),
                PlotPoint::new(cx + 8.0, y - 2.0),
                PlotPoint::new(cx, y + 2.0), // Concavity
                PlotPoint::new(cx - 8.0, y - 2.0),
            ]
        };

        // 1. Filled Concave
        plot.add_shape(Area::new(shape_pts(col_1)).fill(palette.primary));

        // 2. Stroked Concave
        plot.add_shape(
            Area::new(shape_pts(col_2)).stroke(Stroke::new(palette.primary, Measure::Screen(2.0))),
        );

        // 3. Fill + Stroke
        plot.add_shape(
            Area::new(shape_pts(col_3))
                .fill(palette.success)
                .stroke(Stroke::new(palette.text, Measure::Screen(1.0))),
        );

        // 4. Complex (Star-ish)
        let star_pts = vec![
            PlotPoint::new(col_4, y + 8.0),
            PlotPoint::new(col_4 + 2.0, y + 2.0),
            PlotPoint::new(col_4 + 8.0, y + 2.0),
            PlotPoint::new(col_4 + 3.0, y - 2.0),
            PlotPoint::new(col_4 + 5.0, y - 8.0),
            PlotPoint::new(col_4, y - 4.0),
            PlotPoint::new(col_4 - 5.0, y - 8.0),
            PlotPoint::new(col_4 - 3.0, y - 2.0),
            PlotPoint::new(col_4 - 8.0, y + 2.0),
            PlotPoint::new(col_4 - 2.0, y + 2.0),
        ];
        plot.add_shape(Area::new(star_pts).fill(palette.danger));
    }
}
