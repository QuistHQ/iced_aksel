//! A high-performance plotting library for Iced applications.
//!
//! `iced_aksel` provides interactive charts and plots for the Iced GUI framework,
//! built on top of the `aksel` plotting core. It offers flexible axis configuration,
//! multiple shape primitives, and robust interaction handling.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use iced_aksel::{
//!     Chart, State, Axis, Plot, PlotPoint, axis, scale::Linear,
//!     plot::PlotData, shape::Circle, Measure
//! };
//! use iced::{Element, Theme};
//!
//! struct App {
//!     chart_state: State<&'static str, f64>,
//!     data: ScatterData,
//! }
//!
//! #[derive(Debug, Clone)]
//! enum Message {}
//!
//! impl App {
//!     fn new() -> Self {
//!         let mut chart_state = State::new();
//!         chart_state.set_axis("x", Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom));
//!         chart_state.set_axis("y", Axis::new(Linear::new(0.0, 100.0), axis::Position::Left));
//!
//!         Self {
//!             chart_state,
//!             data: ScatterData {
//!                 points: vec![
//!                     PlotPoint::new(10.0, 20.0),
//!                     PlotPoint::new(50.0, 80.0),
//!                     PlotPoint::new(90.0, 30.0),
//!                 ],
//!             },
//!         }
//!     }
//!
//!     fn view(&self) -> Element<Message> {
//!         Chart::new(&self.chart_state)
//!             .plot_data(&self.data, "x", "y")
//!             .into()
//!     }
//! }
//!
//! // Your data struct
//! struct ScatterData {
//!     points: Vec<PlotPoint<f64>>,
//! }
//!
//! // Implement PlotData to define how your data is drawn
//! impl PlotData<f64> for ScatterData {
//!     fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {
//!         for point in &self.points {
//!             plot.add_shape(
//!                 Circle::new(*point, Measure::Screen(5.0))
//!                     .fill(theme.palette().primary)
//!             );
//!         }
//!     }
//! }
//! ```
//!
//! # Core Concepts
//!
//! - **[`Chart`]**: The main widget that renders axes and data
//! - **[`State`]**: Manages axis configuration and chart state
//! - **[`Axis`]**: Configures scales, ticks, grids, and labels
//! - **[`PlotData`]**: Trait for drawable data types
//! - **[`Shape`](crate::shape)**: Primitives for rendering (lines, circles, rectangles, etc.)

use std::{cell::RefCell, fmt::Debug, hash::Hash, ops::Deref};

use aksel::ScreenRect;
use derive_more::{Display, Error};
use iced_core::{
    Clipboard, Color, Element, Event, Layout, Length, Padding, Point, Rectangle, Shell, Size,
    Widget,
    layout::{self, Limits, Node},
    mouse::{self, ScrollDelta},
    renderer::{Quad, Style},
    text::{LineHeight, Shaping, Wrapping},
    touch,
    widget::{Tree, tree},
};

// Re-export aksel
pub use aksel::{Float, Transform, scale, scale::Scale, transform, transform::PlotPoint};

mod action;
mod layer;
mod measure;
mod render;
mod state;
mod style;

pub mod axis;
pub mod plot;
pub mod shape;
pub mod stroke;

pub use axis::Axis;
pub use measure::Measure;
pub use plot::{Plot, PlotData};
pub use shape::Shape;
pub use state::State;
pub use stroke::Stroke;
pub use style::Catalog;

use action::Action;
use axis::{Orientation, Position};
use layer::Layer;
use plot::DragDelta;

// Default value for how many pixels till a drag actually counts as a drag
const DEFAULT_DRAG_DEADBAND: f32 = 10.0;

/// Errors that can occur during chart construction or rendering.
#[derive(Debug, Clone, Error, Display)]
pub enum Error<AxisId> {
    /// Two axes with the same ID were assigned to a single layer.
    #[display("Duplicate axis id's received for a layer: {id:?}")]
    DuplicateAxis {
        /// The id of the axis that was duplicated
        id: AxisId,
    },
    /// Two axes have conflicting orientations (e.g., both horizontal).
    #[display(
        "Conflicting axis orientations: {horizontal:?}({horizontal_orientation:?}) | {vertical:?}(vertical_orientation:?)"
    )]
    AxisConflict {
        /// The ID of the horizontal axis
        horizontal: AxisId,
        /// The orientation of the horizontal axis
        horizontal_orientation: Orientation,
        /// The ID of the vertical axis
        vertical: AxisId,
        /// The orientation of the vertical axis
        vertical_orientation: Orientation,
    },
    /// Referenced an axis ID that doesn't exist in the State.
    #[display("Unknown axis id: '{id:?}'")]
    UnknownAxis {
        /// The ID of the unknown axis
        id: AxisId
    },
}

