use iced::{
    Border, Color, Element, Font, Length, Padding, Pixels, Shadow, Theme,
    widget::{column, container, row, text},
};
use iced_aksel::{
    Axis, Chart, Measure, PlotPoint, State, Stroke,
    axis::{self, GridLine, TickLine, TickResult},
    plot::{Plot, PlotData},
    scale::Linear,
    shape::Polyline,
    style::{AxisStyle, Style},
};

pub fn main() -> iced::Result {
    iced::application(AxesExample::new, AxesExample::update, AxesExample::view)
        .title("Axes Styles Showcase")
        .theme(Theme::Dark)
        .antialiasing(true)
        .run()
}

pub struct AxesExample {
    chart_minimal: State<&'static str, f64>,
    data_minimal: DataLayer,

    chart_science: State<&'static str, f64>,
    data_science: DataLayer,

    chart_finance: State<&'static str, f64>,
    data_finance: DataLayer,
}

#[derive(Debug, Clone)]
pub enum Message {}

impl AxesExample {
    const X_ID: &'static str = "x";
    const Y_ID: &'static str = "y";

    fn new() -> (Self, iced::Task<Message>) {
        let app = Self {
            chart_minimal: Self::configure_minimal(),
            data_minimal: DataLayer::new(1.0),

            chart_science: Self::configure_science(),
            data_science: DataLayer::new(2.5),

            chart_finance: Self::configure_finance(),
            data_finance: DataLayer::new(0.5),
        };
        (app, iced::Task::none())
    }

    fn update(&mut self, _message: Message) -> iced::Task<Message> {
        iced::Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        row![
            self.view_panel(
                "1. The Minimalist",
                "Clean look. Standard colors.",
                Chart::new(&self.chart_minimal)
                    .plot_data(&self.data_minimal, Self::X_ID, Self::Y_ID)
                    .style(Box::new(style_minimal)),
            ),
            self.view_panel(
                "2. The Scientific",
                "High density. Red cursor line.",
                Chart::new(&self.chart_science)
                    .plot_data(&self.data_science, Self::X_ID, Self::Y_ID)
                    .style(Box::new(style_science)),
            ),
            self.view_panel(
                "3. Manual Testing",
                "Comprehensive property list in code.",
                Chart::new(&self.chart_finance)
                    .plot_data(&self.data_finance, Self::X_ID, Self::Y_ID)
                    .style(Box::new(style_finance)),
            ),
        ]
        .spacing(20)
        .padding(20)
        .into()
    }

    fn view_panel<'a>(
        &self,
        title: &'a str,
        subtitle: &'a str,
        chart: Chart<'a, &'static str, f64, Message>,
    ) -> Element<'a, Message> {
        column![
            text(title).size(18).font(Font::MONOSPACE),
            text(subtitle)
                .size(12)
                .color(Color::from_rgb(0.7, 0.7, 0.7)),
            container(chart)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|t: &Theme| container::Style::default()
                    .background(t.extended_palette().background.weak.color)
                    .border(iced::Border {
                        radius: 5.0.into(),
                        ..Default::default()
                    }))
                .padding(10)
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(10)
        .into()
    }

    fn configure_minimal() -> State<&'static str, f64> {
        let mut state = State::new();
        let x_axis = Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom)
            .with_thickness(30.0)
            .without_grid();
        let y_axis = Axis::new(Linear::new(-50.0, 50.0), axis::Position::Left)
            .with_thickness(40.0)
            .without_grid();
        state.set_axis(
            Self::X_ID,
            x_axis.with_cursor_formatter(|v| Some(format!("{:.0}", v))),
        );
        state.set_axis(
            Self::Y_ID,
            y_axis.with_cursor_formatter(|v| Some(format!("{:.0}", v))),
        );
        state
    }

    fn configure_science() -> State<&'static str, f64> {
        let mut state = State::new();
        let renderer = |ctx: axis::TickContext<f64>| {
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
            Self::X_ID,
            Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom)
                .with_thickness(40.0)
                .with_tick_renderer(renderer),
        );
        state.set_axis(
            Self::Y_ID,
            Axis::new(Linear::new(-50.0, 50.0), axis::Position::Left)
                .with_thickness(50.0)
                .with_tick_renderer(renderer),
        );
        state
    }

    fn configure_finance() -> State<&'static str, f64> {
        let mut state = State::new();
        let currency_fmt = |val: f64| Some(format!("${:.2}", val));
        state.set_axis(
            Self::X_ID,
            Axis::new(Linear::new(0.0, 100.0), axis::Position::Top)
                .with_thickness(35.0)
                .with_cursor_formatter(|v| Some(format!("T: {:.0}", v))),
        );
        state.set_axis(
            Self::Y_ID,
            Axis::new(Linear::new(-50.0, 50.0), axis::Position::Right)
                .with_thickness(60.0)
                .with_cursor_formatter(currency_fmt)
                .with_tick_renderer(|ctx| TickResult {
                    grid_line: Some(GridLine::default()),
                    label: Some(format!("${:.0}", ctx.tick.value)),
                    ..Default::default()
                }),
        );
        state
    }
}

