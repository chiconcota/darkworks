//! Hàm tham chiếu CPU cho các [`NodeFxOp`](super::node_fx_ops::NodeFxOp) nhóm quang học / biến dạng
//! (khớp 1:1 với `node_fx.wgsl`).

use filmcraft_color::luma709;
use rayon::prelude::*;

use crate::effects::{dec, enc};
use crate::image::Image;

pub(crate) fn radial_zoom_cpu(img: &mut Image, strength: f32, cx: f32, cy: f32, taps: u32) {
    if strength.abs() < 1e-4 || taps < 2 {
        return;
    }
    let src = img.clone();
    let w = img.w;
    let tcount = taps as usize;
    let inv = 1.0 / tcount as f32;
    let denom = (tcount - 1) as f32;
    img.px.par_chunks_mut(w * 4).enumerate().for_each(|(y, row)| {
        let py = y as f32 + 0.5;
        let dy = cy - py;
        for x in 0..w {
            let px = x as f32 + 0.5;
            let dx = cx - px;
            let mut acc = [0.0f32; 4];
            for s in 0..tcount {
                let frac = s as f32 / denom * strength;
                let p = src.sample_bilinear_clamped(px + dx * frac, py + dy * frac);
                for k in 0..4 {
                    acc[k] += p[k];
                }
            }
            row[x * 4..x * 4 + 4].copy_from_slice(&[acc[0] * inv, acc[1] * inv, acc[2] * inv, acc[3] * inv]);
        }
    });
}

pub(crate) fn chromatic_aberration_cpu(img: &mut Image, shift: f32) {
    if shift.abs() < 0.1 {
        return;
    }
    let src = img.clone();
    let w = img.w;
    img.px.par_chunks_mut(w * 4).enumerate().for_each(|(y, row)| {
        let py = y as f32 + 0.5;
        for x in 0..w {
            let px = x as f32 + 0.5;
            let r = src.sample_bilinear_clamped(px + shift, py)[0];
            let g_a = src.get(x, y);
            let b = src.sample_bilinear_clamped(px - shift, py)[2];
            row[x * 4..x * 4 + 4].copy_from_slice(&[r, g_a[1], b, g_a[3]]);
        }
    });
}

pub(crate) fn ripple_wave_cpu(img: &mut Image, amp: f32, freq: f32, phase: f32) {
    let src = img.clone();
    let w = img.w;
    img.px.par_chunks_mut(w * 4).enumerate().for_each(|(y, row)| {
        let py = y as f32 + 0.5;
        for x in 0..w {
            let px = x as f32 + 0.5;
            let off_y = amp * (px * freq - phase).sin();
            let p = src.sample_bilinear(px, py + off_y);
            row[x * 4..x * 4 + 4].copy_from_slice(&p);
        }
    });
}

pub(crate) fn lens_distort_cpu(img: &mut Image, k: f32, cx: f32, cy: f32, norm: f32) {
    let src = img.clone();
    let w = img.w;
    let inv_norm = 1.0 / norm.max(1e-4);
    img.px.par_chunks_mut(w * 4).enumerate().for_each(|(y, row)| {
        let py = y as f32 + 0.5;
        let dy = (py - cy) * inv_norm;
        for x in 0..w {
            let px = x as f32 + 0.5;
            let dx = (px - cx) * inv_norm;
            let r2 = dx * dx + dy * dy;
            let s = 1.0 - k * r2;
            let p = src.sample_bilinear(cx + dx * s * norm, cy + dy * s * norm);
            row[x * 4..x * 4 + 4].copy_from_slice(&p);
        }
    });
}

pub(crate) fn twirl_cpu(img: &mut Image, ang: f32, rad: f32, cx: f32, cy: f32) {
    let src = img.clone();
    let w = img.w;
    let rad = rad.max(1e-4);
    img.px.par_chunks_mut(w * 4).enumerate().for_each(|(y, row)| {
        let py = y as f32 + 0.5;
        let dy = py - cy;
        for x in 0..w {
            let px = x as f32 + 0.5;
            let dx = px - cx;
            let d = dx.hypot(dy);
            let p = if d >= rad {
                src.sample_bilinear(px, py)
            } else {
                let k = 1.0 - d / rad;
                let (s, c) = (ang * k * k).sin_cos();
                src.sample_bilinear(cx + dx * c - dy * s, cy + dx * s + dy * c)
            };
            row[x * 4..x * 4 + 4].copy_from_slice(&p);
        }
    });
}

pub(crate) fn find_edges_cpu(img: &mut Image, invert: bool, blend: f32) {
    let blend = blend.clamp(0.0, 1.0);
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
            if !invert {
                m = 1.0 - m;
            }
            let o = src.get(x, y);
            let a = o[3];
            for k in 0..3 {
                row[x * 4 + k] = m * a * (1.0 - blend) + o[k] * blend;
            }
            row[x * 4 + 3] = a;
        }
    });
}

