use crate::{radii::ResolvedRadii, stroke::ResolvedStroke};
use iced_core::{
    Color, Font, Pixels, Point, Radians, Rectangle, Size, Vector,
    alignment::{Horizontal, Vertical},
    text::{LineHeight, Wrapping},
};
use iced_graphics::geometry::{self, path::arc::Elliptical};
use iced_graphics::geometry::{
    Path,
    path::{self, Builder},
};
use std::f32::consts::PI;

/// Controls whether a line is extended to the edges of the plot bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineExtensions {
    /// Extend the line infinitely beyond its start point.
    pub start: bool,
    /// Extend the line infinitely beyond its end point.
    pub end: bool,
}

/// Controls arrowhead rendering at the ends of a line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineArrows {
    /// Draw an arrowhead at the start of the line.
    pub start: bool,
    /// Draw an arrowhead at the end of the line.
    pub end: bool,
    /// Arrowhead size.
    pub size: f32,
}

// Describes a **shared** primitive interface between the Mesh and Path backends.
pub enum Primitive {
    Rectangle {
        xy1: Point,
        xy2: Point,
        fill: Option<Color>,
        stroke: Option<ResolvedStroke>,
    },
    Ellipse {
        center: Point,
        radii: ResolvedRadii,
        fill: Option<Color>,
        stroke: Option<ResolvedStroke>,
    },
    Triangle {
        points: [Point; 3],
        fill: Option<Color>,
        stroke: Option<ResolvedStroke>,
    },
    Polygon {
        center: Point,
        radius: f32,
        vertices: u16,
        rotation: f32,
        fill: Option<Color>,
        stroke: Option<ResolvedStroke>,
    },
    Line {
        start: Point,
        end: Point,
        stroke: ResolvedStroke,
        clip_bounds: Rectangle,
        extensions: LineExtensions,
        arrows: LineArrows,
    },
    HorizontalLine {
        y: f32,
        x_start: f32,
        x_end: f32,
        stroke: ResolvedStroke,
        snap: bool,
    },
    VerticalLine {
        x: f32,
        y_start: f32,
        y_end: f32,
        stroke: ResolvedStroke,
        snap: bool,
    },
    PolyLine {
        points: Vec<Point>,
        stroke: ResolvedStroke,
        clip_bounds: Rectangle,
        extensions: LineExtensions,
        arrows: LineArrows,
    },
    BezierCurve {
        start: Point,
        end: Point,
        control_1: Point,
        control_2: Option<Point>,
        stroke: ResolvedStroke,
    },
    Spline {
        points: Vec<Point>,
        stroke: ResolvedStroke,
        tension: f32,
    },
    Arc {
        center: Point,
        radius_inner: f32,
        radius_outer: f32,
        start_angle: f32,
        end_angle: f32,
        fill: Option<Color>,
        stroke: Option<ResolvedStroke>,
    },
    Area {
        points: Vec<Point>,
        fill: Option<Color>,
        stroke: Option<ResolvedStroke>,
    },
    Text {
        font: Font,
        content: String,
        position: Point,
        size: Pixels,
        rotation: f32,
        horizontal_alignment: Horizontal,
        vertical_alignment: Vertical,
        fill: Color,
        /// Override the quality tolerance of the text
        quality: Option<f32>,
        line_height: LineHeight,
        bounds: Size,
        wrapping: Wrapping,
    },
}
