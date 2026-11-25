//! Panel Node Graph (DAG) cho Darkworks.
//!
//! Hiển thị canvas đồ thị `egui_snarl`, thanh công cụ thêm 18 Atomic Nodes đầy đủ,
//! nạp 5 Built-in Presets mẫu, quản lý Preset JSON, và các công cụ tổ chức kiểu ComfyUI.

use std::collections::HashMap;
use darkworks_node_graph::{
    BuiltinPresetKind, DarkNode, DarkworksViewer, GraphHistory, GraphSnapshot, NodeGraphPreset,
    VisualGroup, ZoomAction,
};
use egui::{vec2, Color32, Pos2, Rect, Ui, Vec2};
use egui_snarl::{
    ui::{BackgroundPattern, SnarlStyle},
    Snarl,
};
use std::sync::{Arc, Mutex};

use crate::panels::node_graph_bridge;
use crate::panels::node_graph_groups::{self, GroupMenuAction};
use crate::panels::node_graph_menus;
use crate::FilmcraftApp;

/// Trạng thái lưu trữ của Node Graph Panel trong RAM.
#[derive(Clone)]
pub struct NodeGraphState {
    pub snarl: Snarl<DarkNode>,
    pub groups: Vec<VisualGroup>,
    pub status_message: String,
    pub zoom_action: Option<ZoomAction>,
    pub current_zoom: f32,
    pub current_to_global: egui::emath::TSTransform,
    pub custom_titles: HashMap<egui_snarl::NodeId, String>,
    pub renaming_node: Option<egui_snarl::NodeId>,
    pub renaming_buf: String,
    pub renaming_group: Option<usize>,
    pub renaming_group_buf: String,
    pub last_hovered_group: Option<usize>,
    pub selected_group: Option<usize>,
    pub new_group_title: String,
    pub show_groups_dialog: bool,
    pub is_dragging_group: bool,
    pub last_snarl_id: Option<egui::Id>,
    pub history: GraphHistory,
    pub bound_clip_id: Option<filmcraft_engine::project::ClipId>,
    pub bound_clip_name: String,
    pub last_synced_doc: Option<darkworks_node_graph_core::NodeGraphDoc>,
    pub last_observed_selection: Option<filmcraft_engine::project::ClipId>,
}

impl NodeGraphState {
    pub fn snapshot(&self) -> GraphSnapshot {
        GraphSnapshot {
            snarl: self.snarl.clone(),
            groups: self.groups.clone(),
            custom_titles: self.custom_titles.clone(),
        }
    }

    pub fn push_undo(&mut self) {
        let snap = self.snapshot();
        self.history.push_snapshot(snap);
    }

    pub fn restore_snapshot(&mut self, snap: GraphSnapshot) {
        self.snarl = snap.snarl;
        self.groups = snap.groups;
        self.custom_titles = snap.custom_titles;
        self.selected_group = None;
        self.last_hovered_group = None;
    }

    /// Tách editor khỏi Clip đang gắn sang chế độ Nháp mà không làm mất Node Graph trên Clip cũ.
    pub fn detach_to_scratchpad(&mut self) {
        self.bound_clip_id = None;
        self.bound_clip_name.clear();
        self.last_synced_doc = None;
    }
}

impl Default for NodeGraphState {
    fn default() -> Self {
        let (snarl, groups) = NodeGraphPreset::load_builtin(BuiltinPresetKind::NewsTickerBanner);
        Self {
            snarl,
            groups,
            status_message: "Đã nạp Preset mặc định: News Ticker".to_string(),
            zoom_action: None,
            current_zoom: 1.0,
            current_to_global: egui::emath::TSTransform::IDENTITY,
            custom_titles: HashMap::new(),
            renaming_node: None,
            renaming_buf: String::new(),
            renaming_group: None,
            renaming_group_buf: String::new(),
            last_hovered_group: None,
            selected_group: None,
            new_group_title: String::new(),
            show_groups_dialog: false,
            is_dragging_group: false,
            last_snarl_id: None,
            history: GraphHistory::new(50),
            bound_clip_id: None,
            bound_clip_name: String::new(),
            last_synced_doc: None,
            last_observed_selection: None,
        }
    }
}

pub const NODE_GRAPH_STATE_ID: &str = "darkworks-nodegraph-state";
pub const SNARL_CANVAS_ID: &str = "darkworks-snarl-canvas-fixed-id";

pub fn state_id() -> egui::Id {
    egui::Id::unique(NODE_GRAPH_STATE_ID)
}

