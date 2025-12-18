use iced::{
    Color, Element, Point, Task, Theme,
    mouse::ScrollDelta,
    time::Instant,
    widget::{Space, button, checkbox, column, radio, row, slider, text},
};
use iced_aksel::{
    Axis, Chart, Measure, Plot, PlotData, PlotPoint, State,
    axis::Position,
    plot::{self, DragDelta},
    scale::Linear,
    shape::{Label, VectorLabel},
};
use rand::Rng;

// -----------------------------------------------------------------------------
// Constants & Types
// -----------------------------------------------------------------------------

const AXIS_X: &str = "x";
const AXIS_Y: &str = "y";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LabelType {
    Native,
    Geometric,
}

impl std::fmt::Display for LabelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Native => write!(f, "Native (Iced)"),
            Self::Geometric => write!(f, "Geometric (Mesh)"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SizeMode {
    Screen,
    Plot,
}

impl std::fmt::Display for SizeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Screen => write!(f, "Screen (px)"),
            Self::Plot => write!(f, "Plot (units)"),
        }
    }
}

// -----------------------------------------------------------------------------
// Data Layer
// -----------------------------------------------------------------------------

struct TextItem {
    position: PlotPoint,
    text: String,
    color: Color,
    rotation: f32,
    scale_factor: f32,
}

struct TextLayer {
    items: Vec<TextItem>,
    label_type: LabelType,
    size_mode: SizeMode,
    base_size: f32,
    show_labels: bool,
}

