//! Bộ thực thi đồ thị DAG Node Graph (`darkworks-node-graph-core`) trong `crates/render`.
//!
//! Đánh giá [`NodeGraphDoc`] tại `Tick` hiện tại, tính tín hiệu động (`TimeDriver`, `AudioReactor`)
//! và kết xuất tuần tự từng node sang [`Image`] premultiplied linear-light RGBA f32.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use darkworks_node_graph_core::{DagEvaluator, DarkNode, NodeGraphDoc, WgslParser};
use filmcraft_frame::pool::recycle_f32;
use filmcraft_project::{ClipId, Project, Sequence, TrackItem};
use filmcraft_time::Tick;

use crate::blend::composite;
use crate::gpufx::FxOp;
use crate::image::Image;
use crate::nodes::color::{ColorGradeParams, KeyerProParams};
use crate::nodes::{
    apply_blur_sharpen, apply_color_grade, apply_distort, apply_keyer, apply_mask_alpha,
    apply_transform, apply_transition, compute_audio_rms_map, count_wire_uses, map_blend,
    modulated_scalar, modulated_scalar_add, modulated_scalar_add_scaled, modulated_scalar_mul,
    modulated_scalar_override, modulated_vec2, modulated_vec2_add, render_shape, render_text,
    resolve_video_input, take_or_clone_texture,
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

    let mut textures: HashMap<(usize, usize), Image> = HashMap::new();
    let mut use_counts = count_wire_uses(doc);
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
                let mut img = take_or_clone_texture(doc, &mut textures, &mut use_counts, id, 0, w, h);
                let eff_exposure = modulated_scalar_add_scaled(doc, &sig_ctx, id, 1, *exposure, 2.5);
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
                dimensions,
                repeat_edge,
                bokeh_sides,
                center,
            } => {
                let mut img = take_or_clone_texture(doc, &mut textures, &mut use_counts, id, 0, w, h);
                let eff_radius = modulated_scalar_mul(doc, &sig_ctx, id, 1, *radius);
                apply_blur_sharpen(
                    &mut img,
                    *blur_type,
                    eff_radius,
                    *angle_deg,
                    *highlight_boost,
                    *threshold,
                    *dimensions,
                    *repeat_edge,
                    *bokeh_sides,
                    *center,
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
                let mut img = take_or_clone_texture(doc, &mut textures, &mut use_counts, id, 0, w, h);
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
            } => {
                let mut img = take_or_clone_texture(doc, &mut textures, &mut use_counts, id, 0, w, h);
                let gm = doc
                    .source_for_pin(id, 1)
                    .map(|_| take_or_clone_texture(doc, &mut textures, &mut use_counts, id, 1, w, h));
                let need_matte_out = use_counts.get(&(id, 1)).copied().unwrap_or(0) > 0;
                let matte_out = apply_keyer(
                    &mut img,
                    &KeyerProParams {
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
                        px_scale: opts.scale,
                    },
                    gm.as_ref(),
                    need_matte_out,
                );
                if let Some(gm_img) = gm {
                    recycle_f32(gm_img.px);
                }
                if let Some(m) = matte_out {
                    textures.insert((id, 1), m);
                }
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
                let mut img = take_or_clone_texture(doc, &mut textures, &mut use_counts, id, 0, w, h);
                let pos = modulated_vec2_add(doc, &sig_ctx, id, 1, *position);
                let s_mod = modulated_scalar_mul(doc, &sig_ctx, id, 2, 1.0);
                let rot_mod = modulated_scalar_add(doc, &sig_ctx, id, 3, *rotation_deg);
                let op_mod = modulated_scalar_mul(doc, &sig_ctx, id, 4, *opacity).clamp(0.0, 1.0);
                let res = apply_transform(
                    &mut img,
                    pos,
                    [scale[0] * s_mod, scale[1] * s_mod],
                    rot_mod,
                    *anchor,
                    op_mod,
                    *flip_h,
                    *flip_v,
                    scale_xy,
                );
                recycle_f32(img.px);
                Some(res)
            }
            DarkNode::CropMask {
                top,
                bottom,
                left,
                right,
                feather,
            } => {
                let mut img = take_or_clone_texture(doc, &mut textures, &mut use_counts, id, 0, w, h);
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
                let mut bg = take_or_clone_texture(doc, &mut textures, &mut use_counts, id, 0, w, h);
                let mut fg = take_or_clone_texture(doc, &mut textures, &mut use_counts, id, 1, w, h);
                if doc.source_for_pin(id, 2).is_some() {
                    let mask = take_or_clone_texture(doc, &mut textures, &mut use_counts, id, 2, w, h);
                    apply_mask_alpha(&mut fg, &mask);
                    recycle_f32(mask.px);
                }
                composite(&mut bg, &fg, opacity.clamp(0.0, 1.0), map_blend(*mode));
                recycle_f32(fg.px);
                Some(bg)
            }
            DarkNode::Transition {
                transition_type,
                progress,
                feather,
                direction,
            } => {
                let a = take_or_clone_texture(doc, &mut textures, &mut use_counts, id, 0, w, h);
                let b = take_or_clone_texture(doc, &mut textures, &mut use_counts, id, 1, w, h);
                let prog = modulated_scalar_override(doc, &sig_ctx, id, 2, *progress).clamp(0.0, 1.0);
                let res = apply_transition(&a, &b, *transition_type, prog, *feather, *direction);
                recycle_f32(a.px);
                recycle_f32(b.px);
                Some(res)
            }
            DarkNode::Reroute { .. } => {
                Some(take_or_clone_texture(doc, &mut textures, &mut use_counts, id, 0, w, h))
            }
            DarkNode::CustomWgsl { code, .. } => {
                let in_img = take_or_clone_texture(doc, &mut textures, &mut use_counts, id, 0, w, h);
                let param = modulated_scalar_override(doc, &sig_ctx, id, 1, 1.0);
                if WgslParser::parse_uniforms(code).is_ok()
                    && let Some(runner) = WGSL_RUNNER.get()
                    && let Ok(rendered) = runner(code, &in_img, time_sec as f32, param)
                {
                    recycle_f32(in_img.px);
                    Some(rendered)
                } else {
                    Some(in_img)
                }
            }
            DarkNode::Output => {
                final_output = Some(take_or_clone_texture(doc, &mut textures, &mut use_counts, id, 0, w, h));
                None
            }
            DarkNode::TimeDriver { .. }
            | DarkNode::AudioReactor { .. }
            | DarkNode::ActionNode { .. }
            | DarkNode::BlackNode { .. }
            | DarkNode::StickyNote { .. } => None,
        };

        if let Some(img) = out_img {
            textures.insert((id, 0), img);
        }
    }

    visited.remove(&item.id);
    let out = final_output.or_else(|| {
        plan.steps
            .iter()
            .rev()
            .find_map(|s| textures.remove(&(s.node_id, 0)))
    });
    for (_, img) in textures {
        recycle_f32(img.px);
    }
    out
}

#[cfg(test)]
#[path = "node_graph_tests.rs"]
mod tests;
