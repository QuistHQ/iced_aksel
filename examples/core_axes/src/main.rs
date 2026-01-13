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
// A comprehensive example demonstrating multiple axes stacked on different sides.

pub fn main() -> iced::Result {
    iced::application(AxesShowcase::new, AxesShowcase::update, AxesShowcase::view)
        .title("Axes Styling Showcase")
        .theme(AxesShowcase::theme)
        .run()
}

struct AxesShowcase {
    theme: Theme,
    state: State<&'static str, f64>,
    skip_label_overlapping: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    ThemeChanged(Theme),
    SkipOverlappingToggle(bool),
}

impl AxesShowcase {
    // We define unique IDs for our four axes
    const X_MAIN: &'static str = "x_main";
    const X_SECONDARY: &'static str = "x_secondary";
    const Y_LEFT: &'static str = "y_left";
    const Y_RIGHT: &'static str = "y_right";

    fn new() -> (Self, iced::Task<Message>) {
        let theme = Theme::Dark;
        (
            Self {
                state: axes_setup(true),
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
                self.state = axes_setup(self.skip_label_overlapping);
            }
            Message::SkipOverlappingToggle(status) => {
                self.skip_label_overlapping = status;
                self.state = axes_setup(self.skip_label_overlapping);
            }
        }
        iced::Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        // Theme Section
        let theme_title = text("Theme:");
        let theme_picker = pick_list(Theme::ALL, Some(&self.theme), Message::ThemeChanged);
        let theme_section = row![theme_title, theme_picker,].spacing(16.);

        // Skip overlapping labels settings
        let skip_overlapping_title = text("Skip Overlapping Labels:");
        let skip_overlapping_checkbox =
            checkbox(self.skip_label_overlapping).on_toggle(Message::SkipOverlappingToggle);
        let skip_overlapping_section =
            row![skip_overlapping_title, skip_overlapping_checkbox,].spacing(16.);

        // Chart Section
        let chart_panel = panel("Multi-Axis Showcase", Chart::new(&self.state));

        column![theme_section, skip_overlapping_section, chart_panel,]
            .spacing(20)
            .padding(20)
            .into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}

fn panel<'a>(title: &'a str, chart: Chart<'a, &'static str, f64, Message>) -> Element<'a, Message> {
    column![
        text(title).size(16).font(Font::MONOSPACE),
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

fn axes_setup(skip_overlapping_labels: bool) -> State<&'static str, f64> {
    let mut state = State::new();

    // --------------------------------------------------------
    // 1. Top Axis
    // --------------------------------------------------------
    let x_scale_1 = Linear::new(0., 100.);
    let mut x_main = Axis::new(x_scale_1, axis::Position::Top)
        .with_thickness(50.0)
        .with_tick_renderer(advanced_tick_result())
        .style(|style| {
            style.spine.width = 2.into();
        });

    if skip_overlapping_labels {
        x_main.set_skip_overlapping_labels(6.);
    }

    // --------------------------------------------------------
    // 2. Bottom Axis
    // --------------------------------------------------------
    // This axis uses a different scale and "advanced" styling
    // to distinguish it from the main axis.
    let x_scale_2 = Linear::new(0., 1000.);
    let mut x_secondary = Axis::new(x_scale_2, axis::Position::Bottom)
        .with_thickness(50.0)
        .with_tick_renderer(advanced_tick_result()) // Uses gradients
        .style(|style| {
            style.spine.width = 1.into();
        }); // Avoid grid clutter

    if skip_overlapping_labels {
        x_secondary.set_skip_overlapping_labels(6.);
    }

    // --------------------------------------------------------
    // 3. Left Axis
    // --------------------------------------------------------
    let y_scale_1 = Linear::new(0., 100.);
    let mut y_left = Axis::new(y_scale_1, axis::Position::Left)
        .with_thickness(50.0)
        .with_tick_renderer(simple_tick_result())
        .with_marker_renderer(simple_dynamic_marker())
        .style(|style| {
            style.spine.width = 6.into();
        });
    if skip_overlapping_labels {
        y_left.set_skip_overlapping_labels(6.);
    }
    // --------------------------------------------------------
    // 4. Right Axis
    // --------------------------------------------------------
    let y_scale_2 = Linear::new(-50., 50.);
    let mut y_right = Axis::new(y_scale_2, axis::Position::Right)
        .with_thickness(50.0)
        .with_tick_renderer(simple_tick_result())
        .with_marker_renderer(simple_dynamic_marker())
        .style(|style| {
            style.spine.width = 10.into();
        });
    if skip_overlapping_labels {
        y_right.set_skip_overlapping_labels(6.);
    }
    // Register all axes
    state.set_axis(AxesShowcase::X_MAIN, x_main);
    state.set_axis(AxesShowcase::X_SECONDARY, x_secondary);
    state.set_axis(AxesShowcase::Y_LEFT, y_left);
    state.set_axis(AxesShowcase::Y_RIGHT, y_right);

    state
}

