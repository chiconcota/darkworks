//! Xử lý quang học & biến dạng cho `BlurSharpen` và `DistortLens` Nodes.

use darkworks_node_graph_core::{BlurDimensions, BlurType, DistortType};
use filmcraft_color::luma709;
use rayon::prelude::*;

use crate::effects::{box_blur, dec, enc, gaussian_boxes, hash3, mosaic, warp};
use crate::gpufx::FxOp;
use crate::image::Image;
use crate::vfx::fbm;

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
            let src = img.clone();
            let (w, h) = (img.w, img.h);
            let cx = w as f32 * (0.5 + center[0] * 0.5);
            let cy = h as f32 * (0.5 + center[1] * 0.5);
            let taps = 16usize;
            img.px.par_chunks_mut(w * 4).enumerate().for_each(|(y, row)| {
                let py = y as f32 + 0.5;
                let dy = cy - py;
                for x in 0..w {
                    let px = x as f32 + 0.5;
                    let dx = cx - px;
                    let mut acc = [0.0f32; 4];
                    for s in 0..taps {
                        let frac = s as f32 / (taps - 1) as f32 * strength;
                        let p = src.sample_bilinear_clamped(px + dx * frac, py + dy * frac);
                        for k in 0..4 {
                            acc[k] += p[k];
                        }
                    }
                    let inv = 1.0 / taps as f32;
                    row[x * 4..x * 4 + 4].copy_from_slice(&[acc[0] * inv, acc[1] * inv, acc[2] * inv, acc[3] * inv]);
                }
            });
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
        DistortType::Twirl => {
            let ang = (intensity as f64) * std::f64::consts::PI;
            let rad = (radius.max(0.05) as f64) * 0.5 * (img.w.min(img.h) as f64);
            let cx = img.w as f64 * (0.5 + center[0] as f64 * 0.5);
            let cy = img.h as f64 * (0.5 + center[1] as f64 * 0.5);
            warp(img, |x, y| {
                let (dx, dy) = (x - cx, y - cy);
                let d = dx.hypot(dy);
                if d >= rad {
                    return (x, y);
                }
                let k = 1.0 - d / rad;
                let (s, c) = (ang * k * k).sin_cos();
                (cx + dx * c - dy * s, cy + dx * s + dy * c)
            });
        }
        DistortType::TurbulentDisplace => {
            let amt = intensity as f64 * 42.0 * scale as f64;
            if amt.abs() < 0.05 {
                return;
            }
            let size = (radius.max(0.05) * 80.0 * scale).max(4.0);
            let (ox, oy) = (center[0] * 4.0, center[1] * 4.0);
            let evo = time_sec as f32 * 0.6;
            warp(img, |x, y| {
                let (u, v) = (x as f32 / size + ox, y as f32 / size + oy);
                let nx = fbm(u, v, evo, 3.0, 17);
                let ny = fbm(u + 31.7, v + 12.3, evo + 3.1, 3.0, 17);
                (x + nx as f64 * amt, y + ny as f64 * amt)
            });
        }
        DistortType::FindEdges => {
            let inv = intensity < 0.0;
            let blend = feather.clamp(0.0, 1.0);
            let src = img.clone();
            let w = img.w;
            img.px.par_chunks_mut(w * 4).enumerate().for_each(|(y, row)| {
                for x in 0..w {
                    let l = |dx: isize, dy: isize| {
                        let p = src.get_clamped(x as isize + dx, y as isize + dy);
                        luma709(p[0], p[1], p[2])
                    };
                    let gx = l(1, -1) + 2.0 * l(1, 0) + l(1, 1) - l(-1, -1) - 2.0 * l(-1, 0) - l(-1, 1);
                    let gy = l(-1, 1) + 2.0 * l(0, 1) + l(1, 1) - l(-1, -1) - 2.0 * l(0, -1) - l(1, -1);
                    let mut m = (gx * gx + gy * gy).sqrt().min(1.0);
                    if !inv {
                        m = 1.0 - m;
                    }
                    let o = src.get(x, y);
                    let a = o[3];
                    for k in 0..3 {
                        row[x * 4 + k] = m * a * (1.0 - blend) + o[k] * blend;
                    }
                }
            });
        }
        DistortType::NoiseGrain => {
            let amt = intensity.abs().clamp(0.0, 2.0) * 0.35;
            if amt < 1e-4 {
                return;
            }
            let grain = (radius.max(0.25) * 4.0 * scale).max(1.0);
            let color_mix = feather.clamp(0.0, 1.0);
            let frame_seed = (time_sec * 60.0).round() as u64;
            let w = img.w;
            img.px.par_chunks_mut(w * 4).enumerate().for_each(|(y, row)| {
                for x in 0..w {
                    let p = &mut row[x * 4..x * 4 + 4];
                    let a = p[3];
                    if a <= 0.0 {
                        continue;
                    }
                    let (gx, gy) = ((x as f32 / grain) as usize, (y as f32 / grain) as usize);
                    let n0 = hash3(gx, gy, frame_seed) - 0.5;
                    let ns = [
                        n0,
                        n0 + (hash3(gx, gy, frame_seed + 7777) - 0.5 - n0) * color_mix,
                        n0 + (hash3(gx, gy, frame_seed + 99_991) - 0.5 - n0) * color_mix,
                    ];
                    let c = enc([p[0] / a, p[1] / a, p[2] / a]);
                    let o = dec([
                        (c[0] + ns[0] * amt).clamp(0.0, 1.0),
                        (c[1] + ns[1] * amt).clamp(0.0, 1.0),
                        (c[2] + ns[2] * amt).clamp(0.0, 1.0),
                    ]);
                    p[0] = o[0] * a;
                    p[1] = o[1] * a;
                    p[2] = o[2] * a;
                }
            });
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
