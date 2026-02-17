//! Vector text rendering engine.
//!
//! This module handles the conversion of raw text into geometric meshes (triangles).
//! Unlike standard rasterization (which creates textures), this approach generates
//! resolution-independent geometry that remains sharp at any zoom level.
//!
//! # Key Features
//! * **Infinite Zoom:** Text remains crisp because it is essentially a collection of mathematical curves.
//! * **Dynamic LOD:** The engine automatically adjusts the triangle count based on the size of the text on screen.
//! * **Memory Safety:** Uses an LRU (Least Recently Used) cache to prevent unbounded memory growth.

use crate::render::Text;
use core::f32;
use iced_core::{
    Color, Point,
    alignment::{Horizontal, Vertical},
};
use iced_graphics::{
    color::pack,
    text::{
        cosmic_text::{
            Buffer, Metrics,
            fontdb::ID,
            skrifa::{
                FontRef, GlyphId16, MetadataProvider,
                outline::{DrawSettings, OutlinePen},
                prelude::{LocationRef, Size},
            },
        },
        font_system,
    },
};
use iced_graphics::{mesh::SolidVertex2D, text::cosmic_text};
use lru::LruCache;
use lyon::math::point;
use lyon::path::Path;
use lyon::path::builder::PathBuilder;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, FillVertexConstructor, VertexBuffers,
};
use lyon_path::FillRule;
use std::num::NonZeroUsize;

// -----------------------------------------------------------------------------
// Configuration & Types
// -----------------------------------------------------------------------------

/// The maximum number of tessellated glyphs to keep in memory.
///
/// 2000 glyphs is roughly enough for ~20-30 completely different alphabets
/// or font sizes active simultaneously.
const CACHE_CAPACITY: usize = 2000;

// -----------------------------------------------------------------------------
// Caching Infrastructure
// -----------------------------------------------------------------------------

/// A single tessellated character cached for reuse.
///
/// Storing the `VertexBuffers` directly allows us to "stamp" this geometry
/// into the main mesh cache multiple times without re-running the expensive
/// tessellation math.
#[derive(Clone, Debug)]
struct CachedGlyph {
    pub geometry: VertexBuffers<Point, u16>,
}

impl CachedGlyph {
    /// Creates a new **empty** cached glyph - Useful for non-drawable glyphs
    pub fn empty() -> Self {
        Self {
            geometry: VertexBuffers::new(),
        }
    }

    /// Returns true if the cached glyph is empty
    pub const fn is_empty(&self) -> bool {
        self.geometry.vertices.is_empty() || self.geometry.indices.is_empty()
    }
}

/// Uniquely identifies a specific glyph's geometry across different fonts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CacheKey {
    pub font_id: ID,
    pub face_index: u32,
    pub glyph_id: u16,
    pub size_bucket: u16,
    pub tolerance_bucket: u16,
}

/// A safe wrapper around the glyph cache to enforce correct keying and memory limits.
///
/// Internally uses an `LruCache` to ensure we don't leak memory in long-running applications.
pub struct TextTessellationCache {
    cache: LruCache<CacheKey, CachedGlyph>,
}

impl TextTessellationCache {
    pub fn new() -> Self {
        Self {
            cache: LruCache::new(NonZeroUsize::new(CACHE_CAPACITY).unwrap()),
        }
    }

    /// Retrieves a cached glyph.
    ///
    /// **Note:** Takes `&mut self` because accessing an LRU cache updates the
    /// internal "recency" list.
    fn get(&mut self, key: CacheKey) -> Option<&CachedGlyph> {
        self.cache.get(&key)
    }

    /// Inserts a new glyph into the cache
    fn insert(&mut self, key: CacheKey, glyph: CachedGlyph) {
        self.cache.put(key, glyph);
    }

    /// Clears the entire cache.
    /// Should be called when global quality settings change.
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

impl Default for TextTessellationCache {
    fn default() -> Self {
        Self::new()
    }
}

// -----------------------------------------------------------------------------
// Rendering Contexts (API Refactor)
// -----------------------------------------------------------------------------

/// Holds the mutable "heavy machinery" required to render text.
///
/// This context groups the buffers and caches to avoid passing 10+ arguments
/// to rendering functions.
pub struct TextRenderContext<'a> {
    /// The final destination for the mesh data.
    pub mesh_buffer: &'a mut crate::render::cache::MeshData,
    /// The Lyon tessellator instance (reused to avoid allocation).
    pub tessellator: &'a mut FillTessellator,
    /// The LRU cache for glyph geometry.
    pub glyph_cache: &'a mut TextTessellationCache,
    /// A scratch buffer for intermediate tessellation results.
    pub scratch_geometry: &'a mut VertexBuffers<Point, u16>,
    /// A global multiplier for quality (e.g. from the Chart widget).
    pub quality_multiplier: f32,
}

