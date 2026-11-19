//! Quản lý trạng thái ứng dụng chính `DarkworksApp` và vòng lặp render 60fps.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use crate::panels::{
    inspector::{self, InspectorTab, TransformState},
    menu_bar,
    source_bin::{self, MediaAsset},
    timeline, viewport,
};
use crate::theme;
use darkworks_schema::{ClipSlice, Project, Tick, TimeRange};

/// Trạng thái ứng dụng desktop Darkworks.
pub struct DarkworksApp {
    pub project: Project,
    pub playhead: Tick,
    pub duration: Tick,
    pub is_playing: bool,
    pub zoom_px_per_sec: f32,
    pub active_inspector_tab: InspectorTab,
    pub transform_state: TransformState,
    pub selected_clip_id: Option<String>,
    pub media_assets: Vec<MediaAsset>,
    pub search_query: String,

    // Chỉ số hiệu năng hệ thống
    pub estimated_ram_mb: usize,
    pub frame_counter: u64,
    pub last_time_sec: f64,
    pub current_fps: f32,
}

impl DarkworksApp {
    /// Khởi tạo ứng dụng với cấu hình mặc định.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Áp dụng bảng màu Sci-Fi Dark theme
        theme::apply_theme(&cc.egui_ctx);

        let mut project = Project::default();
        let total_duration = Tick::from_seconds(60); // 1 phút mẫu ban đầu

        // Nạp vài clip mẫu trực quan vào timeline
        if let Some(v1_track) = project.tracks.iter_mut().find(|t| t.id == "V1") {
            v1_track.clips.push(ClipSlice {
                id: "clip_sample_1".to_string(),
                name: "NASA_Launch_Apollo11.mp4".to_string(),
                media_path: Some("/media/sample1.mp4".to_string()),
                timeline_range: TimeRange::new(Tick::ZERO, Tick::from_seconds(15)),
                source_range: TimeRange::new(Tick::ZERO, Tick::from_seconds(15)),
                speed: 1.0,
                is_muted: false,
            });
            v1_track.clips.push(ClipSlice {
                id: "clip_sample_2".to_string(),
                name: "ISS_Earth_Orbit_4K.mp4".to_string(),
                media_path: Some("/media/sample2.mp4".to_string()),
                timeline_range: TimeRange::new(Tick::from_seconds(15), Tick::from_seconds(30)),
                source_range: TimeRange::new(Tick::ZERO, Tick::from_seconds(30)),
                speed: 1.0,
                is_muted: false,
            });
        }

        if let Some(canvas_track) = project.tracks.iter_mut().find(|t| t.id == "V2") {
            canvas_track.clips.push(ClipSlice {
                id: "canvas_intro_title".to_string(),
                name: "🅰️ Vello Karaoke Title: Darkworks Intro".to_string(),
                media_path: None,
                timeline_range: TimeRange::new(Tick::from_seconds(2), Tick::from_seconds(10)),
                source_range: TimeRange::default(),
                speed: 1.0,
                is_muted: false,
            });
        }

        let media_assets = vec![
            MediaAsset {
                name: "NASA_Launch_Apollo11.mp4".to_string(),
                path: "/media/sample1.mp4".to_string(),
                duration_str: "00:15:00".to_string(),
                resolution: "1920x1080".to_string(),
                codec: "H.264 / AAC".to_string(),
            },
            MediaAsset {
                name: "ISS_Earth_Orbit_4K.mp4".to_string(),
                path: "/media/sample2.mp4".to_string(),
                duration_str: "01:00:00".to_string(),
                resolution: "3840x2160".to_string(),
                codec: "HEVC Main10".to_string(),
            },
        ];

