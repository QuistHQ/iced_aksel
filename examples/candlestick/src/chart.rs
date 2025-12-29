use std::{collections::BTreeMap, ops::RangeInclusive};

use chrono::{Datelike, TimeZone, Timelike};
use iced::mouse::ScrollDelta;
use iced_aksel::{
    Axis, Chart, Measure, State,
    axis::{self, GridLine, Position, TickContext, TickLine, TickResult},
    plot::DragDelta,
    scale::Linear,
};

use crate::{
    AxisId, CANDLES_AMOUNT, X_AXIS_ID, Y_AXIS_ID, Y_VOL_AXIS_ID,
    indicators::{calculate_bbands, calculate_sma},
    items::{BbandsItems, Candle, CandleItems, SmaItems, VolumeItems},
};

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

/// Holds settings related to chart rendering and interaction.
#[derive(Debug, Clone, Copy)]
pub struct ChartSettings {
    /// Automatically adjust the Y-axis to fit visible data.
    pub y_lock: bool,
    /// Percentage of padding to add when `y_lock` is enabled (e.g., 0.05 = 5%).
    pub y_lock_margin: f64,
    /// The width of a candle as a percentage (0.0 to 1.0) of its available time interval.
    pub candle_width_ratio: f64,
    /// The fixed "width" of a single candle in plot coordinates (e.g., timestamp difference).
    pub time_interval: i64,

    pub show_volume: bool,

    pub show_sma: bool,
    pub sma_period: usize,

