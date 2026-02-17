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

pub(crate) static NEXT_LAYER_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug)]
pub struct Cached<T> {
    data: T,
    version: u64,
    uid: u64,
}

impl<T> Cached<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            version: 1,
            uid: NEXT_LAYER_ID.fetch_add(1, Ordering::Relaxed),
        }
    }

    pub const fn edit(&mut self) -> &mut T {
        self.version = self.version.wrapping_add(1);
        &mut self.data
    }

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

    fn version(&self) -> u64 {
        self.version
    }

    fn id(&self) -> Option<u64> {
        Some(self.uid)
    }
}
