//! Biên dịch nhánh Node Graph tuyến tính (và nhánh `Blend(All Below)`) thành [`FxOp`] chạy 100% trên GPU.

use std::collections::HashMap;

use darkworks_node_graph_core::{BlendMode, DagEvaluator, DarkNode, NodeGraphDoc};
use filmcraft_project::{Project, Sequence, TrackItem};
use filmcraft_time::Tick;

use crate::Blend;
use crate::gpufx::FxOp;
use crate::nodes::signals::compute_audio_rms_map;
use crate::nodes::transform_blend::map_blend;
use crate::nodes::{blur_distort, color, transform_blend};
use crate::{RenderOptions, SourceProvider};

/// Kết quả biên dịch đồ thị Node Graph chạy 100% trên GPU.
#[derive(Clone, Debug)]
pub(crate) struct GpuGraphPlan {
    pub ops: Vec<FxOp>,
    pub blend_override: Option<(Blend, f32)>,
}

fn is_auto_input(track_name: &str) -> bool {
    let k = track_name.trim();
    k.is_empty() || k.eq_ignore_ascii_case("Auto") || k.eq_ignore_ascii_case("Clip")
}

fn is_self_input(track_name: &str, self_track: Option<(usize, &str)>) -> bool {
    if is_auto_input(track_name) {
        return true;
    }
    let Some((idx, tr_name)) = self_track else {
        return false;
    };
    let k = track_name.trim();
    if k.eq_ignore_ascii_case(tr_name.trim()) {
        return true;
    }
    k.strip_prefix('V')
        .or_else(|| k.strip_prefix('v'))
        .and_then(|rest| rest.parse::<usize>().ok())
        .is_some_and(|one_based| one_based == idx + 1)
}

fn trace_reroutes(doc: &NodeGraphDoc, mut id: usize) -> Option<usize> {
    let mut guard = 0usize;
    while guard <= doc.nodes.len() {
        guard += 1;
        let entry = doc.node(id)?;
        if matches!(entry.node, DarkNode::Reroute { .. }) {
            id = doc.source_for_pin(id, 0)?.0;
        } else {
            return Some(id);
        }
    }
    None
}

fn matches_below_track(track_name: &str, single_below_track: Option<&str>) -> bool {
    let k = track_name.trim();
    k.eq_ignore_ascii_case("All Below")
        || single_below_track.is_some_and(|b| k.eq_ignore_ascii_case(b))
}

/// Kiểm tra đồ thị có cần đọc layer bên dưới (`All Below` / track khác) hay không.
pub(crate) fn graph_needs_below_for(doc: &NodeGraphDoc, self_track: Option<(usize, &str)>) -> bool {
    doc.nodes.iter().any(|e| {
        if let DarkNode::VideoInput { track_name } = &e.node {
            !is_self_input(track_name, self_track)
        } else {
            false
        }
    })
}

/// Kiểm tra đồ thị có cần đọc layer bên dưới (`All Below` / `V1..V4`) hay không.
pub(crate) fn graph_needs_below(doc: &NodeGraphDoc) -> bool {
    graph_needs_below_for(doc, None)
}

/// Kiểm tra đồ thị có dùng lại hình ảnh của layer bên dưới để tự trộn (`Blend`/`Output`) hay không.
pub(crate) fn graph_replaces_below(doc: &NodeGraphDoc) -> bool {
    graph_needs_below(doc)
        && doc.nodes.iter().any(|e| matches!(e.node, DarkNode::Blend { .. }))
}

fn trace_linear_from(
    doc: &NodeGraphDoc,
    start_id: usize,
    allow_all_below_root: bool,
    self_track: Option<(usize, &str)>,
) -> Option<Vec<usize>> {
    let mut rev_chain = Vec::new();
    let mut cur = start_id;
    let mut guard = 0usize;
    while guard <= doc.nodes.len() {
        guard += 1;
        let entry = doc.node(cur)?;
        match &entry.node {
            DarkNode::Output | DarkNode::Reroute { .. } => {
                cur = doc.source_for_pin(cur, 0)?.0;
            }
            DarkNode::VideoInput { track_name } => {
                let k = track_name.trim();
                let ok = is_self_input(k, self_track)
                    || (allow_all_below_root && k.eq_ignore_ascii_case("All Below"));
                if ok {
                    rev_chain.reverse();
                    return Some(rev_chain);
                }
                return None;
            }
            DarkNode::Keyer { .. } => {
                // Nếu nối pin 1 (Garbage Matte) thì fallback sang đường DAG đa luồng CPU.
                if doc.source_for_pin(cur, 1).is_some() {
                    return None;
                }
                rev_chain.push(cur);
                cur = doc.source_for_pin(cur, 0)?.0;
            }
            DarkNode::ColorGrade { .. }
            | DarkNode::CropMask { .. }
            | DarkNode::Transform2D { .. }
            | DarkNode::BlurSharpen { .. }
            | DarkNode::DistortLens { .. } => {
                rev_chain.push(cur);
                cur = doc.source_for_pin(cur, 0)?.0;
            }
            DarkNode::Blend { mode, .. } => {
                // Chỉ pass-through lên GPU khi nối đúng 1 đầu (không có Mask); nối cả 2 đầu do `find_gpu_chain` xử lý riêng.
                if doc.source_for_pin(cur, 2).is_some() {
                    return None;
                }
                match (doc.source_for_pin(cur, 0), doc.source_for_pin(cur, 1)) {
                    (Some((src, _)), None) => {
                        cur = src;
                    }
                    (None, Some((src, _))) if *mode != BlendMode::Dissolve => {
                        rev_chain.push(cur);
                        cur = src;
                    }
                    _ => return None,
                }
            }
            _ => return None,
        }
    }
    None
}

