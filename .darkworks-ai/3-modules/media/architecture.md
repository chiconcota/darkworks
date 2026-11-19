# MODULE KIẾN TRÚC: MEDIA ENGINE & A/V SYNC (media/architecture.md)
@crate: `crates/media` (hoặc `filmcraft-media` / `platform`) | @layer: L1 / Hardware I/O & Playback

## 1. VAI TRÒ & TRÁCH NHIỆM
Chịu trách nhiệm giải mã video phần cứng, nạp audio, quản lý bộ nhớ đệm cache và đồng bộ thời gian âm thanh - hình ảnh (A/V Sync).

## 2. NGUYÊN TẮC KỸ THUẬT
- **Zero-Copy Hardware Decoding:** Sử dụng Linux VA-API / NVDEC giải mã video (H.264, HEVC, ProRes, AV1), đẩy thẳng khung hình lên `wgpu::Texture` mà không copy qua CPU RAM.
- **Ring-Buffer Cache (< 2GB RAM):** Khóa trần bộ đệm RAM hệ thống. Chỉ lưu trữ các frame lân cận Playhead (LRU cache). Không tích trữ tràn lan.
- **A/V Sync (cpal + PipeWire/ALSA):** Sử dụng đồng hồ âm thanh phần cứng làm gốc thời gian chuẩn, hình ảnh bám theo nhịp quét màn hình.
- **Audio Peak Extraction:** Trích xuất mảng sóng âm (Waveform / RMS Peak) để vẽ trên Timeline và cấp tín hiệu cho các Node phản ứng âm thanh.
