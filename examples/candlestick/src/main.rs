//! Chart shape layer example
//!
//! An expanded example of the candlestick chart, now including:
//! - A synchronized volume panel.
//! - Toggleable SMA (Simple Moving Average) indicator.
//! - Toggleable Bollinger Bands (BBands) indicator.
//! - An interactive settings bar with checkboxes and text inputs.

use std::{collections::BTreeMap, ops::RangeInclusive};

use aksel::{PlotPoint, scale::Linear};
use chrono::TimeZone;
use iced::{
    Color, Element, Point, Subscription, Task, Theme,
    mouse::ScrollDelta,
    theme::palette::Extended,
    time::Instant,
    widget::{checkbox, column, pick_list, row, text, text_input},
    window,
};
use iced_aksel::{
    Axis, Chart, DragDelta, Length, Plot, State, Stroke, StrokeStyle,
    axis::{Position, TickLabelContext, TickLine},
    plot, shape,
};

// --- Constants ---

const X_AXIS_ID: AxisId = "x_axis";
const Y_AXIS_ID: AxisId = "y_axis";
const Y_VOL_AXIS_ID: AxisId = "y_vol_axis";

// --- Data ---
const CANDLES_AMOUNT: usize = 100_000;

// --- Shared Types ---

/// Type alias for the unique ID of a chart axis.
pub type AxisId = &'static str;

/// Defines the messages that can be sent to update the application state.
#[derive(Debug, Clone)]
enum Message {
    Tick(Instant),
    SwitchTheme(iced::Theme),
    /// A message to trigger a full rebuild of the chart layers.
    UpdateChart,
    /// A message sent when the main plot area is dragged.
    OnPlotDrag(DragDelta),
    OnPlotDoubleClick(Point),
    OnPlotScroll(iced::Point, ScrollDelta),
    /// A message sent when an axis is dragged (for zooming).
    OnAxisDrag(AxisId, f32),
    OnAxisDoubleClick(AxisId, f32),

    // --- New Messages for UI Settings ---
    /// Toggles the visibility of the Volume panel.
    VolumeToggled(bool),
    /// Toggles the visibility of the SMA indicator.
    SmaToggled(bool),
    /// Reports a change in the SMA period input field.
    SmaPeriodChanged(String),
    /// Toggles the visibility of the Bollinger Bands indicator.
    BBandToggled(bool),
    /// Reports a change in the BBands period input field.
    BBandPeriodChanged(String),
    /// Reports a change in the BBands standard deviation input field.
    BBandStdDevChanged(String),
}

// --- Main Entry Point ---

fn main() -> iced::Result {
    ExampleApp::run()
}

// --- Application Structure ---

/// Represents the top-level state of the iced application.
struct ExampleApp {
    candlestick_chart: CandlestickChart,

    // --- UI State ---
    current_theme: iced::Theme,
    show_volume: bool,
    show_sma: bool,
    sma_period_str: String,
    show_bband: bool,
    bband_period_str: String,
    bband_std_dev_str: String,
    // FPS counter
    last_frame_time: Option<Instant>,
    fps: f32,
}

impl ExampleApp {
    /// Initializes the application and its component.
    fn init() -> (Self, Task<Message>) {
        // --- Default Settings ---
        let default_settings = ChartSettings::default();

        let app = Self {
            current_theme: iced::Theme::Dark,
            candlestick_chart: CandlestickChart::new(default_settings),
            // Default UI state matches the default settings
            show_volume: default_settings.show_volume,
            show_sma: default_settings.show_sma,
            sma_period_str: default_settings.sma_period.to_string(),
            show_bband: default_settings.show_bband,
            bband_period_str: default_settings.bband_period.to_string(),
            bband_std_dev_str: default_settings.bband_std_dev.to_string(),
            last_frame_time: None,
            fps: 0.0,
        };

        // Start with an `UpdateChart` message to trigger the first layer build.
        (app, Task::done(Message::UpdateChart))
    }