pub fn snarl_canvas_id() -> egui::Id {
    egui::Id::unique(SNARL_CANVAS_ID)
}

/// Lấy hoặc khởi tạo NodeGraphState từ egui memory
fn get_or_init_state(ctx: &egui::Context) -> Arc<Mutex<NodeGraphState>> {
    let id = state_id();
    ctx.data_mut(|d| {
        d.get_temp_mut_or_insert_with(id, || Arc::new(Mutex::new(NodeGraphState::default())))
            .clone()
    })
}

/// Xóa các Node đang được chọn hoặc Group đang trỏ chuột (được gọi từ bộ phím tắt toàn cục hoặc phím Delete)
pub fn delete_selected_or_hovered(ctx: &egui::Context) -> bool {
    let state_arc = ctx.data(|d| d.get_temp::<Arc<Mutex<NodeGraphState>>>(state_id()));
    let Some(state_arc) = state_arc else { return false; };
    let Ok(mut state) = state_arc.try_lock() else { return false; };

    if state.renaming_group.is_some() || state.renaming_node.is_some() || ctx.egui_wants_keyboard_input() {
        return false;
    }

    let snarl_id = state.last_snarl_id.unwrap_or_else(snarl_canvas_id);
    let selected_nodes = egui_snarl::ui::get_selected_nodes(snarl_id, ctx);
    if !selected_nodes.is_empty() {
        state.push_undo();
        let count = selected_nodes.len();
        for &node_id in &selected_nodes {
            state.snarl.remove_node(node_id);
            state.custom_titles.remove(&node_id);
        }
        egui_snarl::ui::deselect_all_nodes(snarl_id, ctx);
        state.status_message = format!("Đã xóa {count} node được chọn (Delete)");
        ctx.request_repaint();
        true
    } else if let Some(target_idx) = state.selected_group.or(state.last_hovered_group) {
        state.push_undo();
        if let Some(msg) = node_graph_groups::ungroup_at_index(target_idx, &mut state) {
            state.status_message = format!("{msg} (Delete)");
        }
        state.selected_group = None;
        state.last_hovered_group = None;
        ctx.request_repaint();
        true
    } else {
        false
    }
}

/// Kiểm tra xem Node Graph có đang chọn node nào hoặc hover group nào không
pub fn has_selection(ctx: &egui::Context) -> bool {
    let state_arc = ctx.data(|d| d.get_temp::<Arc<Mutex<NodeGraphState>>>(state_id()));
    let Some(state_arc) = state_arc else { return false; };
    let Ok(state) = state_arc.try_lock() else { return false; };
    let snarl_id = state.last_snarl_id.unwrap_or_else(snarl_canvas_id);
    !egui_snarl::ui::get_selected_nodes(snarl_id, ctx).is_empty()
        || state.selected_group.is_some()
        || state.last_hovered_group.is_some()
}

/// Hoàn tác một bước thao tác trên Node Graph (Ctrl+Z)
pub fn undo(ctx: &egui::Context) -> bool {
    let state_arc = ctx.data(|d| d.get_temp::<Arc<Mutex<NodeGraphState>>>(state_id()));
    let Some(state_arc) = state_arc else { return false; };
    let Ok(mut state) = state_arc.try_lock() else { return false; };
    let current = state.snapshot();
    if let Some(prev) = state.history.undo(current) {
        state.restore_snapshot(prev);
        state.status_message = "Đã hoàn tác (Undo: Ctrl+Z)".to_string();
        ctx.request_repaint();
        true
    } else {
        false
    }
}

/// Làm lại một bước thao tác trên Node Graph (Ctrl+Y / Ctrl+Shift+Z)
pub fn redo(ctx: &egui::Context) -> bool {
    let state_arc = ctx.data(|d| d.get_temp::<Arc<Mutex<NodeGraphState>>>(state_id()));
    let Some(state_arc) = state_arc else { return false; };
    let Ok(mut state) = state_arc.try_lock() else { return false; };
    let current = state.snapshot();
    if let Some(next) = state.history.redo(current) {
        state.restore_snapshot(next);
        state.status_message = "Đã làm lại (Redo: Ctrl+Y)".to_string();
        ctx.request_repaint();
        true
    } else {
        false
    }
}

