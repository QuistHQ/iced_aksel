use iced::widget::checkbox;
use iced::widget::text::LineHeight;
use iced::{
    Border, Color, Element, Font, Length, Padding, Shadow, Theme,
    widget::{column, container, pick_list, row, text},
};
use iced_aksel::axis::{Marker, MarkerBadge, MarkerContext, MarkerLine, TickContext};
use iced_aksel::style::{DashStyle, LabelStyle};
use iced_aksel::{
    Axis, Chart, State,
    axis::{self, GridLine, Label, TickLine, TickResult},
    scale::Linear,
};

// # Axes Styling Showcase
//
// A comprehensive example demonstrating how to customize Axis visualization dynamically.
//
// This showcase contrasts two approaches to rendering Markers and Ticks:
// 1. **Simple:** Inheriting default styles and selectively overriding properties.
// 2. **Advanced:** Constructing visual elements from scratch for granular control.

pub fn main() -> iced::Result {
    iced::application(AxesShowcase::new, AxesShowcase::update, AxesShowcase::view)
        .title("Axes Styling Showcase")
        .theme(AxesShowcase::theme)
        .antialiasing(true)
        .run()
}

// -----------------------------------------------------------------------------
// Application State & Messages
// -----------------------------------------------------------------------------

struct AxesShowcase {
    theme: Theme,
    // The Chart State holds the data and the configuration of the axes.
    state: State<&'static str, f64>,
    // Toggles
    skip_label_overlapping: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    ThemeChanged(Theme),
    SkipOverlappingToggle(bool),
}

impl AxesShowcase {
    // Unique identifiers for our axes
    const X: &'static str = "x";
    const Y: &'static str = "y";

    fn new() -> (Self, iced::Task<Message>) {
        let theme = Theme::Dark;
        // We initialize the chart state immediately
        let initial_state = configure_chart_axes(true);

        (
            Self {
                state: initial_state,
                theme,
                skip_label_overlapping: true,
            },
            iced::Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::ThemeChanged(theme) => {
                self.theme = theme;
                // We recreate the axis configuration when the theme changes
                // to ensure colors match the new theme palette.
                self.state = configure_chart_axes(self.skip_label_overlapping);
            }
            Message::SkipOverlappingToggle(status) => {
                self.skip_label_overlapping = status;
                // Re-run setup to apply the new overlap setting
                self.state = configure_chart_axes(self.skip_label_overlapping);
            }
        }
        iced::Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        // 1. Controls Section
        let controls = row![
            row![
                text("Theme:"),
                pick_list(Theme::ALL, Some(&self.theme), Message::ThemeChanged)
            ]
            .spacing(10)
            .align_y(iced::Alignment::Center),
            row![
                text("Skip Overlapping Labels:"),
                checkbox(self.skip_label_overlapping).on_toggle(Message::SkipOverlappingToggle)
            ]
            .spacing(10)
            .align_y(iced::Alignment::Center),
        ]
        .spacing(30);

        // 2. Chart Section
        // We wrap the chart in a container for visual framing
        let content = column![
            controls,
            panel("Custom Axis Rendering", Chart::new(&self.state))
        ]
        .spacing(20)
        .padding(20);

        content.into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}

// -----------------------------------------------------------------------------
// Chart Configuration Logic
// -----------------------------------------------------------------------------

/// Sets up the Axis definitions, Scales, and custom Renderers.
fn configure_chart_axes(skip_overlapping_labels: bool) -> State<&'static str, f64> {
    let mut state = State::new();

    // --- X-Axis Configuration (The "Simple" Approach) ---
    // We use a standard Linear scale.
    let x_scale = Linear::new(0., 100.);

    let mut x_axis = Axis::new(x_scale, axis::Position::Bottom)
        // Renderers allow us to hijack the drawing process of specific elements
        .with_marker_renderer(simple_dynamic_marker())
        .with_tick_renderer(simple_tick_result())
        .without_grid(); // Clean look, no vertical grid lines

    // --- Y-Axis Configuration (The "Advanced" Approach) ---
    let y_scale = Linear::new(0., 100.);

    let mut y_axis = Axis::new(y_scale, axis::Position::Left)
        // We set a minimum gap (in pixels) between labels to prevent clutter
        .skip_overlapping_labels(6.)
        .with_marker_renderer(advanced_dynamic_marker())
        .with_tick_renderer(advanced_tick_result());

    // Apply the toggle setting from the UI
    if skip_overlapping_labels {
        x_axis.set_skip_overlapping_labels(6.);
        y_axis.set_skip_overlapping_labels(6.);
    }

    state.set_axis(AxesShowcase::X, x_axis);
    state.set_axis(AxesShowcase::Y, y_axis);

    state
}

