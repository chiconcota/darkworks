//! Xử lý biến hình, hòa trộn & chuyển cảnh cho `Transform2D`, `Blend` và `Transition` Nodes.

use darkworks_node_graph_core::{
    BlendMode, DarkNode, EvaluationContext, NodeGraphDoc, TransitionDirection, TransitionType,
};
use filmcraft_geom::{Affine, Vec2};
use rayon::prelude::*;

use crate::blend::{Blend, composite};
use crate::effects::gaussian_boxes;
use crate::gpufx::{FxOp, Resample};
use crate::image::Image;
use crate::nodes::NodeFxOp;
use crate::nodes::signals::{modulated_scalar_add, modulated_scalar_mul, modulated_vec2_add};

#[allow(clippy::too_many_arguments)]
fn build_transform_affine(
    position: [f32; 2],
    scale: [f32; 2],
    rotation_deg: f32,
    anchor: [f32; 2],
    skew_deg: f32,
    skew_axis_deg: f32,
    w: usize,
    h: usize,
    scale_xy: (f32, f32),
) -> Affine {
    let px_pos = Vec2::new(
        w as f64 * 0.5 + position[0] as f64 * scale_xy.0 as f64,
        h as f64 * 0.5 + position[1] as f64 * scale_xy.1 as f64,
    );
    let px_anchor = Vec2::new(w as f64 * anchor[0] as f64, h as f64 * anchor[1] as f64);
    let sk = if skew_deg.abs() > 1e-4 {
        let t = (skew_deg as f64).clamp(-85.0, 85.0).to_radians().tan();
        Affine::rotate_deg(skew_axis_deg as f64)
            .then_apply(&Affine { a: 1.0, b: 0.0, c: t, d: 1.0, e: 0.0, f: 0.0 })
            .then_apply(&Affine::rotate_deg(-(skew_axis_deg as f64)))
    } else {
        Affine::IDENTITY
    };
    Affine::translate(px_pos.x, px_pos.y)
        .then_apply(&Affine::rotate_deg(rotation_deg as f64))
        .then_apply(&sk)
        .then_apply(&Affine::scale(scale[0] as f64, scale[1] as f64))
        .then_apply(&Affine::translate(-px_anchor.x, -px_anchor.y))
}

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
    skew: f32,
    skew_axis: f32,
    scale_xy: (f32, f32),
) -> Image {
    if flip_h {
        FxOp::HFlip.apply(img);
    }
    if flip_v {
        FxOp::VFlip.apply(img);
    }
    let (w, h) = (img.w, img.h);
    let m = build_transform_affine(position, scale, rotation_deg, anchor, skew, skew_axis, w, h, scale_xy);
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

pub(crate) fn apply_transition(
    a: &Image,
    b: &Image,
    ty: TransitionType,
    progress: f32,
    feather: f32,
    direction: TransitionDirection,
) -> Image {
    let (w, h) = (a.w, a.h);
    let mut out = Image::new(w, h);
    let p = progress.clamp(0.0, 1.0);
    let f = feather.max(0.001);
    match ty {
        TransitionType::CrossDissolve => {
            blend_images(a, b, &mut out, p);
        }
        TransitionType::LinearWipe => {
            out.px.par_chunks_mut(w * 4).enumerate().for_each(|(y, row)| {
                let v = y as f32 / h.max(1) as f32;
                for x in 0..w {
                    let u = x as f32 / w.max(1) as f32;
                    let axis = match direction {
                        TransitionDirection::Left => u,
                        TransitionDirection::Right => 1.0 - u,
                        TransitionDirection::Up => v,
                        TransitionDirection::Down => 1.0 - v,
                    };
                    let mix = ((p - axis) / f + 0.5).clamp(0.0, 1.0);
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
        TransitionType::Slide => {
            let (dx, dy) = match direction {
                TransitionDirection::Left => (-p as f64 * w as f64, 0.0),
                TransitionDirection::Right => (p as f64 * w as f64, 0.0),
                TransitionDirection::Up => (0.0, -p as f64 * h as f64),
                TransitionDirection::Down => (0.0, p as f64 * h as f64),
            };
            let (bx, by) = match direction {
                TransitionDirection::Left => (w as f64 + dx, 0.0),
                TransitionDirection::Right => (dx - w as f64, 0.0),
                TransitionDirection::Up => (0.0, h as f64 + dy),
                TransitionDirection::Down => (0.0, dy - h as f64),
            };
            out = a.transformed(w, h, &Affine::translate(dx, dy));
            let b_shifted = b.transformed(w, h, &Affine::translate(bx, by));
            composite(&mut out, &b_shifted, 1.0, Blend::Normal);
        }
        TransitionType::ZoomPush => {
            let center = Vec2::new(w as f64 * 0.5, h as f64 * 0.5);
            let sa = 1.0 + (p as f64) * 0.85;
            let sb = 0.35 + (p as f64) * 0.65;
            let ma = Affine::motion(center, Vec2::new(sa, sa), 0.0, center);
            let mb = Affine::motion(center, Vec2::new(sb, sb), 0.0, center);
            let a_zoom = a.transformed(w, h, &ma);
            let b_zoom = b.transformed(w, h, &mb);
            blend_images(&a_zoom, &b_zoom, &mut out, p);
        }
        TransitionType::DipBlack | TransitionType::DipWhite => {
            let fill_val = if matches!(ty, TransitionType::DipWhite) { 1.0 } else { 0.0 };
            if p < 0.5 {
                let k = 1.0 - p * 2.0;
                out.px.par_chunks_mut(4).zip(a.px.par_chunks(4)).for_each(|(o, pa)| {
                    o[3] = pa[3];
                    for c in 0..3 {
                        o[c] = pa[c] * k + fill_val * pa[3] * (1.0 - k);
                    }
                });
            } else {
                let k = (p - 0.5) * 2.0;
                out.px.par_chunks_mut(4).zip(b.px.par_chunks(4)).for_each(|(o, pb)| {
                    o[3] = pb[3];
                    for c in 0..3 {
                        o[c] = pb[c] * k + fill_val * pb[3] * (1.0 - k);
                    }
                });
            }
        }
        TransitionType::BlurDissolve => {
            let peak_blur = (1.0 - (p * 2.0 - 1.0).abs()) * (16.0 + f * 32.0);
            let mut ba = a.clone();
            let mut bb = b.clone();
            if peak_blur > 0.5 {
                let (rx, ry) = gaussian_boxes(w, h, peak_blur, peak_blur);
                FxOp::Gaussian { rx: rx.clone(), ry: ry.clone(), repeat: true }.apply(&mut ba);
                FxOp::Gaussian { rx, ry, repeat: true }.apply(&mut bb);
            }
            blend_images(&ba, &bb, &mut out, p);
        }
    }
    out
}

fn blend_images(a: &Image, b: &Image, out: &mut Image, mix: f32) {
    out.px.par_chunks_mut(4).zip(a.px.par_chunks(4).zip(b.px.par_chunks(4))).for_each(|(o, (pa, pb))| {
        for k in 0..4 {
            o[k] = pa[k] * (1.0 - mix) + pb[k] * mix;
        }
    });
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

/// Biên dịch `DarkNode::CropMask` thành [`FxOp::Crop`] chạy GPU.
pub(crate) fn push_crop_gpu_op(
    node: &DarkNode,
    size: [usize; 2],
    px_scale: f32,
    ops: &mut Vec<FxOp>,
) {
    let DarkNode::CropMask { top, bottom, left, right, feather } = node else {
        return;
    };
    let [lw, lh] = [size[0] as f32, size[1] as f32];
    let x0 = (left.clamp(0.0, 1.0) * lw).min(lw);
    let y0 = (top.clamp(0.0, 1.0) * lh).min(lh);
    ops.push(FxOp::Crop {
        x0,
        x1: ((1.0 - right.clamp(0.0, 1.0)) * lw).max(x0),
        y0,
        y1: ((1.0 - bottom.clamp(0.0, 1.0)) * lh).max(y0),
        feather: (*feather * px_scale).max(0.0),
    });
}

/// Biên dịch `DarkNode::Transform2D` thành chuỗi [`FxOp`] chạy GPU.
pub(crate) fn push_transform_gpu_ops(
    doc: &NodeGraphDoc,
    sig_ctx: &EvaluationContext,
    id: usize,
    node: &DarkNode,
    size: [usize; 2],
    seq_scale: [f32; 2],
    ops: &mut Vec<FxOp>,
) -> Option<()> {
    let DarkNode::Transform2D {
        position,
        scale,
        rotation_deg,
        anchor,
        opacity,
        flip_h,
        flip_v,
        uniform_scale,
        skew,
        skew_axis,
    } = node
    else {
        return Some(());
    };
    if *flip_h {
        ops.push(FxOp::HFlip);
    }
    if *flip_v {
        ops.push(FxOp::VFlip);
    }
    let pos = modulated_vec2_add(doc, sig_ctx, id, 1, *position);
    let s_mod = modulated_scalar_mul(doc, sig_ctx, id, 2, 1.0);
    let rot_mod = modulated_scalar_add(doc, sig_ctx, id, 3, *rotation_deg);
    let op_mod = modulated_scalar_mul(doc, sig_ctx, id, 4, *opacity).clamp(0.0, 1.0);
    let sc = if *uniform_scale {
        [scale[0] * s_mod, scale[0] * s_mod]
    } else {
        [scale[0] * s_mod, scale[1] * s_mod]
    };
    if sc[0].abs() < 1e-4 || sc[1].abs() < 1e-4 {
        ops.push(FxOp::AlphaAdjust { opacity: 0.0, ignore: false, invert: false, mask_only: false });
        return Some(());
    }
    let [lw, lh] = size;
    let m = build_transform_affine(pos, sc, rot_mod, *anchor, *skew, *skew_axis, lw, lh, (seq_scale[0], seq_scale[1]));
    let inv = m.inverse();
    let b = m.bounds(&filmcraft_geom::Rect::new(0.0, 0.0, lw as f64, lh as f64));
    let rect = [
        (b.x.floor().max(0.0) as usize).min(lw) as u32,
        (b.right().ceil().max(0.0) as usize).min(lw) as u32,
        (b.y.floor().max(0.0) as usize).min(lh) as u32,
        (b.bottom().ceil().max(0.0) as usize).min(lh) as u32,
    ];
    let msx = (m.a * m.a + m.b * m.b).sqrt();
    let msy = (m.c * m.c + m.d * m.d).sqrt();
    let minify = 1.0 / msx.min(msy).max(1e-6);
    if minify > 2.01 && lw >= 4 && lh >= 4 {
        if let Some(inv_m) = inv {
            let taps = ((minify - 1e-4).ceil() as u32).clamp(2, 8);
            ops.push(FxOp::Node(NodeFxOp::ResampleArea {
                inv: [inv_m.a as f32, inv_m.b as f32, inv_m.c as f32, inv_m.d as f32, inv_m.e as f32, inv_m.f as f32],
                rect,
                opacity: op_mod,
                taps,
            }));
        } else {
            ops.push(FxOp::AlphaAdjust { opacity: 0.0, ignore: false, invert: false, mask_only: false });
        }
    } else {
        ops.push(FxOp::Resample(Resample { inv, rect, opacity: op_mod }));
    }
    Some(())
}
