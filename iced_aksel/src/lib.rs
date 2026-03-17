//! A high-performance plotting library for Iced applications.
//!
//! `iced_aksel` provides interactive charts and plots for the Iced GUI framework,
//! built on top of the `aksel` plotting core. It offers flexible axis configuration,
//! multiple shape primitives, and robust interaction handling.
//!
//! # Quick Start
//!
//! To create a simple chart, you need to:
//! 1. Define your [`State`] (which stores axes configuration).
//! 2. Implement [`PlotData`] for your data type.
//! 3. Instantiate the [`Chart`] widget in your view logic.
//!
//! ```rust,no_run
//! use iced_aksel::{
//!     Chart, State, Axis, Plot, PlotPoint, axis, scale::Linear,
//!     plot::PlotData, shape::Ellipse, Measure
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
//!         // Register axes with unique IDs
//!         chart_state.set_axis("x", Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom));
//!         chart_state.set_axis("y", Axis::new(Linear::new(0.0, 100.0), axis::Position::Left));
//!
//!         Self {
//!             chart_state,
//!             data: ScatterData {
//!                 points: vec![
//!                     PlotPoint::new(10.0, 20.0),
//!                     PlotPoint::new(50.0, 80.0),
//!                 ],
//!             },
//!         }
//!     }
//!
//!     fn view(&self) -> Element<Message> {
//!         // Render the chart using the persistent state
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
//!                 Ellipse::new(*point, Measure::Screen(5.0), Measure::Screen(5.0))
//!                     .fill(theme.palette().primary)
//!             );
//!         }
//!     }
//! }
//! ```
//!
//! # Core Concepts
//!
//! - **[`Chart`]**: The main widget that renders axes and data. It handles layout and user events.
//! - **[`State`]**: A persistent struct that manages axis configuration. You should store this in your application's state.
//! - **[`Axis`]**: Configures scales (Linear, Log), ticks, grid lines, and labels.
//! - **[`PlotData`]**: A trait you implement for your own data types to define how they should be rendered.
//! - **[`Shape`]**: Visual primitives (lines, circles, rectangles) used within `PlotData::draw`.

use aksel::ScreenRect;
use derive_more::{Display, Error};
use iced_core::{
    Border, Clipboard, Color, Element, Event, Font, Layout, Length, Padding, Point, Rectangle,
    Shell, Size, Widget, keyboard,
    layout::{self, Limits, Node},
    mouse,
    renderer::{Quad, Style},
    widget::{Tree, tree},
};
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Deref;

// Re-export aksel core types for convenience
pub use aksel::{Float, Transform, scale, scale::Scale, transform, transform::PlotPoint};

mod action;
mod event;
mod layer;
mod measure;
mod memory;
mod render;
mod state;

pub mod axis;
pub mod interaction;
pub mod plot;
pub mod radii;
pub mod shape;
pub mod stroke;
pub mod style;

pub use axis::Axis;
pub use event::*;
pub use interaction::Interaction;
pub use layer::{Cached, LayerId};
pub use measure::Measure;
pub use plot::{Plot, PlotData};
pub use radii::Radii;
pub use render::{Quality, Renderer};
pub use shape::Shape;
pub use state::State;
pub use stroke::Stroke;
pub use style::Catalog;

use crate::interaction::InteractionQuery;
use crate::interaction::area::ResolvedArea;
use crate::memory::{CacheSignature, HoverIdentity};
use crate::render::{LineArrows, LineExtensions, Primitive};
use crate::stroke::ResolvedStroke;
use action::Action;
use axis::{MarkerContext, MarkerPosition, MarkerRequest, Orientation, Position};
use layer::Layer;
use memory::Memory;

/// Default movement threshold (in pixels) to distinguish a click from a drag operation.
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
    /// Two axes have conflicting orientations (e.g., both are horizontal).
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
        id: AxisId,
    },
}

// Internal type aliases for plot event handlers
type ErrorHandler<AxisId, Message> = event::Handler<Message, (Error<AxisId>,)>;
type MoveHandler<Message> = event::Handler<Message, (MoveEvent<Point>,)>;
type HoverHandler<Message> = event::Handler<Message, (keyboard::Modifiers,)>;
type HoverMultipleHandler<Message, T> =
    event::Handler<Message, (Vec<interaction::Id<T>>, keyboard::Modifiers)>;
type DragHandler<Message> = event::Handler<Message, (DragEvent<Delta>,)>;
type ScrollHandler<Message> = event::Handler<Message, (ScrollEvent<Point>,)>;
type PressHandler<Message> = event::Handler<Message, (PressEvent<Point>,)>;
type ReleaseHandler<Message> = event::Handler<Message, (ReleaseEvent<Point>,)>;

// Internal type aliases for axis event handlers
type AxisHoverHandler<AxisId, Message> = event::Handler<Message, (AxisId, f32)>;
type AxisDragHandler<AxisId, Message> = event::Handler<Message, (AxisId, DragEvent<f32>)>;
type AxisScrollHandler<AxisId, Message> = event::Handler<Message, (AxisId, ScrollEvent<f32>)>;
type AxisPressHandler<AxisId, Message> = event::Handler<Message, (AxisId, PressEvent<f32>)>;
type AxisReleaseHandler<AxisId, Message> = event::Handler<Message, (AxisId, ReleaseEvent<f32>)>;

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
///
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
    Tag = (),
    Theme = iced_core::Theme,
    Renderer = iced_renderer::Renderer,
