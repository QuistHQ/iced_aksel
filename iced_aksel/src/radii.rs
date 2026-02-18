//! Radii type implementation

use std::ops::{Add, Div, Mul, Sub};

use crate::Measure;
use aksel::{Float, Transform};

/// A radius that covers both the x and y axes
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Radii<T = f32> {
    /// The radius on the x-axis
    pub x: T,
    /// The radius on the y-axis
    pub y: T,
}

impl<T> Radii<T> {
    /// Creates a new radii
    ///
    /// ```rust
    /// Radii::new(2.0, 5.0) // 2.0 on the x-axis, 5.0 on the y-axis
    /// ```
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    /// Crates a new *uniform* radii
    ///
    /// ```rust
    /// Radii::uniform(2.0) // 2.0 on both axes
    /// ```
    pub const fn uniform(radius: T) -> Self
    where
        T: Copy,
    {
        Self {
            x: radius,
            y: radius,
        }
    }
}

impl<D: Float> Radii<Measure<D>> {
    /// Resolves the [`Radii`] using the current plot [`Transform`]
    pub fn resolve(&self, transform: &Transform<D, f32, f32>) -> ResolvedRadii {
        ResolvedRadii {
            x: self.x.resolve_x(transform),
            y: self.y.resolve_y(transform),
        }
    }
}

// Multiply with singular number
//
// `2.0 * radii`
impl<T: Mul<Output = T> + Copy> Mul<T> for Radii<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

// Multiply with self
//
// `radii * radii`
impl<T: Mul<Output = T>> Mul<Self> for Radii<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

// Divide with singular number
//
// `radii / 2.0`
impl<T: Div<Output = T> + Copy> Div<T> for Radii<T> {
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

// Divide with self
//
// `radii / radii`
impl<T: Div<Output = T>> Div<Self> for Radii<T> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

// Addition with singular number
//
// `radii + 2.0`
impl<T: Add<Output = T> + Copy> Add<T> for Radii<T> {
    type Output = Self;

    fn add(self, rhs: T) -> Self::Output {
        Self {
            x: self.x + rhs,
            y: self.y + rhs,
        }
    }
}

// Addition with self
//
// `radii + radii`
impl<T: Add<Output = T>> Add<Self> for Radii<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

// Subtraction with singular number
//
// `radii - 2.0`
impl<T: Sub<Output = T> + Copy> Sub<T> for Radii<T> {
    type Output = Self;

    fn sub(self, rhs: T) -> Self::Output {
        Self {
            x: self.x - rhs,
            y: self.y - rhs,
        }
    }
}

// Subtraction with self
//
// `radii - radii`
impl<T: Sub<Output = T>> Sub<Self> for Radii<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

/// A radii with all measurements resolved to screen-space pixels.
///
/// Produced by converting a [`Radii<Measure<T>>`](Radii) through a plot transform, or constructed
/// manually from pixel-values.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct ResolvedRadii {
    /// Resolved value of the radius on the x-axis in pixels
    pub x: f32,
    /// Resolved value of the radius on the y-axis in pixels
    pub y: f32,
}

impl ResolvedRadii {
    /// Checks wether or not the Radii values are close to equal (Accounting for sub-pixel
    /// tolerance)
    pub const fn is_uniform(&self) -> bool {
        (self.x - self.y).abs() < 0.001
    }

    /// Returns a new Radii, calling the [`f32::max`] method on the x and y values with the `other`
    /// parameter
    pub const fn max(self, other: f32) -> Self {
        Self {
            x: self.x.max(other),
            y: self.y.max(other),
        }
    }
}

impl Mul<f32> for ResolvedRadii {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Div<f32> for ResolvedRadii {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl Add<f32> for ResolvedRadii {
    type Output = Self;

    fn add(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x + rhs,
            y: self.y + rhs,
        }
    }
}

impl Sub<f32> for ResolvedRadii {
    type Output = Self;

    fn sub(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x - rhs,
            y: self.y - rhs,
        }
    }
}
