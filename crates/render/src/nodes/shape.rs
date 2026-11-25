//! Xử lý kết xuất vector cho `ShapeGenerator` Node (hỗ trợ Position, Stroke & Feather Soft Mask).

use darkworks_node_graph_core::ShapeType;
use filmcraft_text::raster::{Path, fill};

use crate::effects::gaussian_boxes;
use crate::gpufx::FxOp;
use crate::graphics::{bounded_raster_size, paint_mask, premul_linear};
use crate::image::Image;

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_shape(
    shape_type: ShapeType,
    size: [f32; 2],
    corner_radius: f32,
    color: [f32; 4],
    position: [f32; 2],
    stroke_width: f32,
    stroke_color: [f32; 4],
    feather: f32,
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
    let cx = w as f32 * 0.5 + position[0] * scale_xy.0;
    let cy = h as f32 * 0.5 + position[1] * scale_xy.1;
    let (x0, y0, x1, y1) = (cx - sw * 0.5, cy - sh * 0.5, cx + sw * 0.5, cy + sh * 0.5);

    let make_path = |extra: f32| match shape_type {
        ShapeType::Rectangle => Path::rect(x0 - extra, y0 - extra, x1 + extra, y1 + extra),
        ShapeType::RoundedRectangle => Path::round_rect(
            x0 - extra,
            y0 - extra,
            x1 + extra,
            y1 + extra,
            (corner_radius * scale_xy.0.min(scale_xy.1) + extra).max(0.0),
        ),
        ShapeType::Circle => Path::ellipse(cx, cy, (sw * 0.5 + extra).max(0.5), (sh * 0.5 + extra).max(0.5)),
        ShapeType::SolidMatte => Path::rect(0.0, 0.0, w as f32, h as f32),
    };

    let stroke_px = (stroke_width * scale_xy.0.min(scale_xy.1)).max(0.0);
    if stroke_px > 0.25 && stroke_color[3] > 0.001 {
        let outer = make_path(stroke_px);
        paint_path(&mut img, &outer, premul_linear(stroke_color), w, h);
    }

    let inner = make_path(0.0);
    paint_path(&mut img, &inner, linear_col, w, h);

    let f_px = (feather * scale_xy.0.min(scale_xy.1)).max(0.0);
    if f_px > 0.3 {
        let (rx, ry) = gaussian_boxes(w, h, f_px, f_px);
        FxOp::Gaussian { rx, ry, repeat: false }.apply(&mut img);
    }
    img
}

fn paint_path(img: &mut Image, path: &Path, color: [f32; 4], w: usize, h: usize) {
    if let Some(bb) = path.bounds() {
        let bx0 = (bb.0.floor() as i32).max(0);
        let by0 = (bb.1.floor() as i32).max(0);
        let bx1 = (bb.2.ceil() as i32).min(w as i32);
        let by1 = (bb.3.ceil() as i32).min(h as f32 as i32);
        let mw = (bx1 - bx0).max(0) as f32;
        let mh = (by1 - by0).max(0) as f32;
        if let Some((rw, rh)) = bounded_raster_size(mw, mh) {
            let mask = fill(path, rw, rh, -bx0 as f32, -by0 as f32);
            paint_mask(img, &mask, bx0, by0, color, 1.0);
        }
    }
}
