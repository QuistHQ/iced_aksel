use iced::{
    Color, Element, Font, Point, Size, Task, Theme,
    advanced::graphics::text::font_system,
    alignment::{Horizontal, Vertical},
    mouse::ScrollDelta,
    time::Instant,
    widget::{column, pick_list, row, slider, text, text::Wrapping},
};
use iced_aksel::{
    Axis, Chart, Measure, Plot, PlotData, PlotPoint, Quality, State,
    axis::Position,
    plot::DragDelta,
    scale::Linear,
    shape::{Bounds, Label},
};

const AXIS_X: &str = "x";
const AXIS_Y: &str = "y";

// -----------------------------------------------------------------------------
// Data Layer
// -----------------------------------------------------------------------------

struct TextLayer {
    labels: Vec<Label<f64>>,
}

impl PlotData<f64> for TextLayer {
    fn draw(&self, plot: &mut Plot<f64>, _theme: &Theme) {
        for label in &self.labels {
            plot.add_shape(label.clone());
        }
    }
}

// -----------------------------------------------------------------------------
// Application
// -----------------------------------------------------------------------------

struct TextExample {
    state: State<&'static str, f64>,
    layer: TextLayer,

    // Performance & Config
    fps: f32,
    last_frame: Option<Instant>,
    global_quality: f32,
    font: Font,
    selected_family: String,
    families: Vec<String>,
}

#[derive(Debug, Clone)]
enum Message {
    Tick(Instant),
    ChartDragged(DragDelta),
    ChartScrolled(Point, ScrollDelta),
    QualityChanged(f32),
    FontSelected(String),
}

impl TextExample {
    fn new() -> (Self, Task<Message>) {
        let mut state = State::new();

        let mut system = font_system().write().unwrap();
        let families = system
            .raw()
            .db()
            .faces()
            .map(|f| f.families[0].0.clone())
            .collect();
        drop(system);

        // Setup a nice coordinate system
        state.set_axis(
            AXIS_X,
            Axis::new(Linear::new(-10.0, 110.0), Position::Bottom),
        );
        state.set_axis(AXIS_Y, Axis::new(Linear::new(-10.0, 110.0), Position::Left));

        let mut app = Self {
            state,
            layer: TextLayer { labels: Vec::new() },
            fps: 0.0,
            last_frame: None,
            global_quality: 1.0, // Default Standard Quality
            font: iced::Font::DEFAULT,
            selected_family: "Default".to_string(),
            families,
        };

        app.generate_showcase();

        (app, Task::none())
    }

