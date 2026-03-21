//! iced_aksel Drawing Library Example (Hit-Test Debugger Gallery)
use iced::{
    Element, Length, Point, Theme,
    mouse::{self},
    widget::{column, container, text},
};
use iced_aksel::{
    Axis, Cached, Chart, Delta, DragEvent, EnterEvent, Interaction, Measure, PlotPoint, PressEvent,
    State, axis,
    interaction::{self, InteractionStatus, IntoArea},
    plot::{Plot, PlotData},
    scale::Linear,
    shape::{self, Rectangle},
};

fn main() -> iced::Result {
    iced::application(DrawingApp::new, DrawingApp::update, DrawingApp::view)
        .theme(Theme::CatppuccinMocha)
        .run()
}

// -----------------------------------------------------------------------------
// App State
// -----------------------------------------------------------------------------
struct DrawingApp {
    chart_state: State<&'static str, f64>,
    data: Cached<DrawingData>,
}

#[derive(Debug, Clone)]
enum Message {
    // Shape Interactions
    ShapeExit,
    ShapeEnter(interaction::Id, EnterEvent),
    ShapeDragged(interaction::Id, DragEvent<Delta>),
    ShapeSelected(interaction::Id),

    // Global/Plot Interactions
    BackgroundPressed,
}

impl DrawingApp {
    const X: &'static str = "x";
    const Y: &'static str = "y";

    fn new() -> (Self, iced::Task<Message>) {
        let mut state = State::new();

        state.set_axis(
            Self::X,
            Axis::new(Linear::new(0., 100.), axis::Position::Bottom),
        );
        state.set_axis(
            Self::Y,
            Axis::new(Linear::new(0., 100.), axis::Position::Left),
        );

        let mock_data = DrawingData {
            shapes: vec![
                // A Rectangle
                ShapeData {
                    id: interaction::Id::unique(),
                    shape: GalleryShape::Rect { x: 50.0, y: 50.0 },
                },
                // An Arc (Tests polar trigonometry)
                ShapeData {
                    id: interaction::Id::unique(),
                    shape: GalleryShape::Arc { x: 50.0, y: 50.0 },
                },
            ],
            hovered_id: None,
            selected_id: None,
            dragging_id: None,
        };

        (
            Self {
                chart_state: state,
                data: Cached::new(mock_data),
            },
            iced::Task::none(),
        )
    }

    fn update(&mut self, message: Message) {
        match message {
            // --- Shape Interactions ---
            Message::ShapeExit => self.data.edit().hovered_id = None,
            Message::ShapeEnter(id, _modifiers) => self.data.edit().hovered_id = Some(id),
            Message::ShapeDragged(id, DragEvent { delta, .. }) => {
                let data = self.data.edit();

                let (x_min, x_max) = self.chart_state.axis(&Self::X).domain();
                let (y_min, y_max) = self.chart_state.axis(&Self::Y).domain();

                let x_width = x_max - x_min;
                let y_height = y_max - y_min;

                let dx = (delta.x as f64) * x_width;
                let dy = (delta.y as f64) * y_height;

                // Update the coordinates based on the shape type
                if let Some(item) = data.shapes.iter_mut().find(|r| r.id == id) {
                    match &mut item.shape {
                        GalleryShape::Rect { x, y } | GalleryShape::Arc { x, y } => {
                            *x += dx;
                            *y += dy;
                        }
                    }
                };
            }
            Message::ShapeSelected(id) => {
                self.data.edit().selected_id = Some(id);
            }

            // --- Background Interactions ---
            Message::BackgroundPressed => {
                let data = self.data.edit();
                data.selected_id = None;
                data.dragging_id = None;
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let chart = Chart::new(&self.chart_state)
            .debug(true)
            .plot_data(&self.data, Self::X, Self::Y)
            .default_cursor(mouse::Interaction::Crosshair)
            .on_press(|event: PressEvent<Point>| match event.button {
                mouse::Button::Left => Some(Message::BackgroundPressed),
                _ => None,
            });

        column![
            text("Interaction Framework Showcase").size(30),
            text("Both shapes implement hovering, selection and dragging.").size(16),
            text("The rectangle has a higher priority than the Arc. Notice when hovering over both, that the highest priority is focused.").size(16),
            container(chart).width(Length::Fill).height(Length::Fill)
        ]
        .spacing(20)
        .padding(20)
        .into()
    }
}

// -----------------------------------------------------------------------------
// Data Structs
// -----------------------------------------------------------------------------
#[derive(Clone)]
enum GalleryShape {
    Rect { x: f64, y: f64 },
    Arc { x: f64, y: f64 },
}

struct ShapeData {
    id: interaction::Id,
    shape: GalleryShape,
}

struct DrawingData {
    shapes: Vec<ShapeData>,
    hovered_id: Option<interaction::Id>,
    selected_id: Option<interaction::Id>,
    dragging_id: Option<interaction::Id>,
}

impl PlotData<f64, Message> for DrawingData {
    fn draw(&self, plot: &mut Plot<f64, Message>, theme: &Theme) {
        let palette = theme.palette();

        for item in &self.shapes {
            // Determine Color based on interaction state
            let mut color = if let GalleryShape::Rect { .. } = item.shape {
                palette.primary
            } else {
                palette.warning
            };

            if Some(&item.id) == self.selected_id.as_ref() {
                color = palette.danger;
            } else if Some(&item.id) == self.hovered_id.as_ref() {
                color = palette.success; // Turns green on hover!
            }

            let interaction = match &item.shape {
                GalleryShape::Rect { x, y } => {
                    let shape = Rectangle::centered(
                        PlotPoint::new(*x, *y),
                        Measure::Plot(20.0),
                        Measure::Plot(20.0),
                    )
                    .fill(color);

                    // Rectangle interactions has the highest priority of all shapes
                    let area = shape.resolve_area(plot);
                    let interaction = Interaction::new(area).priority(0);
                    plot.render(shape);
                    interaction
                }
                GalleryShape::Arc { x, y } => {
                    let shape = shape::Arc::new(
                        PlotPoint::new(*x, *y),
                        Measure::Plot(15.0),
                        0.0,
                        std::f32::consts::PI,
                    )
                    .inner_radius(Measure::Plot(5.0))
                    .fill(color);

                    let area = shape.resolve_area(plot);
                    let interaction = Interaction::new(area);
                    plot.render(shape);
                    interaction
                }
            };

            plot.push_interaction(
                item.id.clone(),
                interaction
                    .on_enter(Message::ShapeEnter)
                    .on_exit(Message::ShapeExit)
                    .on_press(|id, event: PressEvent<Point>| {
                        (event.button == mouse::Button::Left).then_some(Message::ShapeSelected(id))
                    })
                    .on_drag(Message::ShapeDragged)
                    .cursor(|c: InteractionStatus| {
                        if c.is_pressed && c.is_dragging {
                            Some(mouse::Interaction::Grab)
                        } else if c.is_hovered {
                            Some(mouse::Interaction::Cell)
                        } else {
                            None
                        }
                    }),
            );
        }
    }
}
