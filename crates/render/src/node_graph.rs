//! Bộ thực thi đồ thị DAG Node Graph (`darkworks-node-graph-core`) trong `crates/render`.
//!
//! Đánh giá [`NodeGraphDoc`] tại `Tick` hiện tại, tính tín hiệu động (`TimeDriver`, `AudioReactor`)
//! và kết xuất tuần tự từng node sang [`Image`] premultiplied linear-light RGBA f32.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use darkworks_node_graph_core::{DagEvaluator, DarkNode, NodeGraphDoc, WgslParser};
use filmcraft_project::{ClipId, Project, Sequence, TrackItem};
use filmcraft_time::Tick;

use crate::blend::composite;
use crate::gpufx::FxOp;
use crate::image::Image;
use crate::nodes::color::ColorGradeParams;
use crate::nodes::{
    apply_blur_sharpen, apply_color_grade, apply_distort, apply_keyer, apply_mask_alpha,
    apply_transform, apply_transition, compute_audio_rms_map, input_texture, map_blend,
    modulated_scalar, modulated_scalar_add, modulated_scalar_add_raw, modulated_scalar_mul,
    modulated_scalar_override, modulated_vec2, modulated_vec2_add, render_shape, render_text,
    resolve_video_input,
};
use crate::{RenderOptions, SourceProvider, output_size};

/// Callback thực thi Custom WGSL Shader trên GPU (được `crates/gpu` đăng ký lúc khởi động app).
pub type WgslRunnerFn = fn(code: &str, input: &Image, time_sec: f32, param_scalar: f32) -> Result<Image, String>;

static WGSL_RUNNER: OnceLock<WgslRunnerFn> = OnceLock::new();

/// Đăng ký hàm chạy CustomWgsl shader trên GPU.
pub fn register_wgsl_runner(runner: WgslRunnerFn) {
    let _ = WGSL_RUNNER.set(runner);
}

