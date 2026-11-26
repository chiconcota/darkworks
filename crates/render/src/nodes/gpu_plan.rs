//! Biên dịch nhánh Node Graph tuyến tính thành danh sách [`FxOp`] chạy 100% trên GPU (`LayerFx`).

use std::collections::HashMap;

use darkworks_node_graph_core::{
    BlurDimensions, BlurType, DagEvaluator, DarkNode, DistortType, NodeGraphDoc,
};
use filmcraft_geom::{Affine, Vec2};
use filmcraft_project::{Project, Sequence, TrackItem};
use filmcraft_time::Tick;

use crate::effects::gaussian_boxes;
use crate::gpufx::{FxOp, Resample};
use crate::nodes::signals::{
    compute_audio_rms_map, modulated_scalar, modulated_scalar_add, modulated_scalar_add_scaled,
    modulated_scalar_mul, modulated_vec2_add,
};
use crate::{RenderOptions, SourceProvider};

/// Kiểm tra đồ thị có cần đọc layer bên dưới (`All Below` / `V1..V4`) hay không.
pub(crate) fn graph_needs_below(doc: &NodeGraphDoc) -> bool {
    doc.nodes.iter().any(|e| {
        if let DarkNode::VideoInput { track_name } = &e.node {
            let k = track_name.trim();
            !k.is_empty() && !k.eq_ignore_ascii_case("Auto") && !k.eq_ignore_ascii_case("Clip")
        } else {
            false
        }
    })
}

/// Kiểm tra đồ thị có dùng lại hình ảnh của layer bên dưới để tự trộn (`Blend`/`Output`) hay không.
pub(crate) fn graph_replaces_below(doc: &NodeGraphDoc) -> bool {
    graph_needs_below(doc)
        && doc.nodes.iter().any(|e| matches!(e.node, DarkNode::Blend { .. }))
}

fn find_linear_chain(doc: &NodeGraphDoc, steps_ids: &[usize]) -> Option<Vec<usize>> {
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

    let mut rev_chain = Vec::new();
    let mut cur = target_id;
    let mut guard = 0usize;
    while guard <= doc.nodes.len() {
        guard += 1;
        let entry = doc.node(cur)?;
        match &entry.node {
            DarkNode::Output | DarkNode::Reroute { .. } => {
                let (src, _) = doc.source_for_pin(cur, 0)?;
                cur = src;
            }
            DarkNode::VideoInput { track_name } => {
                let k = track_name.trim();
                if k.is_empty() || k.eq_ignore_ascii_case("Auto") || k.eq_ignore_ascii_case("Clip") {
                    rev_chain.reverse();
                    return Some(rev_chain);
                }
                return None;
            }
            DarkNode::Keyer { .. }
            | DarkNode::ColorGrade { .. }
            | DarkNode::CropMask { .. }
            | DarkNode::Transform2D { .. } => {
                rev_chain.push(cur);
                let (src, _) = doc.source_for_pin(cur, 0)?;
                cur = src;
            }
            DarkNode::BlurSharpen { blur_type, .. } => {
                if matches!(blur_type, BlurType::RadialZoom) {
                    return None;
                }
                rev_chain.push(cur);
                let (src, _) = doc.source_for_pin(cur, 0)?;
                cur = src;
            }
            DarkNode::DistortLens { distort_type, .. } => {
                if !matches!(
                    distort_type,
                    DistortType::Vignette | DistortType::Mirror | DistortType::Offset | DistortType::Posterize
                ) {
                    return None;
                }
                rev_chain.push(cur);
                let (src, _) = doc.source_for_pin(cur, 0)?;
                cur = src;
            }
            _ => return None,
        }
    }
    None
}

