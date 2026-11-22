# Feature: Procedural Motion & Audio Reactivity (DW-02)
@origin: ⚡ Darkworks Core Exclusive | @engine: Node Graph + Audio DSP | @status: Giai đoạn 3

Tính năng kết nối trực tiếp tín hiệu âm thanh thời gian thực (Audio RMS Peak, Pitch, Frequency Bands) từ Track Audio vào các socket thông số của Node Graph, tạo ra chuyển động video tự động giật theo điệu nhạc.

## Sub-features
- **Audio Output Sockets:** Node Audio Source xuất ra các socket giá trị số: `Peak RMS`, `Bass Level`, `Treble Level`, `Pitch`.
- **Value Remapping Node:** Chuyển đổi dải biên độ âm thanh (ví dụ: `0.0..1.0`) sang dải thông số video (ví dụ: Scale `1.0..1.2` hoặc Blur `0..20px`).
- **Procedural Motion Effects:** Tự động tạo hiệu ứng video rung lắc (Camera Shake), viền phát sáng đổi màu (Glow React), và Zoom nảy theo tiếng đập Bass mà không cần đặt keyframe thủ công.

## How to get to it (user POV)
1. Thêm một Node `Audio React` vào bảng Node Graph.
2. Nối dây từ cổng `Audio Peak` của Audio Node vào socket `Scale` của Node Transform.
3. Bấm `Spacebar` phát nhạc, video trên Viewport sẽ tự động phóng to thu nhỏ theo nhịp điệu bài hát.

## Driving it with <harness>
```python
# Nối tín hiệu âm thanh vào thông số video
drive_command("nodegraph.connect", {
    "clip_id": "clip_01",
    "from_node": "audio_react_node",
    "from_socket": "peak_out",
    "to_node": "transform_node",
    "to_socket": "scale_uniform"
})
```

## Gotchas
- Tín hiệu âm thanh phải được chuẩn hóa (normalized) và lọc làm mịn (smoothing filter) để tránh hiện tượng rung hình giật cục khó chịu.