#[inline]
fn hash3_u32(x: u32, y: u32, z: u32) -> f32 {
    let mut h = x.wrapping_mul(0x9E37_79B9) ^ y.wrapping_mul(0x85EB_CA6B) ^ z.wrapping_mul(0xC2B2_AE35);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7FEB_352D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846C_A68B);
    h ^= h >> 16;
    (h >> 8) as f32 / 16_777_216.0
}

pub(crate) fn noise_grain_cpu(img: &mut Image, amt: f32, grain: f32, color_mix: f32, seed: u32) {
    if amt < 1e-4 {
        return;
    }
    let grain = grain.max(1.0);
    let color_mix = color_mix.clamp(0.0, 1.0);
    let w = img.w;
    img.px.par_chunks_mut(w * 4).enumerate().for_each(|(y, row)| {
        let gy = (y as f32 / grain) as u32;
        for x in 0..w {
            let p = &mut row[x * 4..x * 4 + 4];
            let a = p[3];
            if a <= 1e-6 {
                continue;
            }
            let gx = (x as f32 / grain) as u32;
            let n0 = hash3_u32(gx, gy, seed) - 0.5;
            let ns = [
                n0,
                n0 + (hash3_u32(gx, gy, seed.wrapping_add(7777)) - 0.5 - n0) * color_mix,
                n0 + (hash3_u32(gx, gy, seed.wrapping_add(99_991)) - 0.5 - n0) * color_mix,
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

#[inline]
fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[inline]
fn hash_u32(x: i32, y: i32, z: i32, seed: u32) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x9E37_79B9)
        ^ (y as u32).wrapping_mul(0x85EB_CA6B)
        ^ (z as u32).wrapping_mul(0xC2B2_AE35)
        ^ seed.wrapping_mul(0x27D4_EB2F);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7FEB_352D);
    h ^= h >> 15;
    h
}

#[inline]
fn grad3(h: u32, x: f32, y: f32, z: f32) -> f32 {
    let (u, v) = match h & 15 {
        0 => (x, y),
        1 => (-x, y),
        2 => (x, -y),
        3 => (-x, -y),
        4 => (x, z),
        5 => (-x, z),
        6 => (x, -z),
        7 => (-x, -z),
        8 => (y, z),
        9 => (-y, z),
        10 => (y, -z),
        11 => (-y, -z),
        12 => (y, x),
        13 => (-y, z),
        14 => (y, -x),
        _ => (-y, -z),
    };
    let s1 = if (h & 1) == 0 { u } else { -u };
    let s2 = if (h & 2) == 0 { v } else { -v };
    s1 + s2
}

fn noise3_u32(x: f32, y: f32, z: f32, seed: u32) -> f32 {
    let (xi, yi, zi) = (x.floor(), y.floor(), z.floor());
    let (xf, yf, zf) = (x - xi, y - yi, z - zi);
    let (ix, iy, iz) = (xi as i32, yi as i32, zi as i32);
    let (u, v, w) = (fade(xf), fade(yf), fade(zf));
    let g = |dx: i32, dy: i32, dz: i32| {
        grad3(
            hash_u32(ix + dx, iy + dy, iz + dz, seed),
            xf - dx as f32,
            yf - dy as f32,
            zf - dz as f32,
        )
    };
    let l = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let x00 = l(g(0, 0, 0), g(1, 0, 0), u);
    let x10 = l(g(0, 1, 0), g(1, 1, 0), u);
    let x01 = l(g(0, 0, 1), g(1, 0, 1), u);
    let x11 = l(g(0, 1, 1), g(1, 1, 1), u);
    l(l(x00, x10, v), l(x01, x11, v), w) * 0.9
}

fn fbm3(x: f32, y: f32, z: f32) -> f32 {
    let mut sum = 0.0f32;
    let mut amp = 1.0f32;
    let mut freq = 1.0f32;
    let mut norm = 0.0f32;
    for i in 0..3u32 {
        sum += noise3_u32(x * freq, y * freq, z * freq + i as f32 * 17.13, 17 + i) * amp;
        norm += amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    sum / norm
}

pub(crate) fn turbulent_displace_cpu(img: &mut Image, amt: f32, size: f32, ox: f32, oy: f32, evo: f32) {
    if amt.abs() < 0.05 {
        return;
    }
    let src = img.clone();
    let w = img.w;
    let inv_size = 1.0 / size.max(1.0);
    img.px.par_chunks_mut(w * 4).enumerate().for_each(|(y, row)| {
        let py = y as f32 + 0.5;
        let v = py * inv_size + oy;
        for x in 0..w {
            let px = x as f32 + 0.5;
            let u = px * inv_size + ox;
            let nx = fbm3(u, v, evo);
            let ny = fbm3(u + 31.7, v + 12.3, evo + 3.1);
            let p = src.sample_bilinear(px + nx * amt, py + ny * amt);
            row[x * 4..x * 4 + 4].copy_from_slice(&p);
        }
    });
}
