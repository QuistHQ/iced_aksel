use std::hash::Hash;

use aksel::Float;
use derivative::Derivative;
use indexmap::{IndexMap, map::IterMut};

use crate::Axis;

/// Manages the configuration and runtime state of chart axes.
///
/// `State` acts as the central registry for all active axes. It provides methods to
/// register, access, and manipulate axes using unique identifiers. Additionally,
/// it offers helper functions to facilitate coordinate transformations and interaction.
///
/// This struct is designed to be stored in your application's model (persisting across frames)
/// and passed by reference to the `Chart` widget during the view/render phase.
///
/// # Example
///
/// ```rust
/// use iced_aksel::{State, Axis, axis, Chart, scale::Linear};
///
/// // 1. Initialize the state (store this in your app struct)
/// # #[derive(Clone)] enum Message {}
/// let mut chart_state: State<&str, f64> = State::new();
/// chart_state.set_axis("x_axis_id", Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom));
/// chart_state.set_axis("y_axis_id", Axis::new(Linear::new(0.0, 100.0), axis::Position::Right));
///
/// // 2. Pass the state to the Chart during rendering
/// let chart: Chart<&str, f64, Message> = Chart::new(&chart_state);
/// ```
#[derive(Default, Derivative)]
#[derivative(Debug)]
pub struct State<AxisId: Hash + Eq, Domain, Theme = iced_core::Theme> {
    axes: IndexMap<AxisId, Axis<Domain, Theme>>,
}

