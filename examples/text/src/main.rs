use iced::{
    Color, Element, Point, Task, Theme,
    mouse::ScrollDelta,
    time::Instant,
    widget::{Space, button, column, radio, row, slider, text},
};
use iced_aksel::{
    Axis, Chart, Measure, Plot, PlotData, PlotPoint, State, axis::Position, plot::DragDelta,
    scale::Linear, shape::VectorLabel,
};
use rand::Rng;

// -----------------------------------------------------------------------------
// Constants & Types
// -----------------------------------------------------------------------------

const SYMBOLS: [char; 38] = [
    'A', 'B', 'C', 'X', 'Y', 'Z', 'a', 'b', 'c', 'x', 'y', 'z', // Basic Latin
    'Á', 'É', 'Í', 'Ó', 'Ú', 'á', 'é', 'í', 'ó', 'ú', 'ñ', 'ç', 'ø', 'ß', // Accented Latin
    'Δ', 'Ω', 'π', 'λ', // Greek
    'Ж', 'Я', 'ю', 'ф', // Cyrillic
    '∞', '✓', '§', '→', // Symbols & punctuation
];

const AXIS_X: &str = "x";
const AXIS_Y: &str = "y";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppMode {
    Stress,
    Showcase,
}

impl std::fmt::Display for AppMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stress => write!(f, "Stress Test"),
            Self::Showcase => write!(f, "Alphabet Showcase"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LabelType {
    Native,
    Vector,
}

impl std::fmt::Display for LabelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Native => write!(f, "Native (Iced)"),
            Self::Vector => write!(f, "Vector (Mesh)"),
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
    rotation: f32,
    scale_factor: f32,
}

struct TextLayer {
    items: Vec<TextItem>,
    label_type: LabelType,
    size_mode: SizeMode,
    tolerance: f32,
}

impl PlotData<f64> for TextLayer {
    fn draw(&self, plot: &mut Plot<f64>, _theme: &Theme) {
        // Hardcoded base size for consistency
        let base_size = 24.0;

        for item in &self.items {
            let size = match self.label_type {
                // Native always uses Screen size (fixed px)
                LabelType::Native => Measure::Screen(base_size * item.scale_factor),
                // Vector respects the user selection
                LabelType::Vector => match self.size_mode {
                    SizeMode::Screen => Measure::Screen(base_size * item.scale_factor),
                    SizeMode::Plot => {
                        // FIX: Removed the division by 10.0.
                        // Now the size directly maps to plot units (e.g. 24.0 units on 0-1000 axis)
                        Measure::Plot(base_size as f64 * item.scale_factor as f64)
                    }
                },
            };

            let rotation = if self.label_type == LabelType::Native {
                0.0
            } else {
                item.rotation
            };

            plot.add_shape(
                VectorLabel::new(&item.text, item.position)
                    .fill(Color::WHITE)
                    .size(size)
                    .rotation(rotation)
                    .quality(iced_aksel::Quality::Custom(self.tolerance)),
            );
        }
    }
}

// -----------------------------------------------------------------------------
// Application State
// -----------------------------------------------------------------------------

struct TextStressApp {
    state: State<&'static str, f64>,
    layer: TextLayer,

    // App State
    mode: AppMode,

    // Stress Controls
    stress_count: usize,

    // Stats
    fps: f32,
    last_frame: Option<Instant>,
}

#[derive(Debug, Clone)]
enum Message {
    Tick(Instant),
    ChartDragged(DragDelta),
    ChartScrolled(Point, ScrollDelta),

