//! Exports a chart as an SVG document, without opening a window.
use iced::{
    Border, Color, Padding, Shadow, Size, Theme,
    alignment::{Horizontal, Vertical},
};
use iced_aksel::{
    Axis, Chart, Measure, PlotPoint, Radii, State, Stroke,
    axis::{self, LabelBadge, TickContext, TickResult},
    plot::{Plot, PlotData},
    scale::Linear,
    shape::{Ellipse, Label, Polyline, Rectangle},
    stroke::StrokeStyle,
    style::DashStyle,
    svg,
};

const X: &str = "x";
const Y: &str = "y";

/// The chart never runs in an application, so it doesn't produce any messages.
#[derive(Debug, Clone)]
enum Message {}

fn main() -> std::io::Result<()> {
    let mut state: State<&'static str, f64> = State::new();
    state.set_axis(
        X,
        Axis::new(Linear::new(0.0, 10.0), axis::Position::Bottom).with_tick_renderer(ticks),
    );
    state.set_axis(
        Y,
        Axis::new(Linear::new(-1.5, 1.5), axis::Position::Left).with_tick_renderer(ticks),
    );

    let data = Wave;
    let chart: Chart<_, _, Message> = Chart::new(&state)
        .plot_data(&data, X, Y)
        .padding(Padding::new(16.0));

    // The size is the space available to the chart, just like the bounds of a window would be.
    let document = chart.to_svg(
        Size::new(800.0, 500.0),
        &Theme::Light,
        svg::Options::default().background(Color::WHITE),
    );

    std::fs::write("chart.svg", document)?;
    println!("Wrote chart.svg");

    Ok(())
}

/// Labels the major ticks and draws dashed grid lines.
fn ticks(ctx: TickContext<'_, f64>) -> TickResult {
    if ctx.tick.level != 0 {
        return TickResult::default();
    }

    let mut grid_line = ctx.gridline();
    grid_line.dashed = Some(DashStyle::new(4.0, 4.0));

    TickResult {
        tick_line: Some(ctx.tickline()),
        grid_line: Some(grid_line),
        label: Some(ctx.label(format!("{:.1}", ctx.tick.value))),
        label_badge: Some(LabelBadge {
            background: ctx.theme.extended_palette().background.weak.color,
            border: Border::default().rounded(4.0),
            shadow: Shadow::default(),
        }),
        label_priority: None,
    }
}

struct Wave;

impl PlotData<f64, Message> for Wave {
    fn draw(&self, plot: &mut Plot<f64, Message>, theme: &Theme) {
        let palette = theme.palette();

        // Bars
        for i in 0..10 {
            let x = f64::from(i) + 0.5;
            plot.render(
                Rectangle::corners(
                    PlotPoint::new(x - 0.3, 0.0),
                    PlotPoint::new(x + 0.3, x.cos()),
                )
                .fill(palette.primary.scale_alpha(0.3)),
            );
        }

        // Line with markers
        let points: Vec<_> = (0..=100)
            .map(|i| f64::from(i) / 10.0)
            .map(|x| PlotPoint::new(x, x.sin()))
            .collect();

        plot.render(Polyline::new(
            points.clone(),
            Stroke::new(palette.danger, Measure::Screen(2.0)),
        ));

        for point in points.into_iter().step_by(10) {
            plot.render(
                Ellipse::new(point, Radii::uniform(Measure::Screen(4.0)))
                    .fill(palette.background)
                    .stroke(Stroke::new(palette.danger, Measure::Screen(2.0))),
            );
        }

        // Threshold
        plot.render(Polyline::new(
            vec![PlotPoint::new(0.0, 1.0), PlotPoint::new(10.0, 1.0)],
            Stroke::with_style(
                palette.text,
                Measure::Screen(1.0),
                StrokeStyle::Dashed {
                    dash: 6.0,
                    gap: 4.0,
                },
            ),
        ));

        plot.render(
            Label::new("sin(x) ≤ 1", PlotPoint::new(9.8, 1.15))
                .fill(palette.text)
                .size(Measure::Screen(14.0))
                .align(Horizontal::Right, Vertical::Center),
        );
    }
}
