use iced::{
    Color, Length, Theme,
    alignment::Horizontal,
    widget::{column, container, text},
};
use iced_aksel::{
    Axis, Chart, Measure, PlotPoint, State, Stroke,
    axis::{self, Label, TickContext, TickResult},
    plot::{Plot, PlotData},
    scale::{Linear, Logarithmic},
    shape::{Ellipse, Polyline},
};

// -----------------------------------------------------------------------------
// Application Entry
// -----------------------------------------------------------------------------

pub fn main() -> iced::Result {
    iced::application(
        ScalesExample::new,
        ScalesExample::update,
        ScalesExample::view,
    )
    .title("Multi-Axis: Linear vs Logarithmic")
    .antialiasing(true)
    .run()
}

// -----------------------------------------------------------------------------
// Application State
// -----------------------------------------------------------------------------

pub struct ScalesExample {
    chart_state: State<&'static str, f64>,

    // We wrap the data to give them distinct colors
    linear_representation: StyleableData,
    log_representation: StyleableData,
}

#[derive(Debug, Clone)]
pub enum Message {}

impl ScalesExample {
    const AXIS_X: &'static str = "x";
    const AXIS_Y_LIN: &'static str = "y_linear";
    const AXIS_Y_LOG: &'static str = "y_log";

    pub fn new() -> (Self, iced::Task<Message>) {
        // Range: 0 to 100 on X
        let x_min = 0.0;
        let x_max = 100.0;

        // Range: 1 to 100,000 on Y
        let y_min = 1.0;
        let y_max = 100_000.0;

        let mut chart_state = State::new();

        // 1. Setup Bottom X Axis
        chart_state.set_axis(
            Self::AXIS_X,
            Axis::new(Linear::new(x_min, x_max), axis::Position::Bottom)
                .with_tick_renderer(x_axis_tick_renderer)
                .skip_overlapping_labels(6.0),
        );

        // 2. Setup Left Linear Axis (Blue)
        chart_state.set_axis(
            Self::AXIS_Y_LIN,
            Axis::new(Linear::new(y_min, y_max), axis::Position::Left)
                .with_tick_renderer(linear_axis_tick_renderer),
        );

        // 3. Setup Left Log Axis (Red) - Added to the same side!
        chart_state.set_axis(
            Self::AXIS_Y_LOG,
            Axis::new(Logarithmic::new(10.0, y_min, y_max), axis::Position::Left)
                .with_tick_renderer(log_axis_tick_renderer),
        );

        // Prepare data (Linear growth)
        let (points, markers) = generate_linear_data();

        (
            Self {
                chart_state,
                // Assign Blue to Linear Axis representation
                linear_representation: StyleableData {
                    line: points.clone(),
                    markers: markers.clone(),
                    color: |t| t.palette().primary,
                },
                // Assign Red to Log Axis representation
                log_representation: StyleableData {
                    line: points,
                    markers,
                    color: |t| t.palette().danger,
                },
            },
            iced::Task::none(),
        )
    }

    pub fn update(&mut self, _message: Message) -> iced::Task<Message> {
        iced::Task::none()
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        // We create the chart and attach the same data twice,
        // mapping it to different Y-axes.
        let chart = Chart::new(&self.chart_state)
            // Draw Blue line against Linear Axis
            .plot_data(&self.linear_representation, Self::AXIS_X, Self::AXIS_Y_LIN)
            // Draw Red line against Log Axis
            .plot_data(&self.log_representation, Self::AXIS_X, Self::AXIS_Y_LOG);

        // Layout construction
        let header = column![
            text("Multi-Axis Comparison").size(24),
            text("Both lines represent the exact same linear data (y = 1000x).")
                .size(14)
                .color(Color::from_rgb(0.5, 0.5, 0.5)),
        ]
        .spacing(5);

        let chart_container = container(chart)
            .height(Length::Fill)
            .width(Length::Fill)
            .padding(20)
            .style(|t: &Theme| {
                container::Style::default().border(iced::border::color(t.palette().text).width(1.0))
            });

        column![header, chart_container, self.view_legend()]
            .spacing(20)
            .padding(20)
            .into()
    }

