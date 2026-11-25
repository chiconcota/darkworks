//! Lệnh điều phối Node Graph cho `Session::execute` (`nodeGraph.*`).
//!
//! Kết nối đồ thị DAG (`darkworks_node_graph_core::NodeGraphDoc`) vào các `TrackItem`
//! và Adjustment/Overlay Clips trên Timeline với Undo/Redo đầy đủ.

use darkworks_node_graph_core::{ActionScope, DagEvaluator, NodeGraphDoc};
use filmcraft_project::{ClipId, ItemId, ItemKind, Label, Sequence, TrackKind};
use filmcraft_time::{Tick, TimeRange};
use serde_json::{Value, json};

use crate::commands::{CommandSpec, always, bad, bool_p, f64_p, has_seq, str_p, target_clips_p, u64_p};
use crate::{EngineError, Result, Session};

type Run = fn(&mut Session, &Value) -> Result<Value>;
type Enabled = fn(&Session) -> std::result::Result<(), String>;

fn spec(
    id: &'static str,
    label: &'static str,
    params: &'static str,
    enabled: Enabled,
    run: Run,
) -> CommandSpec {
    CommandSpec {
        id,
        label,
        menu: &[],
        shortcut: None,
        params,
        enabled,
        run,
        journal: true,
        named: crate::commands::NamedKeys::NONE,
    }
}

fn parse_graph(p: &Value, cmd: &str) -> Result<NodeGraphDoc> {
    let raw = p
        .get("graph")
        .ok_or_else(|| bad(cmd, "thiếu tham số `graph`"))?;
    let doc: NodeGraphDoc =
        serde_json::from_value(raw.clone()).map_err(|e| bad(cmd, format!("đồ thị không hợp lệ: {e}")))?;
    DagEvaluator::build_execution_plan(&doc).map_err(|e| bad(cmd, format!("DAG lỗi: {e}")))?;
    Ok(doc)
}

fn resolve_target_clips(s: &Session, p: &Value, cmd: &str) -> Result<Vec<ClipId>> {
    if let Some(ids) = target_clips_p(s, p, cmd)? {
        return Ok(ids);
    }
    if s.state.selection.is_empty() {
        return Err(bad(cmd, "hãy chọn ít nhất một clip trên Timeline"));
    }
    Ok(s.state.selection.clone())
}

fn is_video_clip(seq: &Sequence, clip_id: ClipId) -> bool {
    seq.find_item(clip_id)
        .is_some_and(|(track_id, _)| seq.video_tracks.iter().any(|t| t.id == track_id))
}

fn has_selected_video(s: &Session) -> std::result::Result<(), String> {
    has_seq(s)?;
    let seq = s.active_sequence().ok_or_else(|| "no active sequence".to_string())?;
    if s.state
        .selection
        .iter()
        .any(|&cid| is_video_clip(seq, cid))
    {
        Ok(())
    } else {
        Err("select a video clip on the timeline".into())
    }
}

fn adjustment_source(
    pr: &mut filmcraft_project::Project,
    w: u32,
    h: u32,
    rate: filmcraft_time::FrameRate,
    dur: Tick,
) -> ItemId {
    pr.add_item(
        "Node Graph FX",
        Label::Iris,
        ItemKind::AdjustmentLayer {
            width: w,
            height: h,
            rate,
            duration: dur,
        },
        None,
    )
}