fn style_minimal(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    let mut style = iced_aksel::style::default(theme);
    style.axis.cursor.color = palette.primary.base.color;
    style.axis.text_offset = Pixels(10.0);
    style.axis.label.size = Pixels(10.0);
    style.axis.cursor.badge.background = palette.primary.strong.color;
    style
}

fn style_science(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    let mut style = iced_aksel::style::default(theme);
    style.axis.cursor.color = Color::from_rgb(1.0, 0.0, 0.0);
    style.axis.ticks.color = palette.background.strong.text;
    style.axis.label.font = Font::MONOSPACE;
    style.axis.cursor.width = Pixels(1.0);
    style
}

// Full property breakdown for manual testing
fn style_finance(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    let mut style = iced_aksel::style::default(theme);

    // --- AXIS ---
    style.axis.text_offset = Pixels(15.0);

    // Axis Ticks
    style.axis.ticks.color = Color::from_rgb(0.5, 0.8, 0.5); // Green ticks
    style.axis.ticks.width = Pixels(1.0);

    // Axis Labels (static)
    style.axis.label.color = Color::from_rgb(0.5, 0.8, 0.5);
    style.axis.label.size = Pixels(12.0);
    // style.axis.label.font = ...

    // --- CURSOR (Axis) ---
    // Line
    style.axis.cursor.color = Color::from_rgb(0.0, 0.6, 0.0);
    style.axis.cursor.width = Pixels(1.0);
    style.axis.cursor.line_gap = Pixels(5.0);

    // Badge Text
    style.axis.cursor.text.color = Color::WHITE;
    style.axis.cursor.text.size = Pixels(12.0);

    // Badge Container
    style.axis.cursor.badge.background = Color::from_rgb(0.0, 0.6, 0.0);
    style.axis.cursor.badge.padding = Padding::new(2.0);
    style.axis.cursor.badge.border = Border {
        color: Color::WHITE,
        width: 1.0,
        radius: 4.0.into(),
    };
    style.axis.cursor.badge.shadow = Shadow::default();

    // --- GRID ---
    style.grid.color = Color::from_rgba(1.0, 1.0, 1.0, 0.1);
    style.grid.width = Pixels(1.0);

    // --- PLOT CURSOR ---
    style.plot_cursor.color = Color::from_rgb(0.0, 0.6, 0.0);
    style.plot_cursor.width = Pixels(1.0);

    style
}

struct DataLayer {
    points: Vec<PlotPoint<f64>>,
}
impl DataLayer {
    fn new(frequency: f64) -> Self {
        let points = (0..=100)
            .map(|i| {
                let x = i as f64;
                let y = (x * 0.1 * frequency).sin() * 40.0;
                PlotPoint::new(x, y)
            })
            .collect();
        Self { points }
    }
}
impl PlotData<f64> for DataLayer {
    fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {
        plot.add_shape(
            Polyline::new(self.points.clone())
                .stroke(Stroke::new(theme.palette().primary, Measure::Screen(2.0))),
        );
    }
}
