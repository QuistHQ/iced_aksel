use iced::{
    Alignment, Color, Element, Point, Subscription, Task, Theme,
    mouse::ScrollDelta,
    time::Instant,
    widget::{button, checkbox, column, radio, row, slider, text},
};
use iced_aksel::{
    Axis, Chart, Measure, Plot, PlotData, PlotPoint, Quality, State, axis::Position,
    plot::DragDelta, scale::Linear, shape::Label,
};

const AXIS_X: &str = "x";
const AXIS_Y: &str = "y";

// -----------------------------------------------------------------------------
// Data Layer
// -----------------------------------------------------------------------------

struct TextLayer {
    labels: Vec<Label<f64>>,
}

impl PlotData<f64> for TextLayer {
    fn draw(&self, plot: &mut Plot<f64>, _theme: &Theme) {
        for label in &self.labels {
            plot.add_shape(label.clone());
        }
    }
}

// -----------------------------------------------------------------------------
// Application
// -----------------------------------------------------------------------------

struct TextExample {
    state: State<&'static str, f64>,
    layer: TextLayer,

    // Config
    mode: AppMode,
    stress_count: usize,
    global_quality: f32,

    // Stats
    fps: f32,
    last_frame: Option<Instant>,
    frame_times: Vec<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppMode {
    Showcase,
    Stress,
}

#[derive(Debug, Clone)]
enum Message {
    Tick(Instant),
    ChartDragged(DragDelta),
    ChartScrolled(Point, ScrollDelta),

    // Config
    ModeChanged(AppMode),
    QualityChanged(f32),
    RegenerateStress,
}

impl TextExample {
    fn new() -> (Self, Task<Message>) {
        let mut state = State::new();
        state.set_axis(
            AXIS_X,
            Axis::new(Linear::new(-10.0, 110.0), Position::Bottom),
        );
        state.set_axis(AXIS_Y, Axis::new(Linear::new(-10.0, 110.0), Position::Left));

        let mut app = Self {
            state,
            layer: TextLayer { labels: Vec::new() },
            mode: AppMode::Showcase,
            stress_count: 2500,
            global_quality: 1.0,
            fps: 0.0,
            last_frame: None,
            frame_times: Vec::with_capacity(60),
        };

        app.generate_showcase();

        (app, Task::none())
    }

    fn generate_showcase(&mut self) {
        self.layer.labels.clear();

        // 1. Measure::Screen (UI Style)
        // Stays 24px regardless of zoom
        self.layer.labels.push(
            Label::new("Screen Size (24px)", PlotPoint::new(10.0, 90.0))
                .size(Measure::Screen(24.0))
                .fill(Color::from_rgb(0.2, 0.4, 0.8)),
        );

        // 2. Measure::Plot (World Style)
        // Stays 5 units tall (zooms with chart)
        self.layer.labels.push(
            Label::new("Plot Size (5 Units)", PlotPoint::new(10.0, 70.0))
                .size(Measure::Plot(5.0))
                .fill(Color::from_rgb(0.8, 0.2, 0.2)),
        );

        // 3. Rotation
        self.layer.labels.push(
            Label::new("Rotated (45°)", PlotPoint::new(60.0, 70.0))
                .size(Measure::Screen(20.0))
                .rotation(45.0f32.to_radians())
                .fill(Color::from_rgb(0.2, 0.8, 0.2)),
        );

        // 4. Upside Down (Rotation + Alignment)
        self.layer.labels.push(
            Label::new("Upside Down", PlotPoint::new(60.0, 50.0))
                .size(Measure::Screen(20.0))
                .rotation(180.0f32.to_radians())
                .fill(Color::from_rgb(0.5, 0.5, 0.5)),
        );

        // 5. Quality Override
        self.layer.labels.push(
            Label::new("Forced High Quality", PlotPoint::new(10.0, 30.0))
                .size(Measure::Plot(8.0)) // Big text to show curves
                .quality(Quality::High)
                .fill(Color::BLACK),
        );

        self.layer.labels.push(
            Label::new("Tiny Text (Zoom In)", PlotPoint::new(10.0, 10.0))
                .size(Measure::Plot(0.5))
                .fill(Color::BLACK),
        );
    }