// -----------------------------------------------------------------------------
// Renderers: The logic for how things look
// -----------------------------------------------------------------------------

/// A "Simple" marker renderer.
/// Strategy: Ask the context for the default marker, then just change the color.
fn simple_dynamic_marker() -> impl Fn(MarkerContext<f64>) -> Option<Marker> + 'static {
    move |ctx: MarkerContext<f64>| {
        // Logic: Color changes if value is in the lower or upper half
        let relative_color = if ctx.normalized_position <= 0.5 {
            ctx.theme.palette().warning
        } else {
            ctx.theme.palette().danger
        };

        // 1. Generate the default marker with standard formatting
        let default_marker = ctx.marker(format!("{:.2}", ctx.value));

        // 2. Update only the fields we care about
        Some(Marker {
            line: MarkerLine {
                color: relative_color,
                ..default_marker.line
            },
            badge: MarkerBadge {
                background: relative_color,
                ..default_marker.badge
            },
            ..default_marker
        })
    }
}

/// A "Simple" tick renderer.
/// Strategy: Standard text formatting, standard lines.
fn simple_tick_result() -> impl Fn(TickContext<f64>) -> TickResult + 'static {
    move |ctx: TickContext<f64>| {
        let text = format!("{:.2}", ctx.tick.value);

        TickResult {
            label: Some(ctx.label(text)),
            tick_line: Some(ctx.tickline()),
            grid_line: Some(ctx.gridline()),
            label_priority: None,
        }
    }
}

/// An "Advanced" marker renderer.
/// Strategy: Build every component (Label, Line, Badge) manually for pixel-perfect control.
fn advanced_dynamic_marker() -> impl Fn(MarkerContext<f64>) -> Option<Marker> + 'static {
    move |ctx: MarkerContext<f64>| {
        // Interpolate color based on position (0.0 to 1.0)
        let lerp_color = color_lerped(
            &ctx.theme.palette().danger,
            &ctx.theme.palette().warning,
            ctx.normalized_position,
        );

        // 1. Custom Label Styling
        let label = Label::from_style(
            format!("{:.2}", ctx.value),
            LabelStyle {
                size: 12.into(),
                color: ctx.theme.palette().text,
                padding: 4.into(),
                line_height: LineHeight::Relative(1.0),
            },
        );

        // 2. Custom Line styling (thinner, with a gap)
        let line = MarkerLine {
            color: lerp_color,
            width: 1.into(),
            gap: 4.into(),
        };

        // 3. Custom Badge styling
        let badge = MarkerBadge {
            background: lerp_color,
            border: Border::default().rounded(4.),
            shadow: Shadow::default(),
        };

        Some(Marker { label, badge, line })
    }
}

/// An "Advanced" tick renderer.
/// Strategy: Hide labels for minor ticks, apply gradients to major ticks.
fn advanced_tick_result() -> impl Fn(TickContext<f64>) -> TickResult + 'static {
    move |ctx: TickContext<f64>| {
        let is_major_tick = ctx.tick.level == 0;

        // Gradient logic
        let lerp_color = color_lerped(
            &ctx.theme.palette().danger,
            &ctx.theme.palette().warning,
            ctx.normalized_position,
        );

        // Define the visual elements
        let tick_line = TickLine {
            color: lerp_color,
            width: 1.into(),
            length: 4.into(),
        };

        let grid_line = GridLine {
            color: ctx.theme.extended_palette().background.neutral.color,
            width: 1.into(),
            dashed: Some(DashStyle::new(6., 2.)),
        };

        // Conditional Rendering:
        // We only generate a Label if this is a Major tick.
        let label = if is_major_tick {
            Some(Label::from_style(
                format!("{:.2}", ctx.tick.value),
                LabelStyle {
                    color: lerp_color,
                    padding: 4.into(),
                    size: 12.into(),
                    line_height: LineHeight::Relative(1.0),
                },
            ))
        } else {
            None
        };

        TickResult {
            label,
            tick_line: Some(tick_line),
            grid_line: Some(grid_line),
            label_priority: None,
        }
    }
}

// -----------------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------------

fn panel<'a>(title: &'a str, chart: Chart<'a, &'static str, f64, Message>) -> Element<'a, Message> {
    column![
        text(title).size(14).font(Font::MONOSPACE),
        container(chart)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|t: &Theme| container::Style::default()
                .background(t.extended_palette().background.weak.color)
                .border(Border {
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

fn color_lerped(start: &Color, end: &Color, v: f32) -> Color {
    let t = v.clamp(0.0, 1.0);
    Color {
        r: start.r + (end.r - start.r) * t,
        g: start.g + (end.g - start.g) * t,
        b: start.b + (end.b - start.b) * t,
        a: start.a + (end.a - start.a) * t,
    }
}
