#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;
use darkworks_media::{AudioPlayer, FrameCache, MediaSource, ProceduralMotion, SmpteBars, VideoFrame};
use darkworks_schema::Tick;

#[test]
fn test_e2e_simulated_playback_loop() {
    // Kịch bản E2E: Mô phỏng phát video 5 giây ở 60fps (300 frames)
    // Dùng ngân sách test 512MB để kiểm tra khả năng kích hoạt dọn dẹp LRU eviction của FrameCache
    let test_budget = 512 * 1024 * 1024;
    let generator = ProceduralMotion::new(1280, 720, 60, 10);
    let cache: FrameCache<(String, i64)> = FrameCache::new(test_budget);
    let audio_player = AudioPlayer::dummy();

    let fps = 60.0;
    let ticks_per_frame = (darkworks_schema::TICKS_PER_SECOND as f64 / fps) as u64;
    let total_frames = 300;

    audio_player.play();

    for f in 0..total_frames {
        let current_tick = Tick(f * ticks_per_frame);
        let key = (generator.info().id.clone(), f as i64);

        // 1. Lấy hoặc nạp frame vào Cache
        let frame = if let Some(cached) = cache.get(&key) {
            cached
        } else {
            let new_frame = generator
                .video_frame(current_tick)
                .map_err(|e| format!("{e}"))
                .unwrap_or_else(|e| panic!("{}", e));
            cache.insert(key.clone(), Arc::clone(&new_frame));
            new_frame
        };

        assert_eq!(frame.width, 1280);
        assert_eq!(frame.height, 720);

        // 2. Bơm audio tương ứng 1 frame (800 samples ở 48kHz)
        let samples_per_frame = (48000.0 / fps) as usize;
        let audio = generator
            .audio_samples(current_tick, samples_per_frame, 48000)
            .map_err(|e| format!("{e}"))
            .unwrap_or_else(|e| panic!("{}", e));
        audio_player.feed_samples(&audio.samples);
    }

    // 3. Kiểm tra thống kê bộ đệm Cache: Ngân sách được bảo vệ nghiêm ngặt
    let stats = cache.stats();
    assert!(stats.bytes > 0);
    assert!(stats.bytes <= test_budget);
    // 300 frames 720p (~1.1GB) phải được LRU dọn dẹp để chỉ giữ ~139 frames trong 512MB RAM
    assert!(stats.entry_count <= 145);

    // Frame cuối cùng (299) phải còn trong cache
    let latest_frame = cache.get(&(generator.info().id.clone(), 299));
    assert!(latest_frame.is_some());

    // Frame đầu tiên (0) phải đã bị LRU giải phóng an toàn
    let evicted_frame = cache.get(&(generator.info().id.clone(), 0));
    assert!(evicted_frame.is_none());
}

#[test]
fn test_e2e_stress_memory_budget_enforcement() {
    // Kịch bản E2E Stress Test: Ngân sách bộ nhớ bị ép xuống cực nhỏ (16MB)
    // Đẩy vào 50 frames 1080p RGBA8 (Mỗi frame ~8.3MB, tổng ~415MB dữ liệu)
    let small_budget = 16 * 1024 * 1024; // 16 MB
    let cache: FrameCache<i32> = FrameCache::new(small_budget);

    let w = 1920;
    let h = 1080;
    let frame_bytes = (w * h * 4) as usize; // ~8.29 MB

    for i in 0..50 {
        let fake_pixels = vec![i as u8; frame_bytes];
        let frame = Arc::new(VideoFrame::rgba8(w, h, fake_pixels, Tick(i as u64), Tick::SECOND));
        cache.insert(i, frame);

        let current_bytes = cache.bytes();
        // CẤM TUYỆT ĐỐI vượt quá ngân sách bộ nhớ đã định
        assert!(
            current_bytes <= small_budget + frame_bytes,
            "Cache vượt quá trần bộ nhớ cho phép! Hiện tại: {} MB, Ngân sách: {} MB",
            current_bytes / (1024 * 1024),
            small_budget / (1024 * 1024)
        );
    }

    let stats = cache.stats();
    // Cache chỉ được lưu tối đa 1-2 frames 1080p trong ngân sách 16MB
    assert!(stats.entry_count <= 2);
    // Các frames cũ từ 0 đến 45 bắt buộc phải bị giải phóng hoàn toàn
    for old_idx in 0..45 {
        assert!(cache.get(&old_idx).is_none());
    }
}

#[test]
fn test_e2e_smpte_calibration_integrity() {
    // Kịch bản E2E: Kiểm tra tính toàn vẹn của tín hiệu SMPTE Color Bars
    let bars = SmpteBars::new(1920, 1080, 30, 60);
    assert_eq!(bars.info().duration, Tick::from_seconds(60));

    // Lấy frame ở giữa timeline
    let frame = bars.video_frame(Tick::from_seconds(30)).unwrap_or_else(|e| panic!("{}", e));
    assert_eq!(frame.width, 1920);
    assert_eq!(frame.height, 1080);
    let rgba = frame.to_rgba8();
    assert_eq!(rgba.len(), 1920 * 1080 * 4);

    // Lấy 48000 mẫu âm thanh (1 giây đầy đủ)
    let audio = bars.audio_samples(Tick::ZERO, 48000, 48000).unwrap_or_else(|e| panic!("{}", e));
    assert_eq!(audio.channels, 2);
    assert_eq!(audio.samples.len(), 48000 * 2);

    // Đảm bảo âm thanh chuẩn có biên độ không bị clip (> 0.0 và <= 0.3)
    let max_sample = audio.samples.iter().fold(0.0f32, |m, &s| m.max(s.abs()));
    assert!(max_sample > 0.20 && max_sample <= 0.26);
}