> where
    AxisId: Hash + Eq + Clone + Debug,
    Domain: Float,
    Theme: Catalog,
    Renderer: crate::Renderer,
    Message: Clone,
    Tag: Hash + Eq + Clone,
{
    state: &'a State<AxisId, Domain, Theme>,
    layers: Vec<Layer<'a, AxisId, Domain, Message, Tag, Renderer, Theme>>,
    width: Length,
    height: Length,
    class: <Theme as Catalog>::Class<'a>,
    errors: Vec<Error<AxisId>>,
    drag_deadband: f32,
    padding: Padding,
    quality: Quality,
    markers: Vec<MarkerRequest<'a, AxisId, Domain, Theme>>,

    // Fonts
    axis_font: Option<Font>,

    // Interaction Handlers
    on_error: Option<ErrorHandler<AxisId, Message>>,

    // Plot Area Handlers
    on_press: Option<PressHandler<Message>>,
    on_release: Option<ReleaseHandler<Message>>,
    on_drag: Option<DragHandler<Message>>,
    on_hover: Option<HoverHandler<Message>>,
    on_hover_all: Option<HoverMultipleHandler<Message, Tag>>,
    on_move: Option<MoveHandler<Message>>,
    on_scroll: Option<ScrollHandler<Message>>,

    // Axis Handlers
    on_axis_press: Option<AxisPressHandler<AxisId, Message>>,
    on_axis_release: Option<AxisReleaseHandler<AxisId, Message>>,
    on_axis_drag: Option<AxisDragHandler<AxisId, Message>>,
    on_axis_hover: Option<AxisHoverHandler<AxisId, Message>>,
    on_axis_scroll: Option<AxisScrollHandler<AxisId, Message>>,

    debug: bool,
}

impl<'a, AxisId, Domain, Message: std::clone::Clone, Tag, Theme, Renderer>
    Chart<'a, AxisId, Domain, Message, Tag, Theme, Renderer>
