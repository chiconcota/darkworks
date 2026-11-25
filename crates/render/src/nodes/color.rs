//! Xử lý chỉnh màu & tách phông cho `ColorGrade` và `Keyer` Nodes.

use filmcraft_color::srgb_to_linear;
use rayon::prelude::*;

use crate::gpufx::FxOp;
use crate::image::Image;

#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_color_grade(
    img: &mut Image,
    exposure: f32,
    contrast: f32,
    saturation: f32,
    temperature: f32,
    tint: f32,
    gamma: f32,
    invert: bool,
    black_and_white: bool,
    tint_color: [f32; 4],
    tint_amount: f32,
) {
    // 1. B&W và Invert chạy trước để các bước chỉnh màu/nhuộm màu tiếp theo hoạt động đúng trên kết quả
    if black_and_white {
        FxOp::BlackWhite.apply(img);
    }
    if invert {
        FxOp::Invert { channel: 0, blend: 0.0 }.apply(img);
    }
    if exposure.abs() > 1e-4 || (contrast - 1.0).abs() > 1e-4 || (saturation - 1.0).abs() > 1e-4 {
        FxOp::ProcAmp {
            br: exposure * 0.25,
            co: contrast.max(0.0),
            hue: 0.0,
            sat: saturation.max(0.0),
        }
        .apply(img);
    }
    if temperature.abs() > 1e-4 || tint.abs() > 1e-4 {
        let t = temperature * 0.2;
        let tn = tint * 0.2;
        FxOp::ColorBalance {
            sh: [t, -tn, -t],
            md: [t, -tn, -t],
            hi: [t, -tn, -t],
            preserve: true,
        }
        .apply(img);
    }
    if (gamma - 1.0).abs() > 1e-4 {
        FxOp::Gamma { g: gamma.clamp(0.1, 5.0) }.apply(img);
    }
    if tint_amount > 1e-4 {
        FxOp::Tint {
            black: [0.0, 0.0, 0.0],
            white: [tint_color[0], tint_color[1], tint_color[2]],
            amount: tint_amount.clamp(0.0, 1.0),
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
