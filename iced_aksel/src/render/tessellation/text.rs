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
use ttf_parser::OutlineBuilder;

/// The rendering quality of the vector text.
///
/// This controls the Level of Detail (LOD) by adjusting the error tolerance
/// of the tessellation algorithm. Lower tolerance means more triangles (smoother curves).
///
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Quality {
    /// High detail. Suitable for large text (headings) or cinematic rendering.
    /// (Max pixel error: ~0.2px)
    High,
    /// Balanced detail. Indistinguishable from perfect on most screens.
    /// This is the recommended default.
    /// (Max pixel error: ~0.5px)
    Medium,
    /// Low detail. Optimized for performance when rendering thousands of labels.
    /// Curves may look slightly angular when zoomed in.
    /// (Max pixel error: ~1.5px)
    Low,
    /// Custom tolerance in screen pixels.
    /// - Lower values (e.g., 0.05) = Higher Quality, More Vertices.
    /// - Higher values (e.g., 5.0) = Lower Quality, Fewer Vertices.
    Custom(f32),
}

impl Quality {
    /// Converts the quality setting into a specific pixel tolerance value.
    pub fn to_tolerance(self) -> f32 {
        match self {
            Self::High => 0.2,
            Self::Medium => 0.5,
            Self::Low => 1.5,
            Self::Custom(val) => val.max(0.001), // Prevent division by zero later
        }
    }
}

impl Default for Quality {
    fn default() -> Self {
        Self::Medium
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

// --- Vertex Constructor for Lyon ---
struct TextVertexConstructor;

impl FillVertexConstructor<Point> for TextVertexConstructor {
    fn new_vertex(&mut self, vertex: FillVertex) -> Point {
        let position = vertex.position();
        Point::new(position.x, position.y)
    }
}

/// Draws text as a geometric mesh (triangles) instead of using texture atlases.
///
/// This function performs the following steps:
/// 1. Layouts the text using `ab_glyph` (kerning, advance).
/// 2. Calculates the required tessellation tolerance based on the font size and requested quality.
/// 3. Extracts vector paths for each character using `ttf_parser`.
/// 4. Tessellates those paths into triangles using `lyon`.
/// 5. Transforms (scales, rotates, translates) the vertices to their final screen position.
#[allow(clippy::too_many_arguments)]
pub fn draw_geometric_text(
    buffer: &mut MeshBuffer,
    content: &str,
    position: Point,
    font_size_in_pixels: f32,
    rotation_radians: f32,
    color: Color,
    font: &GeometricFont,
    horizontal_alignment: Horizontal,
    vertical_alignment: Vertical,
    quality: Quality,
) {
    if content.is_empty() {
        return;
    }

    let font_layout = &font.layout;
    let font_geometry = &font.geometry;

    // 1. Setup Metrics & Scaling
    let pixel_scale = PxScale::from(font_size_in_pixels);
    let scaled_font = font_layout.as_scaled(pixel_scale);

    // ttf-parser coordinates are in "Font Units" (integers, e.g., 0 to 2048).
    // We need to scale these down to Screen Pixels.
    let font_units_per_em = font_geometry.units_per_em() as f32;
    let geometry_scale_factor = font_size_in_pixels / font_units_per_em;

    // 2. Calculate Level of Detail (LOD)
    // Lyon's tolerance is in the source coordinate system (Font Units).
    // We want the error to be fixed in Screen Pixels.
    // Formula: Tolerance_Units = (Desired_Pixel_Error * Units_Per_Em) / Font_Size_Px
    let desired_pixel_error = quality.to_tolerance();
    // Safety: Clamp size to avoid division by zero for microscopic text
    let safe_font_size = font_size_in_pixels.max(0.001);

    let tessellation_tolerance = (desired_pixel_error * font_units_per_em) / safe_font_size;

    // 3. Measure Text Dimensions for Alignment
    let mut text_width = 0.0;
    let mut last_glyph_id = None;

    for character in content.chars() {
        let glyph_id = font_layout.glyph_id(character);
        if let Some(last) = last_glyph_id {
            text_width += scaled_font.kern(last, glyph_id);
        }
        text_width += scaled_font.h_advance(glyph_id);
        last_glyph_id = Some(glyph_id);
    }

    let ascent = scaled_font.ascent();
    let descent = scaled_font.descent();
    let text_height = ascent - descent;

    // 4. Calculate Alignment Offsets
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

    // 5. Tessellation Loop
    let mut temporary_geometry: VertexBuffers<Point, u16> = VertexBuffers::new();
    let mut tessellator = FillTessellator::new();
    let fill_options = FillOptions::default().with_tolerance(tessellation_tolerance);

    let mut cursor_x = 0.0;
    last_glyph_id = None;

    for character in content.chars() {
        let glyph_id = font_layout.glyph_id(character);

        // Apply kerning (spacing between specific pairs like 'A' and 'V')
        if let Some(last) = last_glyph_id {
            cursor_x += scaled_font.kern(last, glyph_id);
        }

        // ttf-parser uses a u16 ID wrapper
        let ttf_glyph_id = ttf_parser::GlyphId(glyph_id.0);

        let mut path_builder = Path::builder();
        let mut builder_adapter = LyonPathBuilder(&mut path_builder);

        // Extract the vector path from the font file
        if let Some(_) = font_geometry.outline_glyph(ttf_glyph_id, &mut builder_adapter) {
            let path = path_builder.build();

            // Convert the curves into triangles
            let _ = tessellator.tessellate_path(
                &path,
                &fill_options,
                &mut BuffersBuilder::new(&mut temporary_geometry, TextVertexConstructor),
            );
        }

        // Push the generated triangles to the main buffer immediately
        // (This allows us to transform each character individually based on its cursor position)
        flush_character_to_mesh(
            buffer,
            &temporary_geometry,
            position,
            rotation_radians,
            color,
            alignment_offset_x + cursor_x,
            alignment_offset_y,
            geometry_scale_factor,
        );

        // Clear for the next character
        temporary_geometry.vertices.clear();
        temporary_geometry.indices.clear();

        cursor_x += scaled_font.h_advance(glyph_id);
        last_glyph_id = Some(glyph_id);
    }
}

/// Helper function to transform and append a single character's geometry to the main mesh buffer.
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

    // Font coordinates usually have Y going UP. Screen coordinates have Y going DOWN.
    // We flip Y here to correct the orientation.
    let flip_y = -1.0;

    for vertex in &source_geometry.vertices {
        // 1. Scale (Font Units -> Screen Pixels)
        let scaled_x = vertex.x * scale_factor;
        let scaled_y = vertex.y * scale_factor;

        // 2. Local Position (Apply cursor position and alignment)
        let local_x = scaled_x + local_offset_x;
        let local_y = (scaled_y * flip_y) + local_offset_y;

        // 3. Rotation (Around the alignment anchor)
        let rotated_x = local_x * cos - local_y * sin;
        let rotated_y = local_x * sin + local_y * cos;

        // 4. Translation (Move to final screen coordinates)
        let final_x = screen_origin.x + rotated_x;
        let final_y = screen_origin.y + rotated_y;

        mesh.vertices.push(SolidVertex2D {
            position: [final_x, final_y],
            color: packed_color,
        });
    }

    // Append indices, offsetting them by the current vertex count
    for index in &source_geometry.indices {
        mesh.indices.push(start_index + *index as u32);
    }
}
