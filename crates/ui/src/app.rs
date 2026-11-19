//! Quản lý trạng thái ứng dụng chính `DarkworksApp` và vòng lặp render 60fps.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;
use crate::panels::{
    inspector::{self, InspectorTab, TransformState},
    menu_bar,
    source_bin::{self, MediaAsset},
    timeline, viewport,
};
use crate::theme;
use darkworks_media::{
    AudioPlayer, DEFAULT_CACHE_BUDGET_BYTES, FfmpegMediaSource, FrameCache, MediaSource,
    ProceduralMotion, VaApiStatus, probe_vaapi,
};
use darkworks_schema::{ClipSlice, Project, Tick, TimeRange};
use egui::{ColorImage, TextureHandle, TextureOptions};

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

    // Lõi Media, Cache & Phần cứng GPU
    pub frame_cache: Arc<FrameCache<(String, i64)>>,
    pub audio_player: AudioPlayer,
    pub active_source: Arc<dyn MediaSource>,
    pub vaapi_status: VaApiStatus,
    pub current_texture: Option<TextureHandle>,

    // Chỉ số hiệu năng hệ thống
    pub estimated_ram_mb: usize,
    pub frame_counter: u64,
    pub last_time_sec: f64,
    pub current_fps: f32,
    pub last_playhead: Tick,
    pub audio_feed_head: Tick,
}

impl DarkworksApp {
    /// Khởi tạo ứng dụng với cấu hình mặc định.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Áp dụng bảng màu Sci-Fi Dark theme
        theme::apply_theme(&cc.egui_ctx);

        let mut project = Project::default();

        // Khởi tạo Frame Cache với trần 2GB RAM
        let frame_cache = Arc::new(FrameCache::new(DEFAULT_CACHE_BUDGET_BYTES));

        // 1. Khởi tạo nguồn phát media thực tế (1-hour Caribbean Paradise video kèm Background Prefetch)
        let paradise_path = std::path::Path::new("assets/samples/tropical_paradise_1hour.mp4");
        let active_source: Arc<dyn MediaSource> = if paradise_path.exists() {
            match FfmpegMediaSource::open(paradise_path, Arc::clone(&frame_cache)) {
                Ok(src) => Arc::new(src),
                Err(e) => {
                    log::warn!("Không mở được 1-hour video: {e}, fallback sang ProceduralMotion");
                    Arc::new(ProceduralMotion::new(1280, 720, 60, 60))
                }
            }
        } else {
            Arc::new(ProceduralMotion::new(1280, 720, 60, 60))
        };

        let total_duration = active_source.info().duration;

        // Nạp clip 1 tiếng thực tế vào timeline track V1
        if let Some(v1_track) = project.tracks.iter_mut().find(|t| t.id == "V1") {
            v1_track.clips.push(ClipSlice {
                id: "clip_sample_1hour".to_string(),
                name: "tropical_paradise_1hour.mp4".to_string(),
                media_path: Some("assets/samples/tropical_paradise_1hour.mp4".to_string()),
                timeline_range: TimeRange::new(Tick::ZERO, total_duration),
                source_range: TimeRange::new(Tick::ZERO, total_duration),
                speed: 1.0,
                is_muted: false,
            });
        }

        if let Some(canvas_track) = project.tracks.iter_mut().find(|t| t.id == "V2") {
            canvas_track.clips.push(ClipSlice {
                id: "canvas_intro_title".to_string(),
                name: "🅰️ Vello Karaoke Title: Caribbean Paradise 4K".to_string(),
                media_path: None,
                timeline_range: TimeRange::new(Tick::from_seconds(5), Tick::from_seconds(25)),
                source_range: TimeRange::default(),
                speed: 1.0,
                is_muted: false,
            });
        }

        let media_assets = vec![
            MediaAsset {
                name: "tropical_paradise_1hour.mp4".to_string(),
                path: "assets/samples/tropical_paradise_1hour.mp4".to_string(),
                duration_str: "01:00:00:00".to_string(),
                resolution: "640x360".to_string(),
                codec: "H.264 / AAC (1 Hour Tropical Paradise)".to_string(),
            },
            MediaAsset {
                name: "smpte_countdown.mp4".to_string(),
                path: "assets/samples/smpte_countdown.mp4".to_string(),
                duration_str: "00:00:30:00".to_string(),
                resolution: "1280x720".to_string(),
                codec: "H.264 / AAC (880Hz Tone)".to_string(),
            },
            MediaAsset {
                name: "c_major_chord.wav".to_string(),
                path: "assets/samples/c_major_chord.wav".to_string(),
                duration_str: "00:00:30:00".to_string(),
                resolution: "Audio 2ch".to_string(),
                codec: "PCM 44.1kHz Stereo".to_string(),
            },
        ];

