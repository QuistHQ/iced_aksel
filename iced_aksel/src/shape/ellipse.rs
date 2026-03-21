use crate::{
    Measure, Radii, Shape, Stroke,
    interaction::{Area, IntoArea},
    plot,
    radii::ResolvedRadii,
    render::Primitive,
};
use aksel::{Float, PlotPoint};
use iced_core::{Color, Point};

/// A primitive representing an ellipse or circle.
///
/// This shape is defined by a center point and two radii (X and Y).
/// It allows for creating perfect circles (where radii are equal) or stretched ellipses.
///
/// # Usage
///
/// ## 1. Perfect Circle
/// ```rust
/// use iced_aksel::shape::Ellipse;
/// use iced_aksel::{Stroke, Measure};
/// use iced_core::Color;
/// use aksel::PlotPoint;
///
/// let circle = Ellipse::circle(
///     PlotPoint::new(0.0, 0.0),
///     Measure::Screen(10.0)
/// )
/// .fill(Color::WHITE);
/// ```
///
/// ## 2. Stretched Ellipse
/// ```rust
/// use iced_aksel::shape::Ellipse;
/// use iced_aksel::{Stroke, Measure};
/// use iced::Color;
/// use aksel::PlotPoint;
///
/// let oval = Ellipse::new(
///     PlotPoint::new(0.0, 0.0),
///     Measure::Screen(20.0), // Radius X
///     Measure::Screen(10.0)  // Radius Y
/// )
/// .stroke(Stroke::new(Color::BLACK, Measure::Screen(2.0)));
/// ```
#[derive(Debug, Clone)]
pub struct Ellipse<D> {
    /// The center point of the ellipse
    pub center: PlotPoint<D>,
    /// The vertical radius
    pub radii: Radii<Measure<D>>,
    /// The fill color for the ellipse interior
    pub fill: Option<Color>,
    /// The stroke style for the ellipse border
    pub stroke: Option<Stroke<D>>,
}

impl<D: Float, R: crate::Renderer> Shape<D, R> for Ellipse<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        let Self {
            center,
            radii,
            fill,
            stroke,
        } = self;

        let center = Point::new(ctx.x_to_screen(&center.x), ctx.y_to_screen(&center.y));
        let stroke = stroke.map(|s| s.resolve(ctx));

        // If the radii can't be resolved, we don't render anything
        let Some(radii) = radii.resolve(ctx) else {
            return;
        };

        ctx.add_primitive(Primitive::Ellipse {
            center,
            radii,
            fill,
            stroke,
        });
    }
}

impl<D: Float> Ellipse<D> {
    /// Creates a new `Ellipse` defined by a center and separate X and Y radii.
    ///
    /// Note: The shape is invisible by default. You must call `.fill()` or `.stroke()` to render it.
    pub const fn new(center: PlotPoint<D>, radii: Radii<Measure<D>>) -> Self {
        Self {
            center,
            radii,
            fill: None,
            stroke: None,
        }
    }

    /// Creates a perfect `Circle` (an Ellipse where radius X equals radius Y).
    ///
    /// Note: The shape is invisible by default. You must call `.fill()` or `.stroke()` to render it.
    pub const fn circle(center: PlotPoint<D>, radius: Measure<D>) -> Self {
        Self {
            center,
            radii: Radii::uniform(radius),
            fill: None,
            stroke: None,
        }
    }

    /// Sets the fill color.
    #[inline]
    pub const fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }

    /// Sets the stroke style.
    #[inline]
    pub const fn stroke(mut self, stroke: Stroke<D>) -> Self {
        self.stroke = Some(stroke);
        self
    }
}

impl<'a, D: Float, Renderer: crate::Renderer> IntoArea<'a, D, Renderer> for &Ellipse<D> {
    fn resolve_area(self, ctx: &plot::Context<'a, D, Renderer>) -> Area {
        let center = ctx.chart_to_screen(&self.center);
        Area::Ellipse {
            center: Point::new(center.x, center.y),
            radii: self.radii.resolve(ctx).unwrap_or(ResolvedRadii::ZERO),
        }
    }
}
