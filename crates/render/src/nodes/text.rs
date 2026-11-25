//! Xử lý kết xuất chữ cho `TextGenerator` Node.

use filmcraft_text::{Align, ParagraphStyle, TextStyle, layout, render};

use crate::graphics::{bounded_raster_size, paint_mask, premul_linear};
use crate::image::Image;

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_text(
    text: &str,
    font_size: f32,
    color: [f32; 4],
    tracking: f32,
    line_spacing: f32,
    w: usize,
    h: usize,
    scale: f32,
) -> Image {
    let mut img = Image::new(w, h);
    let px = (font_size * scale).max(1.0);
    if text.is_empty() || !px.is_finite() {
        return img;
    }
    let st = TextStyle {
        family: filmcraft_text::fonts::DEFAULT_FAMILY.into(),
        style: "Regular".into(),
        size: px,
        tracking: tracking * scale,
        ..Default::default()
    };
    let ps = ParagraphStyle {
        align: Align::Center,
        leading: (line_spacing.max(0.5) - 1.0) * px,
        ..Default::default()
    };
    let l = layout(text, &st, &ps);
    let b = l.bounds;
    if bounded_raster_size(b[2] - b[0], b[3] - b[1]).is_none() {
        return img;
    }
    let vm = filmcraft_text::fonts::face(
        filmcraft_text::resolve(filmcraft_text::fonts::DEFAULT_FAMILY, "Regular").face,
    )
    .metrics(px);
    let bx = w as f32 * 0.5;
    let by = (h as f32 * 0.5 + vm.cap_height * 0.5).round();
    let (m, x0, y0) = render::rasterize(&l, &render::at(bx, by));
    paint_mask(&mut img, &m, x0, y0, premul_linear(color), 1.0);
    img
}
