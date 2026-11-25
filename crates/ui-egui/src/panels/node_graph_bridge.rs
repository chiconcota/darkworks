//! Cầu nối đồng bộ giữa Node Graph (`Snarl<DarkNode>`), Timeline (`TrackItem`) và Program Monitor.

use darkworks_node_graph::{DarkNode, doc_from_snarl, snarl_from_doc};
use darkworks_node_graph_core::ActionScope;
use egui::{Color32, RichText, Ui};
use filmcraft_engine::project::ClipId;
use serde_json::json;

use super::node_graph::NodeGraphState;
use crate::FilmcraftApp;

fn selected_video_clip(app: &FilmcraftApp) -> Option<(ClipId, String, Option<darkworks_node_graph_core::NodeGraphDoc>)> {
    let seq = app.session.active_sequence()?;
    let cid = app.session.state.selection.iter().copied().find(|&id| {
        seq.find_item(id)
            .is_some_and(|(tid, _)| seq.video_tracks.iter().any(|t| t.id == tid))
    })?;
    let (_, item) = seq.find_item(cid)?;
    Some((cid, item.name.clone(), item.node_graph.as_deref().cloned()))
}

/// Tự động nạp Node Graph khi người dùng chuyển chọn sang clip có gắn Node Graph trên Timeline.
pub fn sync_selection_from_timeline(app: &FilmcraftApp, state: &mut NodeGraphState) {
    let sel = selected_video_clip(app);
    let sel_id = sel.as_ref().map(|(id, _, _)| *id);

    if sel_id == state.last_observed_selection {
        // Nếu đang bound vào một clip nhưng clip đó bị xóa khỏi Timeline -> gỡ binding
        if let Some(bound_id) = state.bound_clip_id
            && app
                .session
                .active_sequence()
                .is_none_or(|seq| seq.find_item(bound_id).is_none())
        {
            state.bound_clip_id = None;
            state.bound_clip_name.clear();
            state.last_synced_doc = None;
        }
        return;
    }

    state.last_observed_selection = sel_id;
    if let Some((cid, name, Some(doc))) = sel {
        let (snarl, groups, custom_titles) = snarl_from_doc(&doc);
        state.snarl = snarl;
        state.groups = groups;
        state.custom_titles = custom_titles;
        state.bound_clip_id = Some(cid);
        state.bound_clip_name = name.clone();
        state.last_synced_doc = Some(doc);
        state.selected_group = None;
        state.last_hovered_group = None;
        state.history.clear();
        state.status_message = format!("Đang sửa Node Graph của clip: {name}");
    } else {
        state.bound_clip_id = None;
        state.bound_clip_name.clear();
        state.last_synced_doc = None;
    }
}

/// Thanh nút thao tác áp dụng Node Graph xuống Timeline & Program Monitor.
pub fn render_timeline_actions(app: &mut FilmcraftApp, ui: &mut Ui, state: &mut NodeGraphState) {
    let sel_video = selected_video_clip(app);
    let has_seq = app.session.active_sequence().is_some();

    ui.separator();
    if let Some(bound_id) = state.bound_clip_id {
        ui.label(
            RichText::new(format!("🔗 Clip: {}", state.bound_clip_name))
                .strong()
                .color(Color32::from_rgb(0, 230, 140)),
        );
        if ui
            .button("📝 New Graph (Nháp)")
            .on_hover_text("Tách sang chế độ Nháp để tạo FX Clip mới mà vẫn giữ nguyên Node Graph của clip này")
            .clicked()
        {
            state.detach_to_scratchpad();
            state.status_message = "Đã tách sang chế độ Nháp để soạn FX Clip mới".to_string();
        }
        if ui
            .button("🗑 Gỡ khỏi Clip")
            .on_hover_text("Xóa hẳn Node Graph khỏi clip đang liên kết trên Timeline")
            .clicked()
        {
            match app.session.execute("nodeGraph.clearFromClip", json!({ "clip": bound_id.0 })) {
                Ok(_) => {
                    state.detach_to_scratchpad();
                    state.status_message = "Đã xóa Node Graph khỏi clip".to_string();
                }
                Err(e) => {
                    state.status_message = format!("✖ {e}");
                }
            }
        }
    } else {
        ui.label(RichText::new("📝 Nháp (Scratchpad)").weak());
    }

    let apply_enabled = sel_video.is_some();
    if ui
        .add_enabled(apply_enabled, egui::Button::new("📌 Apply to Clip"))
        .on_hover_text("Áp dụng Node Graph hiện tại vào video clip đang chọn trên Timeline")
        .clicked()
        && let Some((cid, name, _)) = sel_video
    {
        apply_graph_to_clip(app, state, cid, &name);
    }

    if ui
        .add_enabled(has_seq, egui::Button::new("🎬 Add as FX Clip"))
        .on_hover_text("Tạo Overlay/FX Clip mới tại vùng [In, Out] hoặc Playhead mang Node Graph này")
        .clicked()
    {
        create_overlay_clip(app, state);
    }
}

