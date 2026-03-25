//! Showcase of the core shapes available in iced_aksel.
use iced::Degrees;
use iced::alignment::{Horizontal, Vertical};
use iced::widget::text::Wrapping;
use iced::{
    Color, Element, Length, Size, Theme,
    widget::{column, container},
};
use iced_aksel::{
    Axis, Cached, Chart, Measure, PlotPoint, Quality, State, Stroke,
    axis::{self},
    plot::{Plot, PlotData},
    scale::Linear,
    shape::{
        Arc, Area, Bezier, Bounds, Ellipse, Label, Line, Polygon, Polyline, Rectangle, Spline,
        Triangle,
    },
    stroke::StrokeStyle,
};

// -----------------------------------------------------------------------------
// 1. Application Entry
// -----------------------------------------------------------------------------
fn main() -> iced::Result {
    iced::application(GridTestApp::new, GridTestApp::update, GridTestApp::view)
        .theme(Theme::Dark)
        .antialiasing(true)
        .run()
}

// -----------------------------------------------------------------------------
// 2. Application State
// -----------------------------------------------------------------------------
struct GridTestApp {
    chart_state: State<&'static str, f64>,
    data: Cached<GridData>,
}

#[derive(Debug, Clone)]
enum Message {}

impl GridTestApp {
    const X: &'static str = "x";
    const Y: &'static str = "y";

    fn new() -> (Self, iced::Task<Message>) {
        let mut state = State::new();

        // 0-400 range allows for a 4x4 grid where each cell is 100x100 units
        let x_scale = Linear::new(0.0, 400.0);
        let y_scale = Linear::new(0.0, 400.0); // 0 at bottom

        let x_axis = Axis::new(x_scale, axis::Position::Bottom);
        let y_axis = Axis::new(y_scale, axis::Position::Left);

        state.set_axis(Self::X, x_axis);
        state.set_axis(Self::Y, y_axis);

        (
            Self {
                chart_state: state,
                data: Cached::new(GridData {}),
            },
            iced::Task::none(),
        )
    }

    const fn update(&mut self, _message: Message) {}

    fn view(&self) -> Element<'_, Message> {
        let chart = Chart::new(&self.chart_state)
            .plot_data(&self.data, Self::X, Self::Y)
            .quality(Quality::High);