    fn generate_stress(&mut self) {
        self.layer.labels.clear();
        let side = (self.stress_count as f64).sqrt().ceil() as usize;
        let step = 100.0 / side as f64;

        for x in 0..side {
            for y in 0..side {
                let px = x as f64 * step;
                let py = y as f64 * step;

                self.layer.labels.push(
                    Label::new(format!("{}-{}", x, y), PlotPoint::new(px, py))
                        .size(Measure::Screen(10.0))
                        .fill(Color::from_rgba(0.0, 0.0, 0.0, 0.6))
                        // Optimization: Low quality for small bulk labels
                        .quality(Quality::Low),
                );
            }
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick(now) => {
                if let Some(last) = self.last_frame {
                    let delta = now.duration_since(last).as_secs_f32();
                    if delta > 0.0 {
                        let instant_fps = 1.0 / delta;
                        self.fps = self.fps * 0.9 + instant_fps * 0.1;
                        self.frame_times.push(delta * 1000.0);
                        if self.frame_times.len() > 60 {
                            self.frame_times.remove(0);
                        }
                    }
                }
                self.last_frame = Some(now);
                Task::none()
            }
            Message::ChartDragged(delta) => {
                self.state.pan_axes(AXIS_X, AXIS_Y, delta.x, delta.y);
                Task::none()
            }
            Message::ChartScrolled(pt, delta) => {
                if let ScrollDelta::Lines { y, .. } = delta {
                    let factor = if y > 0.0 { 1.1 } else { 0.9 };
                    self.state.zoom_axes(AXIS_X, AXIS_Y, pt.x, pt.y, factor);
                }
                Task::none()
            }
            Message::ModeChanged(mode) => {
                self.mode = mode;
                match self.mode {
                    AppMode::Showcase => self.generate_showcase(),
                    AppMode::Stress => self.generate_stress(),
                }
                Task::none()
            }
            Message::QualityChanged(q) => {
                self.global_quality = q;
                Task::none()
            }
            Message::RegenerateStress => {
                if self.mode == AppMode::Stress {
                    self.generate_stress();
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // NOTE: We assume Chart now accepts .quality(f32) based on the backend changes.
        // If the Chart widget definition hasn't been updated yet, this method call might need
        // to be added to iced_aksel/src/chart.rs.
        let chart = Chart::new(&self.state)
            .debug(true)
            .plot_data(&self.layer, AXIS_X, AXIS_Y)
            .on_drag(Message::ChartDragged)
            .on_scroll(Message::ChartScrolled);

        let sidebar = column![
            text("Text Engine").size(20),
            text(format!("FPS: {:.0}", self.fps))
                .size(16)
                .color(Color::from_rgb(0.0, 0.8, 0.0)),
            text(format!("Labels: {}", self.layer.labels.len())).size(12),
            text("Mode").size(14),
            row![
                radio(
                    "Showcase",
                    AppMode::Showcase,
                    Some(self.mode),
                    Message::ModeChanged
                ),
                radio(
                    "Stress",
                    AppMode::Stress,
                    Some(self.mode),
                    Message::ModeChanged
                ),
            ]
            .spacing(10),
            text("Global Quality").size(14),
            text(format!("Multiplier: {:.1}x", self.global_quality)).size(12),
            slider(0.1..=3.0, self.global_quality, Message::QualityChanged).step(0.1),
            text("Lower = Faster/Blockier")
                .size(10)
                .color(Color::from_rgb(0.6, 0.6, 0.6)),
            text("Higher = Smoother/Slower")
                .size(10)
                .color(Color::from_rgb(0.6, 0.6, 0.6)),
        ]
        .spacing(15)
        .padding(10)
        .width(200);

        row![chart, sidebar].into()
    }
}

pub fn main() -> iced::Result {
    iced::application(TextExample::new, TextExample::update, TextExample::view)
        .theme(Theme::Dark)
        .subscription(|_| iced::window::frames().map(Message::Tick))
        .antialiasing(true)
        .run()
}
