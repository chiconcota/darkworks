//! Bộ thực thi đồ thị DAG Node Graph (`darkworks-node-graph-core`) trong `crates/render`.
//!
//! Đánh giá [`NodeGraphDoc`] tại `Tick` hiện tại, tính tín hiệu động (`TimeDriver`, `AudioReactor`)
//! và kết xuất tuần tự từng node sang [`Image`] premultiplied linear-light RGBA f32.

use std::collections::HashMap;
use std::sync::OnceLock;

use darkworks_node_graph_core::{
    AudioBand, DagEvaluator, DarkNode, EvaluationContext, NodeGraphDoc, WgslParser,
};
use filmcraft_geom::Affine;
use filmcraft_project::{Project, Sequence, TrackItem};
use filmcraft_time::Tick;

use crate::blend::composite;
use crate::gpufx::FxOp;
use crate::image::Image;
use crate::nodes::{
    apply_blur_sharpen, apply_color_grade, apply_distort, apply_keyer, apply_mask_alpha,
    apply_transform, apply_transition, map_blend, render_shape, render_text,
};
use crate::{RenderOptions, SourceProvider, output_size};

/// Callback thực thi Custom WGSL Shader trên GPU (được `crates/gpu` đăng ký lúc khởi động app).
pub type WgslRunnerFn = fn(code: &str, input: &Image, time_sec: f32, param_scalar: f32) -> Result<Image, String>;

static WGSL_RUNNER: OnceLock<WgslRunnerFn> = OnceLock::new();

/// Đăng ký hàm chạy CustomWgsl shader trên GPU.
pub fn register_wgsl_runner(runner: WgslRunnerFn) {
    let _ = WGSL_RUNNER.set(runner);
}

