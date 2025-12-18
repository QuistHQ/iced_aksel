use iced::{
    Color, Element, Point, Subscription, Task, Theme,
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
enum AppMode {
    Stress,
    Showcase, // The new "Alphabet Grid" mode
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContentMode {
    Labels,
    Symbols,
}

impl std::fmt::Display for ContentMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Labels => write!(f, "Random Labels"),
            Self::Symbols => write!(f, "Random Symbols"),
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
    tolerance: f32,
}

impl PlotData<f64> for TextLayer {
    fn draw(&self, plot: &mut Plot<f64>, _theme: &Theme) {
        if !self.show_labels {
            return;
        }

        match self.label_type {
            LabelType::Native => {
                for item in &self.items {
                    let px_size = self.base_size * item.scale_factor;
                    plot.add_shape(
                        Label::new(&item.text, item.position)
                            .fill(item.color)
                            .size(px_size),
                    );
                }
            }
            LabelType::Vector => {
                for item in &self.items {
                    let size = match self.size_mode {
                        SizeMode::Screen => Measure::Screen(self.base_size * item.scale_factor),
                        SizeMode::Plot => {
                            Measure::Plot((self.base_size / 10.0) as f64 * item.scale_factor as f64)
                        }
                    };

                    plot.add_shape(
                        VectorLabel::new(&item.text, item.position)
                            .fill(item.color)
                            .size(size)
                            .rotation(item.rotation)
                            .quality(iced_aksel::Quality::Custom(self.tolerance)),
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

    // App State
    mode: AppMode,

    // Stress Controls
    stress_count: usize,
    stress_max_rotation: f32,
    stress_randomize_color: bool,
    stress_content_mode: ContentMode,

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
    BaseSizeChanged(f32),
    MaxRotationChanged(f32),
    ToleranceChanged(f32),
    TypeChanged(LabelType),
    SizeModeChanged(SizeMode),
    ContentModeChanged(ContentMode),
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
            mode: AppMode::Showcase, // Default to showcase to see the alphabet
            layer: TextLayer {
                items: Vec::new(),
                label_type: LabelType::Vector,
                size_mode: SizeMode::Screen,
                base_size: 16.0,
                show_labels: true,
                tolerance: 0.5,
            },
            stress_count: 1000,
            stress_max_rotation: 360.0,
            stress_randomize_color: true,
            stress_content_mode: ContentMode::Symbols,
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

                // Get symbols if needed
                let all_symbols: Vec<char> = if self.stress_content_mode == ContentMode::Symbols {
                    iced_aksel::font::default()
                        .characters()
                        .filter(|c| !c.is_control())
                        .collect()
                } else {
                    Vec::new()
                };

                for i in 0..self.stress_count {
                    let x = rng.random_range(0.0..bounds_x);
                    let y = rng.random_range(0.0..bounds_y);

                    let rotation = if self.stress_max_rotation > 0.0 {
                        rng.random_range(-self.stress_max_rotation..self.stress_max_rotation)
                            .to_radians()
                    } else {
                        0.0
                    };

                    let color = if self.stress_randomize_color {
                        Color::from_rgb(rng.random(), rng.random(), rng.random())
                    } else {
                        Color::WHITE
                    };

                    let text = match self.stress_content_mode {
                        ContentMode::Labels => {
                            if i % 5 == 0 {
                                format!("Label {}", i)
                            } else if i % 5 == 1 {
                                "Text".to_string()
                            } else {
                                format!("{}", i)
                            }
                        }
                        ContentMode::Symbols => {
                            if all_symbols.is_empty() {
                                "?".to_string()
                            } else {
                                all_symbols[i % all_symbols.len()].to_string()
                            }
                        }
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
            AppMode::Showcase => {
                // Get EVERY character in the font
                let all_chars: Vec<char> = iced_aksel::font::default()
                    .characters()
                    .filter(|c| !c.is_control())
                    .collect();

                let count = all_chars.len();
                if count == 0 {
                    return;
                }

                // Calculate Grid
                let cols = (count as f64).sqrt().ceil() as usize;
                let rows = (count as f64 / cols as f64).ceil() as usize;

                let cell_w = bounds_x / cols as f64;
                let cell_h = bounds_y / rows as f64;

                // Center the grid in the 0-1000 box
                let start_x = cell_w * 0.5;
                let start_y = bounds_y - (cell_h * 0.5); // Top down layout

                for (i, &char) in all_chars.iter().enumerate() {
                    let col = i % cols;
                    let row = i / cols;

                    let x = start_x + (col as f64 * cell_w);
                    let y = start_y - (row as f64 * cell_h);

                    self.layer.items.push(TextItem {
                        position: PlotPoint::new(x, y),
                        text: char.to_string(),
                        color: Color::WHITE,
                        rotation: 0.0,     // Fixed rotation for showcase
                        scale_factor: 1.0, // Consistent size
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
            Message::BaseSizeChanged(s) => {
                self.layer.base_size = s;
                Task::none()
            }
            Message::MaxRotationChanged(r) => {
                self.stress_max_rotation = r;
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
            Message::ContentModeChanged(mode) => {
                self.stress_content_mode = mode;
                if self.mode == AppMode::Stress {
                    self.generate();
                }
                Task::none()
            }
            Message::ToggleVisibility(v) => {
                self.layer.show_labels = v;
                Task::none()
            }
            Message::ToggleRandomColor(v) => {
                self.stress_randomize_color = v;
                if self.mode == AppMode::Stress {
                    self.generate();
                }
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
            // Common Controls (Apply to both)
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
            control_slider(
                "Size",
                self.layer.base_size,
                100.0,
                Message::BaseSizeChanged
            ),
            row![
                radio(
                    "Px",
                    SizeMode::Screen,
                    Some(self.layer.size_mode),
                    Message::SizeModeChanged
                )
                .size(12),
                radio(
                    "Unit",
                    SizeMode::Plot,
                    Some(self.layer.size_mode),
                    Message::SizeModeChanged
                )
                .size(12),
            ]
            .spacing(10),
            vertical_space(5),
        ];

        // Vector Specifics
        if self.layer.label_type == LabelType::Vector {
            controls = controls.push(
                column![
                    text("Level of Detail").size(12),
                    row![
                        text("High").size(10),
                        slider(0.1..=5.0, self.layer.tolerance, Message::ToleranceChanged)
                            .step(0.1),
                        text("Low").size(10)
                    ]
                    .spacing(5),
                    text(format!("Pixel Error: {:.1}px", self.layer.tolerance)).size(10)
                ]
                .spacing(2),
            );
        }

        controls = controls.push(vertical_space(15));

        // Mode Specific Controls
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
                        if self.layer.label_type == LabelType::Vector {
                            control_slider(
                                "Rotation",
                                self.stress_max_rotation,
                                360.0,
                                Message::MaxRotationChanged,
                            )
                        } else {
                            column![].into()
                        },
                        vertical_space(5),
                        row![
                            radio(
                                "Labels",
                                ContentMode::Labels,
                                Some(self.stress_content_mode),
                                Message::ContentModeChanged
                            )
                            .size(12),
                            radio(
                                "Symbols",
                                ContentMode::Symbols,
                                Some(self.stress_content_mode),
                                Message::ContentModeChanged
                            )
                            .size(12),
                        ]
                        .spacing(5),
                        checkbox_row(
                            "Random Colors",
                            self.stress_randomize_color,
                            Message::ToggleRandomColor
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
                        text("Zoom in to inspect").size(12).style(|_| text::Style {
                            color: Some(Color::from_rgb(0.5, 0.8, 1.0))
                        }),
                        text("mesh quality.").size(12).style(|_| text::Style {
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

fn checkbox_row(
    label: &'static str,
    value: bool,
    msg: fn(bool) -> Message,
) -> Element<'static, Message> {
    row![checkbox(value).on_toggle(msg), text(label).size(14)]
        .spacing(10)
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