    // Config
    ModeChanged(AppMode),
    CountChanged(f32),
    ToleranceChanged(f32),
    TypeChanged(LabelType),
    SizeModeChanged(SizeMode),
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
            mode: AppMode::Showcase,
            layer: TextLayer {
                items: Vec::new(),
                label_type: LabelType::Vector,
                size_mode: SizeMode::Plot,
                tolerance: 0.5,
            },
            stress_count: 1000,
            fps: 0.0,
            last_frame: None,
        };

        app.generate();

        (app, Task::none())
    }

    fn generate(&mut self) {
        let mut rng = rand::rng();
        self.layer.items.clear();

        // 1000x1000 Plot Area
        let bounds_x = 1000.0;
        let bounds_y = 1000.0;

        match self.mode {
            AppMode::Stress => {
                self.layer.items.reserve(self.stress_count);

                for i in 0..self.stress_count {
                    let x = rng.random_range(0.0..bounds_x);
                    let y = rng.random_range(0.0..bounds_y);

                    // Random rotation for stress test
                    let rotation = rng.random_range(0.0..360.0f32).to_radians();

                    // Simple cycling text
                    let text = if i % 2 == 0 {
                        "Text".to_string()
                    } else {
                        "Label".to_string()
                    };

                    self.layer.items.push(TextItem {
                        position: PlotPoint::new(x, y),
                        text,
                        rotation,
                        scale_factor: rng.random_range(0.8..1.5),
                    });
                }
            }
            AppMode::Showcase => {
                let count = SYMBOLS.len();

                // Calculate Grid
                let cols = (count as f64).sqrt().ceil() as usize;
                let rows = (count as f64 / cols as f64).ceil() as usize;

                let cell_w = bounds_x / cols as f64;
                let cell_h = bounds_y / rows as f64;

                // Center the grid in the 0-1000 box
                let start_x = cell_w * 0.5;
                let start_y = bounds_y - (cell_h * 0.5);

                for (i, &char) in SYMBOLS.iter().enumerate() {
                    let col = i % cols;
                    let row = i / cols;

                    let x = (col as f64).mul_add(cell_w, start_x);
                    let y = (row as f64).mul_add(-cell_h, start_y);

                    self.layer.items.push(TextItem {
                        position: PlotPoint::new(x, y),
                        text: char.to_string(),
                        rotation: 0.0,
                        scale_factor: 2.0, // Multiplier for base size
                    });
                }
            }
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
            Message::ModeChanged(mode) => {
                self.mode = mode;
                self.generate();
                Task::none()
            }
            Message::CountChanged(c) => {
                self.stress_count = c as usize;
                if self.mode == AppMode::Stress {
                    self.generate();
                }
                Task::none()
            }
            Message::ToleranceChanged(t) => {
                self.layer.tolerance = t;
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
            Message::Regenerate => {
                self.generate();
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let chart = Chart::new(&self.state)
            .plot_data(&self.layer, AXIS_X, AXIS_Y)
            .on_drag(Message::ChartDragged)
            .on_scroll(Message::ChartScrolled);

        // --- Sidebar ---
        let mut controls = column![
            text("Text Engine").size(20),
            text(format!("FPS: {:.0}", self.fps))
                .size(16)
                .color(Color::from_rgb(0.0, 1.0, 0.0)),
            text(format!("Items: {}", self.layer.items.len()))
                .size(12)
                .color(Color::from_rgb(0.7, 0.7, 0.7)),
            vertical_space(10),
            // Mode Selection
            text("Mode").size(14),
            row![
                radio(
                    "Stress",
                    AppMode::Stress,
                    Some(self.mode),
                    Message::ModeChanged
                ),
                radio(
                    "Showcase",
                    AppMode::Showcase,
                    Some(self.mode),
                    Message::ModeChanged
                ),
            ]
            .spacing(10),
            vertical_space(15),
            // Rendering Type
            text("Rendering").size(14),
            row![
                radio(
                    "Native",
                    LabelType::Native,
                    Some(self.layer.label_type),
                    Message::TypeChanged
                )
                .size(12),
                radio(
                    "Vector",
                    LabelType::Vector,
                    Some(self.layer.label_type),
                    Message::TypeChanged
                )
                .size(12),
            ]
            .spacing(10),
            vertical_space(5),
        ];

        // --- Conditional Controls based on Label Type ---
        if self.layer.label_type == LabelType::Vector {
            controls = controls.push(
                column![
                    text("Size Behavior").size(12),
                    row![
                        radio(
                            "Screen (px)",
                            SizeMode::Screen,
                            Some(self.layer.size_mode),
                            Message::SizeModeChanged
                        )
                        .size(12),
                        radio(
                            "Plot (units)",
                            SizeMode::Plot,
                            Some(self.layer.size_mode),
                            Message::SizeModeChanged
                        )
                        .size(12),
                    ]
                    .spacing(10),
                    vertical_space(5),
                    text("Mesh Quality (Tolerance)").size(12),
                    row![
                        text("High").size(10),
                        slider(0.1..=5.0, self.layer.tolerance, Message::ToleranceChanged)
                            .step(0.1),
                        text("Low").size(10)
                    ]
                    .spacing(5),
                    text(format!("Pixel Error: {:.1}px", self.layer.tolerance)).size(10)
                ]
                .spacing(5),
            );
        } else {
            controls = controls.push(
                column![
                    text("Native Options").size(12),
                    text("Native labels are fixed to")
                        .size(10)
                        .color(Color::from_rgb(0.7, 0.7, 0.7)),
                    text("screen pixels and cannot")
                        .size(10)
                        .color(Color::from_rgb(0.7, 0.7, 0.7)),
                    text("rotate or scale with zoom.")
                        .size(10)
                        .color(Color::from_rgb(0.7, 0.7, 0.7)),
                ]
                .spacing(2),
            );
        }

        controls = controls.push(vertical_space(15));

        // --- Conditional Controls based on App Mode ---
        match self.mode {
            AppMode::Stress => {
                controls = controls.push(
                    column![
                        text("Stress Config").size(14),
                        control_slider(
                            "Count",
                            self.stress_count as f32,
                            50000.0,
                            Message::CountChanged
                        ),
                        vertical_space(10),
                        button("Regenerate")
                            .on_press(Message::Regenerate)
                            .width(iced::Length::Fill),
                    ]
                    .spacing(10),
                );
            }
            AppMode::Showcase => {
                controls = controls.push(
                    column![
                        text("Showcase Info").size(14),
                        text("Displays all available").size(12),
                        text("glyphs in the font.").size(12),
                        vertical_space(5),
                        text("Zoom in to inspect quality.")
                            .size(12)
                            .style(|_| text::Style {
                                color: Some(Color::from_rgb(0.5, 0.8, 1.0))
                            }),
                    ]
                    .spacing(5),
                );
            }
        }

        row![chart, controls.width(250).padding(20).spacing(10)].into()
    }
}

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

fn vertical_space(height: u16) -> Element<'static, Message> {
    Space::new()
        .height(iced::Length::Fixed(height as f32))
        .into()
}

pub fn main() -> iced::Result {
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
