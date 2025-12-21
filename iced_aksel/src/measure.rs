use std::ops::{Add, Div, Mul, Sub};

use aksel::{Float, Transform};

/// Defines how a dimension should be interpreted by the chart.
///
/// `Measure` allows you to specify sizes in either screen pixels (fixed) or plot
/// data units (scales with zoom). This is useful for shapes that should maintain
/// their size regardless of zoom level, or for shapes that should scale with the data.
///
/// # Example
///
/// ```rust
/// use iced_aksel::Measure;
///
/// // Fixed size: always 10 pixels wide, regardless of zoom
/// let fixed: Measure<f64> = Measure::Screen(10.0);
///
/// // Data size: 5.0 units in plot space, scales when zooming
/// let scalable = Measure::Plot(5.0_f64);
/// ```
#[derive(Clone, Copy, Debug)]
pub enum Measure<D> {
    /// Fixed size in screen pixels.
    ///
    /// This size does not change when zooming the chart. Useful for UI elements
    /// like markers, labels, or decorations that should remain visually consistent.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use iced_aksel::Measure;
    /// let marker_size: Measure<f64> = Measure::Screen(5.0); // 5 pixels
    /// ```
    Screen(f32),

    /// Size in chart data units.
    ///
    /// This size scales proportionally when zooming the chart. Useful for data
    /// visualizations where the size represents a meaningful quantity.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use iced_aksel::Measure;
    /// let bar_width = Measure::Plot(10.0_f64); // 10 data units
    /// ```
    Plot(D),
}

impl<D: Float> Measure<D> {
    /// Resolves the measure to screen pixels along the **X axis**.
    ///
    /// * If `Screen`, returns the pixel value directly.
    /// * If `Plot`, calculates the screen distance covered by the data units on the X axis.
    pub fn resolve_x(&self, transform: &Transform<D, f32, f32>) -> f32 {
        match self {
            Self::Screen(px) => *px,
            Self::Plot(units) => {
                let p0 = transform.x_to_screen(&D::zero());
                let p1 = transform.x_to_screen(units);
                (p1 - p0).abs()
            }
        }
    }

    /// Resolves the measure to screen pixels along the **Y axis**.
    ///
    /// * If `Screen`, returns the pixel value directly.
    /// * If `Plot`, calculates the screen distance covered by the data units on the Y axis.
    pub fn resolve_y(&self, transform: &Transform<D, f32, f32>) -> f32 {
        match self {
            Self::Screen(px) => *px,
            Self::Plot(units) => {
                let p0 = transform.y_to_screen(&D::zero());
                let p1 = transform.y_to_screen(units);
                (p1 - p0).abs()
            }
        }
    }
}

// 1. Implement Multiplication: Length * Number
impl<D: Float> Mul<D> for Measure<D> {
    type Output = Self;

    fn mul(self, scalar: D) -> Self {
        match self {
            // Convert scalar to f32 for Screen pixels
            Self::Screen(px) => Self::Screen(px * scalar.to_f32().unwrap()),
            // Direct multiplication for Plot units
            Self::Plot(val) => Self::Plot(val * scalar),
        }
    }
}

// 2. Implement Division: Length / Number
impl<D: Float> Div<D> for Measure<D> {
    type Output = Self;

    fn div(self, scalar: D) -> Self {
        match self {
            Self::Screen(px) => Self::Screen(px / scalar.to_f32().unwrap()),
            Self::Plot(val) => Self::Plot(val / scalar),
        }
    }
}

// --- Addition (Length + Number) ---
impl<D: Float> Add<D> for Measure<D> {
    type Output = Self;

    fn add(self, scalar: D) -> Self {
        match self {
            // Add to pixels (convert scalar to f32)
            Self::Screen(px) => Self::Screen(px + scalar.to_f32().unwrap()),
            // Add to plot units
            Self::Plot(val) => Self::Plot(val + scalar),
        }
    }
}

// --- Subtraction (Length - Number) ---
impl<D: Float> Sub<D> for Measure<D> {
    type Output = Self;

    fn sub(self, scalar: D) -> Self {
        match self {
            Self::Screen(px) => Self::Screen(px - scalar.to_f32().unwrap()),
            Self::Plot(val) => Self::Plot(val - scalar),
        }
    }
}
