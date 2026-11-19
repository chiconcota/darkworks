//! # Darkworks Media Engine
//!
//! Tầng lõi giải mã đa phương tiện, phát hiện phần cứng VA-API trên Linux,
//! quản lý bộ nhớ đệm Ring-Buffer LRU giới hạn cứng < 2GB RAM, và đồng bộ A/V Sync.

pub mod audio_player;
pub mod cache;
pub mod error;
pub mod ffmpeg_source;
pub mod file_source;
pub mod frame;
pub mod generators;
pub mod source;
pub mod vaapi;
pub mod waveform;

pub use audio_player::AudioPlayer;
pub use cache::{DEFAULT_CACHE_BUDGET_BYTES, FrameCache, MAX_CACHE_BUDGET_BYTES};
pub use error::{MediaError, MediaResult};
pub use ffmpeg_source::FfmpegMediaSource;
pub use file_source::FileMediaSource;
pub use frame::{AudioBuffer, PixelFormat, VideoFrame};
pub use generators::{ProceduralMotion, SmpteBars};
pub use source::{AudioTrackInfo, MediaInfo, MediaKind, MediaSource, VideoTrackInfo};
pub use vaapi::{VaApiStatus, probe_vaapi};
pub use waveform::AudioWaveform;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use darkworks_schema::Tick;

    #[test]
    fn test_frame_cache_budget_eviction() {
        // Tạo cache với ngân sách nhỏ (3MB)
        let cache: FrameCache<(String, i64)> = FrameCache::new(3 * 1024 * 1024);
        let w = 500;
        let h = 500;
        // Mỗi frame RGBA8 500x500 = 1MB (1_000_000 bytes)
        for i in 0..5i64 {
            let data = vec![i as u8; (w * h * 4) as usize];
            let frame = Arc::new(VideoFrame::rgba8(w, h, data, Tick(i as u64), Tick::SECOND));
            cache.insert(("src".to_string(), i), frame);
        }

        let stats = cache.stats();
        // Dung lượng cache phải luôn được kiềm tỏa dưới hoặc xấp xỉ ngân sách 3MB
        assert!(stats.bytes <= 3 * 1024 * 1024);
        // Các frame cũ (0, 1) phải bị dọn sạch theo cơ chế LRU
        assert!(cache.get(&("src".to_string(), 0)).is_none());
        assert!(cache.get(&("src".to_string(), 4)).is_some());
    }

    #[test]
    fn test_smpte_generator_frames_and_audio() {
        let generator = SmpteBars::new(1280, 720, 30, 60);
        assert_eq!(generator.info().kind, MediaKind::Synthetic);
        assert!(generator.info().has_video());
        assert!(generator.info().has_audio());

        // Lấy frame tại mốc 2 giây
        let frame = generator.video_frame(Tick::from_seconds_f64(2.0));
        assert!(frame.is_ok());
        if let Ok(f) = frame {
            assert_eq!(f.width, 1280);
            assert_eq!(f.height, 720);
            assert_eq!(f.pts, Tick::from_seconds_f64(2.0));
        }

        // Lấy audio tone 1kHz
        let audio = generator.audio_samples(Tick::ZERO, 4800, 48000);
        assert!(audio.is_ok());
        if let Ok(a) = audio {
            assert_eq!(a.channels, 2);
            assert_eq!(a.samples.len(), 4800 * 2);
        }
    }

    #[test]
    fn test_procedural_motion_generator() {
        let motion = ProceduralMotion::new(640, 360, 60, 10);
        let frame = motion.video_frame(Tick::from_seconds_f64(0.5));
        assert!(frame.is_ok());
        if let Ok(f) = frame {
            assert_eq!(f.width, 640);
            assert_eq!(f.height, 360);
        }
    }

    #[test]
    fn test_audio_waveform_extraction() {
        let mut samples = Vec::new();
        for i in 0..1000 {
            let s = ((i as f32) / 1000.0).sin();
            samples.push(s);
            samples.push(s);
        }
        let buf = AudioBuffer::new(2, 48000, samples, Tick::from_seconds_f64(1.0));
        let wf = AudioWaveform::from_audio_buffer(&buf, 10);
        assert_eq!(wf.peaks.len(), 10);
        assert_eq!(wf.rms.len(), 10);
    }

    #[test]
    fn test_ffmpeg_1hour_video_source() {
        let sample_path = std::path::Path::new("../../assets/samples/tropical_paradise_1hour.mp4");
        if !sample_path.exists() {
            return;
        }

        let cache = Arc::new(FrameCache::new(DEFAULT_CACHE_BUDGET_BYTES));
        let source = FfmpegMediaSource::open(sample_path, cache);
        assert!(source.is_ok());
        if let Ok(src) = source {
            let info = src.info();
            assert_eq!(info.kind, MediaKind::Video);
            assert!(info.duration.as_seconds_f64() >= 3600.0);
            assert_eq!(info.video.as_ref().map(|v| v.width), Some(640));
            assert_eq!(info.video.as_ref().map(|v| v.height), Some(360));

            // Trích xuất 1 khung hình tại phút thứ 30 (1800s)
            let frame = src.video_frame(Tick::from_seconds(1800));
            assert!(frame.is_ok());
            if let Ok(f) = frame {
                assert_eq!(f.width, 640);
                assert_eq!(f.height, 360);
                assert_eq!(f.data.len(), 640 * 360 * 4);
            }

            // Trích xuất audio 1000 mẫu tại phút thứ 30
            let audio = src.audio_samples(Tick::from_seconds(1800), 1000, 48000);
            assert!(audio.is_ok());
            if let Ok(a) = audio {
                assert_eq!(a.channels, 2);
                assert_eq!(a.sample_rate, 48000);
                assert_eq!(a.samples.len(), 2000);
            }
        }
    }
}
