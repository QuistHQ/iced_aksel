use iced::widget::container;
use iced::widget::text::{LineHeight, Shaping};
use iced::{
    Color, Element, Font, Length, Theme,
    widget::{column, text},
};
use iced_aksel::axis::{CursorBadge, CursorLine, CursorResult};
use iced_aksel::style::TextStyle;
use iced_aksel::{
    Axis, Chart, State,
    axis::{self, GridLine, TickLine, TickResult},
    plot::{Plot, PlotData},
    scale::Linear,
    style::{self, Style},
};

// --- CONFIGURATION ---
const ACCENT_COLOR: Color = Color::from_rgb(0.0, 0.9, 0.9); // Cyan/Teal
const GRID_COLOR: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.05); // Very faint
const TEXT_COLOR: Color = Color::from_rgb(0.7, 0.7, 0.7); // Soft gray

pub fn main() -> iced::Result {
    iced::application(
        MinimalShowcase::new,
        MinimalShowcase::update,
        MinimalShowcase::view,
    )
    .title("Polished Axes")
    .theme(Theme::Dark)
    .antialiasing(true)
    .run()
}

struct MinimalShowcase {
    state: State<&'static str, f64>,
    empty_data: EmptyData,
}

#[derive(Debug, Clone)]
pub enum Message {}

impl MinimalShowcase {
    const X: &'static str = "x";
    const Y: &'static str = "y";

    fn new() -> (Self, iced::Task<Message>) {
        (
            Self {
                state: setup_axes(),
                empty_data: EmptyData,
            },
            iced::Task::none(),
        )
    }

    fn update(&mut self, _message: Message) -> iced::Task<Message> {
        iced::Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let chart = Chart::new(&self.state)
            .plot_data(&self.empty_data, Self::X, Self::Y)
            .style(Box::new(style_base));

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

fn setup_axes() -> State<&'static str, f64> {
    let mut state = State::new();

    // X-Axis: Clean, with subtle vertical grids
    state.set_axis(
        MinimalShowcase::X,
        Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom)
            .with_thickness(40.0)
            .with_tick_renderer(|ctx| {
                let mut result = TickResult::new();
                // Styling logic
                if ctx.tick.level == 0 {
                    result = result
                        .label(format!("{:.0}", ctx.tick.value))
                        .text_style(TextStyle)
                        .tick_line(TickLine {
                            thickness: 1.0.into(),
                            length: 6.0.into(),
                            color: TEXT_COLOR,
                        })
                        .grid_line(GridLine {
                            thickness: 1.0.into(),
                            dashed: false,
                            color: GRID_COLOR,
                        });
                } else {
                    result = TickResult::new();
                }

                result
            })
            // Interactive Cursor
            .with_cursor_renderer(|val| {
                Some(
                    CursorResult::new(format!("{:.1}", val))
                        .line(CursorLine {
                            color: ACCENT_COLOR,
                            width: 1.0.into(),
                            gap: 2.0.into(),
                        })
                        .badge(CursorBadge {
                            background: Some(iced::Background::Color(Color::BLACK)),
                            border: Some(iced::Border {
                                color: ACCENT_COLOR,
                                width: 1.0.into(),
                                radius: 4.0.into(),
                            }),
                            text_style: style::TextStyle {
                                color: ACCENT_COLOR,
                                font: Font::MONOSPACE,
                                shaping: Shaping::Auto,
                                line_height: LineHeight::Relative(1.2),
                                size: 12.into(),
                            },
                            ..Default::default()
                        }),
                )
            }),
    );

    // Y-Axis: Right-aligned for a modern dashboard look
    state.set_axis(
        MinimalShowcase::Y,
        Axis::new(Linear::new(0.0, 100.0), axis::Position::Right)
            .with_thickness(50.0)
            .with_tick_renderer(|ctx| {
                if ctx.tick.level == 0 {
                    TickResult::with_label(format!("{:.0}%", ctx.tick.value))
                        .grid_line(GridLine {
                            thickness: 1.0.into(),
                            dashed: true,
                            color: GRID_COLOR,
                        })
                        .tick_line() // Hide ticks on Y, just grid & labels
                } else {
                    TickResult::new()
                }
            })
            .with_cursor_renderer(|val| {
                Some(
                    CursorResult::new(format!("{:.2}%", val))
                        .line(CursorLine {
                            color: ACCENT_COLOR,
                            width: 1.0.into(),
                            gap: 0.0.into(),
                        })
                        .badge(CursorBadge {
                            background: Some(iced::Background::Color(ACCENT_COLOR)),
                            text_style: style::TextStyle {
                                color: Color::WHITE,
                                font: Font::MONOSPACE,
                                shaping: Shaping::Auto,
                                line_height: LineHeight::Relative(1.2),
                                size: 12.into(),
                            },
                            border: None,
                            ..Default::default()
                        }),
                )
            }),
    );

    state
}

// -----------------------------------------------------------------------------
// HELPERS
// -----------------------------------------------------------------------------

struct EmptyData;

// We implement this trait but leave 'draw' empty to render nothing.
impl PlotData<f64> for EmptyData {
    fn draw(&self, _plot: &mut Plot<f64>, _theme: &Theme) {
        // Intentionally empty: The "Zen" of data visualization.
    }
}

fn style_base(theme: &Theme) -> Style {
    let mut style = iced_aksel::style::default(theme);
    // Remove the outer frame for a cleaner look
    style.axis.line.width = 0.0.into();
    style
}
