# Feature: Audio Mixer & Master Sync Clock (FC-03)
@origin: FilmCraft Foundation | @layer: L3 (Audio DSP) + L5 (cpal) | @status: Kế thừa 100%

Bộ trộn âm thanh đa kênh thời gian thực sử dụng backend `cpal` (PipeWire / ALSA trên Linux), đồng bộ khóa cứng A/V Sync với Audio Clock làm Master.

## Sub-features
- **Audio Master Clock:** Playhead bám trực tiếp theo số frame âm thanh phần cứng tiêu thụ, triệt tiêu hiện tượng lệch hình lệch tiếng.
- **Monotonic Audio Feed Head:** Liên tục tịnh tiến đệm âm thanh về phía trước, loại bỏ 100% tiếng rè đứt gãy pha (Phase discontinuity).
- **Audio Mixer Panel:** Điều chỉnh Fader âm lượng từng track, Pan cân bằng trái/phải, Mute / Solo.
- **Real-time VU Meters:** Đồng hồ đo cường độ âm thanh RMS & Peak thời gian thực chuẩn broadcast (-60dBFS đến 0dBFS).

## How to get to it (user POV)
1. Mở panel Audio Mixer từ menu Window ▸ Audio Mixer.
2. Nhấn `Spacebar` phát timeline có chứa clip âm thanh.
3. Quan sát các cột đèn VU nhấp nháy xanh/vàng/đỏ.
4. Kéo thanh trượt Volume trên track A1 để thay đổi âm lượng.

## Driving it with <harness>
```python
# Đặt âm lượng track A1 về -6.0 dB
drive_command("engine.execute", {
    "command": "mixer.setTrackVolume",
    "params": {"track": "A1", "gain_db": -6.0}
})

# Mute track A2
drive_command("engine.execute", {
    "command": "mixer.toggleMute",
    "params": {"track": "A2"}
})
```

## Gotchas
- Không tải toàn bộ file âm thanh dài 1 tiếng vào RAM (ngốn 1.2GB); giải mã đệm on-demand từng block 3 giây để giữ trần RAM < 200MB.
