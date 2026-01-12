use iced::{Theme, time::Instant};
use iced_aksel::{
    Axis, Chart, Measure, PlotPoint, Scale, State,
    axis::{self, TickResult},
    plot::{Plot, PlotData},
    scale::{Linear, Tick, TickIter},
    shape::Rectangle,
};

type AxisId = &'static str;

#[derive(Debug, Clone)]
pub struct BarData {
    pub label: String,
    pub value: f64,
    // Animation state: The value currently being rendered
    pub(crate) current_value: f64,
}

impl From<(String, f64)> for BarData {
    fn from(data: (String, f64)) -> Self {
        Self {
            label: data.0,
            value: data.1,
            // Start at 0.0 so the bar "rises up" when added
            current_value: 0.0,
        }
    }
}

pub enum Orientation {
    Horizontal,
    Vertical,
}

pub struct BarChart {
    // State is owned here, making this a self-contained widget
    state: State<AxisId, f64>,
    data: Vec<BarData>,
    bar_width: f64,
    orientation: Orientation,

    // Animation
    animation_speed: Option<f64>,
    last_tick: Option<Instant>,
    current_x_max: f64, // Controls the animated expansion of the domain
}

impl BarChart {
    const BAR_AXIS: &str = "BAR_AXIS";
    const VALUE_AXIS: &str = "VALUE_AXIS";

    pub fn new(orientation: Orientation) -> Self {
        let mut state = State::new();

        Self::setup_axes(&mut state, &orientation);

        let mut chart = Self {
            state,
            data: Vec::new(),
            bar_width: 0.8,
            orientation,
            animation_speed: None,
            last_tick: None,
            current_x_max: 1.0, // Default domain size
        };

        chart.refresh();
        chart
    }

    pub const fn animated(mut self, speed: f64) -> Self {
        self.animation_speed = Some(speed.max(0.0).min(1.0));
        self
    }

    pub const fn get_data(&self) -> &Vec<BarData> {
        &self.data
    }

    pub fn add_data<T: Into<BarData>>(&mut self, bar_data: T) {
        let mut data = bar_data.into();

        // If animation is disabled, snap to target immediately
        if self.animation_speed.is_none() {
            data.current_value = data.value;
            self.current_x_max = (self.data.len() + 2) as f64;
        }

        self.data.push(data);
        self.refresh();
    }

    pub fn toggle_orientation(&mut self) {
        self.orientation = match self.orientation {
            Orientation::Horizontal => Orientation::Vertical,
            Orientation::Vertical => Orientation::Horizontal,
        };

        Self::setup_axes(&mut self.state, &self.orientation);
        self.refresh();
    }

    // --- Physics ---

    pub fn tick(&mut self, now: Instant) {
        let Some(speed_normalized) = self.animation_speed else {
            return;
        };

        let dt = self
            .last_tick
            .map_or(0.0, |last| (now - last).as_secs_f32() as f64);
        self.last_tick = Some(now);

        // Standard Exponential Smoothing factor
        // 1.0 - exp(-speed * dt) gives the percentage of the gap to close this frame.
        // Higher physics_speed = faster close.
        let physics_speed = speed_normalized * 10.0;
        let alpha = 1.0 - (-physics_speed * dt).exp();

        // 1. Animate Domain Expansion (Moving Right)
        let target_x_max = self.data.len() as f64 + 1.0;
        let diff_x = target_x_max - self.current_x_max;

        // Update Domain
        if diff_x.abs() > 1e-5 {
            self.current_x_max += diff_x * alpha;
        } else {
            self.current_x_max = target_x_max;
        }

        // Check if domain is still significantly expanding
        // If we are more than 0.1 units away from target, we consider it "Expanding"
        // and hold the new bar at 0.
        let is_expanding = diff_x.abs() > 0.15;

        // 2. Animate Bars Rising
        let last_idx = self.data.len().saturating_sub(1);

        for (i, bar) in self.data.iter_mut().enumerate() {
            // Sequence Logic:
            // If this is the newest bar AND the domain is still expanding,
            // keep it at 0.0. It waits for the "stage" to widen before entering.
            if is_expanding && i == last_idx {
                // Ensure it stays at base if we are waiting
                bar.current_value = 0.0;
                continue;
            }

            let diff = bar.value - bar.current_value;
            if diff.abs() > 1e-5 {
                bar.current_value += diff * alpha;
            } else {
                bar.current_value = bar.value;
            }
        }

        self.refresh();
    }

