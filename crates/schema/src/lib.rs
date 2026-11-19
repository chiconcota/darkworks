//! # Darkworks Schema Crate
//!
//! Định nghĩa cấu trúc dữ liệu cốt lõi của Darkworks:
//! - [`Tick`]: Đơn vị thời gian nguyên thủy không trôi khung hình (`1s = 254_016_000_000 ticks`).
//! - [`TimeRange`]: Khoảng thời gian trên Timeline.
//! - [`ClipSlice`]: Một phân đoạn clip trên Track.
//! - [`Project`]: Trạng thái hoàn chỉnh của project lưu dưới dạng `project.json`.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Sub};
use thiserror::Error;

/// Tần số Tick chuẩn trên 1 giây: chia hết cho 24, 25, 30, 48, 50, 60, 120, 23.976, 29.97, 59.94 fps,
/// và các tần số lấy mẫu âm thanh phổ biến 44.1kHz, 48kHz, 96kHz, 192kHz.
pub const TICKS_PER_SECOND: u64 = 254_016_000_000;

/// Lỗi liên quan đến tính toán và phân giải thời gian.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SchemaError {
    #[error("Tần số khung hình phải lớn hơn 0")]
    InvalidFps,
    #[error("Phép trừ thời gian bị tràn số dưới 0")]
    Underflow,
    #[error("Dữ liệu project.json không hợp lệ: {0}")]
    JsonError(String),
}

/// Đơn vị thời gian nguyên thủy của Darkworks.
/// Tránh hoàn toàn sai số dấu phẩy động của `f64`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct Tick(pub u64);

impl Tick {
    pub const ZERO: Self = Self(0);
    pub const SECOND: Self = Self(TICKS_PER_SECOND);

    /// Tạo Tick từ số nguyên giây.
    #[must_use]
    pub const fn from_seconds(secs: u64) -> Self {
        Self(secs.saturating_mul(TICKS_PER_SECOND))
    }

    /// Tạo Tick từ số thực giây (dành cho tính toán thời gian media).
    #[must_use]
    pub fn from_seconds_f64(secs: f64) -> Self {
        if secs <= 0.0 {
            Self::ZERO
        } else {
            Self((secs * TICKS_PER_SECOND as f64).round() as u64)
        }
    }

    /// Tạo Tick từ chỉ số khung hình (frame index) và tốc độ khung hình (fps).
    pub fn from_frame(frame: u64, fps: f64) -> Result<Self, SchemaError> {
        if fps <= 0.0 {
            return Err(SchemaError::InvalidFps);
        }
        let ticks_per_frame = (TICKS_PER_SECOND as f64 / fps).round() as u64;
        Ok(Self(frame.saturating_mul(ticks_per_frame)))
    }

    /// Chuyển đổi sang số thực giây (dành riêng cho hiển thị UI, không dùng cho logic cắt ghép).
    #[must_use]
    pub fn as_seconds_f64(self) -> f64 {
        self.0 as f64 / TICKS_PER_SECOND as f64
    }

    /// Chuyển đổi sang chỉ số khung hình nguyên thủy.
    pub fn as_frame(self, fps: f64) -> Result<u64, SchemaError> {
        if fps <= 0.0 {
            return Err(SchemaError::InvalidFps);
        }
        let ticks_per_frame = (TICKS_PER_SECOND as f64 / fps).round() as u64;
        if ticks_per_frame == 0 {
            return Err(SchemaError::InvalidFps);
        }
        Ok(self.0 / ticks_per_frame)
    }

    /// Định dạng chuỗi Timecode chuẩn `HH:MM:SS:FF`.
    pub fn to_timecode(self, fps: f64) -> String {
        let total_frames = self.as_frame(fps).unwrap_or(0);
        let fps_int = fps.round().max(1.0) as u64;
        let frames = total_frames % fps_int;
        let total_seconds = total_frames / fps_int;
        let seconds = total_seconds % 60;
        let minutes = (total_seconds / 60) % 60;
        let hours = total_seconds / 3600;

        format!("{hours:02}:{minutes:02}:{seconds:02}:{frames:02}")
    }
}

