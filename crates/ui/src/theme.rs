//! Bảng màu và phong cách giao diện Darkworks (Theme & Visuals).

#![deny(clippy::unwrap_used, clippy::expect_used)]

use egui::{Color32, CornerRadius, FontData, FontDefinitions, FontFamily, Stroke, Visuals};
use std::sync::Arc;

pub const COLOR_BG_DARK: Color32 = Color32::from_rgb(10, 14, 20);
pub const COLOR_PANEL_BG: Color32 = Color32::from_rgb(15, 20, 28);
pub const COLOR_BORDER: Color32 = Color32::from_rgb(30, 41, 59);
pub const COLOR_ACCENT_CYAN: Color32 = Color32::from_rgb(52, 224, 249);
pub const COLOR_ACCENT_CYAN_DIM: Color32 = Color32::from_rgb(20, 110, 130);
pub const COLOR_TEXT_PRIMARY: Color32 = Color32::from_rgb(241, 245, 249);
pub const COLOR_TEXT_MUTED: Color32 = Color32::from_rgb(148, 163, 184);

pub const COLOR_TRACK_CANVAS: Color32 = Color32::from_rgb(56, 189, 248);
pub const COLOR_TRACK_VIDEO: Color32 = Color32::from_rgb(59, 130, 246);
pub const COLOR_TRACK_AUDIO: Color32 = Color32::from_rgb(16, 185, 129);
pub const COLOR_PLAYHEAD: Color32 = Color32::from_rgb(239, 68, 68);

/// Thiết lập giao diện tối ưu chuẩn NLE chuyên nghiệp cho egui Context.
pub fn apply_theme(ctx: &egui::Context) {
    // 1. Bảng màu Dark Sci-Fi
    let mut visuals = Visuals::dark();
    visuals.panel_fill = COLOR_PANEL_BG;
    visuals.window_fill = COLOR_PANEL_BG;
    visuals.extreme_bg_color = COLOR_BG_DARK;
    visuals.window_stroke = Stroke::new(1.0, COLOR_BORDER);
    visuals.override_text_color = Some(COLOR_TEXT_PRIMARY);

    visuals.widgets.noninteractive.bg_fill = COLOR_PANEL_BG;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, COLOR_BORDER);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(4);

    visuals.widgets.inactive.bg_fill = Color32::from_rgb(23, 31, 43);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, COLOR_BORDER);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(4);

    visuals.widgets.hovered.bg_fill = Color32::from_rgb(30, 42, 60);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, COLOR_ACCENT_CYAN_DIM);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(4);

    visuals.widgets.active.bg_fill = Color32::from_rgb(38, 54, 76);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, COLOR_ACCENT_CYAN);
    visuals.widgets.active.corner_radius = CornerRadius::same(4);

    ctx.set_visuals(visuals);

    // 2. Nạp font DejaVu Sans (Đảm bảo 100% hiển thị tiếng Việt và ký tự Symbol chuẩn)
    let mut fonts = FontDefinitions::default();
    let font_bytes = include_bytes!("../../../assets/fonts/DejaVuSans.ttf");
    fonts.font_data.insert(
        "DejaVuSans".to_owned(),
        Arc::new(FontData::from_static(font_bytes)),
    );

    if let Some(vec) = fonts.families.get_mut(&FontFamily::Proportional) {
        vec.insert(0, "DejaVuSans".to_owned());
    }
    if let Some(vec) = fonts.families.get_mut(&FontFamily::Monospace) {
        vec.push("DejaVuSans".to_owned());
    }

    ctx.set_fonts(fonts);
}