// Plot/Chart handlers
type ErrorHandler<AxisId, Message> = Box<dyn Fn(Error<AxisId>) -> Message>;
type ClickHandler<Message> = Box<dyn Fn(Point) -> Message>;
type DoubleClickHandler<Message> = Box<dyn Fn(Point) -> Message>;
type DragHandler<Message> = Box<dyn Fn(DragDelta) -> Message>;
type HoverHandler<Message> = Box<dyn Fn(Point) -> Message>;
type ScrollHandler<Message> = Box<dyn Fn(Point, ScrollDelta) -> Message>;

// Axis handlers
type AxisClickHandler<AxisId, Message> = Box<dyn Fn(AxisId, f32) -> Message>;
type AxisDoubleClickHandler<AxisId, Message> = Box<dyn Fn(AxisId, f32) -> Message>;
type AxisDragHandler<AxisId, Message> = Box<dyn Fn(AxisId, f32) -> Message>;
type AxisHoverHandler<AxisId, Message> = Box<dyn Fn(AxisId, f32) -> Message>;
type AxisScrollHandler<AxisId, Message> = Box<dyn Fn(AxisId, f32, ScrollDelta) -> Message>;

/// Internal chart memory
struct Memory<AxisId> {
    action: Action<AxisId>,
    previous_click: Option<mouse::Click>,
    // Add the persistent tessellators here
    tessellators: RefCell<render::Tessellators>,
}

impl<AxisId> Default for Memory<AxisId> {
    fn default() -> Self {
        Self {
            action: Action::default(),
            previous_click: None,
            // Initialize them once. Lyon will reuse the internal Vec capacities.
            tessellators: RefCell::new(render::Tessellators::default()),
        }
    }
}

/// The main charting widget that renders axes and plot data.
///
/// `Chart` manages the layout and rendering of axes, grid lines, and data layers.
/// It supports rich interactions including click, drag, scroll, and hover events
/// on both the plot area and individual axes.
///
/// # Example
///
/// ```rust,no_run
/// use iced_aksel::{Chart, State, Axis, axis, scale::Linear, plot::PlotData};
///
/// # #[derive(Clone)]
/// # enum Message { Scroll(iced::Point, iced::mouse::ScrollDelta) }
/// # struct MyData;
/// # impl PlotData<f64> for MyData {
/// #     fn draw(&self, plot: &mut iced_aksel::Plot<f64>, theme: &iced::Theme) {}
/// # }
/// let mut state: State<&str, f64> = State::new();
/// state.set_axis("x_axis", Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom));
/// state.set_axis("y_axis", Axis::new(Linear::new(0.0, 100.0), axis::Position::Left));
/// let data = MyData;
///
/// let chart = Chart::new(&state)
///     .plot_data(&data, "x_axis", "y_axis")
///     .on_scroll(|pos, delta| Message::Scroll(pos, delta));
/// ```
pub struct Chart<
    'a,
    AxisId,
    Domain,
    Message,
    Theme = iced_core::Theme,
    Renderer = iced_renderer::Renderer,
> where
    AxisId: Hash + Eq + Clone + Debug,
    Domain: Float,
    Theme: Catalog,
    Renderer: plot::Renderer,
{
    state: &'a State<AxisId, Domain>,
    layers: Vec<Layer<'a, AxisId, Domain, Renderer, Theme>>,
    width: Length,
    height: Length,
    class: <Theme as Catalog>::Class<'a>,
    errors: Vec<Error<AxisId>>,
    drag_deadband: f32,
    padding: Padding,

    // Interactions
    on_error: Option<ErrorHandler<AxisId, Message>>,
    on_click: Option<ClickHandler<Message>>,
    on_double_click: Option<DoubleClickHandler<Message>>,
    on_drag: Option<DragHandler<Message>>,
    on_hover: Option<HoverHandler<Message>>,
    on_scroll: Option<ScrollHandler<Message>>,
    on_axis_click: Option<AxisClickHandler<AxisId, Message>>,
    on_axis_double_click: Option<AxisDoubleClickHandler<AxisId, Message>>,
    on_axis_drag: Option<AxisDragHandler<AxisId, Message>>,
    on_axis_hover: Option<AxisHoverHandler<AxisId, Message>>,
    on_axis_scroll: Option<AxisScrollHandler<AxisId, Message>>,

    debug: bool,
}

impl<'a, AxisId, Domain, Message, Theme, Renderer>
    Chart<'a, AxisId, Domain, Message, Theme, Renderer>
