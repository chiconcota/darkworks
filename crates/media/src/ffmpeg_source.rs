use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use darkworks_schema::Tick;
use parking_lot::Mutex;

use crate::cache::FrameCache;
use crate::error::{MediaError, MediaResult};
use crate::frame::{AudioBuffer, PixelFormat, VideoFrame};
use crate::source::{AudioTrackInfo, MediaInfo, MediaKind, MediaSource, VideoTrackInfo};

struct AudioCache {
    start_tick: Tick,
    sample_rate: u32,
    samples: Vec<f32>,
}

/// Nguồn Media giải mã từ file video thực tế bằng FFmpeg Native Pipeline kết hợp Worker Thread ngầm
pub struct FfmpegMediaSource {
    info: MediaInfo,
    path: PathBuf,
    cache: Arc<FrameCache<(String, i64)>>,
    current_playhead_frame: Arc<AtomicI64>,
    seek_command: Arc<AtomicI64>,
    stop_flag: Arc<AtomicBool>,
    active_child: Arc<Mutex<Option<Child>>>,
    worker_handle: Mutex<Option<JoinHandle<()>>>,
    last_frame: Mutex<Option<(i64, Arc<VideoFrame>)>>,
    audio_cache: Mutex<Option<AudioCache>>,
}

impl FfmpegMediaSource {
    /// Mở một file video (MP4, MKV, MOV, WebM...) bằng FFmpeg native kèm Background Prefetch Worker
    pub fn open(path: &Path, cache: Arc<FrameCache<(String, i64)>>) -> MediaResult<Self> {
        if !path.exists() {
            return Err(MediaError::Io(format!("Tệp không tồn tại: {}", path.display())));
        }

        let path_str = path.to_str().ok_or_else(|| {
            MediaError::Io("Đường dẫn tệp chứa ký tự không hợp lệ".to_string())
        })?;

        // 1. Thăm dò thông số video bằng ffprobe JSON
        let output = Command::new("ffprobe")
            .args([
                "-v", "error",
                "-select_streams", "v:0",
                "-show_entries", "stream=width,height,r_frame_rate",
                "-show_entries", "format=duration",
                "-of", "json",
                path_str,
            ])
            .output()
            .map_err(|e| MediaError::Io(format!("Lỗi thực thi ffprobe: {e}")))?;

        if !output.status.success() {
            return Err(MediaError::Decode("ffprobe không đọc được tệp video".into()));
        }

        let json: serde_json::Value = serde_json::from_slice(&output.stdout)
            .map_err(|e| MediaError::Decode(format!("Lỗi đọc metadata JSON ffprobe: {e}")))?;

        let stream = json.get("streams")
            .and_then(|s| s.get(0))
            .ok_or_else(|| MediaError::StreamNotFound("Không tìm thấy luồng video trong tệp".into()))?;

        let width = stream.get("width").and_then(|v| v.as_u64()).unwrap_or(640) as u32;
        let height = stream.get("height").and_then(|v| v.as_u64()).unwrap_or(360) as u32;
        let fps = parse_fps(stream.get("r_frame_rate").and_then(|v| v.as_str()).unwrap_or("30/1"));

        let duration_secs = json.get("format")
            .and_then(|f| f.get("duration"))
            .and_then(|d| d.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(60.0);

        let duration = Tick::from_seconds_f64(duration_secs);
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("video_source")
            .to_string();

        let source_id = format!("ffmpeg_{}", path.display());
        let info = MediaInfo {
            id: source_id.clone(),
            name: file_name,
            kind: MediaKind::Video,
            duration,
            video: Some(VideoTrackInfo {
                width,
                height,
                fps,
                codec: "h264".into(),
                format: PixelFormat::Rgba8,
            }),
            audio: Some(AudioTrackInfo {
                channels: 2,
                sample_rate: 48_000,
                codec: "aac".into(),
            }),
        };

        // 2. Khởi tạo các cờ đồng bộ và luồng giải mã ngầm (Background Prefetch Streamer)
        let current_playhead_frame = Arc::new(AtomicI64::new(0));
        let seek_command = Arc::new(AtomicI64::new(0));
        let stop_flag = Arc::new(AtomicBool::new(false));
        let active_child = Arc::new(Mutex::new(None));

        let worker_params = PrefetchWorkerParams {
            path: path.to_path_buf(),
            source_id,
            width,
            height,
            fps,
            cache: Arc::clone(&cache),
            current_playhead_frame: Arc::clone(&current_playhead_frame),
            seek_command: Arc::clone(&seek_command),
            stop_flag: Arc::clone(&stop_flag),
            active_child: Arc::clone(&active_child),
        };

        let worker_handle = thread::spawn(move || {
            run_prefetch_loop(worker_params);
        });

        Ok(Self {
            info,
            path: path.to_path_buf(),
            cache,
            current_playhead_frame,
            seek_command,
            stop_flag,
            active_child,
            worker_handle: Mutex::new(Some(worker_handle)),
            last_frame: Mutex::new(None),
            audio_cache: Mutex::new(None),
        })
    }
}

impl Drop for FfmpegMediaSource {
    fn drop(&mut self) {
        self.stop_flag.store(true, Ordering::SeqCst);
        if let Some(ref mut child) = *self.active_child.lock() {
            let _ = child.kill();
        }
        if let Some(handle) = self.worker_handle.lock().take() {
            let _ = handle.join();
        }
    }
}

impl MediaSource for FfmpegMediaSource {
    fn info(&self) -> &MediaInfo {
        &self.info
    }