fn find_gpu_chain(
    doc: &NodeGraphDoc,
    steps_ids: &[usize],
    allow_all_below_root: bool,
    self_track: Option<(usize, &str)>,
    single_below_track: Option<&str>,
) -> Option<(Vec<usize>, Option<(BlendMode, f32)>)> {
    let target_id = doc
        .nodes
        .iter()
        .find(|n| matches!(n.node, DarkNode::Output))
        .map(|n| n.id)
        .or_else(|| {
            steps_ids.iter().rev().copied().find(|id| {
                doc.node(*id).is_some_and(|n| {
                    !matches!(
                        n.node,
                        DarkNode::TimeDriver { .. }
                            | DarkNode::AudioReactor { .. }
                            | DarkNode::ActionNode { .. }
                            | DarkNode::StickyNote { .. }
                    )
                })
            })
        })?;

    if !allow_all_below_root {
        let root_id = trace_reroutes(doc, target_id).and_then(|id| {
            let entry = doc.node(id)?;
            if matches!(entry.node, DarkNode::Output) {
                trace_reroutes(doc, doc.source_for_pin(id, 0)?.0)
            } else {
                Some(id)
            }
        });
        if let Some(blend_id) = root_id
            && let Some(blend_entry) = doc.node(blend_id)
            && let DarkNode::Blend { mode, opacity } = &blend_entry.node
            && doc.source_for_pin(blend_id, 2).is_none()
            && let Some((bg_src, _)) = doc.source_for_pin(blend_id, 0)
            && let Some(bg_node_id) = trace_reroutes(doc, bg_src)
            && let Some(bg_entry) = doc.node(bg_node_id)
            && let DarkNode::VideoInput { track_name } = &bg_entry.node
            && matches_below_track(track_name, single_below_track)
            && let Some((fg_src, _)) = doc.source_for_pin(blend_id, 1)
            && let Some(chain) = trace_linear_from(doc, fg_src, false, self_track)
        {
            return Some((chain, Some((*mode, *opacity))));
        }
    }

    trace_linear_from(doc, target_id, allow_all_below_root, self_track).map(|c| (c, None))
}

/// Kiểm tra nhanh đồ thị Clip có biên dịch trọn vẹn lên GPU không (dùng trong `layered_at`).
pub(crate) fn can_plan_clip_graph(
    doc: &NodeGraphDoc,
    self_track: Option<(usize, &str)>,
    single_below_track: Option<&str>,
) -> bool {
    if doc.is_empty() {
        return true;
    }
    let Ok(plan) = DagEvaluator::build_execution_plan(doc) else {
        return false;
    };
    let step_ids: Vec<usize> = plan.steps.iter().map(|s| s.node_id).collect();
    find_gpu_chain(doc, &step_ids, false, self_track, single_below_track).is_some()
}

/// Kiểm tra nhanh đồ thị AdjustmentLayer có biên dịch sang `PlanStep::Adjust` GPU không.
pub(crate) fn can_plan_adjustment_graph(doc: &NodeGraphDoc) -> bool {
    if doc.is_empty() {
        return false;
    }
    let Ok(plan) = DagEvaluator::build_execution_plan(doc) else {
        return false;
    };
    let step_ids: Vec<usize> = plan.steps.iter().map(|s| s.node_id).collect();
    find_gpu_chain(doc, &step_ids, true, None, None).is_some_and(|(_, blend)| blend.is_none())
}

