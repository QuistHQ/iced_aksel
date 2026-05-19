use iced_core::{Pixels, Point, Rectangle};

/// Represents a spatial query in screen-space to test against interactions.
#[derive(Debug, Clone, Copy)]
pub enum InteractionQuery {
    /// A precise point check (e.g., hovering or clicking).
    /// `tolerance_px` expands the hit area to make thin lines/points clickable.
    Point {
        /// The point on screen
        position: Point,
        /// How much tolerance (padding) to add to the point
        tolerance: Pixels,
    },

    /// A bounding box check (e.g., marquee drag selection).
    Bounds(Rectangle),
}

impl InteractionQuery {
    /// Returns the broad-phase bounding box of the query itself.
    pub fn bounds(&self) -> Rectangle {
        match self {
            Self::Point {
                position,
                tolerance,
            } => Rectangle {
                x: position.x - tolerance.0,
                y: position.y - tolerance.0,
                width: tolerance.0 * 2.0,
                height: tolerance.0 * 2.0,
            },
            Self::Bounds(rect) => *rect,
        }
    }
}
