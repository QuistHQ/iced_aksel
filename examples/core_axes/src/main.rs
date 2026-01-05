use iced::theme::Base;
use iced::widget::container;
use iced::{
    Color, Element, Font, Length, Theme,
    widget::{column, text},
};
use iced_aksel::axis::{CursorResult, Position, TickResult};
use iced_aksel::{Axis, Chart, State, scale::Linear};

pub fn main() -> iced::Result {
    iced::application(
        MinimalShowcase::new,
        MinimalShowcase::update,
        MinimalShowcase::view,
    )
    .theme(MinimalShowcase::theme)
    .run()
}

struct MinimalShowcase {
    theme: Theme,
    state: State<&'static str, f64>,
}

#[derive(Debug, Clone)]
pub enum Message {}

impl MinimalShowcase {
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
        let chart = Chart::new(&self.state)
            .width(Length::Fill)
            .height(Length::Fill);
        container(chart).padding(40).into()
    }
}

fn setup_axes(theme: &Theme) -> State<&'static str, f64> {
    let mut state = State::new();
    let danger_color = theme.palette().danger;

    // 1. SIMPLE CUSTOMIZATION
    // "I just want something to be shown on the screen"
    state.set_axis(
        "y",
        Axis::new(Linear::new(0.0, 100.0), Position::Left).with_tick_renderer(|ctx| {
            TickResult::from_tick_style(*ctx.tick_style)
                .label(format!("{:.0}", ctx.tick.value))
                .grid_style(*ctx.grid_style)
        }),
    );

    // 2. ADVANCED LOGIC
    // "I want the ticks to be shown differently based on tick.level"
    state.set_axis(
        "x",
        Axis::new(Linear::new(0.0, 100.0), Position::Bottom)
            .with_text_offset(32)
            .with_tick_renderer(move |ctx| {
                // Take the theme defaults
                let mut tick_style = *ctx.tick_style;
                let grid_style = *ctx.grid_style;

                // Modify based on data
                // This will make the major tick-lines stand out from the minor
                if ctx.tick.level == 0 {
                    tick_style.length = 6.0.into();
                    tick_style.line.color = danger_color; // Red
                } else {
                    tick_style.length = 4.0.into();
                }

                TickResult::from_tick_style(tick_style)
                    .label(format!("{:.0}", ctx.tick.value))
                    .grid_style(grid_style)
            })
            .with_cursor_renderer(|val, style| {
                // Receive the 'style' which is correctly colored for the current Theme
                Some(CursorResult::from_style(*style).label(format!("{:.1}", val)))
            }),
    );

    state
}
