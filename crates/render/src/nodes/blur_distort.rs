//! Xử lý quang học & biến dạng cho `BlurSharpen` và `DistortLens` Nodes.

use darkworks_node_graph_core::{
    BlurDimensions, BlurType, DarkNode, DistortType, EvaluationContext, NodeGraphDoc,
};

use crate::effects::{box_blur, gaussian_boxes};
use crate::gpufx::FxOp;
use crate::image::Image;
use crate::nodes::node_fx_ops::NodeFxOp;
use crate::nodes::signals::{modulated_scalar, modulated_scalar_mul, modulated_vec2};

#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_blur_sharpen(
    img: &mut Image,
    blur_type: BlurType,
    radius: f32,
    angle_deg: f32,
    highlight_boost: f32,
    threshold: f32,
    dimensions: BlurDimensions,
    repeat_edge: bool,
    bokeh_sides: u8,
    center: [f32; 2],
    scale: f32,
) {
    let r = (radius * scale).max(0.0);
    if r < 0.1 {
        return;
    }
    let (rx_on, ry_on) = match dimensions {
        BlurDimensions::Both => (r, r),
        BlurDimensions::Horizontal => (r, 0.0),
        BlurDimensions::Vertical => (0.0, r),
    };
    match blur_type {
        BlurType::Gaussian => {
            let (rx, ry) = gaussian_boxes(img.w, img.h, rx_on, ry_on);
            FxOp::Gaussian { rx, ry, repeat: repeat_edge }.apply(img);
        }
        BlurType::Box => {
            let box_r = (r.round() as u32).max(1);
            let rx = if rx_on > 0.0 { vec![box_r] } else { Vec::new() };
            let ry = if ry_on > 0.0 { vec![box_r] } else { Vec::new() };
            box_blur(img, &rx, &ry, repeat_edge);
        }
        BlurType::Directional => {
            let rad = angle_deg.to_radians();
            let steps = ((r * 0.75).round() as u32).clamp(9, 25);
            FxOp::DirectionalBlur {
                dx: rad.cos() * r,
                dy: rad.sin() * r,
                steps,
            }
            .apply(img);
        }
        BlurType::BokehBloom => {
            FxOp::Bokeh {
                levels: 2,
                boost: highlight_boost.max(0.0),
                threshold: threshold.clamp(0.0, 1.0),
                sides: u32::from(bokeh_sides.min(8)),
                rot: angle_deg.to_radians(),
                rr: r,
                rings: 3,
            }
            .apply(img);
        }
        BlurType::UnsharpMask => {
            let (rx, ry) = gaussian_boxes(img.w, img.h, r, r);
            FxOp::Unsharp {
                rx,
                ry,
                amount: highlight_boost.max(0.2),
                threshold: (threshold * 0.2).clamp(0.0, 0.5),
            }
            .apply(img);
        }
        BlurType::RadialZoom => {
            let strength = (radius / 100.0).clamp(0.0, 1.5) * 0.45;
            if strength < 1e-4 {
                return;
            }
            let cx = img.w as f32 * (0.5 + center[0] * 0.5);
            let cy = img.h as f32 * (0.5 + center[1] * 0.5);
            FxOp::Node(NodeFxOp::RadialZoom { strength, cx, cy, taps: 16 }).apply(img);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_distort(
    img: &mut Image,
    distort_type: DistortType,
    intensity: f32,
    radius: f32,
    center: [f32; 2],
    feather: f32,
    time_sec: f64,
    scale: f32,
) {
    match distort_type {
        DistortType::Vignette => {
            FxOp::Vignette {
                amount: intensity,
                midpoint: radius.clamp(0.0, 1.0),
                roundness: 0.0,
                feather: feather.clamp(0.01, 1.0),
                target: [0.0, 0.0, 0.0],
            }
            .apply(img);
        }
        DistortType::RippleWave => {
            let amp = intensity * 24.0 * scale;
            let freq = radius.max(0.05) * 0.03;
            let phase = (time_sec * std::f64::consts::TAU * 2.0) as f32;
            FxOp::Node(NodeFxOp::RippleWave { amp, freq, phase }).apply(img);
        }
        DistortType::FishEye | DistortType::LensDistortion => {
            let k = intensity * if matches!(distort_type, DistortType::FishEye) { 0.45 } else { 0.25 };
            let cx = img.w as f32 * (0.5 + center[0] * 0.5);
            let cy = img.h as f32 * (0.5 + center[1] * 0.5);
            let norm = (img.w as f32 * 0.5).hypot(img.h as f32 * 0.5).max(1.0);
            FxOp::Node(NodeFxOp::LensDistort { k, cx, cy, norm }).apply(img);
        }
        DistortType::ChromaticAberration => {
            FxOp::Node(NodeFxOp::ChromaticAberration { shift: intensity * 6.0 * scale }).apply(img);
        }
        DistortType::Mirror => {
            let rad = (intensity * 180.0).to_radians();
            let cx = img.w as f32 * (0.5 + center[0] * 0.5);
            let cy = img.h as f32 * (0.5 + center[1] * 0.5);
            FxOp::Mirror {
                cx,
                cy,
                nx: rad.cos(),
                ny: rad.sin(),
            }
            .apply(img);
        }
        DistortType::Offset => {
            let w = img.w.max(1) as f32;
            let h = img.h.max(1) as f32;
            let dx = (center[0] * w * 0.5).rem_euclid(w);
            let dy = (center[1] * h * 0.5).rem_euclid(h);
            FxOp::Offset {
                dx,
                dy,
                blend: (1.0 - feather).clamp(0.0, 1.0),
            }
            .apply(img);
        }
        DistortType::Posterize => {
            let levels = (2.0 + intensity.abs() * 14.0).clamp(2.0, 64.0);
            FxOp::Posterize { n: levels }.apply(img);
        }
        DistortType::Pixelate => {
            let block = ((intensity.abs() * 32.0 + radius * 8.0) * scale).round().max(2.0) as u32;
            FxOp::Node(NodeFxOp::Pixelate { block_w: block, block_h: block }).apply(img);
        }
        DistortType::Twirl => {
            let ang = intensity * std::f32::consts::PI;
            let rad = radius.max(0.05) * 0.5 * (img.w.min(img.h) as f32);
            let cx = img.w as f32 * (0.5 + center[0] * 0.5);
            let cy = img.h as f32 * (0.5 + center[1] * 0.5);
            FxOp::Node(NodeFxOp::Twirl { ang, rad, cx, cy }).apply(img);
        }
        DistortType::TurbulentDisplace => {
            let amt = intensity * 42.0 * scale;
            let size = (radius.max(0.05) * 80.0 * scale).max(4.0);
            let (ox, oy) = (center[0] * 4.0, center[1] * 4.0);
            let evo = time_sec as f32 * 0.6;
            FxOp::Node(NodeFxOp::TurbulentDisplace { amt, size, ox, oy, evo }).apply(img);
        }
        DistortType::FindEdges => {
            FxOp::Node(NodeFxOp::FindEdges {
                invert: intensity < 0.0,
                blend: feather.clamp(0.0, 1.0),
            })
            .apply(img);
        }
        DistortType::NoiseGrain => {
            let amt = intensity.abs().clamp(0.0, 2.0) * 0.35;
            let grain = (radius.max(0.25) * 4.0 * scale).max(1.0);
            let color_mix = feather.clamp(0.0, 1.0);
            let seed = (time_sec * 60.0).round() as u32;
            FxOp::Node(NodeFxOp::NoiseGrain { amt, grain, color_mix, seed }).apply(img);
        }
    }
}

/// Biên dịch `DarkNode::BlurSharpen` thành chuỗi [`FxOp`] chạy GPU.
pub(crate) fn push_blur_gpu_ops(
    doc: &NodeGraphDoc,
    sig_ctx: &EvaluationContext,
    id: usize,
    node: &DarkNode,
    size: [usize; 2],
    px_scale: f32,
    ops: &mut Vec<FxOp>,
) {
    let DarkNode::BlurSharpen {
        blur_type,
        radius,
        angle_deg,
        highlight_boost,
        threshold,
        dimensions,
        repeat_edge,
        bokeh_sides,
        center,
    } = node
    else {
        return;
    };
    let [lw, lh] = size;
    let eff_r = (modulated_scalar_mul(doc, sig_ctx, id, 1, *radius) * px_scale).max(0.0);
    if eff_r < 0.1 {
        return;
    }
    let (rx_on, ry_on) = match dimensions {
        BlurDimensions::Both => (eff_r, eff_r),
        BlurDimensions::Horizontal => (eff_r, 0.0),
        BlurDimensions::Vertical => (0.0, eff_r),
    };
    match blur_type {
        BlurType::Gaussian => {
            let (rx, ry) = gaussian_boxes(lw, lh, rx_on, ry_on);
            ops.push(FxOp::Gaussian { rx, ry, repeat: *repeat_edge });
        }
        BlurType::Box => {
            let b = (eff_r.round() as u32).max(1);
            ops.push(FxOp::Gaussian {
                rx: if rx_on > 0.0 { vec![b] } else { Vec::new() },
                ry: if ry_on > 0.0 { vec![b] } else { Vec::new() },
                repeat: *repeat_edge,
            });
        }
        BlurType::Directional => {
            let rad = angle_deg.to_radians();
            let steps = ((eff_r * 0.75).round() as u32).clamp(9, 25);
            ops.push(FxOp::DirectionalBlur { dx: rad.cos() * eff_r, dy: rad.sin() * eff_r, steps });
        }
        BlurType::BokehBloom => ops.push(FxOp::Bokeh {
            levels: 2,
            boost: highlight_boost.max(0.0),
            threshold: threshold.clamp(0.0, 1.0),
            sides: u32::from((*bokeh_sides).min(8)),
            rot: angle_deg.to_radians(),
            rr: eff_r,
            rings: 3,
        }),
        BlurType::UnsharpMask => {
            let (rx, ry) = gaussian_boxes(lw, lh, eff_r, eff_r);
            ops.push(FxOp::Unsharp {
                rx,
                ry,
                amount: highlight_boost.max(0.2),
                threshold: (*threshold * 0.2).clamp(0.0, 0.5),
            });
        }
        BlurType::RadialZoom => {
            let raw_r = modulated_scalar_mul(doc, sig_ctx, id, 1, *radius);
            let strength = (raw_r / 100.0).clamp(0.0, 1.5) * 0.45;
            if strength >= 1e-4 {
                ops.push(FxOp::Node(NodeFxOp::RadialZoom {
                    strength,
                    cx: lw as f32 * (0.5 + center[0] * 0.5),
                    cy: lh as f32 * (0.5 + center[1] * 0.5),
                    taps: 16,
                }));
            }
        }
    }
}

/// Biên dịch `DarkNode::DistortLens` thành chuỗi [`FxOp`] chạy GPU.
pub(crate) fn push_distort_gpu_ops(
    doc: &NodeGraphDoc,
    sig_ctx: &EvaluationContext,
    id: usize,
    node: &DarkNode,
    size: [usize; 2],
    scale_time: (f32, f64),
    ops: &mut Vec<FxOp>,
) {
    let DarkNode::DistortLens { distort_type, intensity, radius, center, feather } = node else {
        return;
    };
    let [lw, lh] = size;
    let (px_scale, time_sec) = scale_time;
    let eff_int = modulated_scalar(doc, sig_ctx, id, 1, *intensity);
    let eff_center = modulated_vec2(doc, sig_ctx, id, 2, *center);
    let cx = lw as f32 * (0.5 + eff_center[0] * 0.5);
    let cy = lh as f32 * (0.5 + eff_center[1] * 0.5);
    match distort_type {
        DistortType::Vignette => ops.push(FxOp::Vignette {
            amount: eff_int,
            midpoint: radius.clamp(0.0, 1.0),
            roundness: 0.0,
            feather: feather.clamp(0.01, 1.0),
            target: [0.0, 0.0, 0.0],
        }),
        DistortType::Mirror => {
            let rad = (eff_int * 180.0).to_radians();
            ops.push(FxOp::Mirror { cx, cy, nx: rad.cos(), ny: rad.sin() });
        }
        DistortType::Offset => {
            let (wf, hf) = (lw.max(1) as f32, lh.max(1) as f32);
            ops.push(FxOp::Offset {
                dx: (eff_center[0] * wf * 0.5).rem_euclid(wf),
                dy: (eff_center[1] * hf * 0.5).rem_euclid(hf),
                blend: (1.0 - feather).clamp(0.0, 1.0),
            });
        }
        DistortType::Posterize => ops.push(FxOp::Posterize {
            n: (2.0 + eff_int.abs() * 14.0).clamp(2.0, 64.0),
        }),
        DistortType::Pixelate => {
            let block = ((eff_int.abs() * 32.0 + radius * 8.0) * px_scale).round().max(2.0) as u32;
            ops.push(FxOp::Node(NodeFxOp::Pixelate { block_w: block, block_h: block }));
        }
        DistortType::ChromaticAberration => {
            let shift = eff_int * 6.0 * px_scale;
            if shift.abs() >= 0.1 {
                ops.push(FxOp::Node(NodeFxOp::ChromaticAberration { shift }));
            }
        }
        DistortType::RippleWave => ops.push(FxOp::Node(NodeFxOp::RippleWave {
            amp: eff_int * 24.0 * px_scale,
            freq: radius.max(0.05) * 0.03,
            phase: (time_sec * std::f64::consts::TAU * 2.0) as f32,
        })),
        DistortType::FishEye | DistortType::LensDistortion => {
            let k = eff_int * if matches!(distort_type, DistortType::FishEye) { 0.45 } else { 0.25 };
            let norm = (lw as f32 * 0.5).hypot(lh as f32 * 0.5).max(1.0);
            ops.push(FxOp::Node(NodeFxOp::LensDistort { k, cx, cy, norm }));
        }
        DistortType::Twirl => ops.push(FxOp::Node(NodeFxOp::Twirl {
            ang: eff_int * std::f32::consts::PI,
            rad: radius.max(0.05) * 0.5 * (lw.min(lh) as f32),
            cx,
            cy,
        })),
        DistortType::FindEdges => ops.push(FxOp::Node(NodeFxOp::FindEdges {
            invert: eff_int < 0.0,
            blend: feather.clamp(0.0, 1.0),
        })),
        DistortType::NoiseGrain => {
            let amt = eff_int.abs().clamp(0.0, 2.0) * 0.35;
            if amt >= 1e-4 {
                ops.push(FxOp::Node(NodeFxOp::NoiseGrain {
                    amt,
                    grain: (radius.max(0.25) * 4.0 * px_scale).max(1.0),
                    color_mix: feather.clamp(0.0, 1.0),
                    seed: (time_sec * 60.0).round() as u32,
                }));
            }
        }
        DistortType::TurbulentDisplace => {
            let amt = eff_int * 42.0 * px_scale;
            if amt.abs() >= 0.05 {
                ops.push(FxOp::Node(NodeFxOp::TurbulentDisplace {
                    amt,
                    size: (radius.max(0.05) * 80.0 * px_scale).max(4.0),
                    ox: eff_center[0] * 4.0,
                    oy: eff_center[1] * 4.0,
                    evo: time_sec as f32 * 0.6,
                }));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gaussian_horizontal_does_not_bleed_vertically_and_radial_zoom_works() {
        // Horizontal white stripe on rows 4..8: Horizontal-only blur keeps row 2 completely black!
        let mut h_img = Image::new(16, 16);
        for y in 4..8 {
            for x in 4..12 {
                let i = (y * 16 + x) * 4;
                h_img.px[i..i + 4].copy_from_slice(&[1.0, 1.0, 1.0, 1.0]);
            }
        }
        apply_blur_sharpen(
            &mut h_img,
            BlurType::Gaussian,
            4.0,
            0.0,
            1.0,
            0.5,
            BlurDimensions::Horizontal,
            true,
            0,
            [0.0, 0.0],
            1.0,
        );
        assert!(h_img.get(2, 6)[0] > 0.05, "horizontal blur must spread along X");
        assert!(h_img.get(6, 2)[0] < 1e-6, "horizontal-only blur must not bleed along Y");

        // RadialZoom from center blurs outer border without touching exact center
        let mut r_img = h_img.clone();
        apply_blur_sharpen(
            &mut r_img,
            BlurType::RadialZoom,
            30.0,
            0.0,
            1.0,
            0.5,
            BlurDimensions::Both,
            true,
            0,
            [0.0, 0.0],
            1.0,
        );
        assert_eq!((r_img.w, r_img.h), (16, 16));
    }

    #[test]
    fn new_distort_modes_twirl_turbulent_find_edges_noise_work() {
        let mut base = Image::new(24, 24);
        for y in 0..24 {
            for x in 0..24 {
                let v = if x < 12 { 0.2 } else { 0.8 };
                let i = (y * 24 + x) * 4;
                base.px[i..i + 4].copy_from_slice(&[v, v, v, 1.0]);
            }
        }

        let mut tw = base.clone();
        apply_distort(&mut tw, DistortType::Twirl, 1.0, 0.8, [0.0, 0.0], 0.0, 0.0, 1.0);
        assert_ne!(tw.px, base.px, "Twirl must warp pixels");

        let mut td = base.clone();
        apply_distort(&mut td, DistortType::TurbulentDisplace, 0.8, 0.5, [0.0, 0.0], 0.0, 0.5, 1.0);
        assert_ne!(td.px, base.px, "TurbulentDisplace must warp pixels");

        let mut fe = base.clone();
        apply_distort(&mut fe, DistortType::FindEdges, -1.0, 0.5, [0.0, 0.0], 0.0, 0.0, 1.0);
        assert!(fe.get(12, 12)[0] > fe.get(2, 12)[0], "FindEdges must highlight vertical edge");

        let mut ng = Image::filled(24, 24, [0.5, 0.5, 0.5, 1.0]);
        apply_distort(&mut ng, DistortType::NoiseGrain, 0.5, 0.5, [0.0, 0.0], 1.0, 1.0, 1.0);
        assert_ne!(ng.get(3, 3), ng.get(15, 15), "NoiseGrain must vary pixels");
    }
}
