//! Xử lý quang học & biến dạng cho `BlurSharpen` và `DistortLens` Nodes.

use darkworks_node_graph_core::{BlurType, DistortType};
use rayon::prelude::*;

use crate::effects::{box_blur, gaussian_boxes, mosaic, warp};
use crate::gpufx::FxOp;
use crate::image::Image;

pub(crate) fn apply_blur_sharpen(
    img: &mut Image,
    blur_type: BlurType,
    radius: f32,
    angle_deg: f32,
    highlight_boost: f32,
    threshold: f32,
    scale: f32,
) {
    let r = (radius * scale).max(0.0);
    if r < 0.1 {
        return;
    }
    match blur_type {
        BlurType::Gaussian => {
            let (rx, ry) = gaussian_boxes(img.w, img.h, r, r);
            FxOp::Gaussian { rx, ry, repeat: true }.apply(img);
        }
        BlurType::Box => {
            let box_r = (r.round() as u32).max(1);
            box_blur(img, &[box_r], &[box_r], true);
        }
        BlurType::Directional => {
            let rad = angle_deg.to_radians();
            FxOp::DirectionalBlur {
                dx: rad.cos() * r,
                dy: rad.sin() * r,
                steps: 9,
            }
            .apply(img);
        }
        BlurType::BokehBloom => {
            FxOp::Bokeh {
                levels: 2,
                boost: highlight_boost.max(0.0),
                threshold: threshold.clamp(0.0, 1.0),
                sides: 0,
                rot: 0.0,
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
            let amp = intensity as f64 * 24.0 * scale as f64;
            let freq = (radius.max(0.05) as f64) * 0.03;
            let phase = time_sec * std::f64::consts::TAU * 2.0;
            warp(img, |x, y| {
                let off_y = amp * (x * freq - phase).sin();
                (x, y + off_y)
            });
        }
        DistortType::FishEye | DistortType::LensDistortion => {
            let k = intensity as f64 * if matches!(distort_type, DistortType::FishEye) { 0.45 } else { 0.25 };
            let cxp = img.w as f64 * (0.5 + center[0] as f64 * 0.5);
            let cyp = img.h as f64 * (0.5 + center[1] as f64 * 0.5);
            let norm = (img.w as f64 * 0.5).hypot(img.h as f64 * 0.5).max(1.0);
            warp(img, |x, y| {
                let (dx, dy) = ((x - cxp) / norm, (y - cyp) / norm);
                let r2 = dx * dx + dy * dy;
                let s = 1.0 - k * r2;
                (cxp + dx * s * norm, cyp + dy * s * norm)
            });
        }
        DistortType::ChromaticAberration => {
            let shift = intensity * 6.0 * scale;
            if shift.abs() < 0.1 {
                return;
            }
            let src = img.clone();
            let w = img.w;
            img.px.par_chunks_mut(w * 4).enumerate().for_each(|(y, row)| {
                for x in 0..w {
                    let r = src.sample_bilinear_clamped(x as f32 + 0.5 + shift, y as f32 + 0.5)[0];
                    let g_a = src.get(x, y);
                    let b = src.sample_bilinear_clamped(x as f32 + 0.5 - shift, y as f32 + 0.5)[2];
                    row[x * 4..x * 4 + 4].copy_from_slice(&[r, g_a[1], b, g_a[3]]);
                }
            });
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
            let block = ((intensity.abs() * 32.0 + radius * 8.0) * scale).round().max(2.0) as usize;
            mosaic(img, block, block);
        }
    }
}
