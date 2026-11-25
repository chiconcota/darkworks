//! Xử lý kết xuất vector cho `ShapeGenerator` Node.

use darkworks_node_graph_core::ShapeType;
use filmcraft_text::raster::{Path, fill};

use crate::graphics::{bounded_raster_size, paint_mask, premul_linear};
use crate::image::Image;

pub(crate) fn render_shape(
    shape_type: ShapeType,
    size: [f32; 2],
    corner_radius: f32,
    color: [f32; 4],
    w: usize,
    h: usize,
    scale_xy: (f32, f32),
) -> Image {
    let linear_col = premul_linear(color);
    if matches!(shape_type, ShapeType::SolidMatte) {
        return Image::filled(w, h, linear_col);
    }
    let mut img = Image::new(w, h);
    let sw = (size[0].abs() * scale_xy.0).max(1.0);
    let sh = (size[1].abs() * scale_xy.1).max(1.0);
    let cx = w as f32 * 0.5;
    let cy = h as f32 * 0.5;
    let (x0, y0, x1, y1) = (cx - sw * 0.5, cy - sh * 0.5, cx + sw * 0.5, cy + sh * 0.5);
    let path = match shape_type {
        ShapeType::Rectangle => Path::rect(x0, y0, x1, y1),
        ShapeType::RoundedRectangle => {
            Path::round_rect(x0, y0, x1, y1, corner_radius * scale_xy.0.min(scale_xy.1))
        }
        ShapeType::Circle => Path::ellipse(cx, cy, sw * 0.5, sh * 0.5),
        ShapeType::SolidMatte => return Image::filled(w, h, linear_col),
    };
    if let Some(bb) = path.bounds() {
        let bx0 = (bb.0.floor() as i32).max(0);
        let by0 = (bb.1.floor() as i32).max(0);
        let bx1 = (bb.2.ceil() as i32).min(w as i32);
        let by1 = (bb.3.ceil() as i32).min(h as i32);
        let mw = (bx1 - bx0).max(0) as f32;
        let mh = (by1 - by0).max(0) as f32;
        if let Some((rw, rh)) = bounded_raster_size(mw, mh) {
            let mask = fill(&path, rw, rh, -bx0 as f32, -by0 as f32);
            paint_mask(&mut img, &mask, bx0, by0, linear_col, 1.0);
        }
    }
    img
}
