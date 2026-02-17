//! Professional-style audio showcase_spectrum analyzer example.
//!
//! The example focuses on a single smooth curve with a subtle glow/fill to mimic
//! modern mastering tools (FabFilter Pro-Q, Izotope). Real-time audio is captured
//! through `cpal`, transformed with an FFT and rendered on a logarithmic axis.

use cpal::DeviceDescription;
use iced::{
    Element, Pixels, Subscription, Task, Theme,
    theme::{Base, Mode},
    time::Instant,
    widget::{column, pick_list, row, slider, space, text},
    window,
};
use iced_aksel::{
    Axis, Chart, Measure, Plot, PlotData, PlotPoint, State, Stroke,
    axis::{MarkerPosition, Position, TickContext, TickLine, TickResult},
    scale, shape,
};

mod audio;
mod math;

type AxisId = &'static str;

const FREQ_AXIS_ID: AxisId = "freq";
const DB_AXIS_ID: AxisId = "db";

const MIN_FREQ: f64 = 15.0;
const MAX_FREQ: f64 = 22_000.0;
const MIN_DB: f64 = -90.0;
const MAX_DB: f64 = 12.0;

const FFT_SIZE: usize = 8192;
const HOP_SIZE: usize = 512;
const MAX_BUFFER_SIZE: usize = FFT_SIZE * 6;
const TEMPORAL_SMOOTHING: f32 = 0.9;
const POINTS_PER_OCTAVE: usize = 72;
const FFT_GAIN_CORRECTION: f32 = 2.0;

const SMOOTHING_BAND_SHAPE: &[(f64, f64)] = &[
    (20.0, 1.1),
    (50.0, 0.95),
    (100.0, 0.7),
    (200.0, 0.5),
    (400.0, 0.35),
    (1000.0, 0.25),
    (3000.0, 0.19),
    (8000.0, 0.16),
    (20_000.0, 0.13),
];

#[derive(Debug, Clone)]
enum Message {
    Tick(Instant),
    DeviceSelected(String),
    SwitchTheme(iced::Theme),
    ChangeTilt(f64),
}

struct AnalyzerApp {
    current_theme: iced::Theme,
    state: State<AxisId, f64>,
    spectrum_layer: SpectrumLayer,
    output: triple_buffer::Output<Box<[f32]>>,
    magnitudes: Box<[f32]>,
    sample_rate: u32,
    _stream: Option<cpal::Stream>,
    available_devices: Vec<DeviceDescription>,
    selected_device: Option<DeviceDescription>,
    last_frame_time: Option<Instant>,
    fps: f32,
    tilt: f64,
}

impl AnalyzerApp {
    fn init() -> (Self, Task<Message>) {
        let mut state: State<AxisId, f64> = State::new();

        state.set_axis(FREQ_AXIS_ID, create_frequency_axis());
        state.set_axis(DB_AXIS_ID, create_db_axis());

        let host = cpal::default_host();
        let available_devices = audio::enumerate_devices(&host);
        let (stream, selected_device, config, output) = audio::setup_capture_with_device(None);

        (
            Self {
                current_theme: iced::Theme::Dark,
                state,
                spectrum_layer: SpectrumLayer::default(),
                output,
                magnitudes: vec![0.0; FFT_SIZE / 2 + 1].into_boxed_slice(),
                sample_rate: config.map_or(48000, |config| config.sample_rate),
                _stream: stream,
                available_devices,
                selected_device,
                last_frame_time: None,
                fps: 0.0,
                tilt: 4.5,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Tick(now) => {
                if let Some(last) = self.last_frame_time {
                    let dt = now.duration_since(last).as_secs_f32();
                    if dt > 0.0 {
                        let instant = 1.0 / dt;
                        self.fps = self.fps.mul_add(0.85, instant * 0.15);
                    }
                }

                self.last_frame_time = Some(now);
                self.rebuild_curve();
            }
            Message::DeviceSelected(device_name) => {
                println!("Switching to device: {}", device_name);
                let (stream, selected_device, config, output) =
                    audio::setup_capture_with_device(Some(device_name));

                self._stream = stream;
                self.selected_device = selected_device;
                self.output = output;
                self.sample_rate = config.map_or(48000, |config| config.sample_rate);
            }
            Message::SwitchTheme(theme) => {
                self.current_theme = theme;
            }
            Message::ChangeTilt(tilt) => {
                self.tilt = tilt;
            }
        }
    }

    fn rebuild_curve(&mut self) {
        for (slot, mag) in self.magnitudes.iter_mut().zip(self.output.read()) {
            *slot = TEMPORAL_SMOOTHING.mul_add(*slot, (1.0 - TEMPORAL_SMOOTHING) * *mag);
        }

        let magnitudes = &self.magnitudes;
        let sample_rate = self.sample_rate as f64;
        let tilt = self.tilt;

        let log_min = MIN_FREQ.log10();
        let log_max = MAX_FREQ.log10();
        let octaves = (log_max - log_min) / (2.0_f64).log10();
        let num_points = (octaves * POINTS_PER_OCTAVE as f64).round().max(32.0) as usize;
        let step = (log_max - log_min) / num_points as f64;

        let mut curve = Vec::with_capacity(num_points);
        for i in 0..num_points {
            let freq = 10_f64.powf(log_min + step * i as f64);
            let width = math::fractional_width(freq);
            let db = math::sample_fractional_octave(magnitudes, freq, sample_rate, width, tilt);
            curve.push(PlotPoint::new(freq, db));
        }

        self.spectrum_layer.curve = curve;
    }

    fn view(&self) -> Element<'_, Message> {
        let chart = Chart::new(&self.state)
            .plot_data(&self.spectrum_layer, FREQ_AXIS_ID, DB_AXIS_ID)
            .marker(&FREQ_AXIS_ID, MarkerPosition::Cursor, |ctx| {
                Some(ctx.marker(format_frequency_label(ctx.value)))
            })
            .marker(&DB_AXIS_ID, MarkerPosition::Cursor, |ctx| {
                Some(ctx.marker(format_db_label(ctx.value)))
            });

        let pick_row = row![
            text("Audio Input: "),
            pick_list(
                self.available_devices.as_slice(),
                self.selected_device.as_ref(),
                |device| Message::DeviceSelected(device.name().to_owned())
            ),
            text("Theme: "),
            pick_list(iced::Theme::ALL, Some(&self.current_theme), |t| {
                Message::SwitchTheme(t)
            }),
            space::horizontal(),
            text!("tilt: {:.1} db/oct", self.tilt),
            slider(0.0..=6.0, self.tilt, Message::ChangeTilt).step(0.1)
        ]
        .spacing(12);

        let info = row![
            text!("SR: {:.0} Hz", self.sample_rate).size(16),
            text!("FPS: {:.1}", self.fps).size(16),
        ]
        .spacing(24);

        column![pick_row, info, chart]
            .spacing(16)
            .padding(16)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        window::frames().map(Message::Tick)
    }

