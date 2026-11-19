# MODULE KIẾN TRÚC: NODE GRAPH & GPU COMPOSITOR (node-graph/architecture.md)
@crate: `crates/node-graph` (hoặc `node-compositor`) | @layer: L3 / Render Pipeline & Shader

## 1. VAI TRÒ & TRÁCH NHIỆM
Đây là trái tim đồ họa phân biệt Darkworks với FilmCraft gốc. Module này thay thế mô hình danh sách phẳng `Vec<EffectInstance>` bằng mạng lưới đồ thị có hướng không chu trình (Directed Acyclic Graph - DAG).

## 2. KIẾN TRÚC & NGUYÊN LÝ HOẠT ĐỘNG
- **Node Graph UI:** Sử dụng `egui_snarl` để render bảng nối dây trực quan (kéo dây, ghim socket, đổi thông số).
- **Socket & Data Routing:**
  - Socket Hình ảnh: truyền `wgpu::Texture` (RGBA).
  - Socket Âm thanh: truyền giá trị biên độ sóng âm `f32` (RMS Peak).
  - Socket Thông số: truyền `f32`, `Vec2`, `Color` (dùng cho Scale, Position, Opacity).
- **Các Node Nguyên Tử Đầu Tiên:**
  1. `VideoInput`: Lấy texture khung hình video gốc từ clip.
  2. `CanvasInput`: Lấy texture từ Canvas 2D Node (Text, Vector, Karaoke).
  3. `Transform2D`: Zoom, Pan, Rotate, Anchor.
  4. `ColorGrade`: Chỉnh sáng, độ bão hòa, LUT.
  5. `Blur / Glow`: Shader WGSL làm mờ và phát sáng.
  6. `Blend`: Trộn 2 lớp hình ảnh (Normal, Add, Multiply, Screen) có hỗ trợ Mask.
  7. `AudioReactor`: Đọc RMS peak từ audio track đẩy sang socket thông số (Scale/Shake).
  8. `Output`: Khung hình kết xuất cuối cùng đẩy ra Viewport hoặc Video Encoder.
- **Preset System:** Cụm Node được lưu dưới dạng `.preset.json` kèm `exposed_inputs` để dễ dàng nạp, tái sử dụng và reset thông số.
