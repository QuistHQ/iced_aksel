use crate::{font::GeometricFont, render::MeshBuffer};
use ab_glyph::{Font, PxScale, ScaleFont};
use iced_core::{
    Color, Point,
    alignment::{Horizontal, Vertical},
};
use iced_graphics::color::pack; // To convert Color to GPU format
use iced_graphics::mesh::SolidVertex2D; // Needed for MeshBuffer
use lyon::math::point;
use lyon::path::Path;
use lyon::path::builder::PathBuilder;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, FillVertexConstructor, VertexBuffers,
};
use ttf_parser::OutlineBuilder;

// --- Adapter to bridge ttf-parser and lyon ---
struct LyonPathBuilder<'a>(pub &'a mut dyn PathBuilder);

impl<'a> OutlineBuilder for LyonPathBuilder<'a> {
    fn move_to(&mut self, x: f32, y: f32) {
        // Lyon expects attributes for points, we pass empty slice &[]
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

// --- Vertex Constructor ---
struct TextVertexConstructor;

impl FillVertexConstructor<Point> for TextVertexConstructor {
    fn new_vertex(&mut self, vertex: FillVertex) -> Point {
        let p = vertex.position();
        Point::new(p.x, p.y)
    }
}

/// Draws text as a geometric mesh.
#[allow(clippy::too_many_arguments)]
pub fn draw_geometric_text(
    buffer: &mut MeshBuffer,
    content: &str,
    position: Point,
    size_px: f32,
    rotation_rads: f32,
    color: Color,
    font: &GeometricFont,
    horizontal_alignment: Horizontal,
    vertical_alignment: Vertical,
    screen_tolerance: f32,
) {
    if content.is_empty() {
        return;
    }

    let font_layout = &font.layout;
    let font_geometry = &font.geometry;

    // 2. Metrics & Scaling
    let scale = PxScale::from(size_px);
    let scaled_font = font_layout.as_scaled(scale);

    let units_per_em = font_geometry.units_per_em() as f32;
    let geometry_scale = size_px / units_per_em;

    // --- LOD CALCULATION ---
    // Formula: tolerance_in_font_units = (desired_pixel_error / scale_factor)
    // where scale_factor = size_px / units_per_em
    // Therefore: tolerance = (pixel_error * units_per_em) / size_px
    //
    // Safety: Avoid division by zero if text is microscopic
    let safe_size = size_px.max(0.001);
    let lyon_tolerance = (screen_tolerance * units_per_em) / safe_size;

    // 3. Layout Calculation
    let mut width = 0.0;
    let mut last_glyph_id = None;

    for c in content.chars() {
        let glyph_id = font_layout.glyph_id(c);
        if let Some(last) = last_glyph_id {
            width += scaled_font.kern(last, glyph_id);
        }
        width += scaled_font.h_advance(glyph_id);
        last_glyph_id = Some(glyph_id);
    }

    let ascent = scaled_font.ascent();
    let descent = scaled_font.descent();
    let height = ascent - descent;

    // 4. Alignments
    let offset_x = match horizontal_alignment {
        Horizontal::Left => 0.0,
        Horizontal::Center => -width / 2.0,
        Horizontal::Right => -width,
    };

    let offset_y = match vertical_alignment {
        Vertical::Top => ascent,
        Vertical::Center => ascent - (height / 2.0),
        Vertical::Bottom => descent,
    };

    // 5. Tessellation
    let mut geometry: VertexBuffers<Point, u16> = VertexBuffers::new();
    let mut tessellator = FillTessellator::new();

    // APPLY THE CALCULATED TOLERANCE
    let options = FillOptions::default().with_tolerance(lyon_tolerance);

    let mut cursor_x = 0.0;
    last_glyph_id = None;

    for c in content.chars() {
        let glyph_id = font_layout.glyph_id(c);

        if let Some(last) = last_glyph_id {
            cursor_x += scaled_font.kern(last, glyph_id);
        }

        let ttf_glyph_id = ttf_parser::GlyphId(glyph_id.0);

        let mut path_builder = Path::builder();
        let mut bridge = LyonPathBuilder(&mut path_builder);

        if let Some(_) = font_geometry.outline_glyph(ttf_glyph_id, &mut bridge) {
            let path = path_builder.build();

            let _ = tessellator.tessellate_path(
                &path,
                &options,
                &mut BuffersBuilder::new(&mut geometry, TextVertexConstructor),
            );
        }

        flush_char_to_mesh(
            buffer,
            &geometry,
            position,
            rotation_rads,
            color,
            offset_x + cursor_x,
            offset_y,
            geometry_scale,
        );
        geometry.vertices.clear();
        geometry.indices.clear();

        cursor_x += scaled_font.h_advance(glyph_id);
        last_glyph_id = Some(glyph_id);
    }
}

// Helper to push a single character's geometry to the main mesh
fn flush_char_to_mesh(
    target: &mut MeshBuffer,
    source: &VertexBuffers<Point, u16>,
    origin: Point,
    rotation: f32,
    color: Color,
    local_offset_x: f32,
    local_offset_y: f32,
    scale: f32,
) {
    let mesh = target.get_mesh_mut();

    let start_index = mesh.vertices.len() as u32;
    let (sin, cos) = rotation.sin_cos();
    let flip_y = -1.0;
    let packed_color = pack(color);

    for v in &source.vertices {
        // Scale
        let sx = v.x * scale;
        let sy = v.y * scale;

        // Position relative to text origin
        let lx = sx + local_offset_x;
        let ly = (sy * flip_y) + local_offset_y;

        // Rotate
        let rx = lx * cos - ly * sin;
        let ry = lx * sin + ly * cos;

        // Translate to screen
        let final_x = origin.x + rx;
        let final_y = origin.y + ry;

        // Push SolidVertex2D (Required by MeshBuffer)
        mesh.vertices.push(SolidVertex2D {
            position: [final_x, final_y],
            color: packed_color,
        });
    }

    for i in &source.indices {
        mesh.indices.push(start_index + *i as u32);
    }
}
