//! iced_aksel Drawing Library Example (Hit-Test Debugger Gallery)
use iced::{
    Element, Length, Point, Theme, keyboard,
    mouse::{self, ScrollDelta},
    widget::{column, container, text},
};
use iced_aksel::{
    Axis, Cached, Chart, Delta, DragEvent, Interaction, Measure, PlotPoint, PressEvent,
    ReleaseEvent, ScrollEvent, State, Stroke, axis, interaction,
    plot::{Plot, PlotData},
    radii::{Radii, Radius},
    scale::Linear,
    shape::{self, Area, Ellipse, Rectangle, Triangle},
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
    ShapeHovered(interaction::Id, keyboard::Modifiers),
    ShapeDragged(interaction::Id, DragEvent<Delta>),
    ShapeSelected(interaction::Id),

    // Global Interactions
    AddShape(Point),
    DeleteShape(interaction::Id),
    BackgroundHovered,
    BackgroundPressed,
    ChartDragged(Delta),
    ChartScrolled(ScrollEvent<Point>),
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
                // 1. A Rectangle
                ShapeData {
                    id: interaction::Id::unique(),
                    shape: GalleryShape::Rect {
                        x: 10.0,
                        y: 70.0,
                        w: 20.0,
                        h: 20.0,
                    },
                },
                // 2. An Ellipse
                ShapeData {
                    id: interaction::Id::unique(),
                    shape: GalleryShape::Ellipse {
                        x: 50.0,
                        y: 80.0,
                        rx: 15.0,
                        ry: 10.0,
                    },
                },
                // 3. A Triangle
                ShapeData {
                    id: interaction::Id::unique(),
                    shape: GalleryShape::Triangle {
                        p1: (80.0, 70.0),
                        p2: (95.0, 70.0),
                        p3: (87.5, 90.0),
                    },
                },
                // 4. A Concave Polygon (Pac-Man)
                ShapeData {
                    id: interaction::Id::unique(),
                    shape: GalleryShape::Polygon {
                        points: vec![
                            (10.0, 20.0),
                            (30.0, 20.0),
                            (20.0, 30.0),
                            (30.0, 40.0),
                            (10.0, 40.0),
                        ],
                    },
                },
                // 5. A Bezier Curve (Tests curve flattening)
                ShapeData {
                    id: interaction::Id::unique(),
                    shape: GalleryShape::Bezier {
                        start: (10.0, 50.0),
                        c1: (20.0, 10.0),
                        c2: (40.0, 90.0),
                        end: (50.0, 50.0),
                    },
                },
                // 6. An Arc (Tests polar trigonometry)
                ShapeData {
                    id: interaction::Id::unique(),
                    shape: GalleryShape::Arc {
                        center: (80.0, 30.0),
                        outer_r: 15.0,
                        inner_r: 5.0,
                        start_angle: 0.0,
                        end_angle: std::f32::consts::PI, // A half-donut
                    },
                },
                // 7. A Label (Tests Iced Renderer text measurement)
                ShapeData {
                    id: interaction::Id::unique(),
                    shape: GalleryShape::Label {
                        text: "Interactive Text!".to_string(),
                        x: 50.0,
                        y: 50.0,
                        rotation: 0.5,
                    },
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
            Message::ShapeHovered(id, _modifiers) => self.data.edit().hovered_id = Some(id),
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
                        GalleryShape::Rect { x, y, .. } => {
                            *x += dx;
                            *y += dy;
                        }
                        GalleryShape::Ellipse { x, y, .. } => {
                            *x += dx;
                            *y += dy;
                        }
                        GalleryShape::Triangle { p1, p2, p3 } => {
                            p1.0 += dx;
                            p1.1 += dy;
                            p2.0 += dx;
                            p2.1 += dy;
                            p3.0 += dx;
                            p3.1 += dy;
                        }
                        GalleryShape::Polygon { points } => {
                            for p in points.iter_mut() {
                                p.0 += dx;
                                p.1 += dy;
                            }
                        }
                        GalleryShape::Polyline { points, .. } => {
                            for p in points.iter_mut() {
                                p.0 += dx;
                                p.1 += dy;
                            }
                        }
                        GalleryShape::Bezier { start, c1, c2, end } => {
                            start.0 += dx;
                            start.1 += dy;
                            c1.0 += dx;
                            c1.1 += dy;
                            c2.0 += dx;
                            c2.1 += dy;
                            end.0 += dx;
                            end.1 += dy;
                        }
                        GalleryShape::Arc { center, .. } => {
                            center.0 += dx;
                            center.1 += dy;
                        }
                        GalleryShape::Label { x, y, .. } => {
                            *x += dx;
                            *y += dy;
                        }
                    }
                };
            }
            Message::AddShape(point) => {
                let data_x = self.chart_state.axis(&Self::X).denormalize(point.x);
                let data_y = self.chart_state.axis(&Self::Y).denormalize(point.y);

                self.data.edit().shapes.push(ShapeData {
                    id: interaction::Id::unique(),
                    shape: GalleryShape::Rect {
                        x: data_x - 7.5,
                        y: data_y - 7.5,
                        w: 15.0,
                        h: 15.0,
                    },
                })
            }

            Message::DeleteShape(id) => {
                self.data.edit().shapes.retain(|item| item.id != id);
            }
            Message::ShapeSelected(id) => {
                self.data.edit().selected_id = Some(id);
            }

            // --- Background Interactions ---
            Message::BackgroundHovered => self.data.edit().hovered_id = None,
            Message::BackgroundPressed => {
                let data = self.data.edit();
                data.selected_id = None;
                data.dragging_id = None;
            }

            // --- Drag & Zoom Routing ---
            Message::ChartDragged(delta) => {
                self.chart_state
                    .pan_axes(Self::X, Self::Y, delta.x, delta.y);
            }
            Message::ChartScrolled(ScrollEvent {
                delta, position, ..
            }) => {
                let zoom_factor = match delta {
                    ScrollDelta::Lines { y, .. } | ScrollDelta::Pixels { y, .. } => {
                        if y > 0.0 {
                            1.1
                        } else {
                            0.9
                        }
                    }
                };

                self.chart_state
                    .axis_mut(&Self::X)
                    .zoom(zoom_factor, Some(position.x));
                self.chart_state
                    .axis_mut(&Self::Y)
                    .zoom(zoom_factor, Some(position.y));
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let chart = Chart::new(&self.chart_state)
            .debug(true)
            .plot_data(&self.data, Self::X, Self::Y)
            // .on_hover(|_| Message::BackgroundHovered)
            .on_press(|event: PressEvent<Point>| match event.button {
                mouse::Button::Left => Some(Message::BackgroundPressed),
                _ => None,
            })
            .on_release(|event: ReleaseEvent<Point>| {
                (event.button == mouse::Button::Right && !event.was_dragging)
                    .then_some(Message::AddShape(event.position))
            })
            .on_drag(|event: DragEvent<Delta>| {
                (event.button_held == mouse::Button::Left)
                    .then_some(Message::ChartDragged(event.delta))
            })
            .on_scroll(Message::ChartScrolled);

        column![
            text("Interaction Framework Debugger").size(30),
            text("Hover to test hit-bounds. Left-Click/Drag to move. Right-Click to Add/Delete.")
                .size(16),
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
    Rect {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
    },
    Ellipse {
        x: f64,
        y: f64,
        rx: f64,
        ry: f64,
    },
    Triangle {
        p1: (f64, f64),
        p2: (f64, f64),
        p3: (f64, f64),
    },
    Polygon {
        points: Vec<(f64, f64)>,
    },
    Polyline {
        points: Vec<(f64, f64)>,
        stroke_width: f32,
    },
    Bezier {
        start: (f64, f64),
        c1: (f64, f64),
        c2: (f64, f64),
        end: (f64, f64),
    },
    Arc {
        center: (f64, f64),
        outer_r: f64,
        inner_r: f64,
        start_angle: f32,
        end_angle: f32,
    },
    Label {
        text: String,
        x: f64,
        y: f64,
        rotation: f32,
    },
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
            let mut color = palette.primary;
            if Some(&item.id) == self.selected_id.as_ref() {
                color = palette.danger;
            } else if Some(&item.id) == self.hovered_id.as_ref() {
                color = palette.success; // Turns green on hover!
            }

            match &item.shape {
                GalleryShape::Rect { x, y, w, h } => {
                    let shape = Rectangle::corners(
                        PlotPoint::new(*x, *y),
                        PlotPoint::new(*x + *w, *y + *h),
                    )
                    .fill(color);

                    plot.add_interaction(
                        Interaction::new(item.id.clone(), &shape)
                            .on_hover(Message::ShapeHovered)
                            .on_press(|id, event: PressEvent<Point>| {
                                (event.button == mouse::Button::Left)
                                    .then_some(Message::ShapeSelected(id))
                            })
                            .on_release(|id, event: ReleaseEvent<Point>| {
                                (event.button == mouse::Button::Right)
                                    .then_some(Message::DeleteShape(id))
                            })
                            .on_drag(Message::ShapeDragged),
                    );
                    plot.render(shape);
                }
                GalleryShape::Ellipse { x, y, rx, ry } => {
                    let shape = Ellipse::new(
                        PlotPoint::new(*x, *y),
                        Radii {
                            x: Radius(Measure::Plot(*rx)),
                            y: Radius(Measure::Plot(*ry)),
                        },
                    )
                    .fill(color);

                    plot.add_interaction(
                        Interaction::new(item.id.clone(), &shape)
                            .on_hover(Message::ShapeHovered)
                            .on_press(|id, event: PressEvent<Point>| {
                                (event.button == mouse::Button::Left)
                                    .then_some(Message::ShapeSelected(id))
                            })
                            .on_release(|id, event: ReleaseEvent<Point>| {
                                (event.button == mouse::Button::Right)
                                    .then_some(Message::DeleteShape(id))
                            })
                            .on_drag(Message::ShapeDragged),
                    );
                    plot.render(shape);
                }
                GalleryShape::Triangle { p1, p2, p3 } => {
                    let shape = Triangle::vertices([
                        PlotPoint::new(p1.0, p1.1),
                        PlotPoint::new(p2.0, p2.1),
                        PlotPoint::new(p3.0, p3.1),
                    ])
                    .fill(color);

                    plot.add_interaction(
                        Interaction::new(item.id.clone(), &shape)
                            .on_hover(Message::ShapeHovered)
                            .on_press(|id, event: PressEvent<Point>| {
                                (event.button == mouse::Button::Left)
                                    .then_some(Message::ShapeSelected(id))
                            })
                            .on_release(|id, event: ReleaseEvent<Point>| {
                                (event.button == mouse::Button::Right)
                                    .then_some(Message::DeleteShape(id))
                            })
                            .on_drag(Message::ShapeDragged),
                    );
                    plot.render(shape);
                }
                GalleryShape::Polygon { points } => {
                    let plot_points: Vec<PlotPoint<f64>> =
                        points.iter().map(|p| PlotPoint::new(p.0, p.1)).collect();
                    let shape = Area::new(plot_points).fill(color);

                    plot.add_interaction(
                        Interaction::new(item.id.clone(), &shape)
                            .on_hover(Message::ShapeHovered)
                            .on_press(|id, event: PressEvent<Point>| {
                                (event.button == mouse::Button::Left)
                                    .then_some(Message::ShapeSelected(id))
                            })
                            .on_release(|id, event: ReleaseEvent<Point>| {
                                (event.button == mouse::Button::Right)
                                    .then_some(Message::DeleteShape(id))
                            })
                            .on_drag(Message::ShapeDragged),
                    );
                    plot.render(shape);
                }
                GalleryShape::Polyline {
                    points,
                    stroke_width,
                } => {
                    let plot_points: Vec<PlotPoint<f64>> =
                        points.iter().map(|p| PlotPoint::new(p.0, p.1)).collect();
                    let shape = shape::Polyline::new(
                        plot_points,
                        Stroke::new(color, Measure::Screen(*stroke_width)),
                    );

                    plot.add_interaction(
                        Interaction::new(item.id.clone(), &shape)
                            .on_hover(Message::ShapeHovered)
                            .on_press(|id, event: PressEvent<Point>| {
                                (event.button == mouse::Button::Left)
                                    .then_some(Message::ShapeSelected(id))
                            })
                            .on_release(|id, event: ReleaseEvent<Point>| {
                                (event.button == mouse::Button::Right)
                                    .then_some(Message::DeleteShape(id))
                            })
                            .on_drag(Message::ShapeDragged),
                    );
                    plot.render(shape);
                }
                GalleryShape::Bezier { start, c1, c2, end } => {
                    let shape = shape::Bezier::cubic(
                        PlotPoint::new(start.0, start.1),
                        PlotPoint::new(c1.0, c1.1),
                        PlotPoint::new(c2.0, c2.1),
                        PlotPoint::new(end.0, end.1),
                        Stroke::new(color, Measure::Screen(10.0)), // Nice thick line to test tolerance
                    );

                    plot.add_interaction(
                        Interaction::new(item.id.clone(), &shape)
                            .on_hover(Message::ShapeHovered)
                            .on_press(|id, event: PressEvent<Point>| {
                                (event.button == mouse::Button::Left)
                                    .then_some(Message::ShapeSelected(id))
                            })
                            .on_release(|id, event: ReleaseEvent<Point>| {
                                (event.button == mouse::Button::Right)
                                    .then_some(Message::DeleteShape(id))
                            })
                            .on_drag(Message::ShapeDragged),
                    );
                    plot.render(shape);
                }
                GalleryShape::Arc {
                    center,
                    outer_r,
                    inner_r,
                    start_angle,
                    end_angle,
                } => {
                    let shape = shape::Arc::new(
                        PlotPoint::new(center.0, center.1),
                        Measure::Plot(*outer_r),
                        *start_angle,
                        *end_angle,
                    )
                    .inner_radius(Measure::Plot(*inner_r))
                    .fill(color);

                    plot.add_interaction(
                        Interaction::new(item.id.clone(), &shape)
                            .on_hover(Message::ShapeHovered)
                            .on_press(|id, event: PressEvent<Point>| {
                                (event.button == mouse::Button::Left)
                                    .then_some(Message::ShapeSelected(id))
                            })
                            .on_release(|id, event: ReleaseEvent<Point>| {
                                (event.button == mouse::Button::Right)
                                    .then_some(Message::DeleteShape(id))
                            })
                            .on_drag(Message::ShapeDragged),
                    );
                    plot.render(shape);
                }
                GalleryShape::Label {
                    text,
                    x,
                    y,
                    rotation,
                } => {
                    let shape = shape::Label::new(text, PlotPoint::new(*x, *y))
                        .size(Measure::Plot(15.0))
                        .rotation(*rotation)
                        .fill(color);

                    plot.add_interaction(
                        Interaction::new(item.id.clone(), &shape)
                            .on_hover(Message::ShapeHovered)
                            .on_press(|id, event: PressEvent<Point>| {
                                (event.button == mouse::Button::Left)
                                    .then_some(Message::ShapeSelected(id))
                            })
                            .on_release(|id, event: ReleaseEvent<Point>| {
                                (event.button == mouse::Button::Right)
                                    .then_some(Message::DeleteShape(id))
                            })
                            .on_drag(Message::ShapeDragged),
                    );
                    plot.render(shape);
                }
            }
        }
    }
}
