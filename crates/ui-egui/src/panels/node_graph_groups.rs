//! Phân hệ xử lý tương tác, bảng màu, phím tắt và hộp thoại cho Visual Groups (ComfyUI Style).

use darkworks_node_graph::VisualGroup;
use egui::{vec2, Color32, CursorIcon, Pos2, Rect, RichText, Ui, Vec2};

use super::node_graph::NodeGraphState;

/// Hành động phát sinh từ Menu Chuột Phải hoặc phím tắt của Visual Group.
#[derive(Debug, Clone)]
pub enum GroupMenuAction {
    StartRename(usize),
    SetColor(usize, usize),
    AutoFit(usize),
    ToggleLock(usize),
    Ungroup(usize),
    DeleteAll(usize),
}

/// Kết quả tương tác từ Canvas với các Visual Groups trong một khung hình.
#[derive(Default)]
pub struct GroupInteractionResult {
    pub is_dragging: bool,
    pub drag_action: Option<(usize, Rect, Vec2)>,
    pub resize_action: Option<(usize, Vec2)>,
    pub menu_action: Option<GroupMenuAction>,
    pub start_inline_rename: Option<usize>,
    pub hovered_group: Option<usize>,
    pub selected_group: Option<usize>,
}

/// Xử lý tương tác kéo di chuyển, co giãn, nhấp đúp và menu chuột phải trên Canvas.
pub fn handle_group_interactions(
    groups: &[VisualGroup],
    to_global: egui::emath::TSTransform,
    canvas_ui: &mut Ui,
    over_node_header: bool,
    is_renaming: bool,
) -> GroupInteractionResult {
    let mut result = GroupInteractionResult::default();
    if to_global.scaling <= 0.001 {
        return result;
    }

    // Duyệt theo thứ tự đảo ngược (rev): Group con (nằm trên/tạo sau) nhận tương tác trước Group cha
    for (idx, group) in groups.iter().enumerate().rev() {
        // 1. Tay cầm co giãn góc dưới phải (Resize Grip)
        if !group.is_locked {
            let resize_screen = to_global * group.resize_rect();
            let resize_id = canvas_ui.scope_id().with(("vg_resize", idx, group.id));
            let resize_resp = canvas_ui.interact(resize_screen, resize_id, egui::Sense::click_and_drag());

            if resize_resp.hovered() || resize_resp.dragged() {
                canvas_ui.ctx().set_cursor_icon(CursorIcon::ResizeNwSe);
            }
            if resize_resp.dragged_by(egui::PointerButton::Primary) && result.resize_action.is_none() {
                let delta = resize_resp.drag_delta() / to_global.scaling;
                result.resize_action = Some((idx, delta));
                result.is_dragging = true;
                canvas_ui.ctx().request_repaint();
            }
            if resize_resp.double_clicked() && result.menu_action.is_none() {
                result.menu_action = Some(GroupMenuAction::AutoFit(idx));
            }
        }

        // 2. Nút bấm xóa nhanh "✖" (Close Button) trên Header Bar
        let close_screen = to_global * group.close_button_rect();
        let close_id = canvas_ui.scope_id().with(("vg_close", idx, group.id));
        let close_resp = canvas_ui.interact(close_screen, close_id, egui::Sense::click())
            .on_hover_text("Xóa khung Group (giữ nguyên Node con)");

        if close_resp.hovered() {
            canvas_ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
            canvas_ui.painter().rect_filled(
                close_screen,
                4.0,
                Color32::from_rgba_unmultiplied(255, 60, 60, 120),
            );
            canvas_ui.painter().text(
                close_screen.center(),
                egui::Align2::CENTER_CENTER,
                "✖",
                egui::FontId::proportional(13.0 * to_global.scaling.clamp(0.7, 1.4)),
                Color32::WHITE,
            );
        }

        if close_resp.clicked() && result.menu_action.is_none() {
            result.menu_action = Some(GroupMenuAction::Ungroup(idx));
            canvas_ui.ctx().request_repaint();
        }

        // 3. Thanh tiêu đề (Header Bar) - Kéo di chuyển, Nhấp đúp sửa tên, Chuột phải mở menu
        let header_screen = to_global * group.header_drag_rect();
        let header_id = canvas_ui.scope_id().with(("vg_header", idx, group.id));
        let header_resp = canvas_ui.interact(header_screen, header_id, egui::Sense::click_and_drag());
        let is_menu_open = header_resp.context_menu_opened();

        if !result.is_dragging && !is_menu_open {
            if header_resp.clicked() && result.selected_group.is_none() {
                result.selected_group = Some(idx);
            }

            if header_resp.dragged() {
                canvas_ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
            } else if header_resp.hovered() && result.hovered_group.is_none() {
                result.hovered_group = Some(idx);
                if group.is_locked {
                    canvas_ui.ctx().set_cursor_icon(CursorIcon::NotAllowed);
                } else {
                    canvas_ui.ctx().set_cursor_icon(CursorIcon::Grab);
                }
            }

            if !group.is_locked
                && !over_node_header
                && header_resp.dragged_by(egui::PointerButton::Primary)
                && result.drag_action.is_none()
            {
                let delta = header_resp.drag_delta() / to_global.scaling;
                result.drag_action = Some((idx, group.rect(), delta));
                result.is_dragging = true;
                canvas_ui.ctx().request_repaint();
            }

            // Nhấp đúp trên Header Bar: Bắt đầu Inline Rename trực tiếp
            if !group.is_locked && !is_renaming && header_resp.double_clicked() && result.start_inline_rename.is_none() {
                result.start_inline_rename = Some(idx);
            }
        }

        // Menu Ngữ Cảnh Chuột Phải trên Header Bar (luôn được đăng ký trên mọi frame để giữ popup vững chãi)
        let group_title = group.title.clone();
        let is_locked = group.is_locked;
        let current_color = group.color();

        header_resp.context_menu(|ui| {
                ui.label(RichText::new(format!("📁 {group_title}")).strong());
                ui.separator();

                if ui.button("✏️ Đổi tên (Inline Rename)").clicked() {
                    result.menu_action = Some(GroupMenuAction::StartRename(idx));
                    ui.close();
                }

                ui.menu_button("🎨 Bảng màu (Color Palette)", |ui| {
                    for (c_idx, &c_rgba) in VisualGroup::PALETTE.iter().enumerate() {
                        let c = Color32::from_rgba_unmultiplied(c_rgba[0], c_rgba[1], c_rgba[2], 255);
                        let name = VisualGroup::COLOR_NAMES.get(c_idx).copied().unwrap_or("Màu khác");
                        ui.horizontal(|ui| {
                            let (rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), egui::Sense::hover());
                            ui.painter().rect_filled(rect, 3.0, c);
                            if ui.selectable_label(current_color == Color32::from_rgba_unmultiplied(c_rgba[0], c_rgba[1], c_rgba[2], c_rgba[3]), name).clicked() {
                                result.menu_action = Some(GroupMenuAction::SetColor(idx, c_idx));
                                ui.close();
                            }
                        });
                    }
                });

                if ui.button("🎯 Căn vừa Node con (Auto-Fit)").clicked() {
                    result.menu_action = Some(GroupMenuAction::AutoFit(idx));
                    ui.close();
                }

                let lock_label = if is_locked { "🔓 Mở khóa vị trí (Unlock)" } else { "🔒 Khóa vị trí (Lock)" };
                if ui.button(lock_label).clicked() {
                    result.menu_action = Some(GroupMenuAction::ToggleLock(idx));
                    ui.close();
                }

                ui.separator();

                if ui.button("🔓 Rã nhóm (Ungroup - Giữ Node)").clicked() {
                    result.menu_action = Some(GroupMenuAction::Ungroup(idx));
                    ui.close();
                }

                if ui.button(RichText::new("🗑 Xóa toàn bộ (Group & Nodes)").color(Color32::from_rgb(255, 100, 100))).clicked() {
                    result.menu_action = Some(GroupMenuAction::DeleteAll(idx));
                    ui.close();
                }
            });
    }

    result
}