    fn generate_showcase(&mut self) {
        self.layer.labels.clear();

        // 1. Screen-Space Label (UI Style)
        // Stays 24px tall regardless of zoom level. Good for annotations.
        self.layer.labels.push(
            Label::new("Screen Fixed (24px)", PlotPoint::new(10.0, 90.0))
                .font(self.font)
                .size(Measure::Screen(24.0))
                .fill(Color::from_rgb(0.2, 0.4, 0.8)),
        );

        // 2. Plot-Space Label (World Style)
        // Stays 5 units tall. Zooms in/out with the chart. Good for measurements.
        self.layer.labels.push(
            Label::new("World Fixed (5 Units)", PlotPoint::new(10.0, 70.0))
                .font(self.font)
                .size(Measure::Plot(5.0))
                .fill(Color::from_rgb(0.8, 0.2, 0.2)),
        );

        // 3. Rotation Showcase
        self.layer.labels.push(
            Label::new("Rotated 45°", PlotPoint::new(60.0, 70.0))
                .font(self.font)
                .size(Measure::Screen(20.0))
                .rotation(45.0f32.to_radians())
                .fill(Color::from_rgb(0.2, 0.8, 0.2)),
        );

        // 4. Alignment & Upside Down
        self.layer.labels.push(
            Label::new("Upside Down / Centered", PlotPoint::new(60.0, 50.0))
                .font(self.font)
                .size(Measure::Screen(20.0))
                .rotation(180.0f32.to_radians())
                .fill(Color::from_rgb(0.5, 0.5, 0.5)),
        );

        // 5. High Quality Override
        // This label forces 'High' quality regardless of zoom, useful for large headers.
        self.layer.labels.push(
            Label::new("Forced High Quality", PlotPoint::new(10.0, 30.0))
                .font(self.font)
                .size(Measure::Plot(8.0))
                .quality(Quality::High)
                .fill(Color::BLACK),
        );

        // 6. Tiny Detail
        // Zoom in here to test the LOD bucketing!
        self.layer.labels.push(
            Label::new("Zoom In To Read Me", PlotPoint::new(10.0, 10.0))
                .font(self.font)
                .size(Measure::Plot(0.5)) // Very small
                .fill(Color::BLACK),
        );

        // 6. Japanese
        // Showcases more advanced glyph rendering
        self.layer.labels.push(
            Label::new(
                "This japanese text is wrapped! 畝 　ま代　苛ゑニーヌ現グ委る",
                PlotPoint::new(50.0, 10.0),
            )
            .font(self.font)
            .size(Measure::Plot(20.0))
            .align(Horizontal::Left, Vertical::Center)
            .bounds(Bounds::Plot(Size::new(100.0, 100.0)))
            .wrapping(Wrapping::WordOrGlyph)
            .fill(Color::BLACK),
        );
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Update FPS counter
            Message::Tick(now) => {
                if let Some(last) = self.last_frame {
                    let delta = now.duration_since(last).as_secs_f32();
                    if delta > 0.0 {
                        let instant_fps = 1.0 / delta;
                        self.fps = self.fps.mul_add(0.9, instant_fps * 0.1);
                    }
                }
                self.last_frame = Some(now);
            }
            // Pan the chart when dragged
            Message::ChartDragged(delta) => {
                self.state.pan_axes(AXIS_X, AXIS_Y, delta.x, delta.y);
            }
            // Zoom the access on scroll
            Message::ChartScrolled(pt, delta) => {
                if let ScrollDelta::Lines { y, .. } = delta {
                    let factor = if y > 0.0 { 1.1 } else { 0.9 };
                    self.state.zoom_axes(AXIS_X, AXIS_Y, pt.x, pt.y, factor);
                }
            }
            Message::QualityChanged(q) => {
                self.global_quality = q;
            }
            Message::FontSelected(family) => {
                self.selected_family = family.clone();
                // Hack to get a static ref to the family - We leak the family name to get a
                // 'static str
                self.font = dbg!(Font::with_name(Box::leak(family.into_boxed_str())));
                // Update the text so we showcase the new font in action
                self.generate_showcase();
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let chart = Chart::new(&self.state)
            .debug(true) // Shows the tile boundaries
            .quality(self.global_quality)
            .plot_data(&self.layer, AXIS_X, AXIS_Y)
            .on_drag(Message::ChartDragged)
            .on_scroll(Message::ChartScrolled);

        let font_selector = pick_list(
            self.families.as_slice(),
            Some(&self.selected_family),
            Message::FontSelected,
        );

        let sidebar = column![
            text("Text Engine").size(24),
            // Performance Stats
            text(format!("FPS: {:.0}", self.fps))
                .size(16)
                .color(Color::from_rgb(0.0, 0.8, 0.0)),
            text("Global Quality").size(16),
            text(format!("Multiplier: {:.2}x", self.global_quality)).size(12),
            font_selector,
            // Quality Control Slider
            slider(0.1..=3.0, self.global_quality, Message::QualityChanged).step(0.1),
            text("Lower (0.1) = Fast / Blocky")
                .size(10)
                .color(Color::from_rgb(0.5, 0.5, 0.5)),
            text("Higher (3.0) = Slow / Smooth")
                .size(10)
                .color(Color::from_rgb(0.5, 0.5, 0.5)),
            text("Instructions:").size(14),
            text("• Drag to Pan").size(12),
            text("• Scroll to Zoom").size(12),
            text("• Observe 'Zoom In To Read Me' as it sharpens on zoom.").size(12),
        ]
        .spacing(15)
        .padding(20)
        .width(250);

        row![chart, sidebar].into()
    }
}

pub fn main() -> iced::Result {
    iced::application(TextExample::new, TextExample::update, TextExample::view)
        .theme(Theme::Dark)
        .subscription(|_| iced::window::frames().map(Message::Tick))
        .antialiasing(true)
        .run()
}
