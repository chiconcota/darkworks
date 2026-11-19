# MODULE KIẾN TRÚC: CANVAS 2D RENDERER (canvas-renderer/architecture.md)
@crate: `crates/canvas-renderer` | @layer: L3 / 2D Vector & Text Engine

## 1. VAI TRÒ & TRÁCH NHIỆM
Chịu trách nhiệm tạo hình 2D (Motion Graphics, Subtitle Karaoke, Bảng tên Lower-third, Biểu đồ) trực tiếp trên GPU mà không nhúng bất kỳ trình duyệt web nào (nhẹ hơn Chromium 100 lần, 0ms latency).

## 2. NỀN TẢNG CÔNG NGHỆ
- **Vello (Linebender Ecosystem):** Công cụ vẽ đồ họa vector 2D hiệu năng cao bằng Compute Shader trên GPU (`wgpu`).
- **Cosmic-text:** Thư viện layout font chữ, ngắt dòng, shaping font hỗ trợ Unicode tiếng Việt đầy đủ.
- **Vai trò trong Node Graph:** Đóng vai trò là một **Generator Node** (`Canvas2DNode`).
  - Đầu vào: `Tick` (thời gian hiện tại), Script/Dữ liệu JSON định nghĩa hình vẽ.
  - Đầu ra: `wgpu::Texture` RGBA trong suốt.
- **Karaoke Subtitle Engine:** Nhận mảng `Vec<TimedWord>` từ AI worker và tô màu từng chữ theo đúng thời gian thực mượt mà ở 60fps.