/// Đánh giá đồ thị Node Graph gắn trên `item` tại thời điểm sequence `t`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn eval_item_graph(
    project: &Project,
    seq: &Sequence,
    item: &TrackItem,
    t: Tick,
    opts: RenderOptions,
    sources: &dyn SourceProvider,
    clip_image: Option<&Image>,
    below_image: Option<&Image>,
) -> Option<Image> {
    let doc = item.node_graph.as_deref()?;
    if doc.is_empty() {
        return None;
    }
    let plan = DagEvaluator::build_execution_plan(doc).ok()?;
    let (w, h) = output_size(seq, opts.scale);
    let time_sec = (t - item.start).seconds().max(0.0);

    let audio_rms = compute_audio_rms(project, seq, t, &plan, sources);
    let sig_ctx = DagEvaluator::evaluate_signals(&plan, time_sec, audio_rms);

    let sx = w as f32 / seq.settings.width.max(1) as f32;
    let sy = h as f32 / seq.settings.height.max(1) as f32;
    let scale_xy = (sx, sy);

    let mut textures: HashMap<usize, Image> = HashMap::new();
    let mut final_output: Option<Image> = None;

    for step in &plan.steps {
        if filmcraft_media::cancel::cancelled() {
            return None;
        }
        let id = step.node_id;
        let out_img: Option<Image> = match &step.node {
            DarkNode::VideoInput { track_name } => Some(resolve_video_input(
                track_name,
                clip_image,
                below_image,
                w,
                h,
            )),
            DarkNode::ShapeGenerator {
                shape_type,
                size,
                corner_radius,
                color,
            } => Some(render_shape(
                *shape_type,
                *size,
                *corner_radius,
                *color,
                w,
                h,
                scale_xy,
            )),
            DarkNode::TextGenerator {
                text,
                font_size,
                color,
                tracking,
                line_spacing,
            } => Some(render_text(
                text,
                *font_size,
                *color,
                *tracking,
                *line_spacing,
                w,
                h,
                opts.scale,
            )),
            DarkNode::ColorGrade {
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
            } => {
                let mut img = input_texture(doc, &textures, id, 0, w, h);
                apply_color_grade(
                    &mut img,
                    *exposure,
                    *contrast,
                    *saturation,
                    *temperature,
                    *tint,
                    *gamma,
                    *invert,
                    *black_and_white,
                    *tint_color,
                    *tint_amount,
                );
                Some(img)
            }
            DarkNode::BlurSharpen {
                blur_type,
                radius,
                angle_deg,
                highlight_boost,
            } => {
                let mut img = input_texture(doc, &textures, id, 0, w, h);
                apply_blur_sharpen(&mut img, *blur_type, *radius, *angle_deg, *highlight_boost, opts.scale);
                Some(img)
            }
            DarkNode::DistortLens {
                distort_type,
                intensity,
                radius,
                center,
            } => {
                let mut img = input_texture(doc, &textures, id, 0, w, h);
                let eff_intensity = modulated_scalar(doc, &sig_ctx, id, 1, *intensity);
                let eff_center = modulated_vec2(doc, &sig_ctx, id, 2, *center);
                apply_distort(
                    &mut img,
                    *distort_type,
                    eff_intensity,
                    *radius,
                    eff_center,
                    time_sec,
                    opts.scale,
                );
                Some(img)
            }
            DarkNode::Keyer {
                key_color,
                similarity,
                smoothness,
                spill_suppress,
            } => {
                let mut img = input_texture(doc, &textures, id, 0, w, h);
                apply_keyer(&mut img, *key_color, *similarity, *smoothness, *spill_suppress);
                Some(img)
            }
            DarkNode::Transform2D {
                position,
                scale,
                rotation_deg,
                anchor,
                opacity,
                flip_h,
                flip_v,
            } => {
                let mut img = input_texture(doc, &textures, id, 0, w, h);
                let pos = modulated_vec2_add(doc, &sig_ctx, id, 1, *position);
                let s_mod = modulated_scalar_mul(doc, &sig_ctx, id, 2, 1.0);
                let rot_mod = modulated_scalar_add(doc, &sig_ctx, id, 3, *rotation_deg);
                let op_mod = modulated_scalar_mul(doc, &sig_ctx, id, 4, *opacity).clamp(0.0, 1.0);
                Some(apply_transform(
                    &mut img,
                    pos,
                    [scale[0] * s_mod, scale[1] * s_mod],
                    rot_mod,
                    *anchor,
                    op_mod,
                    *flip_h,
                    *flip_v,
                    scale_xy,
                ))
            }
            DarkNode::CropMask {
                top,
                bottom,
                left,
                right,
                feather,
            } => {
                let mut img = input_texture(doc, &textures, id, 0, w, h);
                let x0 = (left.clamp(0.0, 1.0) * w as f32).min(w as f32);
                let x1 = ((1.0 - right.clamp(0.0, 1.0)) * w as f32).max(x0);
                let y0 = (top.clamp(0.0, 1.0) * h as f32).min(h as f32);
                let y1 = ((1.0 - bottom.clamp(0.0, 1.0)) * h as f32).max(y0);
                FxOp::Crop {
                    x0,
                    x1,
                    y0,
                    y1,
                    feather: (*feather * opts.scale).max(0.0),
                }
                .apply(&mut img);
                Some(img)
            }
            DarkNode::Blend { mode, opacity } => {
                let mut bg = input_texture(doc, &textures, id, 0, w, h);
                let mut fg = input_texture(doc, &textures, id, 1, w, h);
                if let Some((src_id, _)) = doc.source_for_pin(id, 2)
                    && let Some(mask_img) = textures.get(&src_id)
                {
                    apply_mask_alpha(&mut fg, mask_img);
                }
                composite(&mut bg, &fg, opacity.clamp(0.0, 1.0), map_blend(*mode));
                Some(bg)
            }
            DarkNode::Transition {
                transition_type,
                progress,
                feather,
            } => {
                let a = input_texture(doc, &textures, id, 0, w, h);
                let b = input_texture(doc, &textures, id, 1, w, h);
                let prog = modulated_scalar_override(doc, &sig_ctx, id, 2, *progress).clamp(0.0, 1.0);
                Some(apply_transition(&a, &b, *transition_type, prog, *feather))
            }
            DarkNode::Reroute { .. } => Some(input_texture(doc, &textures, id, 0, w, h)),
            DarkNode::CustomWgsl { code, .. } => {
                let in_img = input_texture(doc, &textures, id, 0, w, h);
                let param = modulated_scalar_override(doc, &sig_ctx, id, 1, 1.0);
                if WgslParser::parse_uniforms(code).is_ok()
                    && let Some(runner) = WGSL_RUNNER.get()
                    && let Ok(rendered) = runner(code, &in_img, time_sec as f32, param)
                {
                    Some(rendered)
                } else {
                    Some(in_img)
                }
            }
            DarkNode::Output => {
                let img = input_texture(doc, &textures, id, 0, w, h);
                final_output = Some(img.clone());
                Some(img)
            }
            DarkNode::TimeDriver { .. }
            | DarkNode::AudioReactor { .. }
            | DarkNode::ActionNode { .. }
            | DarkNode::BlackNode { .. }
            | DarkNode::StickyNote { .. } => None,
        };

        if let Some(img) = out_img {
            textures.insert(id, img);
        }
    }

    final_output.or_else(|| {
        plan.steps
            .iter()
            .rev()
            .find_map(|s| textures.remove(&s.node_id))
    })
}

