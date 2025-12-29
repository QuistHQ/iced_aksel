use crate::render::MeshBuffer;
use crate::render::text::GeometricFont;
use ab_glyph::{Font, PxScale, ScaleFont};
use iced_core::{
    Color, Point,
    alignment::{Horizontal, Vertical},
};
use iced_graphics::color::pack;
use iced_graphics::mesh::SolidVertex2D;
use lru::LruCache;
use lyon::math::point;
use lyon::path::Path;
use lyon::path::builder::PathBuilder;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, FillVertexConstructor, VertexBuffers,
};
use std::num::NonZeroUsize;
use ttf_parser::OutlineBuilder;

/// The rendering quality of the vector text.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Quality {
    High,
    Medium,
    Low,
    Custom(f32),
}

impl Quality {
    pub fn to_tolerance(self) -> f32 {
        match self {
            Self::High => 0.2,
            Self::Medium => 0.5,
            Self::Low => 1.5,
            Self::Custom(val) => val.max(0.001),
        }
    }
}

impl Default for Quality {
    fn default() -> Self {
        Self::Medium
    }
}

/// A single tessellated character cached for reuse.
#[derive(Clone, Debug)]
pub struct CachedGlyph {
    pub geometry: VertexBuffers<Point, u16>,
    pub tolerance: f32,
}

/// Uniquely identifies a specific glyph's geometry across different fonts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct CacheKey {
    pub font_id: usize,
    pub glyph_id: u16,
}

const CACHE_CAPACITY: usize = 2000;

/// A safe wrapper around the glyph cache to enforce correct keying and memory limits.
pub struct TextTessellationCache {
    cache: LruCache<CacheKey, CachedGlyph>,
}

impl TextTessellationCache {
    pub fn new() -> Self {
        Self {
            cache: LruCache::new(NonZeroUsize::new(CACHE_CAPACITY).unwrap()),
        }
    }

    pub fn get(&mut self, key: CacheKey) -> Option<&CachedGlyph> {
        self.cache.get(&key)
    }

    pub fn insert(&mut self, key: CacheKey, glyph: CachedGlyph) {
        self.cache.put(key, glyph);
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

impl Default for TextTessellationCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Holds the mutable "heavy machinery" required to render text.
pub struct TextRenderContext<'a> {
    pub buffer: &'a mut MeshBuffer,
    pub tessellator: &'a mut FillTessellator,
    pub glyph_cache: &'a mut TextTessellationCache,
    pub scratch_geometry: &'a mut VertexBuffers<Point, u16>,
}

/// The specific properties for a single draw call.
pub struct TextRequest<'a> {
    pub content: &'a str,
    pub position: Point,
    pub font: &'a GeometricFont<'a>,
    pub size: f32,
    pub color: Color,
    pub rotation: f32,
    pub horizontal_alignment: Horizontal,
    pub vertical_alignment: Vertical,
    pub quality: Quality,
    pub quality_multiplier: f32,
    pub letter_spacing: f32,
}

// --- Adapter to bridge ttf-parser commands to lyon commands ---
struct LyonPathBuilder<'a>(pub &'a mut dyn PathBuilder);

impl<'a> OutlineBuilder for LyonPathBuilder<'a> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.begin(point(x, y), &[]);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.0.line_to(point(x, y), &[]);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.0.quadratic_bezier_to(point(x1, y1), point(x, y), &[]);
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.0
            .cubic_bezier_to(point(x1, y1), point(x2, y2), point(x, y), &[]);
    }
    fn close(&mut self) {
        self.0.end(true);
    }
}

struct TextVertexConstructor;

impl FillVertexConstructor<Point> for TextVertexConstructor {
    fn new_vertex(&mut self, vertex: FillVertex) -> Point {
        let position = vertex.position();
        Point::new(position.x, position.y)
    }
}

/// Snaps the calculated tolerance to fixed tiers (LODs) to prevent frequent re-tessellation.
/// Lower tolerance = Higher Quality (more triangles).
fn snap_to_bucket(raw_tolerance: f32) -> f32 {
    if raw_tolerance > 1.5 {
        2.0 // Very Low
    } else if raw_tolerance > 1.0 {
        1.0 // Low
    } else if raw_tolerance > 0.5 {
        0.5 // Medium
    } else if raw_tolerance > 0.2 {
        0.2 // High
    } else if raw_tolerance > 0.1 {
        0.1 // Ultra
    } else {
        0.05 // Extreme (High Zoom)
    }
}

