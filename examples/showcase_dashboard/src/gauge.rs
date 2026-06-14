use iced::{
    Color, Theme,
    alignment::{Horizontal, Vertical},
    time::Instant,
};
use iced_aksel::{
    Axis, Chart, Measure, PlotPoint, State as ChartState,
    axis::{self},
    plot::{Plot, PlotData},
    scale::Linear,
    shape::{Arc, Label, Rectangle},
};
use std::f32::consts::PI;

type AxisId = &'static str;

// --- Constants ---
const GAUGE_RADIUS: f64 = 1.08;

// --- Helper Types ---

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Zone {
    Primary(f64),
    Success(f64),
    Warning(f64),
    Danger(f64),
    Custom(f64, Color),
}

impl Zone {
    pub const fn threshold(&self) -> f64 {
        match self {
            Self::Primary(t) => *t,
            Self::Success(t) => *t,
            Self::Warning(t) => *t,
            Self::Danger(t) => *t,
            Self::Custom(t, _) => *t,
        }
    }

    pub const fn resolve_color(&self, palette: &iced::theme::Palette) -> Color {
        match self {
            Self::Primary(_) => palette.primary,
            Self::Success(_) => palette.success,
            Self::Warning(_) => palette.warning,
            Self::Danger(_) => palette.danger,
            Self::Custom(_, color) => *color,
        }
    }
}

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Placement {
    Top,
    Bottom,
    Center,
    Hidden,
    Custom(f64, f64),
}

// --- The Gauge (Persistent Widget) ---

pub struct Gauge {
    chart_state: ChartState<AxisId, f64>,

    // Config
    label: String,
    unit: String,
    min: f64,
    max: f64,
    padding: f64,

    zones: Vec<Zone>,
    zone_factors: Vec<f64>,

    base_color: Option<Color>,
    decimals: usize,
    custom_formatter: Option<Box<dyn Fn(f64) -> String>>,
    title_placement: Placement,
    value_placement: Placement,
    inner_radius_factor: f64,
    start_angle: f32,
    end_angle: f32,
    tick_count: usize,

    // Appearance
    zone_opacity: f32,

    // Physics
    value: f64,
    target_value: f64,
    animation_speed: Option<f64>,
    last_tick: Option<Instant>,

    debug_mode: bool,
}

#[allow(unused)]
impl Gauge {
    const X_AXIS: &str = "X";
    const Y_AXIS: &str = "Y";

    pub fn new(label: impl Into<String>, min: f64, max: f64) -> Self {
        let mut gauge = Self {
            chart_state: ChartState::new(),
            label: label.into(),
            unit: String::new(),
            min,
            max,
            padding: 0.42,

            zones: Vec::new(),
            zone_factors: Vec::new(),

            base_color: None,
            decimals: 0,
            custom_formatter: None,
            title_placement: Placement::Bottom,
            value_placement: Placement::Center,
            inner_radius_factor: 0.75,
            start_angle: 0.0,
            end_angle: 0.0,
            tick_count: 0,

            zone_opacity: 0.3,

            value: min,
            target_value: min,
            animation_speed: None,
            last_tick: None,
            debug_mode: false,
        };

        gauge.set_span(240.0);
        gauge.update_axes();

        gauge
    }

    // =========================================================
    //  Runtime Logic
    // =========================================================

    pub fn set_value(&mut self, value: f64) {
        self.target_value = value;
        if self.animation_speed.is_none() {
            self.value = value;
            self.update_zone_factors(1.0);
        } else if self.last_tick.is_none() {
            self.last_tick = Some(Instant::now());
        }
    }

    pub fn tick(&mut self, now: Instant) {
        let Some(speed_normalized) = self.animation_speed else {
            return;
        };

        let dt = self
            .last_tick
            .map_or(0.0, |last| (now - last).as_secs_f32() as f64);
        self.last_tick = Some(now);

        // Standard smoothing
        let physics_speed = speed_normalized * 10.0;
        let alpha = 1.0 - (-physics_speed * dt).exp();

        // 1. Animate Value
        let diff = self.target_value - self.value;
        if diff.abs() > 1e-5 {
            self.value = diff.mul_add(alpha, self.value);
        } else {
            self.value = self.target_value;
        }

        // 2. Animate Zones
        self.update_zone_factors(alpha);
    }