    fn theme(&self) -> Theme {
        self.current_theme.clone()
    }

    fn run() -> iced::Result {
        iced::application(Self::init, Self::update, Self::view)
            .theme(Self::theme)
            .subscription(Self::subscription)
            .antialiasing(true)
            .run()
    }
}

#[derive(Default, Clone)]
struct SpectrumLayer {
    pub curve: Vec<PlotPoint<f64>>,
}

impl PlotData<f64> for SpectrumLayer {
    fn draw(&self, plot: &mut Plot<f64>, theme: &iced::Theme) {
        if self.curve.len() < 2 {
            return;
        }

        let palette = theme.extended_palette();

        let mut fill_points = Vec::with_capacity(self.curve.len() + 2);
        fill_points.push(PlotPoint::new(MIN_FREQ, MIN_DB));
        fill_points.extend(self.curve.iter().copied());
        fill_points.push(PlotPoint::new(MAX_FREQ, MIN_DB));

        plot.add_shape(
            shape::Area::new(fill_points).fill(palette.primary.base.color.scale_alpha(0.4)),
        );

        let glow_color = if theme.mode() == Mode::Light {
            palette.primary.strong.color
        } else {
            palette.primary.weak.color
        };

        let glow_stroke = Stroke::new(glow_color, Measure::Screen(6.0));
        plot.add_shape(shape::Polyline::new(self.curve.clone(), glow_stroke));

        let line_stroke = Stroke::new(palette.background.base.text, Measure::Screen(2.2));
        plot.add_shape(shape::Polyline::new(self.curve.clone(), line_stroke));
    }
}

fn create_frequency_axis() -> Axis<f64> {
    Axis::new(
        scale::Logarithmic::new(10.0, MIN_FREQ, MAX_FREQ),
        Position::Bottom,
    )
    .with_tick_renderer(frequency_tick_renderer)
    .skip_overlapping_labels(8.0)
}

fn create_db_axis() -> Axis<f64> {
    Axis::new(scale::Linear::new(MIN_DB, MAX_DB), Position::Left)
        .with_tick_renderer(db_tick_renderer)
        .with_thickness(80.0)
        .skip_overlapping_labels(8.0)
}

fn frequency_tick_renderer(ctx: TickContext<f64, Theme>) -> TickResult {
    let line = TickLine {
        length: Pixels(if ctx.tick.level == 0 { 12.0 } else { 6.0 }),
        ..ctx.tickline()
    };
    let label = format_frequency_label(ctx.tick.value);
    TickResult::with_label(ctx.label(label))
        .tick_line(line)
        .grid_line(ctx.gridline())
}

fn db_tick_renderer(ctx: TickContext<f64, Theme>) -> TickResult {
    let label = format_db_label(ctx.tick.value);
    TickResult::with_label(ctx.label(label))
        .tick_line(ctx.tickline())
        .grid_line(ctx.gridline())
}

fn format_frequency_label(value: f64) -> String {
    if value >= 10_000.0 {
        format!("{:.0} kHz", value / 1000.0)
    } else if value >= 1000.0 {
        format!("{:.1} kHz", value / 1000.0)
    } else {
        format!("{:.0} Hz", value)
    }
}

fn format_db_label(value: f64) -> String {
    format!("{:+.0} dB", value)
}

fn main() -> iced::Result {
    AnalyzerApp::run()
}
