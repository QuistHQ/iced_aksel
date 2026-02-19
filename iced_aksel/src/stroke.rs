//! Types for describing Stroke around a rendered object

use crate::Measure;

use aksel::{Float, Transform};
use iced_core::Color;

/// Defines the visual style of a stroke.
///
/// These styles affect how lines are rendered. The stroke width influences
/// the appearance of each style.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StrokeStyle {
    /// Solid continuous line with no gaps.
    ///
    /// # Example
    ///
    /// ```rust
    /// use iced_aksel::stroke::StrokeStyle;
    /// let style = StrokeStyle::Solid;
    /// ```
    Solid,

    /// Dashed line with alternating segments and gaps.
    ///
    /// # Example
    ///
    /// ```rust
    /// use iced_aksel::stroke::StrokeStyle;
    /// let style = StrokeStyle::Dashed {
    ///     dash: 5.0, // 5px dashes
    ///     gap: 5.0, // 5px gaps
    /// };
    /// ```
    Dashed {
        /// Length of the dashes **in screen space-pixels**
        dash: f32, // TODO: Convert to Measure!
        /// Length of the gaps in-between **in screen-space pixels**
        gap: f32, // TODO: Convert to Measure!
    },

    /// Dotted line with small circular dots.
    ///
    /// # Example
    ///
    /// ```rust
    /// use iced_aksel::stroke::StrokeStyle;
    /// let style = StrokeStyle::Dotted {
    ///    gap: 5.0, // 5px gaps
    /// };
    /// ```
    Dotted {
        /// Length of the gap between dots **in screen-space pixels**
        gap: f32, // TODO: Convert to Measure!
    },
}

/// A stroke configuration with color, thickness, and style.
///
/// Strokes are used to outline shapes like lines, polylines, circles, and polygons.
///
/// # Example
///
/// ```rust
/// use iced_aksel::{Stroke, Measure, stroke::StrokeStyle};
/// use iced::Color;
///
/// // Simple solid stroke
/// let stroke: Stroke<f64> = Stroke::new(Color::from_rgb(1.0, 0.0, 0.0), Measure::Screen(2.0));
///
/// // Dashed stroke
/// let dashed: Stroke<f64> = Stroke::with_style(
///     Color::from_rgb(0.0, 0.0, 1.0),
///     Measure::Screen(3.0),
///     StrokeStyle::Dashed {
///         dash: 5.0,
///         gap: 5.0,
///     }
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stroke<D> {
    /// The color of the stroke.
    pub fill: Color,
    /// The thickness of the stroke.
    ///
    /// Using Measure::Plot here will use geometric mean to scale the thickness.
    /// This is mostly only meant for charts that use *uniform* scaling of it's axes, as it might
    /// scale weirdly if only one axis is scaled.
    pub thickness: Measure<D>,
    /// The visual style of the stroke.
    pub style: StrokeStyle,
}

impl<D: Float> Stroke<D> {
    /// Creates a new solid stroke with the given color and thickness.
    ///
    /// # Example
    ///
    /// ```rust
    /// use iced_aksel::{Stroke, Measure};
    /// use iced::Color;
    ///
    /// let stroke: Stroke<f64> = Stroke::new(Color::from_rgb(1.0, 0.0, 0.0), Measure::Screen(2.0));
    /// ```
    pub const fn new(fill: Color, thickness: Measure<D>) -> Self {
        Self {
            fill,
            thickness,
            style: StrokeStyle::Solid,
        }
    }

    /// Creates a new stroke with the given color, thickness, and style.
    ///
    /// # Example
    ///
    /// ```rust
    /// use iced_aksel::{Stroke, Measure, stroke::StrokeStyle};
    /// use iced::Color;
    ///
    /// let stroke: Stroke<f64> = Stroke::with_style(
    ///     Color::from_rgb(0.0, 1.0, 0.0),
    ///     Measure::Plot(1.0),
    ///     StrokeStyle::Dashed {
    ///         dash: 5.0,
    ///         gap: 5.0,
    ///     }
    /// );
    /// ```
    pub const fn with_style(fill: Color, thickness: Measure<D>, style: StrokeStyle) -> Self {
        Self {
            fill,
            thickness,
            style,
        }
    }

    ///  Resolves the Stroke into screen-pixels using geometric mean, if the thickness is described
    ///  in plot coordinates. Directly translates the width if it's described in pixels.
    pub fn resolve(self, transform: &Transform<D, f32, f32>) -> ResolvedStroke {
        let Self {
            fill,
            thickness,
            style,
        } = self;

        let plot_bounds = transform.plot_bounds();
        let screen_bounds = transform.screen_bounds();

        let thickness = match thickness {
            Measure::Screen(t) => t,
            Measure::Plot(p) => {
                let x_span = (plot_bounds.max_x() - plot_bounds.min_x())
                    .to_f32()
                    .expect("plot to pixel mapping");
                let y_span = (plot_bounds.max_y() - plot_bounds.min_y())
                    .to_f32()
                    .expect("plot to pixel mapping");

                let sx = screen_bounds.width / x_span;
                let sy = screen_bounds.height / y_span;

                // Guard against infinite thickness
                if sx.is_infinite() || sy.is_infinite() {
                    return ResolvedStroke {
                        fill,
                        thickness: 0.0,
                        style,
                    };
                }

                let stroke_scale = (sx.abs() * sy.abs()).sqrt();
                p.to_f32().expect("plot to pixel mapping") * stroke_scale
            }
        };

        ResolvedStroke {
            fill,
            thickness,
            style,
        }
    }
}

/// A stroke with all measurements resolved to screen-space pixels.
///
/// Produced by converting a [`Stroke`] through the current plot transform.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResolvedStroke {
    /// The stroke color.
    pub fill: Color,
    /// Stroke width in screen-space pixels.
    pub thickness: f32,
    /// The dash/dot pattern style of the stroke.
    pub style: StrokeStyle,
}
