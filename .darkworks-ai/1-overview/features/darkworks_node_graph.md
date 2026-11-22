# Feature: DAG Node Graph Engine & Presets (DW-01)
@origin: ⚡ Darkworks Core Exclusive | @engine: egui_snarl + wgpu | @status: Trọng tâm cốt lõi (Giai đoạn 3)

Mạng lưới đồ thị có hướng không chu trình (Directed Acyclic Graph) thay thế hoàn toàn danh sách hiệu ứng tuyến tính tuần tự cũ kỹ, biến 93 shader WGSL thành các Node nguyên tử nối dây trực quan.

## Sub-features
- **Visual Node Canvas:** Bảng đồ thị `egui_snarl` với khả năng zoom/pan vô cực, kết nối socket bằng dây cong Bézier.
- **Parallel Branching & Multipass:** Cho phép rẽ nhánh luồng hình ảnh, áp dụng nhiều hiệu ứng song song và gộp lại qua Blend Node (hỗ trợ 26 blend modes).
- **93 WGSL Shaders as Atomic Nodes:** Toàn bộ 93 shader của FilmCraft được bọc thành các Node độc lập (Transform, Blur, Glow, Chromatic, Color Grade, Keyer...).
- **Preset System (.preset.json):** Đóng gói toàn bộ cụm đồ thị phức tạp thành một file JSON duy nhất để chia sẻ và tái sử dụng.

## How to get to it (user POV)
1. Chọn một Clip trên Timeline và chuyển sang panel **Node Graph** (thay thế panel Effect Controls cũ).
2. Click chuột phải trên Canvas để mở Node Palette, chọn Node cần thêm (ví dụ: `Blur`, `Transform`, `Blend`).
3. Kéo chuột từ socket `Output (Texture)` của Node Video Source vào socket `Input` của Node Blur.
4. Kéo socket `Output` của Node Blur vào `Node Output` để thấy kết quả trên Viewport.

## Driving it with <harness>
```python
# Tạo một Node mới trong đồ thị của clip đang chọn
drive_command("nodegraph.addNode", {
    "clip_id": "clip_01",
    "node_type": "GaussianBlur",
    "position": [200.0, 150.0]
})

# Nối dây giữa 2 socket
drive_command("nodegraph.connect", {
    "clip_id": "clip_01",
    "from_node": "source_node",
    "from_socket": "texture_out",
    "to_node": "blur_node",
    "to_socket": "texture_in"
})

# Nạp một preset đồ thị
drive_command("nodegraph.loadPreset", {
    "clip_id": "clip_01",
    "preset_path": "assets/presets/glow_vignette.preset.json"
})
```

## Gotchas
- Đồ thị phải luôn bảo đảm tính phi chu trình (DAG); nếu phát hiện người dùng kéo dây tạo vòng lặp (cycle loop), hệ thống phải từ chối nối dây và cảnh báo đỏ.
- Mọi Texture luân chuyển giữa các node phải thực hiện 100% trên VRAM (Zero CPU Readback).
