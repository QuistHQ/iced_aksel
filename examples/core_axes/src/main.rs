use iced::widget::container;
use iced::widget::text::{LineHeight, Shaping};
use iced::{
    Color, Element, Font, Length, Shadow, Theme,
    widget::{column, text},
};
use iced_aksel::axis::{CursorBadge, CursorLine, CursorResult};
use iced_aksel::style::{Style, TextStyle};
use iced_aksel::{
    Axis, Chart, State,
    axis::{self, GridLine, TickLine, TickResult},
    scale::Linear,
};

pub fn main() -> iced::Result {
    iced::application(
        MinimalShowcase::new,
        MinimalShowcase::update,
        MinimalShowcase::view,
    )
    .title("Axes Showcase")
    .theme(MinimalShowcase::theme)
    .antialiasing(true)
    .run()
}

struct MinimalShowcase {
    theme: Theme,
    state: State<&'static str, f64>,
}

#[derive(Debug, Clone)]
pub enum Message {}

impl MinimalShowcase {
    const X: &'static str = "x";
    const Y: &'static str = "y";

    fn new() -> (Self, iced::Task<Message>) {
        let theme = Theme::Dark;

        (
            Self {
                state: setup_axes(&theme),
                theme,
            },
            iced::Task::none(),
        )
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn update(&mut self, _message: Message) -> iced::Task<Message> {
        iced::Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let chart = Chart::new(&self.state);

        container(
            column![
                text("Interactive Axis Playground")
                    .size(14)
                    .font(Font::MONOSPACE)
                    .style(text::secondary),
                container(chart)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(container::bordered_box)
            ]
            .spacing(10),
        )
        .padding(40)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

// -----------------------------------------------------------------------------
// AXIS SETUP
// -----------------------------------------------------------------------------

fn setup_axes(theme: &Theme) -> State<&'static str, f64> {
    let mut state = State::new();

    // X-Axis: Clean, with subtle vertical grids

    // ----- General Settings -----
    let text_style = TextStyle {
        color: theme.palette().text,
        font: Font::MONOSPACE,
        shaping: Shaping::Auto,
        line_height: LineHeight::Relative(1.2),
        size: 12.into(),
    };

    // ----- Tick Settings -----
    let tick_line_style = TickLine {
        thickness: 1.0.into(),
        length: 6.0.into(),
        color: theme.palette().text,
    };

    let grid_line_style = GridLine {
        thickness: 1.0.into(),
        dashed: false,
        color: theme.palette().background,
    };

    // ----- Cursor Settings -----
    let cursor_line = CursorLine {
        color: theme.palette().primary,
        width: 1.0.into(),
        gap: 2.0.into(),
    };

    let cursor_badge = CursorBadge {
        background: Some(theme.palette().primary),
        border: Some(iced::Border {
            color: theme.palette().primary,
            width: 1.0.into(),
            radius: 4.0.into(),
        }),
        text_style,
        shadow: Some(Shadow::default()),
        padding: 4.into(),
    };
    state.set_axis(
        MinimalShowcase::X,
        Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom)
            .with_thickness(40.0)
            .with_text_offset(16.0.into())
            .with_tick_renderer(move |ctx| {
                let mut result = TickResult::empty();

                // Styling logic
                if ctx.tick.level == 0 {
                    result = result
                        .label(format!("{:.0}", ctx.tick.value))
                        .text_style(text_style)
                        .tick_line(tick_line_style)
                        .grid_line(grid_line_style);
                } else {
                    result = TickResult::empty();
                }

                result
            })
            // Interactive Cursor
            .with_cursor_renderer(move |val| {
                Some(
                    CursorResult::empty()
                        .cursor_badge(cursor_badge)
                        .cursor_line(cursor_line)
                        .label(format!("{:.0}", val)),
                )
            }),
    );

    state
}
