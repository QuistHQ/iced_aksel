use iced::{
    Color, Element, Length, Theme,
    widget::{checkbox, column, container, pick_list, row, slider, text},
};
use iced_aksel::{
    Axis, Chart, Measure, PlotPoint, State, Stroke,
    axis::{self, GridLine, LabelDecision, TickLine, TickResult},
    plot::{Plot, PlotData},
    scale::Linear,
    shape::Polyline,
};

// =============================================================================
//  APPLICATION ENTRY
// =============================================================================

pub fn main() -> iced::Result {
    iced::application(AxesExample::new, AxesExample::update, AxesExample::view)
        .title("Axes Configuration")
        .theme(Theme::Dark)
        .antialiasing(true)
        .run()
}

// =============================================================================
//  APPLICATION STATE
// =============================================================================

pub struct AxesExample {
    // The chart state holds the configured axes
    chart_state: State<&'static str, f64>,
    // The data to render
    data: DataLayer,
    // The UI settings that control the axis configuration
    settings: AxisSettings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickStyle {
    Simple,
    OnlyMajor,
}

impl std::fmt::Display for TickStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Simple => write!(f, "Simple"),
            Self::OnlyMajor => write!(f, "Only Major"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct AxisSettings {
    // Visibility & Layout
    x_visible: bool,
    y_visible: bool,
    y_position_right: bool,
    show_grid: bool,

    // Logic & Interaction
    show_cursor: bool,   // Renders a label on the axis when hovering
    sparse_labels: bool, // Uses a custom closure to skip specific ticks
    skip_overlap: bool,  // Automatically hides labels that collide

    // Styling
    tick_style: TickStyle,
    thickness: f32,
    label_spacing: f32,
}

impl Default for AxisSettings {
    fn default() -> Self {
        Self {
            x_visible: true,
            y_visible: true,
            y_position_right: false,
            show_grid: true,
            show_cursor: true,
            sparse_labels: false,
            skip_overlap: true,
            tick_style: TickStyle::Simple,
            thickness: 30.0,
            label_spacing: 5.0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleX(bool),
    ToggleY(bool),
    ToggleYSide(bool),
    ToggleGrid(bool),
    ToggleCursor(bool),
    ToggleSparse(bool),
    ToggleSkipOverlap(bool),
    TickStyleChanged(TickStyle),
    ThicknessChanged(f32),
    SpacingChanged(f32),
}

// =============================================================================
//  LOGIC & UPDATES
// =============================================================================

impl AxesExample {
    const X_ID: &'static str = "x";
    const Y_ID: &'static str = "y";

    fn new() -> (Self, iced::Task<Message>) {
        let mut app = Self {
            chart_state: State::new(),
            data: DataLayer::new(),
            settings: AxisSettings::default(),
        };
        // Perform initial configuration
        app.rebuild_axes();
        (app, iced::Task::none())
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
                    content: format!("{:.1}", val),
                    size: 10.into(),
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
        let chart = Chart::new(&self.chart_state).plot_data(&self.data, Self::X_ID, Self::Y_ID);

        let controls = column![
            // --- Section: Visibility ---
            text("Layout & Visibility")
                .size(14)
                .color(Color::from_rgb(0.7, 0.7, 0.7)),
            row![
                checkbox(self.settings.x_visible).on_toggle(Message::ToggleX),
                text("X Axis")
            ]
            .spacing(10),
            row![
                checkbox(self.settings.y_visible).on_toggle(Message::ToggleY),
                text("Y Axis")
            ]
            .spacing(10),
            row![
                checkbox(self.settings.y_position_right).on_toggle(Message::ToggleYSide),
                text("Y Axis on Right")
            ]
            .spacing(10),
            row![
                checkbox(self.settings.show_grid).on_toggle(Message::ToggleGrid),
                text("Grid Lines")
            ]
            .spacing(10),
            // --- Section: Features ---
            text("Labels & Ticks")
                .size(14)
                .color(Color::from_rgb(0.7, 0.7, 0.7)),
            row![
                checkbox(self.settings.show_cursor).on_toggle(Message::ToggleCursor),
                text("Cursor Labels")
            ]
            .spacing(10),
            row![
                checkbox(self.settings.skip_overlap).on_toggle(Message::ToggleSkipOverlap),
                text("Skip Overlapping")
            ]
            .spacing(10),
            row![
                checkbox(self.settings.sparse_labels).on_toggle(Message::ToggleSparse),
                text("Sparse Policy (Custom)")
            ]
            .spacing(10),
            // --- Section: Styling ---
            text("Tick Style")
                .size(14)
                .color(Color::from_rgb(0.7, 0.7, 0.7)),
            pick_list(
                &[TickStyle::Simple, TickStyle::OnlyMajor][..],
                Some(self.settings.tick_style),
                Message::TickStyleChanged
            ),
            text(format!("Thickness: {:.0}px", self.settings.thickness))
                .size(14)
                .color(Color::from_rgb(0.7, 0.7, 0.7)),
            slider(
                10.0..=100.0,
                self.settings.thickness,
                Message::ThicknessChanged
            ),
            text(format!("Spacing: {:.0}px", self.settings.label_spacing))
                .size(14)
                .color(Color::from_rgb(0.7, 0.7, 0.7)),
            slider(
                0.0..=20.0,
                self.settings.label_spacing,
                Message::SpacingChanged
            ),
        ]
        .spacing(12)
        .padding(20)
        .width(250);

        row![
            container(chart)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(20),
            container(controls).height(Length::Fill).style(|t: &Theme| {
                container::Style::default().background(t.extended_palette().background.weak.color)
            })
        ]
        .into()
    }
}

// =============================================================================
//  DATA LAYER
// =============================================================================

// Helper enum to distinguish X/Y during creation
#[derive(Clone, Copy)]
enum AxisKind {
    X,
    Y,
}

struct DataLayer {
    points: Vec<PlotPoint<f64>>,
}

impl DataLayer {
    fn new() -> Self {
        let points = (0..=100)
            .map(|i| {
                let x = i as f64;
                let y = (x * 0.1).sin() * 40.0;
                PlotPoint::new(x, y)
            })
            .collect();
        Self { points }
    }
}

impl PlotData<f64> for DataLayer {
    fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {
        plot.add_shape(
            Polyline::new(self.points.clone())
                .stroke(Stroke::new(theme.palette().primary, Measure::Screen(2.0))),
        );
    }
}