    pub fn refresh(&mut self) {
        self.auto_scale();
        self.update_labels();
    }

    pub fn chart<Message>(&self) -> Chart<'_, AxisId, f64, Message> {
        let (x_axis_id, y_axis_id) = match self.orientation {
            Orientation::Horizontal => (Self::VALUE_AXIS, Self::BAR_AXIS),
            Orientation::Vertical => (Self::BAR_AXIS, Self::VALUE_AXIS),
        };

        Chart::new(&self.state).plot_data(self, x_axis_id, y_axis_id)
    }

    fn auto_scale(&mut self) {
        // Use the animated current_x_max for the domain to create the sliding effect
        let domain_max = self.current_x_max;

        // Update Bar Axis
        if let Some(bar_axis) = self.state.axis_mut_opt(&Self::BAR_AXIS) {
            bar_axis.set_domain(0.0, domain_max);
        }

        // Update Value Axis
        if let Some(value_axis) = self.state.axis_mut_opt(&Self::VALUE_AXIS) {
            let max_value = self
                .data
                .iter()
                .map(|d| d.value)
                .fold(0.0, f64::max)
                .max(10.0)
                * 1.05;

            value_axis.set_domain(0.0, max_value);
        }
    }

    fn setup_axes(state: &mut State<AxisId, f64>, orientation: &Orientation) {
        let bar_scale = Linear::new_with_tick_generator(0.0, 1.0, |scale| {
            let (&start, &end) = scale.domain();
            TickIter::new((start as i64..(end + 1.0) as i64).map(|n| Tick {
                value: n as f64,
                level: 0,
            }))
        });
        let value_scale = Linear::new(0.0, 1.0);

        let (bar_pos, value_pos) = match orientation {
            Orientation::Horizontal => (axis::Position::Left, axis::Position::Bottom),
            Orientation::Vertical => (axis::Position::Bottom, axis::Position::Left),
        };

        state.set_axis(
            Self::VALUE_AXIS,
            Axis::new(value_scale, value_pos)
                .with_tick_renderer(|ctx| match ctx.tick.level {
                    0 => TickResult {
                        label: Some(ctx.label(format!("{:.2}", ctx.tick.value))),
                        tick_line: Some(ctx.tickline()),
                        grid_line: Some(ctx.gridline()),
                        ..Default::default()
                    },
                    _ => TickResult::default(),
                })
                .skip_overlapping_labels(6.),
        );
        state.set_axis(
            Self::BAR_AXIS,
            Axis::new(bar_scale, bar_pos)
                .without_grid()
                .skip_overlapping_labels(6.),
        );
    }

    fn update_labels(&mut self) {
        let labels: Vec<_> = self.data.iter().map(|data| data.label.clone()).collect();
        self.state
            .axis_mut_opt(&Self::BAR_AXIS)
            .unwrap()
            .set_tick_renderer(move |ctx| {
                let idx = ctx.tick.value;
                if idx <= 0. {
                    return TickResult::default();
                }

                // Round to find nearest whole bar index
                let index = idx.round() as usize;
                if index == 0 {
                    return TickResult::default();
                }

                if let Some(text) = labels.get(index - 1) {
                    return TickResult {
                        label: Some(ctx.label(text.clone())),
                        tick_line: Some(ctx.tickline()),
                        ..Default::default()
                    };
                }

                TickResult::default()
            });
    }
}

impl PlotData<f64> for BarChart {
    fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {
        let palette = theme.palette();
        let bar_color = palette.primary;

        for (i, item) in self.data.iter().enumerate() {
            let index = i as f64 + 1.0;
            let val = item.current_value;

            let shape = match self.orientation {
                Orientation::Horizontal => {
                    let center = PlotPoint::new(val / 2.0, index);
                    Rectangle::centered(center, Measure::Plot(val), Measure::Plot(self.bar_width))
                        .fill(bar_color)
                }
                Orientation::Vertical => {
                    let center = PlotPoint::new(index, val / 2.0);
                    Rectangle::centered(center, Measure::Plot(self.bar_width), Measure::Plot(val))
                        .fill(bar_color)
                }
            };

            plot.add_shape(shape);
        }
    }
}
