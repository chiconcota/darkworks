# Feature: AI Local Silence Cut & Subtitles (DW-05)
@origin: ⚡ Darkworks Core Exclusive | @models: Silero VAD (ONNX) + Whisper/Moonshine | @status: Giai đoạn 5

Hệ thống trí tuệ nhân tạo chạy hoàn toàn Offline trên máy nội bộ: băm sạch khoảng lặng video/audio bằng Silero VAD và nhận diện giọng nói tự động tạo phụ đề bằng Whisper.

## Sub-features
- **Silero VAD (Voice Activity Detection):** Dò tìm chính xác các đoạn im lặng trong track âm thanh với độ phân giải mili-giây.
- **⚡ Auto Cut Silence:** Lệnh 1-click tự động thực hiện Extract / Ripple Delete toàn bộ các đoạn im lặng trên timeline dưới 5 giây.
- **Offline Whisper Transcription:** Nhận diện giọng nói tiếng Việt và tiếng Anh, tự động sinh các clip Subtitle trên timeline.
- **Antigravity MCP Integration:** Cổng giao tiếp cho AI Agent điều khiển toàn bộ phần mềm từ terminal thông qua `rmcp`.

## How to get to it (user POV)
1. Chọn một clip video/audio có giọng nói trên Timeline.
2. Click vào nút `⚡ Auto Cut Silence` trên thanh công cụ Timeline.
3. Trong vòng 2-5 giây, toàn bộ các khoảng dừng nghỉ, ậm ừ sẽ tự động bị cắt gọt và timeline được dồn lại liền mạch.
4. Nhấn nút `Generate Subtitles` để tự động tạo track phụ đề bên dưới.

## Driving it with <harness>
```python
# Kích hoạt lệnh Auto Cut Silence qua MCP channel
drive_command("ai.autoCutSilence", {
    "track_id": "A1",
    "threshold_db": -40.0,
    "min_silence_duration_ms": 400
})

# Sinh phụ đề bằng Whisper
drive_command("ai.generateSubtitles", {
    "track_id": "A1",
    "language": "vi"
})
```

## Gotchas
- Mô hình ONNX phải chạy trên luồng ngầm (`ai-workers`), tuyệt đối không được block luồng giao diện chính (UI Thread).