    /// Handles messages and updates the application state.
    fn update(&mut self, message: Message) -> Task<Message> {
        // Flag to indicate if the chart's layers need to be rebuilt
        let mut needs_rebuild = false;

        match message {
            // --- FPS Counter ---
            Message::Tick(now) => {
                // Calculate FPS
                if let Some(last) = self.last_frame_time {
                    let delta = now.duration_since(last);
                    let delta_secs = delta.as_secs_f32();
                    if delta_secs > 0.0 {
                        // Smooth FPS with exponential moving average
                        let instant_fps = 1.0 / delta_secs;
                        self.fps = self.fps.mul_add(0.9, instant_fps * 0.1);
                    }
                }
                self.last_frame_time = Some(now);
            }
            // --- UI Widget Messages ---
            Message::SwitchTheme(theme) => {
                self.current_theme = theme;
            }
            Message::VolumeToggled(is_checked) => {
                self.show_volume = is_checked;
                self.candlestick_chart.settings.show_volume = is_checked;
                needs_rebuild = true;
            }
            Message::SmaToggled(is_checked) => {
                self.show_sma = is_checked;
                self.candlestick_chart.settings.show_sma = is_checked;
                needs_rebuild = true;
            }
            Message::BBandToggled(is_checked) => {
                self.show_bband = is_checked;
                self.candlestick_chart.settings.show_bband = is_checked;
                needs_rebuild = true;
            }
            Message::SmaPeriodChanged(value) => {
                self.sma_period_str = value;
                // Try to parse, but only update the chart if it's a valid number.
                // The chart will use its old value if parsing fails.
                if let Ok(period) = self.sma_period_str.parse::<usize>()
                    && period > 0
                {
                    self.candlestick_chart.settings.sma_period = period;
                    needs_rebuild = true;
                }
            }
            Message::BBandPeriodChanged(value) => {
                self.bband_period_str = value;
                if let Ok(period) = self.bband_period_str.parse::<usize>()
                    && period > 0
                {
                    self.candlestick_chart.settings.bband_period = period;
                    needs_rebuild = true;
                }
            }
            Message::BBandStdDevChanged(value) => {
                self.bband_std_dev_str = value;
                if let Ok(std_dev) = self.bband_std_dev_str.parse::<f64>()
                    && std_dev > 0.0
                {
                    self.candlestick_chart.settings.bband_std_dev = std_dev;
                    needs_rebuild = true;
                }
            }

            // --- Chart Interaction Messages ---
            // These messages are handled directly by the chart component.
            // The component's handle_message will trigger a rebuild internally.
            _ => {
                self.candlestick_chart.handle_message(message);
            }
        }

        // If a UI change triggered a rebuild, send the UpdateChart message.
        if needs_rebuild {
            self.candlestick_chart.handle_message(Message::UpdateChart);
        }

        // No asynchronous tasks are needed in response to updates.
        Task::none()
    }

    /// Renders the application's view.
    fn view(&self) -> Element<'_, Message> {
        // --- Build the Chart Element ---
        let chart = self
            .candlestick_chart
            .view()
            .on_drag(Message::OnPlotDrag)
            .on_scroll(Message::OnPlotScroll)
            .on_axis_drag(Message::OnAxisDrag)
            .on_double_click(Message::OnPlotDoubleClick)
            .on_axis_double_click(Message::OnAxisDoubleClick);

        // --- Build the Settings UI ---
        let settings_bar = self.build_settings_ui();