where
    Domain: Float,
    AxisId: Hash + Eq + Clone + Debug,
    Theme: Catalog,
    Renderer: plot::Renderer,
{
    /// Creates a new chart from the given state.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use iced_aksel::{Chart, State};
    /// # #[derive(Clone)] enum Message {}
    /// let state: State<&str, f64> = State::new();
    /// let chart: Chart<&str, f64, Message> = Chart::new(&state);
    /// ```
    pub fn new(state: &'a State<AxisId, Domain>) -> Self {
        Self {
            state,
            layers: vec![],
            width: Length::Fill,
            height: Length::Fill,
            class: <Theme as Catalog>::default(),
            errors: vec![],
            drag_deadband: DEFAULT_DRAG_DEADBAND,
            padding: Padding::new(0.),
            on_error: None,
            on_click: None,
            on_double_click: None,
            on_drag: None,
            on_hover: None,
            on_scroll: None,
            on_axis_click: None,
            on_axis_double_click: None,
            on_axis_drag: None,
            on_axis_hover: None,
            on_axis_scroll: None,

            debug: false,
        }
    }

    /// Enables the debug overlay, showing vertex and index counts.
    pub const fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Adds a data layer to the chart using the specified axes.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use iced_aksel::{Chart, State, Axis, axis, scale::Linear, plot::PlotData};
    /// # #[derive(Clone)] enum Message {}
    /// # struct MyData;
    /// # impl PlotData<f64> for MyData {
    /// #     fn draw(&self, plot: &mut iced_aksel::Plot<f64>, theme: &iced::Theme) {}
    /// # }
    /// # let mut state: State<&str, f64> = State::new();
    /// # state.set_axis("x_axis", Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom));
    /// # state.set_axis("y_axis", Axis::new(Linear::new(0.0, 100.0), axis::Position::Left));
    /// # let data = MyData;
    /// let chart: Chart<&str, f64, Message> = Chart::new(&state)
    ///     .plot_data(&data, "x_axis", "y_axis");
    /// ```
    pub fn plot_data<T: plot::PlotData<Domain, Renderer, Theme>>(
        mut self,
        items: &'a T,
        x_axis_id: AxisId,
        y_axis_id: AxisId,
    ) -> Self {
        let layer = Layer::new(items, x_axis_id, y_axis_id);
        if verify_layer(&layer, self.state, &mut self.errors) {
            self.layers.push(layer);
        }

        self
    }

    /// Sets the width of the chart.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use iced_aksel::{Chart, State};
    /// # #[derive(Clone)] enum Message {}
    /// # let state: State<&str, f64> = State::new();
    /// let chart: Chart<&str, f64, Message> = Chart::new(&state)
    ///     .width(iced::Length::Fixed(600.0));
    /// ```
    pub const fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the chart.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use iced_aksel::{Chart, State};
    /// # #[derive(Clone)] enum Message {}
    /// # let state: State<&str, f64> = State::new();
    /// let chart: Chart<&str, f64, Message> = Chart::new(&state)
    ///     .height(iced::Length::Fixed(400.0));
    /// ```
    pub const fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the padding around the chart.
    ///
    /// This adds space between the chart edges and its container.
    // TODO: Consider removing this. If we can make the chart show itself perfectly centered, this could
    // be handled by the user using wrapper elements in UI
    pub const fn padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    /// Sets the minimum drag distance in pixels before drag events are triggered.
    ///
    /// Default is 10 pixels. This helps distinguish clicks from drags.
    pub const fn drag_deadband(mut self, distance: f32) -> Self {
        self.drag_deadband = distance;
        self
    }

    /// Sets a callback for chart errors.
    ///
    /// Errors can occur when axes are misconfigured or missing.
    pub fn on_error<F>(mut self, f: F) -> Self
    where
        F: Fn(Error<AxisId>) -> Message + 'static,
    {
        self.on_error = Some(Box::new(f));
        self
    }

    /// Sets a callback for plot area clicks.
    ///
    /// The callback receives normalized coordinates (0.0-1.0) relative to the plot area.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use iced_aksel::{Chart, State};
    /// # #[derive(Clone)] enum Message { PlotClicked(iced::Point) }
    /// # let state: State<&str, f64> = State::new();
    /// let chart: Chart<&str, f64, Message> = Chart::new(&state)
    ///     .on_click(|point| Message::PlotClicked(point));
    /// ```
    pub fn on_click<F>(mut self, f: F) -> Self
    where
        F: Fn(Point) -> Message + 'static,
    {
        self.on_click = Some(Box::new(f));
        self
    }

    /// Sets a callback for plot area double-clicks.
    ///
    /// The callback receives normalized coordinates (0.0-1.0).
    pub fn on_double_click<F>(mut self, f: F) -> Self
    where
        F: Fn(Point) -> Message + 'static,
    {
        self.on_double_click = Some(Box::new(f));
        self
    }

    /// Sets a callback for plot area drag events.
    ///
    /// The callback receives normalized deltas that can be used with axis `pan` methods.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use iced_aksel::{Chart, State, plot::DragDelta};
    /// # #[derive(Clone)] enum Message { Pan(DragDelta) }
    /// # let state: State<&str, f64> = State::new();
    /// let chart: Chart<&str, f64, Message> = Chart::new(&state)
    ///     .on_drag(|delta| Message::Pan(delta));
    /// ```
    pub fn on_drag<F>(mut self, f: F) -> Self
    where
        F: Fn(DragDelta) -> Message + 'static,
    {
        self.on_drag = Some(Box::new(f));
        self
    }

    /// Sets a callback for plot area hover events.
    ///
    /// The callback receives normalized coordinates (0.0-1.0).
    pub fn on_hover<F>(mut self, f: F) -> Self
    where
        F: Fn(Point) -> Message + 'static,
    {
        self.on_hover = Some(Box::new(f));
        self
    }

    /// Sets a callback for plot area scroll events.
    ///
    /// The callback receives normalized coordinates and scroll delta, useful for zooming.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use iced_aksel::{Chart, State};
    /// # use iced::mouse::ScrollDelta;
    /// # #[derive(Clone)] enum Message { Zoom(iced::Point, ScrollDelta) }
    /// # let state: State<&str, f64> = State::new();
    /// let chart: Chart<&str, f64, Message> = Chart::new(&state)
    ///     .on_scroll(|point, delta| Message::Zoom(point, delta));
    /// ```
    pub fn on_scroll<F>(mut self, f: F) -> Self
    where
        F: Fn(Point, ScrollDelta) -> Message + 'static,
    {
        self.on_scroll = Some(Box::new(f));
        self
    }

    /// Sets a callback for axis click events.
    ///
    /// Receives the axis ID and normalized position (0.0-1.0) along that axis.
    pub fn on_axis_click<F>(mut self, f: F) -> Self
    where
        F: Fn(AxisId, f32) -> Message + 'static,
    {
        self.on_axis_click = Some(Box::new(f));
        self
    }

    /// Sets a callback for axis double-click events.
    ///
    /// Receives the axis ID and normalized position (0.0-1.0) along that axis.
    pub fn on_axis_double_click<F>(mut self, f: F) -> Self
    where
        F: Fn(AxisId, f32) -> Message + 'static,
    {
        self.on_axis_double_click = Some(Box::new(f));
        self
    }

    /// Sets a callback for axis drag events.
    ///
    /// Receives the axis ID and normalized delta along that axis.
    pub fn on_axis_drag<F>(mut self, f: F) -> Self
    where
        F: Fn(AxisId, f32) -> Message + 'static,
    {
        self.on_axis_drag = Some(Box::new(f));
        self
    }

    /// Sets a callback for axis scroll events.
    ///
    /// Receives the axis ID, normalized position (0.0-1.0), and scroll delta.
    pub fn on_axis_scroll<F>(mut self, f: F) -> Self
    where
        F: Fn(AxisId, f32, ScrollDelta) -> Message + 'static,
    {
        self.on_axis_scroll = Some(Box::new(f));
        self
    }

    /// Sets a callback for axis hover events.
    ///
    /// Receives the axis ID and normalized position (0.0-1.0) along that axis.
    pub fn on_axis_hover<F>(mut self, f: F) -> Self
    where
        F: Fn(AxisId, f32) -> Message + 'static,
    {
        self.on_axis_hover = Some(Box::new(f));
        self
    }

    fn handle_mouse_press(
        &self,
        memory: &mut Memory<AxisId>,
        layout: Layout,
        cursor: mouse::Cursor,
        shell: &mut Shell<'_, Message>,
    ) {
        // If we click during any other action than idle, we must return
        if Action::Idle != memory.action {
            return;
        }

        let plot_bounds = self.get_plot_layout(layout).bounds();

        // We should be able to ensure that a cursor position exists after the first if statement
        // Which means it is safe to unwrap.
        if cursor.position_over(plot_bounds).is_some() {
            shell.capture_event();

            memory.action = Action::DraggingPlot {
                origin: cursor.position().unwrap(),
                last_position: cursor.position().unwrap(),
                total_delta: 0.0,
            };

            // Now we check for a double-click
            let Some((position, handler)) = cursor.position().zip(self.on_double_click.as_ref())
            else {
                return;
            };

            let new_click = mouse::Click::new(position, mouse::Button::Left, memory.previous_click);

            if new_click.kind() == mouse::click::Kind::Double {
                shell.publish(handler(position));
            }

            memory.previous_click = Some(new_click);

            return;
        }

        for (i, (id, axis)) in self.state.axes().iter().enumerate() {
            let axis_bounds = layout.children().nth(i).unwrap().bounds();

            let Some(position) = cursor.position() else {
                continue;
            };

            if !axis_bounds.contains(position) {
                continue;
            }

            // We should be able to ensure that a cursor position exists after the first if statement
            // Which means it is safe to unwrap.
            let origin = match axis.orientation() {
                Orientation::Horizontal => position.x,
                Orientation::Vertical => position.y,
            };

            shell.capture_event();

            memory.action = Action::DraggingAxis {
                id: id.clone(),
                origin,
                last_position: origin,
                total_delta: 0.0,
            };

            // Now we check for a double-click
            let Some((position, handler)) =
                cursor.position().zip(self.on_axis_double_click.as_ref())
            else {
                return;
            };

            let new_click = mouse::Click::new(position, mouse::Button::Left, memory.previous_click);

            if new_click.kind() == mouse::click::Kind::Double {
                shell.publish(handler(
                    id.clone(),
                    axis.screen_to_normalized(origin, &axis_bounds),
                ));
            }

            memory.previous_click = Some(new_click);

            // we can only be over one axis at a time, so we break the loop after capturing
            break;
        }
    }

    fn handle_mouse_release(
        &self,
        memory: &mut Memory<AxisId>,
        layout: Layout,
        _cursor: mouse::Cursor,
        shell: &mut Shell<'_, Message>,
    ) {
        let Memory { action, .. } = memory;

        // If delta is OVER deadband, we shouldn't send click-events
        if let Some(total_drag_delta) = action.total_drag_delta()
            && total_drag_delta > self.drag_deadband
        {
            return;
        }

        match action {
            Action::Idle => (), // Do nothing
            Action::DraggingPlot { origin, .. } => {
                if let Some(handler) = &self.on_click {
                    let plot_bounds = self.get_plot_layout(layout).bounds();

                    // Translate origin to normalized point (0.0-1.0)
                    let normalized = Point::new(
                        (origin.x - plot_bounds.x) / plot_bounds.width,
                        1.0 - ((origin.y - plot_bounds.y) / plot_bounds.height),
                    );

                    shell.publish(handler(normalized));
                }
            }
            Action::DraggingAxis { id, origin, .. } => {
                // Find the axis and its layout
                if let Some((i, id, axis)) = self.state.axes().get_full(id) {
                    let axis_bounds = layout.children().nth(i).unwrap().bounds();

                    // Translate origin to normalized value (0.0-1.0)
                    let normalized = axis.screen_to_normalized(*origin, &axis_bounds);

                    // Call the axis click handler if it exists
                    if let Some(handler) = &self.on_axis_click {
                        shell.publish(handler(id.clone(), normalized));
                    }
                }
            }
        }
    }

    fn handle_mouse_moved(
        &self,
        memory: &mut Memory<AxisId>,
        layout: Layout,
        cursor: mouse::Cursor,
        shell: &mut Shell<'_, Message>,
    ) {
        let Memory { action, .. } = memory;
        let plot_bounds = self.get_plot_layout(layout).bounds();

        // We should be able to ensure that a cursor position exists after the first if statement
        // Which means it is safe to unwrap.
        if cursor.position_in(plot_bounds).is_some() {
            match action {
                Action::DraggingAxis { .. } => (), // Do nothing
                Action::Idle => {
                    if let Some(handler) = &self.on_hover {
                        let cursor_pos = cursor.position().unwrap();

                        // Translate cursor position to normalized position (0.0-1.0)
                        let normalized = Point::new(
                            (cursor_pos.x - plot_bounds.x) / plot_bounds.width,
                            (cursor_pos.y - plot_bounds.y) / plot_bounds.height,
                        );

                        shell.publish(handler(normalized));
                    }
                    return;
                }
                Action::DraggingPlot {
                    last_position,
                    total_delta,
                    ..
                } => {
                    // When dragging an axis, we want to capture the events, as no other widget should handle this event
                    shell.capture_event();

                    let current_pos = cursor.position().unwrap();

                    // Calculate delta from last position
                    let delta_x = current_pos.x - last_position.x;
                    let delta_y = current_pos.y - last_position.y;

                    // Update total_delta with Euclidean distance
                    let frame_distance = delta_x.hypot(delta_y);
                    *total_delta += frame_distance;

                    // Update last_position
                    *last_position = current_pos;

                    // Only send drag event if we've exceeded the deadband
                    if *total_delta > self.drag_deadband
                        && let Some(handler) = &self.on_drag
                    {
                        // Normalize the delta by plot bounds size
                        // Negate to convert from screen-space drag to data-space pan:
                        // dragging right should show data to the left (decrease the scale)
                        let normalized_delta = DragDelta {
                            x: -delta_x / plot_bounds.width,
                            y: delta_y / plot_bounds.height,
                        };

                        shell.publish(handler(normalized_delta));
                    }

                    return;
                }
            }
        }

        // Handle axis drag (if currently dragging an axis, handle it regardless of cursor position)
        if let Action::DraggingAxis {
            id: dragging_id,
            last_position,
            total_delta,
            ..
        } = action
        {
            // When dragging an axis, we want to capture the events, as no other widget should handle this event
            shell.capture_event();

            if let Some((i, (id, axis))) = self
                .state
                .axes()
                .iter()
                .enumerate()
                .find(|(_, (axis_id, _))| *axis_id == dragging_id)
            {
                let axis_bounds = layout.children().nth(i).unwrap().bounds();
                let cursor_pos = cursor.position().unwrap();
                let screen_value = match axis.orientation() {
                    Orientation::Horizontal => cursor_pos.x,
                    Orientation::Vertical => cursor_pos.y,
                };

                let delta = screen_value - *last_position;
                let frame_distance = delta.abs();
                *total_delta += frame_distance;
                *last_position = screen_value;

                if *total_delta > self.drag_deadband
                    && let Some(handler) = &self.on_axis_drag
                {
                    let normalized_delta = axis.translate_drag_delta(delta, &axis_bounds);
                    shell.publish(handler(id.clone(), normalized_delta));
                }
            }
        }
        // Handle axis hover (only when idle and cursor is over an axis)
        else if matches!(action, Action::Idle) {
            for (i, (id, axis)) in self.state.axes().iter().enumerate() {
                let axis_bounds = layout.children().nth(i).unwrap().bounds();

                if cursor.position_over(axis_bounds).is_none() {
                    continue;
                }

                if let Some(handler) = &self.on_axis_hover {
                    let cursor_pos = cursor.position().unwrap();
                    let screen_value = match axis.orientation() {
                        Orientation::Horizontal => cursor_pos.x,
                        Orientation::Vertical => cursor_pos.y,
                    };
                    let normalized = axis.screen_to_normalized(screen_value, &axis_bounds);
                    shell.publish(handler(id.clone(), normalized));
                }

                break; // Only hover one axis at a time
            }
        }
    }

    #[inline(always)]
    fn get_plot_layout<'b>(&self, layout: Layout<'b>) -> Layout<'b> {
        layout.children().last().unwrap()
    }
}

