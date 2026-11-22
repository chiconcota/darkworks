# KẾ HOẠCH DỰ ÁN: GIAI ĐOẠN 2 (phase-2-timeline-nle.md)
@project: DARKWORKS | @phase: 2 | @status: 🟢 COMPLETED
@goal: Trục thời gian NLE đa track tích hợp cơ chế Mark In / Mark Out chuẩn Lightworks, thao tác cắt xẻ nguyên tử (Extract / Lift / Split), lưu trữ project.json và Hot-Reload thời gian thực.

---

## 1. MỤC TIÊU CỐT LÕI (CORE DELIVERABLES)

1. **Cơ Chế Vùng Chọn Thời Gian Chủ Động (Timeline Mark In / Mark Out System):**
   - Triệt để áp dụng mô hình dựng phim kinh điển của **Lightworks**: Biên tập viên làm chủ dòng thời gian thông qua 2 mốc `Mark In` và `Mark Out`.
   - Mọi thao tác cắt xẻ, xóa dồn, tạo khoảng trống, chèn layer chữ/đồ họa hay áp preset hiệu ứng đều tác động trực tiếp trên vùng `[In, Out]` này.
2. **Lõi Toán Học & Thao Tác Cắt Ghép Nguyên Tử (Atomic Operations trong `crates/schema`):**
   - **Tuyệt đối không hardcode tính năng phức hợp:** Không tạo nút bấm nguyên khối "Auto Cut Silence" hay logic ngầm đơn lẻ. Toàn bộ logic được chia nhỏ thành các hàm toán học nguyên tử (Atomic Primitives) trên `Tick`:
     - `split_clip_at`: Cắt đôi clip tại `Tick` bất kỳ, bảo toàn `source_in` và thời lượng.
     - `clear_track_range`: Tự động tìm giao cắt, băm clip tại 2 đầu mốc và xóa sạch các phân đoạn lọt vào khoảng `[In, Out]`.
     - `shift_track_from`: Tịnh tiến lùi/tiến các clip phía sau để dồn timeline hoặc mở khoảng trống.
     - `extract_range` (Xóa dồn / Ripple Delete theo vùng): Cắt và xóa đoạn `[In, Out]` đồng thời dồn timeline về trước.
     - `lift_range` (Xóa rỗng giữ chỗ theo vùng): Cắt và xóa đoạn `[In, Out]` nhưng giữ nguyên khoảng trống đen (Gap).
3. **Giao Diện Trục Thời Gian Tương Tác 60 FPS (`crates/ui`):**
   - **Hệ thống phím tắt chuẩn Lightworks:**
     - `I`: Đánh dấu Mark In tại Playhead.
     - `O`: Đánh dấu Mark Out tại Playhead.
     - `P`: Xóa bỏ 2 mốc In/Out (Clear Marks).
     - `X`: Extract (Xóa dồn vùng In/Out - Ripple Delete Range).
     - `Z`: Lift (Xóa rỗng vùng In/Out - Lift Range).
     - `S` hoặc `C`: Cắt dao cạo nhanh (Razor Split) tại Playhead.
   - **Trực quan hóa chuẩn xác:**
     - Vẽ dải sáng phủ màu xanh lơ trong suốt (Semi-transparent Highlight Overlay) từ `In` đến `Out` xuyên suốt toàn bộ các Track (V2, V1, A1, A2).
     - Hiển thị 2 mốc ngoặc vuông `[` (In) và `]` (Out) trên thước đo Ruler, cho phép click kéo chuột để co dãn vùng chọn trực tiếp.
4. **Bộ Điều Phối Multi-Track Playhead Evaluator:**
   - Quét các Track theo thứ tự ưu tiên (V2 Canvas/Text ở trên, V1 Video ở dưới) để trích xuất khung hình tương ứng tại vị trí `playhead` đưa lên Viewport GPU.
   - Mix các luồng âm thanh giao cắt tại `playhead` (A1 Dialogue, A2 Music).
5. **Trực Quan Hóa Sóng Âm (Audio Waveform Display):**
   - Trích xuất mảng mẫu đỉnh biên độ (Peak RMS) từ file âm thanh nguồn.
   - Vẽ đường sóng âm thời gian thực bên trong thân Clip trên các Track Audio, giúp người dùng dễ dàng bắt nhịp câu nói để đặt Mark In / Mark Out chính xác từng frame.
