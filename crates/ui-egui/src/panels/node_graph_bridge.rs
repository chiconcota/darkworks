//! Cầu nối đồng bộ giữa Node Graph (`Snarl<DarkNode>`), Timeline (`TrackItem`) và Program Monitor.

use darkworks_node_graph::node_controls::{
    NodeEyedropperActive, NodeEyedropperRequest, eyedropper_active_id, eyedropper_request_id,
};
use darkworks_node_graph::{DarkNode, NodeDataType, doc_from_snarl, snarl_from_doc};
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

/// Đồng bộ trạng thái nút Eyedropper đang bật xuống `egui::Context` trước khi vẽ Canvas Node.
pub fn publish_eyedropper_active(app: &FilmcraftApp, ctx: &egui::Context) {
    if let Some(crate::state::Eyedropper::NodeGraph { node_idx, param, .. }) = &app.ui.eyedropper {
        ctx.data_mut(|d| {
            d.insert_temp(
                eyedropper_active_id(),
                NodeEyedropperActive {
                    node_id: egui_snarl::NodeId(*node_idx),
                    param: param.clone(),
                },
            );
        });
    } else {
        ctx.data_mut(|d| {
            d.remove::<NodeEyedropperActive>(eyedropper_active_id());
        });
    }
}

/// Tiếp nhận cú click vào nút Eyedropper trên bất kỳ Node có bảng màu nào.
pub fn consume_eyedropper_request(app: &mut FilmcraftApp, ctx: &egui::Context, state: &mut NodeGraphState) {
    let req = ctx.data_mut(|d| {
        let r = d.get_temp::<NodeEyedropperRequest>(eyedropper_request_id());
        if r.is_some() {
            d.remove::<NodeEyedropperRequest>(eyedropper_request_id());
        }
        r
    });
    let Some(req) = req else { return };
    let clip = state.bound_clip_id.map(|c| c.0);
    let already = matches!(
        &app.ui.eyedropper,
        Some(crate::state::Eyedropper::NodeGraph { node_idx, param, .. })
            if *node_idx == req.node_id.0 && param == req.param
    );
    if already {
        app.ui.eyedropper = None;
        state.status_message = "Đã tắt Eyedropper".to_string();
    } else {
        let bypass_node = req.param == "keyer.key_color";
        app.ui.eyedropper = Some(crate::state::Eyedropper::NodeGraph {
            clip,
            node_idx: req.node_id.0,
            param: req.param.to_string(),
            bypass_node,
        });
        state.status_message = "🎯 Eyedropper đã bật: click vào Program Monitor để hút màu (Esc hủy)".to_string();
    }
}

/// Tạm thay thế node `Keyer` bằng `Reroute` trên bản sao `Project` để lấy mẫu đúng màu phông gốc.
pub fn bypass_node_in_project(
    ctx: &egui::Context,
    seq: filmcraft_project::ItemId,
    clip_id: u64,
    node_idx: usize,
    project: &mut filmcraft_project::Project,
) {
    crate::panels::node_graph::with_state(ctx, |state| {
        let mut temp_snarl = state.snarl.clone();
        if let Some(n) = temp_snarl.get_node_mut(egui_snarl::NodeId(node_idx)) {
            *n = DarkNode::Reroute {
                data_type: NodeDataType::Texture,
            };
        }
        let doc = doc_from_snarl(&temp_snarl, &state.groups, &state.custom_titles);
        if let Some((_, item)) = project.sequence_mut(seq).and_then(|s| s.find_item_mut(ClipId(clip_id))) {
            item.node_graph = Some(Box::new(doc));
        }
    });
}

/// Áp màu hút được từ Program Monitor vào đúng tham số của Node (cả Clip gắn sẵn lẫn Nháp).
pub fn apply_eyedropper_color(
    ctx: &egui::Context,
    app: &mut FilmcraftApp,
    clip: Option<u64>,
    node_idx: usize,
    param: &str,
    rgb: [u8; 3],
) {
    let c = [
        f32::from(rgb[0]) / 255.0,
        f32::from(rgb[1]) / 255.0,
        f32::from(rgb[2]) / 255.0,
    ];
    let mut updated_doc: Option<(ClipId, darkworks_node_graph_core::NodeGraphDoc)> = None;
    crate::panels::node_graph::with_state(ctx, |state| {
        state.push_undo();
        if let Some(node) = state.snarl.get_node_mut(egui_snarl::NodeId(node_idx)) {
            match (node, param) {
                (DarkNode::Keyer { key_color, .. }, "keyer.key_color") => *key_color = c,
                (DarkNode::ColorGrade { tint_color, .. }, "color.tint_color") => {
                    tint_color[0] = c[0];
                    tint_color[1] = c[1];
                    tint_color[2] = c[2];
                }
                (DarkNode::ColorGrade { shadow_tint, .. }, "color.shadow_tint") => *shadow_tint = c,
                (DarkNode::ColorGrade { highlight_tint, .. }, "color.highlight_tint") => *highlight_tint = c,
                (DarkNode::TextGenerator { color, .. }, "text.color") => {
                    color[0] = c[0];
                    color[1] = c[1];
                    color[2] = c[2];
                }
                (DarkNode::TextGenerator { stroke_color, .. }, "text.stroke_color") => {
                    stroke_color[0] = c[0];
                    stroke_color[1] = c[1];
                    stroke_color[2] = c[2];
                }
                (DarkNode::ShapeGenerator { color, .. }, "shape.color") => {
                    color[0] = c[0];
                    color[1] = c[1];
                    color[2] = c[2];
                }
                (DarkNode::ShapeGenerator { stroke_color, .. }, "shape.stroke_color") => {
                    stroke_color[0] = c[0];
                    stroke_color[1] = c[1];
                    stroke_color[2] = c[2];
                }
                _ => {}
            }
            state.status_message = format!(
                "✔ Đã hút màu #{:02X}{:02X}{:02X} vào {param}",
                rgb[0], rgb[1], rgb[2]
            );
            if let Some(cid) = clip
                && state.bound_clip_id == Some(ClipId(cid))
            {
                let doc = doc_from_snarl(&state.snarl, &state.groups, &state.custom_titles);
                updated_doc = Some((ClipId(cid), doc));
            }
        }
    });
    if let Some((cid, doc)) = updated_doc
        && let Ok(graph_json) = serde_json::to_value(&doc)
        && app
            .session
            .execute("nodeGraph.setOnClip", json!({ "clip": cid.0, "graph": graph_json }))
            .is_ok()
    {
        crate::panels::node_graph::with_state(ctx, |state| {
            state.last_synced_doc = Some(doc);
        });
    }
}