impl<AxisId, Domain, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Chart<'_, AxisId, Domain, Message, Theme, Renderer>
where
    AxisId: Hash + Eq + Debug + Clone + 'static,
    Domain: Float,
    Renderer: plot::Renderer,
    Theme: Catalog,
    Message: Clone,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Memory<AxisId>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(Memory::<AxisId>::default())
    }

    fn children(&self) -> Vec<Tree> {
        // One child per Axis + one for content.
        let mut children: Vec<Tree> = self.state.axes().iter().map(|_| Tree::empty()).collect();
        children.push(Tree::empty()); // content
        children
    }

    fn diff(&self, _tree: &mut Tree) {}

    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(&mut self, _tree: &mut Tree, _renderer: &Renderer, limits: &Limits) -> Node {
        let bounds = limits.resolve(self.width, self.height, Size::ZERO);

        let axis_count = self.state.axes().len();

        // ---------- 1) First pass: measure axis thicknesses ----------

        let mut top_total = self.padding.top;
        let mut bottom_total = self.padding.bottom;
        let mut left_total = self.padding.left;
        let mut right_total = self.padding.right;

        for (_, axis) in self.state.axes() {
            let thickness = axis.thickness().0;
            match axis.position() {
                Position::Top => top_total += thickness,
                Position::Bottom => bottom_total += thickness,
                Position::Left => left_total += thickness,
                Position::Right => right_total += thickness,
            }
        }

        // ---------- 2) Compute chart (plot) area ----------

        let chart_height = (bounds.height - top_total - bottom_total).max(0.0);
        let chart_width = (bounds.width - left_total - right_total).max(0.0);

        let chart_origin = Point::new(left_total, top_total);
        let chart_size = Size::new(chart_width, chart_height);

        // ---------- 3) Second pass: layout & position everything ----------

        let mut children_nodes = Vec::with_capacity(axis_count + 1);

        let mut top_y = self.padding.top;
        let mut bot_y = top_total + chart_height;
        let mut left_x = self.padding.left;
        let mut right_x = left_total + chart_width;

        for (_, axis) in self.state.axes() {
            let thickness = axis.thickness().0;
            let node = match axis.position() {
                Position::Top => {
                    let n = layout_horizontal_axis(chart_width, axis, left_total, top_y, thickness);
                    top_y += thickness;
                    n
                }
                Position::Bottom => {
                    let n = layout_horizontal_axis(chart_width, axis, left_total, bot_y, thickness);
                    bot_y += thickness;
                    n
                }
                Position::Left => {
                    let n = layout_vertical_axis(chart_height, axis, left_x, top_total, thickness);
                    left_x += thickness;
                    n
                }
                Position::Right => {
                    let n = layout_vertical_axis(chart_height, axis, right_x, top_total, thickness);
                    right_x += thickness;
                    n
                }
            };
            children_nodes.push(node);
        }

        // --- Chart content (center plot) ---
        let chart_node = Node::new(chart_size).move_to(chart_origin);
        children_nodes.push(chart_node);

        Node::with_children(bounds, children_nodes)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        if !self.errors.is_empty()
            && let Some(handler) = &self.on_error
        {
            for error in self.errors.drain(..) {
                shell.publish(handler(error));
            }

            // Don't handle events when errors occur ?
            return;
        }

        // Only redraw and handle events if the cursor is within the chart bounds
        let bounds = layout.bounds();
        if cursor.position_over(bounds).is_none() {
            return;
        }

        let memory: &mut Memory<AxisId> = tree.state.downcast_mut();

        // Handle input events
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                self.handle_mouse_press(memory, layout, cursor, shell);
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Touch(touch::Event::FingerLost { .. }) => {
                self.handle_mouse_release(memory, layout, cursor, shell);
                memory.action = Action::Idle;
            }
            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. }) => {
                self.handle_mouse_moved(memory, layout, cursor, shell);
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                if let Some(cursor_pos) = cursor.position() {
                    let plot_bounds = self.get_plot_layout(layout).bounds();

                    // Check if scrolling over plot
                    if cursor.position_over(plot_bounds).is_some() {
                        if let Some(handler) = &self.on_scroll {
                            // Normalize cursor position (0.0-1.0)
                            // Invert y since y=0 is at top in Iced but bottom in plot space
                            let normalized = Point::new(
                                (cursor_pos.x - plot_bounds.x) / plot_bounds.width,
                                1.0 - ((cursor_pos.y - plot_bounds.y) / plot_bounds.height),
                            );
                            // Capture scroll events to prevent other widgets from handling
                            shell.capture_event();
                            shell.publish(handler(normalized, *delta));
                        }
                    } else {
                        // Check if scrolling over an axis
                        for (i, (id, axis)) in self.state.axes().iter().enumerate() {
                            let axis_bounds = layout.children().nth(i).unwrap().bounds();

                            if cursor.position_over(axis_bounds).is_some() {
                                if let Some(handler) = &self.on_axis_scroll {
                                    let screen_value = match axis.orientation() {
                                        Orientation::Horizontal => cursor_pos.x,
                                        Orientation::Vertical => cursor_pos.y,
                                    };

                                    let normalized =
                                        axis.screen_to_normalized(screen_value, &axis_bounds);

                                    // Capture scroll events to prevent other widgets from handling
                                    shell.capture_event();
                                    shell.publish(handler(id.clone(), normalized, *delta));
                                }
                                break;
                            }
                        }
                    }
                }
            }
            // This is missing touch support for zooming - We need to track fingers (TBD - Later)
            _ => {}
        }

        shell.request_redraw();
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let style = theme.style(&self.class);
        let bounds = layout.bounds();
        let plot_bounds = self.get_plot_layout(layout).bounds();

        // Fill in background
        renderer.fill_quad(
            Quad {
                bounds,
                ..Default::default()
            },
            Color::TRANSPARENT,
        );

        // 1. Retrieve the Memory from the Tree directly
        let memory = tree.state.downcast_ref::<Memory<AxisId>>();

        // 2. Lock the tessellators for the duration of this draw call
        let mut tessellators = memory.tessellators.borrow_mut();

        // Init mesh-rendering dependencies
        let mut mesh_buffer = render::MeshBuffer::new(100_000);
        let screen_rect = ScreenRect {
            x: plot_bounds.x,
            y: plot_bounds.y,
            width: plot_bounds.width,
            height: plot_bounds.height,
        };

        // Render axes and grids
        for (i, (_, axis)) in self.state.axes().iter().enumerate() {
            let axis_layout = layout.children().nth(i).unwrap();
            axis.draw::<Renderer>(
                renderer,
                &style,
                axis_layout,
                cursor,
                &plot_bounds,
                &mut mesh_buffer,
                &bounds,
            );
        }

        // Make sure we render the mesh buffer after adding axes/grids to it
        mesh_buffer.render(renderer, &bounds);

        // Render layers
        for layer in &self.layers {
            // This can never fail due to layer verification upon chart initialization
            let x_axis = self.state.axis(&layer.horizontal_axis_id);
            let y_axis = self.state.axis(&layer.vertical_axis_id);
            let transform = Transform::new(&screen_rect, x_axis.deref(), y_axis.deref());

            // Pass the dereferenced mut borrow of tessellators
            let mut plot: Plot<Domain, Renderer> = Plot::new(
                &mut tessellators,
                renderer,
                &plot_bounds,
                &mut mesh_buffer,
                &transform,
            );
            layer.items.draw(&mut plot, theme);
        }

        // --- NEW DEBUG OVERLAY CODE ---
        if self.debug {
            renderer.start_layer(bounds);
            // Get total counts from the buffer (includes all flushed layers)
            let v_count = mesh_buffer.total_vertices();
            let i_count = mesh_buffer.total_indices();

            // Color Coding: Green (Good), Yellow (Heavy), Red (Critical)
            let color = if v_count < 50_000 {
                Color::from_rgb(0.0, 0.7, 0.0) // Dark Green
            } else if v_count < 200_000 {
                Color::from_rgb(0.9, 0.7, 0.0) // Orange/Yellow
            } else {
                Color::from_rgb(0.9, 0.0, 0.0) // Red
            };

            let text = format!("Vertices: {} | Indices: {}", v_count, i_count);

            let position = [bounds.x + 10.0, bounds.y + 10.0];

            let text = iced_core::Text {
                content: text,
                bounds: Size::new(500., 500.),
                size: 32.into(),
                line_height: LineHeight::default(),
                font: renderer.default_font(),
                align_x: iced_core::text::Alignment::Left,
                align_y: iced_core::alignment::Vertical::Top,
                shaping: Shaping::Basic,
                wrapping: Wrapping::None,
            };

            // Draw Text
            renderer.fill_text(text, position.into(), color, bounds);
            renderer.end_layer();
        }
    }
}