6. **Định Dạng Mở `project.json` & File Watcher Hot-Reload:**
   - Lưu trữ toàn vẹn trạng thái dự án (gồm cả mảng Tracks, Clips, `mark_in`, `mark_out`).
   - Lắng nghe sự kiện file thời gian thực: AI Agent hoặc công cụ CLI ngoài chỉnh sửa file `project.json` ➔ Timeline trên giao diện tự động cập nhật ngay lập tức mà không cần khởi động lại app.

---

## 2. CHECKLIST CÁC ĐẦU VIỆC CỤ THỂ (TASKS BREAKDOWN)

- [x] **Task 2.1 [Lõi Toán Học Cắt Xẻ Nguyên Tử trong `crates/schema`]:**
  - Mở rộng cấu trúc dữ liệu:
    - Bổ sung `pub mark_in: Option<Tick>` và `pub mark_out: Option<Tick>` vào struct `Project`.
    - Viết hàm `Project::active_range(&self, playhead: Tick) -> Option<TimeRange>`.
  - Triển khai các hàm toán học nguyên tử trên `Track` và `Project`:
    - `split_clip_at(&mut self, track_id: &str, clip_id: &str, at: Tick) -> Result<(ClipSlice, ClipSlice), SchemaError>`
    - `clear_track_range(&mut self, track_id: &str, range: TimeRange) -> Result<Vec<ClipSlice>, SchemaError>`
    - `shift_track_from(&mut self, track_id: &str, at: Tick, delta: Tick) -> Result<(), SchemaError>`
    - `extract_range(&mut self, range: TimeRange, target_tracks: Option<&[&str]>) -> Result<(), SchemaError>`
    - `lift_range(&mut self, range: TimeRange, target_tracks: Option<&[&str]>) -> Result<(), SchemaError>`
  - Viết unit tests kiểm thử độ chính xác 100% trên `Tick` nguyên thủy: không trôi frame, không tràn số, bảo toàn đồng bộ âm/hình.

- [x] **Task 2.2 [Giao Diện Tương Tác & Phím Tắt Lightworks trong `crates/ui`]:**
  - Lắng nghe sự kiện phím tắt trong `crates/ui/src/panels/timeline.rs`:
    - Nhấn `I` ➔ Gán `project.mark_in = Some(*playhead)`.
    - Nhấn `O` ➔ Gán `project.mark_out = Some(*playhead)`.
    - Nhấn `P` ➔ Xóa sạch `mark_in = None`, `mark_out = None`.
    - Nhấn `X` ➔ Thực thi lệnh `extract_range` trên vùng active range.
    - Nhấn `Z` ➔ Thực thi lệnh `lift_range` trên vùng active range.
    - Nhấn `S` / `C` ➔ Cắt dao cạo tại `playhead`.
  - Vẽ dải Highlight vùng In/Out:
    - Vẽ dải màu phủ trên Ruler và phủ xuyên qua các track.
    - Vẽ 2 dấu ngoặc `[` và `]` trên Ruler; hỗ trợ bắt chuột kéo di chuyển mốc In/Out.
  - Hỗ trợ kéo di chuyển clip trên track và co dãn biên (Trim In / Trim Out handles) kèm cơ chế Snapping.

- [x] **Task 2.3 [Bộ Điều Phối Multi-Track Playhead Evaluator]:**
  - Cập nhật luồng hiển thị trong `crates/ui/src/app.rs`:
    - Tại mỗi frame render, dựa theo `playhead`, duyệt từ trên xuống dưới (V2 ➔ V1) để chọn ra clip hiển thị cao nhất.
    - Chuyển đổi mượt mà giữa các clip nối tiếp nhau khi Playhead chạy qua điểm cắt mà không bị giật, đen hình hay rớt FPS.

- [x] **Task 2.4 [Trực Quan Hóa Sóng Âm Thanh (Audio Waveform)]:**
  - Trích xuất dữ liệu mẫu biên độ (RMS Peak array) trong `crates/media`.
  - Vẽ thanh sóng âm (Waveform bar graph) màu xanh lục bên trong thân Clip trên các Track Audio (A1 Dialogue, A2 Music).

- [x] **Task 2.5 [Lưu Trữ project.json & File Watcher Hot-Reload]:**
  - Bổ sung phím tắt `Ctrl + S` lưu trạng thái dự án ra file `project.json`.
  - Khởi tạo luồng ngầm File Watcher (lắng nghe file `project.json`): khi file bị chỉnh sửa từ bên ngoài (bởi AI Agent hoặc CLI) ➔ Gửi tín hiệu reload về Main UI Thread cập nhật Timeline tức thì.

