//! Định nghĩa các phép toán GPU chuyên biệt cho Darkworks Node Graph (`NodeFxOp`) và hàm tham chiếu CPU.
//!
//! Tách riêng khỏi `gpufx.rs` để giữ code FilmCraft nguyên bản gọn sạch và bảo đảm đối chiếu 1:1
//! giữa CPU reference (`apply`) với shader WGSL tại `crates/gpu/src/node_fx.wgsl`.

use darkworks_node_graph_core::KeyerOutput;
use filmcraft_color::{luma709, srgb_to_linear};
use rayon::prelude::*;

use crate::effects::{box_blur, dec, enc, gaussian_boxes, mosaic};
use crate::image::Image;
use crate::nodes::node_fx_distort_cpu::{
    chromatic_aberration_cpu, find_edges_cpu, lens_distort_cpu, noise_grain_cpu, radial_zoom_cpu,
    ripple_wave_cpu, turbulent_displace_cpu, twirl_cpu,
};

pub const KEYER_OUT_COMPOSITE: u8 = 0;
pub const KEYER_OUT_ALPHA: u8 = 1;
pub const KEYER_OUT_COLOR: u8 = 2;

#[inline]
pub(crate) const fn keyer_output_code(output: KeyerOutput) -> u8 {
    match output {
        KeyerOutput::Composite => KEYER_OUT_COMPOSITE,
        KeyerOutput::AlphaChannel => KEYER_OUT_ALPHA,
        KeyerOutput::ColorChannel => KEYER_OUT_COLOR,
    }
}

/// Các phép toán hiệu ứng GPU dành riêng cho Darkworks Node Graph.
#[derive(Clone, Debug, PartialEq)]
pub enum NodeFxOp {
    ChromaKeyPro {
        key_color: [f32; 3],
        similarity: f32,
        smoothness: f32,
        spill_suppress: f32,
        pedestal: f32,
        highlight: f32,
        shadow: f32,
        choke: f32,
        soften: f32,
        matte_contrast: f32,
        invert_matte: bool,
        desaturate: f32,
        spill_luma: f32,
        output: u8,
        px_scale: f32,
    },
    LumaKey {
        threshold: f32,
        cutoff: f32,
        choke: f32,
        soften: f32,
        matte_contrast: f32,
        invert_matte: bool,
        output: u8,
        px_scale: f32,
    },
    RadialZoom {
        strength: f32,
        cx: f32,
        cy: f32,
        taps: u32,
    },
    Pixelate {
        block_w: u32,
        block_h: u32,
    },
    ChromaticAberration {
        shift: f32,
    },
    RippleWave {
        amp: f32,
        freq: f32,
        phase: f32,
    },
    LensDistort {
        k: f32,
        cx: f32,
        cy: f32,
        norm: f32,
    },
    Twirl {
        ang: f32,
        rad: f32,
        cx: f32,
        cy: f32,
    },
    FindEdges {
        invert: bool,
        blend: f32,
    },
    NoiseGrain {
        amt: f32,
        grain: f32,
        color_mix: f32,
        seed: u32,
    },
    TurbulentDisplace {
        amt: f32,
        size: f32,
        ox: f32,
        oy: f32,
        evo: f32,
    },
}

impl NodeFxOp {
    pub fn gpu_ok(&self) -> bool {
        let fin = |v: &[f32]| v.iter().all(|x| x.is_finite());
        match self {
            Self::ChromaKeyPro {
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
                desaturate,
                spill_luma,
                px_scale,
                output,
                ..
            } => {
                *output <= 2
                    && fin(key_color)
                    && fin(&[
                        *similarity,
                        *smoothness,
                        *spill_suppress,
                        *pedestal,
                        *highlight,
                        *shadow,
                        *choke,
                        *soften,
                        *matte_contrast,
                        *desaturate,
                        *spill_luma,
                        *px_scale,
                    ])
            }
            Self::LumaKey {
                threshold,
                cutoff,
                choke,
                soften,
                matte_contrast,
                px_scale,
                output,
                ..
            } => {
                *output <= 2
                    && fin(&[
                        *threshold,
                        *cutoff,
                        *choke,
                        *soften,
                        *matte_contrast,
                        *px_scale,
                    ])
            }
            Self::RadialZoom { strength, cx, cy, taps } => *taps >= 2 && fin(&[*strength, *cx, *cy]),
            Self::Pixelate { block_w, block_h } => *block_w >= 1 && *block_h >= 1,
            Self::ChromaticAberration { shift } => shift.is_finite(),
            Self::RippleWave { amp, freq, phase } => fin(&[*amp, *freq, *phase]),
            Self::LensDistort { k, cx, cy, norm } => fin(&[*k, *cx, *cy, *norm]) && *norm > 1e-4,
            Self::Twirl { ang, rad, cx, cy } => fin(&[*ang, *rad, *cx, *cy]),
            Self::FindEdges { blend, .. } => blend.is_finite(),
            Self::NoiseGrain { amt, grain, color_mix, .. } => fin(&[*amt, *grain, *color_mix]) && *grain >= 1.0,
            Self::TurbulentDisplace { amt, size, ox, oy, evo } => fin(&[*amt, *size, *ox, *oy, *evo]) && *size >= 1.0,
        }
    }