    fn notify_playhead(&self, time: Tick) {
        let fps = self.info.video.as_ref().map(|v| v.fps).unwrap_or(30);
        let frame_idx = (time.as_seconds_f64() * fps as f64).round() as i64;
        self.current_playhead_frame.store(frame_idx, Ordering::Relaxed);
    }

    fn seek(&self, time: Tick) {
        let fps = self.info.video.as_ref().map(|v| v.fps).unwrap_or(30);
        let frame_idx = (time.as_seconds_f64() * fps as f64).round() as i64;
        self.current_playhead_frame.store(frame_idx, Ordering::SeqCst);
        self.seek_command.store(frame_idx, Ordering::SeqCst);
    }

    fn video_frame(&self, time: Tick) -> MediaResult<Arc<VideoFrame>> {
        let fps = self.info.video.as_ref().map(|v| v.fps).unwrap_or(30);
        let frame_idx = (time.as_seconds_f64() * fps as f64).round() as i64;
        self.current_playhead_frame.store(frame_idx, Ordering::Relaxed);

        // 1. Kiểm tra trong FrameCache (0ms hit từ Background Prefetch Streamer)
        let key = (self.info.id.clone(), frame_idx);
        if let Some(frame) = self.cache.get(&key) {
            let mut last = self.last_frame.lock();
            *last = Some((frame_idx, Arc::clone(&frame)));
            return Ok(frame);
        }

        // 2. Kiểm tra cache frame gần nhất
        {
            let last = self.last_frame.lock();
            if let Some((idx, ref frame)) = *last {
                if idx == frame_idx {
                    return Ok(Arc::clone(frame));
                }
            }
        }

        // 3. Chờ tối đa 20ms xem worker ngầm có nạp xong frame này không
        for _ in 0..4 {
            thread::sleep(Duration::from_millis(5));
            if let Some(frame) = self.cache.get(&key) {
                let mut last = self.last_frame.lock();
                *last = Some((frame_idx, Arc::clone(&frame)));
                return Ok(frame);
            }
        }

        // 4. Nếu vẫn chưa có (cold seek khi đang pause), decode nhanh 1 frame làm fallback
        let sec = time.as_seconds_f64();
        let w = self.info.video.as_ref().map(|v| v.width).unwrap_or(640);
        let h = self.info.video.as_ref().map(|v| v.height).unwrap_or(360);
        let expected_bytes = (w * h * 4) as usize;

        let path_str = self.path.to_str().unwrap_or_default();
        let output = Command::new("ffmpeg")
            .args([
                "-ss", &format!("{:.3}", sec),
                "-i", path_str,
                "-vframes", "1",
                "-f", "rawvideo",
                "-pix_fmt", "rgba",
                "pipe:1",
            ])
            .output()
            .map_err(|e| MediaError::Decode(format!("Lỗi decode video ffmpeg: {e}")))?;

        if output.stdout.len() < expected_bytes {
            return Err(MediaError::Decode(format!(
                "Dữ liệu khung hình không đủ: nhận {} bytes, mong đợi {} bytes",
                output.stdout.len(), expected_bytes
            )));
        }

        let mut raw_pixels = output.stdout;
        raw_pixels.truncate(expected_bytes);

        let frame_duration = Tick::from_seconds_f64(1.0 / fps.max(1) as f64);
        let frame = Arc::new(VideoFrame::rgba8(w, h, raw_pixels, time, frame_duration));

        self.cache.insert(key, Arc::clone(&frame));
        let mut last = self.last_frame.lock();
        *last = Some((frame_idx, Arc::clone(&frame)));

        Ok(frame)
    }

