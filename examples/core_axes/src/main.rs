use iced::Point;
use iced::widget::checkbox;
use iced::widget::text::LineHeight;
use iced::{
    Border, Color, Element, Font, Length, Padding, Shadow, Theme,
    widget::{column, container, pick_list, row, text},
};
use iced_aksel::axis::{
    Marker, MarkerBadge, MarkerContext, MarkerLine, MarkerPosition, TickContext,
};
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
// * **Simple:** Inheriting default styles and selectively overriding properties (e.g., changing badge color based on data thresholds).
// * **Advanced:** Constructing visual elements from scratch for granular control (e.g., creating gradient ticks or custom label formatting).
//
// It also demonstrates axis label rendering management features like `skip_overlapping_labels`.

pub fn main() -> iced::Result {
    iced::application(AxesShowcase::new, AxesShowcase::update, AxesShowcase::view)
        .title("Axes Styling Showcase")
        .theme(AxesShowcase::theme)
        .antialiasing(true)
        .run()
}

struct AxesShowcase {
    theme: Theme,
    state: State<&'static str, f64>,
    cursor_positions: [Option<MarkerPosition<f64>>; 2],

    // Settings
    skip_label_overlapping: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    ThemeChanged(Theme),
    SkipOverlappingToggle(bool),
    PlotHovered(Point),
    AxisHovered(&'static str, f32),
}

impl AxesShowcase {
    const X: &'static str = "x";
    const Y: &'static str = "y";

