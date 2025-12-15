use crate::render::MeshBuffer;
use iced_core::{Color, Point, Vector};
use iced_graphics::{color::pack, mesh::SolidVertex2D};

#[inline]
pub fn draw_line_segment(buffer: &mut MeshBuffer, p1: Point, p2: Point, width: f32, color: Color) {
    let c = pack(color);
    let mesh = buffer.get_mesh_mut();
    let start = mesh.vertices.len() as u32;

    let half_width = width / 2.0;
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    let len = (dx * dx + dy * dy).sqrt();
    let (dx, dy) = if len < 1e-6 {
        (1.0, 0.0)
    } else {
        (dx / len, dy / len)
    };

    let nx = -dy * half_width;
    let ny = dx * half_width;

    let c1 = Point::new(p1.x + nx, p1.y + ny);
    let c2 = Point::new(p1.x - nx, p1.y - ny);
    let c3 = Point::new(p2.x + nx, p2.y + ny);
    let c4 = Point::new(p2.x - nx, p2.y - ny);

    mesh.vertices.extend_from_slice(&[
        SolidVertex2D {
            position: [c1.x, c1.y],
            color: c,
        },
        SolidVertex2D {
            position: [c3.x, c3.y],
            color: c,
        },
        SolidVertex2D {
            position: [c2.x, c2.y],
            color: c,
        },
        SolidVertex2D {
            position: [c4.x, c4.y],
            color: c,
        },
    ]);

    mesh.indices
        .extend_from_slice(&[start, start + 1, start + 2, start + 1, start + 2, start + 3]);
}

#[inline]
pub fn draw_arrowhead(
    buffer: &mut MeshBuffer,
    tip: Point,
    direction: Vector,
    width: f32,
    size_multiplier: f32,
    color: Color,
) {
    let c = pack(color);
    let mesh = buffer.get_mesh_mut();
    let start = mesh.vertices.len() as u32;

    let arrow_len = width * size_multiplier;
    let arrow_width = width * size_multiplier * 0.8;

    let base_center = tip - direction * arrow_len;
    let normal = Vector::new(-direction.y, direction.x) * (arrow_width / 2.0);
    let wing1 = base_center + normal;
    let wing2 = base_center - normal;

    mesh.vertices.extend_from_slice(&[
        SolidVertex2D {
            position: [tip.x, tip.y],
            color: c,
        },
        SolidVertex2D {
            position: [wing1.x, wing1.y],
            color: c,
        },
        SolidVertex2D {
            position: [wing2.x, wing2.y],
            color: c,
        },
    ]);

    mesh.indices
        .extend_from_slice(&[start, start + 1, start + 2]);
}