/// Kiểm tra đồ thị có chứa node `VideoInput` đọc luồng hình nền/track hay không.
pub(crate) fn graph_has_video_input(doc: &NodeGraphDoc) -> bool {
    doc.nodes
        .iter()
        .any(|n| matches!(n.node, DarkNode::VideoInput { .. }))
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
    let mut visited = HashSet::new();
    eval_item_graph_inner(
        project,
        seq,
        item,
        t,
        opts,
        sources,
        clip_image,
        below_image,
        0,
        &mut visited,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn eval_item_graph_inner(
    project: &Project,
    seq: &Sequence,
    item: &TrackItem,
    t: Tick,
    opts: RenderOptions,
    sources: &dyn SourceProvider,
    clip_image: Option<&Image>,
    below_image: Option<&Image>,
    depth: u8,
    visited: &mut HashSet<ClipId>,
) -> Option<Image> {
    let doc = item.node_graph.as_deref()?;
    if doc.is_empty() {
        return None;
    }
    let plan = DagEvaluator::build_execution_plan(doc).ok()?;
    visited.insert(item.id);

    let (w, h) = output_size(seq, opts.scale);
    let time_sec = (t - item.start).seconds().max(0.0);

    let audio_map = compute_audio_rms_map(project, seq, item, t, &plan, sources);
    let fallback_rms = audio_map.values().copied().next().unwrap_or(0.0);
    let sig_ctx = DagEvaluator::evaluate_signals_with_audio(&plan, time_sec, fallback_rms, &audio_map);

    let sx = w as f32 / seq.settings.width.max(1) as f32;
    let sy = h as f32 / seq.settings.height.max(1) as f32;
    let scale_xy = (sx, sy);

    let mut textures: HashMap<usize, Image> = HashMap::new();
    let mut final_output: Option<Image> = None;

    for step in &plan.steps {
        if filmcraft_media::cancel::cancelled() {
            visited.remove(&item.id);
            return None;
        }
        let id = step.node_id;
        let out_img: Option<Image> = match &step.node {
            DarkNode::VideoInput { track_name } => Some(resolve_video_input(
                track_name,
                project,
                seq,
                item,
                t,
                opts,
                sources,
                clip_image,
                below_image,
                w,
                h,
                depth,
                visited,
            )),
            DarkNode::ShapeGenerator {
                shape_type,
                size,
                corner_radius,
                color,
                position,
                stroke_width,
                stroke_color,
                feather,
            } => {
                let eff_pos = modulated_vec2_add(doc, &sig_ctx, id, 0, *position);
                Some(render_shape(
                    *shape_type,
                    *size,
                    *corner_radius,
                    *color,
                    eff_pos,
                    *stroke_width,
                    *stroke_color,
                    *feather,
                    w,
                    h,
                    scale_xy,
                ))
            }
            DarkNode::TextGenerator {
                text,
                font_size,
                color,
                tracking,
                line_spacing,
                position,
                align,
                font_family,
                bold,
                italic,
                stroke_color,
                stroke_width,
            } => {
                let eff_pos = modulated_vec2_add(doc, &sig_ctx, id, 0, *position);
                Some(render_text(
                    text,
                    *font_size,
                    *color,
                    *tracking,
                    *line_spacing,
                    eff_pos,
                    *align,
                    font_family,
                    *bold,
                    *italic,
                    *stroke_color,
                    *stroke_width,
                    w,
                    h,
                    opts.scale,
                ))
            }
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
                highlights,
                shadows,
                whites,
                blacks,
                vibrance,
                faded_film,
                shadow_tint,
                highlight_tint,
            } => {
                let mut img = input_texture(doc, &textures, id, 0, w, h);
                let eff_exposure = modulated_scalar_add_raw(doc, &sig_ctx, id, 1, *exposure);
                apply_color_grade(
                    &mut img,
                    &ColorGradeParams {
                        exposure: eff_exposure,
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
                    },
                );
                Some(img)
            }
            DarkNode::BlurSharpen {
                blur_type,
                radius,
                angle_deg,
                highlight_boost,
                threshold,
            } => {
                let mut img = input_texture(doc, &textures, id, 0, w, h);
                let eff_radius = modulated_scalar_mul(doc, &sig_ctx, id, 1, *radius);
                apply_blur_sharpen(
                    &mut img,
                    *blur_type,
                    eff_radius,
                    *angle_deg,
                    *highlight_boost,
                    *threshold,
                    opts.scale,
                );
                Some(img)
            }
            DarkNode::DistortLens {
                distort_type,
                intensity,
                radius,
                center,
                feather,
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
                    *feather,
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
                direction,
            } => {
                let a = input_texture(doc, &textures, id, 0, w, h);
                let b = input_texture(doc, &textures, id, 1, w, h);
                let prog = modulated_scalar_override(doc, &sig_ctx, id, 2, *progress).clamp(0.0, 1.0);
                Some(apply_transition(&a, &b, *transition_type, prog, *feather, *direction))
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

    visited.remove(&item.id);
    final_output.or_else(|| {
        plan.steps
            .iter()
            .rev()
            .find_map(|s| textures.remove(&s.node_id))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use darkworks_node_graph_core::{BlendMode, NodeEntry, NodeWire, ShapeType};
    use filmcraft_project::{ItemKind, Label, SequenceSettings, TrackKind};
    use filmcraft_time::{FrameRate, TimeRange};

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
                    node: DarkNode::default_shape(
                        ShapeType::SolidMatte,
                        [64.0, 64.0],
                        0.0,
                        [0.0, 0.0, 0.0, 1.0],
                    ),
                    custom_title: None,
                },
                NodeEntry {
                    id: 1,
                    pos: [100.0, 0.0],
                    node: match DarkNode::default_color_grade() {
                        DarkNode::ColorGrade {
                            exposure, contrast, saturation, temperature, tint, gamma,
                            black_and_white, tint_color, tint_amount, highlights, shadows,
                            whites, blacks, vibrance, faded_film, shadow_tint, highlight_tint, ..
                        } => DarkNode::ColorGrade {
                            exposure, contrast, saturation, temperature, tint, gamma, invert: true,
                            black_and_white, tint_color, tint_amount, highlights, shadows,
                            whites, blacks, vibrance, faded_film, shadow_tint, highlight_tint,
                        },
                        other => other,
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
        assert!(center[0] > 0.99 && center[1] > 0.99 && center[2] > 0.99 && center[3] > 0.99);
    }

    #[test]
    fn eval_node_graph_lumetri_tonal_and_feather_shape_work() {
        let mut p = Project::new("test");
        let seq_id = p.new_sequence("s", SequenceSettings { width: 64, height: 64, ..Default::default() }, 1, 0, None);
        let seq = p.sequence(seq_id).expect("sequence").clone();

        let cg = match DarkNode::default_color_grade() {
            DarkNode::ColorGrade {
                exposure, contrast, saturation, temperature, tint, gamma, invert,
                black_and_white, tint_color, tint_amount, ..
            } => DarkNode::ColorGrade {
                exposure, contrast, saturation, temperature, tint, gamma, invert,
                black_and_white, tint_color, tint_amount,
                highlights: -25.0,
                shadows: 30.0,
                whites: 0.0,
                blacks: -10.0,
                vibrance: 20.0,
                faded_film: 35.0,
                shadow_tint: [0.36, 0.52, 0.6],
                highlight_tint: [0.64, 0.53, 0.42],
            },
            other => other,
        };

        let doc = NodeGraphDoc {
            nodes: vec![
                NodeEntry {
                    id: 0,
                    pos: [0.0, 0.0],
                    node: DarkNode::ShapeGenerator {
                        shape_type: ShapeType::Circle,
                        size: [32.0, 32.0],
                        corner_radius: 0.0,
                        color: [0.2, 0.4, 0.8, 1.0],
                        position: [0.0, 0.0],
                        stroke_width: 2.0,
                        stroke_color: [1.0, 1.0, 1.0, 1.0],
                        feather: 4.0,
                    },
                    custom_title: None,
                },
                NodeEntry { id: 1, pos: [200.0, 0.0], node: cg, custom_title: None },
                NodeEntry { id: 2, pos: [400.0, 0.0], node: DarkNode::Output, custom_title: None },
            ],
            wires: vec![
                NodeWire { out_node: 0, out_pin: 0, in_node: 1, in_pin: 0 },
                NodeWire { out_node: 1, out_pin: 0, in_node: 2, in_pin: 0 },
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
            .expect("evaluated lumetri + shape");
        let center = out.get(32, 32);
        let edge = out.get(16, 32);
        let corner = out.get(0, 0);
        assert!(center[2] > center[0] && center[3] > 0.9);
        assert!(edge[3] > 0.01 && edge[3] < 0.99, "edge alpha should be feathered: {}", edge[3]);
        assert!(corner[3] < 1e-5);

        if let Some(bg_node) = item.node_graph.as_deref_mut().and_then(|d| d.nodes.get_mut(0)) {
            bg_node.node = DarkNode::default_shape(ShapeType::SolidMatte, [64.0, 64.0], 0.0, [0.0, 0.0, 0.0, 1.0]);
        }
        let graded_matte = eval_item_graph(&p, &seq, &item, Tick::ZERO, RenderOptions::default(), &|_| None, None, None)
            .expect("evaluated lumetri matte");
        assert!(graded_matte.get(0, 0)[0] > 0.005);
    }

    #[test]
    fn video_input_reads_v2_and_overlay_fx_clip_keeps_v1_visible() {
        let mut p = Project::new("test2");
        let rate = FrameRate::FPS_24;
        let dur = Tick::from_seconds_f64(2.0);
        let seq_id = p.new_sequence("s", SequenceSettings { width: 32, height: 32, ..Default::default() }, 2, 0, None);

        // V1: solid red clip
        let src_v1 = p.add_item("v1", Label::Iris, ItemKind::AdjustmentLayer { width: 32, height: 32, rate, duration: dur }, None);
        let mut ti_v1 = p.make_track_item(src_v1, TrackKind::Video, Tick::ZERO, TimeRange::new(Tick::ZERO, dur), rate).expect("ti1");
        ti_v1.node_graph = Some(Box::new(NodeGraphDoc {
            nodes: vec![
                NodeEntry { id: 0, pos: [0.0, 0.0], node: DarkNode::default_shape(ShapeType::SolidMatte, [32.0, 32.0], 0.0, [1.0, 0.0, 0.0, 1.0]), custom_title: None },
                NodeEntry { id: 1, pos: [200.0, 0.0], node: DarkNode::Output, custom_title: None },
            ],
            wires: vec![NodeWire { out_node: 0, out_pin: 0, in_node: 1, in_pin: 0 }],
            groups: Vec::new(),
        }));

        // V2: small green circle overlay (no VideoInput -> should NOT erase V1!)
        let src_v2 = p.add_item("v2", Label::Iris, ItemKind::AdjustmentLayer { width: 32, height: 32, rate, duration: dur }, None);
        let mut ti_v2 = p.make_track_item(src_v2, TrackKind::Video, Tick::ZERO, TimeRange::new(Tick::ZERO, dur), rate).expect("ti2");
        ti_v2.node_graph = Some(Box::new(NodeGraphDoc {
            nodes: vec![
                NodeEntry { id: 0, pos: [0.0, 0.0], node: DarkNode::default_shape(ShapeType::Circle, [12.0, 12.0], 0.0, [0.0, 1.0, 0.0, 1.0]), custom_title: None },
                NodeEntry { id: 1, pos: [200.0, 0.0], node: DarkNode::Output, custom_title: None },
            ],
            wires: vec![NodeWire { out_node: 0, out_pin: 0, in_node: 1, in_pin: 0 }],
            groups: Vec::new(),
        }));

        {
            let q = p.sequence_mut(seq_id).expect("seq");
            q.video_tracks[0].items.push(ti_v1);
            q.video_tracks[1].items.push(ti_v2);
        }

        let seq = p.sequence(seq_id).expect("seq");
        let rendered = crate::render_seq(&p, seq, Tick::ZERO, RenderOptions::default(), &|_| None);
        // Corner (0,0) keeps V1 red, center (16,16) shows V2 green circle!
        assert!(rendered.get(0, 0)[0] > 0.9 && rendered.get(0, 0)[1] < 0.05);
        assert!(rendered.get(16, 16)[1] > 0.9 && rendered.get(16, 16)[0] < 0.05);

        // Also test VideoInput("V2") reading V2 directly from a graph!
        let mut probe_item: TrackItem = serde_json::from_value(serde_json::json!({
            "id": 999, "item": 2, "name": "probe", "label": "Iris", "start": 0, "duration": 1000, "source_in": 0,
            "speed": 1.0, "enabled": true, "effects": []
        })).expect("probe");
        probe_item.node_graph = Some(Box::new(NodeGraphDoc {
            nodes: vec![
                NodeEntry { id: 0, pos: [0.0, 0.0], node: DarkNode::VideoInput { track_name: "V2".to_string() }, custom_title: None },
                NodeEntry { id: 1, pos: [200.0, 0.0], node: DarkNode::Output, custom_title: None },
            ],
            wires: vec![NodeWire { out_node: 0, out_pin: 0, in_node: 1, in_pin: 0 }],
            groups: Vec::new(),
        }));
        let v2_img = eval_item_graph(&p, seq, &probe_item, Tick::ZERO, RenderOptions::default(), &|_| None, None, None)
            .expect("v2 sampled");
        assert!(v2_img.get(16, 16)[1] > 0.9);
        assert!(v2_img.get(0, 0)[3] < 1e-5);
    }
}