impl Add for Tick {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_add(rhs.0))
    }
}

impl Sub for Tick {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_sub(rhs.0))
    }
}

impl fmt::Display for Tick {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}t ({:.3}s)", self.0, self.as_seconds_f64())
    }
}

/// Khoảng thời gian trên Timeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: Tick,
    pub duration: Tick,
}

impl TimeRange {
    #[must_use]
    pub const fn new(start: Tick, duration: Tick) -> Self {
        Self { start, duration }
    }

    #[must_use]
    pub fn end(self) -> Tick {
        self.start + self.duration
    }

    #[must_use]
    pub fn contains(self, tick: Tick) -> bool {
        tick >= self.start && tick < self.end()
    }
}

/// Loại track trên Timeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackKind {
    Video,
    Audio,
    Canvas,
}

/// Một phân đoạn Clip trên Timeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClipSlice {
    pub id: String,
    pub name: String,
    pub media_path: Option<String>,
    pub timeline_range: TimeRange,
    pub source_range: TimeRange,
    pub speed: f32,
    pub is_muted: bool,
}

impl Default for ClipSlice {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            media_path: None,
            timeline_range: TimeRange::default(),
            source_range: TimeRange::default(),
            speed: 1.0,
            is_muted: false,
        }
    }
}

/// Một Track chứa danh sách các Clip.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub name: String,
    pub kind: TrackKind,
    pub is_locked: bool,
    pub is_hidden: bool,
    pub clips: Vec<ClipSlice>,
}

impl Track {
    pub fn new(id: impl Into<String>, name: impl Into<String>, kind: TrackKind) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            kind,
            is_locked: false,
            is_hidden: false,
            clips: Vec::new(),
        }
    }
}

/// Cấu hình độ phân giải và tốc độ khung hình của Project.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectSettings {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub sample_rate: u32,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            fps: 60.0,
            sample_rate: 48_000,
        }
    }
}

/// Trạng thái toàn vẹn của một dự án Darkworks (`project.json`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub version: u32,
    pub name: String,
    pub settings: ProjectSettings,
    pub tracks: Vec<Track>,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            version: 1,
            name: "Untitled Project".to_string(),
            settings: ProjectSettings::default(),
            tracks: vec![
                Track::new("V2", "Canvas 2D / Subtitles", TrackKind::Canvas),
                Track::new("V1", "Video 1", TrackKind::Video),
                Track::new("A1", "Dialogue / Audio 1", TrackKind::Audio),
                Track::new("A2", "Music / Sound FX", TrackKind::Audio),
            ],
        }
    }
}

impl Project {
    /// Xuất trạng thái ra chuỗi JSON đẹp mắt để lưu vào `project.json`.
    pub fn to_json_pretty(&self) -> Result<String, SchemaError> {
        serde_json::to_string_pretty(self).map_err(|e| SchemaError::JsonError(e.to_string()))
    }

    /// Nạp trạng thái từ chuỗi JSON (hỗ trợ Hot-Reload).
    pub fn from_json_str(json: &str) -> Result<Self, SchemaError> {
        serde_json::from_str(json).map_err(|e| SchemaError::JsonError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick_math() {
        let t1 = Tick::from_seconds(1);
        assert_eq!(t1.0, TICKS_PER_SECOND);
        assert!((t1.as_seconds_f64() - 1.0).abs() < 1e-6);

        let frame_60 = Tick::from_frame(60, 60.0);
        let frame_tick = match frame_60 {
            Ok(t) => t,
            Err(_) => Tick::ZERO,
        };
        assert_eq!(frame_tick.0, TICKS_PER_SECOND);
    }

    #[test]
    fn test_timecode_formatting() {
        let one_sec = Tick::from_seconds(1);
        let tc = one_sec.to_timecode(60.0);
        assert_eq!(tc, "00:00:01:00");
    }

    #[test]
    fn test_project_serialization() {
        let p = Project::default();
        let json_res = p.to_json_pretty();
        assert!(json_res.is_ok());
    }
}