        Self {
            project,
            playhead: Tick::ZERO,
            duration: total_duration,
            is_playing: false,
            zoom_px_per_sec: 40.0,
            active_inspector_tab: InspectorTab::Properties,
            transform_state: TransformState::default(),
            selected_clip_id: Some("clip_sample_1".to_string()),
            media_assets,
            search_query: String::new(),
            estimated_ram_mb: 148, // Mặc định RAM ban đầu < 200MB
            frame_counter: 0,
            last_time_sec: 0.0,
            current_fps: 60.0,
        }
    }

    /// Xử lý phím tắt nhanh toàn cục.
    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            // Spacebar: Play / Pause
            if i.key_pressed(egui::Key::Space) {
                self.is_playing = !self.is_playing;
            }

            // Mũi tên trái: Lùi 1 frame
            if i.key_pressed(egui::Key::ArrowLeft) {
                let ticks_per_frame = (darkworks_schema::TICKS_PER_SECOND as f64 / self.project.settings.fps) as u64;
                self.playhead = Tick(self.playhead.0.saturating_sub(ticks_per_frame));
            }

            // Mũi tên phải: Tiến 1 frame
            if i.key_pressed(egui::Key::ArrowRight) {
                let ticks_per_frame = (darkworks_schema::TICKS_PER_SECOND as f64 / self.project.settings.fps) as u64;
                self.playhead = Tick((self.playhead.0 + ticks_per_frame).min(self.duration.0));
            }

            // Phím S: Split tại playhead
            if i.key_pressed(egui::Key::S) {
                log::info!("Phím tắt S: Split tại playhead {}", self.playhead);
            }

            // Phím X: Ripple delete
            if i.key_pressed(egui::Key::X) {
                log::info!("Phím tắt X: Ripple delete");
            }
        });
    }

    /// Cập nhật nhịp thời gian phát khi đang Play.
    fn update_playback(&mut self, ctx: &egui::Context) {
        if self.is_playing {
            ctx.request_repaint(); // Yêu cầu vẽ liên tục 60fps khi đang phát
            let dt = ctx.input(|i| i.stable_dt) as f64;
            let tick_delta = (dt * darkworks_schema::TICKS_PER_SECOND as f64) as u64;
            let next_tick = self.playhead.0 + tick_delta;

            if next_tick >= self.duration.0 {
                self.playhead = self.duration;
                self.is_playing = false;
            } else {
                self.playhead = Tick(next_tick);
            }
        }

        // Tính FPS thực tế
        self.frame_counter += 1;
        let now = ctx.input(|i| i.time);
        if now - self.last_time_sec >= 0.5 {
            let elapsed = now - self.last_time_sec;
            self.current_fps = (self.frame_counter as f64 / elapsed) as f32;
            self.frame_counter = 0;
            self.last_time_sec = now;
        }
    }
}

impl eframe::App for DarkworksApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let fps = self.project.settings.fps;
        self.handle_shortcuts(&ctx);
        self.update_playback(&ctx);

        // 1. Top Panel: Menu Bar & Status
        egui::Panel::top("top_menu_panel")
            .exact_size(32.0)
            .show(ui, |ui| {
                menu_bar::render(
                    ui,
                    &self.project.name,
                    self.estimated_ram_mb,
                    self.current_fps,
                );
            });

        // 2. Bottom Panel: Timeline (Chiếm khoảng 35% chiều cao cửa sổ)
        egui::Panel::bottom("bottom_timeline_panel")
            .resizable(true)
            .min_size(180.0)
            .default_size(280.0)
            .show(ui, |ui| {
                timeline::render(
                    ui,
                    &mut self.project,
                    &mut self.playhead,
                    self.duration,
                    &mut self.zoom_px_per_sec,
                    &mut self.selected_clip_id,
                    fps,
                );
            });

        // 3. Left Panel: Source / Project Bin
        egui::Panel::left("left_source_bin_panel")
            .resizable(true)
            .min_size(200.0)
            .default_size(280.0)
            .show(ui, |ui| {
                source_bin::render(ui, &self.media_assets, &mut self.search_query);
            });

        // 4. Right Panel: Inspector & Node Graph DAG
        let selected_name = self.selected_clip_id.as_deref();
        egui::Panel::right("right_inspector_panel")
            .resizable(true)
            .min_size(240.0)
            .default_size(320.0)
            .show(ui, |ui| {
                inspector::render(
                    ui,
                    &mut self.active_inspector_tab,
                    &mut self.transform_state,
                    selected_name,
                );
            });

        // 5. Central Panel: Viewport 16:9
        egui::CentralPanel::default().show(ui, |ui| {
            viewport::render(
                ui,
                &mut self.playhead,
                self.duration,
                &mut self.is_playing,
                self.project.settings.fps,
            );
        });
    }
}
