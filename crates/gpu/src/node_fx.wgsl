// Darkworks Node Graph GPU Compute Shaders (`NodeFxOp`).
// Chia sẻ BindGroupLayout & uniform layout `U` với `fx.wgsl`.

struct U {
    i0: vec4<u32>,  // op, width, height, unused
    i1: vec4<u32>,  // op-specific integers
    p0: vec4<f32>,
    p1: vec4<f32>,
    p2: vec4<f32>,
    p3: vec4<f32>,
    p4: vec4<f32>,
    p5: vec4<f32>,
};

@group(0) @binding(0) var<uniform> u: U;
@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(2) var dst: texture_storage_2d<rgba32float, write>;
@group(0) @binding(3) var aux: texture_2d<f32>;

const OP_NODE_CHROMA_KEY_PRO: u32 = 100u;
const OP_NODE_KEYER_MATTE: u32 = 101u;
const OP_NODE_KEYER_CHOKE: u32 = 102u;
const OP_NODE_KEYER_FINISH: u32 = 103u;
const OP_NODE_LUMA_KEY: u32 = 104u;
const OP_NODE_RADIAL_ZOOM: u32 = 105u;
const OP_NODE_PIXELATE: u32 = 106u;
const OP_NODE_CHROMATIC_ABERRATION: u32 = 107u;
const OP_NODE_RIPPLE_WAVE: u32 = 108u;
const OP_NODE_LENS_DISTORT: u32 = 109u;
const OP_NODE_TWIRL: u32 = 110u;
const OP_NODE_FIND_EDGES: u32 = 111u;
const OP_NODE_NOISE_GRAIN: u32 = 112u;
const OP_NODE_TURBULENT_DISPLACE: u32 = 113u;
const OP_NODE_RESAMPLE_AREA: u32 = 114u;

fn size() -> vec2<i32> {
    return vec2<i32>(i32(u.i0.y), i32(u.i0.z));
}

fn ld(p: vec2<i32>) -> vec4<f32> {
    return textureLoad(src, p, 0);
}

fn ld_aux(p: vec2<i32>) -> vec4<f32> {
    return textureLoad(aux, p, 0);
}

fn ld_clamped(p: vec2<i32>) -> vec4<f32> {
    let s = size();
    return textureLoad(src, clamp(p, vec2(0), s - vec2(1)), 0);
}

fn ld_or_clear(p: vec2<i32>) -> vec4<f32> {
    let s = size();
    if p.x < 0 || p.y < 0 || p.x >= s.x || p.y >= s.y {
        return vec4(0.0);
    }
    return textureLoad(src, p, 0);
}

fn sample_bilinear(x: f32, y: f32) -> vec4<f32> {
    let fx = x - 0.5;
    let fy = y - 0.5;
    let x0 = floor(fx);
    let y0 = floor(fy);
    let tx = fx - x0;
    let ty = fy - y0;
    let i = vec2<i32>(i32(x0), i32(y0));
    let a = ld_or_clear(i);
    let b = ld_or_clear(i + vec2(1, 0));
    let c = ld_or_clear(i + vec2(0, 1));
    let d = ld_or_clear(i + vec2(1, 1));
    let top = a + (b - a) * tx;
    let bot = c + (d - c) * tx;
    return top + (bot - top) * ty;
}

fn sample_clamped(x: f32, y: f32) -> vec4<f32> {
    let s = size();
    let fx = clamp(x - 0.5, 0.0, f32(s.x) - 1.0);
    let fy = clamp(y - 0.5, 0.0, f32(s.y) - 1.0);
    let x0 = i32(floor(fx));
    let y0 = i32(floor(fy));
    let x1 = min(x0 + 1, s.x - 1);
    let y1 = min(y0 + 1, s.y - 1);
    let tx = fx - f32(x0);
    let ty = fy - f32(y0);
    let a = ld(vec2(x0, y0));
    let b = ld(vec2(x1, y0));
    let c = ld(vec2(x0, y1));
    let d = ld(vec2(x1, y1));
    let top = a + (b - a) * tx;
    let bot = c + (d - c) * tx;
    return top + (bot - top) * ty;
}

fn linear_to_srgb1(v: f32) -> f32 {
    if v <= 0.0031308 {
        return v * 12.92;
    }
    return 1.055 * pow(v, 1.0 / 2.4) - 0.055;
}

fn srgb_to_linear1(v: f32) -> f32 {
    if v <= 0.04045 {
        return v / 12.92;
    }
    return pow((v + 0.055) / 1.055, 2.4);
}