    /// Calculates active state for each zone and interpolates their factors
    fn update_zone_factors(&mut self, alpha: f64) {
        // Ensure factors vec matches zones
        if self.zone_factors.len() != self.zones.len() {
            self.zone_factors.resize(self.zones.len(), 0.0);
        }

        // Find which zone we are currently in
        let current = self.value;
        let mut prev_threshold = self.min;

        for (i, zone) in self.zones.iter().enumerate() {
            let threshold = zone.threshold();

            // A zone is active if the value is within (prev, current]
            // OR if we are pinned at max and this is the last zone
            let is_active = current > prev_threshold && current <= threshold;

            // Target: 1.0 if active, 0.0 if inactive
            let target = if is_active { 1.0 } else { 0.0 };
            let current_factor = self.zone_factors[i];

            // Interpolate
            let diff = target - current_factor;
            if diff.abs() > 1e-5 {
                self.zone_factors[i] += diff * alpha;
            } else {
                self.zone_factors[i] = target;
            }

            prev_threshold = threshold;
        }
    }

    fn update_axes(&mut self) {
        let limit = GAUGE_RADIUS + self.padding;

        self.chart_state.set_axis(
            Self::X_AXIS,
            Axis::new(Linear::new(-limit, limit), axis::Position::Bottom)
                .invisible()
                .without_grid(),
        );
        self.chart_state.set_axis(
            Self::Y_AXIS,
            Axis::new(Linear::new(-limit, limit), axis::Position::Left)
                .invisible()
                .without_grid(),
        );
    }

    // =========================================================
    //  Setters
    // =========================================================

    pub fn set_padding(&mut self, padding: f64) {
        self.padding = padding.max(0.0);
        self.update_axes();
    }

    pub const fn set_range(&mut self, min: f64, max: f64) {
        self.min = min;
        self.max = max;
    }

    pub fn set_animation_speed(&mut self, speed: Option<f64>) {
        self.animation_speed = speed.map(|s| s.clamp(0.0, 1.0));
    }

    pub fn set_unit(&mut self, unit: impl Into<String>) {
        self.unit = unit.into();
    }

    pub const fn set_thickness(&mut self, factor: f64) {
        self.inner_radius_factor = factor.max(0.1).min(0.99);
    }

    pub fn set_span(&mut self, degrees: f64) {
        let half_span_rad = (degrees.to_radians() / 2.0) as f32;
        self.start_angle = 1.5f32.mul_add(PI, -half_span_rad);
        self.end_angle = 1.5f32.mul_add(PI, half_span_rad);
    }

    pub const fn set_ticks(&mut self, count: usize) {
        self.tick_count = count;
    }

    pub const fn set_decimals(&mut self, decimals: usize) {
        self.decimals = decimals;
    }

    pub fn add_zone(&mut self, zone: Zone) {
        self.zones.push(zone);
        // Sort by threshold to ensure correct order
        self.zones
            .sort_by(|a, b| a.threshold().partial_cmp(&b.threshold()).unwrap());
        // Sync factors length
        self.zone_factors.resize(self.zones.len(), 0.0);
    }

    pub fn clear_zones(&mut self) {
        self.zones.clear();
        self.zone_factors.clear();
    }

    pub const fn set_zone_opacity(&mut self, opacity: f32) {
        self.zone_opacity = opacity.max(0.0).min(1.0);
    }

    pub const fn set_title_pos(&mut self, placement: Placement) {
        self.title_placement = placement;
    }

    pub const fn set_value_pos(&mut self, placement: Placement) {
        self.value_placement = placement;
    }

    // =========================================================
    //  Builder API
    // =========================================================