fn place_overlay_with_graph(
    s: &mut Session,
    cmd: &str,
    name: &str,
    start: Tick,
    dur: Tick,
    want_track: Option<usize>,
    doc: NodeGraphDoc,
) -> Result<ClipId> {
    let seq_id = s.state.active_sequence.ok_or(EngineError::NoSequence)?;
    let name = name.to_string();
    s.edit("Add Node Graph Overlay", |pr, st| {
        let (w, h, rate) = {
            let q = pr.sequence(seq_id).ok_or(EngineError::NoSequence)?;
            (q.settings.width, q.settings.height, q.settings.frame_rate)
        };
        let t = rate.snap(start);
        let dur = rate.snap_nearest(dur).max(rate.frame_duration());
        let src = adjustment_source(pr, w, h, rate, dur);
        let mut ti = pr
            .make_track_item(src, TrackKind::Video, t, TimeRange::new(Tick::ZERO, dur), rate)
            .ok_or_else(|| bad(cmd, "không thể tạo track item cho overlay"))?;
        ti.name = name;
        ti.node_graph = Some(Box::new(doc));
        let clip_id = ti.id;

        let track_id = filmcraft_project::TrackId(pr.alloc_id());
        let q = pr.sequence_mut(seq_id).ok_or(EngineError::NoSequence)?;
        let range = TimeRange::new(t, dur);
        let free = |tr: &filmcraft_project::Track| !tr.items.iter().any(|i| i.range().overlaps(&range));
        let idx = match want_track {
            Some(i) if i < q.video_tracks.len() && free(&q.video_tracks[i]) => i,
            Some(i) if i < q.video_tracks.len() => {
                return Err(bad(cmd, format!("Track V{} đang có clip trong khoảng thời gian này", i + 1)));
            }
            _ => {
                let top = q
                    .video_tracks
                    .iter()
                    .rposition(|tr| tr.items.iter().any(|it| it.range().overlaps(&range)))
                    .map_or(0, |k| k + 1);
                match (top..q.video_tracks.len()).find(|&i| free(&q.video_tracks[i]) && !q.video_tracks[i].locked) {
                    Some(i) => i,
                    None => {
                        let n = q.video_tracks.len() + 1;
                        q.video_tracks.push(filmcraft_project::Track::new(
                            track_id,
                            TrackKind::Video,
                            format!("Video {n}"),
                        ));
                        q.video_tracks.len() - 1
                    }
                }
            }
        };

        q.video_tracks[idx].items.push(ti);
        q.video_tracks[idx].sort();
        q.check().map_err(EngineError::Other)?;
        st.selection = vec![clip_id];
        Ok(clip_id)
    })
}

fn set_on_clip(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "nodeGraph.setOnClip";
    let seq_id = s.state.active_sequence.ok_or(EngineError::NoSequence)?;
    let doc = parse_graph(p, cmd)?;
    let targets = resolve_target_clips(s, p, cmd)?;

    let updated = s.edit("Apply Node Graph to Clip", |pr, _| {
        let q = pr.sequence_mut(seq_id).ok_or(EngineError::NoSequence)?;
        let mut count = 0usize;
        for tr in &mut q.video_tracks {
            if tr.locked {
                continue;
            }
            for it in &mut tr.items {
                if targets.contains(&it.id) {
                    it.node_graph = Some(Box::new(doc.clone()));
                    count += 1;
                }
            }
        }
        if count == 0 {
            return Err(bad(cmd, "không có video clip hợp lệ nào được cập nhật (kiểm tra khóa track)"));
        }
        Ok(count)
    })?;

    Ok(json!({ "updated": updated }))
}

fn create_overlay_clip(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "nodeGraph.createOverlayClip";
    let doc = parse_graph(p, cmd)?;
    let name = str_p(p, "name").unwrap_or("Node Graph FX");
    let use_in_out = bool_p(p, "useMarkInOut").unwrap_or(true);
    let want_track = u64_p(p, "track").map(|v| v as usize);

    let seq = s.active_sequence().ok_or(EngineError::NoSequence)?;
    let (start, dur) = if use_in_out
        && let (Some(i), Some(o)) = (seq.mark_in, seq.mark_out)
        && o > i
    {
        (i, o - i)
    } else {
        let secs = f64_p(p, "seconds").unwrap_or(5.0).max(0.05);
        (s.playhead(), Tick::from_seconds_f64(secs))
    };

    let clip_id = place_overlay_with_graph(s, cmd, name, start, dur, want_track, doc)?;
    Ok(json!({ "clip": clip_id.0, "start": start.0, "duration": dur.0 }))
}

