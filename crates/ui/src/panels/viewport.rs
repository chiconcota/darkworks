//! Màn hình xem trước video 16:9 Zero-Copy (Viewport).

#![deny(clippy::unwrap_used, clippy::expect_used)]

use darkworks_schema::Tick;
use egui::{Color32, CornerRadius, Pos2, RichText, Sense, Stroke, Ui, Vec2};

/// Vẽ phân vùng Viewport ở vị trí trung tâm.
pub fn render(
    ui: &mut Ui,
    playhead: &mut Tick,
    duration: Tick,
    is_playing: &mut bool,
    fps: f64,
) {
    ui.vertical(|ui| {
        // Tiêu đề & Thông số định dạng
        ui.horizontal(|ui| {
            ui.label(RichText::new("🖥️ PROGRAM VIEWPORT").strong().size(13.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new("1920x1080 @ 60fps (Zero-Copy Texture)")
                        .size(11.0)
                        .color(Color32::from_rgb(148, 163, 184)),
                );
            });
        });
        ui.separator();

        // Khung hình hiển thị Video 16:9
        let available_size = ui.available_size() - Vec2::new(0.0, 48.0);
        let target_aspect = 16.0 / 9.0;

        let (preview_width, preview_height) = if available_size.x / available_size.y > target_aspect {
            (available_size.y * target_aspect, available_size.y)
        } else {
            (available_size.x, available_size.x / target_aspect)
        };

        let (rect, _response) = ui.allocate_exact_size(
            Vec2::new(preview_width.max(100.0), preview_height.max(60.0)),
            Sense::hover(),
        );

        // Vẽ nền Letterbox đen tuyền và viền hiển thị
        let painter = ui.painter();
        painter.rect_filled(rect, CornerRadius::same(6), Color32::from_rgb(2, 4, 8));
        painter.rect_stroke(
            rect,
            CornerRadius::same(6),
            Stroke::new(1.0, Color32::from_rgb(30, 41, 59)),
            egui::StrokeKind::Inside,
        );

        // Điểm ngắm tâm (Center Crosshair)
        let center = rect.center();
        painter.line_segment(
            [
                Pos2::new(center.x - 12.0, center.y),
                Pos2::new(center.x + 12.0, center.y),
            ],
            Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 40)),
        );
        painter.line_segment(
            [
                Pos2::new(center.x, center.y - 12.0),
                Pos2::new(center.x, center.y + 12.0),
            ],
            Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 40)),
        );

        // Chữ Placeholder khi chưa load video texture
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            "⚡ DARKWORKS VIEWPORT\nSẵn sàng nhận wgpu::Texture từ crates/media",
            egui::FontId::proportional(14.0),
            Color32::from_rgb(100, 116, 139),
        );

        ui.add_space(8.0);

        // Thanh điều khiển phát (Transport Bar)
        ui.horizontal(|ui| {
            // Timecode hiện tại
            let current_tc = playhead.to_timecode(fps);
            let total_tc = duration.to_timecode(fps);

            ui.label(
                RichText::new(current_tc)
                    .monospace()
                    .strong()
                    .size(15.0)
                    .color(Color32::from_rgb(52, 224, 249)),
            );
            ui.label(
                RichText::new(format!("/ {total_tc}"))
                    .monospace()
                    .size(13.0)
                    .color(Color32::from_rgb(148, 163, 184)),
            );

            ui.add_space(16.0);

            // Nút Tua đầu
            if ui.button("⏮").clicked() {
                *playhead = Tick::ZERO;
            }

            // Lùi 1 khung hình
            if ui.button("◀").clicked() {
                let ticks_per_frame = (darkworks_schema::TICKS_PER_SECOND as f64 / fps) as u64;
                *playhead = Tick(playhead.0.saturating_sub(ticks_per_frame));
            }

            // Nút Play / Pause (phím cách)
            let play_label = if *is_playing { "⏸ Pause" } else { "▶ Play" };
            if ui.button(RichText::new(play_label).strong()).clicked() {
                *is_playing = !*is_playing;
            }

            // Tiến 1 khung hình
            if ui.button("▶").clicked() {
                let ticks_per_frame = (darkworks_schema::TICKS_PER_SECOND as f64 / fps) as u64;
                *playhead = Tick((playhead.0 + ticks_per_frame).min(duration.0));
            }

            // Tua cuối
            if ui.button("⏭").clicked() {
                *playhead = duration;
            }
        });
    });
}