    fn new() -> (Self, iced::Task<Message>) {
        let theme = Theme::Dark;
        (
            Self {
                theme,
                state: axes_setup(true), // <-- OBS: Inside this function is where the magic starts
                cursor_positions: [None, None],

                skip_label_overlapping: false,
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
            Message::PlotHovered(point) => {
                self.cursor_positions = [
                    Some(MarkerPosition::Normalized(point.x)),
                    Some(MarkerPosition::Normalized(point.y)),
                ]
            }
            Message::AxisHovered(id, position) => match id {
                Self::X => self.cursor_positions[0] = Some(MarkerPosition::Normalized(position)),
                Self::Y => self.cursor_positions[1] = Some(MarkerPosition::Normalized(position)),
                _ => unreachable!("Axis {id} not configured"),
            },
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
        let chart_panel = panel(
            "Axes Showcase",
            Chart::new(&self.state)
                .on_hover(Message::PlotHovered)
                .on_axis_hover(Message::AxisHovered)
                .marker_maybe(&Self::X, self.cursor_positions[0], simple_dynamic_marker)
                .marker_maybe(&Self::Y, self.cursor_positions[1], advanced_dynamic_marker),
        );

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
    // Prepare the general state
    let mut state = State::new();

    // ----- X-Axis -----

    // X-Axis basic settings
    let x_placement = axis::Position::Bottom;
    let x_scale = Linear::new(0., 100.);

    // X-Axis dynamic settings
    let mut x_axis = Axis::new(x_scale, x_placement).with_tick_renderer(simple_tick_result());

    if skip_overlapping_labels {
        x_axis.set_skip_overlapping_labels(6.) // <-- Automatically hides labels that would collide
    }

    // ----- Y-Axis -----
    let y_placement = axis::Position::Left;
    let y_scale = Linear::new(0., 100.);

    // For the Y-Axis, we use the "Advanced" renderers to show full manual control
    let y_axis = if skip_overlapping_labels {
        Axis::new(y_scale, y_placement)
            .skip_overlapping_labels(6.)
            .with_tick_renderer(advanced_tick_result())
    } else {
        Axis::new(y_scale, y_placement).with_tick_renderer(advanced_tick_result())
    };

    state.set_axis(AxesShowcase::X, x_axis);
    state.set_axis(AxesShowcase::Y, y_axis);

    state
}

fn simple_dynamic_marker(ctx: MarkerContext<f64>) -> Option<Marker> {
    if !ctx.cursor_on_plot && !ctx.cursor_on_axis {
        // Never render markers if cursor isn't on plot or axis
        return None;
    }

    // Example: Change color based on data thresholds
    let badge_color = if ctx.value <= 50.0 {
        ctx.theme.palette().warning
    } else {
        ctx.theme.palette().danger
    };

    // --- THE EASY WAY ---
    // We use `ctx.marker(String)` to generate a fully populated default Marker.
    // This allows us to use Rust's struct update syntax (default) to
    // only override the specific fields we want to change.

    let default_marker = ctx.marker(format!("{:.2}", ctx.value));

    Some(Marker {
        badge: MarkerBadge {
            background: badge_color, // Override only the background color
            ..default_marker.badge   // Keep the rest (border, shadow, radius) default
        },
        ..default_marker // Keep the label and line styles default
    })
}

fn advanced_dynamic_marker(ctx: MarkerContext<f64>) -> Option<Marker> {
    // --- THE MANUAL WAY ---
    // For full control, we define every aspect of the marker manually.
    // A marker consists of 3 parts: Label, Badge, and Line.
    if !ctx.cursor_on_plot && !ctx.cursor_on_axis {
        // Never render markers if cursor isn't on plot or axis
        return None;
    }

    let lerp_color = color_lerped(
        &ctx.theme.palette().danger,
        &ctx.theme.palette().warning,
        ctx.normalized_position,
    );

    // 1. Label: The text content and its font styling
    let label_text = format!("{:.2}", ctx.value);
    let label_style = LabelStyle {
        size: 12.into(),
        color: ctx.theme.palette().text,
        padding: 4.into(),
        line_height: LineHeight::Relative(1.0),
    };
    let label = Label::from_style(label_text, label_style);

    // 2. Line: The visual connector between the plot data and the badge
    let line = MarkerLine {
        color: lerp_color,
        width: 1.into(),
        gap: 4.into(),
    };

    // 3. Badge: The container/box surrounding the text
    let badge = MarkerBadge {
        background: lerp_color,
        border: Border::default().rounded(4.),
        shadow: Shadow::default(),
    };

    Some(Marker { label, badge, line })
}

fn simple_tick_result() -> impl Fn(TickContext<f64>) -> TickResult + 'static {
    move |ctx: TickContext<f64>| {
        let text = format!("{:.2}", ctx.tick.value);
        let label = ctx.label(text);

        TickResult {
            label: Some(label),
            tick_line: Some(ctx.tickline()),
            grid_line: Some(ctx.gridline()),
            label_priority: None,
        }
    }
}

fn advanced_tick_result() -> impl Fn(TickContext<f64>) -> TickResult + 'static {
    move |ctx: TickContext<f64>| {
        // The library categorizes ticks by "levels". Level 0 is a Major tick.
        let is_major_tick = ctx.tick.level == 0;

        let label_text = format!("{:.2}", ctx.tick.value);

        // Example: Create a color gradient based on the tick's position on the axis.
        // ctx.normalized_position gives us a value between 0.0 (start) and 1.0 (end).
        let lerp_color = color_lerped(
            &ctx.theme.palette().danger,
            &ctx.theme.palette().warning,
            ctx.normalized_position,
        );

        let label_style = LabelStyle {
            color: lerp_color,
            padding: 4.into(),
            size: 12.into(),
            line_height: LineHeight::Relative(1.0),
        };

        let label = Label::from_style(label_text, label_style);

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
        // We only return a Label if it is a Major tick.
        // However, we still return tick_lines and grid_lines for minor ticks.
        if is_major_tick {
            TickResult {
                label: Some(label),
                tick_line: Some(tick_line),
                grid_line: Some(grid_line),
                label_priority: None,
            }
        } else {
            TickResult {
                label: None, // <-- Hides the text for minor ticks
                tick_line: Some(tick_line),
                grid_line: Some(grid_line),
                label_priority: None,
            }
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