/// Vẽ ô nhập liệu sửa tên trực tiếp (Inline Rename) trên Header Bar của Group.
/// Trả về `true` khi đã hoàn tất (Enter, Esc hoặc bấm ra ngoài).
pub fn render_inline_group_rename(
    group_idx: usize,
    state: &mut NodeGraphState,
    canvas_ui: &mut Ui,
) -> bool {
    let Some(header_rect) = state.groups.get(group_idx).map(|g| g.header_rect()) else {
        return true;
    };

    let header_screen = state.current_to_global * header_rect;
    let edit_rect = Rect::from_min_size(
        header_screen.min + vec2(28.0, 3.0),
        vec2((header_screen.width() - 36.0).max(120.0), 24.0),
    );

    let mut edit_ui = canvas_ui.new_child(egui::UiBuilder::new().max_rect(edit_rect));
    let edit = egui::TextEdit::singleline(&mut state.renaming_group_buf)
        .desired_width(edit_rect.width())
        .font(egui::TextStyle::Body);
    let resp = edit_ui.add(edit);
    resp.request_focus();

    if resp.lost_focus() || edit_ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        let trimmed = state.renaming_group_buf.trim();
        if !trimmed.is_empty()
            && let Some(group) = state.groups.get_mut(group_idx)
        {
            group.title = trimmed.to_string();
        }
        true
    } else {
        edit_ui.input(|i| i.key_pressed(egui::Key::Escape))
    }
}

/// Rã nhóm (Ungroup) tại vị trí chỉ định mà KHÔNG xóa node hay dây nối.
pub fn ungroup_at_index(
    idx: usize,
    state: &mut NodeGraphState,
) -> Option<String> {
    if idx >= state.groups.len() {
        return None;
    }
    let removed = state.groups.remove(idx);
    let count = state.snarl.nodes_pos_ids().filter(|(_, pos, _)| removed.contains_node(*pos)).count();
    Some(format!("Đã rã nhóm '{}' (giữ nguyên {} nodes)", removed.title, count))
}