fn apply_graph_to_clip(
    app: &mut FilmcraftApp,
    state: &mut NodeGraphState,
    cid: ClipId,
    name: &str,
) {
    let doc = doc_from_snarl(&state.snarl, &state.groups, &state.custom_titles);
    let Ok(graph_json) = serde_json::to_value(&doc) else {
        state.status_message = "✖ Không thể tuần tự hóa Node Graph".to_string();
        return;
    };
    match app.session.execute("nodeGraph.setOnClip", json!({ "clip": cid.0, "graph": graph_json })) {
        Ok(_) => {
            state.bound_clip_id = Some(cid);
            state.bound_clip_name = name.to_string();
            state.last_synced_doc = Some(doc);
            state.status_message = format!("✔ Đã áp dụng Node Graph xuống clip '{name}'");
        }
        Err(e) => {
            state.status_message = format!("✖ {e}");
        }
    }
}

fn create_overlay_clip(app: &mut FilmcraftApp, state: &mut NodeGraphState) {
    let doc = doc_from_snarl(&state.snarl, &state.groups, &state.custom_titles);
    let Ok(graph_json) = serde_json::to_value(&doc) else {
        state.status_message = "✖ Không thể tuần tự hóa Node Graph".to_string();
        return;
    };
    let clip_name = state
        .groups
        .first()
        .map(|g| g.title.clone())
        .filter(|t| !t.trim().is_empty())
        .unwrap_or_else(|| "Node Graph FX".to_string());

    match app.session.execute(
        "nodeGraph.createOverlayClip",
        json!({ "name": clip_name, "useMarkInOut": true, "seconds": 5.0, "graph": graph_json }),
    ) {
        Ok(res) => {
            if let Some(cid) = res.get("clip").and_then(|v| v.as_u64()).map(ClipId) {
                state.bound_clip_id = Some(cid);
                state.bound_clip_name = clip_name.clone();
                state.last_observed_selection = Some(cid);
                state.last_synced_doc = Some(doc);
            }
            state.status_message = format!("✔ Đã tạo Overlay FX Clip '{clip_name}' trên Timeline");
        }
        Err(e) => {
            state.status_message = format!("✖ {e}");
        }
    }
}

fn scope_str(scope: ActionScope) -> &'static str {
    match scope {
        ActionScope::MarkedInOut => "MarkedInOut",
        ActionScope::SelectedClips => "SelectedClips",
        ActionScope::TrackV1 => "TrackV1",
        ActionScope::TrackV2 => "TrackV2",
        ActionScope::AllTracks => "AllTracks",
    }
}

/// Xử lý nút `[ ▶ RUN WORKFLOW ]` trên `ActionNode` và tự động đồng bộ chỉnh sửa live xuống Clip đang gắn.
pub fn process_action_nodes_and_live_sync(app: &mut FilmcraftApp, ui: &Ui, state: &mut NodeGraphState) {
    // 1. Kích hoạt ActionNode nếu vừa bấm [ ▶ RUN WORKFLOW ]
    let mut triggered_action: Option<(ActionScope, String)> = None;
    for (_, node) in state.snarl.nodes_ids_data_mut() {
        if let DarkNode::ActionNode {
            name,
            scope,
            is_running,
        } = &mut node.value
            && *is_running
        {
            *is_running = false;
            triggered_action = Some((*scope, name.clone()));
            break;
        }
    }
    if let Some((scope, name)) = triggered_action {
        let doc = doc_from_snarl(&state.snarl, &state.groups, &state.custom_titles);
        if let Ok(graph_json) = serde_json::to_value(&doc) {
            match app.session.execute(
                "nodeGraph.applyScope",
                json!({ "scope": scope_str(scope), "name": name, "graph": graph_json }),
            ) {
                Ok(res) => {
                    if let Some(cid) = res.get("clip").and_then(|v| v.as_u64()).map(ClipId) {
                        state.bound_clip_id = Some(cid);
                        state.bound_clip_name = name.clone();
                        state.last_observed_selection = Some(cid);
                        state.last_synced_doc = Some(doc);
                    }
                    state.status_message = format!("✔ ActionNode '{name}' đã chạy cho phạm vi {scope:?}");
                }
                Err(e) => {
                    state.status_message = format!("✖ ActionNode lỗi: {e}");
                }
            }
        }
    }

    // 2. Live sync khi chỉnh sửa đồ thị đang gắn với 1 Clip (chỉ đẩy khi nhả chuột để không spam Undo)
    let pointer_down = ui.input(|i| i.pointer.any_down());
    if !pointer_down
        && let Some(bound_id) = state.bound_clip_id
    {
        let current_doc = doc_from_snarl(&state.snarl, &state.groups, &state.custom_titles);
        if state.last_synced_doc.as_ref() != Some(&current_doc)
            && let Ok(graph_json) = serde_json::to_value(&current_doc)
            && app
                .session
                .execute("nodeGraph.setOnClip", json!({ "clip": bound_id.0, "graph": graph_json }))
                .is_ok()
        {
            state.last_synced_doc = Some(current_doc);
        }
    }
}