// -----------------------------------------------------------------------------
// Helpers & Adapters
// -----------------------------------------------------------------------------

// Adapter to bridge ttf-parser commands (MoveTo, LineTo) to lyon commands.
struct LyonPathBuilder<'a>(pub &'a mut dyn PathBuilder);

impl<'a> OutlinePen for LyonPathBuilder<'a> {
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

/// Snaps the calculated tolerance to fixed tiers (LODs).
///
/// **Why?** Without this, slight zoom changes (e.g. 12.0px -> 12.1px) would
/// result in a tiny change in required tolerance, forcing the engine to
/// re-tessellate the entire cache. By "bucketing" the values, we reuse
/// existing meshes until a significant quality jump is actually needed.
fn snap_to_bucket(raw_tolerance: f32) -> f32 {
    if raw_tolerance > 1.5 {
        2.0 // Very Low Detail
    } else if raw_tolerance > 1.0 {
        1.0 // Low Detail
    } else if raw_tolerance > 0.5 {
        0.5 // Medium Detail
    } else if raw_tolerance > 0.2 {
        0.2 // High Detail
    } else if raw_tolerance > 0.1 {
        0.1 // Ultra Detail
    } else {
        0.05 // Extreme Detail (Maximum zoom)
    }
}

fn size_bucket(px: f32) -> u16 {
    // Example: quarter-pixel buckets
    ((px * 4.0).round().clamp(1.0, u16::MAX as f32)) as u16
}

fn tol_bucket(px: f32) -> u16 {
    // Example: thousandth-pixel buckets
    ((px * 1000.0).round().clamp(1.0, u16::MAX as f32)) as u16
}

// -----------------------------------------------------------------------------
// Main Render Logic
// -----------------------------------------------------------------------------

/// An owned LayoutRun
#[derive(Clone)]
struct OwnedRun {
    line_width: f32,
    line_y: f32,
    line_height: f32,
    glyphs: Vec<cosmic_text::LayoutGlyph>,
}

/// Draws text as a geometric mesh (triangles).
///
/// This is the core function of the text-to-mesh engine. It performs layout,
/// retrieves/generates glyph geometry, and flushes it to the mesh buffer.
pub fn draw_geometric_text(ctx: &mut TextRenderContext, req: Text) {
    if req.content.is_empty() || req.size.0 <= 2.0 {
        return;
    }

    let mut lock = font_system().write().expect("Failed to get font-system");
    let font_system = lock.raw();
    let mut text_buffer = Buffer::new(
        font_system,
        Metrics::new(req.size.into(), req.line_height.into()),
    );

    text_buffer.set_size(font_system, Some(req.bounds.width), Some(req.bounds.height));
    text_buffer.set_wrap(font_system, iced_graphics::text::to_wrap(req.wrapping));
    text_buffer.set_text(
        font_system,
        &req.content,
        &iced_graphics::text::to_attributes(req.font),
        iced_graphics::text::to_shaping(iced_core::text::Shaping::Auto, &req.content),
        None,
    );

    // --- Get layout runs ---
    let runs: Vec<OwnedRun> = {
        let bw = text_buffer.borrow_with(font_system);
        bw.layout_runs()
            .map(|run| OwnedRun {
                line_y: run.line_y,
                line_width: run.line_w,
                line_height: run.line_height,
                glyphs: run.glyphs.to_vec(),
            })
            .collect()
    };

    if runs.is_empty() {
        return;
    }

    // --- Compute vertical alignment ---
    let min_y = runs.first().unwrap().line_y;
    let max_y = runs
        .iter()
        .map(|r| r.line_y + r.line_height)
        .fold(f32::NEG_INFINITY, f32::max);
    let block_height = max_y - min_y;
    let vertical_offset = match req.vertical_alignment {
        Vertical::Top => 0.0,
        Vertical::Center => -block_height / 2.0,
        Vertical::Bottom => -block_height,
    };

    // --- Decide pixel tolerance ---
    let base_error_px = req.tolerance;
    let desired_error_px = base_error_px / ctx.quality_multiplier.max(0.1);
    let tess_tol_px = snap_to_bucket(desired_error_px);
    let tol_bucket_u16 = tol_bucket(tess_tol_px);
    let size_bucket_u16 = size_bucket(req.size.0);

    let fill_options = FillOptions::default()
        .with_tolerance(tess_tol_px)
        .with_fill_rule(FillRule::NonZero);

    // --- Render glyphs ---
    for run in runs {
        // Horizontal offset
        let horizontal_offset = match req.horizontal_alignment {
            Horizontal::Left => 0.0,
            Horizontal::Center => -run.line_width / 2.0,
            Horizontal::Right => -run.line_width,
        };

        for glyph in run.glyphs {
            // Find the chosen face index in fontdb (important for TTC)
            let face_info = match font_system.db().face(glyph.font_id) {
                Some(info) => info,
                None => continue,
            };
            let face_index = face_info.index;

            let key = CacheKey {
                font_id: glyph.font_id,
                face_index,
                glyph_id: glyph.glyph_id,
                size_bucket: size_bucket_u16,
                tolerance_bucket: tol_bucket_u16,
            };

            // --- Create cached glyph ---
            if ctx.glyph_cache.get(key).is_none() {
                // Get font from system
                let font_arc = match font_system.get_font(glyph.font_id, glyph.font_weight) {
                    Some(f) => f,
                    // Font not found - Just continue
                    None => continue,
                };

                // Create skrifa font ref
                let font_ref = match FontRef::from_index(font_arc.data(), face_index) {
                    Ok(f) => f,
                    // Same here - Not found, just continue
                    Err(_) => continue,
                };

                // Create the outline
                let outlines = font_ref.outline_glyphs();
                let gid = GlyphId16::new(glyph.glyph_id);
                let outline_glyph = match outlines.get(gid.into()) {
                    Some(og) => og,
                    // Missing outline (e.g. Bitmap emoji)
                    None => {
                        // We cache an empty glyph to avoid calculations next frame
                        ctx.glyph_cache.insert(key, CachedGlyph::empty());
                        continue;
                    }
                };

                // Create a "pan" to draw the path
                let mut path_builder = Path::builder();
                let mut pen = LyonPathBuilder(&mut path_builder);

                // Create the settings for drawing
                let settings =
                    DrawSettings::unhinted(Size::new(glyph.font_size), LocationRef::default());

                // Draw the glyph and cache it
                if outline_glyph.draw(settings, &mut pen).is_ok() {
                    let path = path_builder.build();

                    ctx.scratch_geometry.clear();

                    let _ = ctx.tessellator.tessellate_path(
                        &path,
                        &fill_options,
                        &mut BuffersBuilder::new(ctx.scratch_geometry, TextVertexConstructor),
                    );

                    ctx.glyph_cache.insert(
                        key,
                        CachedGlyph {
                            geometry: ctx.scratch_geometry.clone(),
                        },
                    );
                } else {
                    // We can't draw - Insert empty cache key
                    ctx.glyph_cache.insert(key, CachedGlyph::empty());
                }
            }

            // --- Use cached glyph ---
            if let Some(cached) = ctx.glyph_cache.get(key)
                && !cached.is_empty()
            {
                let local_x = horizontal_offset + glyph.font_size.mul_add(glyph.x_offset, glyph.x);
                let local_y = vertical_offset
                    + glyph
                        .font_size
                        .mul_add(-glyph.y_offset, run.line_y + glyph.y);

                flush_character_to_mesh(
                    ctx.mesh_buffer,
                    &cached.geometry,
                    req.position,
                    req.rotation,
                    req.fill,
                    local_x,
                    local_y,
                );
            }
        }
    }

    drop(lock); // To avoid clippy lint
}

/// Transforms local glyph geometry to world/screen space and pushes it to the mesh buffer.
fn flush_character_to_mesh(
    target_buffer: &mut crate::render::cache::MeshData,
    source_geometry: &VertexBuffers<Point, u16>,
    screen_origin: Point,
    rotation_radians: f32,
    color: Color,
    local_offset_x: f32,
    local_offset_y: f32,
) {
    let mesh = target_buffer.get_mesh_mut();
    let start_index = mesh.vertices.len() as u32;

    let (sin, cos) = rotation_radians.sin_cos();
    let packed_color = pack(color);

    // Fonts are usually Y-up, screens are Y-down.
    let flip_y = -1.0;

    // Transform every vertex in the glyph
    for vertex in &source_geometry.vertices {
        // Position relative to the word/line start
        let local_x = vertex.x + local_offset_x;
        let local_y = vertex.y.mul_add(flip_y, local_offset_y);

        // Rotate around the text origin
        let rotated_x = local_x.mul_add(cos, -(local_y * sin));
        let rotated_y = local_x.mul_add(sin, local_y * cos);

        // Translate to final screen position
        let final_x = screen_origin.x + rotated_x;
        let final_y = screen_origin.y + rotated_y;

        mesh.vertices.push(SolidVertex2D {
            position: [final_x, final_y],
            color: packed_color,
        });
    }

    // Offset indices to match the new vertex positions in the global buffer
    for index in &source_geometry.indices {
        mesh.indices.push(start_index + *index as u32);
    }
}