impl<'a, AxisId, Domain, Message, Theme, Renderer>
    From<Chart<'a, AxisId, Domain, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    AxisId: Hash + Eq + Debug + Clone + 'static,
    Domain: Float,
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: plot::Renderer,
{
    fn from(plot: Chart<'a, AxisId, Domain, Message, Theme, Renderer>) -> Self {
        Element::new(plot)
    }
}

#[inline(always)]
fn layout_horizontal_axis<Domain: Float>(
    chart_width: f32,
    axis: &Axis<Domain>,
    x: f32,
    y: f32,
    height: f32,
) -> Node {
    let limits = Limits::new(
        Size::new(chart_width, height),
        Size::new(chart_width, height),
    );
    axis.layout(&limits).move_to(Point::new(x, y))
}

#[inline(always)]
fn layout_vertical_axis<Domain: Float>(
    chart_height: f32,
    axis: &Axis<Domain>,
    x: f32,
    y: f32,
    width: f32,
) -> Node {
    let limits = Limits::new(
        Size::new(width, chart_height),
        Size::new(width, chart_height),
    );
    axis.layout(&limits).move_to(Point::new(x, y))
}

#[inline(always)]
fn verify_layer<'a, AxisId: Hash + Eq + Clone, Domain: Float, Renderer, Theme>(
    layer: &Layer<'a, AxisId, Domain, Renderer, Theme>,
    state: &'a State<AxisId, Domain>,
    errors: &mut Vec<Error<AxisId>>,
) -> bool {
    let x_id = &layer.horizontal_axis_id;
    let y_id = &layer.vertical_axis_id;

    if x_id == y_id {
        errors.push(Error::DuplicateAxis { id: x_id.clone() });
        return false;
    }

    let Some(x) = state.axis_opt(x_id) else {
        errors.push(Error::UnknownAxis { id: x_id.clone() });
        return false;
    };
    let Some(y) = state.axis_opt(y_id) else {
        errors.push(Error::UnknownAxis { id: y_id.clone() });
        return false;
    };

    let horizontal_orientation = x.orientation();
    let vertical_orientation = y.orientation();
    if horizontal_orientation == vertical_orientation {
        errors.push(Error::AxisConflict {
            horizontal: x_id.clone(),
            horizontal_orientation,
            vertical: y_id.clone(),
            vertical_orientation,
        });
        return false;
    }

    true
}