- [x] **Task 2.6 [Kiểm Toán E2E, Clippy & Giới Hạn Dòng Code]:**
  - Kiểm toán kích thước toàn bộ các file Rust: tuân thủ nghiêm ngặt chuẩn `< 700 dòng code` mỗi file (tách các sub-modules nếu `timeline.rs` phình to).
  - Khóa trần `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`, 0 cảnh báo clippy.
  - Chạy toàn diện bộ kiểm thử `python3 .agents/harness/run_e2e_audit.py`.

---

## 3. TIÊU CHUẨN NGHIỆM THU GIAI ĐOẠN 2 (DEFINITION OF DONE - DoD 2)

- [x] **DoD 2.1 (Mark In / Out Chuẩn Lightworks):** Nhấn `I` và `O` tạo thành công vùng chọn thời gian; Ruler và toàn bộ các track hiển thị dải sáng nổi bật; kéo được 2 mốc `[` và `]` bằng chuột.
- [x] **DoD 2.2 (Extract / Xóa Dồn Vùng - Phím `X`):** Đánh dấu vùng `[In, Out]` bao trùm 1 hoặc nhiều clip, bấm phím `X` ➔ Toàn bộ đoạn giữa bị băm và xóa sạch, các clip phía sau dồn khít về mốc `In`, không để lại khoảng trống, video phát tiếp mượt mà.
- [x] **DoD 2.3 (Lift / Xóa Giữ Chỗ Vùng - Phím `Z`):** Đánh dấu vùng `[In, Out]`, bấm phím `Z` ➔ Đoạn giữa bị xóa nhưng giữ nguyên khoảng trống (gap).
- [x] **DoD 2.4 (Razor Split - Phím `S`/`C`):** Bấm phím `S` tại `playhead` ➔ Clip tại điểm đó bị cắt đôi chính xác từng tick nguyên thủy.
- [x] **DoD 2.5 (Audio Waveform):** Các clip trên track Audio (A1, A2) hiển thị đường sóng âm chính xác tương ứng với nhịp âm thanh trong video.
- [x] **DoD 2.6 (Hot-Reload `project.json`):** Mở trình soạn thảo bên ngoài sửa file `project.json` ➔ Giao diện Darkworks tự động cập nhật ngay trên màn hình.
- [x] **DoD 2.7 (Khóa Trần RAM & Kỷ Luật Code):** Mức RAM đo được bằng `htop` luôn `< 2GB (2048MB)`, mọi file `.rs` đạt chuẩn `< 700 dòng`, `cargo clippy --workspace --all-targets` sạch sẽ 0 lỗi.

---

## 4. MA TRẬN PHÒNG NGỪA RỦI RO & THIẾT QUÂN LUẬT

| Rủi Ro Kỹ Thuật | Nguyên Nhân Tiềm Ẩn | Biện Pháp Phòng Chống Bắt Buộc |
| :--- | :--- | :--- |
| **Gãy / Lệch Pha Âm Thanh Khi Cắt Xẻ** | Sai số số thực `f64` tích lũy khi cắt hoặc dồn timeline. | **Tuyệt đối 100% sử dụng `Tick` nguyên thủy** (`TICKS_PER_SECOND = 254_016_000_000`). Không dùng float cho logic cắt dồn. |
| **Phình Kích Thước File (`timeline.rs` chạm 700 dòng)** | Dồn cả vẽ Ruler, vẽ Tracks, bắt phím In/Out và vẽ Clips vào một file. | Tách nhỏ thành các module sạch: `timeline/ruler.rs`, `timeline/tracks.rs`, `timeline/marks.rs` nếu chạm ngưỡng 500-600 dòng. |
| **Deadlock Giữa File Watcher & UI Thread** | File Watcher ghi đè trực tiếp `project` trong khi UI đang duyệt vẽ. | Dùng `crossbeam_channel` truyền tín hiệu `ProjectReloadEvent`; UI Thread chỉ thay thế `project` ở đầu vòng lặp render, không dùng khóa Mutex chặn nhau. |
| **Bẫy Hardcode Tính Năng Phức Hợp** | Tạo nút bấm riêng cho từng tính năng AI hoặc macro. | Tuân thủ triết lý Atomic Node: Chỉ cung cấp các hàm cắt xẻ và vùng chọn In/Out nguyên thủy để sau này ráp vào Node Graph. |
