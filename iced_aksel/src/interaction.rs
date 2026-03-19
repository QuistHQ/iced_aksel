use std::hash::Hash;

use aksel::{Float, Transform};
use derivative::Derivative;
use iced_core::{Point, Rectangle, keyboard, mouse};
use indexmap::IndexMap;
use rapidhash::fast::RandomState;

use crate::event::{self, PressEvent, ReleaseEvent};

pub mod area;
mod id;
mod math;
mod query;

pub use area::Area;
pub use id::Id;
pub use query::InteractionQuery;

use area::ResolvedArea;

type HoverHandler<Message, Tag> = event::Handler<Message, (Id<Tag>, keyboard::Modifiers)>;
type DragHandler<Message, Tag> = event::Handler<Message, (Id<Tag>, event::DragEvent<event::Delta>)>;
type PressHandler<Message, Tag> = event::Handler<Message, (Id<Tag>, PressEvent<Point>)>;
type ReleaseHandler<Message, Tag> = event::Handler<Message, (Id<Tag>, ReleaseEvent<Point>)>;
type CursorHandler = event::Handler<mouse::Interaction, (InteractionStatus,)>;

pub struct Interaction<D, Message: Clone, Tag: Hash + Eq + Clone = ()> {
    pub(crate) id: Id<Tag>,
    pub(crate) priority: u16,
    pub(crate) area: Area<D>,
    pub(crate) cursor_handler: Option<CursorHandler>,
    pub(crate) on_hover: Option<HoverHandler<Message, Tag>>,
    pub(crate) on_drag: Option<DragHandler<Message, Tag>>,
    pub(crate) on_press: Option<PressHandler<Message, Tag>>,
    pub(crate) on_release: Option<ReleaseHandler<Message, Tag>>,
}

impl<D: Float, Message: Clone, T: Hash + Eq + Clone> Interaction<D, Message, T> {
    pub(crate) fn resolve<R: iced_core::text::Renderer<Font = iced_core::Font>>(
        self,
        transform: &Transform<D, f32, f32>,
        renderer: &R,
    ) -> (Id<T>, ResolvedInteraction<Message, T>) {
        let Self {
            id,
            priority,
            area,
            cursor_handler,
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
                priority,
                area,
                bounding_box,
                cursor_handler,
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
            priority: u16::MAX,
            area,
            cursor_handler: None,
            on_hover: None,
            on_drag: None,
            on_press: None,
            on_release: None,
        }
    }

    /// Sets the priority of the interaction.
    ///
    /// 0 = highest priority.
    /// 255 = lowest priority.
    ///
    /// Defaults to 255.
    pub const fn priority(mut self, prio: u16) -> Self {
        self.priority = prio;
        self
    }

    /// Sets a dynamic cursor for this interaction based on its current status.
    pub fn cursor<F, Mk>(mut self, f: F) -> Self
    where
        F: crate::event::IntoHandler<mouse::Interaction, (InteractionStatus,), Mk>,
    {
        self.cursor_handler = Some(f.into_handler());
        self
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
pub(crate) struct ResolvedInteraction<Message: Clone, Tag: Hash + Eq + Clone> {
    pub priority: u16,
    pub area: ResolvedArea,
    pub bounding_box: Rectangle,

    #[derivative(Debug = "ignore")]
    pub cursor_handler: Option<CursorHandler>,
    #[derivative(Debug = "ignore")]
    pub on_hover: Option<HoverHandler<Message, Tag>>,
    #[derivative(Debug = "ignore")]
    pub on_drag: Option<DragHandler<Message, Tag>>,
    #[derivative(Debug = "ignore")]
    pub on_press: Option<PressHandler<Message, Tag>>,
    #[derivative(Debug = "ignore")]
    pub on_release: Option<ReleaseHandler<Message, Tag>>,
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
    pub(crate) fn query(
        &self,
        query: &InteractionQuery,
    ) -> impl Iterator<Item = (&Id<T>, &ResolvedInteraction<Message, T>)> {
        let query_bounds = query.bounds();

        self.0.iter().rev().filter(move |(_, interaction)| {
            math::rect_intersects_rect(&interaction.area.bounding_box(), &query_bounds)
                && interaction.area.intersects(query)
        })
    }

    pub(crate) fn query_filtered<P>(
        &self,
        query: &InteractionQuery,
        predicate: P,
    ) -> impl Iterator<Item = (&Id<T>, &ResolvedInteraction<Message, T>)>
    where
        P: Fn(&ResolvedInteraction<Message, T>) -> bool,
    {
        self.query(query)
            .filter(move |(_, interaction)| predicate(interaction))
    }

    /// Queries the cache for the interaction that intersect the given query and has the highest
    /// priority.
    pub(crate) fn query_prioritized<P>(
        &self,
        query: &InteractionQuery,
        predicate: P,
    ) -> Option<(Id<T>, &ResolvedInteraction<Message, T>)>
    where
        P: Fn(&ResolvedInteraction<Message, T>) -> bool,
    {
        let mut current = None;
        let mut highest_priority_seen = None;

        self.query_filtered(query, predicate)
            .for_each(|(id, interaction)| {
                if highest_priority_seen.is_none_or(|p| p > interaction.priority) {
                    current = Some((id.clone(), interaction));
                    highest_priority_seen = Some(interaction.priority);
                }
            });

        current
    }
}

impl<Message: Clone, T: Hash + Eq + Clone> Default for InteractionsCache<Message, T> {
    fn default() -> Self {
        Self::new()
    }
}

/// The current state of an interaction, used to determine dynamic styling like cursors.
#[derive(Debug, Clone, Copy)]
pub struct InteractionStatus {
    /// Whether the mouse is currently hovering over this specific interaction.
    pub is_hovered: bool,
    /// Whether the mouse button is currently pressed down on this interaction.
    pub is_pressed: bool,
    /// Whether the interaction is currently being dragged (surpassed the drag deadband).
    pub is_dragging: bool,

    /// The button held. Only present if dragging or pressed.
    pub button_held: Option<mouse::Button>,
    /// The kind of click used to start dragging or pressing.
    pub click_kind: Option<mouse::click::Kind>,
    /// The current state of keyboard modifiers (Shift, Control, Alt, etc.).
    pub modifiers: iced_core::keyboard::Modifiers,
}