        // Display the settings bar and then the chart.
        column![settings_bar, chart].spacing(10).padding(20).into()
    }

    /// Helper method to build the settings UI.
    fn build_settings_ui(&self) -> Element<'_, Message> {
        // --- Theme toggle ---
        let theme_toggle = pick_list(iced::Theme::ALL, Some(&self.current_theme), |t| {
            Message::SwitchTheme(t)
        });

        // --- Volume Toggle ---
        let volume_toggle = checkbox(self.show_volume)
            .label("Show volume")
            .on_toggle(Message::VolumeToggled);

        // --- X-axis Domain Display ---
        let x_domain = self
            .candlestick_chart
            .state
            .get_axis(&X_AXIS_ID)
            .map(|axis| axis.scale().domain())
            .unwrap_or((&0.0, &100.0));

        let domain_display = text(format!(
            "Range: {:.0} - {:.0} ({:.0} candles)",
            x_domain.0,
            x_domain.1,
            x_domain.1 - x_domain.0
        ))
        .size(16);

        // --- FPS Display ---
        let fps_display = text(format!("FPS: {:.1}", self.fps)).size(16);

        let toggle_row = row![volume_toggle, theme_toggle, domain_display, fps_display].spacing(10);

        // --- SMA Settings Row ---
        let sma_toggle = checkbox(self.show_sma)
            .label("Show SMA")
            .on_toggle(Message::SmaToggled);
        let sma_period_input = text_input("Period", &self.sma_period_str)
            .on_input(Message::SmaPeriodChanged)
            .padding(5);

        let sma_row = row![
            sma_toggle,
            text("Period:"),
            // Conditionally add the text input only if the checkbox is checked
            if self.show_sma {
                sma_period_input
            } else {
                // Keep layout consistent with a disabled-like placeholder
                sma_period_input.on_input(|_| Message::UpdateChart) // No-op
            }
        ]
        .spacing(10);

        // --- Bollinger Bands Settings Row ---
        let bband_toggle = checkbox(self.show_bband)
            .label("Bollinger Bands")
            .on_toggle(Message::BBandToggled);
        let bband_period_input =
            text_input("Period", &self.bband_period_str).on_input(Message::BBandPeriodChanged);

        let bband_std_dev_input = text_input("StdDev", &self.bband_std_dev_str)
            .on_input(Message::BBandStdDevChanged)
            .on_input(Message::BBandStdDevChanged)
            .padding(5);

        let bband_row = row![
            bband_toggle,
            text("Period:"),
            if self.show_bband {
                bband_period_input
            } else {
                bband_period_input.on_input(|_| Message::UpdateChart)
            },
            text("StdDev:"),
            if self.show_bband {
                bband_std_dev_input
            } else {
                bband_std_dev_input.on_input(|_| Message::UpdateChart)
            }
        ]
        .spacing(10);

        // --- Combine all settings into a column ---
        column![toggle_row, sma_row, bband_row].spacing(10).into()
    }

    /// Defines the application's theme.
    fn theme(&self) -> Theme {
        self.current_theme.clone()
    }

    /// Subscribes to frame updates for FPS counter.
    fn subscription(&self) -> Subscription<Message> {
        window::frames().map(Message::Tick)
    }

    /// Runs the iced application.
    fn run() -> iced::Result {
        iced::application(Self::init, Self::update, Self::view)
            .theme(Self::theme)
            .subscription(Self::subscription)
            .antialiasing(true)
            .run()
    }
}

// --- Chart Component ---

/// Holds settings related to chart rendering and interaction.
#[derive(Debug, Clone, Copy)]
struct ChartSettings {
    /// Automatically adjust the Y-axis to fit visible data.
    y_lock: bool,
    /// Percentage of padding to add when `y_lock` is enabled (e.g., 0.05 = 5%).
    y_lock_margin: f64,
    /// The width of a candle as a percentage (0.0 to 1.0) of its available time interval.
    candle_width_ratio: f64,
    /// The fixed "width" of a single candle in plot coordinates (e.g., timestamp difference).
    time_interval: i64,

    // --- New Indicator Settings ---
    show_volume: bool,
    show_sma: bool,
    sma_period: usize,
    show_bband: bool,
    bband_period: usize,
    bband_std_dev: f64,
}

impl Default for ChartSettings {
    /// Default settings for the chart.
    fn default() -> Self {
        Self {
            y_lock: true,
            y_lock_margin: 0.05,
            candle_width_ratio: 0.5,
            time_interval: 1, // Default, will be recalculated
            show_volume: true,
            show_sma: false,
            sma_period: 20,
            show_bband: false,
            bband_period: 20,
            bband_std_dev: 2.0,
        }
    }
}

/// Manages the state and rendering of the candlestick and volume charts.
struct CandlestickChart {
    /// Configuration for chart behavior.
    settings: ChartSettings,
    /// The raw candle data, mapped by timestamp.
    data: BTreeMap<i64, Candle>,

    /// The single state managing all axes (X, Y-Price, Y-Volume).
    pub state: State<AxisId, f64>,

    /// Items for price candlesticks (X -> Y_AXIS_ID).
    candle_items: CandleItems,
    /// Items for volume bars (X -> Y_VOL_AXIS_ID).
    volume_items: VolumeItems,
    /// Items for SMA line (X -> Y_AXIS_ID).
    sma_items: SmaItems,
    /// Items for Bollinger Band lines (X -> Y_AXIS_ID).
    bband_items: BbandsItems,
}

