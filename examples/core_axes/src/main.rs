use iced::{
    Border, Color, Element, Font, Length, Padding, Pixels, Shadow, Theme,
    widget::{column, container, row, text},
};
use iced_aksel::{
    Axis, Chart, Measure, PlotPoint, State, Stroke,
    axis::{self, GridLine, TickLine, TickResult},
    plot::{Plot, PlotData},
    scale::Linear,
    shape::Polyline,
    style::{AxisStyle, Style},
};

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
    const X: &'static str = "x";
    const Y: &'static str = "y";

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

    /// This function applies the current `settings` to the Chart State.
    /// It demonstrates the "Builder Pattern" used to configure axes.
    fn rebuild_axes(&mut self) {
        let x_axis = self.configure_axis(AxisKind::X);
        let y_axis = self.configure_axis(AxisKind::Y);

        self.chart_state.set_axis(Self::X_ID, x_axis);
        self.chart_state.set_axis(Self::Y_ID, y_axis);
    }

    /// A helper that acts as a factory for Axis configuration.
    /// It demonstrates the standard pipeline:
    /// Scale -> Position -> Dimensions -> Logic -> Visuals
    fn configure_axis(&self, kind: AxisKind) -> Axis<f64> {
        let s = self.settings;

        // ---------------------------------------------------------------------
        // 1. BASE CONFIGURATION
        //    Set up the scale (domain) and where the axis sits on the screen.
        // ---------------------------------------------------------------------
        let (scale, position) = match kind {
            AxisKind::X => (Linear::new(0.0, 100.0), axis::Position::Bottom),
            AxisKind::Y => {
                let pos = if s.y_position_right {
                    axis::Position::Right
                } else {
                    axis::Position::Left
                };
                (Linear::new(-50.0, 50.0), pos)
            }
        };

        let mut axis = Axis::new(scale, position)
            .with_thickness(s.thickness)
            .with_label_spacing(s.label_spacing)
            .without_grid(); // Start clean so we can toggle it conditionally later

        // ---------------------------------------------------------------------
        // 2. VISIBILITY
        // ---------------------------------------------------------------------
        // If the user wants this axis hidden, we hide it and stop configuring.
        let is_visible = match kind {
            AxisKind::X => s.x_visible,
            AxisKind::Y => s.y_visible,
        };

        if !is_visible {
            return axis.invisible();
        }

        // ---------------------------------------------------------------------
        // 3. LABEL POLICY (Collision Avoidance)
        // ---------------------------------------------------------------------
        if s.sparse_labels {
            // OPTION A: Manual Logic
            // "I want full control over exactly which numbers appear."
            axis = axis.with_custom_label_policy(move |ctx| {
                let val = ctx.tick.value as i32;

                // Example: Only show even numbers on X, multiples of 10 on Y
                let keep_label = match kind {
                    AxisKind::X => val % 2 == 0,
                    AxisKind::Y => val % 10 == 0,
                };

                if keep_label {
                    LabelDecision::Render
                } else {
                    LabelDecision::Skip
                }
            });
        } else if s.skip_overlap {
            // OPTION B: Automatic Logic
            // "Just make sure they don't touch each other."
            axis = axis.skip_overlapping_labels(10.0);
        }

        // ---------------------------------------------------------------------
        // 4. INTERACTIVITY
        // ---------------------------------------------------------------------
        if s.show_cursor {
            axis = axis.with_cursor_formatter(|val| {
                Some(axis::Label {
                    content: format!("{:.0}", val),
                    size: 12.into(),
                    ..Default::default()
                })
            });
        }

        // ---------------------------------------------------------------------
        // 5. VISUALS (Ticks, Grids & Labels)
        //    This uses the modern `TickResult` pattern to return all parts at once.
        // ---------------------------------------------------------------------
        axis = match s.tick_style {
            // STYLE 1: Render everything
            TickStyle::Simple => axis.with_tick_renderer(move |ctx| {
                // 1. Prepare the Grid (only if enabled)
                let grid = if s.show_grid {
                    Some(GridLine::default())
                } else {
                    None
                };

                // 2. Return the result
                TickResult {
                    tick_line: Some(TickLine::default()),
                    grid_line: grid,
                    label: Some(format!("{:.0}", ctx.tick.value).into()),
                }
            }),

            // STYLE 2: Only render Major ticks (Level 0)
            TickStyle::OnlyMajor => axis.with_tick_renderer(move |ctx| {
                // 1. Filter: If this is a minor tick (level > 0), render nothing.
                if ctx.tick.level > 0 {
                    return TickResult::default();
                }

                // 2. Prepare the Grid (only if enabled)
                let grid = if s.show_grid {
                    Some(GridLine::default())
                } else {
                    None
                };

                // 3. Return the result (Major ticks only)
                TickResult {
                    tick_line: Some(TickLine::default()),
                    grid_line: grid,
                    label: Some(format!("{:.0}", ctx.tick.value).into()),
                }
            }),
        };

        axis
    }

    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::ToggleX(v) => self.settings.x_visible = v,
            Message::ToggleY(v) => self.settings.y_visible = v,
            Message::ToggleYSide(v) => self.settings.y_position_right = v,
            Message::ToggleGrid(v) => self.settings.show_grid = v,
            Message::ToggleCursor(v) => self.settings.show_cursor = v,
            Message::ToggleSparse(v) => self.settings.sparse_labels = v,
            Message::ToggleSkipOverlap(v) => self.settings.skip_overlap = v,
            Message::TickStyleChanged(v) => self.settings.tick_style = v,
            Message::ThicknessChanged(v) => self.settings.thickness = v,
            Message::SpacingChanged(v) => self.settings.label_spacing = v,
        }
        self.rebuild_axes();
        iced::Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        row![
            self.panel(
                "1. Minimal Layout",
                "Hidden Y-axis. No Grid.",
                Chart::new(&self.minimal_state)
                    .plot_data(&self.minimal_data, Self::X, Self::Y)
                    .style(Box::new(style_base))
            ),
            self.panel(
                "2. Engineering Layout",
                "Custom Ruler Ticks. Monospace.",
                Chart::new(&self.engineering_state)
                    .plot_data(&self.engineering_data, Self::X, Self::Y)
                    .style(Box::new(style_engineering))
            ),
            self.panel(
                "3. Custom Placement",
                "Top & Right Axes. Badges.",
                Chart::new(&self.custom_state)
                    .plot_data(&self.custom_data, Self::X, Self::Y)
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

// -----------------------------------------------------------------------------
// 1. MINIMAL CONFIGURATION
// -----------------------------------------------------------------------------

fn setup_minimal_axes() -> State<&'static str, f64> {
    let mut state = State::new();

    // X-Axis: Standard look, no grid
    state.set_axis(
        AxesShowcase::X,
        Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom)
            .with_thickness(45.0)
            .without_grid()
            .with_cursor_formatter(|v| Some(format!("{:.0}", v))),
    );

    // Y-Axis: Invisible but active for scaling
    state.set_axis(
        AxesShowcase::Y,
        Axis::new(Linear::new(-1.2, 1.2), axis::Position::Left).invisible(),
    );

    state
}

