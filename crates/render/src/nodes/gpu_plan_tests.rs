use super::*;
use crate::image::Image;
use crate::nodes::node_fx_ops::{KEYER_OUT_ALPHA, NodeFxOp, keyer_output_code};
use crate::plan::{DeviceCaps, FramePlan, PlanStep, execute_cpu, plan_frame};
use crate::{SharedSource, SourceMap};
use darkworks_node_graph_core::{BlendMode, DistortType, KeyerOutput, NodeEntry, NodeGraphDoc, NodeWire, TimeMode};
use filmcraft_media::generators::GeneratorSource;
use filmcraft_media::{Generator, MediaSource};
use filmcraft_project::{ItemKind, Label, MediaClip, MediaRef, SequenceSettings, TrackKind};
use filmcraft_time::{FrameRate, TICKS_PER_SECOND, TimeRange};
use std::sync::Arc;

fn add_matte(p: &mut Project, map: &mut SourceMap, color: [f32; 4]) -> filmcraft_project::ItemId {
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
            node: DarkNode::default_keyer(),
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
    let gplan = eval_graph_to_fx_plan(&p, s, top, doc, Tick(1000), RenderOptions::default(), 32, 32, 1.0, &map, Some((1, "V2")), Some("V1")).unwrap();
    assert!(matches!(gplan.ops.as_slice(), [FxOp::Node(NodeFxOp::ChromaKeyPro { .. })]));

    let plan = plan_frame(&p, seq, Tick(1000), RenderOptions::default(), DeviceCaps::ALL, &map);
    match &plan {
        FramePlan::Layers { layers, .. } => {
            assert_eq!(layers.len(), 2);
            assert!(layers[1].fx.as_ref().is_some_and(|fx| matches!(fx.ops.as_slice(), [FxOp::Node(NodeFxOp::ChromaKeyPro { .. })])));
        }
        FramePlan::Composite { .. } => panic!("expected GPU FramePlan::Layers, got Composite"),
        FramePlan::Image(_) => panic!("expected GPU FramePlan::Layers, got CPU FramePlan::Image"),
    }
    let img = execute_cpu(&plan);
    let c = img.get(16, 16);
    assert!(c[0] > 0.99 && c[1] < 0.01 && c[2] < 0.01, "expected red below keyed green: {c:?}");
}

#[test]
fn four_node_blend_all_below_graph_compiles_to_gpu_layers() {
    let mut p = Project::new("blend-below-gpu");
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
    let ti_bg = p.make_track_item(red_bg, TrackKind::Video, Tick::ZERO, TimeRange::new(Tick::ZERO, r.tick_of(24)), r).unwrap();
    p.sequence_mut(seq).unwrap().video_tracks[0].items.push(ti_bg);

    let mut ti_fg = p.make_track_item(green_fg, TrackKind::Video, Tick::ZERO, TimeRange::new(Tick::ZERO, r.tick_of(24)), r).unwrap();
    let mut doc = NodeGraphDoc::new();
    doc.nodes = vec![
        NodeEntry { id: 1, node: DarkNode::VideoInput { track_name: "Auto".into() }, pos: [0.0, 0.0], custom_title: None },
        NodeEntry { id: 2, node: DarkNode::default_keyer(), pos: [100.0, 0.0], custom_title: None },
        NodeEntry { id: 3, node: DarkNode::VideoInput { track_name: "All Below".into() }, pos: [0.0, 120.0], custom_title: None },
        NodeEntry { id: 4, node: DarkNode::Blend { mode: BlendMode::Screen, opacity: 0.75 }, pos: [220.0, 60.0], custom_title: None },
        NodeEntry { id: 5, node: DarkNode::Output, pos: [340.0, 60.0], custom_title: None },
    ];
    doc.wires = vec![
        NodeWire { out_node: 1, out_pin: 0, in_node: 2, in_pin: 0 },
        NodeWire { out_node: 3, out_pin: 0, in_node: 4, in_pin: 0 },
        NodeWire { out_node: 2, out_pin: 0, in_node: 4, in_pin: 1 },
        NodeWire { out_node: 4, out_pin: 0, in_node: 5, in_pin: 0 },
    ];
    ti_fg.node_graph = Some(Box::new(doc));
    p.sequence_mut(seq).unwrap().video_tracks[1].items.push(ti_fg);

    let plan = plan_frame(&p, seq, Tick(1000), RenderOptions::default(), DeviceCaps::ALL, &map);
    match &plan {
        FramePlan::Layers { layers, .. } => {
            assert_eq!(layers.len(), 2);
            assert_eq!(layers[1].blend, Blend::Screen);
            assert!((layers[1].opacity - 0.75).abs() < 1e-5);
            assert!(layers[1].fx.as_ref().is_some_and(|fx| !fx.ops.is_empty()));
        }
        other => panic!("expected GPU FramePlan::Layers for 4-node Blend(All Below), got {}", match other {
            FramePlan::Composite { .. } => "Composite",
            FramePlan::Image(_) => "CPU Image",
            _ => "other",
        }),
    }
}

