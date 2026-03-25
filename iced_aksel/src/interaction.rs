//! Interaction module
//!
//! Describes plot interaction structures and methods

use std::hash::Hash;

use derivative::Derivative;
use iced_core::{Point, Rectangle, mouse};
use indexmap::IndexMap;
use rapidhash::fast::RandomState;

use crate::event::{self, PressEvent, ReleaseEvent};

mod area;
mod id;
mod math;
mod query;

pub use area::{Area, IntoArea};
pub use id::Id;
pub use query::InteractionQuery;

type EnterHandler<Message, Tag> = event::Handler<Message, (Id<Tag>, event::EnterEvent)>;
type ExitHandler<Message, Tag> = event::Handler<Message, (Id<Tag>, event::ExitEvent)>;
type DragHandler<Message, Tag> = event::Handler<Message, (Id<Tag>, event::DragEvent<event::Delta>)>;
type PressHandler<Message, Tag> = event::Handler<Message, (Id<Tag>, PressEvent<Point>)>;
type ReleaseHandler<Message, Tag> = event::Handler<Message, (Id<Tag>, ReleaseEvent<Point>)>;
type CursorHandler = event::Handler<mouse::Interaction, (InteractionStatus,)>;

/// An interactable plot-area
///
/// # Examples
///
/// ```no_run
/// use iced_aksel::{Interaction, PlotPoint, Measure};
/// use iced_aksel::shape::Rectangle;
/// use iced_aksel::interaction::IntoArea;
///
/// # fn example(plot: &mut iced_aksel::plot::Plot<f64, Message>) {
/// #[derive(Debug, Clone)]
/// enum Message {
///     RectPressed,
/// }
///
/// // Create a shape
/// let shape = Rectangle::centered(
///     PlotPoint::new(50.0, 50.0),
///     Measure::Plot(20.0),
///     Measure::Plot(10.0),
/// );
///
/// // Convert shape to area and create interaction
/// let area = shape.resolve_area(plot);
/// let interaction = Interaction::new(area)
///     .on_press(Message::RectPressed);
///
/// plot.render(shape);
/// # }
/// ```
pub struct Interaction<Message: Clone, Tag = ()> {
    pub(crate) priority: u16,
    pub(crate) area: Area,
    pub(crate) cursor_handler: Option<CursorHandler>,
    pub(crate) on_enter: Option<EnterHandler<Message, Tag>>,
    pub(crate) on_exit: Option<ExitHandler<Message, Tag>>,
    pub(crate) on_drag: Option<DragHandler<Message, Tag>>,
    pub(crate) on_press: Option<PressHandler<Message, Tag>>,
    pub(crate) on_release: Option<ReleaseHandler<Message, Tag>>,
}

impl<Message: Clone, Tag: Hash + Eq + Clone> Interaction<Message, Tag> {
    /// Creates a new [`Interaction`]
    pub const fn new(area: Area) -> Self {
        Self {
            priority: u16::MAX,
            area,
            cursor_handler: None,
            on_enter: None,
            on_exit: None,
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
        /// Sets the event handler for when the cursor enters the interaction
        enter: (Id<Tag>, event::EnterEvent);

        /// Sets the event handler for when the cursor enters the interaction
        exit: (Id<Tag>, event::ExitEvent);

        /// Sets the event handler for interaction dragging
        drag: (Id<Tag>, event::DragEvent<event::Delta>);

        /// Sets the event handler for interaction mouse presses
        press: (Id<Tag>, PressEvent<Point>);

        /// Sets the event handler for interaction mouse releases
        release: (Id<Tag>, ReleaseEvent<Point>);
    );
}

/// A stored interaction waiting to be tested against mouse events.
#[derive(Derivative)]
#[derivative(Debug)]
pub(crate) struct ResolvedInteraction<Message: Clone, Tag> {
    pub priority: u16,
    pub area: Area,
    pub bounding_box: Rectangle,

    #[derivative(Debug = "ignore")]
    pub cursor_handler: Option<CursorHandler>,
    #[derivative(Debug = "ignore")]
    pub on_enter: Option<EnterHandler<Message, Tag>>,
    #[derivative(Debug = "ignore")]
    pub on_exit: Option<ExitHandler<Message, Tag>>,
    #[derivative(Debug = "ignore")]
    pub on_drag: Option<DragHandler<Message, Tag>>,
    #[derivative(Debug = "ignore")]
    pub on_press: Option<PressHandler<Message, Tag>>,
    #[derivative(Debug = "ignore")]
    pub on_release: Option<ReleaseHandler<Message, Tag>>,
}

impl<Message: Clone, Tag> ResolvedInteraction<Message, Tag> {
    pub fn new(interaction: Interaction<Message, Tag>) -> Option<Self> {
        let Interaction {
            priority,
            area,
            cursor_handler,
            on_enter,
            on_exit,
            on_drag,
            on_press,
            on_release,
            ..
        } = interaction;
        let bounding_box = area.bounding_box();
        Some(Self {
            priority,
            area,
            bounding_box,
            cursor_handler,
            on_enter,
            on_exit,
            on_drag,
            on_press,
            on_release,
        })
    }
}

/// The registry that collects hitboxes during the drawing phase.
#[derive(Debug)]
pub(crate) struct InteractionsCache<Message: Clone, Tag>(
    IndexMap<Id<Tag>, ResolvedInteraction<Message, Tag>, RandomState>,
);

impl<Message: Clone, Tag: Hash + Eq + Clone> InteractionsCache<Message, Tag> {
    pub(crate) fn new() -> Self {
        Self(IndexMap::with_hasher(RandomState::new()))
    }

    pub(crate) fn iter(
        &self,
    ) -> indexmap::map::Iter<'_, Id<Tag>, ResolvedInteraction<Message, Tag>> {
        self.0.iter()
    }

    pub(crate) fn get(&self, id: &Id<Tag>) -> Option<&ResolvedInteraction<Message, Tag>> {
        self.0.get(id)
    }

    /// Push an interaction to the cache
    pub(crate) fn insert(&mut self, id: Id<Tag>, interaction: ResolvedInteraction<Message, Tag>) {
        self.0.insert(id, interaction);
    }

    // Clear the inner vector, re-using the allocated space next time we push
    pub(crate) fn clear(&mut self) {
        self.0.clear();
    }

    /// Queries the cache for all interactions that intersect the given query.
    pub(crate) fn query(
        &self,
        query: &InteractionQuery,
    ) -> impl Iterator<Item = (&Id<Tag>, &ResolvedInteraction<Message, Tag>)> {
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
    ) -> impl Iterator<Item = (&Id<Tag>, &ResolvedInteraction<Message, Tag>)>
    where
        P: Fn(&ResolvedInteraction<Message, Tag>) -> bool,
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
    ) -> Option<(Id<Tag>, &ResolvedInteraction<Message, Tag>)>
    where
        P: Fn(&ResolvedInteraction<Message, Tag>) -> bool,
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

impl<Message: Clone, Tag: Hash + Eq + Clone> Default for InteractionsCache<Message, Tag> {
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