/// Biên dịch đồ thị Node Graph tuyến tính của `item` thành chuỗi [`FxOp`] chạy GPU; `None` nếu cần CPU.
#[allow(clippy::too_many_arguments)]
pub(crate) fn eval_graph_to_fx_ops(
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
) -> Option<Vec<FxOp>> {
    if doc.is_empty() || !opts.effects {
        return None;
    }
    let plan = DagEvaluator::build_execution_plan(doc).ok()?;
    let step_ids: Vec<usize> = plan.steps.iter().map(|s| s.node_id).collect();
    let chain = find_linear_chain(doc, &step_ids)?;

    let time_sec = (t - item.start).seconds().max(0.0);
    let audio_map: HashMap<usize, f32> = compute_audio_rms_map(project, seq, item, t, &plan, sources);
    let fallback_rms = audio_map.values().copied().next().unwrap_or(0.0);
    let sig_ctx = DagEvaluator::evaluate_signals_with_audio(&plan, time_sec, fallback_rms, &audio_map);

    let sx = lw as f32 / seq.settings.width.max(1) as f32;
    let sy = lh as f32 / seq.settings.height.max(1) as f32;
    let mut ops = Vec::new();

    for id in chain {
        let entry = doc.node(id)?;
        match &entry.node {
            DarkNode::Keyer {
                key_color,
                similarity,
                smoothness,
                spill_suppress,
            } => ops.push(FxOp::ChromaKey {
                key_color: *key_color,
                similarity: *similarity,
                smoothness: *smoothness,
                spill_suppress: *spill_suppress,
            }),
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
                if *black_and_white {
                    ops.push(FxOp::BlackWhite);
                }
                if *invert {
                    ops.push(FxOp::Invert { channel: 0, blend: 0.0 });
                }
                let eff_exp = modulated_scalar_add_scaled(doc, &sig_ctx, id, 1, *exposure, 2.5);
                let has_split = shadow_tint.iter().any(|&v| (v - 0.5).abs() > 1e-3)
                    || highlight_tint.iter().any(|&v| (v - 0.5).abs() > 1e-3);
                let use_lum = highlights.abs() > 1e-3
                    || shadows.abs() > 1e-3
                    || whites.abs() > 1e-3
                    || blacks.abs() > 1e-3
                    || vibrance.abs() > 1e-3
                    || faded_film.abs() > 1e-3
                    || has_split;
                let temp = (temperature / 100.0).clamp(-1.5, 1.5);
                let tn = (tint / 100.0).clamp(-1.5, 1.5);
                if use_lum {
                    let aspect = if lh > 0 { (lw as f32 / lh as f32).max(1e-4) } else { 1.0 };
                    ops.push(FxOp::Lumetri {
                        gains: [1.0 + 0.35 * temp, 1.0 - 0.3 * tn, 1.0 - 0.35 * temp],
                        exposure: 2.0_f32.powf(eff_exp.clamp(-5.0, 5.0)),
                        contrast: (*contrast - 1.0).clamp(-1.0, 2.0),
                        hl: *highlights / 100.0,
                        sh: *shadows / 100.0,
                        wh: *whites / 100.0,
                        bl: *blacks / 100.0,
                        sat: saturation.max(0.0),
                        creative_on: has_split || faded_film.abs() > 1e-3 || vibrance.abs() > 1e-3,
                        faded: (*faded_film / 100.0).clamp(0.0, 1.0),
                        vib: *vibrance / 100.0,
                        st: *shadow_tint,
                        ht: *highlight_tint,
                        vignette_on: false,
                        va: 0.0,
                        vmid: 0.5,
                        vround: 0.0,
                        vfeather: 0.5,
                        aspect,
                        gpu_capable: true,
                    });
                } else {
                    if eff_exp.abs() > 1e-4 {
                        ops.push(FxOp::Lumetri {
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
                        });
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
            DarkNode::BlurSharpen {
                blur_type,
                radius,
                angle_deg,
                highlight_boost,
                threshold,
                dimensions,
                repeat_edge,
                bokeh_sides,
                ..
            } => {
                let eff_r = (modulated_scalar_mul(doc, &sig_ctx, id, 1, *radius) * px_scale).max(0.0);
                if eff_r >= 0.1 {
                    let (rx_on, ry_on) = match dimensions {
                        BlurDimensions::Both => (eff_r, eff_r),
                        BlurDimensions::Horizontal => (eff_r, 0.0),
                        BlurDimensions::Vertical => (0.0, eff_r),
                    };
                    match blur_type {
                        BlurType::Gaussian => {
                            let (rx, ry) = gaussian_boxes(lw, lh, rx_on, ry_on);
                            ops.push(FxOp::Gaussian { rx, ry, repeat: *repeat_edge });
                        }
                        BlurType::Box => {
                            let b = (eff_r.round() as u32).max(1);
                            ops.push(FxOp::Gaussian {
                                rx: if rx_on > 0.0 { vec![b] } else { Vec::new() },
                                ry: if ry_on > 0.0 { vec![b] } else { Vec::new() },
                                repeat: *repeat_edge,
                            });
                        }
                        BlurType::Directional => {
                            let rad = angle_deg.to_radians();
                            let steps = ((eff_r * 0.75).round() as u32).clamp(9, 25);
                            ops.push(FxOp::DirectionalBlur { dx: rad.cos() * eff_r, dy: rad.sin() * eff_r, steps });
                        }
                        BlurType::BokehBloom => ops.push(FxOp::Bokeh {
                            levels: 2,
                            boost: highlight_boost.max(0.0),
                            threshold: threshold.clamp(0.0, 1.0),
                            sides: u32::from((*bokeh_sides).min(8)),
                            rot: angle_deg.to_radians(),
                            rr: eff_r,
                            rings: 3,
                        }),
                        BlurType::UnsharpMask => {
                            let (rx, ry) = gaussian_boxes(lw, lh, eff_r, eff_r);
                            ops.push(FxOp::Unsharp {
                                rx,
                                ry,
                                amount: highlight_boost.max(0.2),
                                threshold: (*threshold * 0.2).clamp(0.0, 0.5),
                            });
                        }
                        BlurType::RadialZoom => return None,
                    }
                }
            }
            DarkNode::DistortLens {
                distort_type,
                intensity,
                radius,
                center,
                feather,
            } => {
                let eff_int = modulated_scalar(doc, &sig_ctx, id, 1, *intensity);
                match distort_type {
                    DistortType::Vignette => ops.push(FxOp::Vignette {
                        amount: eff_int,
                        midpoint: radius.clamp(0.0, 1.0),
                        roundness: 0.0,
                        feather: feather.clamp(0.01, 1.0),
                        target: [0.0, 0.0, 0.0],
                    }),
                    DistortType::Mirror => {
                        let rad = (eff_int * 180.0).to_radians();
                        ops.push(FxOp::Mirror {
                            cx: lw as f32 * (0.5 + center[0] * 0.5),
                            cy: lh as f32 * (0.5 + center[1] * 0.5),
                            nx: rad.cos(),
                            ny: rad.sin(),
                        });
                    }
                    DistortType::Offset => {
                        let (wf, hf) = (lw.max(1) as f32, lh.max(1) as f32);
                        ops.push(FxOp::Offset {
                            dx: (center[0] * wf * 0.5).rem_euclid(wf),
                            dy: (center[1] * hf * 0.5).rem_euclid(hf),
                            blend: (1.0 - feather).clamp(0.0, 1.0),
                        });
                    }
                    DistortType::Posterize => ops.push(FxOp::Posterize {
                        n: (2.0 + eff_int.abs() * 14.0).clamp(2.0, 64.0),
                    }),
                    _ => return None,
                }
            }
            DarkNode::CropMask { top, bottom, left, right, feather } => ops.push(FxOp::Crop {
                x0: (left.clamp(0.0, 1.0) * lw as f32).min(lw as f32),
                x1: ((1.0 - right.clamp(0.0, 1.0)) * lw as f32).max((left.clamp(0.0, 1.0) * lw as f32).min(lw as f32)),
                y0: (top.clamp(0.0, 1.0) * lh as f32).min(lh as f32),
                y1: ((1.0 - bottom.clamp(0.0, 1.0)) * lh as f32).max((top.clamp(0.0, 1.0) * lh as f32).min(lh as f32)),
                feather: (*feather * px_scale).max(0.0),
            }),
            DarkNode::Transform2D {
                position,
                scale,
                rotation_deg,
                anchor,
                opacity,
                flip_h,
                flip_v,
            } => {
                if *flip_h {
                    ops.push(FxOp::HFlip);
                }
                if *flip_v {
                    ops.push(FxOp::VFlip);
                }
                let pos = modulated_vec2_add(doc, &sig_ctx, id, 1, *position);
                let s_mod = modulated_scalar_mul(doc, &sig_ctx, id, 2, 1.0);
                let rot_mod = modulated_scalar_add(doc, &sig_ctx, id, 3, *rotation_deg);
                let op_mod = modulated_scalar_mul(doc, &sig_ctx, id, 4, *opacity).clamp(0.0, 1.0);
                let sc = [scale[0] * s_mod, scale[1] * s_mod];
                if sc[0].abs() < 0.5 || sc[1].abs() < 0.5 {
                    return None;
                }
                let px_pos = Vec2::new(lw as f64 * 0.5 + pos[0] as f64 * sx as f64, lh as f64 * 0.5 + pos[1] as f64 * sy as f64);
                let px_anchor = Vec2::new(lw as f64 * anchor[0] as f64, lh as f64 * anchor[1] as f64);
                let m = Affine::motion(px_pos, Vec2::new(sc[0] as f64, sc[1] as f64), rot_mod as f64, px_anchor);
                ops.push(FxOp::Resample(Resample {
                    inv: m.inverse(),
                    rect: [0, lw as u32, 0, lh as u32],
                    opacity: op_mod,
                }));
            }
            _ => return None,
        }
    }

    ops.iter().all(FxOp::gpu_ok).then_some(ops)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::Image;
    use crate::plan::{DeviceCaps, FramePlan, execute_cpu, plan_frame};
    use crate::{SharedSource, SourceMap};
    use darkworks_node_graph_core::{NodeEntry, NodeGraphDoc, NodeWire};
    use filmcraft_media::generators::GeneratorSource;
    use filmcraft_media::{Generator, MediaSource};
    use filmcraft_project::{ItemKind, Label, MediaClip, MediaRef, SequenceSettings, TrackKind};
    use filmcraft_time::{FrameRate, TICKS_PER_SECOND, TimeRange};
    use std::sync::Arc;

    fn add_matte(
        p: &mut Project,
        map: &mut SourceMap,
        color: [f32; 4],
    ) -> filmcraft_project::ItemId {
        let g = GeneratorSource::new(
            Generator::ColorMatte { color },
            32,
            32,
            FrameRate::FPS_24,
            Tick(2 * TICKS_PER_SECOND),
        );
        let info = g.info().clone();
        let generator = g.generator.clone();
        let id = p.add_item(
            &info.name.clone(),
            Label::Iris,
            ItemKind::Media(Box::new(MediaClip {
                media: MediaRef::Generator(generator),
                info,
                interpret: Default::default(),
                mark_in: None,
                mark_out: None,
                markers: vec![],
                offline: false,
                proxy: None,
                identity: None,
            })),
            None,
        );
        map.0.insert(id, Arc::new(g) as SharedSource);
        id
    }

    fn keyer_doc(track_name: &str) -> NodeGraphDoc {
        let mut doc = NodeGraphDoc::new();
        doc.nodes = vec![
            NodeEntry {
                id: 1,
                node: DarkNode::VideoInput { track_name: track_name.into() },
                pos: [0.0, 0.0],
                custom_title: None,
            },
            NodeEntry {
                id: 2,
                node: DarkNode::Keyer {
                    key_color: [0.0, 1.0, 0.0],
                    similarity: 0.4,
                    smoothness: 0.2,
                    spill_suppress: 0.9,
                },
                pos: [100.0, 0.0],
                custom_title: None,
            },
            NodeEntry {
                id: 3,
                node: DarkNode::Output,
                pos: [200.0, 0.0],
                custom_title: None,
            },
        ];
        doc.wires = vec![
            NodeWire { out_node: 1, out_pin: 0, in_node: 2, in_pin: 0 },
            NodeWire { out_node: 2, out_pin: 0, in_node: 3, in_pin: 0 },
        ];
        doc
    }

    #[test]
    fn linear_keyer_graph_compiles_to_gpu_plan_without_cpu_fallback() {
        let mut p = Project::new("keyer-gpu");
        let mut map = SourceMap::default();
        let red_bg = add_matte(&mut p, &mut map, [1.0, 0.0, 0.0, 1.0]);
        let green_fg = add_matte(&mut p, &mut map, [0.0, 1.0, 0.0, 1.0]);
        let seq = p.new_sequence(
            "s",
            SequenceSettings { width: 32, height: 32, frame_rate: FrameRate::FPS_24, ..Default::default() },
            2,
            2,
            None,
        );
        let r = FrameRate::FPS_24;
        for (track, item_id, with_keyer) in [(0, red_bg, false), (1, green_fg, true)] {
            let mut ti = p
                .make_track_item(item_id, TrackKind::Video, Tick::ZERO, TimeRange::new(Tick::ZERO, r.tick_of(24)), r)
                .unwrap();
            ti.scale_to_frame = true;
            if with_keyer {
                ti.node_graph = Some(Box::new(keyer_doc("Auto")));
            }
            p.sequence_mut(seq).unwrap().video_tracks[track].items.push(ti);
        }

        let s = p.sequence(seq).unwrap();
        let top = &s.video_tracks[1].items[0];
        let doc = top.node_graph.as_ref().unwrap();
        assert!(!graph_needs_below(doc));
        let ops = eval_graph_to_fx_ops(&p, s, top, doc, Tick(1000), RenderOptions::default(), 32, 32, 1.0, &map);
        assert!(matches!(ops.as_deref(), Some([FxOp::ChromaKey { .. }])));

        let plan = plan_frame(&p, seq, Tick(1000), RenderOptions::default(), DeviceCaps::ALL, &map);
        match &plan {
            FramePlan::Layers { layers, .. } => {
                assert_eq!(layers.len(), 2);
                assert!(layers[1].fx.as_ref().is_some_and(|fx| matches!(fx.ops.as_slice(), [FxOp::ChromaKey { .. }])));
            }
            FramePlan::Composite { .. } => panic!("expected GPU FramePlan::Layers, got Composite"),
            FramePlan::Image(_) => panic!("expected GPU FramePlan::Layers, got CPU FramePlan::Image"),
        }
        let img = execute_cpu(&plan);
        let c = img.get(16, 16);
        assert!(c[0] > 0.99 && c[1] < 0.01 && c[2] < 0.01, "expected red below keyed green: {c:?}");
    }

    #[test]
    fn chroma_key_removes_antialiased_green_fringe() {
        let mut img = Image {
            w: 4,
            h: 1,
            px: vec![
                1.0, 0.0, 0.0, 1.0, // foreground red
                0.35, 0.55, 0.05, 1.0, // green-fringe edge next to key green
                0.02, 0.90, 0.02, 1.0, // key green background
                0.0, 1.0, 0.0, 1.0, // pure key green
            ],
        };
        FxOp::ChromaKey {
            key_color: [0.0, 1.0, 0.0],
            similarity: 0.4,
            smoothness: 0.2,
            spill_suppress: 0.95,
        }
        .apply(&mut img);
        assert!(img.get(0, 0)[3] > 0.99, "foreground stays opaque");
        assert!(img.get(3, 0)[3] < 0.01, "pure green becomes transparent");
        let edge = img.get(1, 0);
        assert!(edge[1] <= edge[0].max(edge[2]) + 0.06, "green fringe must be suppressed on edge: {edge:?}");
    }

    #[test]
    fn graph_needs_below_distinguishes_auto_from_track_refs() {
        assert!(!graph_needs_below(&keyer_doc("Auto")));
        assert!(!graph_needs_below(&keyer_doc("Clip")));
        assert!(graph_needs_below(&keyer_doc("V1")));
        assert!(graph_needs_below(&keyer_doc("All Below")));
    }
}
