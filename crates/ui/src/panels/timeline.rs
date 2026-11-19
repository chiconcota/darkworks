//! Trục thời gian đa Track (Timeline NLE).

#![deny(clippy::unwrap_used, clippy::expect_used)]

use darkworks_schema::{Project, Tick, TrackKind};
use egui::{Color32, CornerRadius, Pos2, Rect, RichText, Sense, Stroke, Ui, Vec2};

/// Vẽ phân vùng Timeline ở nửa dưới màn hình.
pub fn render(
    ui: &mut Ui,
    project: &mut Project,
    playhead: &mut Tick,
    duration: Tick,
    zoom_px_per_sec: &mut f32,
    selected_clip_id: &mut Option<String>,
    _fps: f64,
) {
    ui.vertical(|ui| {
        // Thanh công cụ nhanh Timeline (Quick Action Toolbar)
        ui.horizontal(|ui| {
            ui.label(RichText::new("◷ TIMELINE").strong().size(13.0));
            ui.separator();

            if ui.button("✂ Split (S)").clicked() {
                log::info!("Thực hiện lệnh Split tại playhead: {}", playhead);
            }

            if ui.button("⚡ Auto Cut Silence (Silero VAD)").clicked() {
                log::info!("Kích hoạt AI Auto Cut Silence");
            }

            if ui.button("¶ Auto Subtitle (Whisper)").clicked() {
                log::info!("Kích hoạt AI Auto Subtitle");
            }

            if ui.button("✕ Ripple Delete (X)").clicked() {
                log::info!("Thực hiện lệnh Ripple Delete");
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add(
                    egui::Slider::new(zoom_px_per_sec, 10.0..=300.0)
                        .text("⚲ Zoom")
                        .suffix(" px/s"),
                );
            });
        });
        ui.separator();

        // Khu vực vẽ Tracks & Ruler cuộn ngang
        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let track_header_width = 160.0;
                let total_seconds = duration.as_seconds_f64().max(60.0);
                let timeline_content_width = (total_seconds as f32) * (*zoom_px_per_sec);
                let total_width = track_header_width + timeline_content_width;

                let row_height = 36.0;
                let ruler_height = 24.0;
                let total_height = ruler_height + (project.tracks.len() as f32) * row_height + 20.0;

                let (rect, response) = ui.allocate_exact_size(
                    Vec2::new(total_width.max(ui.available_width()), total_height),
                    Sense::click_and_drag(),
                );

                let painter = ui.painter();

                // 1. Vẽ nền Timeline
                painter.rect_filled(rect, CornerRadius::ZERO, Color32::from_rgb(12, 16, 23));

                // 2. Vẽ Ruler thời gian (Thước đo)
                let ruler_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), ruler_height));
                painter.rect_filled(ruler_rect, CornerRadius::ZERO, Color32::from_rgb(18, 24, 34));
                painter.line_segment(
                    [ruler_rect.left_bottom(), ruler_rect.right_bottom()],
                    Stroke::new(1.0, Color32::from_rgb(30, 41, 59)),
                );

                // Điểm vạch thời gian trên Ruler
                let start_x = rect.min.x + track_header_width;
                let step_seconds = if *zoom_px_per_sec > 150.0 {
                    1.0
                } else if *zoom_px_per_sec > 60.0 {
                    5.0
                } else {
                    10.0
                };

                let mut current_sec = 0.0;
                while current_sec <= total_seconds {
                    let mark_x = start_x + (current_sec as f32) * (*zoom_px_per_sec);
                    if mark_x > rect.max.x {
                        break;
                    }

                    // Vạch chia
                    painter.line_segment(
                        [Pos2::new(mark_x, rect.min.y + 12.0), Pos2::new(mark_x, rect.min.y + ruler_height)],
                        Stroke::new(1.0, Color32::from_rgb(71, 85, 105)),
                    );

                    // Nhãn thời gian
                    let mins = (current_sec as u32) / 60;
                    let secs = (current_sec as u32) % 60;
                    painter.text(
                        Pos2::new(mark_x + 3.0, rect.min.y + 3.0),
                        egui::Align2::LEFT_TOP,
                        format!("{mins:02}:{secs:02}"),
                        egui::FontId::monospace(10.0),
                        Color32::from_rgb(148, 163, 184),
                    );

                    current_sec += step_seconds;
                }

                // 3. Xử lý click/kéo Playhead trên Ruler hoặc nội dung Timeline
                if response.dragged() || response.clicked() {
                    if let Some(mouse_pos) = response.interact_pointer_pos() {
                        if mouse_pos.x >= start_x {
                            let click_offset_x = (mouse_pos.x - start_x).max(0.0);
                            let target_seconds = click_offset_x / (*zoom_px_per_sec);
                            let target_tick = (target_seconds as f64 * darkworks_schema::TICKS_PER_SECOND as f64) as u64;
                            *playhead = Tick(target_tick.min(duration.0));
                        }
                    }
                }

                // 4. Vẽ từng Track và các Clip bên trong
                for (i, track) in project.tracks.iter().enumerate() {
                    let track_y = rect.min.y + ruler_height + (i as f32) * row_height;
                    let track_rect = Rect::from_min_size(
                        Pos2::new(rect.min.x, track_y),
                        Vec2::new(rect.width(), row_height),
                    );

                    // Nền track xen kẽ
                    let bg_color = if i % 2 == 0 {
                        Color32::from_rgb(15, 20, 28)
                    } else {
                        Color32::from_rgb(13, 17, 24)
                    };
                    painter.rect_filled(track_rect, CornerRadius::ZERO, bg_color);
                    painter.line_segment(
                        [track_rect.left_bottom(), track_rect.right_bottom()],
                        Stroke::new(1.0, Color32::from_rgb(25, 34, 48)),
                    );

                    // Tiêu đề Track (Header) bên trái
                    let header_rect = Rect::from_min_size(
                        Pos2::new(rect.min.x, track_y),
                        Vec2::new(track_header_width, row_height),
                    );
                    painter.rect_filled(header_rect, CornerRadius::ZERO, Color32::from_rgb(20, 28, 40));
                    painter.line_segment(
                        [header_rect.right_top(), header_rect.right_bottom()],
                        Stroke::new(1.0, Color32::from_rgb(30, 41, 59)),
                    );

                    // Màu đại diện của Track Kind
                    let kind_color = match track.kind {
                        TrackKind::Canvas => Color32::from_rgb(56, 189, 248),
                        TrackKind::Video => Color32::from_rgb(59, 130, 246),
                        TrackKind::Audio => Color32::from_rgb(16, 185, 129),
                    };
                    painter.rect_filled(
                        Rect::from_min_size(header_rect.min, Vec2::new(4.0, row_height)),
                        CornerRadius::ZERO,
                        kind_color,
                    );

                    // Tên Track
                    painter.text(
                        Pos2::new(header_rect.min.x + 10.0, header_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        &track.name,
                        egui::FontId::proportional(12.0),
                        Color32::from_rgb(226, 232, 240),
                    );

                    // Vẽ các Clips nằm trên track này
                    for clip in &track.clips {
                        let clip_start_sec = clip.timeline_range.start.as_seconds_f64() as f32;
                        let clip_dur_sec = clip.timeline_range.duration.as_seconds_f64() as f32;

                        let clip_x = start_x + clip_start_sec * (*zoom_px_per_sec);
                        let clip_w = (clip_dur_sec * (*zoom_px_per_sec)).max(8.0);

                        let clip_rect = Rect::from_min_size(
                            Pos2::new(clip_x, track_y + 3.0),
                            Vec2::new(clip_w, row_height - 6.0),
                        );

                        let is_selected = selected_clip_id.as_deref() == Some(&clip.id);
                        let clip_bg = if is_selected {
                            Color32::from_rgb(37, 99, 235) // Xanh sáng khi chọn
                        } else {
                            Color32::from_rgb(30, 58, 138)
                        };

                        let stroke_color = if is_selected {
                            Color32::from_rgb(52, 224, 249)
                        } else {
                            Color32::from_rgb(60, 130, 246)
                        };

                        painter.rect_filled(clip_rect, CornerRadius::same(3), clip_bg);
                        painter.rect_stroke(
                            clip_rect,
                            CornerRadius::same(3),
                            Stroke::new(1.0, stroke_color),
                            egui::StrokeKind::Inside,
                        );

                        // Tên clip
                        painter.text(
                            Pos2::new(clip_rect.min.x + 6.0, clip_rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            &clip.name,
                            egui::FontId::proportional(11.0),
                            Color32::WHITE,
                        );
                    }
                }

                // 5. Vẽ Playhead (Đường chỉ đỏ quét dọc Timeline)
                let playhead_sec = playhead.as_seconds_f64() as f32;
                let playhead_x = start_x + playhead_sec * (*zoom_px_per_sec);

                // Đường kẻ dọc
                painter.line_segment(
                    [Pos2::new(playhead_x, rect.min.y), Pos2::new(playhead_x, rect.max.y)],
                    Stroke::new(2.0, Color32::from_rgb(239, 68, 68)),
                );

                // Đầu playhead (Hình tam giác/lá cờ trên ruler)
                let head_top = rect.min.y;
                let head_points = vec![
                    Pos2::new(playhead_x - 6.0, head_top),
                    Pos2::new(playhead_x + 6.0, head_top),
                    Pos2::new(playhead_x + 6.0, head_top + 10.0),
                    Pos2::new(playhead_x, head_top + 16.0),
                    Pos2::new(playhead_x - 6.0, head_top + 10.0),
                ];
                painter.add(egui::Shape::convex_polygon(
                    head_points,
                    Color32::from_rgb(239, 68, 68),
                    Stroke::NONE,
                ));
            });
    });
}
