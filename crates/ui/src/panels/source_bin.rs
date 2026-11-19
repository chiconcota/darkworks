//! Phân vùng quản lý tài nguyên Media (Source / Project Bin).

#![deny(clippy::unwrap_used, clippy::expect_used)]

use egui::{Color32, RichText, ScrollArea, Ui};

/// Đại diện một tệp media trong Source Bin.
#[derive(Debug, Clone)]
pub struct MediaAsset {
    pub name: String,
    pub path: String,
    pub duration_str: String,
    pub resolution: String,
    pub codec: String,
}

/// Vẽ phân vùng Source Bin ở góc trên bên trái.
pub fn render(ui: &mut Ui, assets: &[MediaAsset], search_query: &mut String) {
    ui.vertical(|ui| {
        // Tiêu đề phân vùng
        ui.horizontal(|ui| {
            ui.label(RichText::new("📁 PROJECT BIN").strong().size(13.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("➕ Import").clicked() {
                    log::info!("Mở hộp thoại nhập tài nguyên media");
                }
            });
        });
        ui.separator();

        // Ô tìm kiếm tài nguyên
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.text_edit_singleline(search_query);
        });
        ui.add_space(4.0);

        // Danh sách tài nguyên
        ScrollArea::vertical().show(ui, |ui| {
            if assets.is_empty() {
                ui.add_space(20.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("Chưa có tệp media nào")
                            .color(Color32::from_rgb(148, 163, 184))
                            .italics(),
                    );
                    ui.label(
                        RichText::new("Kéo thả video vào đây hoặc bấm ➕ Import")
                            .size(11.0)
                            .color(Color32::from_rgb(100, 116, 139)),
                    );
                });
            } else {
                for asset in assets {
                    if !search_query.is_empty()
                        && !asset.name.to_lowercase().contains(&search_query.to_lowercase())
                    {
                        continue;
                    }

                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("🎬");
                            ui.vertical(|ui| {
                                ui.label(RichText::new(&asset.name).strong());
                                ui.label(
                                    RichText::new(format!(
                                        "{} • {} • {}",
                                        asset.duration_str, asset.resolution, asset.codec
                                    ))
                                    .size(11.0)
                                    .color(Color32::from_rgb(148, 163, 184)),
                                );
                            });
                        });
                    });
                    ui.add_space(2.0);
                }
            }
        });
    });
}
