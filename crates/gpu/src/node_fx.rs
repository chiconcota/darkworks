//! Biên dịch và điều phối các [`NodeFxOp`] của Darkworks Node Graph sang `node_fx.wgsl`.

use filmcraft_render::effects::gaussian_boxes;
use filmcraft_render::nodes::NodeFxOp;

use crate::fx::{Step, StepPipeline, boxes, step};

const OP_NODE_CHROMA_KEY_PRO: u32 = 100;
const OP_NODE_KEYER_MATTE: u32 = 101;
const OP_NODE_KEYER_CHOKE: u32 = 102;
const OP_NODE_KEYER_FINISH: u32 = 103;
const OP_NODE_LUMA_KEY: u32 = 104;
const OP_NODE_RADIAL_ZOOM: u32 = 105;
const OP_NODE_PIXELATE: u32 = 106;
const OP_NODE_CHROMATIC_ABERRATION: u32 = 107;
const OP_NODE_RIPPLE_WAVE: u32 = 108;
const OP_NODE_LENS_DISTORT: u32 = 109;
const OP_NODE_TWIRL: u32 = 110;
const OP_NODE_FIND_EDGES: u32 = 111;
const OP_NODE_NOISE_GRAIN: u32 = 112;
const OP_NODE_TURBULENT_DISPLACE: u32 = 113;
const OP_NODE_RESAMPLE_AREA: u32 = 114;

fn node_step(code: u32, i1: [u32; 4], p: &[f32]) -> Step {
    let mut s = step(code, i1, p);
    s.pipeline = StepPipeline::NodePx;
    s
}

fn push_keyer_cleanup_passes(
    i1: [u32; 4],
    params: &[f32],
    choke: f32,
    soften: f32,
    px_scale: f32,
    size: (u32, u32),
    out: &mut Vec<Step>,
) {
    let (w, h) = size;
    let ps = px_scale.max(0.01);
    let mut s0 = node_step(OP_NODE_KEYER_MATTE, i1, params);
    s0.keep_orig = true;
    out.push(s0);

    if choke > 1e-4 {
        let r = (choke * 4.0 * ps).max(0.5);
        let (rx, ry) = gaussian_boxes(w as usize, h as usize, r, r);
        let start = out.len();
        boxes(&rx, false, false, out);
        boxes(&ry, true, false, out);
        for s in &mut out[start..] {
            s.keep_orig = true;
        }
        let mut s_choke = node_step(OP_NODE_KEYER_CHOKE, i1, params);
        s_choke.keep_orig = true;
        s_choke.read_orig = true;
        out.push(s_choke);
    }

    if soften > 1e-4 {
        let r = soften * 10.0 * ps;
        let (rx, ry) = gaussian_boxes(w as usize, h as usize, r, r);
        let start = out.len();
        boxes(&rx, false, false, out);
        boxes(&ry, true, false, out);
        for s in &mut out[start..] {
            s.keep_orig = true;
        }
    }

    let mut fin = node_step(OP_NODE_KEYER_FINISH, i1, params);
    fin.read_orig = true;
    fin.keep_orig = false;
    out.push(fin);
}