// -----------------------------------------------------------------------------
// 2. ENGINEERING CONFIGURATION
// -----------------------------------------------------------------------------

fn setup_engineering_axes() -> State<&'static str, f64> {
    let mut state = State::new();

    // Define a custom renderer for a technical "ruler" look
    let ruler_renderer = |ctx: axis::TickContext<f64>| {
        if ctx.tick.level == 0 {
            TickResult {
                tick_line: Some(TickLine {
                    length: 8.0.into(),
                    thickness: 1.5.into(),
                }),
                grid_line: Some(GridLine {
                    thickness: 1.0.into(),
                }),
                label: Some(format!("{:.1}", ctx.tick.value)),
                ..Default::default()
            }
        } else {
            TickResult {
                tick_line: Some(TickLine {
                    length: 4.0.into(),
                    thickness: 1.0.into(),
                }),
                grid_line: Some(GridLine {
                    thickness: 0.5.into(),
                }),
                label: None,
                ..Default::default()
            }
        }
    };

    state.set_axis(
        AxesShowcase::Y,
        Axis::new(Linear::new(-4.0, 4.0), axis::Position::Left)
            .with_thickness(50.0)
            .with_tick_renderer(ruler_renderer),
    );

    state.set_axis(
        AxesShowcase::X,
        Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom)
            .with_thickness(35.0)
            .with_tick_renderer(ruler_renderer),
    );

    state
}