    fn view_legend(&self) -> iced::Element<'_, Message> {
        container(
            text("Blue: Linear Axis (Outer Left)  |  Red: Logarithmic Axis (Inner Left)")
                .size(14)
                .color(Color::from_rgb(0.5, 0.5, 0.5)),
        )
        .width(Length::Fill)
        .align_x(Horizontal::Center)
        .into()
    }
}

// -----------------------------------------------------------------------------
// Data Layer
// -----------------------------------------------------------------------------

struct StyleableData {
    line: Vec<PlotPoint<f64>>,
    markers: Vec<PlotPoint<f64>>,
    color: fn(&Theme) -> Color,
}

impl PlotData<f64> for StyleableData {
    fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {
        let color = (self.color)(theme);

        // Draw the line
        plot.add_shape(
            Polyline::new(self.line.clone()).stroke(Stroke::new(color, Measure::Screen(2.5))),
        );

        // Draw markers
        for point in &self.markers {
            plot.add_shape(
                Ellipse::new(*point, Measure::Screen(4.0), Measure::Screen(4.0)).fill(color),
            );
        }
    }
}

fn generate_linear_data() -> (Vec<PlotPoint<f64>>, Vec<PlotPoint<f64>>) {
    // Generate linear data: y = 1000 * x
    // We start x at 1.0 to ensure y >= 1000.0, avoiding Log(0) issues on the log axis.

    let line = (1..=100)
        .map(|i| {
            let x = i as f64;
            let y = 1000.0 * x;
            PlotPoint::new(x, y)
        })
        .collect();

    let markers = (0..=10)
        .map(|i| {
            let x = (i * 10) as f64;
            // Handle 0 explicitly for safety, though the loop above skips it.
            // If x is 0, we can't plot it on log, so we plot min value or skip.
            let safe_x = if x < 1.0 { 1.0 } else { x };
            let y = 1000.0 * safe_x;
            PlotPoint::new(safe_x, y)
        })
        .collect();

    (line, markers)
}

// -----------------------------------------------------------------------------
// Tick Renderers
// -----------------------------------------------------------------------------

fn x_axis_tick_renderer(ctx: TickContext<f64, Theme>) -> TickResult {
    if ctx.tick.level != 0 {
        return TickResult::default();
    }

    TickResult {
        label: Some(ctx.label(format!("{:.0}", ctx.tick.value))),
        tick_line: Some(ctx.tickline()),
        ..Default::default()
    }
}

// Blue ticks for Linear Scale
fn linear_axis_tick_renderer(ctx: TickContext<f64, Theme>) -> TickResult {
    if ctx.tick.level != 0 {
        return TickResult::default();
    }

    // Initialize values from context
    let mut label = ctx.label(format!("{:.0}", ctx.tick.value));
    let mut tick_line = ctx.tickline();

    // Overwrite colors (Primary Blue)
    tick_line.color = ctx.theme.palette().primary;
    label.color = ctx.theme.palette().primary;

    TickResult {
        label: Some(label),
        tick_line: Some(tick_line),
        ..Default::default()
    }
}

// Red ticks for Log Scale
fn log_axis_tick_renderer(ctx: TickContext<f64, Theme>) -> TickResult {
    // 1. Early exit for details too small to matter
    if ctx.tick.level > 1 {
        return TickResult::default();
    }

    // 2. Prepare the line (We need this for both Level 0 and Level 1)
    let mut tick_line = ctx.tickline();
    tick_line.color = ctx.theme.palette().danger;

    // 3. Prepare the label (ONLY for Level 0)
    let label = if ctx.tick.level == 0 {
        let mut label = ctx.label(format!("{:.0}", ctx.tick.value));
        label.color = ctx.theme.palette().danger;
        Some(label)
    } else {
        // Optional: You can make minor tick lines shorter or thinner here if you want
        tick_line.width = 1.into();
        tick_line.length = 4.into();
        None
    };

    TickResult {
        label,
        tick_line: Some(tick_line),
        ..Default::default()
    }
}
