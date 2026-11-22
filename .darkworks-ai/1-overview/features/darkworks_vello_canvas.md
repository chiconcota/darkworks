# Feature: Vello Canvas 2D GPU Node (DW-04)
@origin: ⚡ Darkworks Core Exclusive | @engine: vello + cosmic-text | @status: Trọng tâm cốt lõi (Giai đoạn 4)

Generator Node đồ họa vector 2D chạy trực tiếp bằng Compute Shader trên VRAM GPU, thay thế hoàn toàn việc nhúng trình duyệt web (Chromium/WebKit) để tiết kiệm 1GB RAM và triệt tiêu độ trễ chuyển pixel về CPU.

## Sub-features
- **Vello GPU Compute Pipeline:** Render đường cong Bézier, hình khối và font glyphs trực tiếp sang `wgpu::Texture` (Zero-copy).
- **Karaoke Subtitle Generator:** Hiệu ứng chữ Karaoke nhảy đổi màu (gradient highlight) theo từng từ ăn khớp với nhịp phát âm dựa trên đơn vị `Tick`.
- **Vector & SVG Generator:** Hiển thị hình khối vector (Rectangle, Circle, Star) và nạp file `.svg` sắc nét vô cực ở mọi độ phân giải (từ 1080p đến 8K).
- **Lower-Third & Infographic Composites:** Đóng gói cụm nhiều layer chữ và hình khối thành bảng tin tức hoàn chỉnh.

## How to get to it (user POV)
1. Trong panel Node Graph, thêm Node `Text / Subtitle` hoặc `Vector Shape`.
2. Gõ nội dung văn bản trong bảng tham số, chọn font chữ, kích cỡ và màu sắc.
3. Bật tùy chọn `Karaoke Mode` và liên kết với file timestamp phụ đề.
4. Nối output texture của Canvas 2D Node vào Blend Node đè lên video nền.

## Driving it with <harness>
```python
# Thêm một Vello Text Node vào đồ thị
drive_command("nodegraph.addNode", {
    "clip_id": "clip_01",
    "node_type": "CanvasText",
    "params": {
        "text": "Darkworks NLE Video Editor",
        "font_family": "DejaVu Sans",
        "font_size": 48.0,
        "color": [1.0, 1.0, 1.0, 1.0],
        "stroke_width": 2.0
    }
})
```

## Gotchas
- Tuyệt đối cấm sử dụng Chromium / CEF / WebKit nhúng; mọi tác vụ render 2D phải đi qua Vello compute shader trên GPU.