    pub fn apply(&self, img: &mut Image) {
        if img.w == 0 || img.h == 0 {
            return;
        }
        match self {
            Self::ChromaKeyPro {
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
            } => chroma_key_pro_cpu(
                img,
                *key_color,
                *similarity,
                *smoothness,
                *spill_suppress,
                *pedestal,
                *highlight,
                *shadow,
                *choke,
                *soften,
                *matte_contrast,
                *invert_matte,
                *desaturate,
                *spill_luma,
                *output,
                *px_scale,
            ),
            Self::LumaKey {
                threshold,
                cutoff,
                choke,
                soften,
                matte_contrast,
                invert_matte,
                output,
                px_scale,
            } => luma_key_cpu(
                img,
                *threshold,
                *cutoff,
                *choke,
                *soften,
                *matte_contrast,
                *invert_matte,
                *output,
                *px_scale,
            ),
            Self::RadialZoom { strength, cx, cy, taps } => radial_zoom_cpu(img, *strength, *cx, *cy, *taps),
            Self::Pixelate { block_w, block_h } => mosaic(img, *block_w as usize, *block_h as usize),
            Self::ChromaticAberration { shift } => chromatic_aberration_cpu(img, *shift),
            Self::RippleWave { amp, freq, phase } => ripple_wave_cpu(img, *amp, *freq, *phase),
            Self::LensDistort { k, cx, cy, norm } => lens_distort_cpu(img, *k, *cx, *cy, *norm),
            Self::Twirl { ang, rad, cx, cy } => twirl_cpu(img, *ang, *rad, *cx, *cy),
            Self::FindEdges { invert, blend } => find_edges_cpu(img, *invert, *blend),
            Self::NoiseGrain { amt, grain, color_mix, seed } => noise_grain_cpu(img, *amt, *grain, *color_mix, *seed),
            Self::TurbulentDisplace { amt, size, ox, oy, evo } => turbulent_displace_cpu(img, *amt, *size, *ox, *oy, *evo),
        }
    }
}

#[inline]
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0).max(1e-5)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[derive(Clone, Copy)]
struct ChromaMatteParams {
    k_y: f32,
    k_cbcr: [f32; 2],
    kmag2: f32,
    tol: f32,
    gain: f32,
    pedestal: f32,
    highlight: f32,
    shadow: f32,
}

#[inline]
fn chroma_key_pro_matte(px: [f32; 4], m_p: ChromaMatteParams) -> f32 {
    let a = px[3];
    if a <= 1e-6 {
        return 0.0;
    }
    let v = enc([px[0] / a, px[1] / a, px[2] / a]);
    let ycc = filmcraft_color::rgb_to_ycbcr(v[0], v[1], v[2], filmcraft_color::Matrix::Bt709);
    let proj = (ycc[1] * m_p.k_cbcr[0] + ycc[2] * m_p.k_cbcr[1]) / m_p.kmag2;
    let d_cb = ycc[1] - proj * m_p.k_cbcr[0];
    let d_cr = ycc[2] - proj * m_p.k_cbcr[1];
    let perp = (d_cb * d_cb + d_cr * d_cr).sqrt() / m_p.kmag2.sqrt();
    let keyness = proj.clamp(0.0, 1.5) * (1.0 - smoothstep(0.0, m_p.tol, perp));
    let mut m = (1.0 - keyness * m_p.gain).clamp(0.0, 1.0);
    m += (ycc[0] - m_p.k_y).max(0.0) * m_p.highlight * 2.0 * keyness.min(1.0);
    m += (m_p.k_y - ycc[0]).max(0.0) * m_p.shadow * 2.0 * keyness.min(1.0);
    let ped = m_p.pedestal.clamp(0.0, 0.90);
    ((m.clamp(0.0, 1.0) - ped) / (1.0 - ped)).clamp(0.0, 1.0)
}

