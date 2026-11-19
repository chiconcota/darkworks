# MODULE KIẾN TRÚC: MEDIA ENGINE & A/V SYNC (media/architecture.md)
@crate: `crates/media` | @layer: L1 / Hardware I/O & Playback | @status: 🟢 Hoàn thành cấu trúc & Kiểm toán

## 1. VAI TRÒ & TRÁCH NHIỆM
Chịu trách nhiệm phát hiện và giải mã video phần cứng (Linux VA-API), nạp audio, quản lý bộ nhớ đệm cache dạng Ring-Buffer LRU và đồng bộ thời gian âm thanh - hình ảnh (A/V Sync).

## 2. NGUYÊN TẮC KỸ THUẬT & THÀNH PHẦN CỐT LÕI

### 2.1. Cấu Trúc Khung Hình & Mẫu Âm Thanh (`frame.rs`)
- `VideoFrame`: Bọc dữ liệu pixel trong `Arc<Vec<u8>>` để chia sẻ zero-copy giữa LRU Cache và Viewport.
- Định dạng hỗ trợ: `Rgba8`, `Nv12` (chuẩn VA-API/NVDEC), `Yuv420p`.
- Chuyển đổi màu sắc nhanh bằng `rayon` theo không gian màu chuẩn ITU-R BT.709.
- `AudioBuffer`: Mẫu âm thanh số thực 32-bit float (`f32`) xen kẽ (Interleaved L/R).

### 2.2. Ring-Buffer LRU Frame Cache (`cache.rs`)
- **Phân tầng 2 cấp độ bộ nhớ:**
  - `DEFAULT_CACHE_BUDGET_BYTES = 1GB` (1024MB): Mức 1 tiêu chuẩn khi nạp và phát video dài.
  - `MAX_CACHE_BUDGET_BYTES = 2GB` (2048MB): Mức trần cứng tối thượng theo Khẩu quyết #1.
- **Thuật toán Eviction vi mô mượt mà $O(1)$:**
  - Sử dụng hàng đợi `VecDeque<(K, u64)>` theo dõi thứ tự truy cập thời gian thực.
  - Khi chạm budget, giải phóng từng frame cũ nhất một cách êm ái ($O(1)$) ngay khi có frame mới vào.
  - **Zero Lock Contention:** Nhả khóa `Mutex` trước khi hủy dữ liệu byte của frame, loại bỏ hoàn toàn độ trễ giải phóng bộ nhớ trên UI Thread.
- Luồng an toàn (Thread-safe) sử dụng `parking_lot::Mutex`.

### 2.3. Thăm Dò Phần Cứng Linux VA-API (`vaapi.rs`)
- Sử dụng dynamic loading (`libloading`) nạp `libva.so.2` và `libva-drm.so.2` khi runtime.
- Liên kết trực tiếp với `/dev/dri/renderD128` (hoặc `renderD129`).
- Tự động nhận diện Vendor (AMD Radeon Gallium/Mesa, Intel iHD) và các codec phần cứng (H.264, HEVC, VP9, AV1).

### 2.4. Đồng Bộ A/V Sync & Audio Player (`audio_player.rs`)
- Sử dụng `cpal` phát âm thanh chuẩn qua PipeWire/ALSA.
- Cung cấp đồng hồ thời gian nguyên thủy `Tick Clock` chuẩn xác theo mẫu âm thanh thực tế.
- Bộ đệm vòng Ring-Buffer audio tự động chống nghẽn (Buffer underrun protection).

### 2.5. Trình Giả Lập Mẫu & Kiểm Thử (`generators.rs`)
- `SmpteBars`: Thanh màu 75% SMPTE Color Bars tiêu chuẩn kèm âm chuẩn 1000Hz (-12 dBFS).
- `ProceduralMotion`: Hoạt họa chuyển động 60fps giả lập để kiểm thử hiệu năng rendering và bộ đệm không cần file ngoại vi.

### 2.6. Trích Xuất Sóng Âm Trực Quan (`waveform.rs`)
- Tính toán mảng Peak và RMS trên từng dải mẫu âm thanh, phục vụ vẽ Waveform trên trục Timeline.

### 2.7. Nguồn Đa Phương Tiện Thực Tế & Luồng Prefetch Ngầm (`ffmpeg_source.rs`)
- `FfmpegMediaSource`: Hỗ trợ nạp file video thực tế (MP4/MKV) dài nhiều giờ (đã nghiệm thu video 1 tiếng `tropical_paradise_1hour.mp4`).
- **Background Prefetch Worker Thread:** Khởi chạy một tiến trình ngầm đọc streaming pipe từ FFmpeg ở tốc độ cao (~53x real-time), liên tục giải mã trước ~45 frames vào `FrameCache` (2GB cap).
- **Zero UI-Thread Blocking:** Vòng lặp giao diện chính (Main egui Thread) chỉ đọc khung hình tức thì (<0.001ms) từ RAM cache, đảm bảo tốc độ ổn định 60 FPS khi phát video 1 tiếng.
- **Audio On-Demand Caching:** Đệm từng khối âm thanh PCM 3 giây, giữ RAM toàn bộ ứng dụng chỉ ở mức ~200MB - 300MB.