fn compute_audio_rms(
    project: &Project,
    seq: &Sequence,
    t: Tick,
    plan: &darkworks_node_graph_core::DagDocPlan,
    sources: &dyn SourceProvider,
) -> f32 {
    let audio_node = plan.steps.iter().find_map(|s| match &s.node {
        DarkNode::AudioReactor { band, smooth, .. } => Some((*band, *smooth)),
        _ => None,
    });
    let Some((band, _smooth)) = audio_node else {
        return 0.0;
    };
    let sr = seq.settings.sample_rate.max(8000);
    let win_frames = 1024usize;
    let start = ((t.seconds() * sr as f64).round() as i64).saturating_sub((win_frames / 2) as i64);
    let buf = crate::audio::mix_sequence(project, seq, start, win_frames, sources);
    if buf.channels.is_empty() || buf.channels[0].is_empty() {
        return 0.0;
    }

    let ch0 = &buf.channels[0];
    let n = ch0.len();
    let mut sum_sq = 0.0f64;
    let mut lp = 0.0f32;
    let alpha = match band {
        AudioBand::SubBass => 0.04,
        AudioBand::Bass => 0.09,
        AudioBand::LowMid => 0.22,
        AudioBand::HighMid => 0.45,
        AudioBand::Treble | AudioBand::MasterPeak => 1.0,
    };

    for i in 0..n {
        let mut s = ch0[i];
        if buf.channels.len() > 1 {
            s = 0.5 * (s + buf.channels[1][i]);
        }
        let val = match band {
            AudioBand::MasterPeak => s,
            AudioBand::Treble => {
                let prev = if i > 0 { ch0[i - 1] } else { s };
                s - prev
            }
            _ => {
                lp += alpha * (s - lp);
                lp
            }
        };
        sum_sq += (val as f64) * (val as f64);
    }
    ((sum_sq / n as f64).sqrt() as f32 * 2.2).clamp(0.0, 1.0)
}

fn resolve_video_input(
    _track_name: &str,
    clip_image: Option<&Image>,
    below_image: Option<&Image>,
    w: usize,
    h: usize,
) -> Image {
    if let Some(img) = clip_image.or(below_image) {
        if img.w == w && img.h == h {
            img.clone()
        } else {
            img.transformed(
                w,
                h,
                &Affine::scale(
                    w as f64 / img.w.max(1) as f64,
                    h as f64 / img.h.max(1) as f64,
                ),
            )
        }
    } else {
        Image::new(w, h)
    }
}

fn input_texture(
    doc: &NodeGraphDoc,
    textures: &HashMap<usize, Image>,
    node_id: usize,
    pin: usize,
    w: usize,
    h: usize,
) -> Image {
    doc.source_for_pin(node_id, pin)
        .and_then(|(src_id, _)| textures.get(&src_id))
        .cloned()
        .unwrap_or_else(|| Image::new(w, h))
}

fn modulated_scalar(doc: &NodeGraphDoc, ctx: &EvaluationContext, node_id: usize, pin: usize, base: f32) -> f32 {
    if let Some((src_id, _)) = doc.source_for_pin(node_id, pin)
        && let Some(&v) = ctx.scalar_outputs.get(&src_id)
    {
        return base * v;
    }
    base
}

fn modulated_scalar_mul(doc: &NodeGraphDoc, ctx: &EvaluationContext, node_id: usize, pin: usize, base: f32) -> f32 {
    if let Some((src_id, _)) = doc.source_for_pin(node_id, pin)
        && let Some(&v) = ctx.scalar_outputs.get(&src_id)
    {
        return base * v.max(0.01);
    }
    base
}

fn modulated_scalar_add(doc: &NodeGraphDoc, ctx: &EvaluationContext, node_id: usize, pin: usize, base: f32) -> f32 {
    if let Some((src_id, _)) = doc.source_for_pin(node_id, pin)
        && let Some(&v) = ctx.scalar_outputs.get(&src_id)
    {
        return base + v * 36.0;
    }
    base
}