#[test]
fn adjustment_layer_graph_compiles_to_gpu_composite_adjust_step() {
    let mut p = Project::new("fx-clip-gpu");
    let mut map = SourceMap::default();
    let red_bg = add_matte(&mut p, &mut map, [1.0, 0.0, 0.0, 1.0]);
    let adj_id = p.add_item(
        "FX Clip",
        Label::Iris,
        ItemKind::AdjustmentLayer {
            width: 32,
            height: 32,
            rate: FrameRate::FPS_24,
            duration: Tick(2 * TICKS_PER_SECOND),
        },
        None,
    );
    let seq = p.new_sequence(
        "s",
        SequenceSettings { width: 32, height: 32, frame_rate: FrameRate::FPS_24, ..Default::default() },
        2,
        2,
        None,
    );
    let r = FrameRate::FPS_24;
    let ti_bg = p.make_track_item(red_bg, TrackKind::Video, Tick::ZERO, TimeRange::new(Tick::ZERO, r.tick_of(24)), r).unwrap();
    p.sequence_mut(seq).unwrap().video_tracks[0].items.push(ti_bg);

    let mut ti_adj = p.make_track_item(adj_id, TrackKind::Video, Tick::ZERO, TimeRange::new(Tick::ZERO, r.tick_of(24)), r).unwrap();
    let mut doc = NodeGraphDoc::new();
    doc.nodes = vec![
        NodeEntry { id: 1, node: DarkNode::VideoInput { track_name: "All Below".into() }, pos: [0.0, 0.0], custom_title: None },
        NodeEntry { id: 2, node: DarkNode::default_distort(DistortType::Twirl, 0.5, 0.6, [0.0, 0.0]), pos: [100.0, 0.0], custom_title: None },
        NodeEntry { id: 3, node: DarkNode::Output, pos: [200.0, 0.0], custom_title: None },
    ];
    doc.wires = vec![
        NodeWire { out_node: 1, out_pin: 0, in_node: 2, in_pin: 0 },
        NodeWire { out_node: 2, out_pin: 0, in_node: 3, in_pin: 0 },
    ];
    ti_adj.node_graph = Some(Box::new(doc));
    p.sequence_mut(seq).unwrap().video_tracks[1].items.push(ti_adj);

    let plan = plan_frame(&p, seq, Tick(1000), RenderOptions::default(), DeviceCaps::ALL, &map);
    match &plan {
        FramePlan::Composite { steps, .. } => {
            assert!(steps.iter().any(|s| matches!(s, PlanStep::Adjust { ops } if matches!(ops.as_slice(), [FxOp::Node(NodeFxOp::Twirl { .. })]))));
        }
        other => panic!("expected GPU FramePlan::Composite with PlanStep::Adjust, got {}", match other {
            FramePlan::Layers { .. } => "Layers",
            FramePlan::Image(_) => "CPU Image",
            _ => "other",
        }),
    }
}

