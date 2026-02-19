mod cache;
mod primitive;
mod text;

pub use cache::RenderCache;
pub use primitive::{LineArrows, LineExtensions, Primitive};
pub use text::Text;

/// The rendering quality of a buffer.
///
/// This controls the error tolerance of the tessellation algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Quality {
    /// High triangle count, very smooth curves.
    High,
    #[default]
    /// Balanced performance and visual fidelity.
    Medium,
    /// Low triangle count, "blocky" curves. Best for performance.
    Low,
    /// Custom value.
    Custom {
        /// Tessellation quality multiplier. Higher is better (`High` = 2.0).
        ///
        /// Clamped to the range `0.1..=5.0`.
        tessellation: f32,
        /// Text rendering tolerance. Lower is better (`High` = 0.2).
        ///
        /// Values below `0.001` are treated as `0.001`.
        text: f32,
    },
}

impl Quality {
    /// Converts the quality setting into a text tolerance value.
    /// Lower values mean higher precision (smoother text outlines).
    pub(crate) const fn to_text_tolerance(self) -> f32 {
        match self {
            Self::High => 0.2,
            Self::Medium => 0.5,
            Self::Low => 1.5,
            Self::Custom { text, .. } => text.max(0.001),
        }
    }

    /// Converts the quality setting into a tessellation tolerance value.
    /// Higher values mean higher precision (more triangles)
    pub(crate) const fn to_tessellation_quality(self) -> f32 {
        match self {
            Self::High => 2.0,
            Self::Medium => 1.0,
            Self::Low => 0.5,
            Self::Custom { tessellation, .. } => tessellation.clamp(0.1, 5.0),
        }
    }
}

/// The preferred rendering backend for a renderer.
pub enum Backend {
    /// GPU-accelerated mesh-based rendering (e.g. wgpu).
    Mesh,
    /// Software path-based rendering (e.g. tiny-skia).
    Path,
}

/// Renderer requirements for plotting.
pub trait Renderer<Font = iced_core::Font>:
    iced_core::Renderer
    + iced_core::text::Renderer<Font = Font>
    + iced_graphics::geometry::Renderer
    + iced_graphics::mesh::Renderer
{
    /// Returns the preferred rendering backend for this renderer.
    fn preferred_backend(&self) -> Backend;
}

impl<A, B> Renderer<A::Font> for iced_renderer::fallback::Renderer<A, B>
where
    A: Renderer,
    B: Renderer<A::Font, Paragraph = A::Paragraph, Editor = A::Editor>,
{
    fn preferred_backend(&self) -> Backend {
        match self {
            Self::Primary(primary) => primary.preferred_backend(),
            Self::Secondary(secondary) => secondary.preferred_backend(),
        }
    }
}

impl Renderer for iced_wgpu::Renderer {
    fn preferred_backend(&self) -> Backend {
        Backend::Mesh
    }
}

impl Renderer for iced_tiny_skia::Renderer {
    fn preferred_backend(&self) -> Backend {
        Backend::Path
    }
}