// --- Renderers ---

fn simple_dynamic_marker() -> impl Fn(MarkerContext<f64>) -> Option<Marker> + 'static {
    move |ctx: MarkerContext<f64>| {
        let badge_color = if ctx.value <= 50.0 {
            ctx.theme.palette().warning
        } else {
            ctx.theme.palette().danger
        };

        let default_marker = ctx.marker(format!("{:.2}", ctx.value));

        Some(Marker {
            badge: MarkerBadge {
                background: badge_color,
                ..default_marker.badge
            },
            ..default_marker
        })
    }
}

fn advanced_dynamic_marker() -> impl Fn(MarkerContext<f64>) -> Option<Marker> + 'static {
    move |ctx: MarkerContext<f64>| {
        let lerp_color = color_lerped(
            &ctx.theme.palette().danger,
            &ctx.theme.palette().warning,
            ctx.normalized_position,
        );

        let label = Label::from_style(
            format!("{:.2}", ctx.value),
            LabelStyle {
                size: 12.into(),
                color: ctx.theme.palette().text,
                padding: 4.into(),
                line_height: LineHeight::Relative(1.0),
            },
        );

        let line = MarkerLine {
            color: lerp_color,
            width: 1.into(),
            gap: 4.into(),
        };

        let badge = MarkerBadge {
            background: lerp_color,
            border: Border::default().rounded(4.),
            shadow: Shadow::default(),
        };

        Some(Marker { label, badge, line })
    }
}

fn simple_tick_result() -> impl Fn(TickContext<f64>) -> TickResult + 'static {
    move |ctx: TickContext<f64>| {
        let text = format!("{:.0}", ctx.tick.value);
        TickResult {
            label: Some(ctx.label(text)),
            tick_line: Some(ctx.tickline()),
            grid_line: Some(ctx.gridline()),
            label_priority: None,
        }
    }
}

fn advanced_tick_result() -> impl Fn(TickContext<f64>) -> TickResult + 'static {
    move |ctx: TickContext<f64>| {
        // Gradient color based on position (0.0 to 1.0)
        let lerp_color = color_lerped(
            &ctx.theme.palette().primary,
            &ctx.theme.palette().success,
            ctx.normalized_position,
        );

        let label = Label::from_style(
            format!("{:.0}", ctx.tick.value),
            LabelStyle {
                color: lerp_color,
                padding: 4.into(),
                size: 10.into(), // Slightly smaller font
                line_height: LineHeight::Relative(1.0),
            },
        );

        let tick_line = TickLine {
            color: lerp_color,
            width: 2.into(), // Thicker ticks
            length: 6.into(),
        };

        let grid_line = GridLine {
            color: ctx.theme.extended_palette().background.neutral.color,
            width: 1.into(),
            dashed: Some(DashStyle::new(4., 4.)), // Dashed grid
        };

        // Render label only for major ticks (Level 0)
        let label = if ctx.tick.level == 0 {
            Some(label)
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

fn color_lerped(start: &Color, end: &Color, v: f32) -> Color {
    let t = v.clamp(0.0, 1.0);
    Color {
        r: start.r + (end.r - start.r) * t,
        g: start.g + (end.g - start.g) * t,
        b: start.b + (end.b - start.b) * t,
        a: start.a + (end.a - start.a) * t,
    }
}