#[allow(clippy::too_many_arguments)]
fn compile_chain_ops(
    project: &Project,
    seq: &Sequence,
    item: &TrackItem,
    doc: &NodeGraphDoc,
    chain: &[usize],
    plan: &darkworks_node_graph_core::DagDocPlan,
    t: Tick,
    lw: usize,
    lh: usize,
    px_scale: f32,
    sources: Option<&dyn SourceProvider>,
) -> Option<Vec<FxOp>> {
    let time_sec = (t - item.start).seconds().max(0.0);
    let clip_duration_sec = item.duration.seconds().max(0.0);
    let audio_map: HashMap<usize, f32> = sources
        .map(|s| compute_audio_rms_map(project, seq, item, t, plan, s))
        .unwrap_or_default();
    let fallback_rms = audio_map.values().copied().next().unwrap_or(0.0);
    let sig_ctx = DagEvaluator::evaluate_signals_for_clip(
        plan,
        time_sec,
        clip_duration_sec,
        fallback_rms,
        &audio_map,
    );

    let sx = lw as f32 / seq.settings.width.max(1) as f32;
    let sy = lh as f32 / seq.settings.height.max(1) as f32;
    let mut ops = Vec::new();

    for &id in chain {
        let entry = doc.node(id)?;
        match &entry.node {
            DarkNode::Keyer { .. } => color::push_keyer_gpu_op(&entry.node, px_scale, &mut ops),
            DarkNode::ColorGrade { .. } => {
                color::push_color_grade_gpu_ops(doc, &sig_ctx, id, &entry.node, [lw, lh], &mut ops);
            }
            DarkNode::BlurSharpen { .. } => blur_distort::push_blur_gpu_ops(
                doc,
                &sig_ctx,
                id,
                &entry.node,
                [lw, lh],
                px_scale,
                &mut ops,
            ),
            DarkNode::DistortLens { .. } => blur_distort::push_distort_gpu_ops(
                doc,
                &sig_ctx,
                id,
                &entry.node,
                [lw, lh],
                (px_scale, time_sec),
                &mut ops,
            ),
            DarkNode::CropMask { .. } => {
                transform_blend::push_crop_gpu_op(&entry.node, [lw, lh], px_scale, &mut ops);
            }
            DarkNode::Transform2D { .. } => {
                transform_blend::push_transform_gpu_ops(
                    doc,
                    &sig_ctx,
                    id,
                    &entry.node,
                    [lw, lh],
                    [sx, sy],
                    &mut ops,
                )?;
            }
            DarkNode::Blend { opacity, .. } => {
                let op = opacity.clamp(0.0, 1.0);
                if (op - 1.0).abs() > 1e-4 {
                    ops.push(FxOp::AlphaAdjust {
                        opacity: op,
                        ignore: false,
                        invert: false,
                        mask_only: false,
                    });
                }
            }
            _ => return None,
        }
    }

    ops.iter().all(FxOp::gpu_ok).then_some(ops)
}

/// Biên dịch đồ thị Node Graph của Media Clip thành [`GpuGraphPlan`] (`LayerFx` + optional `Blend`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn eval_graph_to_fx_plan(
    project: &Project,
    seq: &Sequence,
    item: &TrackItem,
    doc: &NodeGraphDoc,
    t: Tick,
    opts: RenderOptions,
    lw: usize,
    lh: usize,
    px_scale: f32,
    sources: &dyn SourceProvider,
    self_track: Option<(usize, &str)>,
    single_below_track: Option<&str>,
) -> Option<GpuGraphPlan> {
    if doc.is_empty() || !opts.effects {
        return None;
    }
    let plan = DagEvaluator::build_execution_plan(doc).ok()?;
    let step_ids: Vec<usize> = plan.steps.iter().map(|s| s.node_id).collect();
    let (chain, blend_opt) = find_gpu_chain(doc, &step_ids, false, self_track, single_below_track)?;
    let ops = compile_chain_ops(
        project,
        seq,
        item,
        doc,
        &chain,
        &plan,
        t,
        lw,
        lh,
        px_scale,
        Some(sources),
    )?;
    let blend_override = blend_opt.map(|(m, op)| (map_blend(m), op.clamp(0.0, 1.0)));
    Some(GpuGraphPlan { ops, blend_override })
}

/// Biên dịch đồ thị Node Graph của `AdjustmentLayer` thành chuỗi [`FxOp`] cho `PlanStep::Adjust`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn eval_adjustment_graph_to_fx_ops(
    project: &Project,
    seq: &Sequence,
    item: &TrackItem,
    doc: &NodeGraphDoc,
    t: Tick,
    opts: RenderOptions,
    lw: usize,
    lh: usize,
    px_scale: f32,
    sources: Option<&dyn SourceProvider>,
) -> Option<Vec<FxOp>> {
    if doc.is_empty() || !opts.effects {
        return None;
    }
    let plan = DagEvaluator::build_execution_plan(doc).ok()?;
    let step_ids: Vec<usize> = plan.steps.iter().map(|s| s.node_id).collect();
    let (chain, blend_opt) = find_gpu_chain(doc, &step_ids, true, None, None)?;
    if blend_opt.is_some() {
        return None;
    }
    compile_chain_ops(project, seq, item, doc, &chain, &plan, t, lw, lh, px_scale, sources)
}

#[cfg(test)]
#[path = "gpu_plan_tests.rs"]
mod tests;