    pub fn padding(mut self, padding: f64) -> Self {
        self.set_padding(padding);
        self
    }

    pub const fn debug(mut self) -> Self {
        self.debug_mode = true;
        self
    }

    pub fn animated(mut self, speed: f64) -> Self {
        self.set_animation_speed(Some(speed));
        self
    }

    pub fn unit(mut self, unit: impl Into<String>) -> Self {
        self.set_unit(unit);
        self
    }

    pub const fn thickness(mut self, factor: f64) -> Self {
        self.set_thickness(factor);
        self
    }

    pub fn span(mut self, degrees: f64) -> Self {
        self.set_span(degrees);
        self
    }

    pub const fn ticks(mut self, count: usize) -> Self {
        self.set_ticks(count);
        self
    }

    pub const fn decimals(mut self, count: usize) -> Self {
        self.set_decimals(count);
        self
    }

    pub fn zone(mut self, zone: Zone) -> Self {
        self.add_zone(zone);
        self
    }

    pub const fn zone_opacity(mut self, opacity: f32) -> Self {
        self.set_zone_opacity(opacity);
        self
    }

    pub const fn base_color(mut self, color: Color) -> Self {
        self.base_color = Some(color);
        self
    }

    pub const fn title_pos(mut self, placement: Placement) -> Self {
        self.set_title_pos(placement);
        self
    }

    pub const fn value_pos(mut self, placement: Placement) -> Self {
        self.set_value_pos(placement);
        self
    }

    pub fn format<F>(mut self, formatter: F) -> Self
    where
        F: Fn(f64) -> String + 'static,
    {
        self.custom_formatter = Some(Box::new(formatter));
        self
    }

    // =========================================================
    //  View & Output
    // =========================================================

    pub fn chart(&self) -> Chart<'_, AxisId, f64, crate::Message> {
        Chart::new(&self.chart_state).plot_data(self, Self::X_AXIS, Self::Y_AXIS)
    }

    // --- Getters ---
    pub const fn get_value(&self) -> f64 {
        self.target_value
    }
    pub const fn get_visual_value(&self) -> f64 {
        self.value
    }
    pub const fn get_range(&self) -> (f64, f64) {
        (self.min, self.max)
    }
    pub const fn get_padding(&self) -> f64 {
        self.padding
    }
}

// --- Drawing Logic ---

