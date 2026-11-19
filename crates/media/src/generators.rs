use std::sync::Arc;
use darkworks_schema::Tick;
use crate::error::MediaResult;
use crate::frame::{AudioBuffer, PixelFormat, VideoFrame};
use crate::source::{AudioTrackInfo, MediaInfo, MediaKind, MediaSource, VideoTrackInfo};

/// Máy phát mẫu thanh màu SMPTE Color Bars tiêu chuẩn (Rec.709) kèm âm chuẩn 1000Hz
pub struct SmpteBars {
    info: MediaInfo,
    cached_frame: Arc<VideoFrame>,
}

impl SmpteBars {
    pub fn new(width: u32, height: u32, fps: u32, duration_secs: u64) -> Self {
        let duration = Tick::from_seconds(duration_secs);
        let info = MediaInfo {
            id: "generator_smpte_bars".into(),
            name: "SMPTE Color Bars (1000Hz Tone)".into(),
            kind: MediaKind::Synthetic,
            duration,
            video: Some(VideoTrackInfo {
                width,
                height,
                fps,
                codec: "raw_rgba".into(),
                format: PixelFormat::Rgba8,
            }),
            audio: Some(AudioTrackInfo {
                channels: 2,
                sample_rate: 48_000,
                codec: "pcm_f32".into(),
            }),
        };

        let raw_pixels = generate_smpte_pixels(width, height);
        let frame = Arc::new(VideoFrame::rgba8(
            width,
            height,
            raw_pixels,
            Tick::ZERO,
            Tick::from_seconds_f64(1.0 / fps.max(1) as f64),
        ));

        Self {
            info,
            cached_frame: frame,
        }
    }
}

impl MediaSource for SmpteBars {
    fn info(&self) -> &MediaInfo {
        &self.info
    }

    fn video_frame(&self, time: Tick) -> MediaResult<Arc<VideoFrame>> {
        let mut f = (*self.cached_frame).clone();
        f.pts = time;
        Ok(Arc::new(f))
    }

    fn audio_samples(&self, start: Tick, count: usize, sample_rate: u32) -> MediaResult<AudioBuffer> {
        let mut samples = Vec::with_capacity(count * 2);
        let start_sec = start.as_seconds_f64();
        let freq = 1000.0; // 1kHz reference tone
        let amp = 0.25; // -12 dBFS

        for i in 0..count {
            let t = start_sec + (i as f64 / sample_rate as f64);
            let s = (2.0 * std::f64::consts::PI * freq * t).sin() as f32 * amp;
            samples.push(s); // Left
            samples.push(s); // Right
        }

        Ok(AudioBuffer::new(2, sample_rate, samples, start))
    }
}

/// Máy phát hoạt họa chuyển động giả lập (Procedural Motion) để kiểm tra 60fps & đệm khung hình
pub struct ProceduralMotion {
    info: MediaInfo,
}

impl ProceduralMotion {
    pub fn new(width: u32, height: u32, fps: u32, duration_secs: u64) -> Self {
        let duration = Tick::from_seconds(duration_secs);
        let info = MediaInfo {
            id: "generator_procedural".into(),
            name: "Procedural Motion 60fps Generator".into(),
            kind: MediaKind::Synthetic,
            duration,
            video: Some(VideoTrackInfo {
                width,
                height,
                fps,
                codec: "raw_rgba".into(),
                format: PixelFormat::Rgba8,
            }),
            audio: Some(AudioTrackInfo {
                channels: 2,
                sample_rate: 48_000,
                codec: "pcm_f32".into(),
            }),
        };

        Self { info }
    }
}

impl MediaSource for ProceduralMotion {
    fn info(&self) -> &MediaInfo {
        &self.info
    }