impl CandlestickChart {
    /// Creates a new `CandlestickChart` with given settings and data.
    pub fn new(settings: ChartSettings) -> Self {
        let candle_data = generate_candlestick_data();
        let time_interval = Self::find_time_interval(&candle_data).unwrap_or(1);
        let y_lock_margin = settings.y_lock_margin;

        // --- Define Initial Ranges ---
        let initial_x_range = (0.0, 100.0);
        let initial_y_range = get_y_range_with_margin(&candle_data, initial_x_range, y_lock_margin)
            .unwrap_or((0.0, 100.0));
        let initial_y_vol_range = get_y_vol_range_with_margin(&candle_data, initial_x_range, 0.1) // 10% margin for volume
            .unwrap_or((0.0, 1000.0));

        // --- Create Single Chart State ---
        let mut state = State::new();
        state.set_axis(X_AXIS_ID, Self::create_x_axis(initial_x_range));
        state.set_axis(Y_AXIS_ID, Self::create_y_axis(initial_y_range));
        state.set_axis(Y_VOL_AXIS_ID, Self::create_vol_axis(initial_y_vol_range));

        Self {
            settings: ChartSettings {
                time_interval,
                ..settings
            },
            data: candle_data,
            state,
            candle_items: CandleItems {
                candles: Vec::new(),
                candle_width: Length::Plot(1.0),
            },
            volume_items: VolumeItems {
                candles: Vec::new(),
                bar_width: Length::Plot(2.0),
            },
            sma_items: SmaItems { points: Vec::new() },
            bband_items: BbandsItems {
                upper: Vec::new(),
                middle: Vec::new(),
                lower: Vec::new(),
            },
        }
    }