fn modulated_scalar_override(doc: &NodeGraphDoc, ctx: &EvaluationContext, node_id: usize, pin: usize, base: f32) -> f32 {
    if let Some((src_id, _)) = doc.source_for_pin(node_id, pin)
        && let Some(&v) = ctx.scalar_outputs.get(&src_id)
    {
        return v;
    }
    base
}

fn modulated_vec2(doc: &NodeGraphDoc, ctx: &EvaluationContext, node_id: usize, pin: usize, base: [f32; 2]) -> [f32; 2] {
    if let Some((src_id, _)) = doc.source_for_pin(node_id, pin)
        && let Some(&v) = ctx.vec2_outputs.get(&src_id)
    {
        return [base[0] + v[0], base[1] + v[1]];
    }
    base
}

fn modulated_vec2_add(doc: &NodeGraphDoc, ctx: &EvaluationContext, node_id: usize, pin: usize, base: [f32; 2]) -> [f32; 2] {
    if let Some((src_id, _)) = doc.source_for_pin(node_id, pin)
        && let Some(&v) = ctx.vec2_outputs.get(&src_id)
    {
        return [base[0] + v[0], base[1] + v[1]];
    }
    if let Some((src_id, _)) = doc.source_for_pin(node_id, pin)
        && let Some(&s) = ctx.scalar_outputs.get(&src_id)
    {
        return [base[0] + s * 60.0, base[1]];
    }
    base
}

#[cfg(test)]
mod tests {
    use super::*;
    use darkworks_node_graph_core::{BlendMode, NodeEntry, NodeWire, ShapeType};
    use filmcraft_project::SequenceSettings;

    #[test]
    fn eval_node_graph_shape_and_color_invert_produces_expected_pixels() {
        let mut p = Project::new("test");
        let seq_id = p.new_sequence("s", SequenceSettings { width: 64, height: 64, ..Default::default() }, 1, 0, None);
        let seq = p.sequence(seq_id).expect("sequence").clone();

        let doc = NodeGraphDoc {
            nodes: vec![
                NodeEntry {
                    id: 0,
                    pos: [0.0, 0.0],
                    node: DarkNode::ShapeGenerator {
                        shape_type: ShapeType::SolidMatte,
                        size: [64.0, 64.0],
                        corner_radius: 0.0,
                        color: [0.0, 0.0, 0.0, 1.0],
                    },
                    custom_title: None,
                },
                NodeEntry {
                    id: 1,
                    pos: [100.0, 0.0],
                    node: DarkNode::ColorGrade {
                        exposure: 0.0,
                        contrast: 1.0,
                        saturation: 1.0,
                        temperature: 0.0,
                        tint: 0.0,
                        gamma: 1.0,
                        invert: true,
                        black_and_white: false,
                        tint_color: [1.0, 1.0, 1.0, 1.0],
                        tint_amount: 0.0,
                    },
                    custom_title: None,
                },
                NodeEntry {
                    id: 2,
                    pos: [200.0, 0.0],
                    node: DarkNode::Blend { mode: BlendMode::Normal, opacity: 1.0 },
                    custom_title: None,
                },
                NodeEntry {
                    id: 3,
                    pos: [300.0, 0.0],
                    node: DarkNode::Output,
                    custom_title: None,
                },
            ],
            wires: vec![
                NodeWire { out_node: 0, out_pin: 0, in_node: 1, in_pin: 0 },
                NodeWire { out_node: 0, out_pin: 0, in_node: 2, in_pin: 0 },
                NodeWire { out_node: 1, out_pin: 0, in_node: 2, in_pin: 1 },
                NodeWire { out_node: 2, out_pin: 0, in_node: 3, in_pin: 0 },
            ],
            groups: Vec::new(),
        };

        let mut item: TrackItem = serde_json::from_value(serde_json::json!({
            "id": 1, "item": 2, "name": "ng", "label": "Iris", "start": 0, "duration": 1000, "source_in": 0,
            "speed": 1.0, "enabled": true, "effects": []
        }))
        .expect("item");
        item.node_graph = Some(Box::new(doc));

        let out = eval_item_graph(&p, &seq, &item, Tick::ZERO, RenderOptions::default(), &|_| None, None, None)
            .expect("evaluated image");
        assert_eq!((out.w, out.h), (64, 64));
        let center = out.get(32, 32);
        // Đen đảo màu thành trắng (linear ~1.0)
        assert!(center[0] > 0.99 && center[1] > 0.99 && center[2] > 0.99 && center[3] > 0.99);
    }
}
