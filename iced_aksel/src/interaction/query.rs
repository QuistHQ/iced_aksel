use crate::IntoHandler;

use super::{Id, ResolvedInteraction};
use iced_core::{Point, Rectangle};
use std::hash::Hash;

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

type QueryHandler<Message, R> = crate::event::Handler<Message, (R,)>;
type QueryFilter<Message, Tag> = Box<dyn Fn(Id<Tag>, ResolvedInteraction<Message, Tag>) -> bool>;

pub enum QueryMode<Message, Tag: Hash + Eq + Clone> {
    Closest(QueryHandler<Message, Id<Tag>>),
    All(QueryHandler<Message, Vec<Id<Tag>>>),
    Any(QueryHandler<Message, Id<Tag>>),
}

pub struct QueryBuilder {
    area: Rectangle,
}

impl QueryBuilder {
    pub fn mode<Message: Clone, Tag: Hash + Eq + Clone>(
        self,
        mode: QueryMode<Message, Tag>,
    ) -> Query<Message, Tag> {
        let Self { area } = self;
        Query {
            filter: None,
            prioritized: false,
            mode,
            area,
        }
    }

    pub fn closest<Message: Clone, Tag: Hash + Eq + Clone, F>(self, f: F) -> Query<Message, Tag>
    where
        F: IntoHandler<Message, (Id<Tag>,)>,
    {
        let Self { area } = self;
        Query {
            filter: None,
            mode: QueryMode::Closest(f.into_handler()),
            prioritized: false,
            area,
        }
    }
}

pub struct Query<Message: Clone, Tag: Hash + Eq + Clone> {
    filter: Option<QueryFilter<Message, Tag>>,
    mode: QueryMode<Message, Tag>,
    prioritized: bool,
    area: Rectangle,
}

impl<Message: Clone, Tag: Hash + Eq + Clone> Query<Message, Tag> {
    pub fn point(point: Point, tolerance_px: f32) -> QueryBuilder {
        let area = Rectangle {
            x: point.x - tolerance_px,
            y: point.y - tolerance_px,
            width: tolerance_px * 2.0,
            height: tolerance_px * 2.0,
        };
        QueryBuilder { area }
    }

    pub const fn rect(rect: Rectangle) -> QueryBuilder {
        QueryBuilder { area: rect }
    }
}