#[test]
fn chroma_key_pro_alpha_and_edge_fringe_behave_accurately() {
    let mut img = Image {
        w: 4,
        h: 1,
        px: vec![
            1.0, 0.0, 0.0, 1.0,
            0.35, 0.55, 0.05, 1.0,
            0.02, 0.90, 0.02, 1.0,
            0.0, 1.0, 0.0, 1.0,
        ],
    };
    FxOp::Node(NodeFxOp::ChromaKeyPro {
        key_color: [0.0, 1.0, 0.0],
        similarity: 0.4,
        smoothness: 0.2,
        spill_suppress: 0.95,
        pedestal: 0.10,
        highlight: 0.10,
        shadow: 0.50,
        choke: 0.0,
        soften: 0.0,
        matte_contrast: 0.0,
        invert_matte: false,
        desaturate: 0.25,
        spill_luma: 0.50,
        output: keyer_output_code(KeyerOutput::Composite),
        px_scale: 1.0,
    })
    .apply(&mut img);
    assert!(img.get(0, 0)[3] > 0.99);
    assert!(img.get(3, 0)[3] < 0.01);
    let edge = img.get(1, 0);
    assert!(edge[1] <= edge[0].max(edge[2]) + 0.06, "green fringe suppressed: {edge:?}");

    let mut alpha_view = Image { w: 2, h: 1, px: vec![1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0] };
    FxOp::Node(NodeFxOp::ChromaKeyPro {
        key_color: [0.0, 1.0, 0.0],
        similarity: 0.4,
        smoothness: 0.2,
        spill_suppress: 0.9,
        pedestal: 0.10,
        highlight: 0.10,
        shadow: 0.50,
        choke: 0.0,
        soften: 0.0,
        matte_contrast: 0.0,
        invert_matte: false,
        desaturate: 0.25,
        spill_luma: 0.50,
        output: KEYER_OUT_ALPHA,
        px_scale: 1.0,
    })
    .apply(&mut alpha_view);
    assert!(alpha_view.get(0, 0)[0] > 0.99 && alpha_view.get(0, 0)[3] > 0.99);
    assert!(alpha_view.get(1, 0)[0] < 0.01 && alpha_view.get(1, 0)[3] > 0.99);
}

#[test]
fn graph_needs_below_distinguishes_auto_from_track_refs() {
    assert!(!graph_needs_below(&keyer_doc("Auto")));
    assert!(!graph_needs_below(&keyer_doc("Clip")));
    assert!(graph_needs_below(&keyer_doc("V1")));
    assert!(graph_needs_below(&keyer_doc("All Below")));
    assert!(!graph_needs_below_for(&keyer_doc("V2"), Some((1, "V2"))));
    assert!(graph_needs_below_for(&keyer_doc("V1"), Some((1, "V2"))));
}

#[test]
fn video_input_matching_clip_track_compiles_to_gpu_layers() {
    let mut p = Project::new("self-track-gpu");
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
    let ti_bg = p.make_track_item(red_bg, TrackKind::Video, Tick::ZERO, TimeRange::new(Tick::ZERO, r.tick_of(24)), r).unwrap();
    p.sequence_mut(seq).unwrap().video_tracks[0].items.push(ti_bg);

    let mut ti_fg = p.make_track_item(green_fg, TrackKind::Video, Tick::ZERO, TimeRange::new(Tick::ZERO, r.tick_of(24)), r).unwrap();
    ti_fg.node_graph = Some(Box::new(keyer_doc("V2")));
    p.sequence_mut(seq).unwrap().video_tracks[1].items.push(ti_fg);

    let plan = plan_frame(&p, seq, Tick(1000), RenderOptions::default(), DeviceCaps::ALL, &map);
    match &plan {
        FramePlan::Layers { layers, .. } => {
            assert_eq!(layers.len(), 2);
            assert!(layers[1].fx.as_ref().is_some_and(|fx| !fx.ops.is_empty()));
        }
        other => panic!("expected GPU FramePlan::Layers for VideoInput(V2) on V2, got {}", match other {
            FramePlan::Composite { .. } => "Composite",
            FramePlan::Image(_) => "CPU Image",
            _ => "other",
        }),
    }
}

