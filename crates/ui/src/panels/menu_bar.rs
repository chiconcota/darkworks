//! Thanh menu lệnh và chỉ số hệ thống thời gian thực.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use egui::{Color32, RichText, Ui, containers::menu::MenuBar};

/// Vẽ thanh menu chính phía trên cùng của ứng dụng.
pub fn render(ui: &mut Ui, project_name: &str, ram_mb: usize, budget_mb: usize, fps: f32) {
    MenuBar::new().ui(ui, |ui| {
        ui.horizontal(|ui| {
            // Logo / Tên thương hiệu
            ui.label(
                RichText::new("⚡ DARKWORKS")
                    .strong()
                    .color(Color32::from_rgb(52, 224, 249)),
            );
            ui.separator();

            // Menu File
            ui.menu_button("File", |ui| {
                if ui.button("+ New Project (Ctrl+N)").clicked() {
                    ui.close();
                }
                if ui.button("▤ Open Project... (Ctrl+O)").clicked() {
                    ui.close();
                }
                if ui.button("◆ Save Project (Ctrl+S)").clicked() {
                    ui.close();
                }
                ui.separator();
                if ui.button("✕ Exit").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            // Menu Edit
            ui.menu_button("Edit", |ui| {
                if ui.button("◀ Undo (Ctrl+Z)").clicked() {
                    ui.close();
                }
                if ui.button("▶ Redo (Ctrl+Y)").clicked() {
                    ui.close();
                }
                ui.separator();
                if ui.button("✂ Split at Playhead (S)").clicked() {
                    ui.close();
                }
                if ui.button("✕ Ripple Delete (X)").clicked() {
                    ui.close();
                }
            });

            // Menu View
            ui.menu_button("View", |ui| {
                if ui.button("🔍 Zoom In (Ctrl+=)").clicked() {
                    ui.close();
                }
                if ui.button("🔍 Zoom Out (Ctrl+-)").clicked() {
                    ui.close();
                }
                if ui.button("🎯 Zoom to Fit (Shift+Z)").clicked() {
                    ui.close();
                }
            });

            // Menu AI & Automation
            ui.menu_button("AI Tools", |ui| {
                if ui.button("⚡ Auto Cut Silence (Silero VAD)").clicked() {
                    ui.close();
                }
                if ui.button("💬 Generate Subtitles (Whisper Local)").clicked() {
                    ui.close();
                }
            });

            // Tiêu đề dự án hiện tại
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Chỉ số bộ nhớ (Khóa trần RAM < 2GB theo Khẩu quyết #1, Mức 1: 1024MB)
                let ram_color = if ram_mb >= 1900 {
                    Color32::from_rgb(239, 68, 68) // Đỏ cảnh báo (> 1.9GB)
                } else if ram_mb >= budget_mb.saturating_sub(64) {
                    Color32::from_rgb(245, 158, 11) // Vàng tiệm cận budget (ví dụ 1GB)
                } else {
                    Color32::from_rgb(16, 185, 129) // Xanh lá an toàn
                };

                ui.label(
                    RichText::new("GPU: Vulkan Zero-Copy")
                        .size(11.0)
                        .color(Color32::from_rgb(148, 163, 184)),
                );
                ui.separator();

                ui.label(
                    RichText::new(format!("RAM: {ram_mb}MB / {budget_mb}MB (2GB Cap)"))
                        .size(11.0)
                        .color(ram_color),
                );
                ui.separator();

                ui.label(
                    RichText::new(format!("{fps:.0} FPS"))
                        .size(11.0)
                        .color(Color32::from_rgb(52, 224, 249)),
                );
                ui.separator();

                ui.label(
                    RichText::new(format!("▤ {project_name}"))
                        .size(12.0)
                        .strong()
                        .color(Color32::from_rgb(226, 232, 240)),
                );
            });
        });
    });
}
