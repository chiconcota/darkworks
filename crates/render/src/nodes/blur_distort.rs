//! Xử lý quang học & biến dạng cho `BlurSharpen` và `DistortLens` Nodes.

use darkworks_node_graph_core::{BlurType, DistortType};
use rayon::prelude::*;

use crate::effects::{gaussian_boxes, warp};
use crate::gpufx::FxOp;
use crate::image::Image;

pub(crate) fn apply_blur_sharpen(
    img: &mut Image,
    blur_type: BlurType,
    radius: f32,
    angle_deg: f32,
    highlight_boost: f32,
    scale: f32,
) {
    let r = (radius * scale).max(0.0);
    if r < 0.1 {
        return;
    }
    match blur_type {
        BlurType::Gaussian | BlurType::Box => {
            let (rx, ry) = gaussian_boxes(img.w, img.h, r, r);
            FxOp::Gaussian { rx, ry, repeat: true }.apply(img);
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
                threshold: 0.65,
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
                amount: highlight_boost.max(0.5),
                threshold: 0.02,
            }
            .apply(img);
        }
    }
}

pub(crate) fn apply_distort(
    img: &mut Image,
    distort_type: DistortType,
    intensity: f32,
    radius: f32,
    center: [f32; 2],
    time_sec: f64,
    scale: f32,
) {
    match distort_type {
        DistortType::Vignette => {
            FxOp::Vignette {
                amount: intensity,
                midpoint: radius.clamp(0.0, 1.0),
                roundness: 0.0,
                feather: 0.4,
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
    }
}