// -----------------------------------------------------------------------------
// 3. CUSTOM PLACEMENT & BADGES
// -----------------------------------------------------------------------------

fn setup_custom_axes() -> State<&'static str, f64> {
    let mut state = State::new();

    // X-Axis on TOP
    state.set_axis(
        AxesShowcase::X,
        Axis::new(Linear::new(0.0, 100.0), axis::Position::Top)
            .with_thickness(45.0)
            .with_cursor_formatter(|v| Some(format!("T: {:.1}s", v))),
    );

    // Y-Axis on RIGHT
    state.set_axis(
        AxesShowcase::Y,
        Axis::new(Linear::new(-1.5, 1.5), axis::Position::Right)
            .with_thickness(55.0)
            .with_tick_renderer(|ctx| {
                // Show grid only for major ticks
                let is_major = ctx.tick.level == 0;
                TickResult {
                    grid_line: Some(GridLine {
                        thickness: if is_major { 1.0.into() } else { 0.0.into() },
                    }),
                    tick_line: Some(TickLine {
                        thickness: 1.0.into(),
                        length: 4.0.into(),
                    }),
                    label: if is_major {
                        Some(format!("{:.1}", ctx.tick.value))
                    } else {
                        None
                    },
                    ..Default::default()
                }
            })
            .with_cursor_formatter(|v| Some(format!("{:.3}", v))),
    );

    state
}

// -----------------------------------------------------------------------------
// STYLES
// -----------------------------------------------------------------------------

/// Base style that strictly adheres to the current Theme
fn style_base(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    let mut style = iced_aksel::style::default(theme);

    // Ensure we use the theme's text color
    style.axis.label.color = palette.background.strong.text;
    style.axis.ticks.color = palette.background.strong.text;

    // Use primary color for interaction elements
    style.axis.cursor.color = palette.primary.base.color;
    style.axis.cursor.badge.background = palette.primary.base.color;
    style.axis.cursor.text.color = palette.primary.base.text;

    style.plot_cursor.color = palette.primary.base.color;

    style
}

/// Engineering style: Monospace font, but same colors
fn style_engineering(theme: &Theme) -> Style {
    let mut style = style_base(theme);

    style.axis.label.font = Font::MONOSPACE;
    style.axis.cursor.text.font = Font::MONOSPACE;

    style
}

// -----------------------------------------------------------------------------
// DATA GENERATION
// -----------------------------------------------------------------------------

struct SineWave {
    points: Vec<PlotPoint<f64>>,
}

impl SineWave {
    fn new(frequency: f64, amplitude: f64, count: usize) -> Self {
        let points = (0..=count)
            .map(|i| {
                let x = (i as f64 / count as f64) * 100.0;
                let y = (x * 0.1 * frequency).sin() * amplitude;
                PlotPoint::new(x, y)
            })
            .collect();

        Self { points }
    }
}

impl PlotData<f64> for SineWave {
    fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {
        let palette = theme.extended_palette();

        plot.add_shape(Polyline::new(self.points.clone()).stroke(Stroke::new(
            palette.primary.base.color,
            Measure::Screen(2.0),
        )));
    }
}
