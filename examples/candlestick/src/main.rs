//! Chart shape layer example
//!
//! An expanded example of the candlestick chart, now including:
//! - A synchronized volume panel.
//! - Toggleable SMA (Simple Moving Average) indicator.
//! - Toggleable Bollinger Bands (BBands) indicator.
//! - An interactive settings bar with checkboxes and text inputs.

use iced::{
    Element, Subscription, Task, Theme,
    time::Instant,
    widget::{checkbox, column, pick_list, row, text, text_input},
    window,
};

use crate::chart::{CandlestickChart, ChartSettings};

mod chart;
mod indicators;
mod items;

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
    Chart(chart::Message),

    // --- Messages for UI Settings ---
    /// Switches the current theme to a new one.
    SwitchTheme(iced::Theme),
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

impl From<chart::Message> for Message {
    fn from(value: chart::Message) -> Self {
        Self::Chart(value)
    }
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
        (app, Task::done(chart::Message::UpdateChart.into()))
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
            // Other messages are handled directly by the candlestick chart.
            // The component's handle_message will trigger a rebuild internally.
            Message::Chart(message) => {
                self.candlestick_chart.handle_message(message);
            }
        }

        // If a UI change triggered a rebuild, send the UpdateChart message.
        if needs_rebuild {
            self.candlestick_chart
                .handle_message(chart::Message::UpdateChart);
        }

        // No asynchronous tasks are needed in response to updates.
        Task::none()
    }

    /// Renders the application's view.
    fn view(&self) -> Element<'_, Message> {
        // --- Build the Chart Element ---
        let chart = self.candlestick_chart.view();

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
            .axis_opt(&X_AXIS_ID)
            .map(|axis| axis.domain())
            .unwrap_or((&0.0, &100.0));

        let domain_display = text(format!(
            "Range: {:.0} - {:.0} ({:.0} candles)",
            x_domain.0,
            x_domain.1,
            (x_domain.1 - x_domain.0).min(CANDLES_AMOUNT as f64)
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

        let sma_row = row![sma_toggle, text("Period:"), sma_period_input].spacing(10);

        // --- Bollinger Bands Settings Row ---
        let bband_toggle = checkbox(self.show_bband)
            .label("Bollinger Bands")
            .on_toggle(Message::BBandToggled);
        let bband_period_input =
            text_input("Period", &self.bband_period_str).on_input(Message::BBandPeriodChanged);
        let bband_std_dev_input = text_input("StdDev", &self.bband_std_dev_str)
            .on_input(Message::BBandStdDevChanged)
            .padding(5);

        let bband_row = row![
            bband_toggle,
            text("Period:"),
            bband_period_input,
            text("StdDev:"),
            bband_std_dev_input
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
            .run()
    }
}
