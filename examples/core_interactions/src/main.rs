use iced::{
    Color, Element, Length, Subscription, Theme,
    alignment::{Horizontal, Vertical},
    keyboard,
    widget::{column, container, row, text},
};
use iced_aksel::{
    Axis, Chart, PlotPoint, State, Stroke,
    axis::{self, TickLine, TickResult},
    plot::{self, Plot, PlotData},
    scale::Linear,
    shape::Polyline,
};

// -----------------------------------------------------------------------------
// Application Entry
// -----------------------------------------------------------------------------

pub fn main() -> iced::Result {
    iced::application(Interactions::new, Interactions::update, Interactions::view)
        .title("Interactions: Signal Analyzer")
        .subscription(Interactions::subscription)
        .theme(Theme::Dark)
        .antialiasing(true)
        .run()
}

// -----------------------------------------------------------------------------
// Application State
// -----------------------------------------------------------------------------

pub struct Interactions {
    // Chart State
    state: State<&'static str, f64>,
    // Data
    signal: SignalData,
    // Interactions
    cursor_position: Option<(f64, f64)>, // The data value under the mouse
    modifiers: keyboard::Modifiers,      // For Shift/Ctrl zooming
}

#[derive(Debug, Clone)]
pub enum Message {
    Scrolled(iced::Point, iced::mouse::ScrollDelta),
    Dragged(plot::DragDelta),
    Hovered(iced::Point),
    Unhovered,
    ModifiersChanged(keyboard::Modifiers),
}

impl Interactions {
    const AXIS_X: &'static str = "time";
    const AXIS_Y: &'static str = "amplitude";

    pub fn new() -> (Self, iced::Task<Message>) {
        // 0. Initialize chart state
        let mut state = State::new();

        // 1. Create the scales for the axes
        let linear_x = Linear::new(0.0, 10.0);
        let linear_y = Linear::new(-10.0, 10.0);

        // 2. Create the axes with the new `TickResult` paradigm

        // ---------------------------------------------------------------------
        // X AXIS: Time (Seconds)
        // Logic: Show simple ticks and labels, but ONLY for Major ticks.
        // ---------------------------------------------------------------------
        let axis_x = Axis::new(linear_x, axis::Position::Bottom)
            .skip_overlapping_labels(6.)
            .with_tick_renderer(|ctx| {
                // If it's a Major tick (Level 0), show it.
                if ctx.tick.level == 0 {
                    TickResult {
                        label: Some(ctx.label(format!("{:.1}s", ctx.tick.value))),
                        tick_line: Some(ctx.tickline()),
                        ..Default::default()
                    }
                } else {
                    TickResult::default()
                }
            });

        // ---------------------------------------------------------------------
        // Y AXIS: Voltage (Volts)
        // Logic:
        // - Major ticks: Standard line + Text Label
        // - Minor ticks: Small/Thin line + NO Text
        // ---------------------------------------------------------------------
        let axis_y = Axis::new(linear_y, axis::Position::Left).with_tick_renderer(|ctx| {
            let is_major = ctx.tick.level == 0;
            let label = is_major.then(|| ctx.label(format!("{:.1}V", ctx.tick.value)));
            let tick_line = is_major.then(|| ctx.tickline()).or_else(|| {
                Some(TickLine {
                    width: 0.5.into(),
                    length: 2.5.into(),
                    ..ctx.tickline()
                })
            });

            TickResult {
                tick_line,
                label,
                ..Default::default()
            }
        });

        // 3. Add the axes to the state
        state.set_axis(Self::AXIS_X, axis_x);
        state.set_axis(Self::AXIS_Y, axis_y);

        (
            Self {
                state,
                signal: SignalData::new(),
                cursor_position: None,
                modifiers: keyboard::Modifiers::default(),
            },
            iced::Task::none(),
        )
    }

