use std::sync::Arc;
use darkworks_schema::Tick;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

/// Định dạng pixel của khung hình video
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PixelFormat {
    /// 8-bit RGBA (4 bytes/pixel, chuẩn cho Viewport & GPU upload)
    Rgba8,
    /// NV12 (Y-plane full res, UV-plane interleaved half res, chuẩn VA-API/NVDEC)
    Nv12,
    /// YUV420 Planar (Y, U, V phân tách riêng lẻ)
    Yuv420p,
}

impl PixelFormat {
    pub fn bytes_per_pixel(&self) -> f32 {
        match self {
            Self::Rgba8 => 4.0,
            Self::Nv12 | Self::Yuv420p => 1.5,
        }
    }
}

/// Khung hình video đã giải mã (được bọc trong `Arc` để chia sẻ zero-copy giữa Cache và GPU)
#[derive(Debug, Clone)]
pub struct VideoFrame {
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
    pub data: Arc<Vec<u8>>,
    pub pts: Tick,
    pub duration: Tick,
}

impl VideoFrame {
    /// Tạo khung hình RGBA8 tiêu chuẩn
    pub fn rgba8(width: u32, height: u32, data: Vec<u8>, pts: Tick, duration: Tick) -> Self {
        Self {
            width,
            height,
            format: PixelFormat::Rgba8,
            data: Arc::new(data),
            pts,
            duration,
        }
    }

    /// Dung lượng bộ nhớ thực tế (Bytes) dùng cho LRU Cache
    pub fn byte_size(&self) -> usize {
        self.data.len()
    }

    /// Chuyển đổi khung hình sang RGBA8 để nạp vào GPU Texture của Viewport
    pub fn to_rgba8(&self) -> Arc<Vec<u8>> {
        match self.format {
            PixelFormat::Rgba8 => Arc::clone(&self.data),
            PixelFormat::Nv12 => Arc::new(nv12_to_rgba8(self.width, self.height, &self.data)),
            PixelFormat::Yuv420p => Arc::new(yuv420p_to_rgba8(self.width, self.height, &self.data)),
        }
    }
}

/// Bộ đệm âm thanh PCM dạng số thực `f32` (chuẩn âm thanh studio 32-bit float)
#[derive(Debug, Clone)]
pub struct AudioBuffer {
    pub channels: u16,
    pub sample_rate: u32,
    /// Mẫu âm thanh xen kẽ (Interleaved samples: [L, R, L, R...])
    pub samples: Vec<f32>,
    pub pts: Tick,
}

impl AudioBuffer {
    pub fn new(channels: u16, sample_rate: u32, samples: Vec<f32>, pts: Tick) -> Self {
        Self {
            channels,
            sample_rate,
            samples,
            pts,
        }
    }

    pub fn frame_count(&self) -> usize {
        if self.channels == 0 {
            0
        } else {
            self.samples.len() / self.channels as usize
        }
    }

    pub fn byte_size(&self) -> usize {
        self.samples.len() * std::mem::size_of::<f32>()
    }
}

/// Chuyển đổi NV12 sang RGBA8 song song bằng Rayon theo chuẩn Rec.709
fn nv12_to_rgba8(width: u32, height: u32, nv12: &[u8]) -> Vec<u8> {
    let w = width as usize;
    let h = height as usize;
    let y_plane_size = w * h;
    let uv_plane = &nv12[y_plane_size..];

    let mut rgba = vec![0u8; w * h * 4];

    rgba.par_chunks_mut(w * 4)
        .enumerate()
        .for_each(|(y, row)| {
            let y_row_start = y * w;
            let uv_row_start = (y / 2) * w;

            for x in 0..w {
                let y_val = nv12[y_row_start + x] as f32;
                let uv_idx = uv_row_start + (x & !1);
                let u_val = uv_plane.get(uv_idx).copied().unwrap_or(128) as f32 - 128.0;
                let v_val = uv_plane.get(uv_idx + 1).copied().unwrap_or(128) as f32 - 128.0;

                // Chuẩn Rec.709 Limited Range (Studio Swing)
                let c = (y_val - 16.0) * (255.0 / 219.0);
                let r = (c + 1.5748 * v_val).clamp(0.0, 255.0) as u8;
                let g = (c - 0.1873 * u_val - 0.4681 * v_val).clamp(0.0, 255.0) as u8;
                let b = (c + 1.8556 * u_val).clamp(0.0, 255.0) as u8;

                let px = x * 4;
                row[px] = r;
                row[px + 1] = g;
                row[px + 2] = b;
                row[px + 3] = 255;
            }
        });

    rgba
}

/// Chuyển đổi YUV420 Planar sang RGBA8
fn yuv420p_to_rgba8(width: u32, height: u32, yuv: &[u8]) -> Vec<u8> {
    let w = width as usize;
    let h = height as usize;
    let y_size = w * h;
    let u_size = (w / 2) * (h / 2);

    let y_plane = &yuv[..y_size];
    let u_plane = &yuv[y_size..y_size + u_size];
    let v_plane = &yuv[y_size + u_size..];

    let mut rgba = vec![0u8; w * h * 4];

    rgba.par_chunks_mut(w * 4)
        .enumerate()
        .for_each(|(y, row)| {
            let y_row_start = y * w;
            let uv_w = w / 2;
            let uv_row_start = (y / 2) * uv_w;

            for x in 0..w {
                let y_val = y_plane[y_row_start + x] as f32;
                let uv_idx = uv_row_start + (x / 2);
                let u_val = u_plane.get(uv_idx).copied().unwrap_or(128) as f32 - 128.0;
                let v_val = v_plane.get(uv_idx).copied().unwrap_or(128) as f32 - 128.0;

                let c = (y_val - 16.0) * (255.0 / 219.0);
                let r = (c + 1.5748 * v_val).clamp(0.0, 255.0) as u8;
                let g = (c - 0.1873 * u_val - 0.4681 * v_val).clamp(0.0, 255.0) as u8;
                let b = (c + 1.8556 * u_val).clamp(0.0, 255.0) as u8;

                let px = x * 4;
                row[px] = r;
                row[px + 1] = g;
                row[px + 2] = b;
                row[px + 3] = 255;
            }
        });

    rgba
}
