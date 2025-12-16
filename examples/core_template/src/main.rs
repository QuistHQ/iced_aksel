use iced::{
    Color, Element, Length, Theme,
    widget::{column, container},
};
use iced_aksel::{
    Axis, Chart, Measure, PlotPoint, State,
    axis::{self},
    plot::{Plot, PlotData},
    scale::Linear,
    shape::Rectangle,
};
use rand::Rng;

// -----------------------------------------------------------------------------
// Application Entry
// -----------------------------------------------------------------------------

pub fn main() -> iced::Result {
    iced::application(
        SimpleExample::new,
        SimpleExample::update,
        SimpleExample::view,
    )
    .theme(Theme::Dark)
    .antialiasing(true)
    .run()
}

// -----------------------------------------------------------------------------
// Application State
// -----------------------------------------------------------------------------

struct SimpleExample {
    chart_state: State<&'static str, f64>,
    stars: StarField,
}

#[derive(Debug, Clone)]
enum Message {
    Scrolled(iced::Point, iced::mouse::ScrollDelta),
}

impl SimpleExample {
    const X_ID: &'static str = "x";
    const Y_ID: &'static str = "y";

    const WORLD_SIZE: f64 = 100.0;
    // Maximum allowable span (World Diameter: -100 to 100 = 200)
    const MAX_SPAN: f64 = 200.0;

    fn new() -> (Self, iced::Task<Message>) {
        let mut chart_state = State::new();

        // 1. Setup Axes (Invisible)
        chart_state.set_axis(
            Self::X_ID,
            Axis::new(
                Linear::new(-Self::WORLD_SIZE, Self::WORLD_SIZE),
                axis::Position::Bottom,
            )
            .invisible(),
        );

        chart_state.set_axis(
            Self::Y_ID,
            Axis::new(
                Linear::new(-Self::WORLD_SIZE, Self::WORLD_SIZE),
                axis::Position::Left,
            )
            .invisible(),
        );

        // 2. Generate Data (Static)
        let stars = StarField::generate(1000, Self::WORLD_SIZE);

        (Self { chart_state, stars }, iced::Task::none())
    }

    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::Scrolled(cursor_norm, delta) => {
                let delta_y = match delta {
                    iced::mouse::ScrollDelta::Lines { y, .. }
                    | iced::mouse::ScrollDelta::Pixels { y, .. } => y,
                };

                // Simple Zoom Logic
                let factor = if delta_y > 0.0 { 1.10 } else { 0.90 };

                // Get current zoom level
                let current_span = self
                    .chart_state
                    .axis_opt(&Self::X_ID)
                    .map(|ax| {
                        let (min, max) = ax.domain();
                        max - min
                    })
                    .unwrap_or(0.0);

                // Check if the NEXT zoom level would exceed the world size
                let next_span = current_span / factor;
                let should_clamp = factor < 1.0 && next_span >= Self::MAX_SPAN;

                if should_clamp {
                    // Snap to exact world bounds
                    if let Some(axis) = self.chart_state.axis_mut_opt(&Self::X_ID) {
                        axis.set_domain(-Self::WORLD_SIZE, Self::WORLD_SIZE);
                    }
                    if let Some(axis) = self.chart_state.axis_mut_opt(&Self::Y_ID) {
                        axis.set_domain(-Self::WORLD_SIZE, Self::WORLD_SIZE);
                    }
                } else {
                    // Standard Zoom
                    if let Some(axis) = self.chart_state.axis_mut_opt(&Self::X_ID) {
                        axis.zoom(factor as f32, Some(cursor_norm.x));
                    }
                    if let Some(axis) = self.chart_state.axis_mut_opt(&Self::Y_ID) {
                        axis.zoom(factor as f32, Some(cursor_norm.y));
                    }
                }
            }
        }
        iced::Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let chart = Chart::new(&self.chart_state)
            .plot_data(&self.stars, Self::X_ID, Self::Y_ID)
            .on_scroll(Message::Scrolled);

        column![container(chart).width(Length::Fill).height(Length::Fill)].into()
    }
}

// -----------------------------------------------------------------------------
// Data Model
// -----------------------------------------------------------------------------

struct Star {
    position: PlotPoint,
    color: Color,
}

struct StarField {
    objects: Vec<Star>,
}

impl StarField {
    fn generate(count: usize, range: f64) -> Self {
        let mut rng = rand::rng();
        let mut objects = Vec::with_capacity(count);

        for _ in 0..count {
            objects.push(Star {
                position: PlotPoint {
                    x: rng.random_range(-range..range),
                    y: rng.random_range(-range..range),
                },
                color: Color {
                    a: rng.random_range(0.2..0.8),
                    ..Color::WHITE
                },
            });
        }

        Self { objects }
    }
}

impl PlotData<f64> for StarField {
    fn draw(&self, plot: &mut Plot<f64>, _theme: &Theme) {
        // Very small fixed size (1 screen pixel)
        let size = Measure::Screen(1.0);

        for obj in &self.objects {
            plot.add_shape(Rectangle::centered(obj.position, size, size).fill(obj.color));
        }
    }
}
