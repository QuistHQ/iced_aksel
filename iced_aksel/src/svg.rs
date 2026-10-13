//! Exporting charts as SVG documents.
//!
//! A [`Chart`](crate::Chart) can be written to an SVG document with
//! [`Chart::to_svg`](crate::Chart::to_svg). The export doesn't need a window or a renderer, so it
//! can be called straight from your `update` logic.
//!
//! Text is exported as glyph outlines, so the document looks the same everywhere and doesn't
//! depend on the fonts installed on the machine that displays or prints it.
//!
//! # Limitations
//! * Glyphs without an outline (e.g. bitmap emoji) are skipped.
//! * Shadows of label badges are not exported.
//! * Markers and the debug overlay are interactive and therefore not exported.

use std::fmt::Write;
use std::marker::PhantomData;

use iced_core::{
    Color, Font, Point, Rectangle, Size,
    alignment::{Horizontal, Vertical},
    border,
    renderer::Quad,
};
use iced_graphics::text::{
    cosmic_text::skrifa::{
        FontRef, GlyphId16, MetadataProvider,
        outline::{DrawSettings, OutlinePen},
        prelude::{LocationRef, Size as FontSize},
    },
    font_system,
};
use lyon_path::Event;

use crate::render::{
    Primitive, RenderCache, Surface,
    cache::primitive_path,
    text::{Text, glyph_position, layout_text},
};
use crate::stroke::{ResolvedStroke, StrokeStyle};

/// The id of the clip path of the plot area.
const PLOT_CLIP_ID: &str = "aksel-plot";

/// Options for exporting a chart as an SVG document.
///
/// # Example
///
/// ```rust
/// use iced_aksel::svg::Options;
/// use iced::Color;
///
/// let options = Options::default().background(Color::WHITE);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Options {
    /// The background color of the document. Transparent if `None` (default).
    pub background: Option<Color>,
    /// The font used for text that doesn't specify one.
    ///
    /// On screen this is the default font of the application, which is not known when exporting.
    pub default_font: Font,
}

impl Options {
    /// Sets the background color of the document.
    pub const fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    /// Sets the font used for text that doesn't specify one.
    pub const fn default_font(mut self, font: Font) -> Self {
        self.default_font = font;
        self
    }
}

/// An SVG document a chart is drawn onto.
pub struct Document<Renderer: crate::Renderer> {
    options: Options,
    /// Everything outside of the plot area: ticks, labels and spines.
    decorations: String,
    /// The content of the plot area: grid lines and data layers.
    pub cache: RenderCache<Renderer>,
    renderer: PhantomData<Renderer>,
}

impl<Renderer: crate::Renderer> Document<Renderer> {
    pub const fn new(options: Options) -> Self {
        Self {
            options,
            decorations: String::new(),
            cache: RenderCache::new_svg(),
            renderer: PhantomData,
        }
    }

    /// Writes the document, clipping the content of the plot area to `plot_bounds`.
    pub fn finish(self, size: Size, plot_bounds: Rectangle) -> String {
        let mut out = String::new();

        out.push_str("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"");
        write_number(&mut out, size.width);
        out.push_str("\" height=\"");
        write_number(&mut out, size.height);
        out.push_str("\" viewBox=\"0 0 ");
        write_number(&mut out, size.width);
        out.push(' ');
        write_number(&mut out, size.height);
        out.push_str("\">\n");

        if let Some(background) = self.options.background {
            write_rectangle(
                &mut out,
                Rectangle::with_size(size),
                border::Radius::default(),
                Some(background),
                None,
            );
        }

        out.push_str(&self.decorations);

        let _ = write!(out, "<defs><clipPath id=\"{PLOT_CLIP_ID}\">");
        out.push_str("<rect");
        write_bounds(&mut out, plot_bounds);
        out.push_str("/></clipPath></defs>\n");
        let _ = writeln!(out, "<g clip-path=\"url(#{PLOT_CLIP_ID})\">");

        if let RenderCache::Svg(primitives) = &self.cache {
            for primitive in primitives {
                write_primitive(&mut out, primitive);
            }
        }

        out.push_str("</g>\n</svg>\n");
        out
    }
}

impl<Renderer: crate::Renderer> Surface for Document<Renderer> {
    type Renderer = Renderer;

