# MODULE KIẾN TRÚC: AI WORKERS & SPEECH PROCESSING (ai-workers/architecture.md)
@crate: `crates/ai-workers` | @layer: L4 / Background Jobs & Local ML

## 1. VAI TRÒ & TRÁCH NHIỆM
Chạy các tiến trình nền phục vụ tự động hóa biên tập video hoàn toàn trên máy cục bộ (Local), không phụ thuộc vào API đám mây và không cần cài đặt Python.

## 2. NĂNG LỰC CỐT LÕI
- **Voice Activity Detection (Silero VAD):**
  - Chạy mô hình ONNX qua thư viện `ort`.
  - Quét file âm thanh 1 tiếng trong 2-3 giây để tìm chính xác các đoạn im lặng và sinh ra danh sách `CutRange` để tự động băm video (Auto Cut Silence).
- **Speech-to-Text & Subtitles (Whisper / Moonshine):**
  - Nhúng `whisper-rs` hoặc mô hình Moonshine ONNX.
  - Nhận diện giọng nói, tách từ khóa kèm timestamp chi tiết (`Vec<TimedWord>`) để phục vụ hiệu ứng chữ nhảy Karaoke trên Canvas 2D.
- **Computer Vision & Visual Inference (Face Swap / Rembg):**
  - Chạy mô hình ONNX qua `ort` tăng tốc GPU (DirectML / Vulkan).
  - Cung cấp lõi xử lý per-frame inference cho các Node đồ họa: Hoán đổi khuôn mặt (Face Swap / Swiper) và Tách nền tự động không phông xanh (AI Matte / Rembg).
- **Tiến Trình Bất Đồng Bộ (Background Job):**
  - Báo tiến độ (%) về UI qua Channel; hỗ trợ người dùng bấm nút Hủy (Cancel) bất cứ lúc nào.
