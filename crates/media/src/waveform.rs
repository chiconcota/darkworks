use darkworks_schema::Tick;
use crate::frame::AudioBuffer;

/// Cấu trúc sóng âm trực quan phục vụ vẽ Waveform trên Timeline
#[derive(Debug, Clone, Default)]
pub struct AudioWaveform {
    /// Mảng đỉnh cực đại (Peak) chuẩn hóa [0.0, 1.0]
    pub peaks: Vec<f32>,
    /// Mảng công suất trung bình (RMS) chuẩn hóa [0.0, 1.0]
    pub rms: Vec<f32>,
    pub duration: Tick,
}

impl AudioWaveform {
    /// Trích xuất Waveform từ AudioBuffer với số lượng điểm ảnh xác định
    pub fn from_audio_buffer(buffer: &AudioBuffer, target_points: usize) -> Self {
        let total_frames = buffer.frame_count();
        if total_frames == 0 || target_points == 0 {
            return Self::default();
        }

        let frames_per_point = (total_frames / target_points).max(1);
        let mut peaks = Vec::with_capacity(target_points);
        let mut rms = Vec::with_capacity(target_points);

        let channels = buffer.channels as usize;
        let samples = &buffer.samples;

        for chunk_idx in 0..target_points {
            let start_frame = chunk_idx * frames_per_point;
            let end_frame = ((chunk_idx + 1) * frames_per_point).min(total_frames);
            if start_frame >= total_frames {
                break;
            }

            let mut max_abs = 0.0f32;
            let mut sum_sq = 0.0f32;
            let mut sample_count = 0usize;

            for f in start_frame..end_frame {
                for c in 0..channels {
                    let idx = f * channels + c;
                    if let Some(&s) = samples.get(idx) {
                        let abs_s = s.abs();
                        if abs_s > max_abs {
                            max_abs = abs_s;
                        }
                        sum_sq += s * s;
                        sample_count += 1;
                    }
                }
            }

            let point_peak = max_abs.clamp(0.0, 1.0);
            let point_rms = if sample_count > 0 {
                (sum_sq / sample_count as f32).sqrt().clamp(0.0, 1.0)
            } else {
                0.0
            };

            peaks.push(point_peak);
            rms.push(point_rms);
        }

        Self {
            peaks,
            rms,
            duration: buffer.pts,
        }
    }
}
