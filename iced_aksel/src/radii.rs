//! Radii type implementation

use std::ops::{Add, Div, Mul, Sub};

use crate::Measure;
use aksel::{Float, Transform};

fn safeguard_radius(radius: f32) -> Option<ResolvedRadius> {
    (radius > 0.5).then_some(ResolvedRadius(radius))
}

/// A singular Radius
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Radius<T = f32>(pub T);

impl<T> Radius<T> {
    /// Creates a new [`Radius`]
    pub const fn new(radius: T) -> Self {
        Self(radius)
    }
}

impl<T: Float> From<T> for Radius<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T: Float> From<Measure<T>> for Radius<Measure<T>> {
    fn from(value: Measure<T>) -> Self {
        Self::new(value)
    }
}

impl<T: Float> Radius<T> {
    /// Resolves the [`Radius`] to screen-space pixels. Will return None if:
    /// * The radius is < 0.5, as this would result in being invisible when rendered
    /// * `T` can't be cast to a `f32`
    pub fn resolve(&self) -> Option<ResolvedRadius> {
        let radius = self.0.to_f32()?;
        safeguard_radius(radius)
    }
}

impl<T: Float> Radius<Measure<T>> {
    /// Resolves the [`Radius`] to screen-space pixels using the minimum
    /// value of the resolved X- and Y-axis values of the provided transform.
    ///
    /// Will return None if:
    /// * The radius is < 0.5 as this would result in being invisible when rendered
    pub fn resolve_isotropic(&self, transform: &Transform<T, f32, f32>) -> Option<ResolvedRadius> {
        let x = self.0.resolve_x(transform);
        let y = self.0.resolve_y(transform);
        let radius = x.min(y);
        safeguard_radius(radius)
    }

    /// Resolves the [`Radius`] to screen-space pixels using the X-axis
    /// of the provided transform. Will return None if:
    /// * The radius is < 0.5 as this would result in being invisible when rendered
    pub fn resolve_x(&self, transform: &Transform<T, f32, f32>) -> Option<ResolvedRadius> {
        let radius = self.0.resolve_x(transform);
        safeguard_radius(radius)
    }

    /// Resolves the [`Radius`] to screen-space pixels using the Y-axis
    /// of the provided transform. Will return None if:
    /// * The radius is < 0.5 as this would result in being invisible when rendered
    pub fn resolve_y(&self, transform: &Transform<T, f32, f32>) -> Option<ResolvedRadius> {
        let radius = self.0.resolve_y(transform);
        safeguard_radius(radius)
    }
}

/// A radius that covers both the x and y axes
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Radii<T = f32> {
    /// The radius on the x-axis
    pub x: Radius<T>,
    /// The radius on the y-axis
    pub y: Radius<T>,
}

impl<T> Radii<T> {
    /// Creates a new radii
    pub const fn new(x: T, y: T) -> Self {
        Self {
            x: Radius(x),
            y: Radius(y),
        }
    }

    /// Creates a new *uniform* radii (X and Y values are the exact same)
    pub const fn uniform(radius: T) -> Self
    where
        T: Copy,
    {
        Self {
            x: Radius(radius),
            y: Radius(radius),
        }
    }
}

impl<T: Float> Radii<T> {
    /// Resolves the [`Radii`] to screen-space. Will return None if:
    /// * Either of the inner radii can't be cast to a `f32`
    /// * Either of the inner radii is < 0.5 as this would result in being invisible when rendered
    pub fn resolve(&self) -> Option<ResolvedRadii> {
        Some(ResolvedRadii {
            x: self.x.resolve()?.0,
            y: self.y.resolve()?.0,
        })
    }
}

impl<D: Float> Radii<Measure<D>> {
    /// Resolves the [`Radii`] using the current plot [`Transform`]. Will return None if
    /// either of the inner radii is < 0.5 as this would result in being invisible when rendered
    pub fn resolve(&self, transform: &Transform<D, f32, f32>) -> Option<ResolvedRadii> {
        Some(ResolvedRadii {
            x: self.x.resolve_x(transform)?.0,
            y: self.y.resolve_y(transform)?.0,
        })
    }
}

/// A radius in screen-space pixels
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct ResolvedRadius(pub(crate) f32);

/// A radii with all measurements resolved to screen-space pixels.
///
/// Produced by converting a [`Radii<Measure<T>>`](Radii) through a plot transform, or constructed
/// manually from pixel-values.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct ResolvedRadii {
    /// Resolved value of the radius on the x-axis in pixels
    pub(crate) x: f32,
    /// Resolved value of the radius on the y-axis in pixels
    pub(crate) y: f32,
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

// ==== RADIUS ====
impl<T: Mul<Output = T>> Mul<T> for Radius<T> {
    type Output = Self;
    fn mul(self, rhs: T) -> Self::Output {
        Self(self.0 * rhs)
    }
}
impl<T: Mul<Output = T>> Mul<Self> for Radius<T> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}
impl<T: Div<Output = T>> Div<T> for Radius<T> {
    type Output = Self;
    fn div(self, rhs: T) -> Self::Output {
        Self(self.0 / rhs)
    }
}
impl<T: Div<Output = T>> Div<Self> for Radius<T> {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}
impl<T: Add<Output = T>> Add<T> for Radius<T> {
    type Output = Self;
    fn add(self, rhs: T) -> Self::Output {
        Self(self.0 + rhs)
    }
}
impl<T: Add<Output = T>> Add<Self> for Radius<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
impl<T: Sub<Output = T>> Sub<T> for Radius<T> {
    type Output = Self;
    fn sub(self, rhs: T) -> Self::Output {
        Self(self.0 - rhs)
    }
}
impl<T: Sub<Output = T>> Sub<Self> for Radius<T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

// ==== RADII ====
impl<T: Mul<Output = T> + Copy> Mul<T> for Radii<T> {
    type Output = Self;
    fn mul(self, rhs: T) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}
impl<T: Mul<Output = T>> Mul<Self> for Radii<T> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}
impl<T: Div<Output = T> + Copy> Div<T> for Radii<T> {
    type Output = Self;
    fn div(self, rhs: T) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}
impl<T: Div<Output = T>> Div<Self> for Radii<T> {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}
impl<T: Add<Output = T> + Copy> Add<T> for Radii<T> {
    type Output = Self;
    fn add(self, rhs: T) -> Self::Output {
        Self {
            x: self.x + rhs,
            y: self.y + rhs,
        }
    }
}
impl<T: Add<Output = T>> Add<Self> for Radii<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl<T: Sub<Output = T> + Copy> Sub<T> for Radii<T> {
    type Output = Self;
    fn sub(self, rhs: T) -> Self::Output {
        Self {
            x: self.x - rhs,
            y: self.y - rhs,
        }
    }
}
impl<T: Sub<Output = T>> Sub<Self> for Radii<T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

// ==== RESOLVED RADII ====
impl Mul<f32> for ResolvedRadii {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}
impl Mul<Self> for ResolvedRadii {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
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
impl Div<Self> for ResolvedRadii {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
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
impl Add<Self> for ResolvedRadii {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
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
impl Sub<Self> for ResolvedRadii {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