#[test]
fn video_input_matching_clip_track_with_blend_v1_compiles_to_gpu_layers() {
    let mut p = Project::new("self-track-blend-gpu");
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
    let ti_bg = p.make_track_item(red_bg, TrackKind::Video, Tick::ZERO, TimeRange::new(Tick::ZERO, r.tick_of(12)), r).unwrap();
    p.sequence_mut(seq).unwrap().video_tracks[0].items.push(ti_bg);

    let mut ti_fg = p.make_track_item(green_fg, TrackKind::Video, Tick::ZERO, TimeRange::new(Tick::ZERO, r.tick_of(24)), r).unwrap();
    let mut doc = NodeGraphDoc::new();
    doc.nodes = vec![
        NodeEntry { id: 1, node: DarkNode::VideoInput { track_name: "V2".into() }, pos: [0.0, 0.0], custom_title: None },
        NodeEntry { id: 2, node: DarkNode::default_keyer(), pos: [100.0, 0.0], custom_title: None },
        NodeEntry { id: 3, node: DarkNode::VideoInput { track_name: "V1".into() }, pos: [0.0, 120.0], custom_title: None },
        NodeEntry { id: 4, node: DarkNode::Blend { mode: BlendMode::Normal, opacity: 0.85 }, pos: [220.0, 60.0], custom_title: None },
        NodeEntry { id: 5, node: DarkNode::Output, pos: [340.0, 60.0], custom_title: None },
    ];
    doc.wires = vec![
        NodeWire { out_node: 1, out_pin: 0, in_node: 2, in_pin: 0 },
        NodeWire { out_node: 3, out_pin: 0, in_node: 4, in_pin: 0 },
        NodeWire { out_node: 2, out_pin: 0, in_node: 4, in_pin: 1 },
        NodeWire { out_node: 4, out_pin: 0, in_node: 5, in_pin: 0 },
    ];
    ti_fg.node_graph = Some(Box::new(doc));
    p.sequence_mut(seq).unwrap().video_tracks[1].items.push(ti_fg);

    // 1. Khi V1 có clip nền tại tick 1000
    let plan_with_bg = plan_frame(&p, seq, Tick(1000), RenderOptions::default(), DeviceCaps::ALL, &map);
    match &plan_with_bg {
        FramePlan::Layers { layers, .. } => {
            assert_eq!(layers.len(), 2);
            assert_eq!(layers[1].blend, Blend::Normal);
            assert!((layers[1].opacity - 0.85).abs() < 1e-5);
            assert!(layers[1].fx.as_ref().is_some_and(|fx| !fx.ops.is_empty()));
        }
        other => panic!("expected GPU FramePlan::Layers with V1 bg, got {}", match other {
            FramePlan::Composite { .. } => "Composite",
            FramePlan::Image(_) => "CPU Image",
            _ => "other",
        }),
    }

    // 2. Khi V1 trống tại tick cuối clip V2 (vẫn phải chạy GPU, không rơi về CPU)
    let plan_empty_bg = plan_frame(&p, seq, r.tick_of(18), RenderOptions::default(), DeviceCaps::ALL, &map);
    match &plan_empty_bg {
        FramePlan::Layers { layers, .. } => {
            assert_eq!(layers.len(), 1);
            assert_eq!(layers[0].blend, Blend::Normal);
            assert!((layers[0].opacity - 0.85).abs() < 1e-5);
            assert!(layers[0].fx.as_ref().is_some_and(|fx| !fx.ops.is_empty()));
        }
        other => panic!("expected GPU FramePlan::Layers with empty V1 gap, got {}", match other {
            FramePlan::Composite { .. } => "Composite",
            FramePlan::Image(_) => "CPU Image",
            _ => "other",
        }),
    }
}

