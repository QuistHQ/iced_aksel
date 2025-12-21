//! Types for describing Stroke around a rendered object

use crate::Measure;

use iced_core::Color;

/// Defines the visual style of a stroke.
///
/// These styles affect how lines are rendered. The stroke width influences
/// the appearance of each style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    /// let style = StrokeStyle::Dashed;
    /// ```
    Dashed,

    /// Dotted line with small circular dots.
    ///
    /// # Example
    ///
    /// ```rust
    /// use iced_aksel::stroke::StrokeStyle;
    /// let style = StrokeStyle::Dotted;
    /// ```
    Dotted,
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
///     StrokeStyle::Dashed
/// );
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Stroke<D> {
    /// The color of the stroke.
    pub fill: Color,
    /// The thickness of the stroke.
    pub thickness: Measure<D>,
    /// The visual style of the stroke.
    pub style: StrokeStyle,
}

impl<D> Stroke<D> {
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
    ///     StrokeStyle::Dashed
    /// );
    /// ```
    pub const fn with_style(fill: Color, thickness: Measure<D>, style: StrokeStyle) -> Self {
        Self {
            fill,
            thickness,
            style,
        }
    }
}