/// Căn vừa kích thước group ôm sát các Node bên trong.
pub fn auto_fit_group_at_index(
    idx: usize,
    state: &mut NodeGraphState,
) -> bool {
    if idx >= state.groups.len() {
        return false;
    }
    let node_rects: Vec<Rect> = state.snarl
        .nodes_pos_ids()
        .map(|(_, pos, _)| Rect::from_min_size(pos, vec2(220.0, 160.0)))
        .collect();
    if let Some(group) = state.groups.get_mut(idx) {
        group.auto_fit(node_rects)
    } else {
        false
    }
}

/// Xóa cả khung Visual Group VÀ toàn bộ Node con nằm bên trong.
pub fn delete_group_and_nodes_at_index(
    idx: usize,
    state: &mut NodeGraphState,
) -> Option<String> {
    if idx >= state.groups.len() {
        return None;
    }
    let removed = state.groups.remove(idx);
    let mut nodes_to_delete = Vec::new();
    for (id, pos, _) in state.snarl.nodes_pos_ids() {
        if removed.contains_node(pos) {
            nodes_to_delete.push(id);
        }
    }
    let count = nodes_to_delete.len();
    for id in nodes_to_delete {
        state.snarl.remove_node(id);
        state.custom_titles.remove(&id);
    }
    Some(format!("Đã xóa nhóm '{}' và {} node thành viên", removed.title, count))
}

/// Hiển thị hộp thoại nổi Quản lý Visual Groups.
pub fn show_groups_dialog(
    state: &mut NodeGraphState,
    snarl_id: egui::Id,
    ctx: &egui::Context,
) {
    if !state.show_groups_dialog {
        return;
    }

    let mut show_dialog = state.show_groups_dialog;
    egui::Window::new("📑 Quản lý Visual Groups")
        .open(&mut show_dialog)
        .resizable(false)
        .collapsible(false)
        .default_size(vec2(380.0, 280.0))
        .show(ctx, |ui| {
            ui.label(RichText::new("➕ Thêm Visual Group mới").strong());
            ui.horizontal(|ui| {
                ui.label("Tên:");
                ui.text_edit_singleline(&mut state.new_group_title);
                let default_name = format!("Group {}", state.groups.len() + 1);
                let title = if state.new_group_title.trim().is_empty() {
                    default_name
                } else {
                    state.new_group_title.trim().to_string()
                };
                if ui.button("Tạo nhóm").clicked() {
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
                    state.groups.push(VisualGroup::new(next_id, title, pos, size, color));
                    state.new_group_title.clear();
                    state.status_message = "Đã tạo Visual Group mới".to_string();
                }
            });

            ui.separator();

            if state.groups.is_empty() {
                ui.label(RichText::new("Chưa có nhóm nào trên Canvas").weak());
            } else {
                ui.label(RichText::new("Danh sách Nhóm:").strong());
                let mut group_to_remove = None;
                let mut group_to_fit = None;

                egui::ScrollArea::vertical().max_height(160.0).show(ui, |ui| {
                    for (idx, group) in state.groups.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(format!("{}.", idx + 1));
                            ui.text_edit_singleline(&mut group.title);

                            if ui.button("🎯").on_hover_text("Căn vừa Node con").clicked() {
                                group_to_fit = Some(idx);
                            }

                            let lock_icon = if group.is_locked { "🔒" } else { "🔓" };
                            if ui.button(lock_icon).on_hover_text("Khóa / Mở khóa vị trí").clicked() {
                                group.toggle_lock();
                            }

                            if ui.button(RichText::new("🗑").color(Color32::from_rgb(255, 100, 100)))
                                .on_hover_text("Xóa khung nhóm này (giữ Node)")
                                .clicked()
                            {
                                group_to_remove = Some(idx);
                            }
                        });
                    }
                });

                if let Some(idx) = group_to_fit
                    && auto_fit_group_at_index(idx, state)
                {
                    state.status_message = "Đã căn vừa khung nhóm theo các Node con".to_string();
                }

                if let Some(idx) = group_to_remove
                    && let Some(msg) = ungroup_at_index(idx, state)
                {
                    state.status_message = msg;
                }

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("🧹 Dọn sạch tất cả nhóm (Giữ Node)").clicked() {
                        state.groups.clear();
                        state.status_message = "Đã dọn sạch tất cả Visual Groups".to_string();
                    }
                    if ui.button(RichText::new("💥 Xóa sạch nhóm & Node").color(Color32::from_rgb(255, 80, 80))).clicked() {
                        while !state.groups.is_empty() {
                            delete_group_and_nodes_at_index(0, state);
                        }
                        state.status_message = "Đã xóa sạch toàn bộ Visual Groups và các Node bên trong".to_string();
                    }
                });
            }
        });
    state.show_groups_dialog = show_dialog;
}
