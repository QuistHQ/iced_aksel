use iced_core::{Point, Rectangle};

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