impl PlotData<f64> for TextLayer {
    fn draw(&self, plot: &mut Plot<f64>, _theme: &Theme) {
        if !self.show_labels {
            return;
        }

        match self.label_type {
            LabelType::Native => {
                for item in &self.items {
                    // Fix: Native Label takes raw f32 (pixels).
                    // We calculate the pixel size here.
                    let px_size = self.base_size * item.scale_factor;

                    // Native labels do not support Plot-Space sizing or Rotation in standard Iced.
                    // We render them as standard screen-aligned text.
                    plot.add_shape(
                        Label::new(&item.text, item.position)
                            .fill(item.color)
                            .size(px_size),
                    );
                }
            }
            LabelType::Geometric => {
                for item in &self.items {
                    let size = match self.size_mode {
                        SizeMode::Screen => Measure::Screen(self.base_size * item.scale_factor),
                        // Map plot units roughly to a fraction of base size for demo purposes
                        SizeMode::Plot => {
                            Measure::Plot((self.base_size / 10.0) as f64 * item.scale_factor as f64)
                        }
                    };

                    plot.add_shape(
                        VectorLabel::new(&item.text, item.position)
                            .fill(item.color)
                            .size(size)
                            .rotation(item.rotation),
                    );
                }
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Application State
// -----------------------------------------------------------------------------

struct TextStressApp {
    state: State<&'static str, f64>,
    layer: TextLayer,

    // Controls
    count: usize,
    max_rotation: f32,
    randomize_color: bool,

    // Stats
    fps: f32,
    last_frame: Option<Instant>,
}

#[derive(Debug, Clone)]
enum Message {
    Tick(Instant),
    ChartDragged(DragDelta),
    ChartScrolled(Point, ScrollDelta),
    CountChanged(f32),
    BaseSizeChanged(f32),
    MaxRotationChanged(f32),
    TypeChanged(LabelType),
    SizeModeChanged(SizeMode),
    ToggleVisibility(bool),
    ToggleRandomColor(bool),
    Regenerate,
}

impl TextStressApp {
    fn new() -> (Self, Task<Message>) {
        let mut state = State::new();
        state.set_axis(
            AXIS_X,
            Axis::new(Linear::new(0.0, 1000.0), Position::Bottom),
        );
        state.set_axis(AXIS_Y, Axis::new(Linear::new(0.0, 1000.0), Position::Left));

        let mut app = Self {
            state,
            layer: TextLayer {
                items: Vec::new(),
                label_type: LabelType::Geometric,
                size_mode: SizeMode::Screen,
                base_size: 16.0,
                show_labels: true,
            },
            count: 1000,
            max_rotation: 360.0,
            randomize_color: true,
            fps: 0.0,
            last_frame: None,
        };

        app.generate();

        (app, Task::none())
    }

    fn generate(&mut self) {
        // Fix: Use rand::rng() instead of thread_rng() if on rand 0.9,
        // but older rand (0.8) uses thread_rng(). Assuming 0.8 based on typical ecosystem.
        let mut rng = rand::rng();
        self.layer.items.clear();
        self.layer.items.reserve(self.count);

        let bounds_x = 1000.0;
        let bounds_y = 1000.0;

        for i in 0..self.count {
            // Fix: Use random_range instead of gen_range
            let x = rng.random_range(0.0..bounds_x);
            let y = rng.random_range(0.0..bounds_y);

            let rotation = if self.max_rotation > 0.0 {
                rng.random_range(-self.max_rotation..self.max_rotation)
                    .to_radians()
            } else {
                0.0
            };

            let color = if self.randomize_color {
                // Fix: Use rng.random() instead of gen()
                Color::from_rgb(rng.random(), rng.random(), rng.random())
            } else {
                Color::WHITE
            };

            let text = if i % 5 == 0 {
                format!("Label {}", i)
            } else if i % 5 == 1 {
                "Text".to_string()
            } else {
                format!("{}", i)
            };

            self.layer.items.push(TextItem {
                position: PlotPoint::new(x, y),
                text,
                color,
                rotation,
                scale_factor: rng.random_range(0.8..1.5),
            });
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick(now) => {
                if let Some(last) = self.last_frame {
                    let delta = now.duration_since(last).as_secs_f32();
                    if delta > 0.0 {
                        self.fps = self.fps * 0.9 + (1.0 / delta) * 0.1;
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
            Message::CountChanged(c) => {
                self.count = c as usize;
                self.generate();
                Task::none()
            }
            Message::BaseSizeChanged(s) => {
                self.layer.base_size = s;
                Task::none()
            }
            Message::MaxRotationChanged(r) => {
                self.max_rotation = r;
                self.generate();
                Task::none()
            }
            Message::TypeChanged(t) => {
                self.layer.label_type = t;
                Task::none()
            }
            Message::SizeModeChanged(m) => {
                self.layer.size_mode = m;
                Task::none()
            }
            Message::ToggleVisibility(v) => {
                self.layer.show_labels = v;
                Task::none()
            }
            Message::ToggleRandomColor(v) => {
                self.randomize_color = v;
                self.generate();
                Task::none()
            }
            Message::Regenerate => {
                self.generate();
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let chart = Chart::new(&self.state)
            .debug(true)
            .plot_data(&self.layer, AXIS_X, AXIS_Y)
            .on_drag(Message::ChartDragged)
            .on_scroll(Message::ChartScrolled);

        let controls = column![
            text("Text Stress Test").size(20),
            text(format!("FPS: {:.0}", self.fps))
                .size(16)
                .color(Color::from_rgb(0.0, 1.0, 0.0)),
            vertical_space(10),
            text("Label Type").size(14),
            row![
                radio(
                    "Native",
                    LabelType::Native,
                    Some(self.layer.label_type),
                    Message::TypeChanged
                ),
                radio(
                    "Geometric",
                    LabelType::Geometric,
                    Some(self.layer.label_type),
                    Message::TypeChanged
                ),
            ]
            .spacing(10),
            vertical_space(10),
            control_slider("Count", self.count as f32, 50000.0, Message::CountChanged),
            control_slider(
                "Font Size",
                self.layer.base_size,
                100.0,
                Message::BaseSizeChanged
            ),
            if self.layer.label_type == LabelType::Geometric {
                control_slider(
                    "Max Rotation",
                    self.max_rotation,
                    360.0,
                    Message::MaxRotationChanged,
                )
            } else {
                column![].into()
            },
            vertical_space(10),
            text("Size Mode").size(14),
            row![
                radio(
                    "Screen",
                    SizeMode::Screen,
                    Some(self.layer.size_mode),
                    Message::SizeModeChanged
                ),
                radio(
                    "Plot",
                    SizeMode::Plot,
                    Some(self.layer.size_mode),
                    Message::SizeModeChanged
                ),
            ]
            .spacing(10),
            vertical_space(10),
            checkbox_row(
                "Random Colors",
                self.randomize_color,
                Message::ToggleRandomColor
            ),
            checkbox_row("Visible", self.layer.show_labels, Message::ToggleVisibility),
            vertical_space(20),
            button("Regenerate")
                .on_press(Message::Regenerate)
                .width(iced::Length::Fill),
        ]
        .width(250)
        .padding(20)
        .spacing(10);

        row![chart, controls].into()
    }
}

// Fix: Use 'static lifetime for label to avoid "lifetime must outlive static" error
fn control_slider(
    label: &'static str,
    value: f32,
    max: f32,
    msg: fn(f32) -> Message,
) -> Element<'static, Message> {
    column![
        row![text(label).size(12), text(format!("{:.0}", value)).size(12)].spacing(5),
        slider(0.0..=max, value, msg).step(1.0)
    ]
    .spacing(2)
    .into()
}

// Fix: checkbox now requires manual label handling
fn checkbox_row(
    label: &'static str,
    value: bool,
    msg: fn(bool) -> Message,
) -> Element<'static, Message> {
    row![checkbox(value).on_toggle(msg), text(label).size(14)]
        .spacing(10)
        .into()
}

// Fix: Cast u16 to f32/Length
fn vertical_space(height: u16) -> Element<'static, Message> {
    Space::new()
        .width(iced::Length::Fixed(0.0))
        .height(iced::Length::Fixed(height as f32))
        .into()
}

pub fn main() -> iced::Result {
    // Fix: Explicitly type the closure arguments or simplify to avoid "not general enough" error
    iced::application(
        TextStressApp::new,
        TextStressApp::update,
        TextStressApp::view,
    )
    .theme(Theme::Dark)
    .subscription(|_| iced::window::frames().map(Message::Tick))
    .antialiasing(true)
    .run()
}