where
    Domain: Float,
    AxisId: Hash + Eq + Clone + Debug,
    Tag: Hash + Eq + Clone + Debug,
    Theme: Catalog,
    Renderer: crate::Renderer,
{
    /// Creates a new chart from the given state.
    ///
    /// The `State` contains the configuration of all axes. It is separated from the
    /// `Chart` widget to allow persistence across frames and ease of manipulation
    /// (e.g., zooming/panning) from your update logic.
    pub fn new(state: &'a State<AxisId, Domain, Theme>) -> Self {
        Self {
            state,
            layers: vec![],
            width: Length::Fill,
            height: Length::Fill,
            class: <Theme as Catalog>::default(),
            errors: vec![],
            drag_deadband: DEFAULT_DRAG_DEADBAND,
            padding: Padding::new(0.),
            quality: Quality::Medium,
            markers: Vec::with_capacity(state.axes().len()),

            // Handlers and fonts default to None
            axis_font: None,
            on_error: None,
            on_drag: None,
            on_hover: None,
            on_hover_all: None,
            on_move: None,
            on_scroll: None,
            on_press: None,
            on_release: None,
            on_axis_press: None,
            on_axis_release: None,
            on_axis_drag: None,
            on_axis_hover: None,
            on_axis_scroll: None,

            debug: false,
        }
    }

    /// Sets the style of the chart.
    pub fn style(mut self, style: <Theme as Catalog>::Class<'a>) -> Self {
        self.class = style;
        self
    }

    /// Enables or disables the debug overlay.
    ///
    /// When enabled, an overlay will display performance metrics such as
    /// vertex and index counts in the corner of the chart.
    pub const fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Sets the global rendering quality multiplier.
    ///
    /// Controls the Level of Detail (LOD) for curves and text.
    /// * `1.0`: Standard quality (Default).
    /// * `< 1.0`: Lower quality, higher performance.
    /// * `> 1.0`: Higher quality, smoother curves.
    pub const fn quality(mut self, quality: Quality) -> Self {
        self.quality = quality;
        self
    }

    /// Sets the font used to render the [`Axis`] labels and [`axis::Marker`]
    pub const fn axes_font(mut self, font: Font) -> Self {
        self.axis_font = Some(font);
        self
    }

    /// Adds a data layer to the chart.
    ///
    /// The data will be plotted using the coordinate system defined by the two specified axes.
    /// Multiple layers can be added to a single chart, potentially using different axes.
    ///
    /// ***OBS***: It's important to note that the axis ID's **must** be present in [`State`]
    pub fn plot_data<T: plot::PlotData<Domain, Message, Tag, Renderer, Theme>>(
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
    pub const fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the chart.
    pub const fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the padding around the chart.
    ///
    /// This adds space between the widget bounds and the chart contents (axes/plot).
    pub const fn padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    /// Sets the minimum drag distance in pixels before drag events are triggered.
    ///
    /// Default is 10 pixels. This helps prevent accidental drags when the user intended to click.
    pub const fn drag_deadband(mut self, distance: f32) -> Self {
        self.drag_deadband = distance;
        self
    }

    /// Sets a marker to be drawn on the given axis, at the given position, if the position is Some
    pub fn marker_maybe<F>(
        mut self,
        axis_id: &'a AxisId,
        position: Option<MarkerPosition<Domain>>,
        renderer: F,
    ) -> Self
    where
        F: for<'ctx> Fn(MarkerContext<'ctx, Domain, Theme>) -> Option<axis::Marker> + 'static,
    {
        if let Some(position) = position {
            self.markers.push(MarkerRequest {
                axis_id,
                position,
                renderer: Box::new(renderer),
            });
        }
        self
    }

    /// Sets a marker to be drawn on the given axis, at the given position
    pub fn marker<F>(
        mut self,
        axis_id: &'a AxisId,
        position: MarkerPosition<Domain>,
        renderer: F,
    ) -> Self
    where
        F: for<'ctx> Fn(MarkerContext<'ctx, Domain, Theme>) -> Option<axis::Marker> + 'static,
    {
        self.markers.push(MarkerRequest {
            axis_id,
            position,
            renderer: Box::new(renderer),
        });
        self
    }

    event::impl_handlers!(
        /// Sets the event handler for chart configuration errors.
        ///
        /// Errors can occur when axes referenced in `plot_data` are missing from the `State`
        /// or have conflicting orientations.
        error: (Error<AxisId>,);

        /// Sets the event handler for mouse presses on the main plot area.
        press: (PressEvent<Point>,);

        /// Sets the event handler for mouse releases on the main plot area.
        release: (ReleaseEvent<Point>,);

        /// Sets the event handler for drag events on the main plot area.
        drag: (DragEvent<Delta>,);

        /// Sets the event handler for when the mouse hovers the main plot area.
        hover: (keyboard::Modifiers,);

        /// Sets the event handler for when multiple interactions are hovered at once,
        /// returning every interaction::Id currently hovered.
        hover_all: (Vec<interaction::Id<Tag>>, keyboard::Modifiers);

        /// Sets the event handler for hover identity changes
        move: (MoveEvent<Point>,);

        /// Sets a callback for scroll events (mouse wheel) on the main plot area.
        scroll: (ScrollEvent<Point>,);


        /// Sets the event handler for mouse presses on an axis.
        axis_press: (AxisId, PressEvent<f32>);

        /// Sets the event handler for mouse releases on an axis.
        axis_release: (AxisId, ReleaseEvent<f32>);

        /// Sets the event handler for dragging on an axis.
        axis_drag: (AxisId, DragEvent<f32>);

        /// Sets the event handler for hovering on an axis.
        axis_hover: (AxisId, f32);

        /// Sets the event handler for scrolling on an axis
        axis_scroll: (AxisId, ScrollEvent<f32>);
    );

    /// Internal handler for mouse press events.
    /// Determines if the user mouse-pressed on the plot or an axis and updates the internal state.
    fn handle_mouse_press(
        &self,
        memory: &mut Memory<AxisId, Message, Tag, Renderer>,
        layout: Layout,
        shell: &mut Shell<'_, Message>,
        click: mouse::Click,
        button: mouse::Button,
    ) {
        // If we press during any other action than idle, we must return
        if Action::Idle != memory.action {
            return;
        }

        let plot_bounds = self.get_plot_layout(layout).bounds();
        let mouse_pos = click.position();

        // Check if press is on the plot area
        if plot_bounds.contains(mouse_pos) {
            shell.capture_event();

            let mut interaction_id = None;
            let interactions = memory.interaction_cache.borrow();

            // Build the query with a 5px hover/click tolerance
            let query = InteractionQuery::Point {
                position: mouse_pos,
                tolerance_px: 5.0,
            };

            for (id, interaction) in interactions.query(&query).into_iter().rev() {
                if let Some(handler) = &interaction.on_press {
                    // TODO: Priority sorting - Which id should we actually save?
                    // We just save the top-most Id for now
                    if interaction_id.is_none() && interaction.on_drag.is_some() {
                        interaction_id = Some(id.clone());
                    }

                    let normalized = Point::new(
                        (mouse_pos.x - plot_bounds.x) / plot_bounds.width,
                        1.0 - ((mouse_pos.y - plot_bounds.y) / plot_bounds.height),
                    );

                    let event = PressEvent::new(
                        normalized,
                        button,
                        click.kind(),
                        memory.keyboard_modifiers,
                    );

                    if let Some(message) = handler.run((id.clone(), event)) {
                        shell.publish(message);
                    }

                    // You can't press more than thing at a time
                    break;
                }
            }

            let handled = interaction_id.is_some();

            memory.action = Action::DraggingPlot {
                interaction_id,
                origin: mouse_pos,
                last_position: mouse_pos,
                total_delta: 0.0,
                button,
                click_kind: click.kind(),
            };

            // Make sure we don't test plot, if a shape was pressed already
            if handled {
                return;
            }

            if let Some(handler) = &self.on_press {
                let normalized = Point::new(
                    (mouse_pos.x - plot_bounds.x) / plot_bounds.width,
                    1.0 - ((mouse_pos.y - plot_bounds.y) / plot_bounds.height),
                );
                let event =
                    PressEvent::new(normalized, button, click.kind(), memory.keyboard_modifiers);
                if let Some(message) = handler.run((event,)) {
                    shell.publish(message);
                }
            }

            return;
        }

        // Check if press is on any axis
        for (i, (id, axis)) in self.state.axes().iter().enumerate() {
            let axis_bounds = layout.children().nth(i).unwrap().bounds();

            if !axis_bounds.contains(mouse_pos) {
                continue;
            }

            let origin = match axis.orientation() {
                Orientation::Horizontal => mouse_pos.x,
                Orientation::Vertical => mouse_pos.y,
            };

            shell.capture_event();

            memory.action = Action::DraggingAxis {
                id: id.clone(),
                origin,
                last_position: origin,
                total_delta: 0.0,
                button,
                click_kind: click.kind(),
            };

            // Handle double-click on axis
            if let Some(handler) = self.on_axis_press.as_ref()
                && let Some(message) = handler.run((
                    id.clone(),
                    PressEvent::new(
                        axis.screen_to_normalized(origin, &axis_bounds),
                        button,
                        click.kind(),
                        memory.keyboard_modifiers,
                    ),
                ))
            {
                shell.publish(message);
            }

            // We can only interact with one axis at a time
            return;
        }
    }

    /// Internal handler for mouse release events.
    /// Triggers click events if the drag distance was within the deadband.
    fn handle_mouse_release(
        &self,
        memory: &mut Memory<AxisId, Message, Tag, Renderer>,
        layout: Layout,
        shell: &mut Shell<'_, Message>,
        previous_click_kind: Option<mouse::click::Kind>,
        button: mouse::Button,
    ) {
        let Memory { action, .. } = memory;

        // If total drag exceeded deadband, it was a drag, not a click.
        let was_dragging = action
            .total_drag_delta()
            .is_some_and(|delta| delta > self.drag_deadband);

        match action {
            Action::Idle => (), // Do nothing
            Action::DraggingPlot { origin, .. } => {
                let plot_bounds = self.get_plot_layout(layout).bounds();
                let interactions = memory.interaction_cache.borrow();

                let query = InteractionQuery::Point {
                    position: *origin,
                    tolerance_px: 5.0,
                };

                for (id, interaction) in interactions.query(&query).into_iter().rev() {
                    if let Some(handler) = &interaction.on_release {
                        let normalized = Point::new(
                            (origin.x - plot_bounds.x) / plot_bounds.width,
                            1.0 - ((origin.y - plot_bounds.y) / plot_bounds.height),
                        );
                        let event = ReleaseEvent::new(
                            normalized,
                            button,
                            previous_click_kind,
                            memory.keyboard_modifiers,
                            was_dragging,
                        );

                        if let Some(message) = handler.run((id.clone(), event)) {
                            shell.publish(message);
                            // You can't press more than thing at a time
                            return;
                        }
                    }
                }

                if let Some(handler) = &self.on_release {
                    let normalized = Point::new(
                        (origin.x - plot_bounds.x) / plot_bounds.width,
                        1.0 - ((origin.y - plot_bounds.y) / plot_bounds.height),
                    );
                    if let Some(message) = handler.run((ReleaseEvent::new(
                        normalized,
                        button,
                        previous_click_kind,
                        memory.keyboard_modifiers,
                        was_dragging,
                    ),))
                    {
                        shell.publish(message);
                    }
                }
            }
            Action::DraggingAxis { id, origin, .. } => {
                if let Some((i, id, axis)) = self.state.axes().get_full(id) {
                    let axis_bounds = layout.children().nth(i).unwrap().bounds();
                    let normalized = axis.screen_to_normalized(*origin, &axis_bounds);
                    if let Some(handler) = &self.on_axis_release
                        && let Some(message) = handler.run((
                            id.clone(),
                            ReleaseEvent::new(
                                normalized,
                                button,
                                previous_click_kind,
                                memory.keyboard_modifiers,
                                was_dragging,
                            ),
                        ))
                    {
                        shell.publish(message);
                    }
                }
            }
        }
    }

    /// Internal handler for mouse movement.
    /// Manages hover states and processes drag deltas.
    ///
    /// Return true if hover identity has changed
    fn handle_mouse_moved(
        &self,
        memory: &mut Memory<AxisId, Message, Tag, Renderer>,
        layout: Layout,
        shell: &mut Shell<'_, Message>,
        mouse_pos: Point,
    ) -> bool {
        let Memory { action, .. } = memory;
        let plot_bounds = self.get_plot_layout(layout).bounds();

        // Mouse is over the plot area
        if plot_bounds.contains(mouse_pos) {
            match action {
                Action::DraggingAxis { .. } => (), // Ignore if dragging axis
                Action::Idle => {
                    let normalized = Point::new(
                        (mouse_pos.x - plot_bounds.x) / plot_bounds.width,
                        1.0 - ((mouse_pos.y - plot_bounds.y) / plot_bounds.height),
                    );
                    if let Some(handler) = &self.on_move
                        && let Some(message) =
                            handler.run((MoveEvent::new(normalized, memory.keyboard_modifiers),))
                    {
                        shell.publish(message);
                    }

                    let mut all = self.on_hover_all.as_ref().map(|_| vec![]);
                    let mut new_identity = None;

                    // Check the Interaction Registry for hovers!
                    let interactions = memory.interaction_cache.borrow();
                    let query = InteractionQuery::Point {
                        position: mouse_pos,
                        tolerance_px: 5.0,
                    };

                    for (id, _interaction) in interactions.query(&query).into_iter().rev() {
                        
                        let identity = HoverIdentity::Interaction(id.clone());
                        if new_identity.is_none() {
                            new_identity = Some(identity);
                        }

                        let Some(ids) = &mut all else {
                            // Early return if we don't look for all hovers
                            return false;
                        };

                        ids.push(id.clone());
                    }

                    if let Some(ids) = all
                        && let Some(message) = self
                            .on_hover_all
                            .as_ref()
                            .and_then(|f| f.run((ids, memory.keyboard_modifiers)))
                    {
                        shell.publish(message);
                    }

                    let identity = new_identity.unwrap_or(HoverIdentity::Plot);

                    if memory.last_hovered_identity != identity {
                        memory.last_hovered_identity = identity;
                        return true;
                    }

                    return false;
                }
                Action::DraggingPlot {
                    last_position,
                    total_delta,
                    interaction_id,
                    button,
                    click_kind,
                    ..
                } => {
                    let delta_x = mouse_pos.x - last_position.x;
                    let delta_y = mouse_pos.y - last_position.y;
                    *total_delta += delta_x.hypot(delta_y);
                    *last_position = mouse_pos;

                    if *total_delta < self.drag_deadband {
                        return false;
                    };

                    // Interaction present - Use that instead
                    if let Some(id) = interaction_id {
                        shell.capture_event();

                        let interactions = memory.interaction_cache.borrow();
                        let Some(interaction) = interactions.get(id) else {
                            return false;
                        };

                        let Some(handler) = interaction.on_drag.as_ref() else {
                            return false;
                        };

                        // For interaction: shape/interaction moves with cursor
                        // x: positive right, y: negative down (chart coords go up)
                        let normalized_delta = Delta {
                            x: delta_x / plot_bounds.width,
                            y: -delta_y / plot_bounds.height,
                        };

                        let event = DragEvent::new(
                            normalized_delta,
                            *button,
                            *click_kind,
                            memory.keyboard_modifiers,
                        );

                        if let Some(message) = handler.run((id.clone(), event)) {
                            shell.publish(message);
                            // Drag events can never propagate, so we return here
                            return false;
                        };
                    }

                    if let Some(handler) = &self.on_drag {
                        shell.capture_event();

                        // For chart: dragging right pans chart right (data moves left)
                        // x: negative right, y: positive down
                        let normalized_delta = Delta {
                            x: -delta_x / plot_bounds.width,
                            y: delta_y / plot_bounds.height,
                        };

                        let event = DragEvent::new(
                            normalized_delta,
                            *button,
                            *click_kind,
                            memory.keyboard_modifiers,
                        );

                        if let Some(message) = handler.run((event,)) {
                            shell.publish(message);
                        }
                    }

                    return false;
                }
            }
        }

        // Handle active axis drag
        if let Action::DraggingAxis {
            id: dragging_id,
            last_position,
            total_delta,
            button,
            click_kind,
            ..
        } = action
        {
            shell.capture_event();
            if let Some((i, (id, axis))) = self
                .state
                .axes()
                .iter()
                .enumerate()
                .find(|(_, (axis_id, _))| *axis_id == dragging_id)
            {
                let axis_bounds = layout.children().nth(i).unwrap().bounds();
                let screen_value = match axis.orientation() {
                    axis::Orientation::Horizontal => mouse_pos.x,
                    axis::Orientation::Vertical => mouse_pos.y,
                };

                let delta = screen_value - *last_position;
                *total_delta += delta.abs();
                *last_position = screen_value;

                if *total_delta > self.drag_deadband
                    && let Some(handler) = &self.on_axis_drag
                {
                    let normalized_delta = axis.translate_drag_delta(delta, &axis_bounds);
                    let event = DragEvent::new(
                        normalized_delta,
                        *button,
                        *click_kind,
                        memory.keyboard_modifiers,
                    );
                    if let Some(message) = handler.run((id.clone(), event)) {
                        shell.publish(message);
                    }
                }
            }
        }
        // Handle axis hover
        else if matches!(action, Action::Idle) {
            for (i, (id, axis)) in self.state.axes().iter().enumerate() {
                let axis_bounds = layout.children().nth(i).unwrap().bounds();

                if !axis_bounds.contains(mouse_pos) {
                    memory.last_hovered_identity = HoverIdentity::Axis(id.clone());
                    continue;
                }

                if let Some(handler) = &self.on_axis_hover {
                    let screen_value = match axis.orientation() {
                        axis::Orientation::Horizontal => mouse_pos.x,
                        axis::Orientation::Vertical => mouse_pos.y,
                    };
                    let normalized = axis.screen_to_normalized(screen_value, &axis_bounds);
                    if let Some(message) = handler.run((id.clone(), normalized)) {
                        shell.publish(message);
                    }
                }

                break;
            }
        }

        false
    }

    #[inline(always)]
    fn get_plot_layout<'b>(&self, layout: Layout<'b>) -> Layout<'b> {
        // The plot area is always the last child in the layout list
        layout.children().last().unwrap()
    }

    /// Iterates over axes to find the inner-most spines and draws connecting squares at the corners.
    fn draw_spine_corners(
        &self,
        layout: Layout<'_>,
        style: &style::Style,
        plot: Rectangle,
        renderer: &mut Renderer,
    ) {
        // Track the "inner-most" spine properties for each side
        let mut left: Option<(f32, Color)> = None;
        let mut right: Option<(f32, Color)> = None;
        let mut top: Option<(f32, Color)> = None;
        let mut bottom: Option<(f32, Color)> = None;

        // Track the edge coordinates to determine closeness to the plot
        let mut max_left_edge = f32::MIN;
        let mut min_right_edge = f32::MAX;
        let mut max_top_edge = f32::MIN;
        let mut min_bottom_edge = f32::MAX;

        // 1. Find the winners
        for (i, (_, axis)) in self.state.axes().iter().enumerate() {
            if !axis.is_visible() {
                continue;
            }

            if style.axis.spine.width.0 <= 0.0 {
                continue;
            }

            let style = axis.create_style(style).spine;
            let bounds = layout.children().nth(i).unwrap().bounds();
            let data = (style.width.0, style.color);

            match axis.position() {
                Position::Left => {
                    let edge = bounds.x + bounds.width;
                    if edge >= max_left_edge {
                        max_left_edge = edge;
                        left = Some(data);
                    }
                }
                Position::Right => {
                    let edge = bounds.x;
                    if edge <= min_right_edge {
                        min_right_edge = edge;
                        right = Some(data);
                    }
                }
                Position::Top => {
                    let edge = bounds.y + bounds.height;
                    if edge >= max_top_edge {
                        max_top_edge = edge;
                        top = Some(data);
                    }
                }
                Position::Bottom => {
                    let edge = bounds.y;
                    if edge <= min_bottom_edge {
                        min_bottom_edge = edge;
                        bottom = Some(data);
                    }
                }
            }
        }

        // 2. Render the corners

        // Bottom-Left
        if let (Some((lw, lc)), Some((bw, _))) = (left, bottom) {
            let top_left = Point::new(plot.x - lw, plot.y + plot.height);
            let size = Size::new(lw, bw);
            renderer.fill_quad(
                Quad {
                    bounds: Rectangle::new(top_left, size),
                    snap: true,
                    ..Default::default()
                },
                lc,
            );
        }

        // Top-Left
        if let (Some((lw, lc)), Some((tw, _))) = (left, top) {
            let top_left = Point::new(plot.x - lw, plot.y - tw);
            let size = Size::new(lw, tw);
            renderer.fill_quad(
                Quad {
                    bounds: Rectangle::new(top_left, size),
                    snap: true,
                    ..Default::default()
                },
                lc,
            );
        }

        // Bottom-Right
        if let (Some((rw, rc)), Some((bw, _))) = (right, bottom) {
            let top_left = Point::new(plot.x + plot.width, plot.y + plot.height);
            let size = Size::new(rw, bw);
            renderer.fill_quad(
                Quad {
                    bounds: Rectangle::new(top_left, size),
                    snap: true,
                    ..Default::default()
                },
                rc,
            );
        }

        // Top-Right
        if let (Some((rw, rc)), Some((tw, _))) = (right, top) {
            let top_left = Point::new(plot.x + plot.width, plot.y - tw);
            let size = Size::new(rw, tw);
            renderer.fill_quad(
                Quad {
                    bounds: Rectangle::new(top_left, size),
                    snap: true,
                    ..Default::default()
                },
                rc,
            );
        }
    }
}

