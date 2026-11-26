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

#[test]
fn audio_reactor_a2_bass_drives_color_grade_exposure_visibly() {
    use std::sync::Arc;
    use darkworks_node_graph_core::AudioBand;
    use filmcraft_frame::{AudioBuffer, VideoFrame};
    use filmcraft_media::{AudioStreamInfo, MediaError, MediaInfo, MediaKind, MediaSource};
    use filmcraft_project::{MediaClip, MediaRef};

    struct BassSrc(MediaInfo);
    impl MediaSource for BassSrc {
        fn info(&self) -> &MediaInfo { &self.0 }
        fn video_frame(&self, _: filmcraft_media::FrameRequest) -> filmcraft_media::Result<Arc<VideoFrame>> {
            Err(MediaError::NoStream("video"))
        }
        fn audio(&self, start: i64, frames: usize, sr: u32) -> filmcraft_media::Result<AudioBuffer> {
            let ch: Vec<f32> = (0..frames).map(|i| {
                let t_sec = (start + i as i64) as f32 / sr.max(1) as f32;
                if (0.5..=2.0).contains(&t_sec) { 0.6 * (std::f32::consts::TAU * 80.0 * t_sec).sin() } else { 0.0 }
            }).collect();
            Ok(AudioBuffer { sample_rate: sr, channels: vec![ch.clone(), ch] })
        }
    }

    let mut p = Project::new("audio_test");
    let rate = FrameRate::FPS_24;
    let dur = Tick::from_seconds_f64(4.0);
    let seq_id = p.new_sequence("s", SequenceSettings { width: 16, height: 16, ..Default::default() }, 1, 2, None);

    let a2_info = MediaInfo {
        name: "bass.mp3".into(),
        kind: MediaKind::AudioOnly,
        duration: dur,
        video: None,
        audio_streams: vec![AudioStreamInfo { sample_rate: 48_000, channels: 2, codec: "MP3".into(), bits_per_sample: None }],
        container: "MP3".into(),
        start_timecode: None,
        file_size: None,
    };
    let a2_item = p.add_item("bass.mp3", Label::Green, ItemKind::Media(Box::new(MediaClip {
        media: MediaRef::File { path: "bass.mp3".into() },
        info: a2_info.clone(),
        interpret: Default::default(),
        mark_in: None, mark_out: None, markers: vec![], offline: false, proxy: None, identity: None,
    })), None);
    let a2_ti = p.make_track_item(a2_item, TrackKind::Audio, Tick::ZERO, TimeRange::new(Tick::ZERO, dur), rate).expect("a2");
    p.sequence_mut(seq_id).expect("seq").audio_tracks[1].items.push(a2_ti);

    let mut video_ti: TrackItem = serde_json::from_value(serde_json::json!({
        "id": 77, "item": 1, "name": "v1", "label": "Iris", "start": 0, "duration": dur.0, "source_in": 0,
        "speed": 1.0, "enabled": true, "effects": []
    })).expect("video_ti");
    let mut audio_node = DarkNode::default_audio_reactor(AudioBand::Bass, 0.80, 0.20);
    if let DarkNode::AudioReactor { track_name, .. } = &mut audio_node { *track_name = "A2".into(); }
    let mut cg_node = DarkNode::default_color_grade();
    if let DarkNode::ColorGrade { exposure, .. } = &mut cg_node { *exposure = 0.15; }

    video_ti.node_graph = Some(Box::new(NodeGraphDoc {
        nodes: vec![
            NodeEntry { id: 0, pos: [0.0, 0.0], node: DarkNode::VideoInput { track_name: "V1".into() }, custom_title: None },
            NodeEntry { id: 1, pos: [0.0, 200.0], node: audio_node, custom_title: None },
            NodeEntry { id: 2, pos: [250.0, 0.0], node: cg_node, custom_title: None },
            NodeEntry { id: 3, pos: [500.0, 0.0], node: DarkNode::Output, custom_title: None },
        ],
        wires: vec![
            NodeWire { out_node: 0, out_pin: 0, in_node: 2, in_pin: 0 },
            NodeWire { out_node: 1, out_pin: 0, in_node: 2, in_pin: 1 },
            NodeWire { out_node: 2, out_pin: 0, in_node: 3, in_pin: 0 },
        ],
        groups: Vec::new(),
    }));

    let src: filmcraft_media::SharedSource = Arc::new(BassSrc(a2_info));
    let provider = |id: filmcraft_project::ItemId| (id == a2_item).then(|| src.clone());
    let clip_img = Image::filled(16, 16, [0.1, 0.1, 0.1, 1.0]);
    p.sequence_mut(seq_id).expect("seq").video_tracks[0].items.push(video_ti.clone());
    let seq = p.sequence(seq_id).expect("seq");

    let beat_luma = eval_item_graph(&p, seq, &video_ti, Tick::from_seconds_f64(1.0), RenderOptions::default(), &provider, Some(&clip_img), None).expect("beat").get(8, 8)[0];
    let silence_luma = eval_item_graph(&p, seq, &video_ti, Tick::from_seconds_f64(3.0), RenderOptions::default(), &provider, Some(&clip_img), None).expect("silence").get(8, 8)[0];
    assert!(beat_luma > silence_luma * 2.0, "bass beat ({beat_luma}) must brighten video > 2x vs silence ({silence_luma})");
}
