use iced::widget::Column;
use iced::{
    Color, Element, Font, Length, Padding, Theme,
    widget::{column, container, row, text},
};
use iced_aksel::{
    Axis, Chart, Measure, PlotPoint, State, Stroke,
    axis::{self, GridLine, TickLine, TickResult},
    plot::{Plot, PlotData},
    scale::Linear,
    shape::Polyline,
    style::Style,
};

pub fn main() -> iced::Result {
    iced::application(AxesShowcase::new, AxesShowcase::update, AxesShowcase::view)
        .title("Axes Styling Showcase")
        .theme(Theme::Dark)
        .antialiasing(true)
        .run()
}

struct AxesShowcase {
    // State
    minimal_state: State<&'static str, f64>,
    minimal_data: SineWave,
}

#[derive(Debug, Clone)]
pub enum Message {}

impl AxesShowcase {
    const X: &'static str = "x";
    const Y: &'static str = "y";

    fn new() -> (Self, iced::Task<Message>) {
        (
            Self {
                minimal_state: setup_minimal_axes(),
                minimal_data: SineWave::new(1.0, 0.8, 50),
            },
            iced::Task::none(),
        )
    }

    fn update(&mut self, _message: Message) -> iced::Task<Message> {
        iced::Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        // 1. Prepare the content "Atoms"
        // These are self-contained. They don't know they are in a row yet.

        let minimal_chart_panel = self.panel(
            "1. Minimal Layout",
            "Hidden Y-axis. No Grid.",
            Chart::new(&self.minimal_state)
                .plot_data(&self.minimal_data, Self::X, Self::Y)
                .style(Box::new(style_base)),
        );

        // 3. The Layout Structure
        row![minimal_chart_panel.width(Length::Fill),]
            .padding(20)
            .into()
    }

    fn panel<'a>(
        &self,
        title: &'a str,
        subtitle: &'a str,
        chart: Chart<'a, &'static str, f64, Message>,
    ) -> Column<'a, Message> {
        column![
            text(title).size(16).font(Font::MONOSPACE),
            text(subtitle).size(12),
            container(chart).width(Length::Fill).height(Length::Fill)
        ]
        .spacing(10)
        .width(Length::Fill)
    }
}

// -----------------------------------------------------------------------------
// 1. MINIMAL CONFIGURATION
// -----------------------------------------------------------------------------

fn setup_minimal_axes() -> State<&'static str, f64> {
    let mut state = State::new();

    // X-Axis: Standard look, no grid
    state.set_axis(
        AxesShowcase::X,
        Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom)
            .with_thickness(45.0)
            .with_cursor_formatter(|v| Some(format!("{:.0}", v))),
    );

    // Y-Axis: Invisible but active for scaling
    state.set_axis(
        AxesShowcase::Y,
        Axis::new(Linear::new(-1.2, 1.2), axis::Position::Left)
            .with_thickness(45.0)
            .with_cursor_formatter(|v| Some(format!("{:.0}", v))),
    );

    state
}
// -----------------------------------------------------------------------------
// STYLES
// -----------------------------------------------------------------------------

/// Base style that strictly adheres to the current Theme
fn style_base(theme: &Theme) -> Style {
    let style = iced_aksel::style::default(theme);

    style
}
// -----------------------------------------------------------------------------
// DATA GENERATION
// -----------------------------------------------------------------------------

struct SineWave {
    points: Vec<PlotPoint<f64>>,
}

impl SineWave {
    fn new(frequency: f64, amplitude: f64, count: usize) -> Self {
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
