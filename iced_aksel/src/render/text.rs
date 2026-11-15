use iced_core::{
    Color, Font, Pixels, Point, Size,
    alignment::{Horizontal, Vertical},
    text::Wrapping,
};
use iced_graphics::text::cosmic_text::{self, Buffer, Metrics};

/// A Text to draw on the screen
pub struct Text {
    pub font: Font,
    pub content: String,
    pub position: Point,
    pub size: Pixels,
    pub rotation: f32,
    pub horizontal_alignment: Horizontal,
    pub vertical_alignment: Vertical,
    pub fill: Color,
    pub tolerance: f32,
    pub line_height: Pixels,
    pub bounds: Size,
    pub wrapping: Wrapping,
}

/// A laid out line of text.
#[derive(Clone)]
pub struct OwnedRun {
    pub line_width: f32,
    pub line_y: f32,
    pub line_height: f32,
    pub glyphs: Vec<cosmic_text::LayoutGlyph>,
}

impl OwnedRun {
    /// Returns the horizontal offset of the line, relative to the anchor of the text.
    pub fn horizontal_offset(&self, alignment: Horizontal) -> f32 {
        match alignment {
            Horizontal::Left => 0.0,
            Horizontal::Center => -self.line_width / 2.0,
            Horizontal::Right => -self.line_width,
        }
    }
}

/// The laid out lines of a [`Text`], relative to its anchor.
pub struct TextLayout {
    pub runs: Vec<OwnedRun>,
    /// The vertical offset of the whole text block, relative to the anchor of the text.
    pub vertical_offset: f32,
}

/// Shapes and lays out a [`Text`].
///
/// Returns `None` if there is nothing to draw.
pub fn layout_text(font_system: &mut cosmic_text::FontSystem, req: &Text) -> Option<TextLayout> {
    if req.content.is_empty() || req.size.0 <= 2.0 {
        return None;
    }

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

    // --- Compute vertical alignment ---
    let min_y = runs.first()?.line_y;
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

    Some(TextLayout {
        runs,
        vertical_offset,
    })
}

/// Returns the origin of a glyph on a line with the given baseline, relative to the line start.
pub fn glyph_position(glyph: &cosmic_text::LayoutGlyph, line_y: f32) -> (f32, f32) {
    (
        glyph.font_size.mul_add(glyph.x_offset, glyph.x),
        glyph.font_size.mul_add(-glyph.y_offset, line_y + glyph.y),
    )
}