    /// Returns the single `Chart` widget containing all layers.
    pub fn view(&self) -> Chart<'_, AxisId, f64, Message> {
        Chart::new(&self.state)
            .layer(&self.candle_items, X_AXIS_ID, Y_AXIS_ID)
            .layer(&self.volume_items, X_AXIS_ID, Y_VOL_AXIS_ID)
            .layer(&self.sma_items, X_AXIS_ID, Y_AXIS_ID)
            .layer(&self.bband_items, X_AXIS_ID, Y_AXIS_ID)
    }

    /// Handles incoming messages and updates the chart state.
    pub fn handle_message(&mut self, message: Message) {
        match message {
            Message::UpdateChart => {
                self.rebuild_layers();
            }
            Message::OnPlotDrag(delta) => {
                self.handle_plot_drag(delta);
                self.rebuild_layers();
            }
            Message::OnPlotScroll(cursor_pos, delta) => {
                self.handle_plot_scroll(cursor_pos, delta);
                self.rebuild_layers();
            }
            Message::OnAxisDrag(id, delta) => {
                self.handle_axis_drag(id, delta);
                self.rebuild_layers();
            }
            Message::OnPlotDoubleClick(point) => {
                println!("Doubleclicked plot at {point:?}")
            }
            Message::OnAxisDoubleClick(id, point) => {
                println!("Doubleclicked axis '{id}' at {point:?}")
            }
            // Other messages are handled by ExampleApp
            _ => {}
        }
    }

    // --- Event Handlers ---

    fn handle_plot_scroll(&mut self, cursor_pos: iced::Point, delta: ScrollDelta) {
        match delta {
            ScrollDelta::Lines { x: _, y } => {
                // Each scroll line = 10% zoom change
                // y = 1.0 (scroll up) → factor = 1.1 (zoom in)
                // y = -1.0 (scroll down) → factor = ~0.909 (zoom out)
                let factor = 1.1f32.powf(y);

                self.state
                    .get_axis_mut(&X_AXIS_ID)
                    .unwrap()
                    .scale_mut()
                    .zoom(factor as f64, Some(cursor_pos.x as f64));
                self.state
                    .get_axis_mut(&Y_AXIS_ID)
                    .unwrap()
                    .scale_mut()
                    .zoom(factor as f64, Some(cursor_pos.y as f64));
            }
            ScrollDelta::Pixels { x: _, y } => {
                // For pixel-based scrolling (touchpad)
                // Divide by larger number for less sensitive zooming
                let factor = 1.0 + y / 500.0;

                self.state
                    .get_axis_mut(&X_AXIS_ID)
                    .unwrap()
                    .scale_mut()
                    .zoom(factor as f64, Some(cursor_pos.x as f64));
                self.state
                    .get_axis_mut(&Y_AXIS_ID)
                    .unwrap()
                    .scale_mut()
                    .zoom(factor as f64, Some(cursor_pos.y as f64));
            }
        }
    }

    /// Logic for panning the chart.
    fn handle_plot_drag(&mut self, delta: DragDelta) {
        // --- Pan X-Axis ---
        self.state
            .get_axis_mut(&X_AXIS_ID)
            .expect("X-axis must exist")
            .scale_mut()
            .pan(delta.x as f64);

        // --- Pan Y-Axes (Conditionally) ---
        if !self.settings.y_lock {
            self.state
                .get_axis_mut(&Y_AXIS_ID)
                .expect("Price Y-axis must exist")
                .scale_mut()
                .pan(delta.y as f64);
        }
        if self.settings.show_volume {
            self.state
                .get_axis_mut(&Y_VOL_AXIS_ID)
                .expect("Volume Y-axis must exist")
                .scale_mut()
                .pan(delta.y as f64);
        }
    }

    /// Logic for zooming the chart by dragging an axis.
    fn handle_axis_drag(&mut self, id: AxisId, delta: f32) {
        let factor = delta.mul_add(2.0, 1.0);
        match id {
            X_AXIS_ID => {
                let anchor = Some(1.0);
                self.state
                    .get_axis_mut(&X_AXIS_ID)
                    .expect("X-axis must exist")
                    .scale_mut()
                    .zoom(factor as f64, anchor);
            }
            Y_AXIS_ID => {
                self.state
                    .get_axis_mut(&Y_AXIS_ID)
                    .expect("Price Y-axis must exist")
                    .scale_mut()
                    .zoom(factor as f64, Some(0.5));
                self.settings.y_lock = false;
            }
            Y_VOL_AXIS_ID => {
                self.state
                    .get_axis_mut(&Y_VOL_AXIS_ID)
                    .expect("Volume Y-axis must exist")
                    .scale_mut()
                    .zoom(factor as f64, Some(0.5));
            }
            _ => {}
        }
    }

    // --- Private Helpers ---

    /// Clears and rebuilds all shape layers based on the current data and view.
    fn rebuild_layers(&mut self) {
        // --- Pre-Render State Synchronization ---
        if self.settings.y_lock {
            self.update_y_lock_domain();
        }
        if self.settings.show_volume {
            self.update_y_vol_domain();
        }
        // --- End State Synchronization ---

        let candle_width_plot =
            self.settings.time_interval as f64 * self.settings.candle_width_ratio;
        let candle_width = Length::Plot(candle_width_plot);

        let x_domain = self
            .state
            .get_axis(&X_AXIS_ID)
            .expect("X-axis must exist")
            .scale()
            .domain();
        let visible_x_range = (x_domain.0.floor() as i64)..=(x_domain.1.ceil() as i64);

        let calculation_offset = self.settings.bband_period.max(self.settings.sma_period) as i64;
        let calculation_x_range = ((x_domain.0.floor() as i64).saturating_sub(calculation_offset))
            ..=(x_domain.1.ceil() as i64);

        // --- Collect visible candle data ---
        let visible_candles: Vec<(i64, Candle)> = self
            .data
            .range(visible_x_range)
            .map(|(time, candle)| (*time, *candle))
            .collect();

        // --- Update candle items ---
        self.candle_items.candles = visible_candles.clone();
        self.candle_items.candle_width = candle_width;

        // --- Update volume items (if enabled) ---
        if self.settings.show_volume {
            self.volume_items.candles = visible_candles;
            self.volume_items.bar_width = candle_width * 1.8;
        } else {
            self.volume_items.candles.clear();
        }

        let calculation_candles: Vec<(i64, Candle)> = self
            .data
            .range(calculation_x_range)
            .map(|(time, candle)| (*time, *candle))
            .collect();

        // --- Update Bollinger Bands items (if enabled) ---
        if self.settings.show_bband {
            let (upper, middle, lower) = calculate_bbands(
                &calculation_candles,
                self.settings.bband_period,
                self.settings.bband_std_dev,
            );
            self.bband_items.upper = upper;
            self.bband_items.middle = middle;
            self.bband_items.lower = lower;
        } else {
            self.bband_items.upper.clear();
            self.bband_items.middle.clear();
            self.bband_items.lower.clear();
        }

        // --- Update SMA items (if enabled) ---
        if self.settings.show_sma {
            self.sma_items.points = calculate_sma(&calculation_candles, self.settings.sma_period);
        } else {
            self.sma_items.points.clear();
        }
    }

    /// Finds the smallest time difference between consecutive data points.
    fn find_time_interval(data: &BTreeMap<i64, Candle>) -> Option<i64> {
        data.keys()
            .skip(1)
            .zip(data.keys())
            .map(|(next, prev)| next - prev)
            .min()
    }

    /// Recalculates and sets the Price Y-axis domain to fit the visible data.
    fn update_y_lock_domain(&mut self) {
        let x_domain = self
            .state
            .get_axis(&X_AXIS_ID)
            .expect("X-axis must exist")
            .scale()
            .domain();

        let new_y_range = get_y_range_with_margin(
            &self.data,
            (*x_domain.0, *x_domain.1),
            self.settings.y_lock_margin,
        );

        if let Some((min, max)) = new_y_range {
            self.state
                .get_axis_mut(&Y_AXIS_ID)
                .expect("Price Y-axis must exist")
                .scale_mut()
                .set_domain(min, max);
        }
    }

    /// Recalculates and sets the Volume Y-axis domain to fit the visible data.
    fn update_y_vol_domain(&mut self) {
        let x_domain = self
            .state
            .get_axis(&X_AXIS_ID)
            .expect("X-axis must exist")
            .scale()
            .domain();

        let new_y_vol_range =
            get_y_vol_range_with_margin(&self.data, (*x_domain.0, *x_domain.1), 0.1); // 10% margin

        if let Some((min, max)) = new_y_vol_range {
            self.state
                .get_axis_mut(&Y_VOL_AXIS_ID)
                .expect("Volume Y-axis must exist")
                .scale_mut()
                .set_domain(min, max);
        }
    }

    /// Factory for creating the main X-axis (with labels).
    fn create_x_axis(range: (f64, f64)) -> Axis<f64> {
        let scale = Linear::new(range.0, range.1);
        let tick_renderer = |ctx: TickLabelContext<f64>| -> Option<TickLine> {
            let timestamp_seconds = ctx.tick.value as i64 * 60; // Assuming 1 unit = 1 minute
            let datetime = chrono::Utc.timestamp_opt(timestamp_seconds, 0).single()?;
            let text = datetime.format("%H:%M").to_string();
            Some(TickLine::simple(text))
        };
        Axis::new(scale, Position::Bottom)
            .with_tick_renderer(tick_renderer)
            .skip_overlapping_labels(8.0)
    }

    /// Factory for creating the Price Y-axis.
    fn create_y_axis(range: (f64, f64)) -> Axis<f64> {
        let scale = Linear::new(range.0, range.1);
        let tick_renderer = |ctx: TickLabelContext<f64>| -> Option<TickLine> {
            Some(TickLine::simple(format!("{:.2}", ctx.tick.value)))
        };
        Axis::new(scale, Position::Right)
            .with_tick_renderer(tick_renderer)
            .skip_overlapping_labels(8.0)
    }

    /// Factory for creating the Volume Y-axis.
    fn create_vol_axis(range: (f64, f64)) -> Axis<f64> {
        let scale = Linear::new(range.0, range.1);
        Axis::new(scale, Position::Right).invisible()
    }
}