impl<AxisId, D, Theme> State<AxisId, D, Theme>
where
    AxisId: Hash + Eq + Clone,
    D: Float,
{
    /// Creates a new empty chart state with no axes.
    ///
    /// Use [`set_axis`](Self::set_axis) or [`with_axis`](Self::with_axis) to add axes.
    pub fn new() -> Self {
        Self {
            axes: IndexMap::new(),
        }
    }

    /// Builder-style method to add an axis during construction.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use iced_aksel::{State, Axis, axis, scale::Linear};
    /// let state: State<&str, f64> = State::new()
    ///     .with_axis("x", Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom))
    ///     .with_axis("y", Axis::new(Linear::new(0.0, 100.0), axis::Position::Left));
    /// ```
    pub fn with_axis(mut self, id: impl Into<AxisId>, axis: Axis<D, Theme>) -> Self {
        self.axes.insert(id.into(), axis);
        self
    }

    /// Returns a reference to all axes in the state.
    pub const fn axes(&self) -> &IndexMap<AxisId, Axis<D, Theme>> {
        &self.axes
    }

    /// Adds or replaces an axis with the given ID.
    ///
    /// Returns the previous axis if one existed with this ID.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use iced_aksel::{State, Axis, axis, scale::Linear};
    /// let mut state: State<&str, f64> = State::new();
    /// state.set_axis("x", Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom));
    /// ```
    pub fn set_axis(
        &mut self,
        id: impl Into<AxisId>,
        axis: Axis<D, Theme>,
    ) -> Option<Axis<D, Theme>> {
        self.axes.insert(id.into(), axis)
    }

    /// Removes an axis from the state.
    ///
    /// Returns the removed axis if it existed.
    pub fn remove_axis(&mut self, id: &AxisId) -> Option<Axis<D, Theme>> {
        self.axes.swap_remove(id)
    }

    /// Checks if an axis with the given ID exists.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use iced_aksel::State;
    /// # let state: State<&str, f64> = State::new();
    /// if state.has_axis(&"x") {
    ///     // axis exists
    /// }
    /// ```
    pub fn has_axis(&self, id: &AxisId) -> bool {
        self.axes.contains_key(id)
    }

    // -------------------------------------------------------------------------
    // B. Data Access
    // -------------------------------------------------------------------------

    /// Returns a reference to an axis by ID.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use iced_aksel::{State, Axis, axis, scale::Linear};
    /// # let mut state: State<&'static str, f64> = State::new();
    /// # state.set_axis("x", Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom));
    /// let x_axis =  state.axis(&"x");
    /// let (min, max) = x_axis.domain();
    /// ```
    pub fn axis_opt(&self, id: &AxisId) -> Option<&Axis<D, Theme>> {
        self.axes.get(id)
    }

    /// Returns a reference to an axis by ID.
    ///
    /// Panics if the axis doesn't exist.
    pub fn axis(&self, id: &AxisId) -> &Axis<D, Theme> {
        self.axes.get(id).unwrap()
    }

    /// Returns a mutable reference to an axis by ID.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use iced_aksel::{State, Axis, axis, scale::Linear};
    /// # let mut state: State<&'static str, f64> = State::new();
    /// # state.set_axis("x", Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom));
    /// state.axis_mut(&"x").pan(0.1);
    /// ```
    pub fn axis_mut_opt(&mut self, id: &AxisId) -> Option<&mut Axis<D, Theme>> {
        self.axes.get_mut(id)
    }

    /// Returns a mutable reference to an axis by ID.
    ///
    /// Panics if the axis doesn't exist.
    pub fn axis_mut(&mut self, id: &AxisId) -> &mut Axis<D, Theme> {
        self.axes.get_mut(id).unwrap()
    }

    /// Returns an iterator over all axes mutably.
    ///
    /// Useful when you need to modify multiple axes simultaneously.
    pub fn axes_iter_mut(&mut self) -> IterMut<'_, AxisId, Axis<D, Theme>> {
        self.axes.iter_mut()
    }

    // -------------------------------------------------------------------------
    // C. Coordinated Logic Helpers (The "Controller")
    // -------------------------------------------------------------------------

    /// Returns the current domain (min, max) of a specific axis.
    pub fn domain(&self, id: &AxisId) -> Option<(&D, &D)> {
        self.axes.get(id).map(|a| a.domain())
    }

    /// Returns a mutable reference to the internal axis map.
    pub const fn axes_mut(&mut self) -> &mut IndexMap<AxisId, Axis<D, Theme>> {
        &mut self.axes
    }

    /// Removes all axes except those in the provided list.
    pub fn retain_axes(&mut self, active_axes: &[AxisId]) {
        self.axes.retain(|k, _| active_axes.contains(k));
    }

    /// Returns an iterator over visible axes only.
    pub fn visible_axes(&self) -> impl Iterator<Item = (&AxisId, &Axis<D, Theme>)> {
        self.axes.iter().filter(|(_, axis)| axis.is_visible())
    }

    /// Pans both X and Y axes simultaneously.
    ///
    /// The deltas are normalized (0.0-1.0) values relative to each axis's domain.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use iced_aksel::{State, Axis, axis, scale::Linear};
    /// # let mut state: State<&'static str, f64> = State::new();
    /// # state.set_axis("x", Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom));
    /// # state.set_axis("y", Axis::new(Linear::new(0.0, 100.0), axis::Position::Left));
    /// // Pan 10% right and 5% up
    /// state.pan_axes("x", "y", 0.1, 0.05);
    /// ```
    pub fn pan_axes(
        &mut self,
        x_scale: AxisId,
        y_scale: AxisId,
        normalized_delta_x: f32,
        normalized_delta_y: f32,
    ) {
        if let Some(axis) = self.axes.get_mut(&x_scale) {
            axis.pan(normalized_delta_x);
        }
        if let Some(axis) = self.axes.get_mut(&y_scale) {
            axis.pan(normalized_delta_y);
        }
    }

    /// Zooms both X and Y axes simultaneously around the given anchor points.
    ///
    /// Anchor points are normalized (0.0-1.0). Factor > 1.0 zooms in, < 1.0 zooms out.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use iced_aksel::{State, Axis, axis, scale::Linear};
    /// # let mut state: State<&'static str, f64> = State::new();
    /// # state.set_axis("x", Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom));
    /// # state.set_axis("y", Axis::new(Linear::new(0.0, 100.0), axis::Position::Left));
    /// // Zoom in 2x around the center
    /// state.zoom_axes("x", "y", 0.5, 0.5, 2.0);
    /// ```
    pub fn zoom_axes(
        &mut self,
        x_scale: AxisId,
        y_scale: AxisId,
        x_anchor_norm: f32,
        y_anchor_norm: f32,
        factor: f32,
    ) {
        if let Some(axis) = self.axes.get_mut(&x_scale) {
            axis.zoom(factor, Some(x_anchor_norm));
        }
        if let Some(axis) = self.axes.get_mut(&y_scale) {
            axis.zoom(factor, Some(y_anchor_norm));
        }
    }

    /// Sets the domain (min, max) of an axis.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use iced_aksel::{State, Axis, axis, scale::Linear};
    /// # let mut state: State<&'static str, f64> = State::new();
    /// # state.set_axis("x", Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom));
    /// state.set_domain(&"x", 0.0, 200.0);
    /// ```
    pub fn set_domain(&mut self, id: &AxisId, min: D, max: D) {
        if let Some(axis) = self.axes.get_mut(id) {
            axis.set_domain(min, max);
        }
    }
}