/// Hiển thị Node Graph Panel trong dock của Darkworks.
pub fn show(app: &mut FilmcraftApp, ui: &mut Ui, rect: Rect) {
    if ui.rect_contains_pointer(rect) && ui.input(|i| i.pointer.any_pressed() || i.pointer.any_down()) {
        app.ui.focused = crate::dock::PanelKind::NodeGraph;
    }
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect));
    let ui = &mut child;

    let state_arc = get_or_init_state(ui.ctx());
    let Ok(mut state_guard) = state_arc.lock() else {
        ui.label("Lỗi khóa bộ nhớ Node Graph");
        return;
    };
    let state = &mut *state_guard;
    node_graph_bridge::sync_selection_from_timeline(app, state);

    // 1. THANH CÔNG CỤ TRÊN CÙNG (TOOLBAR)
    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new("🌟 Node Graph").strong().color(Color32::from_rgb(0, 210, 255)));
        node_graph_bridge::render_timeline_actions(app, ui, state);
        ui.separator();

        // Menu Thêm Node & Load Preset
        node_graph_menus::render_add_node_menu(ui, state);
        node_graph_menus::render_load_preset_menu(ui, state);

        let can_undo = state.history.can_undo();
        let can_redo = state.history.can_redo();
        if ui.add_enabled(can_undo, egui::Button::new("↶ Undo")).on_hover_text("Hoàn tác thao tác vừa thực hiện [Ctrl+Z]").clicked() {
            let current = state.snapshot();
            if let Some(prev) = state.history.undo(current) {
                state.restore_snapshot(prev);
                state.status_message = "Đã hoàn tác (Undo)".to_string();
            }
        }
        if ui.add_enabled(can_redo, egui::Button::new("↷ Redo")).on_hover_text("Làm lại thao tác vừa hoàn tác [Ctrl+Y / Ctrl+Shift+Z]").clicked() {
            let current = state.snapshot();
            if let Some(next) = state.history.redo(current) {
                state.restore_snapshot(next);
                state.status_message = "Đã làm lại (Redo)".to_string();
            }
        }

        if ui.button("📑 Quản lý Visual Groups").on_hover_text("Mở hộp thoại tạo, đổi tên và xóa Visual Groups").clicked() {
            state.show_groups_dialog = !state.show_groups_dialog;
        }

        if ui.button("💾 Export Preset").clicked() {
            let preset = NodeGraphPreset {
                name: "Darkworks Exported Preset".to_string(),
                version: "1.0.0".to_string(),
                description: "Exported from Darkworks Node Graph".to_string(),
                nodes: state.snarl.node_ids().map(|(_id, node)| node.clone()).collect(),
                groups: state.groups.clone(),
            };
            if let Ok(json) = preset.to_json_string() {
                ui.ctx().copy_text(json);
                state.status_message = "Đã copy Preset JSON vào Clipboard!".to_string();
            }
        }

        if ui.button("⚡ Validate DAG").clicked() {
            match darkworks_node_graph::DagEvaluator::build_execution_plan(&state.snarl) {
                Ok(plan) => {
                    state.status_message = format!("✔ DAG Hợp lệ ({} bước tuần tự, {} tầng song song)", plan.steps.len(), plan.execution_levels.len());
                }
                Err(e) => {
                    state.status_message = format!("✖ {}", e);
                }
            }
        }

        if ui.button("🔄 Reset").clicked() {
            state.push_undo();
            let (snarl, groups) = NodeGraphPreset::load_builtin(BuiltinPresetKind::NewsTickerBanner);
            state.snarl = snarl;
            state.groups = groups;
            state.custom_titles.clear();
            state.selected_group = None;
            state.last_hovered_group = None;
            state.detach_to_scratchpad();
            state.status_message = "Đã đặt lại Preset mặc định (chế độ Nháp)".to_string();
        }

        if ui.button("🗑 Clear All").clicked() {
            state.push_undo();
            state.snarl = Snarl::new();
            state.groups.clear();
            state.custom_titles.clear();
            state.selected_group = None;
            state.last_hovered_group = None;
            state.detach_to_scratchpad();
            state.status_message = "Đã dọn sạch đồ thị (chế độ Nháp)".to_string();
        }

        ui.separator();
        if ui.button("🗑 Delete").on_hover_text("Xóa Node đang chọn hoặc Group [Delete / Backspace]").clicked() {
            delete_selected_or_hovered(ui.ctx());
        }
        ui.separator();
        if ui.button(egui::RichText::new("🔍 -").strong()).on_hover_text("Thu nhỏ (Zoom Out 80%)").clicked() {
            state.zoom_action = Some(ZoomAction::ZoomOut);
        }
        let zoom_pct = (state.current_zoom * 100.0).round() as i32;
        if ui.button(format!("{zoom_pct}%")).on_hover_text("Khôi phục tỷ lệ 100%").clicked() {
            state.zoom_action = Some(ZoomAction::Reset100);
        }
        if ui.button(egui::RichText::new("🔍 +").strong()).on_hover_text("Phóng to (Zoom In 125%)").clicked() {
            state.zoom_action = Some(ZoomAction::ZoomIn);
        }
        if ui.button("🎯 Fit View").on_hover_text("Căn giữa toàn bộ Node (Fit All)").clicked() {
            state.zoom_action = Some(ZoomAction::FitAll);
        }
    });

    // 2. THANH TRẠNG THÁI (STATUS BAR) GỌN GÀNG RIÊNG BIỆT
    if !state.status_message.is_empty() {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("ℹ ").color(Color32::from_rgb(0, 210, 255)).small());
            ui.label(egui::RichText::new(&state.status_message).weak().small());
        });
    }

    ui.separator();

    // 3. CANVAS HIỂN THỊ ĐỒ THỊ SNARL (Visual Groups vẽ bên trong Snarl qua draw_background)
    let snarl_id = snarl_canvas_id();
    state.last_snarl_id = Some(snarl_id);

    // Xử lý phím tắt Gom nhóm (Ctrl + G) và Rã nhóm (Ctrl + Shift + G)
    let ungroup_shortcut = ui.input(|i| (i.modifiers.command || i.modifiers.ctrl) && i.modifiers.shift && i.key_pressed(egui::Key::G));
    let group_shortcut = ui.input(|i| (i.modifiers.command || i.modifiers.ctrl) && !i.modifiers.shift && i.key_pressed(egui::Key::G));
    let f2_shortcut = ui.input(|i| i.key_pressed(egui::Key::F2));

    if ungroup_shortcut {
        let selected_nodes = egui_snarl::ui::get_selected_nodes(snarl_id, ui.ctx());
        // Tìm group mục tiêu để rã: ưu tiên group chứa node đang chọn, hoặc group vừa rê chuột
        let target_idx = state.groups.iter().position(|g| {
            selected_nodes.iter().any(|&nid| {
                state.snarl.get_node_info(nid).is_some_and(|info| g.contains_node(info.pos))
            })
        }).or(state.last_hovered_group).or_else(|| if state.groups.is_empty() { None } else { Some(state.groups.len() - 1) });

        if let Some(idx) = target_idx {
            state.push_undo();
            if let Some(msg) = node_graph_groups::ungroup_at_index(idx, state) {
                state.status_message = format!("{msg} (Ctrl+Shift+G)");
            }
        }
    } else if group_shortcut {
        state.push_undo();
        let selected_nodes = egui_snarl::ui::get_selected_nodes(snarl_id, ui.ctx());
        let mut bb = Rect::NOTHING;
        for &node_id in &selected_nodes {
            if let Some(info) = state.snarl.get_node_info(node_id) {
                bb.extend_with(info.pos);
                bb.extend_with(info.pos + vec2(220.0, 160.0));
            }
        }
        let next_id = state.groups.len() + 1;
        let color = VisualGroup::default_color(state.groups.len());
        let (pos, size) = if bb.is_finite() {
            let expanded = bb.expand(35.0);
            (expanded.min, expanded.size())
        } else {
            let offset = (state.groups.len() as f32) * 50.0;
            (Pos2::new(120.0 + offset, 120.0 + offset), Vec2::new(450.0, 300.0))
        };
        state.groups.push(VisualGroup::new(
            next_id,
            format!("Group {next_id}"),
            pos,
            size,
            color,
        ));
        if selected_nodes.is_empty() {
            state.status_message = format!("Đã tạo Group {next_id} (Ctrl+G)");
        } else {
            state.status_message = format!("Đã tạo Group {next_id} bao quanh {} node (Ctrl+G)", selected_nodes.len());
        }
    }

    if f2_shortcut
        && state.renaming_group.is_none()
        && let Some(hovered) = state.last_hovered_group
        && let Some(group) = state.groups.get(hovered)
        && !group.is_locked
    {
        let title = group.title.clone();
        state.renaming_group = Some(hovered);
        state.renaming_group_buf = title;
    }

    let mut style = SnarlStyle::new();
    style.bg_pattern = Some(BackgroundPattern::Grid(egui_snarl::ui::Grid::new(vec2(20.0, 20.0), 0.0)));
    style.centering = Some(true);

    let canvas_rect = ui.available_rect_before_wrap();
    let mut canvas_ui = ui.new_child(egui::UiBuilder::new().max_rect(canvas_rect));

    // 1. Kết xuất Snarl Canvas trước để hiển thị đồ thị và xác định to_global chính xác
    let mut viewer = DarkworksViewer::with_full_state(
        state.zoom_action.take(),
        state.current_zoom,
        state.current_to_global,
        state.is_dragging_group,
        state.groups.clone(),
        state.custom_titles.clone(),
        state.renaming_node,
        state.renaming_buf.clone(),
    );

    egui_snarl::ui::SnarlWidget::new()
        .id(snarl_id)
        .style(style)
        .show(&mut state.snarl, &mut viewer, &mut canvas_ui);

    state.current_to_global = viewer.to_global;
    state.current_zoom = viewer.current_scale;
    state.custom_titles = viewer.custom_titles;
    state.renaming_node = viewer.renaming_node;
    state.renaming_buf = viewer.renaming_buf;
    if let Some(removed) = viewer.node_to_remove {
        state.push_undo();
        state.snarl.remove_node(removed);
        state.custom_titles.remove(&removed);
        egui_snarl::ui::deselect_all_nodes(snarl_id, ui.ctx());
    }

    // Xử lý phím tắt Xóa hàng loạt (Delete / Backspace / Ctrl+Delete / Ctrl+Backspace)
    let is_typing = state.renaming_group.is_some()
        || state.renaming_node.is_some()
        || ui.ctx().egui_wants_keyboard_input();
    let delete_shortcut = !is_typing
        && ui.input_mut(|i| {
            i.consume_key(egui::Modifiers::NONE, egui::Key::Delete)
                || i.consume_key(egui::Modifiers::NONE, egui::Key::Backspace)
                || i.consume_key(egui::Modifiers::COMMAND, egui::Key::Delete)
                || i.consume_key(egui::Modifiers::COMMAND, egui::Key::Backspace)
                || i.consume_key(egui::Modifiers::CTRL, egui::Key::Delete)
                || i.consume_key(egui::Modifiers::CTRL, egui::Key::Backspace)
        });

    if delete_shortcut {
        delete_selected_or_hovered(ui.ctx());
    }

    // Xử lý phím tắt Hoàn tác (Ctrl + Z) và Làm lại (Ctrl + Y / Ctrl + Shift + Z)
    let undo_shortcut = !is_typing
        && ui.input_mut(|i| {
            i.consume_key(egui::Modifiers::COMMAND, egui::Key::Z)
                || i.consume_key(egui::Modifiers::CTRL, egui::Key::Z)
        });
    let redo_shortcut = !is_typing
        && ui.input_mut(|i| {
            i.consume_key(egui::Modifiers::COMMAND | egui::Modifiers::SHIFT, egui::Key::Z)
                || i.consume_key(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, egui::Key::Z)
                || i.consume_key(egui::Modifiers::COMMAND, egui::Key::Y)
                || i.consume_key(egui::Modifiers::CTRL, egui::Key::Y)
        });

    if undo_shortcut {
        undo(ui.ctx());
    } else if redo_shortcut {
        redo(ui.ctx());
    }

    // 2. LỚP PHỦ TƯƠNG TÁC ƯU TIÊN (TOPMOST OVERLAY) CHO VISUAL GROUPS
    // Đặt layer trên cùng (sublayer của snarl_layer_id) để bắt trọn Hover/Click/Drag trước snarl background
    let snarl_layer_id = egui::LayerId::new(canvas_ui.layer_id().order, snarl_id);
    let overlay_id = ui.make_persistent_id("darkworks-snarl-overlay");
    let overlay_layer_id = egui::LayerId::new(canvas_ui.layer_id().order, overlay_id);
    ui.ctx().set_sublayer(snarl_layer_id, overlay_layer_id);
    let mut overlay_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(canvas_rect)
            .layer_id(overlay_layer_id),
    );

    let to_global = state.current_to_global;
    let over_node_header = if to_global.scaling > 0.001 {
        let pointer_pos = ui.input(|i| i.pointer.latest_pos());
        let mouse_graph = pointer_pos.map(|p| to_global.inverse() * p);
        mouse_graph.is_some_and(|mg| {
            state.snarl.nodes_pos_ids().any(|(_, pos, _)| {
                // Chỉ nhường chuột khi con trỏ nằm trực tiếp trên thanh Header Bar của Node (~26px)
                Rect::from_min_size(pos, vec2(180.0, 26.0)).contains(mg)
            })
        })
    } else {
        false
    };

    let group_res = node_graph_groups::handle_group_interactions(
        &state.groups,
        to_global,
        &mut overlay_ui,
        over_node_header,
        state.renaming_group.is_some(),
    );

    state.is_dragging_group = group_res.is_dragging;

    if let Some(hovered) = group_res.hovered_group {
        state.last_hovered_group = Some(hovered);
    }

    if let Some(selected) = group_res.selected_group {
        state.selected_group = Some(selected);
    }

    if let Some((idx, delta)) = group_res.resize_action
        && let Some(group) = state.groups.get_mut(idx)
    {
        group.max[0] = (group.max[0] + delta.x).max(group.min[0] + 120.0);
        group.max[1] = (group.max[1] + delta.y).max(group.min[1] + 80.0);
    }

    if let Some((idx, old_rect, delta)) = group_res.drag_action {
        // 1. Tịnh tiến group đang kéo
        if let Some(group) = state.groups.get_mut(idx) {
            group.translate(delta);
        }
        // 2. Tịnh tiến tất cả group con lồng trọn 100% bên trong group đang kéo (Strict Containment)
        for (other_idx, other_group) in state.groups.iter_mut().enumerate() {
            if other_idx != idx
                && old_rect.contains(other_group.rect().min)
                && old_rect.contains(other_group.rect().max)
            {
                other_group.translate(delta);
            }
        }
        // 3. Tịnh tiến tất cả node con nằm trong group đang kéo
        for node in state.snarl.nodes_info_mut() {
            if old_rect.contains(node.pos) {
                node.pos += delta;
            }
        }
    }

    if let Some(idx) = group_res.start_inline_rename
        && let Some(group) = state.groups.get(idx)
    {
        let title = group.title.clone();
        state.renaming_group = Some(idx);
        state.renaming_group_buf = title;
    }

    if let Some(menu_action) = group_res.menu_action {
        match menu_action {
            GroupMenuAction::StartRename(idx) => {
                if let Some(group) = state.groups.get(idx) {
                    let title = group.title.clone();
                    state.renaming_group = Some(idx);
                    state.renaming_group_buf = title;
                }
            }
            GroupMenuAction::SetColor(idx, color_idx) => {
                if let Some(group) = state.groups.get_mut(idx) {
                    group.set_color_index(color_idx);
                    state.status_message = format!("Đã đổi màu nhóm '{}'", group.title);
                }
            }
            GroupMenuAction::AutoFit(idx) => {
                if node_graph_groups::auto_fit_group_at_index(idx, state) {
                    state.status_message = "Đã căn vừa khung nhóm theo các Node con (Auto-Fit)".to_string();
                }
            }
            GroupMenuAction::ToggleLock(idx) => {
                if let Some(group) = state.groups.get_mut(idx) {
                    let locked = group.toggle_lock();
                    state.status_message = if locked {
                        format!("Đã khóa vị trí nhóm '{}'", group.title)
                    } else {
                        format!("Đã mở khóa vị trí nhóm '{}'", group.title)
                    };
                }
            }
            GroupMenuAction::Ungroup(idx) => {
                state.push_undo();
                if let Some(msg) = node_graph_groups::ungroup_at_index(idx, state) {
                    state.status_message = msg;
                }
            }
            GroupMenuAction::DeleteAll(idx) => {
                state.push_undo();
                if let Some(msg) = node_graph_groups::delete_group_and_nodes_at_index(idx, state) {
                    state.status_message = msg;
                }
            }
        }
    }

    // Vẽ widget sửa tên nhóm trực tiếp (Inline Rename) trên lớp phủ overlay nếu đang kích hoạt
    if let Some(renaming_idx) = state.renaming_group {
        let is_done = node_graph_groups::render_inline_group_rename(
            renaming_idx,
            state,
            &mut overlay_ui,
        );
        if is_done {
            state.renaming_group = None;
        }
    }

    // 3. HỘP THOẠI DIALOG QUẢN LÝ VISUAL GROUPS (Floating Window)
    node_graph_groups::show_groups_dialog(
        state,
        snarl_id,
        ui.ctx(),
    );

    // 4. THỰC THI ACTION NODE & ĐỒNG BỘ LIVE XUỐNG TIMELINE
    node_graph_bridge::process_action_nodes_and_live_sync(app, ui, state);
}
