use crate::render::MeshBuffer;
use crate::render::text::GeometricFont;
use ab_glyph::{Font, PxScale, ScaleFont};
use iced_core::{
    Color, Point,
    alignment::{Horizontal, Vertical},
};
use iced_graphics::color::pack;
use iced_graphics::mesh::SolidVertex2D;
use lyon::math::point;
use lyon::path::Path;
use lyon::path::builder::PathBuilder;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, FillVertexConstructor, VertexBuffers,
};
use std::collections::HashMap;
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
pub struct CacheKey {
    pub font_id: usize,
    pub glyph_id: u16,
}

/// A safe wrapper around the glyph cache to enforce correct keying.
pub struct TextTessellationCache {
    map: HashMap<CacheKey, CachedGlyph>,
}

impl TextTessellationCache {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn get(&self, key: CacheKey) -> Option<&CachedGlyph> {
        self.map.get(&key)
    }

    pub fn insert(&mut self, key: CacheKey, glyph: CachedGlyph) {
        self.map.insert(key, glyph);
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }
}

impl Default for TextTessellationCache {
    fn default() -> Self {
        Self::new()
    }
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

/// Draws text as a geometric mesh (triangles).
#[allow(clippy::too_many_arguments)]
pub fn draw_geometric_text(
    buffer: &mut MeshBuffer,
    content: &str,
    position: Point,
    font_size_in_pixels: f32,
    rotation_radians: f32,
    color: Color,
    font: &GeometricFont,
    font_id: usize,
    horizontal_alignment: Horizontal,
    vertical_alignment: Vertical,
    quality: Quality,
    quality_multiplier: f32,
    scratch_geometry: &mut VertexBuffers<Point, u16>,
    tessellator: &mut FillTessellator,
    glyph_cache: &mut TextTessellationCache,
) {
    if content.is_empty() {
        return;
    }

    let font_layout = &font.layout;
    let font_geometry = &font.geometry;

    let pixel_scale = PxScale::from(font_size_in_pixels);
    let scaled_font = font_layout.as_scaled(pixel_scale);
    let font_units_per_em = font_geometry.units_per_em() as f32;
    let geometry_scale_factor = font_size_in_pixels / font_units_per_em;

    let base_error = quality.to_tolerance();
    let desired_pixel_error = base_error / quality_multiplier.max(0.1);
    let safe_font_size = font_size_in_pixels.max(0.001);

    let tessellation_tolerance = (desired_pixel_error * font_units_per_em) / safe_font_size;
    let spacing_multiplier = 1.2;

    let mut text_width = 0.0;
    let mut last_glyph_id = None;

    for character in content.chars() {
        let glyph_id = font_layout.glyph_id(character);
        if let Some(last) = last_glyph_id {
            text_width += scaled_font.kern(last, glyph_id);
        }
        text_width += scaled_font.h_advance(glyph_id) * spacing_multiplier;
        last_glyph_id = Some(glyph_id);
    }

    let ascent = scaled_font.ascent();
    let descent = scaled_font.descent();
    let text_height = ascent - descent;

    let alignment_offset_x = match horizontal_alignment {
        Horizontal::Left => 0.0,
        Horizontal::Center => -text_width / 2.0,
        Horizontal::Right => -text_width,
    };

    let alignment_offset_y = match vertical_alignment {
        Vertical::Top => ascent,
        Vertical::Center => ascent - (text_height / 2.0),
        Vertical::Bottom => descent,
    };

    let mut cursor_x = 0.0;
    last_glyph_id = None;
    let fill_options = FillOptions::default().with_tolerance(tessellation_tolerance);

    for character in content.chars() {
        let glyph_id = font_layout.glyph_id(character);
        let glyph_index = glyph_id.0;

        let cache_key = CacheKey {
            font_id,
            glyph_id: glyph_index,
        };

        if let Some(last) = last_glyph_id {
            cursor_x += scaled_font.kern(last, glyph_id);
        }

        let needs_update = if let Some(cached) = glyph_cache.get(cache_key) {
            cached.tolerance > tessellation_tolerance + 0.0001
        } else {
            true
        };

        if needs_update {
            scratch_geometry.vertices.clear();
            scratch_geometry.indices.clear();

            let ttf_glyph_id = ttf_parser::GlyphId(glyph_index);
            let mut path_builder = Path::builder();
            let mut builder_adapter = LyonPathBuilder(&mut path_builder);

            if let Some(_) = font_geometry.outline_glyph(ttf_glyph_id, &mut builder_adapter) {
                let path = path_builder.build();
                let _ = tessellator.tessellate_path(
                    &path,
                    &fill_options,
                    &mut BuffersBuilder::new(scratch_geometry, TextVertexConstructor),
                );
            }

            glyph_cache.insert(
                cache_key,
                CachedGlyph {
                    geometry: scratch_geometry.clone(),
                    tolerance: tessellation_tolerance,
                },
            );
        }

        if let Some(cached_glyph) = glyph_cache.get(cache_key) {
            flush_character_to_mesh(
                buffer,
                &cached_glyph.geometry,
                position,
                rotation_radians,
                color,
                alignment_offset_x + cursor_x,
                alignment_offset_y,
                geometry_scale_factor,
            );
        }

        cursor_x += scaled_font.h_advance(glyph_id) * spacing_multiplier;
        last_glyph_id = Some(glyph_id);
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
