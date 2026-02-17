use iced::{Color, theme::palette::Extended};
use iced_aksel::{Measure, Plot, PlotData, PlotPoint, Stroke, shape, stroke::StrokeStyle};

/// Represents a single showcase_candlestick.
#[derive(Debug, Clone, Copy)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
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

// --- Items Implementations for Layers ---

/// Holds candle data for rendering candlesticks
pub struct CandleItems {
    pub candles: Vec<(i64, Candle)>,
    pub candle_width: Measure<f64>,
}

impl PlotData<f64> for CandleItems {
    fn draw(&self, plot: &mut Plot<f64>, theme: &iced::Theme) {
        let palette = theme.extended_palette();
        // Create rectangles from candle data during draw
        for (time, candle) in &self.candles {
            let x = *time as f64;

            let color = candle.color(palette);

            let wick = shape::Line::new(
                PlotPoint::new(x, candle.high),
                PlotPoint::new(x, candle.low),
                Stroke::new(color, Measure::Screen(1.0)),
            );

            let body_y_center = (candle.open + candle.close) / 2.0;
            let body = shape::Rectangle::centered(
                PlotPoint::new(x, body_y_center),
                self.candle_width,
                Measure::Plot((candle.open - candle.close).abs()),
            )
            .fill(color);

            plot.add_shape(wick);
            plot.add_shape(body);
        }
    }
}

/// Holds volume bar data
pub struct VolumeItems {
    pub candles: Vec<(i64, Candle)>,
    pub bar_width: Measure<f64>,
}

impl PlotData<f64> for VolumeItems {
    fn draw(&self, plot: &mut Plot<f64>, theme: &iced::Theme) {
        let palette = theme.extended_palette();
        // Create volume bars from candle data during draw
        for (time, candle) in &self.candles {
            let color = candle.volume_color(palette);
            let x_position = *time as f64;

            let bar = shape::Rectangle::centered(
                PlotPoint::new(x_position, 0.0),
                self.bar_width,
                Measure::Plot(candle.volume / 10.0),
            )
            .fill(color);

            plot.add_shape(bar);
        }
    }
}

/// Holds SMA line data
pub struct SmaItems {
    pub points: Vec<PlotPoint<f64>>,
}

impl PlotData<f64> for SmaItems {
    fn draw(&self, plot: &mut Plot<f64>, theme: &iced::Theme) {
        let palette = theme.palette();

        if !self.points.is_empty() {
            let sma_line = shape::Polyline::new(
                self.points.clone(),
                Stroke::with_style(palette.warning, Measure::Screen(1.5), StrokeStyle::Solid),
            );
            plot.add_shape(sma_line);
        }
    }
}

/// Holds Bollinger Bands line data
pub struct BbandsItems {
    pub upper: Vec<PlotPoint<f64>>,
    pub middle: Vec<PlotPoint<f64>>,
    pub lower: Vec<PlotPoint<f64>>,
}

impl PlotData<f64> for BbandsItems {
    fn draw(&self, plot: &mut Plot<f64>, theme: &iced::Theme) {
        let palette = theme.palette();

        if !self.upper.is_empty() {
            let upper_line = shape::Polyline::new(
                self.upper.clone(),
                Stroke {
                    fill: palette.text.scale_alpha(0.5),
                    thickness: Measure::Screen(1.0),
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
                    thickness: Measure::Screen(1.0),
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
                    thickness: Measure::Screen(1.0),
                    style: StrokeStyle::Solid,
                },
            );
            plot.add_shape(lower_line);
        }
    }
}