    fn default_font(&self) -> Font {
        self.options.default_font
    }

    fn fill_quad(&mut self, quad: Quad, color: Color) {
        let Quad { bounds, border, .. } = quad;

        write_rectangle(
            &mut self.decorations,
            bounds,
            border.radius,
            Some(color),
            None,
        );

        // The border of a quad is drawn on the inside of its bounds
        if border.width > 0.0 {
            let inset = border.width / 2.0;
            let shrink = |radius: f32| (radius - inset).max(0.0);

            write_rectangle(
                &mut self.decorations,
                bounds.shrink(inset),
                border::Radius {
                    top_left: shrink(border.radius.top_left),
                    top_right: shrink(border.radius.top_right),
                    bottom_right: shrink(border.radius.bottom_right),
                    bottom_left: shrink(border.radius.bottom_left),
                },
                None,
                Some(&ResolvedStroke {
                    fill: border.color,
                    thickness: border.width,
                    style: StrokeStyle::Solid,
                }),
            );
        }
    }

    fn fill_text(
        &mut self,
        text: iced_core::Text<String, Font>,
        position: Point,
        color: Color,
        _clip_bounds: Rectangle,
    ) {
        write_text(
            &mut self.decorations,
            &Text {
                font: text.font,
                content: text.content,
                position,
                size: text.size,
                rotation: 0.0,
                horizontal_alignment: Horizontal::Left,
                vertical_alignment: Vertical::Top,
                fill: color,
                tolerance: 0.0,
                line_height: text.line_height.to_absolute(text.size),
                bounds: text.bounds,
                wrapping: text.wrapping,
            },
        );
    }

    // The decorations are only ever clipped to the bounds of the whole chart
    fn start_layer(&mut self, _bounds: Rectangle) {}

    fn end_layer(&mut self) {}

    fn add_primitive(&mut self, primitive: Primitive) {
        self.cache.add_primitive(primitive);
    }
}

fn write_primitive(out: &mut String, primitive: &Primitive) {
    if let Primitive::Text {
        font,
        content,
        position,
        size,
        rotation,
        horizontal_alignment,
        vertical_alignment,
        fill,
        line_height,
        bounds,
        wrapping,
        ..
    } = primitive
    {
        write_text(
            out,
            &Text {
                font: *font,
                content: content.clone(),
                position: *position,
                size: *size,
                rotation: rotation.0,
                horizontal_alignment: *horizontal_alignment,
                vertical_alignment: *vertical_alignment,
                fill: *fill,
                tolerance: 0.0,
                line_height: line_height.to_absolute(*size),
                bounds: *bounds,
                wrapping: *wrapping,
            },
        );
        return;
    }

    let Some((path, fill, stroke)) = primitive_path(primitive) else {
        return;
    };

    // Lines have no area to fill
    let fill = match primitive {
        Primitive::Line { .. }
        | Primitive::HorizontalLine { .. }
        | Primitive::VerticalLine { .. } => None,
        _ => fill,
    };

    let mut data = String::new();
    for event in path.raw() {
        match event {
            Event::Begin { at } => write_command(&mut data, 'M', &[at.x, at.y]),
            Event::Line { to, .. } => write_command(&mut data, 'L', &[to.x, to.y]),
            Event::Quadratic { ctrl, to, .. } => {
                write_command(&mut data, 'Q', &[ctrl.x, ctrl.y, to.x, to.y]);
            }
            Event::Cubic {
                ctrl1, ctrl2, to, ..
            } => write_command(
                &mut data,
                'C',
                &[ctrl1.x, ctrl1.y, ctrl2.x, ctrl2.y, to.x, to.y],
            ),
            Event::End { close, .. } => {
                if close {
                    data.push('Z');
                }
            }
        }
    }

    write_path(out, &data, fill, stroke);
}

/// Writes a text as the outlines of its glyphs.
fn write_text(out: &mut String, text: &Text) {
    let mut lock = font_system().write().expect("Failed to get font-system");
    let font_system = lock.raw();

    let Some(layout) = layout_text(font_system, text) else {
        return;
    };

    let (sin, cos) = text.rotation.sin_cos();
    let mut data = String::new();

    for run in &layout.runs {
        let horizontal_offset = run.horizontal_offset(text.horizontal_alignment);

        for glyph in &run.glyphs {
            // Find the chosen face index in fontdb (important for TTC)
            let Some(face_index) = font_system.db().face(glyph.font_id).map(|face| face.index)
            else {
                continue;
            };
            let Some(font) = font_system.get_font(glyph.font_id, glyph.font_weight) else {
                continue;
            };
            let Ok(font_ref) = FontRef::from_index(font.data(), face_index) else {
                continue;
            };
            // Missing outline (e.g. Bitmap emoji)
            let Some(outline) = font_ref
                .outline_glyphs()
                .get(GlyphId16::new(glyph.glyph_id).into())
            else {
                continue;
            };

            let (glyph_x, glyph_y) = glyph_position(glyph, run.line_y);
            let mut pen = GlyphPen {
                data: &mut data,
                origin: text.position,
                offset_x: horizontal_offset + glyph_x,
                offset_y: layout.vertical_offset + glyph_y,
                sin,
                cos,
            };

            let settings =
                DrawSettings::unhinted(FontSize::new(glyph.font_size), LocationRef::default());
            let _ = outline.draw(settings, &mut pen);
        }
    }

    drop(lock);

    write_path(out, &data, Some(text.fill), None);
}

/// Writes the outline of a glyph as path data, placed at its final position in the document.
struct GlyphPen<'a> {
    data: &'a mut String,
    /// The anchor of the text, which it is rotated around.
    origin: Point,
    offset_x: f32,
    offset_y: f32,
    sin: f32,
    cos: f32,
}