// --- Data Structures & Utilities ---

/// Represents a single candlestick.
#[derive(Debug, Clone, Copy)]
struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

// --- Items Implementations for Layers ---

/// Holds candle data for rendering candlesticks
struct CandleItems {
    candles: Vec<(i64, Candle)>,
    candle_width: Length<f64>,
}

impl<R: plot::Renderer> plot::Items<f64, R> for CandleItems {
    fn draw(&self, plot: &mut Plot<'_, f64, R>, theme: &iced::Theme) {
        let palette = theme.extended_palette();
        let bounds = plot.bounds();
        // Create rectangles from candle data during draw
        for (time, candle) in &self.candles {
            let x = *time as f64;

            if !bounds.contains_x(&x) {
                continue;
            };

            let color = candle.color(palette);

            let wick = shape::Line::new(
                PlotPoint::new(x, candle.high),
                PlotPoint::new(x, candle.low),
                Stroke::new(color, Length::Screen(1.0)),
            );

            let body_y_center = (candle.open + candle.close) / 2.0;
            let body = shape::Rectangle::new(
                PlotPoint::new(x, body_y_center),
                self.candle_width,
                Length::Plot((candle.open - candle.close).abs()),
            )
            .fill(color);

            plot.add_shape(wick);
            plot.add_shape(body);
        }
    }
}

/// Holds volume bar data
struct VolumeItems {
    candles: Vec<(i64, Candle)>,
    bar_width: Length<f64>,
}

