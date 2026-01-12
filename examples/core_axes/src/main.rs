use iced::{
    Color, Element, Font, Length, Padding, Theme,
    widget::{column, container, pick_list, row, text},
};
use iced_aksel::{
    Axis, Chart, Measure, PlotPoint, State, Stroke,
    axis::{self, GridLine, Label, TickContext, TickLine, TickResult},
    plot::{Plot, PlotData},
    scale::Linear,
    shape::Polyline,
    style::DashStyle,
};

pub fn main() -> iced::Result {
    iced::application(AxesShowcase::new, AxesShowcase::update, AxesShowcase::view)
        .title("Axes Styling Showcase")
        .theme(AxesShowcase::theme)
        .antialiasing(true)
        .run()
}

struct AxesShowcase {
    theme: iced::Theme,
    minimal_state: State<&'static str, f64>,
    minimal_data: SineWave,

    engineering_state: State<&'static str, f64>,
    engineering_data: SineWave,

    custom_state: State<&'static str, f64>,
    custom_data: SineWave,
}

#[derive(Debug, Clone)]
pub enum Message {
    ThemeChanged(Theme),
}

impl AxesShowcase {
    const X: &'static str = "x";
    const Y: &'static str = "y";

    fn new() -> (Self, iced::Task<Message>) {
        (
            Self {
                theme: iced::Theme::Dark,
                minimal_state: setup_minimal_axes(),
                minimal_data: SineWave::new(1.0, 0.8, 50),

                engineering_state: setup_engineering_axes(),
                engineering_data: SineWave::new(2.5, 3.5, 100),

                custom_state: setup_custom_axes(),
                custom_data: SineWave::new(1.5, 0.8, 80),
            },
            iced::Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::ThemeChanged(theme) => self.theme = theme,
        }
        iced::Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        column![
            row![
                text("Theme: "),
                pick_list(iced::Theme::ALL, Some(&self.theme), Message::ThemeChanged)
            ]
            .padding(20),
            row![
                self.panel(
                    "1. Minimal Layout",
                    "Hidden Y-axis. No Grid.",
                    Chart::new(&self.minimal_state).plot_data(&self.minimal_data, Self::X, Self::Y)
                ),
                self.panel(
                    "2. Engineering Layout",
                    "Custom Ruler Ticks. Monospace.",
                    Chart::new(&self.engineering_state).plot_data(
                        &self.engineering_data,
                        Self::X,
                        Self::Y
                    )
                ),
                self.panel(
                    "3. Custom Placement",
                    "Top & Right Axes. Badges.",
                    Chart::new(&self.custom_state).plot_data(&self.custom_data, Self::X, Self::Y)
                ),
            ]
            .spacing(20)
            .padding(20)
        ]
        .spacing(20)
        .padding(20)
        .into()
    }

    fn theme(&self) -> iced::Theme {
        self.theme.clone()
    }

    fn panel<'a>(
        &self,
        title: &'a str,
        subtitle: &'a str,
        chart: Chart<'a, &'static str, f64, Message>,
    ) -> Element<'a, Message> {
        column![
            text(title).size(16).font(Font::MONOSPACE),
            text(subtitle)
                .size(12)
                .color(Color::from_rgb(0.6, 0.6, 0.6)),
            container(chart)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|t: &Theme| container::Style::default()
                    .background(t.extended_palette().background.weak.color)
                    .border(iced::Border {
                        radius: 8.0.into(),
                        color: Color::from_rgba(1.0, 1.0, 1.0, 0.05),
                        width: 1.0
                    }))
                .padding(Padding::new(15.))
        ]
        .spacing(10)
        .width(Length::Fill)
        .into()
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
            .without_grid()
            .with_marker_renderer(|ctx| Some(ctx.marker(format!("{:.0}", ctx.value)))),
    );

    // Y-Axis: Invisible but active for scaling
    state.set_axis(
        AxesShowcase::Y,
        Axis::new(Linear::new(-1.2, 1.2), axis::Position::Left).invisible(),
    );

    state
}

// -----------------------------------------------------------------------------
// 2. ENGINEERING CONFIGURATION
// -----------------------------------------------------------------------------

fn setup_engineering_axes() -> State<&'static str, f64> {
    let mut state = State::new();

    // Define a custom renderer for a technical "ruler" look
    let ruler_renderer = |ctx: TickContext<f64>| {
        if ctx.tick.level == 0 {
            TickResult {
                tick_line: Some(TickLine {
                    length: 8.0.into(),
                    width: 1.5.into(),
                    ..ctx.tickline()
                }),
                grid_line: Some(GridLine {
                    width: 1.0.into(),
                    ..ctx.gridline()
                }),
                label: Some(Label {
                    font: Some(Font::MONOSPACE),
                    ..ctx.label(format!("{:.1}", ctx.tick.value))
                }),
                ..Default::default()
            }
        } else {
            TickResult {
                tick_line: Some(TickLine {
                    length: 4.0.into(),
                    width: 1.0.into(),
                    ..ctx.tickline()
                }),
                grid_line: Some(GridLine {
                    width: 0.5.into(),
                    ..ctx.gridline()
                }),
                label: None,
                ..Default::default()
            }
        }
    };

    state.set_axis(
        AxesShowcase::Y,
        Axis::new(Linear::new(-4.0, 4.0), axis::Position::Left)
            .with_thickness(50.0)
            .with_tick_renderer(ruler_renderer),
    );

    state.set_axis(
        AxesShowcase::X,
        Axis::new(Linear::new(0.0, 100.0), axis::Position::Top)
            .with_thickness(35.0)
            .with_tick_renderer(ruler_renderer),
    );

    state
}

// -----------------------------------------------------------------------------
// 3. CUSTOM PLACEMENT & BADGES
// -----------------------------------------------------------------------------

fn setup_custom_axes() -> State<&'static str, f64> {
    let mut state = State::new();

    // X-Axis on TOP
    state.set_axis(
        AxesShowcase::X,
        Axis::new(Linear::new(0.0, 100.0), axis::Position::Top)
            .with_thickness(45.0)
            .style(|style| {
                // Override style to have dashed gridlines, without changing the renderer itself
                style.grid.dashed = Some(DashStyle {
                    gap_length: 5.0,
                    dash_length: 5.0,
                });
            })
            .with_marker_renderer(|ctx| Some(ctx.marker(format!("T: {:.1}s", ctx.value)))),
    );

    // Y-Axis on RIGHT
    state.set_axis(
        AxesShowcase::Y,
        Axis::new(Linear::new(-1.5, 1.5), axis::Position::Right)
            .with_thickness(55.0)
            .with_tick_renderer(|ctx| {
                let is_major = ctx.tick.level == 0;

                // Show grid only for major ticks
                let grid_line = is_major.then(|| GridLine {
                    width: 1.0.into(),
                    dashed: Some(DashStyle {
                        gap_length: 5.0,
                        dash_length: 5.0,
                    }),
                    ..ctx.gridline()
                });
                // Show labels only for major ticks
                let label = is_major.then(|| ctx.label(format!("{:.1}", ctx.tick.value)));
                TickResult {
                    grid_line,
                    label,
                    tick_line: Some(TickLine {
                        width: 1.0.into(),
                        length: 4.0.into(),
                        ..ctx.tickline()
                    }),
                    ..Default::default()
                }
            })
            .with_marker_renderer(|ctx| Some(ctx.marker(format!("{:.3}", ctx.value)))),
    );

    state
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
