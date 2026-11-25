//! Xử lý chỉnh màu & tách phông cho `ColorGrade` và `Keyer` Nodes.

use filmcraft_color::srgb_to_linear;
use rayon::prelude::*;

use crate::gpufx::FxOp;
use crate::image::Image;

/// Tham số chỉnh màu mở rộng của `ColorGrade` Node (bao gồm toàn bộ bộ điều khiển Lumetri cơ bản + Split Tone).
#[derive(Debug, Clone, Copy)]
pub(crate) struct ColorGradeParams {
    pub exposure: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub temperature: f32,
    pub tint: f32,
    pub gamma: f32,
    pub invert: bool,
    pub black_and_white: bool,
    pub tint_color: [f32; 4],
    pub tint_amount: f32,
    pub highlights: f32,
    pub shadows: f32,
    pub whites: f32,
    pub blacks: f32,
    pub vibrance: f32,
    pub faded_film: f32,
    pub shadow_tint: [f32; 3],
    pub highlight_tint: [f32; 3],
}

pub(crate) fn apply_color_grade(img: &mut Image, p: &ColorGradeParams) {
    // 1. B&W và Invert chạy trước để các bước chỉnh màu/nhuộm màu tiếp theo hoạt động đúng trên kết quả
    if p.black_and_white {
        FxOp::BlackWhite.apply(img);
    }
    if p.invert {
        FxOp::Invert { channel: 0, blend: 0.0 }.apply(img);
    }

    let has_split_tone = p.shadow_tint.iter().any(|&v| (v - 0.5).abs() > 1e-3)
        || p.highlight_tint.iter().any(|&v| (v - 0.5).abs() > 1e-3);
    let use_lumetri = p.highlights.abs() > 1e-3
        || p.shadows.abs() > 1e-3
        || p.whites.abs() > 1e-3
        || p.blacks.abs() > 1e-3
        || p.vibrance.abs() > 1e-3
        || p.faded_film.abs() > 1e-3
        || has_split_tone;

    if use_lumetri {
        let temp = (p.temperature / 100.0).clamp(-1.5, 1.5);
        let tint = (p.tint / 100.0).clamp(-1.5, 1.5);
        let gains = [1.0 + 0.35 * temp, 1.0 - 0.3 * tint, 1.0 - 0.35 * temp];
        let aspect = if img.h > 0 {
            (img.w as f32 / img.h as f32).max(1e-4)
        } else {
            1.0
        };
        FxOp::Lumetri {
            gains,
            exposure: 2.0_f32.powf(p.exposure.clamp(-5.0, 5.0)),
            contrast: (p.contrast - 1.0).clamp(-1.0, 2.0),
            hl: p.highlights / 100.0,
            sh: p.shadows / 100.0,
            wh: p.whites / 100.0,
            bl: p.blacks / 100.0,
            sat: p.saturation.max(0.0),
            creative_on: has_split_tone || p.faded_film.abs() > 1e-3 || p.vibrance.abs() > 1e-3,
            faded: (p.faded_film / 100.0).clamp(0.0, 1.0),
            vib: p.vibrance / 100.0,
            st: p.shadow_tint,
            ht: p.highlight_tint,
            vignette_on: false,
            va: 0.0,
            vmid: 0.5,
            vround: 0.0,
            vfeather: 0.5,
            aspect,
            gpu_capable: true,
        }
        .apply(img);
    } else {
        if p.exposure.abs() > 1e-4 || (p.contrast - 1.0).abs() > 1e-4 || (p.saturation - 1.0).abs() > 1e-4 {
            FxOp::ProcAmp {
                br: p.exposure * 0.25,
                co: p.contrast.max(0.0),
                hue: 0.0,
                sat: p.saturation.max(0.0),
            }
            .apply(img);
        }
        if p.temperature.abs() > 1e-4 || p.tint.abs() > 1e-4 {
            let t = p.temperature * 0.2;
            let tn = p.tint * 0.2;
            FxOp::ColorBalance {
                sh: [t, -tn, -t],
                md: [t, -tn, -t],
                hi: [t, -tn, -t],
                preserve: true,
            }
            .apply(img);
        }
    }

    if (p.gamma - 1.0).abs() > 1e-4 {
        FxOp::Gamma { g: p.gamma.clamp(0.1, 5.0) }.apply(img);
    }
    if p.tint_amount > 1e-4 {
        FxOp::Tint {
            black: [0.0, 0.0, 0.0],
            white: [p.tint_color[0], p.tint_color[1], p.tint_color[2]],
            amount: p.tint_amount.clamp(0.0, 1.0),
        }
        .apply(img);
    }
}

pub(crate) fn apply_keyer(
    img: &mut Image,
    key_color: [f32; 3],
    similarity: f32,
    smoothness: f32,
    spill_suppress: f32,
) {
    let key_lin = [
        srgb_to_linear(key_color[0]),
        srgb_to_linear(key_color[1]),
        srgb_to_linear(key_color[2]),
    ];
    let sim = similarity.max(0.01);
    let sm = smoothness.max(0.01);
    img.px.par_chunks_mut(4).for_each(|p| {
        let a = p[3];
        if a <= 0.0 {
            return;
        }
        let rgb = [p[0] / a, p[1] / a, p[2] / a];
        let dist = ((rgb[0] - key_lin[0]).powi(2)
            + (rgb[1] - key_lin[1]).powi(2)
            + (rgb[2] - key_lin[2]).powi(2))
        .sqrt();
        let matte = ((dist - sim) / sm).clamp(0.0, 1.0);
        let na = a * matte;
        let g = if spill_suppress > 0.0 {
            let max_rb = rgb[0].max(rgb[2]);
            rgb[1] - (rgb[1] - max_rb).max(0.0) * spill_suppress.clamp(0.0, 1.0)
        } else {
            rgb[1]
        };
        p[0] = rgb[0] * na;
        p[1] = g * na;
        p[2] = rgb[2] * na;
        p[3] = na;
    });
}
