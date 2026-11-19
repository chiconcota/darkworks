# MODULE KIẾN TRÚC: USER INTERFACE (ui/architecture.md)
@crate: `crates/ui` (hoặc `ui-egui`) | @layer: L6 / Presentation & Interaction

## 1. VAI TRÒ & TRÁCH NHIỆM
Chịu trách nhiệm hiển thị giao diện đồ họa 60fps siêu mượt bằng `egui`, chia làm 4 phân vùng làm việc chuẩn của một NLE chuyên nghiệp.

## 2. 4 PHÂN VÙNG GIAO DIỆN
1. **Source / Project Bin (Góc trên bên trái):** Quản lý tài nguyên media (video gốc, âm thanh, ảnh, preset).
2. **Viewport (Góc trên ở giữa/phải):** Khung xem trước (Preview) render trực tiếp từ GPU `wgpu::Texture`.
3. **Inspector / Node Graph Panel (Bên phải):**
   - Chế độ cơ bản: Bảng trượt thông số đơn giản (Inspector) dựa trên các `exposed_inputs` của Preset.
   - Chế độ nâng cao: Nút bấm [Open Node Graph] mở cửa sổ mạng lưới `egui_snarl` để nối dây chi tiết.
4. **Timeline (Nửa dưới màn hình):**
   - Hiển thị các Track Video, Audio, Canvas.
   - Thao tác chuột mượt mà: Scrubbing (tua nhanh), Split (phím S), Ripple Delete (phím X), kéo dãn biên clip.
   - Thanh công cụ nhanh: ⚡ Auto Cut Silence, 💬 Auto Subtitle.
