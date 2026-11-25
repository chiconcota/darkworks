//! Xử lý biến hình, hòa trộn & chuyển cảnh cho `Transform2D`, `Blend` và `Transition` Nodes.

use darkworks_node_graph_core::{BlendMode, TransitionType};
use filmcraft_geom::{Affine, Vec2};
use rayon::prelude::*;

use crate::blend::{Blend, composite};
use crate::gpufx::FxOp;
use crate::image::Image;

#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_transform(
    img: &mut Image,
    position: [f32; 2],
    scale: [f32; 2],
    rotation_deg: f32,
    anchor: [f32; 2],
    opacity: f32,
    flip_h: bool,
    flip_v: bool,
    scale_xy: (f32, f32),
) -> Image {
    if flip_h {
        FxOp::HFlip.apply(img);
    }
    if flip_v {
        FxOp::VFlip.apply(img);
    }
    let (w, h) = (img.w, img.h);
    let px_pos = Vec2::new(
        w as f64 * 0.5 + position[0] as f64 * scale_xy.0 as f64,
        h as f64 * 0.5 + position[1] as f64 * scale_xy.1 as f64,
    );
    let px_anchor = Vec2::new(w as f64 * anchor[0] as f64, h as f64 * anchor[1] as f64);
    let m = Affine::motion(
        px_pos,
        Vec2::new(scale[0] as f64, scale[1] as f64),
        rotation_deg as f64,
        px_anchor,
    );
    let mut out = img.transformed(w, h, &m);
    if (opacity - 1.0).abs() > 1e-4 {
        out.scale_alpha(opacity.clamp(0.0, 1.0));
    }
    out
}

pub(crate) fn apply_mask_alpha(fg: &mut Image, mask: &Image) {
    let w = fg.w.min(mask.w);
    let h = fg.h.min(mask.h);
    for y in 0..h {
        for x in 0..w {
            let fi = (y * fg.w + x) * 4;
            let mi = (y * mask.w + x) * 4;
            let m = mask.px[mi + 3].clamp(0.0, 1.0);
            for k in 0..4 {
                fg.px[fi + k] *= m;
            }
        }
    }
}

pub(crate) fn apply_transition(a: &Image, b: &Image, ty: TransitionType, progress: f32, feather: f32) -> Image {
    let (w, h) = (a.w, a.h);
    let mut out = Image::new(w, h);
    let p = progress.clamp(0.0, 1.0);
    let f = feather.max(0.001);
    match ty {
        TransitionType::CrossDissolve => {
            out.px.par_chunks_mut(4).zip(a.px.par_chunks(4).zip(b.px.par_chunks(4))).for_each(|(o, (pa, pb))| {
                for k in 0..4 {
                    o[k] = pa[k] * (1.0 - p) + pb[k] * p;
                }
            });
        }
        TransitionType::LinearWipe => {
            out.px.par_chunks_mut(w * 4).enumerate().for_each(|(y, row)| {
                for x in 0..w {
                    let u = x as f32 / w.max(1) as f32;
                    let mix = ((p - u) / f + 0.5).clamp(0.0, 1.0);
                    let i = (y * w + x) * 4;
                    for k in 0..4 {
                        row[x * 4 + k] = a.px[i + k] * (1.0 - mix) + b.px[i + k] * mix;
                    }
                }
            });
        }
        TransitionType::Iris => {
            let (cx, cy) = (w as f32 * 0.5, h as f32 * 0.5);
            let max_r = cx.hypot(cy).max(1.0);
            out.px.par_chunks_mut(w * 4).enumerate().for_each(|(y, row)| {
                for x in 0..w {
                    let dist = (x as f32 - cx).hypot(y as f32 - cy) / max_r;
                    let mix = ((p - dist) / f + 0.5).clamp(0.0, 1.0);
                    let i = (y * w + x) * 4;
                    for k in 0..4 {
                        row[x * 4 + k] = a.px[i + k] * (1.0 - mix) + b.px[i + k] * mix;
                    }
                }
            });
        }
        TransitionType::Slide | TransitionType::ZoomPush => {
            let shift_x = p as f64 * w as f64;
            let ma = Affine::translate(-shift_x, 0.0);
            let mb = Affine::translate(w as f64 - shift_x, 0.0);
            out = a.transformed(w, h, &ma);
            let b_shifted = b.transformed(w, h, &mb);
            composite(&mut out, &b_shifted, 1.0, Blend::Normal);
        }
    }
    out
}

pub(crate) fn map_blend(mode: BlendMode) -> Blend {
    match mode {
        BlendMode::Normal => Blend::Normal,
        BlendMode::Dissolve => Blend::Dissolve,
        BlendMode::Darken => Blend::Darken,
        BlendMode::Multiply => Blend::Multiply,
        BlendMode::ColorBurn => Blend::ColorBurn,
        BlendMode::LinearBurn => Blend::LinearBurn,
        BlendMode::DarkerColor => Blend::DarkerColor,
        BlendMode::Lighten => Blend::Lighten,
        BlendMode::Screen => Blend::Screen,
        BlendMode::ColorDodge => Blend::ColorDodge,
        BlendMode::LinearDodge | BlendMode::Add => Blend::LinearDodge,
        BlendMode::LighterColor => Blend::LighterColor,
        BlendMode::Overlay => Blend::Overlay,
        BlendMode::SoftLight => Blend::SoftLight,
        BlendMode::HardLight => Blend::HardLight,
        BlendMode::VividLight => Blend::VividLight,
        BlendMode::LinearLight => Blend::LinearLight,
        BlendMode::PinLight => Blend::PinLight,
        BlendMode::HardMix => Blend::HardMix,
        BlendMode::Difference => Blend::Difference,
        BlendMode::Exclusion => Blend::Exclusion,
        BlendMode::Subtract => Blend::Subtract,
        BlendMode::Divide => Blend::Divide,
        BlendMode::Hue => Blend::Hue,
        BlendMode::Saturation => Blend::Saturation,
        BlendMode::Color => Blend::Color,
        BlendMode::Luminosity => Blend::Luminosity,
    }
}