        column![container(chart).width(Length::Fill).height(Length::Fill)]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

// -----------------------------------------------------------------------------
// 3. The Grid Data
// -----------------------------------------------------------------------------
struct GridData {}

impl GridData {
    // Helper to get center of a grid cell
    // Rows: 0 (Bottom) to 3 (Top). Cols: 0 (Left) to 3 (Right)
    const fn cell(&self, row: usize, col: usize) -> PlotPoint {
        PlotPoint::new(
            (col as f64).mul_add(100.0, 50.0),
            (row as f64).mul_add(100.0, 50.0),
        )
    }
}

impl PlotData<f64, Message> for GridData {
    fn draw(&self, plot: &mut Plot<f64, Message>, _theme: &Theme) {
        // =====================================================================
        // ROW 0: BASIC GEOMETRY (Rect, Triangle, Circle, Poly)
        // =====================================================================

        // Col 0: Rectangle (Filled)
        let c = self.cell(0, 0);
        plot.render(
            Rectangle::centered(c, Measure::Screen(40.0), Measure::Screen(30.0))
                .fill(Color::from_rgb(0.2, 0.4, 0.8)),
        );

        // Col 1: Triangle (Rotated & Stroked)
        let c = self.cell(0, 1);
        plot.render(
            Triangle::centered(c, Measure::Screen(40.0), Measure::Screen(40.0)).stroke(
                Stroke::new(Color::from_rgb(0.2, 0.8, 0.2), Measure::Screen(3.0)),
            ),
        );
        // Add a center dot to verify centering
        plot.render(Ellipse::circle(c, Measure::Screen(2.0)).fill(Color::WHITE));

        // Col 2: Circle (Dashed Stroke)
        let c = self.cell(0, 2);
        plot.render(
            Ellipse::circle(c, Measure::Screen(25.0)).stroke(Stroke::with_style(
                Color::from_rgb(0.8, 0.2, 0.2),
                Measure::Screen(3.0),
                StrokeStyle::Dashed {
                    dash: 5.0,
                    gap: 5.0,
                },
            )),
        );

        // Col 3: Polygon (Hexagon, Rotated)
        let c = self.cell(0, 3);
        plot.render(
            Polygon::new(c, Measure::Screen(30.0), 6)
                .rotation(Degrees(30.0)) // Flat top
                .fill(Color::from_rgb(0.8, 0.8, 0.2))
                .stroke(Stroke::new(Color::BLACK, Measure::Screen(2.0))),
        );

        // =====================================================================
        // ROW 1: LINES & CONNECTORS
        // =====================================================================

        // Col 0: Simple Line Segment
        let c = self.cell(1, 0);
        plot.render(Line::new(
            PlotPoint::new(c.x - 30.0, c.y - 30.0),
            PlotPoint::new(c.x + 30.0, c.y + 30.0),
            Stroke::new(Color::WHITE, Measure::Screen(2.0)),
        ));

        // Col 1: Line with Arrows (Start & End)
        let c = self.cell(1, 1);
        plot.render(
            Line::new(
                PlotPoint::new(c.x - 30.0, c.y),
                PlotPoint::new(c.x + 30.0, c.y),
                Stroke::new(Color::from_rgb(0.4, 0.8, 1.0), Measure::Screen(3.0)),
            )
            .arrows(true)
            .arrow_size(4.0),
        );

        // Col 2: Infinite Line (Clipped)
        let c = self.cell(1, 2);
        plot.render(
            Line::new(
                PlotPoint::new(c.x, c.y - 10.0),
                PlotPoint::new(c.x + 20.0, c.y + 10.0),
                Stroke::with_style(
                    Color::from_rgba(1.0, 1.0, 1.0, 0.5),
                    Measure::Screen(1.0),
                    StrokeStyle::Dotted { gap: 3.0 },
                ),
            )
            .infinite(),
        );

        // Col 3: Polyline (M-Shape)
        let c = self.cell(1, 3);
        plot.render(
            Polyline::new(
                vec![
                    PlotPoint::new(c.x - 40.0, c.y - 20.0),
                    PlotPoint::new(c.x - 10.0, c.y + 20.0),
                    PlotPoint::new(c.x + 10.0, c.y - 20.0),
                    PlotPoint::new(c.x + 40.0, c.y + 20.0),
                ],
                Stroke::new(Color::from_rgb(1.0, 0.5, 0.8), Measure::Screen(2.0)),
            )
            .arrow_start(true)
            .arrow_end(true),
        );

        // =====================================================================
        // ROW 2: CURVES & COMPLEX SHAPES
        // =====================================================================

        // Col 0: Arc (Donut)
        let c = self.cell(2, 0);
        plot.render(
            Arc::new(c, Measure::Screen(30.0), 0.0, 4.71) // 270 deg
                .inner_radius(Measure::Screen(15.0))
                .fill(Color::from_rgb(1.0, 0.5, 0.0))
                .stroke(Stroke::new(Color::BLACK, Measure::Screen(1.0))),
        );

        // Col 1: Bezier (Quadratic)
        let c = self.cell(2, 1);
        plot.render(Bezier::quadratic(
            PlotPoint::new(c.x - 30.0, c.y - 20.0),
            PlotPoint::new(c.x, c.y + 50.0), // Control
            PlotPoint::new(c.x + 30.0, c.y - 20.0),
            Stroke::new(Color::from_rgb(0.5, 1.0, 0.5), Measure::Screen(2.0)),
        ));

        // Col 2: Spline (Through Points)
        let c = self.cell(2, 2);
        let pts = vec![
            PlotPoint::new(c.x - 40.0, c.y),
            PlotPoint::new(c.x - 20.0, c.y + 30.0),
            PlotPoint::new(c.x + 20.0, c.y - 30.0),
            PlotPoint::new(c.x + 40.0, c.y),
        ];
        plot.render(Spline::new(
            pts.clone(),
            Stroke::new(Color::from_rgb(0.5, 0.5, 1.0), Measure::Screen(2.0)),
        ));
        // Dots to verify fit
        for p in pts {
            plot.render(Ellipse::circle(p, Measure::Screen(3.0)).fill(Color::WHITE));
        }

        // Col 3: Area (Filled Blob)
        let c = self.cell(2, 3);
        plot.render(
            Area::new(vec![
                PlotPoint::new(c.x - 30.0, c.y - 30.0),
                PlotPoint::new(c.x - 10.0, c.y + 20.0),
                PlotPoint::new(c.x + 30.0, c.y + 30.0),
                PlotPoint::new(c.x + 10.0, c.y - 20.0),
            ])
            .fill(Color::from_rgba(0.8, 0.2, 0.8, 0.5))
            .stroke(Stroke::new(Color::WHITE, Measure::Screen(1.0))),
        );

        // =====================================================================
        // ROW 3: TEXT & LABELS
        // =====================================================================

        // Col 0: Plain Text (Centered)
        let c = self.cell(3, 0);
        plot.render(
            Label::new("Center", c)
                .align(Horizontal::Center, Vertical::Center)
                .fill(Color::WHITE)
                .size(Measure::Screen(16.0)),
        );
        plot.render(Ellipse::circle(c, Measure::Screen(2.0)).fill(Color::from_rgb(1.0, 0.0, 0.0)));

        // Col 1: Rotated Text
        let c = self.cell(3, 1);
        plot.render(
            Label::new("Rotated 45", c)
                .rotation(0.785)
                .align(Horizontal::Center, Vertical::Center)
                .fill(Color::from_rgb(0.8, 0.8, 0.8))
                .size(Measure::Screen(14.0)),
        );
        // Crosshair to check rotation center
        plot.render(Line::new(
            PlotPoint::new(c.x - 10.0, c.y),
            PlotPoint::new(c.x + 10.0, c.y),
            Stroke::new(Color::from_rgb(1.0, 0.0, 0.0), Measure::Screen(1.0)),
        ));
        plot.render(Line::new(
            PlotPoint::new(c.x, c.y - 10.0),
            PlotPoint::new(c.x, c.y + 10.0),
            Stroke::new(Color::from_rgb(1.0, 0.0, 0.0), Measure::Screen(1.0)),
        ));

        // Col 2: Wrapped Text
        let c = self.cell(3, 2);
        // 1. Define the Box Dimensions
        let box_width = 40.0;
        let box_height = 40.0; // The red box size

        // 2. Define the Top-Left Anchor (Highest Y in Plot Space)
        let top_left = PlotPoint::new(c.x, c.y + box_height);

        // 3. Draw the Label anchored at Top-Left
        plot.render(
            Label::new("This text wraps inside the red box.", top_left)
                // Constrain width, but height can be whatever
                .bounds(Bounds::Plot(Size::new(box_width, 150.0)))
                .wrapping(Wrapping::Word)
                .align(Horizontal::Left, Vertical::Top) // Text hangs down from Top-Left
                .size(Measure::Screen(12.0))
                .fill(Color::WHITE),
        );

        // 4. Draw the Red Box (checking corners)
        // We draw from Bottom-Left (c) to Top-Right (c + size)
        plot.render(
            Rectangle::corners(c, PlotPoint::new(c.x + box_width, c.y + box_height)).stroke(
                Stroke::new(Color::from_rgb(1.0, 0.0, 0.0), Measure::Screen(1.0)),
            ),
        );

        // Col 3: Plot-Scaled Text (Zoom Test)
        let c = self.cell(3, 3);
        plot.render(
            Label::new("Scales with Zoom", c)
                .size(Measure::Plot(5.0)) // Height in plot units
                .align(Horizontal::Center, Vertical::Center)
                .fill(Color::from_rgb(0.5, 1.0, 0.5)),
        );
    }
}