    fn audio_samples(&self, start: Tick, count: usize, sample_rate: u32) -> MediaResult<AudioBuffer> {
        let channels = 2usize;
        let needed_samples = count * channels;

        // 1. Kiểm tra bộ đệm audio nội bộ xem có thể tái sử dụng không
        {
            let cache = self.audio_cache.lock();
            if let Some(ref c) = *cache {
                if c.sample_rate == sample_rate && start >= c.start_tick {
                    let offset_sec = (start.0 - c.start_tick.0) as f64 / darkworks_schema::TICKS_PER_SECOND as f64;
                    let offset_sample = (offset_sec * sample_rate as f64) as usize * channels;
                    if offset_sample + needed_samples <= c.samples.len() {
                        let slice = c.samples[offset_sample..offset_sample + needed_samples].to_vec();
                        return Ok(AudioBuffer::new(channels as u16, sample_rate, slice, start));
                    }
                }
            }
        }

        // 2. Trích xuất khối 3 giây audio qua FFmpeg và lưu đệm
        let start_sec = start.as_seconds_f64();
        let duration_sec = ((count as f64 / sample_rate as f64) + 2.5).min(5.0);
        let path_str = self.path.to_str().unwrap_or_default();

        let output = Command::new("ffmpeg")
            .args([
                "-ss", &format!("{:.3}", start_sec),
                "-i", path_str,
                "-t", &format!("{:.3}", duration_sec),
                "-f", "f32le",
                "-acodec", "pcm_f32le",
                "-ar", &sample_rate.to_string(),
                "-ac", "2",
                "pipe:1",
            ])
            .output()
            .map_err(|e| MediaError::Decode(format!("Lỗi decode audio ffmpeg: {e}")))?;

        let mut samples = Vec::with_capacity(output.stdout.len() / 4);
        for chunk in output.stdout.chunks_exact(4) {
            let val = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            samples.push(val);
        }

        let samples_len = samples.len();
        {
            let mut cache = self.audio_cache.lock();
            *cache = Some(AudioCache {
                start_tick: start,
                sample_rate,
                samples: samples.clone(),
            });
        }

        let result_samples = if samples_len >= needed_samples {
            samples[..needed_samples].to_vec()
        } else {
            let mut padded = samples;
            padded.resize(needed_samples, 0.0);
            padded
        };

        Ok(AudioBuffer::new(channels as u16, sample_rate, result_samples, start))
    }
}

struct PrefetchWorkerParams {
    path: PathBuf,
    source_id: String,
    width: u32,
    height: u32,
    fps: u32,
    cache: Arc<FrameCache<(String, i64)>>,
    current_playhead_frame: Arc<AtomicI64>,
    seek_command: Arc<AtomicI64>,
    stop_flag: Arc<AtomicBool>,
    active_child: Arc<Mutex<Option<Child>>>,
}

/// Vòng lặp giải mã ngầm streaming liên tục đón đầu Playhead
fn run_prefetch_loop(params: PrefetchWorkerParams) {
    let frame_bytes = (params.width * params.height * 4) as usize;
    let frame_duration = Tick::from_seconds_f64(1.0 / params.fps.max(1) as f64);
    let mut current_decoded_frame: i64 = 0;
    let path_str = params.path.to_str().unwrap_or_default().to_string();

    while !params.stop_flag.load(Ordering::Relaxed) {
        // 1. Kiểm tra lệnh Seek từ người dùng
        let seek_to = params.seek_command.swap(-1, Ordering::SeqCst);
        if seek_to >= 0 {
            if let Some(mut child) = params.active_child.lock().take() {
                let _ = child.kill();
                let _ = child.wait();
            }
            current_decoded_frame = seek_to;
        }

        let playhead = params.current_playhead_frame.load(Ordering::Relaxed);

        // 2. Giới hạn khoảng cách đệm trước: duy trì 45 frames (~1.5s) đón đầu playhead
        if current_decoded_frame > playhead + 45 {
            thread::sleep(Duration::from_millis(15));
            continue;
        }

        // Nếu playhead nhảy vượt quá xa luồng streaming hiện tại (> 15 frames), restart stream
        if playhead > current_decoded_frame + 15 || current_decoded_frame < playhead - 10 {
            if let Some(mut child) = params.active_child.lock().take() {
                let _ = child.kill();
                let _ = child.wait();
            }
            current_decoded_frame = playhead;
        }

        // 3. Nếu chưa có child process streaming, khởi tạo một tiến trình mới
        let is_running = params.active_child.lock().is_some();
        if !is_running {
            let start_sec = (current_decoded_frame as f64 / params.fps.max(1) as f64).max(0.0);
            let child_res = Command::new("ffmpeg")
                .args([
                    "-ss", &format!("{:.3}", start_sec),
                    "-i", &path_str,
                    "-f", "rawvideo",
                    "-pix_fmt", "rgba",
                    "pipe:1",
                ])
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn();

            match child_res {
                Ok(child) => {
                    *params.active_child.lock() = Some(child);
                }
                Err(_) => {
                    thread::sleep(Duration::from_millis(50));
                    continue;
                }
            }
        }

        // 4. Đọc 1 khung hình từ stdout của child streaming process
        let mut frame_buf = vec![0u8; frame_bytes];
        let mut read_success = false;

        {
            let mut guard = params.active_child.lock();
            if let Some(ref mut child) = *guard {
                if let Some(ref mut stdout) = child.stdout {
                    if stdout.read_exact(&mut frame_buf).is_ok() {
                        read_success = true;
                    }
                }
            }
        }

        if read_success {
            let key = (params.source_id.clone(), current_decoded_frame);
            let pts = Tick::from_seconds_f64(current_decoded_frame as f64 / params.fps.max(1) as f64);
            let frame = Arc::new(VideoFrame::rgba8(params.width, params.height, frame_buf, pts, frame_duration));
            params.cache.insert(key, frame);
            current_decoded_frame += 1;
        } else {
            // EOF hoặc tiến trình bị ngắt: dọn dẹp child và chờ
            if let Some(mut child) = params.active_child.lock().take() {
                let _ = child.wait();
            }
            thread::sleep(Duration::from_millis(20));
        }
    }

    // Kết thúc: dọn dẹp tiến trình con
    if let Some(mut child) = params.active_child.lock().take() {
        let _ = child.kill();
        let _ = child.wait();
    }
}

fn parse_fps(fps_str: &str) -> u32 {
    if let Some((num, den)) = fps_str.split_once('/') {
        let n: f64 = num.parse().unwrap_or(30.0);
        let d: f64 = den.parse().unwrap_or(1.0);
        if d > 0.0 {
            (n / d).round() as u32
        } else {
            30
        }
    } else {
        fps_str.parse().unwrap_or(30)
    }
}