#[test]
fn single_wired_blend_stays_on_gpu_layers() {
    let mut p = Project::new("single-blend-gpu");
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
    let ti_bg = p.make_track_item(red_bg, TrackKind::Video, Tick::ZERO, TimeRange::new(Tick::ZERO, r.tick_of(24)), r).unwrap();
    p.sequence_mut(seq).unwrap().video_tracks[0].items.push(ti_bg);

    // Case 1: VideoInput(Auto) -> Transform2D(0.5) -> Keyer -> Blend(Background only) -> Output
    let mut ti_fg = p.make_track_item(green_fg, TrackKind::Video, Tick::ZERO, TimeRange::new(Tick::ZERO, r.tick_of(24)), r).unwrap();
    let mut doc_bg_only = NodeGraphDoc::new();
    doc_bg_only.nodes = vec![
        NodeEntry { id: 1, node: DarkNode::VideoInput { track_name: "Auto".into() }, pos: [0.0, 0.0], custom_title: None },
        NodeEntry {
            id: 2,
            node: DarkNode::Transform2D {
                position: [0.0, 0.0],
                scale: [0.5, 0.5],
                rotation_deg: 0.0,
                anchor: [0.5, 0.5],
                opacity: 1.0,
                flip_h: false,
                flip_v: false,
                uniform_scale: true,
                skew: 0.0,
                skew_axis: 0.0,
            },
            pos: [100.0, 0.0],
            custom_title: None,
        },
        NodeEntry { id: 3, node: DarkNode::default_keyer(), pos: [200.0, 0.0], custom_title: None },
        NodeEntry { id: 4, node: DarkNode::Blend { mode: BlendMode::Normal, opacity: 1.0 }, pos: [300.0, 0.0], custom_title: None },
        NodeEntry { id: 5, node: DarkNode::Output, pos: [400.0, 0.0], custom_title: None },
    ];
    doc_bg_only.wires = vec![
        NodeWire { out_node: 1, out_pin: 0, in_node: 2, in_pin: 0 },
        NodeWire { out_node: 2, out_pin: 0, in_node: 3, in_pin: 0 },
        NodeWire { out_node: 3, out_pin: 0, in_node: 4, in_pin: 0 },
        NodeWire { out_node: 4, out_pin: 0, in_node: 5, in_pin: 0 },
    ];
    ti_fg.node_graph = Some(Box::new(doc_bg_only));
    p.sequence_mut(seq).unwrap().video_tracks[1].items.push(ti_fg);

    let plan1 = plan_frame(&p, seq, Tick(1000), RenderOptions::default(), DeviceCaps::ALL, &map);
    match &plan1 {
        FramePlan::Layers { layers, .. } => {
            assert_eq!(layers.len(), 2);
            let ops = &layers[1].fx.as_ref().expect("gpu fx").ops;
            assert!(matches!(ops.as_slice(), [FxOp::Resample(_), FxOp::Node(NodeFxOp::ChromaKeyPro { .. })]));
        }
        _ => panic!("expected GPU FramePlan::Layers for Blend(bg only)"),
    }

    // Case 2: VideoInput(Auto) -> Keyer -> Blend(Foreground only, opacity 0.6) -> Output
    let mut doc_fg_only = NodeGraphDoc::new();
    doc_fg_only.nodes = vec![
        NodeEntry { id: 1, node: DarkNode::VideoInput { track_name: "Auto".into() }, pos: [0.0, 0.0], custom_title: None },
        NodeEntry { id: 2, node: DarkNode::default_keyer(), pos: [100.0, 0.0], custom_title: None },
        NodeEntry { id: 3, node: DarkNode::Blend { mode: BlendMode::Screen, opacity: 0.6 }, pos: [200.0, 0.0], custom_title: None },
        NodeEntry { id: 4, node: DarkNode::Output, pos: [300.0, 0.0], custom_title: None },
    ];
    doc_fg_only.wires = vec![
        NodeWire { out_node: 1, out_pin: 0, in_node: 2, in_pin: 0 },
        NodeWire { out_node: 2, out_pin: 0, in_node: 3, in_pin: 1 },
        NodeWire { out_node: 3, out_pin: 0, in_node: 4, in_pin: 0 },
    ];
    p.sequence_mut(seq).unwrap().video_tracks[1].items[0].node_graph = Some(Box::new(doc_fg_only));

    let plan2 = plan_frame(&p, seq, Tick(1000), RenderOptions::default(), DeviceCaps::ALL, &map);
    match &plan2 {
        FramePlan::Layers { layers, .. } => {
            assert_eq!(layers.len(), 2);
            let ops = &layers[1].fx.as_ref().expect("gpu fx").ops;
            assert!(matches!(
                ops.as_slice(),
                [FxOp::Node(NodeFxOp::ChromaKeyPro { .. }), FxOp::AlphaAdjust { opacity, .. }] if (*opacity - 0.6).abs() < 1e-5
            ));
        }
        _ => panic!("expected GPU FramePlan::Layers for Blend(fg only)"),
    }
}