#[inline]
fn luma_key_matte(px: [f32; 4], lo: f32, hi: f32) -> f32 {
    let a = px[3];
    if a <= 1e-6 {
        return 0.0;
    }
    let v = enc([px[0] / a, px[1] / a, px[2] / a]);
    let y = luma709(v[0], v[1], v[2]);
    smoothstep(lo, hi, y)
}

fn cleanup_matte_plane(
    matte: &mut [f32],
    w: usize,
    h: usize,
    choke: f32,
    soften: f32,
    px_scale: f32,
) {
    let choke = choke.clamp(0.0, 1.0);
    let soften = soften.clamp(0.0, 1.0);
    let ps = px_scale.max(0.01);
    if choke > 1e-4 {
        let r = (choke * 4.0 * ps).max(0.5);
        let (rx, ry) = gaussian_boxes(w, h, r, r);
        let mut blur_img = Image {
            w,
            h,
            px: matte.iter().flat_map(|&a| [a, 0.0, 0.0, 0.0]).collect(),
        };
        box_blur(&mut blur_img, &rx, &ry, false);
        let c = (choke * 0.5).min(0.95);
        for (a, chunk) in matte.iter_mut().zip(blur_img.px.as_chunks::<4>().0.iter()) {
            let choked = ((chunk[0] - c) / (1.0 - c)).clamp(0.0, 1.0);
            *a = a.min(choked);
        }
    }
    if soften > 1e-4 {
        let r = soften * 10.0 * ps;
        let (rx, ry) = gaussian_boxes(w, h, r, r);
        let mut blur_img = Image {
            w,
            h,
            px: matte.iter().flat_map(|&a| [a, 0.0, 0.0, 0.0]).collect(),
        };
        box_blur(&mut blur_img, &rx, &ry, false);
        for (a, chunk) in matte.iter_mut().zip(blur_img.px.as_chunks::<4>().0.iter()) {
            *a = chunk[0].clamp(0.0, 1.0);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn chroma_key_pro_cpu(
    img: &mut Image,
    key_color: [f32; 3],
    similarity: f32,
    smoothness: f32,
    spill_suppress: f32,
    pedestal: f32,
    highlight: f32,
    shadow: f32,
    choke: f32,
    soften: f32,
    matte_contrast: f32,
    invert_matte: bool,
    desaturate: f32,
    spill_luma: f32,
    output: u8,
    px_scale: f32,
) {
    let k = filmcraft_color::rgb_to_ycbcr(
        key_color[0],
        key_color[1],
        key_color[2],
        filmcraft_color::Matrix::Bt709,
    );
    let kmag2 = (k[1] * k[1] + k[2] * k[2]).max(1e-6);
    let tol = 0.08 + smoothness.clamp(0.0, 1.0) * 0.50;
    let gain = 1.0 + 2.0 * similarity.clamp(0.0, 1.0);
    let ped = pedestal.clamp(0.0, 0.90);
    let hl = highlight.clamp(0.0, 1.0);
    let sh = shadow.clamp(0.0, 1.0);
    let spill = spill_suppress.clamp(0.0, 1.0);
    let desat = desaturate.clamp(0.0, 1.0);
    let sluma = spill_luma.clamp(0.0, 1.0);
    let m_contrast = matte_contrast.clamp(0.0, 1.0);
    let dom = if key_color[1] >= key_color[0] && key_color[1] >= key_color[2] {
        1
    } else if key_color[2] >= key_color[0] {
        2
    } else {
        0
    };
    let (w, h) = (img.w, img.h);
    let src = img.clone();
    let k_cbcr = [k[1], k[2]];

    let matte_p = ChromaMatteParams {
        k_y: k[0],
        k_cbcr,
        kmag2,
        tol,
        gain,
        pedestal: ped,
        highlight: hl,
        shadow: sh,
    };

    let has_blur_cleanup = choke > 1e-4 || soften > 1e-4;
    let mut matte = vec![0.0f32; w * h];
    matte.par_chunks_mut(w).enumerate().for_each(|(y, row)| {
        let (xm, ym) = (w.saturating_sub(1) as isize, h.saturating_sub(1) as isize);
        for (x, out_a) in row.iter_mut().enumerate() {
            let o = src.get(x, y);
            let mut a = chroma_key_pro_matte(o, matte_p);
            if !has_blur_cleanup && a > 0.0 && a < 0.98 {
                let mut nb_min = 1.0f32;
                for (dx, dy) in [(-1isize, 0), (1, 0), (0, -1), (0, 1)] {
                    let nx = (x as isize + dx).clamp(0, xm) as usize;
                    let ny = (y as isize + dy).clamp(0, ym) as usize;
                    nb_min = nb_min.min(chroma_key_pro_matte(src.get(nx, ny), matte_p));
                }
                if nb_min < 0.05 {
                    a = a.min(0.5 * (a + nb_min));
                }
            }
            *out_a = a;
        }
    });

    if has_blur_cleanup {
        cleanup_matte_plane(&mut matte, w, h, choke, soften, px_scale);
    }

    img.px
        .par_chunks_mut(w * 4)
        .zip(matte.par_chunks(w))
        .enumerate()
        .for_each(|(y, (row, mrow))| {
            for x in 0..w {
                let o = src.get(x, y);
                let src_a = o[3];
                if src_a <= 1e-6 {
                    continue;
                }
                let mut a = mrow[x];
                if m_contrast > 1e-4 {
                    let g = 1.0 + m_contrast * 4.0;
                    a = ((a - 0.5) * g + 0.5).clamp(0.0, 1.0);
                }
                if invert_matte {
                    a = 1.0 - a;
                }
                let mut v = enc([o[0] / src_a, o[1] / src_a, o[2] / src_a]);
                let (o1, o2) = (v[(dom + 1) % 3], v[(dom + 2) % 3]);
                let limit = o1.max(o2);
                let excess = (v[dom] - limit).max(0.0);
                if excess > 0.0 && spill > 0.0 {
                    let l0 = luma709(v[0], v[1], v[2]);
                    v[dom] -= excess * spill;
                    let l1 = luma709(v[0], v[1], v[2]);
                    let lm = l1;
                    let k_desat = desat * (excess * 4.0).min(1.0);
                    v = v.map(|q| lm + (q - lm) * (1.0 - k_desat));
                    let comp = (l0 - l1) * sluma;
                    v = v.map(|q| q + comp);
                }
                if a < 0.98 && spill > 0.0 {
                    let lm = luma709(v[0], v[1], v[2]);
                    let k_edge = (1.0 - a) * spill * (0.35 + desat * 0.45);
                    v = v.map(|q| q + (lm - q) * k_edge);
                }
                let p = &mut row[x * 4..x * 4 + 4];
                let res = dec(v);
                match output {
                    KEYER_OUT_ALPHA => {
                        let g = srgb_to_linear(a);
                        p.copy_from_slice(&[g * src_a, g * src_a, g * src_a, src_a]);
                    }
                    KEYER_OUT_COLOR => {
                        p.copy_from_slice(&[res[0] * src_a, res[1] * src_a, res[2] * src_a, src_a]);
                    }
                    _ => {
                        let na = src_a * a;
                        p.copy_from_slice(&[res[0] * na, res[1] * na, res[2] * na, na]);
                    }
                }
            }
        });
}

#[allow(clippy::too_many_arguments)]
fn luma_key_cpu(
    img: &mut Image,
    threshold: f32,
    cutoff: f32,
    choke: f32,
    soften: f32,
    matte_contrast: f32,
    invert_matte: bool,
    output: u8,
    px_scale: f32,
) {
    let lo = threshold.clamp(0.0, 1.0);
    let hi = (lo + cutoff.max(0.01)).min(1.5);
    let m_contrast = matte_contrast.clamp(0.0, 1.0);
    let (w, h) = (img.w, img.h);
    let src = img.clone();
    let mut matte: Vec<f32> = src
        .px
        .par_chunks_exact(4)
        .map(|p| luma_key_matte([p[0], p[1], p[2], p[3]], lo, hi))
        .collect();
    if choke > 1e-4 || soften > 1e-4 {
        cleanup_matte_plane(&mut matte, w, h, choke, soften, px_scale);
    }
    img.px
        .par_chunks_mut(w * 4)
        .zip(matte.par_chunks(w))
        .enumerate()
        .for_each(|(y, (row, mrow))| {
            for x in 0..w {
                let o = src.get(x, y);
                let src_a = o[3];
                if src_a <= 1e-6 {
                    continue;
                }
                let mut a = mrow[x];
                if m_contrast > 1e-4 {
                    let g = 1.0 + m_contrast * 4.0;
                    a = ((a - 0.5) * g + 0.5).clamp(0.0, 1.0);
                }
                if invert_matte {
                    a = 1.0 - a;
                }
                let p = &mut row[x * 4..x * 4 + 4];
                match output {
                    KEYER_OUT_ALPHA => {
                        let g = srgb_to_linear(a);
                        p.copy_from_slice(&[g * src_a, g * src_a, g * src_a, src_a]);
                    }
                    KEYER_OUT_COLOR => p.copy_from_slice(&o),
                    _ => p.copy_from_slice(&[o[0] * a, o[1] * a, o[2] * a, src_a * a]),
                }
            }
        });
}