impl GlyphPen<'_> {
    fn transform(&self, x: f32, y: f32) -> [f32; 2] {
        // Fonts are Y-up, documents are Y-down
        let local_x = x + self.offset_x;
        let local_y = self.offset_y - y;

        [
            self.origin.x + local_x.mul_add(self.cos, -(local_y * self.sin)),
            self.origin.y + local_x.mul_add(self.sin, local_y * self.cos),
        ]
    }
}

impl OutlinePen for GlyphPen<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        let to = self.transform(x, y);
        write_command(self.data, 'M', &to);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let to = self.transform(x, y);
        write_command(self.data, 'L', &to);
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        let [cx0, cy0] = self.transform(cx0, cy0);
        let [x, y] = self.transform(x, y);
        write_command(self.data, 'Q', &[cx0, cy0, x, y]);
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        let [cx0, cy0] = self.transform(cx0, cy0);
        let [cx1, cy1] = self.transform(cx1, cy1);
        let [x, y] = self.transform(x, y);
        write_command(self.data, 'C', &[cx0, cy0, cx1, cy1, x, y]);
    }

    fn close(&mut self) {
        self.data.push('Z');
    }
}

/// Writes a rectangle with optionally rounded corners.
fn write_rectangle(
    out: &mut String,
    bounds: Rectangle,
    radius: border::Radius,
    fill: Option<Color>,
    stroke: Option<&ResolvedStroke>,
) {
    if bounds.width <= 0.0 || bounds.height <= 0.0 || !is_visible(fill, stroke) {
        return;
    }

    let max_radius = bounds.width.min(bounds.height) / 2.0;
    let [top_left, top_right, bottom_right, bottom_left] = [
        radius.top_left,
        radius.top_right,
        radius.bottom_right,
        radius.bottom_left,
    ]
    .map(|radius| radius.clamp(0.0, max_radius));

    if top_left + top_right + bottom_right + bottom_left <= 0.0 {
        out.push_str("<rect");
        write_bounds(out, bounds);
        write_paint(out, fill, stroke);
        out.push_str("/>\n");
        return;
    }

    let Rectangle {
        x,
        y,
        width,
        height,
    } = bounds;
    let (right, bottom) = (x + width, y + height);

    let mut data = String::new();
    let corner = |data: &mut String, radius: f32, to: [f32; 2]| {
        if radius > 0.0 {
            write_command(data, 'A', &[radius, radius, 0.0, 0.0, 1.0, to[0], to[1]]);
        }
    };

    write_command(&mut data, 'M', &[x + top_left, y]);
    write_command(&mut data, 'L', &[right - top_right, y]);
    corner(&mut data, top_right, [right, y + top_right]);
    write_command(&mut data, 'L', &[right, bottom - bottom_right]);
    corner(&mut data, bottom_right, [right - bottom_right, bottom]);
    write_command(&mut data, 'L', &[x + bottom_left, bottom]);
    corner(&mut data, bottom_left, [x, bottom - bottom_left]);
    write_command(&mut data, 'L', &[x, y + top_left]);
    corner(&mut data, top_left, [x + top_left, y]);
    data.push('Z');

    write_path(out, &data, fill, stroke);
}

