use std::sync::Arc;
use darkworks_schema::Tick;
use serde::{Deserialize, Serialize};
use crate::error::MediaResult;
use crate::frame::{AudioBuffer, PixelFormat, VideoFrame};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaKind {
    Video,
    Audio,
    Synthetic,
    StillImage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoTrackInfo {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub codec: String,
    pub format: PixelFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTrackInfo {
    pub channels: u16,
    pub sample_rate: u32,
    pub codec: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaInfo {
    pub id: String,
    pub name: String,
    pub kind: MediaKind,
    pub duration: Tick,
    pub video: Option<VideoTrackInfo>,
    pub audio: Option<AudioTrackInfo>,
}

impl MediaInfo {
    pub fn has_video(&self) -> bool {
        self.video.is_some()
    }

    pub fn has_audio(&self) -> bool {
        self.audio.is_some()
    }
}

/// Giao diện nguồn dữ liệu Media (File, Video giả lập, Audio track)
pub trait MediaSource: Send + Sync {
    /// Thông tin metadata tổng quan của media
    fn info(&self) -> &MediaInfo;

    /// Trích xuất khung hình video tương ứng với mốc thời gian `time`
    fn video_frame(&self, time: Tick) -> MediaResult<Arc<VideoFrame>>;

    /// Trích xuất chuỗi mẫu audio bắt đầu từ mốc thời gian `start`
    fn audio_samples(&self, start: Tick, count: usize, sample_rate: u32) -> MediaResult<AudioBuffer>;

    /// Thông báo mốc thời gian hiện tại của Playhead để nạp trước (Prefetch)
    fn notify_playhead(&self, _time: Tick) {}

    /// Yêu cầu chuyển nhanh vị trí phát (Seek)
    fn seek(&self, _time: Tick) {}
}