    fn video_frame(&self, time: Tick) -> MediaResult<Arc<VideoFrame>> {
        let w = self.info.video.as_ref().map(|v| v.width).unwrap_or(1920);
        let h = self.info.video.as_ref().map(|v| v.height).unwrap_or(1080);
        let t = time.as_seconds_f64();

        let mut pixels = vec![0u8; (w * h * 4) as usize];
        let cx = (w as f32 * 0.5) + ((t * 2.0).cos() as f32 * w as f32 * 0.3);
        let cy = (h as f32 * 0.5) + ((t * 1.5).sin() as f32 * h as f32 * 0.25);
        let radius = 120.0 + ((t * 4.0).sin() as f32 * 30.0);
        let r2 = radius * radius;

        for y in 0..h {
            let dy = y as f32 - cy;
            let dy2 = dy * dy;
            let row_idx = (y * w * 4) as usize;

            for x in 0..w {
                let dx = x as f32 - cx;
                let dist2 = dx * dx + dy2;
                let idx = row_idx + (x * 4) as usize;

                if dist2 <= r2 {
                    // Hình tròn phát sáng màu Neon Cyan (Darkworks)
                    let glow = 1.0 - (dist2 / r2);
                    pixels[idx] = (16.0 * glow) as u8;
                    pixels[idx + 1] = (220.0 * glow) as u8;
                    pixels[idx + 2] = (240.0 * glow) as u8;
                    pixels[idx + 3] = 255;
                } else {
                    // Nền xám đen kỹ thuật số với lưới nhẹ
                    let grid = if (x % 64 == 0) || (y % 64 == 0) { 32u8 } else { 16u8 };
                    pixels[idx] = grid;
                    pixels[idx + 1] = grid;
                    pixels[idx + 2] = grid + 4;
                    pixels[idx + 3] = 255;
                }
            }
        }

        let fps = self.info.video.as_ref().map(|v| v.fps).unwrap_or(60);
        Ok(Arc::new(VideoFrame::rgba8(
            w,
            h,
            pixels,
            time,
            Tick::from_seconds_f64(1.0 / fps.max(1) as f64),
        )))
    }

    fn audio_samples(&self, start: Tick, count: usize, sample_rate: u32) -> MediaResult<AudioBuffer> {
        let mut samples = Vec::with_capacity(count * 2);
        let start_sec = start.as_seconds_f64();

        for i in 0..count {
            let t = start_sec + (i as f64 / sample_rate as f64);
            // Hợp âm synth ấm áp (A4 440Hz + C#5 554Hz) kèm hiệu ứng stereo panning nhẹ
            let s_left = ((2.0 * std::f64::consts::PI * 440.0 * t).sin() * 0.20
                + (2.0 * std::f64::consts::PI * 554.37 * t).sin() * 0.10) as f32;
            let s_right = ((2.0 * std::f64::consts::PI * 440.0 * t).sin() * 0.10
                + (2.0 * std::f64::consts::PI * 554.37 * t).sin() * 0.20) as f32;
            samples.push(s_left);
            samples.push(s_right);
        }

        Ok(AudioBuffer::new(2, sample_rate, samples, start))
    }
}

/// Sinh các thanh màu tiêu chuẩn 75% SMPTE Color Bars
fn generate_smpte_pixels(width: u32, height: u32) -> Vec<u8> {
    let mut pixels = vec![0u8; (width * height * 4) as usize];
    // 7 thanh màu chuẩn: White, Yellow, Cyan, Green, Magenta, Red, Blue
    let bars: [[u8; 3]; 7] = [
        [191, 191, 191], // Gray/White 75%
        [191, 191, 0],   // Yellow
        [0, 191, 191],   // Cyan
        [0, 191, 0],     // Green
        [191, 0, 191],   // Magenta
        [191, 0, 0],     // Red
        [0, 0, 191],     // Blue
    ];

    let w = width as usize;
    let h = height as usize;
    let top_h = (h * 2) / 3;
    let bar_w = w / bars.len();

    for y in 0..top_h {
        let row_idx = y * w * 4;
        for (b, rgb) in bars.iter().enumerate() {
            let start_x = b * bar_w;
            let end_x = if b == bars.len() - 1 { w } else { (b + 1) * bar_w };
            for x in start_x..end_x {
                let idx = row_idx + x * 4;
                pixels[idx] = rgb[0];
                pixels[idx + 1] = rgb[1];
                pixels[idx + 2] = rgb[2];
                pixels[idx + 3] = 255;
            }
        }
    }

    // Phần dưới: Thanh đồng bộ đen/xám/trắng
    for y in top_h..h {
        let row_idx = y * w * 4;
        for x in 0..w {
            let idx = row_idx + x * 4;
            let val = if x < w / 6 {
                0 // Blue
            } else if x < (w * 2) / 6 {
                255 // White
            } else if x < (w * 3) / 6 {
                0 // Magenta
            } else {
                16 // Dark Gray
            };
            pixels[idx] = val;
            pixels[idx + 1] = val;
            pixels[idx + 2] = val;
            pixels[idx + 3] = 255;
        }
    }

    pixels
}