impl<AxisId, Domain, Message, Theme, Tag, Renderer> Widget<Message, Theme, Renderer>
    for Chart<'_, AxisId, Domain, Message, Tag, Theme, Renderer>
where
    AxisId: Hash + Eq + Debug + Clone + 'static,
    Domain: Float + 'static,
    Tag: Hash + Eq + Clone + Debug + 'static,
    Renderer: crate::Renderer + iced_core::text::Renderer<Font = iced_core::Font> + 'static,
    Theme: Catalog,
    Message: Clone + 'static,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Memory<AxisId, Message, Tag, Renderer>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(Memory::<AxisId, Message, Tag, Renderer>::new())
    }

    fn children(&self) -> Vec<Tree> {
        // One child per Axis + one for the content (plot area).
        let mut children: Vec<Tree> = self.state.axes().iter().map(|_| Tree::empty()).collect();
        children.push(Tree::empty()); // content
        children
    }

    fn diff(&self, _tree: &mut Tree) {}

    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        let memory: &mut Memory<AxisId, Message, Tag, Renderer> = tree.state.downcast_mut();
        memory.make_sure_cache_is_initialized(renderer, self.quality);

        let bounds = limits.resolve(self.width, self.height, Size::ZERO);

        let axis_count = self.state.axes().len();

        // Pass 1: Measure total thickness required for axes on each side
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

        // Pass 2: Calculate the remaining area for the actual chart
        let chart_height = (bounds.height - top_total - bottom_total).max(0.0);
        let chart_width = (bounds.width - left_total - right_total).max(0.0);

        let chart_origin = Point::new(left_total, top_total);
        let chart_size = Size::new(chart_width, chart_height);

        // Pass 3: Generate layout nodes for everything
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

        // Add the chart content node (center plot)
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
                if let Some(message) = handler.run((error,)) {
                    shell.publish(message);
                }
            }
            return;
        }

        let signature = CacheSignature::new(self.state, &layout, &self.layers);
        let memory: &mut Memory<AxisId, Message, Tag, Renderer> = tree.state.downcast_mut();
        memory.update(signature);
        memory.update_partitions(self.get_plot_layout(layout).bounds());

        // Only handle events if the cursor is near the chart
        let bounds = layout.bounds();
        let Some(mouse_pos) = cursor.position_over(bounds) else {
            return;
        };

        // Handle input events
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(button)) => {
                let new_click = memory.update_click(mouse_pos, *button);
                self.handle_mouse_press(memory, layout, shell, new_click, *button);
            }
            Event::Mouse(mouse::Event::ButtonReleased(button)) => {
                let previous_click_kind = memory.previous_click.take().map(|c| c.kind());
                self.handle_mouse_release(memory, layout, shell, previous_click_kind, *button);
                memory.action = Action::Idle;
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                let changed = self.handle_mouse_moved(memory, layout, shell, *position);
                if changed {
                    println!("Identity changed! {:?}", memory.last_hovered_identity);
                    match &memory.last_hovered_identity {
                        HoverIdentity::Plot => {
                            if let Some(message) = self
                                .on_hover
                                .as_ref()
                                .and_then(|handler| handler.run((memory.keyboard_modifiers,)))
                            {
                                shell.publish(message);
                            }
                        }
                        HoverIdentity::Interaction(id) => {
                            if let Some(message) = memory
                                .interaction_cache
                                .borrow()
                                .get(id)
                                .and_then(|interaction| {
                                    interaction.on_hover.as_ref().map(
                                        |handler: &Handler<
                                            Message,
                                            (interaction::Id<Tag>, keyboard::Modifiers),
                                        >| {
                                            handler.run((id.clone(), memory.keyboard_modifiers))
                                        },
                                    )
                                })
                                .flatten()
                            {
                                shell.publish(message);
                            }
                        }
                        _ => unimplemented!(),
                    }
                }
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                if let Some(cursor_pos) = cursor.position() {
                    let plot_bounds = self.get_plot_layout(layout).bounds();

                    // Check if scrolling over the plot area
                    if cursor.position_over(plot_bounds).is_some() {
                        if let Some(handler) = &self.on_scroll {
                            shell.capture_event();

                            // Normalize cursor position (0.0-1.0)
                            let normalized = Point::new(
                                (cursor_pos.x - plot_bounds.x) / plot_bounds.width,
                                1.0 - ((cursor_pos.y - plot_bounds.y) / plot_bounds.height),
                            );

                            let event =
                                ScrollEvent::new(normalized, *delta, memory.keyboard_modifiers);

                            if let Some(message) = handler.run((event,)) {
                                shell.publish(message);
                            }
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

                                    shell.capture_event();

                                    let event = ScrollEvent::new(
                                        normalized,
                                        *delta,
                                        memory.keyboard_modifiers,
                                    );

                                    if let Some(message) = handler.run((id.clone(), event)) {
                                        shell.publish(message);
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
            }
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                memory.update_modifiers(*modifiers)
            }
            // TODO: Add multi-touch support for zooming
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
        let screen_rect = ScreenRect {
            x: plot_bounds.x,
            y: plot_bounds.y,
            width: plot_bounds.width,
            height: plot_bounds.height,
        };

        // Retrieve the Memory from the Tree directly
        let memory = tree
            .state
            .downcast_ref::<Memory<AxisId, Message, Tag, Renderer>>();

        // Get cache from memory
        let Some(mut cache) = memory.get_cache_mut() else {
            return;
        };

        // Draw axes
        for (i, (_, axis)) in self.state.axes().iter().enumerate() {
            // We only care about layout bounds here to determine position
            let axis_layout = layout.children().nth(i).unwrap();

            // Draw the axis itself (Ticks, labels, spine and gridlines)
            axis.draw::<Renderer>(
                renderer,
                theme,
                &style,
                axis_layout,
                &plot_bounds,
                &mut cache,
                &bounds,
            );
        }

        // Connect axis spines
        self.draw_spine_corners(layout, &style, plot_bounds, renderer);

        // Draw data layers if the cache needs redraw
        if cache.needs_redraw() {
            let mut interactions = memory.interaction_cache.borrow_mut();

            for layer in &self.layers {
                // These axes are guaranteed to exist because of `verify_layer` check
                let x_axis = self.state.axis(&layer.horizontal_axis_id);
                let y_axis = self.state.axis(&layer.vertical_axis_id);
                let transform = Transform::new(&screen_rect, x_axis.deref(), y_axis.deref());

                let mut plot: Plot<Domain, Message, Tag, Renderer> =
                    Plot::new(renderer, &mut cache, &transform, &mut interactions);

                // User code draws shapes into the plot here
                layer.items.draw(&mut plot, theme);
            }

            // Populate the debug cache
            if self.debug {
                if let Some(debug_cache_cell) = &memory.debug_cache {
                    let mut debug_cache = debug_cache_cell.borrow_mut();

                    // Safely recreate the mesh to clear old debug primitives
                    let mut new_debug_cache = match renderer.preferred_backend() {
                        crate::render::Backend::Mesh => crate::render::RenderCache::new_mesh(),
                        crate::render::Backend::Path => crate::render::RenderCache::new_path(),
                    };
                    new_debug_cache.set_quality(self.quality);

                    for (_, interaction) in interactions.iter() {
                        if let Some(primitive) = build_debug_primitive(&interaction.area) {
                            new_debug_cache.add_primitive(primitive);
                        }
                    }

                    // Assign the fresh cache
                    *debug_cache = new_debug_cache;
                }
            }
        }

        // Draw markers
        for marker_request in &self.markers {
            let Some((idx, _id, axis)) = self.state.axes().get_full(marker_request.axis_id) else {
                continue;
            };

            let axis_bounds = layout.child(idx).bounds();

            let Some((marker, normalized_position)) = marker_request.create_marker(
                axis,
                &axis_bounds,
                &plot_bounds,
                cursor,
                &style.axis,
                theme,
            ) else {
                continue;
            };

            axis.draw_marker_overlay(
                renderer,
                normalized_position,
                marker,
                axis_bounds,
                &bounds,
                style.axis.text_offset,
            );
        }

        // DEBUG!
        // memory.draw_partitions(renderer, plot_bounds);

        // Draw the currently cached primitives
        cache.draw(renderer, &plot_bounds);

        // Draw exact mathematical shape interactions if debug is enabled
        if self.debug {
            if let Some(mut debug_cache) = memory.get_debug_cache_mut() {
                debug_cache.draw(renderer, &plot_bounds);
            };
        }
        // --- INTERACTION DEBUG VISUALIZER ---
        if self.debug {
            for (_, interaction) in memory.interaction_cache.borrow().iter() {
                // Only draw the intersection of the bounding box and the plot area
                if let Some(clipped_bounds) = interaction.bounding_box.intersection(&plot_bounds) {
                    renderer.fill_quad(
                        Quad {
                            bounds: clipped_bounds,
                            border: iced_core::Border::default()
                                .color(Color::from_rgba(1.0, 0.0, 0.0, 0.8))
                                .width(1.0),
                            ..Default::default()
                        },
                        Color::from_rgba(1.0, 0.0, 0.0, 0.1),
                    );
                }
            }
        }
    }
}

// Boilerplate conversions and helpers

impl<'a, AxisId, Domain, Message, Tag, Theme, Renderer>
    From<Chart<'a, AxisId, Domain, Message, Tag, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    AxisId: Hash + Eq + Debug + Clone + 'static,
    Domain: Float + 'static,
    Message: Clone + 'a + 'static,
    Theme: Catalog + 'a,
    Renderer: crate::Renderer + iced_core::text::Renderer<Font = iced_core::Font> + 'static,
    Tag: Hash + Eq + Clone + Debug + 'static,
{
    fn from(plot: Chart<'a, AxisId, Domain, Message, Tag, Theme, Renderer>) -> Self {
        Element::new(plot)
    }
}

#[inline(always)]
fn layout_horizontal_axis<Domain: Float, Theme>(
    chart_width: f32,
    axis: &Axis<Domain, Theme>,
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
fn layout_vertical_axis<Domain: Float, Theme>(
    chart_height: f32,
    axis: &Axis<Domain, Theme>,
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
fn verify_layer<
    'a,
    AxisId: Hash + Eq + Clone,
    Domain: Float,
    Message,
    Tag: Hash + Eq + Clone,
    Renderer,
    Theme,
>(
    layer: &Layer<'a, AxisId, Domain, Message, Tag, Renderer, Theme>,
    state: &'a State<AxisId, Domain, Theme>,
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

/// Translates a mathematical interaction area into a renderable stroke outline.
pub(crate) fn build_debug_primitive(area: &ResolvedArea) -> Option<Primitive> {
    // A standard 1px red stroke for our X-Ray lines
    let debug_stroke = ResolvedStroke {
        thickness: 1.0,
        fill: Color::from_rgba(1.0, 0.0, 0.0, 0.8),
        style: crate::stroke::StrokeStyle::Solid,
    };

    match area {
        ResolvedArea::Rect(rect) => Some(Primitive::Rectangle {
            xy1: Point::new(rect.x, rect.y),
            xy2: Point::new(rect.x + rect.width, rect.y + rect.height),
            fill: None,
            stroke: Some(debug_stroke),
        }),
        ResolvedArea::LineSegment {
            p1,
            p2,
            stroke_width_px,
        } => Some(Primitive::Line {
            start: *p1,
            end: *p2,
            stroke: ResolvedStroke {
                thickness: *stroke_width_px,
                ..debug_stroke
            },
            clip_bounds: Rectangle::INFINITE, // Will be clipped by plot bounds later
            extensions: LineExtensions {
                start: false,
                end: false,
            },
            arrows: LineArrows {
                start: false,
                end: false,
                size: 0.0,
            },
        }),
        ResolvedArea::Ellipse { center, rx, ry } => Some(Primitive::Ellipse {
            center: *center,
            radii: crate::radii::ResolvedRadii { x: *rx, y: *ry },
            fill: None,
            stroke: Some(debug_stroke),
        }),
        ResolvedArea::Triangle { p1, p2, p3 } => Some(Primitive::Triangle {
            points: [*p1, *p2, *p3],
            fill: None,
            stroke: Some(debug_stroke),
        }),
        ResolvedArea::Polygon { points } => Some(Primitive::Area {
            points: points.clone(),
            fill: None,
            stroke: Some(debug_stroke),
        }),
        ResolvedArea::Polyline {
            points,
            stroke_width_px,
        } => Some(Primitive::PolyLine {
            points: points.clone(),
            stroke: ResolvedStroke {
                thickness: *stroke_width_px,
                ..debug_stroke
            },
            clip_bounds: Rectangle::INFINITE,
            extensions: LineExtensions {
                start: false,
                end: false,
            },
            arrows: LineArrows {
                start: false,
                end: false,
                size: 0.0,
            },
        }),
        ResolvedArea::RegularPolygon {
            center,
            radius_px,
            vertices,
            rotation_rads,
        } => Some(Primitive::Polygon {
            center: *center,
            radius: crate::radii::ResolvedRadius(*radius_px),
            vertices: *vertices,
            rotation: iced_core::Radians(*rotation_rads),
            fill: None,
            stroke: Some(debug_stroke),
        }),
        ResolvedArea::Arc {
            center,
            radius_outer,
            radius_inner,
            start_angle,
            end_angle,
        } => Some(Primitive::Arc {
            center: *center,
            radius_inner: Some(crate::radii::ResolvedRadius(*radius_inner)),
            radius_outer: crate::radii::ResolvedRadius(*radius_outer),
            start_angle: iced_core::Radians(*start_angle),
            end_angle: iced_core::Radians(*end_angle),
            fill: None,
            stroke: Some(debug_stroke),
        }),
        ResolvedArea::Custom(_) => None, // Cannot easily draw custom dynamic interactions
    }
}