#[test]
fn transform_small_scale_scale_mod_and_skew_stay_on_gpu() {
    let mut p = Project::new("transform-gpu-upgrade");
    let mut map = SourceMap::default();
    let src = add_matte(&mut p, &mut map, [0.2, 0.6, 0.9, 1.0]);
    let seq = p.new_sequence(
        "s",
        SequenceSettings { width: 32, height: 32, frame_rate: FrameRate::FPS_24, ..Default::default() },
        1,
        1,
        None,
    );
    let r = FrameRate::FPS_24;
    let mut ti = p.make_track_item(src, TrackKind::Video, Tick::ZERO, TimeRange::new(Tick::ZERO, r.tick_of(24)), r).unwrap();

    // 1. Scale = 0.25 (< 0.5) + uniform_scale + skew -> ResampleArea GPU
    let mut doc_small = NodeGraphDoc::new();
    doc_small.nodes = vec![
        NodeEntry { id: 1, node: DarkNode::VideoInput { track_name: "Auto".into() }, pos: [0.0, 0.0], custom_title: None },
        NodeEntry {
            id: 2,
            node: DarkNode::Transform2D {
                position: [2.0, -2.0],
                scale: [0.25, 0.9],
                rotation_deg: 15.0,
                anchor: [0.5, 0.5],
                opacity: 0.9,
                flip_h: false,
                flip_v: false,
                uniform_scale: true,
                skew: 12.0,
                skew_axis: 30.0,
            },
            pos: [120.0, 0.0],
            custom_title: None,
        },
        NodeEntry { id: 3, node: DarkNode::Output, pos: [240.0, 0.0], custom_title: None },
    ];
    doc_small.wires = vec![
        NodeWire { out_node: 1, out_pin: 0, in_node: 2, in_pin: 0 },
        NodeWire { out_node: 2, out_pin: 0, in_node: 3, in_pin: 0 },
    ];
    ti.node_graph = Some(Box::new(doc_small));
    p.sequence_mut(seq).unwrap().video_tracks[0].items.push(ti);

    let plan_small = plan_frame(&p, seq, Tick(1000), RenderOptions::default(), DeviceCaps::ALL, &map);
    match &plan_small {
        FramePlan::Layers { layers, .. } => {
            assert_eq!(layers.len(), 1);
            let ops = &layers[0].fx.as_ref().expect("gpu fx").ops;
            assert!(matches!(
                ops.as_slice(),
                [FxOp::Node(NodeFxOp::ResampleArea { taps, .. })] if *taps >= 4
            ));
        }
        _ => panic!("expected GPU FramePlan::Layers for Transform2D scale < 0.5"),
    }

    // 2. TimeDriver -> Scale Mod (pin 2) cũng phải chạy GPU
    let mut doc_mod = NodeGraphDoc::new();
    doc_mod.nodes = vec![
        NodeEntry { id: 1, node: DarkNode::VideoInput { track_name: "Auto".into() }, pos: [0.0, 0.0], custom_title: None },
        NodeEntry {
            id: 2,
            node: DarkNode::TimeDriver {
                speed: 1.0,
                mode: TimeMode::Linear,
                amplitude: 0.1,
                offset: 0.2,
                vec2_axis: [0.0, 0.0],
            },
            pos: [0.0, 100.0],
            custom_title: None,
        },
        NodeEntry { id: 3, node: DarkNode::default_transform(), pos: [140.0, 0.0], custom_title: None },
        NodeEntry { id: 4, node: DarkNode::Output, pos: [280.0, 0.0], custom_title: None },
    ];
    doc_mod.wires = vec![
        NodeWire { out_node: 1, out_pin: 0, in_node: 3, in_pin: 0 },
        NodeWire { out_node: 2, out_pin: 0, in_node: 3, in_pin: 2 },
        NodeWire { out_node: 3, out_pin: 0, in_node: 4, in_pin: 0 },
    ];
    p.sequence_mut(seq).unwrap().video_tracks[0].items[0].node_graph = Some(Box::new(doc_mod));

    let plan_mod = plan_frame(&p, seq, r.tick_of(12), RenderOptions::default(), DeviceCaps::ALL, &map);
    match &plan_mod {
        FramePlan::Layers { layers, .. } => {
            assert_eq!(layers.len(), 1);
            let ops = &layers[0].fx.as_ref().expect("gpu fx").ops;
            assert!(matches!(ops.as_slice(), [FxOp::Node(NodeFxOp::ResampleArea { .. })]));
        }
        _ => panic!("expected GPU FramePlan::Layers for Transform2D with Scale Mod"),
    }
}