/// Draws text as a geometric mesh (triangles).
pub fn draw_geometric_text(ctx: &mut TextRenderContext, req: TextRequest) {
    if req.content.is_empty() {
        return;
    }

    let font_layout = &req.font.layout;
    let font_geometry = &req.font.geometry;
    let font_id = req.font.id;

    // 1. Setup Metrics & Scaling
    let pixel_scale = PxScale::from(req.size);
    let scaled_font = font_layout.as_scaled(pixel_scale);

    let font_units_per_em = font_geometry.units_per_em() as f32;
    let geometry_scale_factor = req.size / font_units_per_em;

    // Quality Settings
    let base_error = req.quality.to_tolerance();
    let desired_pixel_error = base_error / req.quality_multiplier.max(0.1);
    let safe_font_size = req.size.max(0.001);

    // Calculate raw tolerance
    let raw_tolerance = (desired_pixel_error * font_units_per_em) / safe_font_size;

    // Snap to bucket (Fix for Jitter/Churn)
    // We pick the next highest quality bucket to ensure the user's visual requirement is met.
    let tessellation_tolerance = snap_to_bucket(raw_tolerance);

    let fill_options = FillOptions::default().with_tolerance(tessellation_tolerance);

    // Layout Metrics
    let ascent = scaled_font.ascent();
    let descent = scaled_font.descent();
    let line_gap = scaled_font.line_gap();
    let line_height = ascent - descent + line_gap;

    // 2. Layout Phase: Measure lines
    let lines: Vec<&str> = req.content.lines().collect();
    let line_count = lines.len() as f32;
    let total_block_height = line_height * line_count;

    let mut current_y = match req.vertical_alignment {
        Vertical::Top => ascent,
        Vertical::Center => ascent - (total_block_height / 2.0) + (line_height / 2.0),
        Vertical::Bottom => ascent - total_block_height + line_height,
    };

    // 3. Render Phase
    for line_text in lines {
        // Measure line width
        let mut line_width = 0.0;
        let mut last_glyph_id = None;

        for character in line_text.chars() {
            let glyph_id = font_layout.glyph_id(character);
            if let Some(last) = last_glyph_id {
                line_width += scaled_font.kern(last, glyph_id);
            }
            line_width += scaled_font.h_advance(glyph_id) * req.letter_spacing;
            last_glyph_id = Some(glyph_id);
        }

        let alignment_offset_x = match req.horizontal_alignment {
            Horizontal::Left => 0.0,
            Horizontal::Center => -line_width / 2.0,
            Horizontal::Right => -line_width,
        };

        let mut cursor_x = 0.0;
        last_glyph_id = None;

        for character in line_text.chars() {
            let glyph_id = font_layout.glyph_id(character);
            let glyph_index = glyph_id.0;

            let cache_key = CacheKey {
                font_id,
                glyph_id: glyph_index,
            };

            // Apply Kerning
            if let Some(last) = last_glyph_id {
                cursor_x += scaled_font.kern(last, glyph_id);
            }

            // --- Cache & Tessellation Logic ---
            let needs_update = if let Some(cached) = ctx.glyph_cache.get(cache_key) {
                // If the cached version is rougher (higher tolerance) than what we need, update it.
                // Since we use buckets, this comparison is stable.
                cached.tolerance > tessellation_tolerance + 0.0001
            } else {
                true
            };

            if needs_update {
                ctx.scratch_geometry.vertices.clear();
                ctx.scratch_geometry.indices.clear();

                let ttf_glyph_id = ttf_parser::GlyphId(glyph_index);
                let mut path_builder = Path::builder();
                let mut builder_adapter = LyonPathBuilder(&mut path_builder);

                if font_geometry
                    .outline_glyph(ttf_glyph_id, &mut builder_adapter)
                    .is_some()
                {
                    let path = path_builder.build();
                    let _ = ctx.tessellator.tessellate_path(
                        &path,
                        &fill_options,
                        &mut BuffersBuilder::new(ctx.scratch_geometry, TextVertexConstructor),
                    );
                }

                ctx.glyph_cache.insert(
                    cache_key,
                    CachedGlyph {
                        geometry: ctx.scratch_geometry.clone(),
                        tolerance: tessellation_tolerance,
                    },
                );
            }

            if let Some(cached_glyph) = ctx.glyph_cache.get(cache_key) {
                flush_character_to_mesh(
                    ctx.buffer,
                    &cached_glyph.geometry,
                    req.position,
                    req.rotation,
                    req.color,
                    alignment_offset_x + cursor_x,
                    current_y,
                    geometry_scale_factor,
                );
            }

            // Advance Cursor
            cursor_x += scaled_font.h_advance(glyph_id) * req.letter_spacing;
            last_glyph_id = Some(glyph_id);
        }

        current_y += line_height;
    }
}

fn flush_character_to_mesh(
    target_buffer: &mut MeshBuffer,
    source_geometry: &VertexBuffers<Point, u16>,
    screen_origin: Point,
    rotation_radians: f32,
    color: Color,
    local_offset_x: f32,
    local_offset_y: f32,
    scale_factor: f32,
) {
    let mesh = target_buffer.get_mesh_mut();
    let start_index = mesh.vertices.len() as u32;

    let (sin, cos) = rotation_radians.sin_cos();
    let packed_color = pack(color);
    let flip_y = -1.0;

    for vertex in &source_geometry.vertices {
        let scaled_x = vertex.x * scale_factor;
        let scaled_y = vertex.y * scale_factor;

        let local_x = scaled_x + local_offset_x;
        let local_y = (scaled_y * flip_y) + local_offset_y;

        let rotated_x = local_x * cos - local_y * sin;
        let rotated_y = local_x * sin + local_y * cos;

        let final_x = screen_origin.x + rotated_x;
        let final_y = screen_origin.y + rotated_y;

        mesh.vertices.push(SolidVertex2D {
            position: [final_x, final_y],
            color: packed_color,
        });
    }

    for index in &source_geometry.indices {
        mesh.indices.push(start_index + *index as u32);
    }
}