    pub show_bband: bool,
    pub bband_period: usize,
    pub bband_std_dev: f64,
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

#[derive(Debug, Clone)]
pub enum Message {
    /// A message to trigger a full rebuild of the chart layers.
    UpdateChart,
    /// A message sent when the main plot area is dragged.
    OnPlotDrag(DragDelta),
    OnPlotScroll(iced::Point, ScrollDelta),
    /// A message sent when an axis is dragged (for zooming).
    OnAxisDrag(AxisId, f32),
    OnAxisDoubleClick(AxisId),
}

/// Manages the state and rendering of the candlestick and volume charts.
pub struct CandlestickChart {
    /// Configuration for chart behavior.
    pub settings: ChartSettings,
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
                candle_width: Measure::Plot(1.0),
            },
            volume_items: VolumeItems {
                candles: Vec::new(),
                bar_width: Measure::Plot(2.0),
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
    pub fn view(&self) -> Chart<'_, AxisId, f64, crate::Message> {
        Chart::new(&self.state)
            .plot_data(&self.candle_items, X_AXIS_ID, Y_AXIS_ID)
            .plot_data(&self.volume_items, X_AXIS_ID, Y_VOL_AXIS_ID)
            .plot_data(&self.sma_items, X_AXIS_ID, Y_AXIS_ID)
            .plot_data(&self.bband_items, X_AXIS_ID, Y_AXIS_ID)
            .on_drag(|delta| Message::OnPlotDrag(delta).into())
            .on_scroll(|anchor, delta| Message::OnPlotScroll(anchor, delta).into())
            .on_axis_drag(|axis, delta| Message::OnAxisDrag(axis, delta).into())
            .on_axis_double_click(|id, _| Message::OnAxisDoubleClick(id).into())
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
            Message::OnAxisDoubleClick(id) => {
                if id == Y_AXIS_ID {
                    self.settings.y_lock = true;
                    self.update_y_lock_domain();
                }
            }
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
                    .axis_mut(&X_AXIS_ID)
                    .zoom(factor, Some(cursor_pos.x));
                self.state
                    .axis_mut(&Y_AXIS_ID)
                    .zoom(factor, Some(cursor_pos.y));
            }
            ScrollDelta::Pixels { x: _, y } => {
                // For pixel-based scrolling (touchpad)
                // Divide by larger number for less sensitive zooming
                let factor = 1.0 + y / 500.0;

                self.state
                    .axis_mut(&X_AXIS_ID)
                    .zoom(factor, Some(cursor_pos.x));
                self.state
                    .axis_mut(&Y_AXIS_ID)
                    .zoom(factor, Some(cursor_pos.y));
            }
        }

        self.clamp_x_axis();
    }

    fn clamp_x_axis(&mut self) {
        let x_axis = self.state.axis_mut(&X_AXIS_ID);

        // Make sure we don't go outside the bounds of the candles
        let (&min, &max) = x_axis.domain();
        x_axis.set_domain(min.max(0.0), max.min(CANDLES_AMOUNT as f64));
    }

    /// Logic for panning the chart.
    fn handle_plot_drag(&mut self, delta: DragDelta) {
        // --- Pan X-Axis ---
        self.state.axis_mut(&X_AXIS_ID).pan(delta.x);
        self.clamp_x_axis();

        // --- Pan Y-Axes (Conditionally) ---
        if !self.settings.y_lock {
            self.state.axis_mut(&Y_AXIS_ID).pan(delta.y);
        }
        if self.settings.show_volume {
            self.state.axis_mut(&Y_VOL_AXIS_ID).pan(delta.y);
        }
    }

    /// Logic for zooming the chart by dragging an axis.
    fn handle_axis_drag(&mut self, id: AxisId, delta: f32) {
        let factor = delta.mul_add(2.0, 1.0);
        match id {
            X_AXIS_ID => {
                let anchor = Some(1.0);
                self.state.axis_mut(&X_AXIS_ID).zoom(factor, anchor);
            }
            Y_AXIS_ID => {
                self.state.axis_mut(&Y_AXIS_ID).zoom(factor, Some(0.5));
                self.settings.y_lock = false;
            }
            Y_VOL_AXIS_ID => {
                self.state.axis_mut(&Y_VOL_AXIS_ID).zoom(factor, Some(0.5));
            }
            _ => {}
        }
    }

    // --- Private Helpers ---

    /// Clears and rebuilds all shape layers based on the current data and view.
    fn rebuild_layers(&mut self) {
        if self.settings.y_lock {
            self.update_y_lock_domain();
        }
        if self.settings.show_volume {
            self.update_y_vol_domain();
        }

        let candle_width_plot =
            self.settings.time_interval as f64 * self.settings.candle_width_ratio;
        let candle_width = Measure::Plot(candle_width_plot);

        let x_domain = self.state.axis(&X_AXIS_ID).domain();
        let visible_x_range = (x_domain.0.floor() as i64 - 1)..=(x_domain.1.ceil() as i64);

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
        let x_domain = self.state.axis(&X_AXIS_ID).domain();

        let new_y_range = get_y_range_with_margin(
            &self.data,
            (*x_domain.0, *x_domain.1),
            self.settings.y_lock_margin,
        );

        if let Some((min, max)) = new_y_range {
            self.state.axis_mut(&Y_AXIS_ID).set_domain(min, max);
        }
    }

    /// Recalculates and sets the Volume Y-axis domain to fit the visible data.
    fn update_y_vol_domain(&mut self) {
        let x_domain = self.state.axis(&X_AXIS_ID).domain();

        let new_y_vol_range =
            get_y_vol_range_with_margin(&self.data, (*x_domain.0, *x_domain.1), 0.1); // 10% margin

        if let Some((min, max)) = new_y_vol_range {
            self.state.axis_mut(&Y_VOL_AXIS_ID).set_domain(min, max);
        }
    }

    // Factory for creating the main X-axis (with labels).
    fn create_x_axis(range: (f64, f64)) -> Axis<f64> {
        let scale = Linear::new(range.0, range.1);
        let mut current_month = u32::MAX;
        let mut shown_month = false;

        let tick_renderer = move |ctx: TickContext<f64>| -> TickResult {
            let span = ctx.scale_span() as i64;
            let timestamp_seconds = ctx.tick.value as i64 * 60; // Assuming 1 unit = 1 minute

            let Some(datetime) = chrono::Utc.timestamp_opt(timestamp_seconds, 0).single() else {
                return TickResult::new();
            };

            let text = match span {
                // Under 1 week (showing minutes/hours)
                ..10080 => {
                    shown_month = false;

                    // 1. Determine the Tick Interval (Step) based on Zoom level (span)
                    // span is in minutes.
                    // 180 mins = 3 hours
                    // 720 mins = 12 hours
                    let step = match span {
                        0..=180 => 5,     // If zoomed in < 3 hours, show every 5 mins
                        181..=720 => 15,  // If < 12 hours, show every 15 mins
                        721..=1440 => 30, // If < 24 hours, show every 30 mins
                        _ => 60,          // Otherwise show hourly
                    };

                    // 2. Filter: If the minute is not a multiple of the step, hide it.
                    // Exception: Always show the start of a new day (00:00) regardless of step.
                    let is_midnight = datetime.minute() == 0 && datetime.hour() == 0;

                    if !is_midnight && datetime.minute() % step != 0 {
                        return TickResult::new();
                    }

                    // 3. Format the text
                    if is_midnight {
                        datetime.format("%a").to_string()
                    } else {
                        datetime.format("%H:%M").to_string()
                    }
                }
                // 10080 minutes = 7 days (showing days/months)
                10080.. => {
                    if datetime.month() != current_month {
                        current_month = datetime.month();
                        shown_month = false;
                    }

                    if !shown_month {
                        shown_month = true;
                        datetime.format("%b").to_string()
                    } else {
                        datetime.format("%d").to_string()
                    }
                }
            };

            // Standard label generation
            let label = match ctx.tick.level {
                0 => Some(text.into()),
                1 => Some(text.into()),
                _ => None,
            };

            let grid_line = match ctx.tick.level {
                0 => Some(GridLine::default()),
                _ => None,
            };

            let tick_line = match ctx.tick.level {
                0 => Some(TickLine::default()),
                _ => None,
            };

            TickResult {
                label,
                grid_line,
                tick_line,
                ..Default::default()
            }
        };

        Axis::new(scale, Position::Bottom)
            .with_tick_renderer(tick_renderer)
            .with_cursor_formatter(|x| {
                let timestamp_seconds = x as i64 * 60;
                let datetime = chrono::Utc.timestamp_opt(timestamp_seconds, 0).single()?;
                Some(datetime.format("%a %d %b '%g %H:%M").to_string())
            })
            .skip_overlapping_labels(12.0)
    }

    /// Factory for creating the Price Y-axis.
    fn create_y_axis(range: (f64, f64)) -> Axis<f64> {
        let scale = Linear::new(range.0, range.1);
        Axis::new(scale, Position::Right)
            .with_tick_renderer(|ctx: TickContext<f64>| -> TickResult {
                TickResult::default().label(format!("{:.2}", ctx.tick.value))
            })
            .with_cursor_formatter(|x| Some(format!("{x:.2}")))
            .skip_overlapping_labels(8.0)
    }

    /// Factory for creating the Volume Y-axis.
    fn create_vol_axis(range: (f64, f64)) -> Axis<f64> {
        let scale = Linear::new(range.0, range.1);
        Axis::new(scale, Position::Right).invisible().without_grid()
    }
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
