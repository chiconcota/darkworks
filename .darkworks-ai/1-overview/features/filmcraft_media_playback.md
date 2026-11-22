# Feature: Media Playback & VA-API Hardware Decode (FC-01)
@origin: FilmCraft Foundation | @layer: L5 (Platform) + L3 (Media) | @status: Kế thừa 100%

Hệ thống demuxer container (MP4, MOV, MKV, WebM, ProRes) và giải mã video phần cứng OS Native (Linux VA-API / NVDEC, macOS VideoToolbox) với cơ chế Ring-Buffer LRU cache đảm bảo RAM luôn < 2GB.

## Sub-features
- **Native Demuxer:** Đọc trực tiếp ISOBMFF (MP4/MOV) và Matroska (MKV) thuần Rust, không qua FFmpeg CLI.
- **Hardware VA-API Decoder:** Ánh xạ khung hình H.264/HEVC trực tiếp sang `wgpu::Texture` qua DMA-BUF / DRM.
- **FrameCache LRU Ring-Buffer:** Giới hạn cứng dung lượng khung hình giải mã < 1024MB (video cache) / trần toàn app < 2048MB.
- **Dual Monitor Playback:** Source Viewer (preview clip trong bin) và Program Viewer (preview timeline).

## How to get to it (user POV)
1. Kéo thả file video (`.mp4`, `.mov`, `.mkv`) vào panel Project / Media Bin.
2. Double click vào clip trong bin để nạp lên Source Monitor.
3. Kéo clip xuống Timeline để xem trên Program Monitor.
4. Bấm `Spacebar` để Play/Pause, phím `J-K-L` để tua lùi / dừng / tua nhanh (Shuttle speed).

## Driving it with <harness>
```python
# Mở một media file qua control channel
drive_command("engine.execute", {
    "command": "project.importFile",
    "params": {"path": "/mnt/Data/darkworks/assets/tests/sample.mp4"}
})

# Điều khiển playback trên Playhead
drive_command("engine.execute", {
    "command": "playback.toggle"
})

# Tua đến vị trí tick xác định (ví dụ giây thứ 5 = 5 * 254_016_000_000 ticks)
drive_command("engine.execute", {
    "command": "timeline.seek",
    "params": {"tick": 5 * 254016000000}
})
```

## Gotchas
- Trên Linux không có card Intel/AMD hỗ trợ VA-API hoặc thiếu `/dev/dri/renderD128`, hệ thống sẽ tự động fallback sang phần mềm decoder an toàn (`filmcraft_codecs`).
- Tuyệt đối không spawn subprocess `ffmpeg` bên ngoài; mọi thao tác giải mã phải diễn ra qua pipeline nội bộ.
