use std::{
    hash::Hash,
    sync::atomic::{AtomicU64, Ordering},
};

use aksel::Float;
use derivative::Derivative;

use crate::PlotData;

use super::plot;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Layer<'a, AxisId, Domain, Renderer, Theme> {
    pub(crate) horizontal_axis_id: AxisId,
    pub(crate) vertical_axis_id: AxisId,

    #[derivative(Debug = "ignore")]
    pub(crate) items: &'a dyn plot::PlotData<Domain, Renderer, Theme>,
}

impl<'a, AxisId: Hash + Eq, D: Float, R: crate::Renderer, Theme> Layer<'a, AxisId, D, R, Theme> {
    pub const fn new<T: plot::PlotData<D, R, Theme>>(
        items: &'a T,
        horizontal_axis_id: AxisId,
        vertical_axis_id: AxisId,
    ) -> Self {
        Self {
            horizontal_axis_id,
            vertical_axis_id,
            items,
        }
    }
}

static NEXT_LAYER_ID: AtomicU64 = AtomicU64::new(1);

/// An ID for a layer
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LayerId(u64);

impl Default for LayerId {
    fn default() -> Self {
        Self(NEXT_LAYER_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl LayerId {
    /// Creates a new layer id.
    ///
    /// This ensures no collision will occur, since we increment the underlying u64 atomically each
    /// time we create a new id.
    pub fn new() -> Self {
        Self::default()
    }
}

/// A versioned wrapper that caches [`PlotData`](crate::PlotData) across frames.
///
/// Wrapping your data in `Cached` allows the renderer to skip re-drawing when
/// the data has not changed since the last frame. The cache is invalidated only
/// when you call [`edit`](Self::edit), which bumps an internal version counter.
///
/// # Example
///
/// ```rust,no_run
/// use iced_aksel::Cached;
///
/// let mut data = Cached::new(vec![1.0_f64, 2.0, 3.0]);
///
/// // Reading does not invalidate the cache — version stays the same.
/// let _ = data.get();
///
/// // Editing returns a mutable reference and bumps the internal version,
/// // signalling to the renderer that this layer must be redrawn.
/// data.edit().push(4.0);
/// assert_eq!(data.get().len(), 4);
/// ```
///
/// See [`PlotData`](crate::PlotData) for details on how versioning integrates
/// with the rendering pipeline.
#[derive(Debug)]
pub struct Cached<T> {
    data: T,
    version: u64,
    uid: LayerId,
}

impl<T> Cached<T> {
    /// Creates a new `Cached` wrapper around the given data.
    pub fn new(data: T) -> Self {
        Self {
            data,
            version: 1,
            uid: LayerId::new(),
        }
    }

    /// Returns a mutable reference to the inner data and increments the version.
    ///
    /// The bumped version signals to the renderer that the layer must be redrawn
    /// on the next frame.
    pub const fn edit(&mut self) -> &mut T {
        self.version = self.version.wrapping_add(1);
        &mut self.data
    }

    /// Returns a shared reference to the inner data without incrementing the version.
    pub const fn get(&self) -> &T {
        &self.data
    }
}

impl<D: Float, Renderer: crate::Renderer, T: PlotData<D, Renderer>> PlotData<D, Renderer>
    for Cached<T>
{
    fn draw(&self, plot: &mut plot::Plot<D, Renderer>, theme: &iced_core::Theme) {
        self.data.draw(plot, theme);
    }

    fn version(&self) -> Option<u64> {
        Some(self.version)
    }

    fn id(&self) -> Option<LayerId> {
        Some(self.uid)
    }
}
