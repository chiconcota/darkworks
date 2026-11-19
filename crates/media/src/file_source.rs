use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use darkworks_schema::Tick;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use crate::error::{MediaError, MediaResult};
use crate::frame::{AudioBuffer, PixelFormat, VideoFrame};
use crate::source::{AudioTrackInfo, MediaInfo, MediaKind, MediaSource, VideoTrackInfo};

/// Nguồn Media từ tệp tin cục bộ (Audio, Hình ảnh tĩnh, Video)
pub struct FileMediaSource {
    info: MediaInfo,
    audio_buffer: Option<AudioBuffer>,
    still_frame: Option<Arc<VideoFrame>>,
}

impl FileMediaSource {
    /// Mở một file âm thanh (WAV, MP3, AAC, FLAC, OGG...) bằng symphonia
    pub fn open_audio(path: &Path) -> MediaResult<Self> {
        let file = File::open(path)
            .map_err(|e| MediaError::Io(format!("{}: {e}", path.display())))?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            hint.with_extension(ext);
        }

        let format_opts = FormatOptions::default();
        let metadata_opts = MetadataOptions::default();
        let decoder_opts = DecoderOptions::default();

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &format_opts, &metadata_opts)
            .map_err(|e| MediaError::Decode(format!("Probe error: {e}")))?;

        let mut reader = probed.format;
        let track = reader
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
            .ok_or_else(|| MediaError::StreamNotFound("Không tìm thấy audio track".into()))?;

        let sample_rate = track.codec_params.sample_rate.unwrap_or(48_000);
        let channels = track.codec_params.channels.map(|c| c.count() as u16).unwrap_or(2);
        let codec_name = format!("{:?}", track.codec_params.codec);

        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &decoder_opts)
            .map_err(|e| MediaError::Decode(format!("Không thể khởi tạo decoder: {e}")))?;

        let mut all_samples: Vec<f32> = Vec::new();

        loop {
            let packet = match reader.next_packet() {
                Ok(p) => p,
                Err(SymphoniaError::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => {
                    log::debug!("Kết thúc đọc audio packet: {e}");
                    break;
                }
            };

            match decoder.decode(&packet) {
                Ok(decoded) => {
                    let spec = *decoded.spec();
                    let mut sample_buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
                    sample_buf.copy_interleaved_ref(decoded);
                    all_samples.extend_from_slice(sample_buf.samples());
                }
                Err(e) => {
                    log::debug!("Lỗi decode packet: {e}");
                    break;
                }
            }
        }

        let total_frames = if channels > 0 { all_samples.len() / channels as usize } else { 0 };
        let duration_secs = total_frames as f64 / sample_rate as f64;
        let duration = Tick::from_seconds_f64(duration_secs);

        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("audio_source")
            .to_string();

        let info = MediaInfo {
            id: format!("audio_{}", path.display()),
            name: file_name,
            kind: MediaKind::Audio,
            duration,
            video: None,
            audio: Some(AudioTrackInfo {
                channels,
                sample_rate,
                codec: codec_name,
            }),
        };

        let buffer = AudioBuffer::new(channels, sample_rate, all_samples, Tick::ZERO);

        Ok(Self {
            info,
            audio_buffer: Some(buffer),
            still_frame: None,
        })
    }

    /// Mở một file ảnh tĩnh (PNG, JPEG, WebP)
    pub fn open_image(path: &Path, default_duration_secs: u64) -> MediaResult<Self> {
        let img = image::open(path)
            .map_err(|e| MediaError::Decode(format!("Lỗi đọc ảnh {}: {e}", path.display())))?
            .to_rgba8();

        let (width, height) = img.dimensions();
        let raw_pixels = img.into_raw();

        let duration = Tick::from_seconds(default_duration_secs);
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("image_source")
            .to_string();

        let info = MediaInfo {
            id: format!("image_{}", path.display()),
            name: file_name,
            kind: MediaKind::StillImage,
            duration,
            video: Some(VideoTrackInfo {
                width,
                height,
                fps: 30,
                codec: "image_rgba".into(),
                format: PixelFormat::Rgba8,
            }),
            audio: None,
        };

        let frame = Arc::new(VideoFrame::rgba8(
            width,
            height,
            raw_pixels,
            Tick::ZERO,
            duration,
        ));

        Ok(Self {
            info,
            audio_buffer: None,
            still_frame: Some(frame),
        })
    }
}

impl MediaSource for FileMediaSource {
    fn info(&self) -> &MediaInfo {
        &self.info
    }

    fn video_frame(&self, time: Tick) -> MediaResult<Arc<VideoFrame>> {
        if let Some(ref frame) = self.still_frame {
            let mut f = (**frame).clone();
            f.pts = time;
            Ok(Arc::new(f))
        } else {
            Err(MediaError::StreamNotFound("Không có video track trong file này".into()))
        }
    }

    fn audio_samples(&self, start: Tick, count: usize, sample_rate: u32) -> MediaResult<AudioBuffer> {
        let Some(ref buf) = self.audio_buffer else {
            return Err(MediaError::StreamNotFound("Không có audio track trong file này".into()));
        };

        let channels = buf.channels as usize;
        let start_sample = (start.as_seconds_f64() * buf.sample_rate as f64) as usize;
        let start_idx = start_sample * channels;
        let end_idx = (start_idx + count * channels).min(buf.samples.len());

        let samples = if start_idx < buf.samples.len() {
            buf.samples[start_idx..end_idx].to_vec()
        } else {
            Vec::new()
        };

        Ok(AudioBuffer::new(buf.channels, sample_rate, samples, start))
    }
}