fn enc(c: vec3<f32>) -> vec3<f32> {
    let m = max(c, vec3(0.0));
    return vec3(linear_to_srgb1(m.x), linear_to_srgb1(m.y), linear_to_srgb1(m.z));
}

fn dec(c: vec3<f32>) -> vec3<f32> {
    let m = clamp(c, vec3(0.0), vec3(1.0));
    return vec3(srgb_to_linear1(m.x), srgb_to_linear1(m.y), srgb_to_linear1(m.z));
}

fn luma709(c: vec3<f32>) -> f32 {
    return 0.2126 * c.x + 0.7152 * c.y + 0.0722 * c.z;
}

fn smoothstep_fx(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = clamp((x - edge0) / max(edge1 - edge0, 1e-5), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

fn chroma_pro_matte(px: vec4<f32>) -> f32 {
    if px.a <= 1e-6 {
        return 0.0;
    }
    let v = enc(px.rgb / px.a);
    let kr = 0.2126;
    let kb = 0.0722;
    let kg = 1.0 - kr - kb;
    let y = kr * v.x + kg * v.y + kb * v.z;
    let cb = (v.z - y) / (2.0 * (1.0 - kb));
    let cr = (v.x - y) / (2.0 * (1.0 - kr));
    let k_cbcr = u.p0.xy;
    let kmag2 = u.p0.z;
    let tol = u.p0.w;
    let gain = u.p1.x;
    let ped = u.p1.y;
    let hl = u.p1.z;
    let sh = u.p1.w;
    let k_y = u.p2.x;
    let proj = (cb * k_cbcr.x + cr * k_cbcr.y) / kmag2;
    let d_cb = cb - proj * k_cbcr.x;
    let d_cr = cr - proj * k_cbcr.y;
    let perp = sqrt(d_cb * d_cb + d_cr * d_cr) / sqrt(kmag2);
    let keyness = clamp(proj, 0.0, 1.5) * (1.0 - smoothstep_fx(0.0, tol, perp));
    var m = clamp(1.0 - keyness * gain, 0.0, 1.0);
    m += max(y - k_y, 0.0) * hl * 2.0 * min(keyness, 1.0);
    m += max(k_y - y, 0.0) * sh * 2.0 * min(keyness, 1.0);
    return clamp((clamp(m, 0.0, 1.0) - ped) / (1.0 - ped), 0.0, 1.0);
}

fn luma_matte(px: vec4<f32>) -> f32 {
    if px.a <= 1e-6 {
        return 0.0;
    }
    let v = enc(px.rgb / px.a);
    let y = luma709(v);
    return smoothstep_fx(u.p3.z, u.p3.w, y);
}

fn finish_keyer(o: vec4<f32>, raw_a: f32) -> vec4<f32> {
    if o.a <= 1e-6 {
        return o;
    }
    var a = raw_a;
    let m_contrast = u.p3.x;
    if m_contrast > 1e-4 {
        let g = 1.0 + m_contrast * 4.0;
        a = clamp((a - 0.5) * g + 0.5, 0.0, 1.0);
    }
    if u.i1.w != 0u {
        a = 1.0 - a;
    }
    let out_mode = u.i1.z;
    if u.i1.x == 0u {
        let dom = u.i1.y;
        let spill = u.p2.y;
        let desat = u.p2.z;
        let sluma = u.p2.w;
        var v = enc(o.rgb / o.a);
        let limit = select(select(max(v.x, v.y), max(v.x, v.z), dom == 1u), max(v.y, v.z), dom == 0u);
        let dom_val = select(select(v.z, v.y, dom == 1u), v.x, dom == 0u);
        let excess = max(dom_val - limit, 0.0);
        if excess > 0.0 && spill > 0.0 {
            let l0 = luma709(v);
            if dom == 0u { v.x -= excess * spill; }
            else if dom == 1u { v.y -= excess * spill; }
            else { v.z -= excess * spill; }
            let l1 = luma709(v);
            let k_desat = desat * min(excess * 4.0, 1.0);
            v = mix(v, vec3(l1), k_desat);
            v = v + vec3((l0 - l1) * sluma);
        }
        if a < 0.98 && spill > 0.0 {
            let lm = luma709(v);
            let k_edge = (1.0 - a) * spill * (0.35 + desat * 0.45);
            v = mix(v, vec3(lm), k_edge);
        }
        let res = dec(v);
        if out_mode == 1u {
            let g = srgb_to_linear1(a);
            return vec4(vec3(g * o.a), o.a);
        } else if out_mode == 2u {
            return vec4(res * o.a, o.a);
        } else {
            let na = o.a * a;
            return vec4(res * na, na);
        }
    } else {
        if out_mode == 1u {
            let g = srgb_to_linear1(a);
            return vec4(vec3(g * o.a), o.a);
        } else if out_mode == 2u {
            return o;
        } else {
            return vec4(o.rgb * a, o.a * a);
        }
    }
}

fn hash3_u32(x: u32, y: u32, z: u32) -> f32 {
    var h = (x * 0x9E3779B9u) ^ (y * 0x85EBCA6Bu) ^ (z * 0xC2B2AE35u);
    h ^= h >> 16u;
    h *= 0x7FEB352Du;
    h ^= h >> 15u;
    h *= 0x846CA68Bu;
    h ^= h >> 16u;
    return f32(h >> 8u) / 16777216.0;
}

fn fade(t: f32) -> f32 {
    return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
}

fn hash_u32(x: i32, y: i32, z: i32, seed: u32) -> u32 {
    var h = (u32(x) * 0x9E3779B9u) ^ (u32(y) * 0x85EBCA6Bu) ^ (u32(z) * 0xC2B2AE35u) ^ (seed * 0x27D4EB2Fu);
    h ^= h >> 16u;
    h *= 0x7FEB352Du;
    h ^= h >> 15u;
    return h;
}

fn grad3(h: u32, x: f32, y: f32, z: f32) -> f32 {
    let idx = h & 15u;
    var u0 = x;
    var v0 = y;
    if idx == 1u { u0 = -x; v0 = y; }
    else if idx == 2u { u0 = x; v0 = -y; }
    else if idx == 3u { u0 = -x; v0 = -y; }
    else if idx == 4u { u0 = x; v0 = z; }
    else if idx == 5u { u0 = -x; v0 = z; }
    else if idx == 6u { u0 = x; v0 = -z; }
    else if idx == 7u { u0 = -x; v0 = -z; }
    else if idx == 8u { u0 = y; v0 = z; }
    else if idx == 9u { u0 = -y; v0 = z; }
    else if idx == 10u { u0 = y; v0 = -z; }
    else if idx == 11u { u0 = -y; v0 = -z; }
    else if idx == 12u { u0 = y; v0 = x; }
    else if idx == 13u { u0 = -y; v0 = z; }
    else if idx == 14u { u0 = y; v0 = -x; }
    else if idx == 15u { u0 = -y; v0 = -z; }
    let s1 = select(-u0, u0, (h & 1u) == 0u);
    let s2 = select(-v0, v0, (h & 2u) == 0u);
    return s1 + s2;
}

fn noise3_u32(x: f32, y: f32, z: f32, seed: u32) -> f32 {
    let xi = floor(x);
    let yi = floor(y);
    let zi = floor(z);
    let xf = x - xi;
    let yf = y - yi;
    let zf = z - zi;
    let ix = i32(xi);
    let iy = i32(yi);
    let iz = i32(zi);
    let fu = fade(xf);
    let fv = fade(yf);
    let fw = fade(zf);
    let x00 = mix(
        grad3(hash_u32(ix, iy, iz, seed), xf, yf, zf),
        grad3(hash_u32(ix + 1, iy, iz, seed), xf - 1.0, yf, zf),
        fu,
    );
    let x10 = mix(
        grad3(hash_u32(ix, iy + 1, iz, seed), xf, yf - 1.0, zf),
        grad3(hash_u32(ix + 1, iy + 1, iz, seed), xf - 1.0, yf - 1.0, zf),
        fu,
    );
    let x01 = mix(
        grad3(hash_u32(ix, iy, iz + 1, seed), xf, yf, zf - 1.0),
        grad3(hash_u32(ix + 1, iy, iz + 1, seed), xf - 1.0, yf, zf - 1.0),
        fu,
    );
    let x11 = mix(
        grad3(hash_u32(ix, iy + 1, iz + 1, seed), xf, yf - 1.0, zf - 1.0),
        grad3(hash_u32(ix + 1, iy + 1, iz + 1, seed), xf - 1.0, yf - 1.0, zf - 1.0),
        fu,
    );
    return mix(mix(x00, x10, fv), mix(x01, x11, fv), fw) * 0.9;
}

fn fbm3(x: f32, y: f32, z: f32) -> f32 {
    var sum = 0.0;
    var amp = 1.0;
    var freq = 1.0;
    var norm = 0.0;
    for (var i = 0u; i < 3u; i++) {
        sum += noise3_u32(x * freq, y * freq, z * freq + f32(i) * 17.13, 17u + i) * amp;
        norm += amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    return sum / norm;
}

@compute @workgroup_size(16, 16)
fn node_fx_px(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= u.i0.y || id.y >= u.i0.z {
        return;
    }
    let p = vec2<i32>(id.xy);
    let pc = vec2<f32>(p) + 0.5;
    let op = u.i0.x;
    var out = vec4(0.0);

    switch op {
        case OP_NODE_CHROMA_KEY_PRO: {
            let o = ld(p);
            if o.a <= 1e-6 {
                out = o;
            } else {
                var a = chroma_pro_matte(o);
                if a > 0.0 && a < 0.98 {
                    let hi = size() - vec2(1, 1);
                    let n0 = chroma_pro_matte(ld(clamp(p + vec2(-1, 0), vec2(0), hi)));
                    let n1 = chroma_pro_matte(ld(clamp(p + vec2(1, 0), vec2(0), hi)));
                    let n2 = chroma_pro_matte(ld(clamp(p + vec2(0, -1), vec2(0), hi)));
                    let n3 = chroma_pro_matte(ld(clamp(p + vec2(0, 1), vec2(0), hi)));
                    let nb_min = min(min(n0, n1), min(n2, n3));
                    if nb_min < 0.05 {
                        a = min(a, 0.5 * (a + nb_min));
                    }
                }
                out = finish_keyer(o, a);
            }
        }
        case OP_NODE_LUMA_KEY: {
            let o = ld(p);
            let a = luma_matte(o);
            out = finish_keyer(o, a);
        }
        case OP_NODE_KEYER_MATTE: {
            let o = ld(p);
            let a = select(luma_matte(o), chroma_pro_matte(o), u.i1.x == 0u);
            out = vec4(a, a, a, 1.0);
        }
        case OP_NODE_KEYER_CHOKE: {
            let o = ld_aux(p);
            let raw_a = select(luma_matte(o), chroma_pro_matte(o), u.i1.x == 0u);
            let bb = ld(p).r;
            let c = u.p3.y;
            let choked = clamp((bb - c) / (1.0 - c), 0.0, 1.0);
            let a = min(raw_a, choked);
            out = vec4(a, a, a, 1.0);
        }
        case OP_NODE_KEYER_FINISH: {
            let o = ld_aux(p);
            let a = clamp(ld(p).r, 0.0, 1.0);
            out = finish_keyer(o, a);
        }
        case OP_NODE_RADIAL_ZOOM: {
            let strength = u.p0.x;
            let c = u.p0.yz;
            let taps = u.i1.x;
            if abs(strength) < 1e-4 || taps < 2u {
                out = ld(p);
            } else {
                let d = c - pc;
                var acc = vec4(0.0);
                let denom = f32(taps - 1u);
                for (var i = 0u; i < taps; i++) {
                    let frac = f32(i) / denom * strength;
                    acc += sample_clamped(pc.x + d.x * frac, pc.y + d.y * frac);
                }
                out = acc / f32(taps);
            }
        }
        case OP_NODE_PIXELATE: {
            let s = vec2<f32>(size());
            let bw = max(s.x / f32(max(u.i1.x, 1u)), 1.0);
            let bh = max(s.y / f32(max(u.i1.y, 1u)), 1.0);
            let cxp = (floor((f32(p.x) + 1e-4) / bw) + 0.5) * bw;
            let cyp = (floor((f32(p.y) + 1e-4) / bh) + 0.5) * bh;
            out = sample_clamped(cxp, cyp);
        }
        case OP_NODE_CHROMATIC_ABERRATION: {
            let shift = u.p0.x;
            if abs(shift) < 0.1 {
                out = ld(p);
            } else {
                let r = sample_clamped(pc.x + shift, pc.y).r;
                let ga = ld(p);
                let b = sample_clamped(pc.x - shift, pc.y).b;
                out = vec4(r, ga.g, b, ga.a);
            }
        }
        case OP_NODE_RIPPLE_WAVE: {
            let off_y = u.p0.x * sin(pc.x * u.p0.y - u.p0.z);
            out = sample_bilinear(pc.x, pc.y + off_y);
        }
        case OP_NODE_LENS_DISTORT: {
            let k = u.p0.x;
            let c = u.p0.yz;
            let norm = u.p0.w;
            let inv_norm = 1.0 / max(norm, 1e-4);
            let d = (pc - c) * inv_norm;
            let r2 = dot(d, d);
            let s = 1.0 - k * r2;
            out = sample_bilinear(c.x + d.x * s * norm, c.y + d.y * s * norm);
        }
        case OP_NODE_TWIRL: {
            let ang = u.p0.x;
            let rad = max(u.p0.y, 1e-4);
            let c = u.p0.zw;
            let d = pc - c;
            let dist = length(d);
            if dist >= rad {
                out = sample_bilinear(pc.x, pc.y);
            } else {
                let k = 1.0 - dist / rad;
                let a = ang * k * k;
                let sn = sin(a);
                let cs = cos(a);
                out = sample_bilinear(c.x + d.x * cs - d.y * sn, c.y + d.x * sn + d.y * cs);
            }
        }
        case OP_NODE_FIND_EDGES: {
            let l_nw = luma709(ld_clamped(p + vec2(-1, -1)).rgb);
            let l_n  = luma709(ld_clamped(p + vec2(0, -1)).rgb);
            let l_ne = luma709(ld_clamped(p + vec2(1, -1)).rgb);
            let l_w  = luma709(ld_clamped(p + vec2(-1, 0)).rgb);
            let l_e  = luma709(ld_clamped(p + vec2(1, 0)).rgb);
            let l_sw = luma709(ld_clamped(p + vec2(-1, 1)).rgb);
            let l_s  = luma709(ld_clamped(p + vec2(0, 1)).rgb);
            let l_se = luma709(ld_clamped(p + vec2(1, 1)).rgb);
            let gx = l_ne + 2.0 * l_e + l_se - l_nw - 2.0 * l_w - l_sw;
            let gy = l_sw + 2.0 * l_s + l_se - l_nw - 2.0 * l_n - l_ne;
            var m = min(sqrt(gx * gx + gy * gy), 1.0);
            if u.i1.x == 0u {
                m = 1.0 - m;
            }
            let o = ld(p);
            let blend = clamp(u.p0.x, 0.0, 1.0);
            out = vec4(vec3(m * o.a) * (1.0 - blend) + o.rgb * blend, o.a);
        }
        case OP_NODE_NOISE_GRAIN: {
            let o = ld(p);
            let amt = u.p0.x;
            if o.a <= 1e-6 || amt < 1e-4 {
                out = o;
            } else {
                let grain = max(u.p0.y, 1.0);
                let color_mix = clamp(u.p0.z, 0.0, 1.0);
                let seed = u.i1.x;
                let gx = u32(f32(p.x) / grain);
                let gy = u32(f32(p.y) / grain);
                let n0 = hash3_u32(gx, gy, seed) - 0.5;
                let n1 = n0 + (hash3_u32(gx, gy, seed + 7777u) - 0.5 - n0) * color_mix;
                let n2 = n0 + (hash3_u32(gx, gy, seed + 99991u) - 0.5 - n0) * color_mix;
                let c = enc(o.rgb / o.a);
                let res = dec(clamp(c + vec3(n0, n1, n2) * amt, vec3(0.0), vec3(1.0)));
                out = vec4(res * o.a, o.a);
            }
        }
        case OP_NODE_TURBULENT_DISPLACE: {
            let amt = u.p0.x;
            if abs(amt) < 0.05 {
                out = ld(p);
            } else {
                let inv_size = 1.0 / max(u.p0.y, 1.0);
                let cu = pc.x * inv_size + u.p0.z;
                let cv = pc.y * inv_size + u.p0.w;
                let evo = u.p1.x;
                let nx = fbm3(cu, cv, evo);
                let ny = fbm3(cu + 31.7, cv + 12.3, evo + 3.1);
                out = sample_bilinear(pc.x + nx * amt, pc.y + ny * amt);
            }
        }
        case OP_NODE_RESAMPLE_AREA: {
            if u32(p.x) < u.i1.x || u32(p.x) >= u.i1.y || u32(p.y) < u.i1.z || u32(p.y) >= u.i1.w {
                out = vec4(0.0);
            } else {
                let s = size();
                let taps = clamp(i32(u.p1.w + 0.5), 2, 8);
                let inv_n = 1.0 / f32(taps);
                var acc = vec4(0.0);
                for (var iy: i32 = 0; iy < taps; iy++) {
                    let py = pc.y + (f32(iy) + 0.5) * inv_n - 0.5;
                    for (var ix: i32 = 0; ix < taps; ix++) {
                        let px = pc.x + (f32(ix) + 0.5) * inv_n - 0.5;
                        let uu = u.p0.x * px + u.p0.z * py + u.p1.x;
                        let vv = u.p0.y * px + u.p0.w * py + u.p1.y;
                        if uu >= -1.0 && vv >= -1.0 && uu <= f32(s.x) + 1.0 && vv <= f32(s.y) + 1.0 {
                            acc = acc + sample_bilinear(uu, vv);
                        }
                    }
                }
                out = (acc * (inv_n * inv_n)) * u.p1.z;
            }
        }
        default: {
            out = ld(p);
        }
    }

    textureStore(dst, p, out);
}
