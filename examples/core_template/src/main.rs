//! Minimal iced_aksel plot example.
use iced::{
    Element, Length, Theme,
    widget::{column, container},
};
use iced_aksel::{
    Axis, Chart, State,
    axis::{self},
    plot::{Plot, PlotData},
    scale::Linear,
};

// -----------------------------------------------------------------------------
// 1. Application Entry
// -----------------------------------------------------------------------------
fn main() -> iced::Result {
    iced::application(TemplateApp::new, TemplateApp::update, TemplateApp::view)
        .theme(Theme::Light)
        .run()
}

// -----------------------------------------------------------------------------
// 2. Application State
// -----------------------------------------------------------------------------
struct TemplateApp {
    chart_state: State<&'static str, f64>,
    data: MyData, // <--- Your custom data struct
}

#[derive(Debug, Clone)]
enum Message {}

impl TemplateApp {
    const X: &'static str = "x";
    const Y: &'static str = "y";

    fn new() -> (Self, iced::Task<Message>) {
        let mut state = State::new();

        // -- Setup Axes --
        // Customizable: Change Linear to Logarithmic or adjust ranges here.
        let x_scale = Linear::new(0.0, 100.0);
        let y_scale = Linear::new(0.0, 100.0);

        let x_axis = Axis::new(x_scale, axis::Position::Bottom);
        let y_axis = Axis::new(y_scale, axis::Position::Left);

        state.set_axis(Self::X, x_axis);
        state.set_axis(Self::Y, y_axis);

        (
            Self {
                chart_state: state,
                data: MyData {},
            },
            iced::Task::none(),
        )
    }

    const fn update(&mut self, message: Message) {
        match message {}
    }

    fn view(&self) -> Element<'_, Message> {
        let chart = Chart::new(&self.chart_state).plot_data(&self.data, Self::X, Self::Y);

        column![container(chart)]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

// -----------------------------------------------------------------------------
// 3. Data & Drawing Logic
// -----------------------------------------------------------------------------

// This is a placeholder for the custom struct you can use to try out the charting capabilities.
struct MyData {}

impl MyData {}

impl PlotData<f64> for MyData {
    fn draw(&self, _plot: &mut Plot<f64>, _theme: &Theme) {}
}