    pub fn subscription(&self) -> Subscription<Message> {
        // Listen for modifier keys to enable axis-locking
        iced::event::listen_with(|event, _status, _window_id| {
            if let iced::Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) = event {
                Some(Message::ModifiersChanged(modifiers))
            } else {
                None
            }
        })
    }

    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers;
            }
            Message::Scrolled(cursor_norm, delta) => {
                // 1. Extract direction for scroll
                let delta_y = match delta {
                    iced::mouse::ScrollDelta::Lines { y, .. }
                    | iced::mouse::ScrollDelta::Pixels { y, .. } => y,
                };

                // 1.a Determine factor for scrolling (10% of view)
                // Positive Delta (Scroll Up) -> Zoom In (> 1.0)
                // Negative Delta (Scroll Down) -> Zoom Out (< 1.0)
                let factor = if delta_y > 0.0 { 1.10 } else { 0.90 };

                // 2. Determine Targets (Default: X, Shift: Y, Ctrl: Both)
                let zoom_x = self.modifiers.command() || !self.modifiers.shift();
                let zoom_y = self.modifiers.command() || self.modifiers.shift();

                // 3. Apply Zoom
                if zoom_x && let Some(axis) = self.state.axis_mut_opt(&Self::AXIS_X) {
                    axis.zoom(factor, Some(cursor_norm.x));
                }
                if zoom_y && let Some(axis) = self.state.axis_mut_opt(&Self::AXIS_Y) {
                    axis.zoom(factor, Some(cursor_norm.y));
                }
            }
            Message::Dragged(delta) => {
                // Pan Logic:
                // We invert both axes to match the "Camera/Viewport" control feel requested.
                self.state
                    .pan_axes(Self::AXIS_X, Self::AXIS_Y, delta.x, delta.y);
            }
            Message::Hovered(cursor_norm) => {
                // Calculate real data values from normalized cursor position
                let x_axis = self.state.axis_opt(&Self::AXIS_X);
                let y_axis = self.state.axis_opt(&Self::AXIS_Y);

                if let (Some(x_ax), Some(y_ax)) = (x_axis, y_axis) {
                    let x = x_ax.denormalize_opt(cursor_norm.x);
                    // Invert Y for denormalization to match Cartesian coordinates
                    let y = y_ax.denormalize_opt(1.0 - cursor_norm.y);

                    if let (Some(x), Some(y)) = (x, y) {
                        self.cursor_position = Some((x, y));
                    }
                }
            }
            Message::Unhovered => {
                self.cursor_position = None;
            }
        }
        iced::Task::none()
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        // 1. Header Readout
        let header_text = if let Some((x, y)) = self.cursor_position {
            format!("Time: {:.3}s  |  Amplitude: {:.3}V", x, y)
        } else {
            "Hover to inspect signal.".to_string()
        };

        let header = container(
            text(header_text)
                .size(16)
                .font(iced::font::Font::MONOSPACE)
                .color(if self.cursor_position.is_some() {
                    Color::WHITE
                } else {
                    Color::from_rgb(0.7, 0.7, 0.7)
                }),
        )
        .padding(15)
        .width(Length::Fill)
        .align_x(Horizontal::Center)
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            container::Style::default()
                .background(palette.background.weak.color)
                .border(
                    iced::border::Border::default()
                        .rounded(1.0)
                        .color(palette.background.strong.color),
                )
        });

        // 2. Modifier Help Panel
        let controls = row![
            control_badge("Scroll", "Zoom X"),
            control_badge("Shift+Scroll", "Zoom Y"),
            control_badge("Ctrl+Scroll", "Zoom Both"),
            control_badge("Drag", "Pan"),
        ]
        .spacing(20)
        .padding(10)
        .align_y(Vertical::Center);

        // 3. The Chart
        let chart = Chart::new(&self.state)
            .plot_data(&self.signal, Self::AXIS_X, Self::AXIS_Y)
            .on_scroll(Message::Scrolled)
            .on_drag(Message::Dragged)
            .on_hover(Message::Hovered)
            .on_error(|_| Message::Unhovered);

        // 4. Layout
        column![
            header,
            container(controls)
                .width(Length::Fill)
                .align_x(Horizontal::Center)
                .style(|theme: &Theme| {
                    container::Style::default()
                        .background(theme.extended_palette().background.weak.color)
                }),
            container(chart)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(20)
        ]
        .into()
    }
}

// -----------------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------------

fn control_badge<'a>(key: &'a str, description: &'a str) -> Element<'a, Message> {
    row![
        container(text(key).size(12).color(Color::BLACK))
            .padding([2, 6])
            .style(|theme: &Theme| {
                let palette = theme.extended_palette();
                container::Style::default()
                    .background(palette.primary.strong.color)
                    .border(iced::border::rounded(4))
            }),
        text(description)
            .size(12)
            .color(Color::from_rgb(0.7, 0.7, 0.7))
    ]
    .spacing(5)
    .align_y(Vertical::Center)
    .into()
}

// -----------------------------------------------------------------------------
// Data Layer
// -----------------------------------------------------------------------------

struct SignalData {
    points: Vec<PlotPoint<f64>>,
}

impl SignalData {
    fn new() -> Self {
        // Generate 2000 points of a noisy signal
        let points = (0..=2000)
            .map(|i| {
                let x = i as f64 / 100.0;
                let y = (x * 50.0)
                    .sin()
                    .mul_add(0.5, (x * 3.0).sin().mul_add(5.0, (x * 12.0).sin() * 2.0));
                PlotPoint::new(x, y)
            })
            .collect();
        Self { points }
    }
}

impl PlotData<f64> for SignalData {
    fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {
        plot.add_shape(Polyline::new(self.points.clone()).stroke(Stroke::new(
            theme.palette().primary,
            iced_aksel::Measure::Screen(1.5),
        )));
    }
}