fn write_path(out: &mut String, data: &str, fill: Option<Color>, stroke: Option<&ResolvedStroke>) {
    if data.is_empty() || !is_visible(fill, stroke) {
        return;
    }

    out.push_str("<path d=\"");
    out.push_str(data);
    out.push('"');
    write_paint(out, fill, stroke);
    out.push_str("/>\n");
}

/// Returns true if the given fill and stroke leave a visible mark.
fn is_visible(fill: Option<Color>, stroke: Option<&ResolvedStroke>) -> bool {
    fill.is_some_and(|fill| fill.a > 0.0)
        || stroke.is_some_and(|stroke| stroke.thickness > 0.0 && stroke.fill.a > 0.0)
}

fn write_bounds(out: &mut String, bounds: Rectangle) {
    for (name, value) in [
        ("x", bounds.x),
        ("y", bounds.y),
        ("width", bounds.width),
        ("height", bounds.height),
    ] {
        let _ = write!(out, " {name}=\"");
        write_number(out, value);
        out.push('"');
    }
}

/// Writes the fill and stroke attributes of an element.
fn write_paint(out: &mut String, fill: Option<Color>, stroke: Option<&ResolvedStroke>) {
    match fill {
        Some(fill) if fill.a > 0.0 => write_color(out, "fill", fill),
        _ => out.push_str(" fill=\"none\""),
    }

    let Some(stroke) = stroke.filter(|stroke| stroke.thickness > 0.0 && stroke.fill.a > 0.0) else {
        return;
    };

    write_color(out, "stroke", stroke.fill);
    out.push_str(" stroke-width=\"");
    write_number(out, stroke.thickness);
    out.push('"');

    match stroke.style {
        StrokeStyle::Solid => {}
        StrokeStyle::Dashed { dash, gap } => {
            out.push_str(" stroke-dasharray=\"");
            write_number(out, dash);
            out.push(' ');
            write_number(out, gap);
            out.push('"');
        }
        StrokeStyle::Dotted { gap } => {
            out.push_str(" stroke-dasharray=\"0 ");
            write_number(out, gap);
            out.push_str("\" stroke-linecap=\"round\"");
        }
    }
}

fn write_color(out: &mut String, attribute: &str, color: Color) {
    let [r, g, b, _] = color.into_rgba8();
    let _ = write!(out, " {attribute}=\"#{r:02x}{g:02x}{b:02x}\"");

    if color.a < 1.0 {
        let _ = write!(out, " {attribute}-opacity=\"");
        write_number(out, color.a);
        out.push('"');
    }
}

/// Writes a path command followed by its arguments.
fn write_command(data: &mut String, command: char, arguments: &[f32]) {
    data.push(command);
    for (i, argument) in arguments.iter().enumerate() {
        if i > 0 {
            data.push(' ');
        }
        write_number(data, *argument);
    }
}