fn parse_scope(raw: Option<&str>, cmd: &str) -> Result<ActionScope> {
    match raw.unwrap_or("SelectedClips") {
        "MarkedInOut" => Ok(ActionScope::MarkedInOut),
        "SelectedClips" => Ok(ActionScope::SelectedClips),
        "TrackV1" => Ok(ActionScope::TrackV1),
        "TrackV2" => Ok(ActionScope::TrackV2),
        "AllTracks" => Ok(ActionScope::AllTracks),
        other => Err(bad(cmd, format!("phạm vi không hợp lệ `{other}`"))),
    }
}

fn apply_scope(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "nodeGraph.applyScope";
    let doc = parse_graph(p, cmd)?;
    let scope = parse_scope(str_p(p, "scope"), cmd)?;
    let name = str_p(p, "name").unwrap_or("Action Workflow FX");

    match scope {
        ActionScope::MarkedInOut => {
            let seq = s.active_sequence().ok_or(EngineError::NoSequence)?;
            let (start, dur) = if let (Some(i), Some(o)) = (seq.mark_in, seq.mark_out)
                && o > i
            {
                (i, o - i)
            } else {
                (s.playhead(), Tick::from_seconds_f64(5.0))
            };
            let clip_id = place_overlay_with_graph(s, cmd, name, start, dur, None, doc)?;
            Ok(json!({ "mode": "overlay", "clip": clip_id.0 }))
        }
        ActionScope::SelectedClips => {
            let updated = set_on_clip(s, p)?;
            Ok(updated)
        }
        ActionScope::TrackV1 | ActionScope::TrackV2 | ActionScope::AllTracks => {
            let seq_id = s.state.active_sequence.ok_or(EngineError::NoSequence)?;
            let updated = s.edit("Apply Node Graph Workflow", |pr, _| {
                let q = pr.sequence_mut(seq_id).ok_or(EngineError::NoSequence)?;
                let mut count = 0usize;
                for (idx, tr) in q.video_tracks.iter_mut().enumerate() {
                    if tr.locked {
                        continue;
                    }
                    let match_track = match scope {
                        ActionScope::TrackV1 => idx == 0,
                        ActionScope::TrackV2 => idx == 1,
                        ActionScope::AllTracks => true,
                        _ => false,
                    };
                    if !match_track {
                        continue;
                    }
                    for it in &mut tr.items {
                        it.node_graph = Some(Box::new(doc.clone()));
                        count += 1;
                    }
                }
                if count == 0 {
                    return Err(bad(cmd, "không tìm thấy video clip nào trong phạm vi track đã chọn"));
                }
                Ok(count)
            })?;
            Ok(json!({ "mode": "clips", "updated": updated }))
        }
    }
}

fn clear_from_clip(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "nodeGraph.clearFromClip";
    let seq_id = s.state.active_sequence.ok_or(EngineError::NoSequence)?;
    let targets = resolve_target_clips(s, p, cmd)?;

    let cleared = s.edit("Clear Node Graph from Clip", |pr, _| {
        let q = pr.sequence_mut(seq_id).ok_or(EngineError::NoSequence)?;
        let mut count = 0usize;
        for tr in &mut q.video_tracks {
            if tr.locked {
                continue;
            }
            for it in &mut tr.items {
                if targets.contains(&it.id) && it.node_graph.take().is_some() {
                    count += 1;
                }
            }
        }
        Ok(count)
    })?;

    Ok(json!({ "cleared": cleared }))
}