impl PlotData<f64, crate::Message> for Gauge {
    fn draw(&self, plot: &mut Plot<f64, crate::Message>, theme: &Theme) {
        // Debug Overlay
        if self.debug_mode {
            let limit = GAUGE_RADIUS + self.padding;
            plot.render(
                Rectangle::centered(
                    PlotPoint::new(0.0, 0.0),
                    Measure::Plot(limit * 2.0),
                    Measure::Plot(limit * 2.0),
                )
                .fill(Color::from_rgba(1.0, 0.0, 0.0, 0.2)),
            );
        }

        let palette = theme.palette();

        // 1. Resolve Active Color (For the main bar)
        let active_color = if self.zones.is_empty() {
            self.base_color.unwrap_or(palette.primary)
        } else {
            self.zones
                .iter()
                .find(|z| self.value <= z.threshold())
                .map(|z| z.resolve_color(&palette))
                .unwrap_or_else(|| self.zones.last().unwrap().resolve_color(&palette))
        };

        let track_color = Color {
            a: 0.1,
            ..palette.text
        };

        // 2. Geometry
        let center = PlotPoint::new(0.0, 0.0);
        let radius = Measure::Plot(1.0);
        let inner_radius = Measure::Plot(self.inner_radius_factor);

        let total_sweep = self.end_angle - self.start_angle;
        let safe_denominator = if self.max == self.min {
            1.0
        } else {
            self.max - self.min
        };
        let value_ratio = ((self.value - self.min) / safe_denominator).clamp(0.0, 1.0);
        let value_angle = (value_ratio as f32).mul_add(total_sweep, self.start_angle);

        // 3. Draw Zones (Animated)
        if !self.zones.is_empty() {
            let mut current_angle = self.start_angle;

            for (i, zone) in self.zones.iter().enumerate() {
                let threshold = zone.threshold();
                let zone_raw_color = zone.resolve_color(&palette);

                let zone_ratio = ((threshold - self.min) / safe_denominator).clamp(0.0, 1.0);
                let zone_end_angle = (zone_ratio as f32).mul_add(total_sweep, self.start_angle);

                // Get animation factor for this zone (0.0 = inactive, 1.0 = active)
                let factor = self.zone_factors.get(i).copied().unwrap_or(0.0);

                // Animation Logic:
                // 1. Opacity: Interpolate between base (dull) and 1.0 (bright)
                let alpha = (1.0 - self.zone_opacity).mul_add(factor as f32, self.zone_opacity);
                let zone_color = Color {
                    a: alpha,
                    ..zone_raw_color
                };

                // 2. Thickness "Pop": Grow radius slightly when active
                // Base: 1.08. Active Boost: +0.04
                let pop = 0.00 * factor;
                let current_radius = Measure::Plot(GAUGE_RADIUS + pop);
                let current_inner = Measure::Plot(1.02);

                if zone_end_angle > current_angle {
                    plot.render(
                        Arc::new(center, current_radius, current_angle, zone_end_angle)
                            .inner_radius(current_inner)
                            .fill(zone_color),
                    );
                    current_angle = zone_end_angle;
                }
            }
        }

        // 4. Main Track
        plot.render(
            Arc::new(center, radius, self.start_angle, self.end_angle)
                .inner_radius(inner_radius)
                .fill(track_color),
        );

        // 5. Active Bar
        if value_ratio > 0.001 {
            plot.render(
                Arc::new(center, radius, self.start_angle, value_angle)
                    .inner_radius(inner_radius)
                    .fill(active_color),
            );
        }

        // 6. Ticks
        if self.tick_count > 1 {
            let step = total_sweep / (self.tick_count as f32 - 1.0);
            let tick_len = 0.05;
            let tick_pos = inner_radius;
            let tick_inner = match inner_radius {
                Measure::Plot(v) => Measure::Plot(v - tick_len),
                _ => Measure::Plot(0.0),
            };
            let tick_color = Color {
                a: 0.5,
                ..palette.text
            };

            for i in 0..self.tick_count {
                let angle = (i as f32).mul_add(step, self.start_angle);
                let half_deg = 0.5f32.to_radians();
                plot.render(
                    Arc::new(center, tick_pos, angle - half_deg, angle + half_deg)
                        .inner_radius(tick_inner)
                        .fill(tick_color),
                );
            }
        }

        // 7. Text
        let resolve_pos = |p: Placement| -> Option<(PlotPoint<f64>, Vertical)> {
            match p {
                Placement::Top => Some((PlotPoint::new(0.0, 0.4), Vertical::Bottom)),
                Placement::Bottom => Some((PlotPoint::new(0.0, -0.4), Vertical::Top)),
                Placement::Center => Some((PlotPoint::new(0.0, 0.2), Vertical::Center)),
                Placement::Custom(x, y) => Some((PlotPoint::new(x, y), Vertical::Center)),
                Placement::Hidden => None,
            }
        };

        if let Some((pos, vert)) = resolve_pos(self.value_placement) {
            let text = self.custom_formatter.as_ref().map_or_else(
                || format!("{:.p$}{}", self.value, self.unit, p = self.decimals),
                |fmt| fmt(self.value),
            );
            plot.render(
                Label::new(text, pos)
                    .fill(active_color)
                    .size(Measure::Screen(32.0))
                    .align(Horizontal::Center, vert),
            );
        }

        if let Some((pos, vert)) = resolve_pos(self.title_placement) {
            plot.render(
                Label::new(&self.label, pos)
                    .fill(Color {
                        a: 0.7,
                        ..palette.text
                    })
                    .size(Measure::Screen(16.0))
                    .align(Horizontal::Center, vert),
            );
        }
    }
}
