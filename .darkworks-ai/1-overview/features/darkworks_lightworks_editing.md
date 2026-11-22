# Feature: Lightworks Mark & Park Editing (DW-03)
@origin: ⚡ Darkworks Core Exclusive | @layer: Timeline UX | @status: 🟢 Hoàn thành nghiệm thu

Cấy ghép trọn vẹn triết lý biên tập phim đỉnh cao của Lightworks vào Timeline: cơ chế đánh dấu Mark & Park không cần vùng chọn chuột, phím tắt tốc độ cao và dải sáng Cyan highlight xuyên suốt các track.

## Sub-features
- **Mark & Park Shortcuts:**
  - Phím `I`: Đánh dấu Mark In.
  - Phím `O`: Đánh dấu Mark Out.
  - Phím `P`: Xóa mốc In/Out (Clear Marks).
  - Phím `X`: **Extract** (Xóa dồn - Ripple Delete vùng chọn In/Out trên các active tracks).
  - Phím `Z`: **Lift** (Xóa rỗng - Xóa vùng chọn In/Out để lại khoảng trống Gap chuẩn NLE).
  - Phím `S` hoặc `C`: **Split** (Cắt clip tại Playhead).
- **Trực quan hóa dải sáng Cyan:**
  - Dải phủ màu Cyan highlight mờ (`rgba(56, 189, 248, 25)`) phủ xuyên suốt toàn bộ chiều cao của các Track giữa mốc In và Out.
  - 2 mốc ngoặc vuông `[ IN` và `OUT ]` nổi bật trên thanh Ruler.
- **Context Menu chuột phải:** Menu ngữ cảnh thao tác nhanh (Split, Ripple Delete, Lift, Delete, Toggle Mute).

## How to get to it (user POV)
1. Đưa Playhead đến điểm bắt đầu cảnh muốn cắt, nhấn phím `I` (xuất hiện ngoặc `[ IN`).
2. Đưa Playhead đến điểm kết thúc, nhấn phím `O` (xuất hiện ngoặc `OUT ]` và dải màu Cyan highlight).
3. Nhấn phím `X` để xóa sạch đoạn này và tự động dồn các clip phía sau lại.
4. Hoặc nhấn phím `Z` để xóa đoạn này nhưng giữ nguyên khoảng trống Gap.
5. Nhấn `Ctrl + Z` để Undo ngay lập tức nếu muốn khôi phục.

## Driving it with <harness>
```python
# Đánh dấu In tại giây thứ 2
drive_command("engine.execute", {
    "command": "markers.markIn",
    "params": {"seconds": 2.0}
})

# Đánh dấu Out tại giây thứ 6
drive_command("engine.execute", {
    "command": "markers.markOut",
    "params": {"seconds": 6.0}
})

# Thực hiện lệnh Extract (Ripple Delete vùng chọn In/Out)
drive_command("engine.execute", {
    "command": "sequence.extract"
})

# Xác nhận mốc In/Out đã được dọn sạch và clip đã dồn lại
```

## Gotchas
- Phím `Z` đơn lẻ là lệnh Lift (xóa rỗng để lại khoảng trống Gap); tổ hợp `Ctrl + Z` là lệnh Undo. Phải phân định rõ ràng trên bộ lắng nghe sự kiện phím của egui.
