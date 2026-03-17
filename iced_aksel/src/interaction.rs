use std::hash::Hash;

use aksel::{Float, Transform};
use derivative::Derivative;
use iced_core::{Point, Rectangle, keyboard};
use indexmap::IndexMap;
use rapidhash::fast::RandomState;

use crate::event::{self, PressEvent, ReleaseEvent};

pub mod area;
mod id;
mod math;

pub use area::Area;
pub use id::Id;

use area::ResolvedArea;

type HoverHandler<Message, T = ()> = event::Handler<Message, (Id<T>, keyboard::Modifiers)>;
type DragHandler<Message, T = ()> =
    event::Handler<Message, (Id<T>, event::DragEvent<event::Delta>)>;
type PressHandler<Message, T = ()> = event::Handler<Message, (Id<T>, PressEvent<Point>)>;
type ReleaseHandler<Message, T = ()> = event::Handler<Message, (Id<T>, ReleaseEvent<Point>)>;

pub struct Interaction<D, Message: Clone, T: Hash + Eq + Clone = ()> {
    pub(crate) id: Id<T>,
    pub(crate) area: Area<D>,
    pub(crate) on_hover: Option<HoverHandler<Message, T>>,
    pub(crate) on_drag: Option<DragHandler<Message, T>>,
    pub(crate) on_press: Option<PressHandler<Message, T>>,
    pub(crate) on_release: Option<ReleaseHandler<Message, T>>,
}

impl<D: Float, Message: Clone, T: Hash + Eq + Clone> Interaction<D, Message, T> {
    pub(crate) fn resolve<R: iced_core::text::Renderer<Font = iced_core::Font>>(
        self,
        transform: &Transform<D, f32, f32>,
        renderer: &R,
    ) -> (Id<T>, ResolvedInteraction<Message, T>) {
        let Self {
            id,
            area,
            on_hover,
            on_drag,
            on_press,
            on_release,
        } = self;

        let area = area.resolve(transform, renderer);
        let bounding_box = area.bounding_box();

        (
            id,
            ResolvedInteraction {
                area,
                bounding_box,
                on_hover,
                on_drag,
                on_press,
                on_release,
            },
        )
    }

    pub fn new(id: impl Into<Id<T>>, area: impl Into<Area<D>>) -> Self {
        let id = id.into();
        let area = area.into();
        Self {
            id,
            area,
            on_hover: None,
            on_drag: None,
            on_press: None,
            on_release: None,
        }
    }

    event::impl_handlers!(
        /// Sets the event handler for interaction hovering
        hover: (Id<T>, keyboard::Modifiers);

        /// Sets the event handler for interaction dragging
        drag: (Id<T>, event::DragEvent<event::Delta>);

        /// Sets the event handler for interaction mouse presses
        press: (Id<T>, PressEvent<Point>);

        /// Sets the event handler for interaction mouse releases
        release: (Id<T>, ReleaseEvent<Point>);
    );
}

/// A stored interaction waiting to be tested against mouse events.
#[derive(Derivative)]
#[derivative(Debug)]
pub(crate) struct ResolvedInteraction<Message: Clone, T: Hash + Eq + Clone = ()> {
    pub area: ResolvedArea,
    pub bounding_box: Rectangle,

    #[derivative(Debug = "ignore")]
    pub on_hover: Option<HoverHandler<Message, T>>,
    #[derivative(Debug = "ignore")]
    pub on_drag: Option<DragHandler<Message, T>>,
    #[derivative(Debug = "ignore")]
    pub on_press: Option<PressHandler<Message, T>>,
    #[derivative(Debug = "ignore")]
    pub on_release: Option<ReleaseHandler<Message, T>>,
}

/// The registry that collects hitboxes during the drawing phase.
#[derive(Debug)]
pub struct InteractionsCache<Message: Clone, Tag: Hash + Eq + Clone>(
    IndexMap<Id<Tag>, ResolvedInteraction<Message, Tag>, RandomState>,
);

impl<Message: Clone, T: Hash + Eq + Clone> InteractionsCache<Message, T> {
    pub fn new() -> Self {
        Self(IndexMap::with_hasher(RandomState::new()))
    }

    pub(crate) fn iter(&self) -> indexmap::map::Iter<'_, Id<T>, ResolvedInteraction<Message, T>> {
        self.0.iter()
    }

    pub(crate) fn get(&self, id: &Id<T>) -> Option<&ResolvedInteraction<Message, T>> {
        self.0.get(id)
    }

    /// Push an interaction to the cache
    pub(crate) fn insert(&mut self, id: Id<T>, interaction: ResolvedInteraction<Message, T>) {
        self.0.insert(id, interaction);
    }

    // Clear the inner vector, re-using the allocated space next time we push
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Queries the cache for all interactions that intersect the given query.
    pub fn query(
        &self,
        query: &InteractionQuery,
    ) -> Vec<(&Id<T>, &ResolvedInteraction<Message, T>)> {
        let mut hits = Vec::new();
        let query_bounds = query.bounds();

        for (id, interaction) in self.0.iter() {
            if math::rect_intersects_rect(&interaction.area.bounding_box(), &query_bounds) {
                if interaction.area.intersects(query) {
                    hits.push((id, interaction));
                }
            }
        }

        hits
    }
}

impl<Message: Clone, T: Hash + Eq + Clone> Default for InteractionsCache<Message, T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a spatial query in screen-space to test against interactions.
#[derive(Debug, Clone, Copy)]
pub enum InteractionQuery {
    /// A precise point check (e.g., hovering or clicking).
    /// `tolerance_px` expands the hit area to make thin lines/points clickable.
    Point { position: Point, tolerance_px: f32 },

    /// A bounding box check (e.g., marquee drag selection).
    Bounds(Rectangle),
}

impl InteractionQuery {
    /// Returns the broad-phase bounding box of the query itself.
    pub(crate) fn bounds(&self) -> Rectangle {
        match self {
            Self::Point {
                position,
                tolerance_px,
            } => Rectangle {
                x: position.x - tolerance_px,
                y: position.y - tolerance_px,
                width: tolerance_px * 2.0,
                height: tolerance_px * 2.0,
            },
            Self::Bounds(rect) => *rect,
        }
    }
}