impl<R: plot::Renderer> plot::Items<f64, R> for VolumeItems {
    fn draw(&self, plot: &mut Plot<'_, f64, R>, theme: &iced::Theme) {
        let palette = theme.extended_palette();
        // Create volume bars from candle data during draw
        for (time, candle) in &self.candles {
            let color = candle.volume_color(palette);
            let x_position = *time as f64;

            let bar = shape::Rectangle::new(
                PlotPoint::new(x_position, 0.0),
                self.bar_width,
                Length::Plot(candle.volume / 10.0),
            )
            .fill(color);

            plot.add_shape(bar);
        }
    }
}

/// Holds SMA line data
struct SmaItems {
    points: Vec<PlotPoint<f64>>,
}

impl<R: plot::Renderer> plot::Items<f64, R> for SmaItems {
    fn draw(&self, plot: &mut Plot<'_, f64, R>, theme: &iced::Theme) {
        let palette = theme.palette();

        if !self.points.is_empty() {
            let sma_line = shape::Polyline::new(
                self.points.clone(),
                Stroke {
                    fill: palette.warning,
                    thickness: Length::Screen(1.5),
                    style: StrokeStyle::Solid,
                },
            );
            plot.add_shape(sma_line);
        }
    }
}

/// Holds Bollinger Bands line data
struct BbandsItems {
    upper: Vec<PlotPoint<f64>>,
    middle: Vec<PlotPoint<f64>>,
    lower: Vec<PlotPoint<f64>>,
}

impl<R: plot::Renderer> plot::Items<f64, R> for BbandsItems {
    fn draw(&self, plot: &mut Plot<'_, f64, R>, theme: &iced::Theme) {
        let palette = theme.palette();

        if !self.upper.is_empty() {
            let upper_line = shape::Polyline::new(
                self.upper.clone(),
                Stroke {
                    fill: palette.text.scale_alpha(0.5),
                    thickness: Length::Screen(1.0),
                    style: StrokeStyle::Solid,
                },
            );
            plot.add_shape(upper_line);
        }

        if !self.middle.is_empty() {
            let middle_line = shape::Polyline::new(
                self.middle.clone(),
                Stroke {
                    fill: palette.primary.scale_alpha(0.5),
                    thickness: Length::Screen(1.0),
                    style: StrokeStyle::Solid,
                },
            );
            plot.add_shape(middle_line);
        }

        if !self.lower.is_empty() {
            let lower_line = shape::Polyline::new(
                self.lower.clone(),
                Stroke {
                    fill: palette.text.scale_alpha(0.5),
                    thickness: Length::Screen(1.0),
                    style: StrokeStyle::Solid,
                },
            );
            plot.add_shape(lower_line);
        }
    }
}

impl Candle {
    /// The color of the candle, based on its open and close prices.
    fn color(&self, palette: &Extended) -> Color {
        if self.close > self.open {
            palette.success.base.color
        } else if self.close < self.open {
            palette.danger.base.color
        } else if palette.is_dark {
            palette.background.weak.color
        } else {
            palette.background.strong.color
        }
    }

    fn volume_color(&self, palette: &Extended) -> Color {
        if self.close > self.open {
            palette.success.base.color
        } else if self.close < self.open {
            palette.danger.base.color
        } else if palette.is_dark {
            palette.background.weak.color
        } else {
            palette.background.strong.color
        }
        .scale_alpha(0.5)
    }
}

/*
    REMOVED Polyline struct definition.
    REMOVED impl chart::shape::Shape for Polyline.
    These are now assumed to exist in the chart::shape module.
*/

/// Generates a set of pseudo-random candlestick data.
fn generate_candlestick_data() -> BTreeMap<i64, Candle> {
    let mut data = BTreeMap::new();
    let mut previous_close = rand::random::<f64>().mul_add(20.0, 100.0);

    for i in 0..CANDLES_AMOUNT {
        let open = previous_close;
        let high_delta = rand::random::<f64>() * 10.0;
        let low_delta = rand::random::<f64>() * -10.0;
        let mut high = open + high_delta;
        let mut low = open + low_delta;

        if high < low {
            std::mem::swap(&mut high, &mut low);
        }

        let close = (high - low).mul_add(rand::random::<f64>(), low);

        data.insert(
            i as i64,
            Candle {
                open,
                high,
                low,
                close,
                volume: rand::random::<f64>().mul_add(5_000_000.0, 1_000_000.0), // More realistic volume
            },
        );

        previous_close = close;
    }
    data
}