pub(crate) fn push_node_steps(op: &NodeFxOp, w: u32, h: u32, out: &mut Vec<Step>) {
    match op {
        NodeFxOp::ChromaKeyPro {
            key_color,
            similarity,
            smoothness,
            spill_suppress,
            pedestal,
            highlight,
            shadow,
            choke,
            soften,
            matte_contrast,
            invert_matte,
            desaturate,
            spill_luma,
            output,
            px_scale,
        } => {
            let dom = if key_color[1] >= key_color[0] && key_color[1] >= key_color[2] {
                1u32
            } else if key_color[2] >= key_color[0] {
                2u32
            } else {
                0u32
            };
            let kr = 0.2126f32;
            let kb = 0.0722f32;
            let kg = 1.0 - kr - kb;
            let ky = kr * key_color[0] + kg * key_color[1] + kb * key_color[2];
            let kcb = (key_color[2] - ky) / (2.0 * (1.0 - kb));
            let kcr = (key_color[0] - ky) / (2.0 * (1.0 - kr));
            let kmag2 = (kcb * kcb + kcr * kcr).max(1e-6);
            let tol = 0.08 + smoothness.clamp(0.0, 1.0) * 0.50;
            let gain = 1.0 + 2.0 * similarity.clamp(0.0, 1.0);
            let ped = pedestal.clamp(0.0, 0.90);
            let hl = highlight.clamp(0.0, 1.0);
            let sh = shadow.clamp(0.0, 1.0);
            let spill = spill_suppress.clamp(0.0, 1.0);
            let desat = desaturate.clamp(0.0, 1.0);
            let sluma = spill_luma.clamp(0.0, 1.0);
            let m_contrast = matte_contrast.clamp(0.0, 1.0);
            let choke_v = choke.clamp(0.0, 1.0);
            let soften_v = soften.clamp(0.0, 1.0);
            let choke_c = (choke_v * 0.5).min(0.95);
            let i1 = [0, dom, u32::from(*output), u32::from(*invert_matte)];
            let p = [
                kcb, kcr, kmag2, tol,
                gain, ped, hl, sh,
                ky, spill, desat, sluma,
                m_contrast, choke_c, 0.0, 0.0,
            ];
            if choke_v <= 1e-4 && soften_v <= 1e-4 {
                out.push(node_step(OP_NODE_CHROMA_KEY_PRO, i1, &p));
            } else {
                push_keyer_cleanup_passes(i1, &p, choke_v, soften_v, *px_scale, (w, h), out);
            }
        }
        NodeFxOp::LumaKey {
            threshold,
            cutoff,
            choke,
            soften,
            matte_contrast,
            invert_matte,
            output,
            px_scale,
        } => {
            let lo = threshold.clamp(0.0, 1.0);
            let hi = (lo + cutoff.max(0.01)).min(1.5);
            let choke_v = choke.clamp(0.0, 1.0);
            let soften_v = soften.clamp(0.0, 1.0);
            let choke_c = (choke_v * 0.5).min(0.95);
            let i1 = [1, 0, u32::from(*output), u32::from(*invert_matte)];
            let p = [
                0.0, 0.0, 1.0, 1.0,
                1.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0,
                matte_contrast.clamp(0.0, 1.0), choke_c, lo, hi,
            ];
            if choke_v <= 1e-4 && soften_v <= 1e-4 {
                out.push(node_step(OP_NODE_LUMA_KEY, i1, &p));
            } else {
                push_keyer_cleanup_passes(i1, &p, choke_v, soften_v, *px_scale, (w, h), out);
            }
        }
        NodeFxOp::RadialZoom { strength, cx, cy, taps } => {
            out.push(node_step(OP_NODE_RADIAL_ZOOM, [*taps, 0, 0, 0], &[*strength, *cx, *cy]));
        }
        NodeFxOp::Pixelate { block_w, block_h } => {
            out.push(node_step(OP_NODE_PIXELATE, [*block_w, *block_h, 0, 0], &[]));
        }
        NodeFxOp::ChromaticAberration { shift } => {
            out.push(node_step(OP_NODE_CHROMATIC_ABERRATION, [0; 4], &[*shift]));
        }
        NodeFxOp::RippleWave { amp, freq, phase } => {
            out.push(node_step(OP_NODE_RIPPLE_WAVE, [0; 4], &[*amp, *freq, *phase]));
        }
        NodeFxOp::LensDistort { k, cx, cy, norm } => {
            out.push(node_step(OP_NODE_LENS_DISTORT, [0; 4], &[*k, *cx, *cy, *norm]));
        }
        NodeFxOp::Twirl { ang, rad, cx, cy } => {
            out.push(node_step(OP_NODE_TWIRL, [0; 4], &[*ang, *rad, *cx, *cy]));
        }
        NodeFxOp::FindEdges { invert, blend } => {
            out.push(node_step(OP_NODE_FIND_EDGES, [u32::from(*invert), 0, 0, 0], &[*blend]));
        }
        NodeFxOp::NoiseGrain { amt, grain, color_mix, seed } => {
            out.push(node_step(OP_NODE_NOISE_GRAIN, [*seed, 0, 0, 0], &[*amt, *grain, *color_mix]));
        }
        NodeFxOp::TurbulentDisplace { amt, size, ox, oy, evo } => {
            out.push(node_step(OP_NODE_TURBULENT_DISPLACE, [0; 4], &[*amt, *size, *ox, *oy, *evo]));
        }
        NodeFxOp::ResampleArea { inv, rect, opacity, taps } => {
            out.push(node_step(
                OP_NODE_RESAMPLE_AREA,
                *rect,
                &[inv[0], inv[1], inv[2], inv[3], inv[4], inv[5], *opacity, *taps as f32],
            ));
        }
    }
}
