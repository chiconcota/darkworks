//! Xử lý kết xuất chữ cho `TextGenerator` Node (hỗ trợ Font, Bold/Italic, Align, Position & Stroke).

use darkworks_node_graph_core::TextAlign;
use filmcraft_text::raster::fill;
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
    position: [f32; 2],
    align: TextAlign,
    font_family: &str,
    bold: bool,
    italic: bool,
    stroke_color: [f32; 4],
    stroke_width: f32,
    w: usize,
    h: usize,
    scale: f32,
) -> Image {
    let mut img = Image::new(w, h);
    let px = (font_size * scale).max(1.0);
    if text.is_empty() || !px.is_finite() {
        return img;
    }

    let family = if font_family.trim().is_empty() {
        filmcraft_text::fonts::DEFAULT_FAMILY
    } else {
        font_family.trim()
    };
    let style_name = match (bold, italic) {
        (true, true) => "Bold Italic",
        (true, false) => "Bold",
        (false, true) => "Italic",
        (false, false) => "Regular",
    };

    let st = TextStyle {
        family: family.into(),
        style: style_name.into(),
        size: px,
        tracking: tracking * scale,
        faux_bold: bold,
        faux_italic: italic,
        ..Default::default()
    };
    let text_align = match align {
        TextAlign::Left => Align::Left,
        TextAlign::Center => Align::Center,
        TextAlign::Right => Align::Right,
    };
    let ps = ParagraphStyle {
        align: text_align,
        leading: (line_spacing.max(0.5) - 1.0) * px,
        ..Default::default()
    };

    let l = layout(text, &st, &ps);
    let b = l.bounds;
    if bounded_raster_size(b[2] - b[0], b[3] - b[1]).is_none() {
        return img;
    }
    let vm = filmcraft_text::fonts::face(filmcraft_text::resolve(family, style_name).face).metrics(px);
    let bx = w as f32 * 0.5 + position[0] * scale;
    let by = (h as f32 * 0.5 + vm.cap_height * 0.5 + position[1] * scale).round();
    let xf = render::at(bx, by);

    let stroke_px = (stroke_width * scale).max(0.0);
    if stroke_px > 0.25 && stroke_color[3] > 0.001 {
        let mut outline = render::layout_path(&l, &xf);
        outline.embolden(stroke_px);
        if let Some(bb) = outline.bounds() {
            let sx0 = (bb.0.floor() as i32 - 1).max(0);
            let sy0 = (bb.1.floor() as i32 - 1).max(0);
            let sx1 = (bb.2.ceil() as i32 + 1).min(w as i32);
            let sy1 = (bb.3.ceil() as i32 + 1).min(h as i32);
            let mw = (sx1 - sx0).max(0) as f32;
            let mh = (sy1 - sy0).max(0) as f32;
            if let Some((rw, rh)) = bounded_raster_size(mw, mh) {
                let sm = fill(&outline, rw, rh, -sx0 as f32, -sy0 as f32);
                paint_mask(&mut img, &sm, sx0, sy0, premul_linear(stroke_color), 1.0);
            }
        }
    }

    let (m, x0, y0) = render::rasterize(&l, &xf);
    paint_mask(&mut img, &m, x0, y0, premul_linear(color), 1.0);
    img
}