// --- Price Data Helpers ---

/// Finds the min (low) and max (high) prices within a given x-range.
fn get_visible_y_range(
    data: &BTreeMap<i64, Candle>,
    x_range: RangeInclusive<i64>,
) -> Option<(f64, f64)> {
    let visible_candles = data.range(x_range);

    let y_low = visible_candles
        .clone()
        .map(|(_key, candle)| candle.low)
        .min_by(|a, b| a.total_cmp(b));

    let y_high = visible_candles
        .map(|(_key, candle)| candle.high)
        .max_by(|a, b| a.total_cmp(b));

    y_low.zip(y_high)
}

/// Calculates a Y-domain (min, max) for the price chart.
fn get_y_range_with_margin(
    data: &BTreeMap<i64, Candle>,
    x_domain: (f64, f64),
    margin_percent: f64,
) -> Option<(f64, f64)> {
    let x_range = (x_domain.0.floor() as i64)..=(x_domain.1.ceil() as i64);

    get_visible_y_range(data, x_range).map(|(y_low, y_high)| {
        let margin_amount = (y_high - y_low + 1e-6) * margin_percent;
        (y_low - margin_amount, y_high + margin_amount)
    })
}

// --- Volume Data Helpers ---

/// Finds the max volume within a given x-range.
fn get_visible_y_vol_range(
    data: &BTreeMap<i64, Candle>,
    x_range: RangeInclusive<i64>,
) -> Option<f64> {
    data.range(x_range)
        .map(|(_key, candle)| candle.volume)
        .max_by(|a, b| a.total_cmp(b))
}

/// Calculates a Y-domain (0, max) for the volume chart.
fn get_y_vol_range_with_margin(
    data: &BTreeMap<i64, Candle>,
    x_domain: (f64, f64),
    margin_percent: f64,
) -> Option<(f64, f64)> {
    let x_range = (x_domain.0.floor() as i64)..=(x_domain.1.ceil() as i64);

    get_visible_y_vol_range(data, x_range).map(|y_high| {
        let margin_amount = (y_high + 1e-6) * margin_percent;
        (0.0, y_high + margin_amount) // Volume axis always starts at 0
    })
}

// --- Indicator Calculation Helpers ---

/// Calculates the Simple Moving Average (SMA) for the given data.
fn calculate_sma(data: &[(i64, Candle)], period: usize) -> Vec<PlotPoint> {
    if period == 0 || data.len() < period {
        return Vec::new();
    }

    data.windows(period)
        .map(|window| {
            let sum: f64 = window.iter().map(|(_, candle)| candle.close).sum();
            let avg = sum / period as f64;
            let (timestamp, _) = window.last().unwrap(); // SMA point aligns with the end of the window
            PlotPoint::new(*timestamp as f64, avg)
        })
        .collect()
}

/// Calculates the Bollinger Bands (Upper, Middle, Lower) for the given data.
fn calculate_bbands(
    data: &[(i64, Candle)],
    period: usize,
    std_dev_mult: f64,
) -> (Vec<PlotPoint>, Vec<PlotPoint>, Vec<PlotPoint>) {
    if period == 0 || data.len() < period {
        return (Vec::new(), Vec::new(), Vec::new());
    }

    let mut upper = Vec::with_capacity(data.len() - period + 1);
    let mut middle = Vec::with_capacity(data.len() - period + 1);
    let mut lower = Vec::with_capacity(data.len() - period + 1);

    for window in data.windows(period) {
        let (timestamp, _) = window.last().unwrap();
        let closes: Vec<f64> = window.iter().map(|(_, c)| c.close).collect();

        // Calculate Mean (which is the SMA)
        let sum: f64 = closes.iter().sum();
        let mean = sum / period as f64;

        // Calculate Standard Deviation
        let variance = closes
            .iter()
            .map(|&close| {
                let diff = close - mean;
                diff * diff
            })
            .sum::<f64>()
            / period as f64;
        let std_dev = variance.sqrt();

        let upper_band = std_dev.mul_add(std_dev_mult, mean);
        let lower_band = std_dev.mul_add(-std_dev_mult, mean);

        upper.push(PlotPoint::new(*timestamp as f64, upper_band));
        middle.push(PlotPoint::new(*timestamp as f64, mean));
        lower.push(PlotPoint::new(*timestamp as f64, lower_band));
    }

    (upper, middle, lower)
}
