# Feature: Multi-Track Timeline & NLE Core (FC-02)
@origin: FilmCraft Foundation | @layer: L4 (Project/Edit) + L6 (UI) | @status: Kế thừa 100%

Hệ thống trục thời gian đa track chuẩn NLE với độ chính xác khung hình theo Tick nguyên thủy, hỗ trợ Snapping, Ripple Edit, Trim và lưu trữ trạng thái bất biến qua `Arc<Project>`.

## Sub-features
- **Multi-Track Stacking:** Hỗ trợ không giới hạn track Video (V1, V2, V3...) và track Audio (A1, A2, A3...).
- **Deterministic Tick Time:** Trục thời gian tính toán hoàn toàn bằng số nguyên `Tick` (`1 giây = 254_016_000_000 ticks`), loại bỏ sai số trôi khung hình của số thực `f64`.
- **Snapping Engine:** Tự động hút dính biên clip, Playhead và Marker với bán kính 8px.
- **Instant Undo/Redo:** Cơ chế Copy-on-Write `Arc<Project>`: `Ctrl+Z` (Undo) và `Ctrl+Y` (Redo) tức thì với chi phí RAM cực thấp.

## How to get to it (user POV)
1. Mở panel Timeline ở góc dưới giao diện.
2. Dùng chuột kéo giữ clip để di chuyển giữa các track và vị trí thời gian.
3. Rê chuột vào 2 đầu mép clip để Trim (kéo dài / thu ngắn thời lượng clip).
4. Nhấn `Ctrl + Z` để hoàn tác mọi thao tác vừa thực hiện.

## Driving it with <harness>
```python
# Thêm clip vào track V1 tại tick 0
drive_command("engine.execute", {
    "command": "timeline.insertClip",
    "params": {
        "track_id": "V1",
        "media_id": "clip_01",
        "start_tick": 0,
        "duration_ticks": 10 * 254016000000
    }
})

# Thực hiện Undo
drive_command("engine.execute", {"command": "history.undo"})

# Thực hiện Redo
drive_command("engine.execute", {"command": "history.redo"})
```

## Gotchas
- Khi Playhead đi qua khoảng trống (Gap giữa 2 clip), Timeline phải hiển thị khung hình màu đen thuần khiết thay vì treo giữ frame của clip trước đó.
- Bấm phím `Z` đơn lẻ là lệnh Lift của Lightworks (xóa rỗng giữ chỗ); bấm `Ctrl + Z` là lệnh Undo toàn cục.
