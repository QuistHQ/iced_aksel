use iced::{Color, Element, Theme};
use iced_aksel::{
    Axis,
    Chart,
    Measure,
    Plot,
    PlotData,
    PlotPoint,
    State,
    Stroke,
    axis::Position,
    scale::Linear,
    shape::{Area, Label, Polygon}, // Import both
};

const X_AXIS: &str = "x";
const Y_AXIS: &str = "y";

pub fn main() -> iced::Result {
    iced::application(
        PolygonGallery::new,
        PolygonGallery::update,
        PolygonGallery::view,
    )
    .title("Polygon vs Zone")
    .antialiasing(true)
    .run()
}

struct PolygonGallery {
    state: State<&'static str, f64>,
}

#[derive(Debug, Clone)]
enum Message {}

impl PolygonGallery {
    fn new() -> (Self, iced::Task<Message>) {
        let mut state = State::new();
        state.set_axis(X_AXIS, Axis::new(Linear::new(0.0, 100.0), Position::Bottom));
        state.set_axis(Y_AXIS, Axis::new(Linear::new(0.0, 100.0), Position::Left));

        (Self { state }, iced::Task::none())
    }

    fn update(&mut self, _msg: Message) -> iced::Task<Message> {
        iced::Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        Chart::new(&self.state)
            .plot_data(self, X_AXIS, Y_AXIS)
            .into()
    }
}

impl PlotData<f64> for PolygonGallery {
    fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {
        let palette = theme.palette();

        // 1. Regular Polygons (Markers)
        // Defined by Center + Radius
        plot.add_shape(
            Label::new("Polygon (Markers)", PlotPoint::new(20.0, 90.0)).fill(palette.text),
        );

        // Triangle
        plot.add_shape(
            Polygon::new(PlotPoint::new(10.0, 80.0), Measure::Screen(15.0), 3)
                .fill(palette.primary),
        );

        // Diamond (Rotated Square)
        plot.add_shape(
            Polygon::new(PlotPoint::new(30.0, 80.0), Measure::Screen(15.0), 4)
                .rotation(45.0)
                .stroke(Stroke::new(palette.success, Measure::Screen(2.0))),
        );

        // Hexagon
        plot.add_shape(
            Polygon::new(PlotPoint::new(50.0, 80.0), Measure::Screen(15.0), 6).fill(palette.danger),
        );

        // 2. Zones (Arbitrary Areas)
        // Defined by a list of points
        plot.add_shape(Label::new("Zone (Areas)", PlotPoint::new(20.0, 50.0)).fill(palette.text));

        // A custom trapezoid shape
        let trapezoid = vec![
            PlotPoint::new(10.0, 20.0),
            PlotPoint::new(20.0, 40.0),
            PlotPoint::new(40.0, 40.0),
            PlotPoint::new(50.0, 20.0),
        ];

        plot.add_shape(
            Area::new(trapezoid)
                .fill(Color::from_rgba(0.2, 0.4, 0.8, 0.5))
                .stroke(Stroke::new(palette.primary, Measure::Screen(2.0))),
        );

        // A closed loop (Convex)
        let loop_shape = vec![
            PlotPoint::new(60.0, 20.0),
            PlotPoint::new(60.0, 40.0),
            PlotPoint::new(80.0, 40.0),
            PlotPoint::new(90.0, 30.0),
            PlotPoint::new(80.0, 20.0),
        ];

        plot.add_shape(Area::new(loop_shape).fill(Color::from_rgba(0.2, 0.8, 0.2, 0.5)));
    }
}
