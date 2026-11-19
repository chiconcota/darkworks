//! Bảng điều khiển thuộc tính (Inspector) & Chế độ xem trước đồ thị DAG.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use egui::{Color32, RichText, ScrollArea, Ui};

/// Tab đang hoạt động trong phân vùng Inspector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InspectorTab {
    #[default]
    Properties,
    NodeGraph,
}

/// Thuộc tính biến hình (Transform) của một Clip.
#[derive(Debug, Clone)]
pub struct TransformState {
    pub pos_x: f32,
    pub pos_y: f32,
    pub scale: f32,
    pub rotation_deg: f32,
    pub opacity: f32,
    pub blend_mode: String,
}

impl Default for TransformState {
    fn default() -> Self {
        Self {
            pos_x: 0.0,
            pos_y: 0.0,
            scale: 100.0,
            rotation_deg: 0.0,
            opacity: 100.0,
            blend_mode: "Normal".to_string(),
        }
    }
}

/// Vẽ phân vùng Inspector ở góc trên bên phải.
pub fn render(
    ui: &mut Ui,
    active_tab: &mut InspectorTab,
    transform: &mut TransformState,
    selected_clip: Option<&str>,
) {
    ui.vertical(|ui| {
        // Tab chuyển đổi chế độ (Dùng symbol chuẩn)
        ui.horizontal(|ui| {
            ui.selectable_value(active_tab, InspectorTab::Properties, "⚙ Properties");
            ui.selectable_value(active_tab, InspectorTab::NodeGraph, "◈ Node Graph (DAG)");
        });
        ui.separator();

        ScrollArea::vertical().show(ui, |ui| match *active_tab {
            InspectorTab::Properties => render_properties_tab(ui, transform, selected_clip),
            InspectorTab::NodeGraph => render_nodegraph_tab(ui),
        });
    });
}

fn render_properties_tab(
    ui: &mut Ui,
    transform: &mut TransformState,
    selected_clip: Option<&str>,
) {
    if let Some(clip_name) = selected_clip {
        ui.label(RichText::new(format!("Đang chọn: {clip_name}")).strong().color(Color32::from_rgb(52, 224, 249)));
        ui.add_space(8.0);

        // Nhóm biến hình (Transform - dùng symbol ∡)
        ui.collapsing("∡ Transform (Chuyển động)", |ui| {
            ui.horizontal(|ui| {
                ui.label("Vị trí X:");
                ui.add(egui::DragValue::new(&mut transform.pos_x).speed(1.0).suffix(" px"));
            });
            ui.horizontal(|ui| {
                ui.label("Vị trí Y:");
                ui.add(egui::DragValue::new(&mut transform.pos_y).speed(1.0).suffix(" px"));
            });
            ui.horizontal(|ui| {
                ui.label("Tỉ lệ (Scale):");
                ui.add(egui::Slider::new(&mut transform.scale, 0.0..=500.0).suffix("%"));
            });
            ui.horizontal(|ui| {
                ui.label("Góc xoay:");
                ui.add(egui::DragValue::new(&mut transform.rotation_deg).speed(0.5).suffix("°"));
            });
        });

        // Nhóm hòa trộn (Compositing - dùng symbol ◐)
        ui.collapsing("◐ Compositing (Độ mờ & Hòa trộn)", |ui| {
            ui.horizontal(|ui| {
                ui.label("Độ mờ (Opacity):");
                ui.add(egui::Slider::new(&mut transform.opacity, 0.0..=100.0).suffix("%"));
            });
            ui.horizontal(|ui| {
                ui.label("Blend Mode:");
                egui::ComboBox::from_id_salt("blend_mode_selector")
                    .selected_text(&transform.blend_mode)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut transform.blend_mode, "Normal".to_string(), "Normal");
                        ui.selectable_value(&mut transform.blend_mode, "Multiply".to_string(), "Multiply");
                        ui.selectable_value(&mut transform.blend_mode, "Screen".to_string(), "Screen");
                        ui.selectable_value(&mut transform.blend_mode, "Overlay".to_string(), "Overlay");
                        ui.selectable_value(&mut transform.blend_mode, "Add".to_string(), "Linear Add");
                    });
            });
        });
    } else {
        ui.add_space(20.0);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new("Chưa chọn clip nào trên Timeline")
                    .color(Color32::from_rgb(148, 163, 184))
                    .italics(),
            );
            ui.label(
                RichText::new("Bấm chọn một Clip để tinh chỉnh thông số")
                    .size(11.0)
                    .color(Color32::from_rgb(100, 116, 139)),
            );
        });
    }
}

fn render_nodegraph_tab(ui: &mut Ui) {
    ui.vertical(|ui| {
        ui.label(RichText::new("Sơ đồ Node Graph của Clip (DAG)").strong());
        ui.label(
            RichText::new("Kiến trúc thay thế hoàn toàn Linear Stack cũ.")
                .size(11.0)
                .color(Color32::from_rgb(148, 163, 184)),
        );
        ui.add_space(8.0);

        // Khung đồ họa minh họa node
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("▶ [Video Input]");
                ui.label("➔");
                ui.label("∡ [Transform]");
                ui.label("➔");
                ui.label("◉ [Output]");
            });
        });

        ui.add_space(12.0);
        if ui.button("⚡ Mở Cửa Sổ Node Graph Toàn Cảnh (egui_snarl)").clicked() {
            log::info!("Mở cửa sổ đồ thị Node Graph");
        }
    });
}
