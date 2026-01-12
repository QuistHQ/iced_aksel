mod shared;

use iced::{
    Color, Element, Length, Padding, Theme,
    widget::{column, container, row, text},
};
use iced_aksel::{Chart, State};
use shared::*;

pub fn main() -> iced::Result {
    iced::application(AxesShowcase::new, AxesShowcase::update, AxesShowcase::view)
        .title("Axes Styling Showcase")
        .theme(Theme::Dark)
        .antialiasing(true)
        .run()
}

struct AxesShowcase {
    minimal_state: State<&'static str, f64>,
    minimal_data: SineWave,

    engineering_state: State<&'static str, f64>,
    engineering_data: SineWave,

    custom_state: State<&'static str, f64>,
    custom_data: SineWave,
}

#[derive(Debug, Clone)]
pub enum Message {}

impl AxesShowcase {
    fn new() -> (Self, iced::Task<Message>) {
        (
            Self {
                minimal_state: setup_minimal_axes(),
                minimal_data: SineWave::new(1.0, 0.8, 50),

                engineering_state: setup_engineering_axes(),
                engineering_data: SineWave::new(2.5, 3.5, 100),

                custom_state: setup_custom_axes(),
                custom_data: SineWave::new(1.5, 0.8, 80),
            },
            iced::Task::none(),
        )
    }

    fn update(&mut self, _message: Message) -> iced::Task<Message> {
        iced::Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        row![
            self.panel(
                "1. Minimal Layout",
                "Hidden Y-axis. No Grid.",
                Chart::new(&self.minimal_state)
                    .plot_data(&self.minimal_data, X, Y)
                    .style(Box::new(style_base))
            ),
            self.panel(
                "2. Engineering Layout",
                "Custom Ruler Ticks. Monospace.",
                Chart::new(&self.engineering_state)
                    .plot_data(&self.engineering_data, X, Y)
                    .style(Box::new(style_engineering))
            ),
            self.panel(
                "3. Custom Placement",
                "Top & Right Axes. Badges.",
                Chart::new(&self.custom_state)
                    .plot_data(&self.custom_data, X, Y)
                    .style(Box::new(style_base))
            ),
        ]
        .spacing(20)
        .padding(20)
        .into()
    }

    fn panel<'a>(
        &self,
        title: &'a str,
        subtitle: &'a str,
        chart: Chart<'a, &'static str, f64, Message>,
    ) -> Element<'a, Message> {
        use iced::Font;

        column![
            text(title).size(16).font(Font::MONOSPACE),
            text(subtitle)
                .size(12)
                .color(Color::from_rgb(0.6, 0.6, 0.6)),
            container(chart)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|t: &Theme| container::Style::default()
                    .background(t.extended_palette().background.weak.color)
                    .border(iced::Border {
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
}