        // Khởi tạo Audio Player
        let audio_player = match AudioPlayer::try_new() {
            Ok(p) => p,
            Err(e) => {
                log::warn!("Audio player fallback: {e}");
                AudioPlayer::dummy()
            }
        };

        // Thăm dò khả năng giải mã phần cứng VA-API trên Linux
        let vaapi_status = probe_vaapi();

        Self {
            project,
            playhead: Tick::ZERO,
            duration: total_duration,
            is_playing: false,
            zoom_px_per_sec: 2.0,
            active_inspector_tab: InspectorTab::Properties,
            transform_state: TransformState::default(),
            selected_clip_id: Some("clip_sample_1hour".to_string()),
            media_assets,
            search_query: String::new(),

            frame_cache,
            audio_player,
            active_source,
            vaapi_status,
            current_texture: None,

            estimated_ram_mb: 48,
            frame_counter: 0,
            last_time_sec: 0.0,
            current_fps: 60.0,
            last_playhead: Tick::ZERO,
            audio_feed_head: Tick::ZERO,
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
                self.audio_player.seek(self.playhead);
                self.audio_feed_head = self.playhead;
                self.active_source.seek(self.playhead);
                self.last_playhead = self.playhead;
            }

            // Mũi tên phải: Tiến 1 frame
            if i.key_pressed(egui::Key::ArrowRight) {
                let ticks_per_frame = (darkworks_schema::TICKS_PER_SECOND as f64 / self.project.settings.fps) as u64;
                self.playhead = Tick((self.playhead.0 + ticks_per_frame).min(self.duration.0));
                self.audio_player.seek(self.playhead);
                self.audio_feed_head = self.playhead;
                self.active_source.seek(self.playhead);
                self.last_playhead = self.playhead;
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

    /// Cập nhật nhịp thời gian phát khi đang Play và đồng bộ âm thanh.
    fn update_playback(&mut self, ctx: &egui::Context) {
        // 1. Đồng bộ khi người dùng Tua (Scrubbing / Seeking) từ UI
        if self.playhead != self.last_playhead {
            self.audio_player.seek(self.playhead);
            self.audio_feed_head = self.playhead;
            self.active_source.seek(self.playhead);
            self.last_playhead = self.playhead;
        }

        // 2. Đồng bộ trạng thái Play/Pause giữa UI và Card âm thanh
        if self.is_playing != self.audio_player.is_playing() {
            if self.is_playing {
                self.audio_player.seek(self.playhead);
                self.audio_feed_head = self.playhead;
                self.audio_player.play();

                // Pre-buffer 0.5 giây âm thanh (~24.000 samples) lập tức chống buffer underrun
                let pre_frames = (self.audio_player.sample_rate() as usize) / 2;
                if let Ok(audio_buf) = self.active_source.audio_samples(
                    self.audio_feed_head,
                    pre_frames,
                    self.audio_player.sample_rate(),
                ) {
                    self.audio_player.feed_samples(&audio_buf.samples);
                    let delta_ticks = (pre_frames as u64 * darkworks_schema::TICKS_PER_SECOND)
                        / self.audio_player.sample_rate().max(1) as u64;
                    self.audio_feed_head = Tick(self.audio_feed_head.0 + delta_ticks);
                }
            } else {
                self.audio_player.pause();
            }
        }

        // 3. Tiến trình phát thời gian thực 60fps (Audio Master Clock)
        if self.is_playing {
            ctx.request_repaint(); // Yêu cầu vẽ liên tục 60fps khi đang phát

            if self.audio_player.has_stream() {
                let audio_tick = self.audio_player.current_tick();
                if audio_tick > self.playhead {
                    self.playhead = audio_tick;
                }
            } else {
                let dt = ctx.input(|i| i.stable_dt) as f64;
                let tick_delta = (dt * darkworks_schema::TICKS_PER_SECOND as f64) as u64;
                self.playhead = Tick(self.playhead.0 + tick_delta);
            }

            if self.playhead >= self.duration {
                self.playhead = self.duration;
                self.is_playing = false;
                self.audio_player.pause();
            }
            self.last_playhead = self.playhead;

            // 4. Luôn duy trì mức đệm âm thanh an toàn phía trước (khoảng 0.5 - 1.0 giây)
            // Lấy mẫu tịnh tiến từ audio_feed_head để bảo đảm tính liên tục của pha sóng âm
            let current_buf = self.audio_player.buffer_len();
            let target_buf = (self.audio_player.sample_rate() as usize) * 2; // ~1s buffer
            if current_buf < target_buf && self.audio_feed_head < self.duration {
                let needed_frames = (target_buf - current_buf) / 2;
                if needed_frames > 0 {
                    if let Ok(audio_buf) = self.active_source.audio_samples(
                        self.audio_feed_head,
                        needed_frames,
                        self.audio_player.sample_rate(),
                    ) {
                        self.audio_player.feed_samples(&audio_buf.samples);
                        let delta_ticks = (needed_frames as u64 * darkworks_schema::TICKS_PER_SECOND)
                            / self.audio_player.sample_rate().max(1) as u64;
                        self.audio_feed_head = Tick(self.audio_feed_head.0 + delta_ticks);
                    }
                }
            }
        }

        // Cập nhật thống kê RAM từ Ring-Buffer cache
        let cache_bytes = self.frame_cache.bytes();
        self.estimated_ram_mb = 48 + (cache_bytes / (1024 * 1024));

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

    /// Lấy hoặc nạp khung hình video vào Viewport qua Ring-Buffer Cache
    fn update_viewport_texture(&mut self, ctx: &egui::Context) {
        let fps = self.project.settings.fps;
        let frame_idx = (self.playhead.as_seconds_f64() * fps).round() as i64;
        let key = (self.active_source.info().id.clone(), frame_idx);

        // Báo cho Worker Thread giải mã ngầm biết vị trí playhead để duy trì prefetch đón đầu
        self.active_source.notify_playhead(self.playhead);

        let video_frame = if let Some(cached) = self.frame_cache.get(&key) {
            cached
        } else if let Ok(new_frame) = self.active_source.video_frame(self.playhead) {
            self.frame_cache.insert(key, Arc::clone(&new_frame));
            new_frame
        } else {
            return;
        };

        // Chuyển pixel sang RGBA8 nạp vào egui ColorImage
        let rgba = video_frame.to_rgba8();
        let color_image = ColorImage::from_rgba_unmultiplied(
            [video_frame.width as usize, video_frame.height as usize],
            &rgba,
        );

        self.current_texture = Some(ctx.load_texture(
            "viewport_video_frame",
            color_image,
            TextureOptions::LINEAR,
        ));
    }
}

impl eframe::App for DarkworksApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let fps = self.project.settings.fps;
        self.handle_shortcuts(&ctx);
        self.update_playback(&ctx);
        self.update_viewport_texture(&ctx);

        // 1. Top Panel: Menu Bar & Status
        egui::Panel::top("top_menu_panel")
            .exact_size(32.0)
            .show(ui, |ui| {
                menu_bar::render(
                    ui,
                    &self.project.name,
                    self.estimated_ram_mb,
                    self.frame_cache.budget() / (1024 * 1024),
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
                if let Some(clicked_path) = source_bin::render(ui, &self.media_assets, &mut self.search_query) {
                    let path = std::path::Path::new(&clicked_path);
                    if path.exists() {
                        if let Ok(new_src) = FfmpegMediaSource::open(path, Arc::clone(&self.frame_cache)) {
                            self.duration = new_src.info().duration;
                            self.active_source = Arc::new(new_src);
                            self.playhead = Tick::ZERO;
                            self.audio_feed_head = Tick::ZERO;
                            self.audio_player.seek(Tick::ZERO);
                            self.frame_cache.clear();
                        }
                    }
                }
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
        let vaapi_badge = if self.vaapi_status.available {
            Some("⚡ VA-API: Hardware Accelerated")
        } else {
            Some("⚙ CPU Fallback")
        };

        let stats = self.frame_cache.stats();
        let cache_str = format!(
            "RAM: {}MB / {}MB | Hits: {}",
            stats.bytes / (1024 * 1024),
            stats.budget / (1024 * 1024),
            stats.hits
        );

        egui::CentralPanel::default().show(ui, |ui| {
            viewport::render(
                ui,
                viewport::ViewportParams {
                    playhead: &mut self.playhead,
                    duration: self.duration,
                    is_playing: &mut self.is_playing,
                    fps,
                    video_texture: self.current_texture.as_ref(),
                    vaapi_badge,
                    cache_badge: Some(&cache_str),
                },
            );
        });
    }
}