pub fn commands() -> Vec<CommandSpec> {
    vec![
        spec(
            "nodeGraph.setOnClip",
            "Apply Node Graph to Clip",
            r#"{"clip":id?,"clips":[id]?,"graph":object}"#,
            has_selected_video,
            set_on_clip,
        ),
        spec(
            "nodeGraph.createOverlayClip",
            "Create Node Graph Overlay Clip",
            r#"{"name":str?,"seconds":f64?,"useMarkInOut":bool?,"track":u64?,"graph":object}"#,
            has_seq,
            create_overlay_clip,
        ),
        spec(
            "nodeGraph.applyScope",
            "Run Node Graph Workflow",
            r#"{"scope":"MarkedInOut"|"SelectedClips"|"TrackV1"|"TrackV2"|"AllTracks"?,"name":str?,"clip":id?,"clips":[id]?,"graph":object}"#,
            has_seq,
            apply_scope,
        ),
        spec(
            "nodeGraph.clearFromClip",
            "Clear Node Graph from Clip",
            r#"{"clip":id?,"clips":[id]?}"#,
            has_selected_video,
            clear_from_clip,
        ),
        CommandSpec {
            id: "nodeGraph.getFromClip",
            label: "Inspect Clip Node Graph",
            menu: &[],
            shortcut: None,
            params: r#"{"clip":id?}"#,
            enabled: always,
            run: |s, p| {
                let seq = s.active_sequence().ok_or(EngineError::NoSequence)?;
                let cid = u64_p(p, "clip")
                    .map(ClipId)
                    .or_else(|| s.state.selection.first().copied());
                let graph = cid
                    .and_then(|id| seq.find_item(id))
                    .and_then(|(_, it)| it.node_graph.as_deref().cloned());
                Ok(json!({ "clip": cid.map(|c| c.0), "graph": graph }))
            },
            journal: false,
            named: crate::commands::NamedKeys::NONE,
        },
    ]
}

#[cfg(test)]
mod tests {
    use darkworks_node_graph_core::{DarkNode, NodeEntry, NodeGraphDoc, NodeWire, ShapeType};
    use serde_json::json;

    use crate::Session;

    fn sample_doc() -> NodeGraphDoc {
        NodeGraphDoc {
            nodes: vec![
                NodeEntry {
                    id: 0,
                    pos: [0.0, 0.0],
                    node: DarkNode::ShapeGenerator {
                        shape_type: ShapeType::SolidMatte,
                        size: [1920.0, 1080.0],
                        corner_radius: 0.0,
                        color: [0.1, 0.8, 0.3, 1.0],
                    },
                    custom_title: None,
                },
                NodeEntry {
                    id: 1,
                    pos: [200.0, 0.0],
                    node: DarkNode::Output,
                    custom_title: None,
                },
            ],
            wires: vec![NodeWire {
                out_node: 0,
                out_pin: 0,
                in_node: 1,
                in_pin: 0,
            }],
            groups: Vec::new(),
        }
    }

    #[test]
    fn node_graph_commands_attach_overlay_undo_and_roundtrip_json() {
        let mut s = Session::default();
        s.execute("file.openDemoProject", json!({})).expect("open demo");

        let graph = serde_json::to_value(sample_doc()).expect("serialize doc");
        let res = s
            .execute(
                "nodeGraph.createOverlayClip",
                json!({ "name": "Test Overlay", "seconds": 3.0, "useMarkInOut": false, "graph": graph }),
            )
            .expect("create overlay");
        let clip_id = res["clip"].as_u64().expect("clip id");

        let seq_id = s.state.active_sequence.expect("active seq");
        let seq = s.active_sequence().expect("seq");
        let (_, item) = seq.find_item(filmcraft_project::ClipId(clip_id)).expect("overlay item");
        assert!(item.node_graph.is_some());

        // Round-trip project JSON
        let saved = s.project.to_json();
        let reloaded = filmcraft_project::Project::from_json(&saved).expect("reload project");
        let rel_seq = reloaded.sequence(seq_id).expect("seq");
        let (_, rel_item) = rel_seq.find_item(filmcraft_project::ClipId(clip_id)).expect("rel item");
        assert_eq!(rel_item.node_graph, item.node_graph);

        // Undo phải xóa clip overlay vừa tạo
        s.execute("edit.undo", json!({})).expect("undo");
        let seq_after = s.active_sequence().expect("seq");
        assert!(seq_after.find_item(filmcraft_project::ClipId(clip_id)).is_none());
    }
}
