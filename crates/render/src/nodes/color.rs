//! Xử lý chỉnh màu & tách phông cho `ColorGrade` và `Keyer` Nodes.

use darkworks_node_graph_core::{
    DarkNode, EvaluationContext, KeyerMode, KeyerOutput, NodeGraphDoc,
};

use crate::gpufx::FxOp;
use crate::image::Image;
use crate::nodes::node_fx_ops::{KEYER_OUT_ALPHA, KEYER_OUT_COMPOSITE, NodeFxOp, keyer_output_code};
use crate::nodes::signals::modulated_scalar_add_scaled;
use crate::nodes::transform_blend::apply_mask_alpha;

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

fn lumetri_grade_op(p: &ColorGradeParams, size: [usize; 2], creative_on: bool) -> FxOp {
    let temp = (p.temperature / 100.0).clamp(-1.5, 1.5);
    let tint = (p.tint / 100.0).clamp(-1.5, 1.5);
    let aspect = if size[1] > 0 { (size[0] as f32 / size[1] as f32).max(1e-4) } else { 1.0 };
    FxOp::Lumetri {
        gains: [1.0 + 0.35 * temp, 1.0 - 0.3 * tint, 1.0 - 0.35 * temp],
        exposure: 2.0_f32.powf(p.exposure.clamp(-5.0, 5.0)),
        contrast: (p.contrast - 1.0).clamp(-1.0, 2.0),
        hl: p.highlights / 100.0,
        sh: p.shadows / 100.0,
        wh: p.whites / 100.0,
        bl: p.blacks / 100.0,
        sat: p.saturation.max(0.0),
        creative_on,
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
    let creative_on = has_split_tone || p.faded_film.abs() > 1e-3 || p.vibrance.abs() > 1e-3;
    let use_lumetri = p.highlights.abs() > 1e-3
        || p.shadows.abs() > 1e-3
        || p.whites.abs() > 1e-3
        || p.blacks.abs() > 1e-3
        || creative_on;

    if use_lumetri {
        lumetri_grade_op(p, [img.w, img.h], creative_on).apply(img);
    } else {
        if p.exposure.abs() > 1e-4 {
            let gain = 2.0_f32.powf(p.exposure.clamp(-5.0, 5.0));
            img.map_rgb(|c, _, _| [c[0] * gain, c[1] * gain, c[2] * gain]);
        }
        if (p.contrast - 1.0).abs() > 1e-4 || (p.saturation - 1.0).abs() > 1e-4 {
            FxOp::ProcAmp {
                br: 0.0,
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

fn lumetri_exposure_op(eff_exp: f32) -> FxOp {
    FxOp::Lumetri {
        gains: [1.0, 1.0, 1.0],
        exposure: 2.0_f32.powf(eff_exp.clamp(-5.0, 5.0)),
        contrast: 0.0,
        hl: 0.0,
        sh: 0.0,
        wh: 0.0,
        bl: 0.0,
        sat: 1.0,
        creative_on: false,
        faded: 0.0,
        vib: 0.0,
        st: [0.5; 3],
        ht: [0.5; 3],
        vignette_on: false,
        va: 0.0,
        vmid: 0.5,
        vround: 0.0,
        vfeather: 0.5,
        aspect: 1.0,
        gpu_capable: true,
    }
}

/// Biên dịch `DarkNode::ColorGrade` thành chuỗi [`FxOp`] chạy GPU.
pub(crate) fn push_color_grade_gpu_ops(
    doc: &NodeGraphDoc,
    sig_ctx: &EvaluationContext,
    id: usize,
    node: &DarkNode,
    size: [usize; 2],
    ops: &mut Vec<FxOp>,
) {
    let DarkNode::ColorGrade {
        exposure,
        contrast,
        saturation,
        temperature,
        tint,
        gamma,
        invert,
        black_and_white,
        tint_color,
        tint_amount,
        highlights,
        shadows,
        whites,
        blacks,
        vibrance,
        faded_film,
        shadow_tint,
        highlight_tint,
    } = node
    else {
        return;
    };
    if *black_and_white {
        ops.push(FxOp::BlackWhite);
    }
    if *invert {
        ops.push(FxOp::Invert { channel: 0, blend: 0.0 });
    }
    let eff_exp = modulated_scalar_add_scaled(doc, sig_ctx, id, 1, *exposure, 2.5);
    let has_split = shadow_tint.iter().any(|&v| (v - 0.5).abs() > 1e-3)
        || highlight_tint.iter().any(|&v| (v - 0.5).abs() > 1e-3);
    let creative_on = has_split || faded_film.abs() > 1e-3 || vibrance.abs() > 1e-3;
    let use_lum = highlights.abs() > 1e-3
        || shadows.abs() > 1e-3
        || whites.abs() > 1e-3
        || blacks.abs() > 1e-3
        || creative_on;
    if use_lum {
        let p = ColorGradeParams {
            exposure: eff_exp,
            contrast: *contrast,
            saturation: *saturation,
            temperature: *temperature,
            tint: *tint,
            gamma: *gamma,
            invert: *invert,
            black_and_white: *black_and_white,
            tint_color: *tint_color,
            tint_amount: *tint_amount,
            highlights: *highlights,
            shadows: *shadows,
            whites: *whites,
            blacks: *blacks,
            vibrance: *vibrance,
            faded_film: *faded_film,
            shadow_tint: *shadow_tint,
            highlight_tint: *highlight_tint,
        };
        ops.push(lumetri_grade_op(&p, size, creative_on));
    } else {
        if eff_exp.abs() > 1e-4 {
            ops.push(lumetri_exposure_op(eff_exp));
        }
        if (*contrast - 1.0).abs() > 1e-4 || (*saturation - 1.0).abs() > 1e-4 {
            ops.push(FxOp::ProcAmp {
                br: 0.0,
                co: contrast.max(0.0),
                hue: 0.0,
                sat: saturation.max(0.0),
            });
        }
        if temperature.abs() > 1e-4 || tint.abs() > 1e-4 {
            let (t0, tn0) = (*temperature * 0.2, *tint * 0.2);
            ops.push(FxOp::ColorBalance {
                sh: [t0, -tn0, -t0],
                md: [t0, -tn0, -t0],
                hi: [t0, -tn0, -t0],
                preserve: true,
            });
        }
    }
    if (*gamma - 1.0).abs() > 1e-4 {
        ops.push(FxOp::Gamma { g: gamma.clamp(0.1, 5.0) });
    }
    if *tint_amount > 1e-4 {
        ops.push(FxOp::Tint {
            black: [0.0, 0.0, 0.0],
            white: [tint_color[0], tint_color[1], tint_color[2]],
            amount: tint_amount.clamp(0.0, 1.0),
        });
    }
}

/// Tham số tách phông Pro của `Keyer` (`Ultra Keyer`).
#[derive(Debug, Clone, Copy)]
pub(crate) struct KeyerProParams {
    pub key_color: [f32; 3],
    pub similarity: f32,
    pub smoothness: f32,
    pub spill_suppress: f32,
    pub mode: KeyerMode,
    pub output: KeyerOutput,
    pub pedestal: f32,
    pub highlight: f32,
    pub shadow: f32,
    pub choke: f32,
    pub soften: f32,
    pub matte_contrast: f32,
    pub invert_matte: bool,
    pub desaturate: f32,
    pub spill_luma: f32,
    pub luma_threshold: f32,
    pub luma_cutoff: f32,
    pub px_scale: f32,
}

fn keyer_fx_op(p: &KeyerProParams, out_code: u8) -> FxOp {
    match p.mode {
        KeyerMode::ChromaKey => FxOp::Node(NodeFxOp::ChromaKeyPro {
            key_color: p.key_color,
            similarity: p.similarity,
            smoothness: p.smoothness,
            spill_suppress: p.spill_suppress,
            pedestal: p.pedestal,
            highlight: p.highlight,
            shadow: p.shadow,
            choke: p.choke,
            soften: p.soften,
            matte_contrast: p.matte_contrast,
            invert_matte: p.invert_matte,
            desaturate: p.desaturate,
            spill_luma: p.spill_luma,
            output: out_code,
            px_scale: p.px_scale,
        }),
        KeyerMode::LumaKey => FxOp::Node(NodeFxOp::LumaKey {
            threshold: p.luma_threshold,
            cutoff: p.luma_cutoff,
            choke: p.choke,
            soften: p.soften,
            matte_contrast: p.matte_contrast,
            invert_matte: p.invert_matte,
            output: out_code,
            px_scale: p.px_scale,
        }),
    }
}

/// Biên dịch `DarkNode::Keyer` thành [`FxOp`] chạy GPU.
pub(crate) fn push_keyer_gpu_op(node: &DarkNode, px_scale: f32, ops: &mut Vec<FxOp>) {
    let DarkNode::Keyer {
        key_color,
        similarity,
        smoothness,
        spill_suppress,
        mode,
        output,
        pedestal,
        highlight,
        shadow,
        choke,
        soften,
        matte_contrast,
        invert_matte,
        desaturate,
        spill_luma,
        luma_threshold,
        luma_cutoff,
    } = node
    else {
        return;
    };
    let p = KeyerProParams {
        key_color: *key_color,
        similarity: *similarity,
        smoothness: *smoothness,
        spill_suppress: *spill_suppress,
        mode: *mode,
        output: *output,
        pedestal: *pedestal,
        highlight: *highlight,
        shadow: *shadow,
        choke: *choke,
        soften: *soften,
        matte_contrast: *matte_contrast,
        invert_matte: *invert_matte,
        desaturate: *desaturate,
        spill_luma: *spill_luma,
        luma_threshold: *luma_threshold,
        luma_cutoff: *luma_cutoff,
        px_scale,
    };
    ops.push(keyer_fx_op(&p, keyer_output_code(*output)));
}

pub(crate) fn apply_keyer(
    img: &mut Image,
    p: &KeyerProParams,
    garbage_matte: Option<&Image>,
    need_matte_out: bool,
) -> Option<Image> {
    let matte_out = if need_matte_out {
        let mut m = img.clone();
        keyer_fx_op(p, KEYER_OUT_ALPHA).apply(&mut m);
        if let Some(gm) = garbage_matte {
            apply_mask_alpha(&mut m, gm);
        }
        // Đồng bộ alpha theo giá trị matte để Matte Out cắm thẳng được vào Blend Mask / Garbage Matte.
        for px in m.px.as_chunks_mut::<4>().0 {
            let a = px[0].clamp(0.0, 1.0);
            px[3] *= a;
        }
        Some(m)
    } else {
        None
    };

    keyer_fx_op(p, keyer_output_code(p.output)).apply(img);
    if let Some(gm) = garbage_matte
        && keyer_output_code(p.output) == KEYER_OUT_COMPOSITE
    {
        apply_mask_alpha(img, gm);
    }
    matte_out
}