/// Writes a number with up to three decimals.
fn write_number(out: &mut String, value: f32) {
    if !value.is_finite() {
        out.push('0');
        return;
    }

    let start = out.len();
    let _ = write!(out, "{value:.3}");

    let trimmed = out[start..]
        .trim_end_matches('0')
        .trim_end_matches('.')
        .len();
    out.truncate(start + trimmed);

    if &out[start..] == "-0" {
        out.replace_range(start.., "0");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Axis, Chart, Measure, Plot, PlotData, PlotPoint, State, Stroke, axis::Position,
        scale::Linear, shape,
    };
    use iced_core::Theme;

    #[derive(Debug, Clone)]
    enum Message {}

    struct Shapes;

    impl PlotData<f64, Message> for Shapes {
        fn draw(&self, plot: &mut Plot<f64, Message>, _theme: &Theme) {
            plot.render(
                shape::Rectangle::corners(PlotPoint::new(2.0, 2.0), PlotPoint::new(4.0, 6.0))
                    .fill(Color::from_rgb8(0x11, 0x22, 0x33)),
            );
            plot.render(shape::Polyline::new(
                vec![PlotPoint::new(0.0, 0.0), PlotPoint::new(10.0, 10.0)],
                Stroke::with_style(
                    Color::from_rgba8(0xaa, 0xbb, 0xcc, 0.5),
                    Measure::Screen(2.0),
                    StrokeStyle::Dashed {
                        dash: 6.0,
                        gap: 3.0,
                    },
                ),
            ));
            plot.render(shape::Label::new("Label", PlotPoint::new(5.0, 5.0)));
        }
    }

    fn export(options: Options) -> String {
        let mut state: State<&str, f64> = State::new();
        state.set_axis(
            "x",
            Axis::new(Linear::new(0.0, 10.0), Position::Bottom).with_thickness(20.0),
        );
        state.set_axis(
            "y",
            Axis::new(Linear::new(0.0, 10.0), Position::Left).with_thickness(20.0),
        );

        let chart: Chart<_, _, Message> = Chart::new(&state).plot_data(&Shapes, "x", "y");
        chart.to_svg(Size::new(220.0, 120.0), &Theme::Light, options)
    }

    fn number(value: f32) -> String {
        let mut out = String::new();
        write_number(&mut out, value);
        out
    }

    #[test]
    fn numbers_are_compact() {
        assert_eq!(number(1.0), "1");
        assert_eq!(number(100.0), "100");
        assert_eq!(number(1.5), "1.5");
        assert_eq!(number(-0.25), "-0.25");
        assert_eq!(number(0.12345), "0.123");
        assert_eq!(number(-0.0001), "0");
        assert_eq!(number(f32::NAN), "0");
        assert_eq!(number(f32::INFINITY), "0");
    }

    #[test]
    fn document_has_the_requested_size() {
        let document = export(Options::default());

        assert!(document.starts_with(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"220\" height=\"120\" viewBox=\"0 0 220 120\">\n"
        ));
        assert!(document.ends_with("</g>\n</svg>\n"));
    }

    #[test]
    fn background_is_transparent_by_default() {
        let background = "<rect x=\"0\" y=\"0\" width=\"220\" height=\"120\" fill=\"#ffffff\"/>";

        assert!(!export(Options::default()).contains(background));
        assert!(export(Options::default().background(Color::WHITE)).contains(background));
    }

    #[test]
    fn plot_area_is_clipped() {
        let document = export(Options::default());

        // The axes take 20 pixels on the left and at the bottom
        assert!(document.contains(
            "<clipPath id=\"aksel-plot\"><rect x=\"20\" y=\"0\" width=\"200\" height=\"100\"/></clipPath>"
        ));
        assert!(document.contains("<g clip-path=\"url(#aksel-plot)\">"));
    }

    #[test]
    fn shapes_are_placed_in_screen_space() {
        let document = export(Options::default());

        assert!(document.contains("<path d=\"M60 40L100 40L100 80L60 80Z\" fill=\"#112233\"/>"));
        assert!(document.contains(
            "<path d=\"M20 100L220 0\" fill=\"none\" stroke=\"#aabbcc\" stroke-opacity=\"0.5\" \
             stroke-width=\"2\" stroke-dasharray=\"6 3\"/>"
        ));
    }

    #[test]
    fn text_is_never_exported_as_text() {
        assert!(!export(Options::default()).contains("<text"));
    }

    #[test]
    fn rounded_rectangles_use_arcs() {
        let mut out = String::new();
        write_rectangle(
            &mut out,
            Rectangle::new(Point::new(0.0, 0.0), Size::new(20.0, 10.0)),
            border::Radius::new(2.0),
            Some(Color::BLACK),
            None,
        );

        assert_eq!(
            out,
            "<path d=\"M2 0L18 0A2 2 0 0 1 20 2L20 8A2 2 0 0 1 18 10L2 10A2 2 0 0 1 0 8L0 2\
             A2 2 0 0 1 2 0Z\" fill=\"#000000\"/>\n"
        );
    }
}
