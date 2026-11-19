# QUY TRÌNH KIỂM THỬ ĐẦU-CUỐI (E2E TEST WORKFLOW)
@project: DARKWORKS | @file: `e2e-test-workflow.md` | @type: QA & Development Gate

Tài liệu này cung cấp kịch bản kiểm thử End-to-End (E2E) toàn diện để bạn hoặc AI Agent có thể tự động và tương tác kiểm tra độ ổn định, hiệu năng và tính năng mới trong Darkworks.

---

## ⚡ PHẦN 1: KIỂM THỬ TỰ ĐỘNG 1-CLICK (AUTOMATED GATE AUDIT)

Bất kỳ khi nào hoàn thành một tính năng mới hoặc một Phase, chạy duy nhất lệnh sau từ thư mục gốc của dự án:

```bash
python3 .agents/harness/run_e2e_audit.py
```

### 7 Cổng Kiểm Soát Tự Động Được Kích Hoạt:
1. **Hardware Node & Driver Gate:** Kiểm tra thiết bị `/dev/dri/renderD128` và runtime `libva.so.2` để xác nhận chế độ tăng tốc phần cứng.
2. **Cargo Check Workspace:** Đảm bảo toàn bộ crates (`schema`, `media`, `ui`, `darkworks`) liên kết chặt chẽ không lỗi type hay lifetime.
3. **Workspace Test Suite (Unit & E2E):** Chạy toàn bộ 10 bài test logic, bao gồm:
   - Mô phỏng vòng lặp phát 300 frames ở 60fps.
   - Stress test ép bộ đệm RAM 16MB với 50 frames 1080p, kiểm tra cơ chế giải phóng LRU Eviction.
   - Kiểm tra tính toán mốc thời gian `Tick` và định dạng Timecode `HH:MM:SS:FF`.
   - Kiểm tra trích xuất sóng âm Peak & RMS.
4. **Clippy Strict Linter:** Quét mã nguồn với chuẩn thiết quân luật `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`.
5. **Audit Kích Thước File (< 700 dòng):** Kiểm toán qua `.agents/harness/check_file_size.py`.
6. **Binary Release Build:** Biên dịch file thực thi tối ưu hóa cao `target/release/darkworks`.
7. **App Smoke Test (Launch & Adapter):** Khởi chạy ngầm binary trong 2.5 giây trên Wayland/X11 để xác nhận khởi tạo GPU adapter Vulkan/OpenGL, nạp font và đóng an toàn không crash.

---

## 🎮 PHẦN 2: KỊCH BẢN KIỂM THỬ TRỰC QUAN TRÊN GIAO DIỆN (INTERACTIVE GUI CHECKLIST)

Chạy ứng dụng trực tiếp bằng lệnh:
```bash
cargo run --release
```

Hãy thực hiện lần lượt các thao tác sau để nghiệm thu tính năng mới:

### Bước 1: Kiểm Tra Màn Hình Viewport & Phần Cứng
- [ ] Quan sát góc trên bên trái của Program Viewport: Có huy hiệu xanh **`⚡ VA-API: Hardware Accelerated`** (hoặc `⚙ CPU Fallback` nếu không có card rời).
- [ ] Màn hình Viewport hiển thị hoạt họa phát sáng màu Neon Cyan chuyển động mượt mà 60fps.

### Bước 2: Kiểm Tra Bộ Đệm RAM & LRU Eviction
- [ ] Quan sát góc trên bên phải Viewport: Thông số hiển thị dạng **`RAM: X MB / 512MB | Hits: Y`**.
- [ ] Quan sát thanh Menu Bar trên cùng: Chỉ số **`RAM: ~50-60 MB`**.
- [ ] Để ứng dụng phát liên tục vài phút: Mức RAM không được tăng dốc đứng quá 512MB và tuyệt đối không chạm trần 2GB.

### Bước 3: Kiểm Tra Bấm Phát & Phím Tắt (Transport Controls)
- [ ] **Bấm phím `Spacebar`:** Ứng dụng chuyển đổi tức thì giữa trạng thái `▶ Play` và `⏸ Pause`.
- [ ] **Bấm phím mũi tên `Trái` / `Phải`:** Thanh Playhead lùi hoặc tiến chính xác từng khung hình đơn lẻ.
- [ ] **Bấm nút `⏮` (Tua đầu) / `⏭` (Tua cuối):** Playhead nhảy ngay về đầu `00:00:00:00` hoặc cuối thời lượng timeline.
- [ ] **Kéo rê thanh Playhead (Scrubbing) trên Timeline:** Khung hình trên Viewport cập nhật tức thì theo vị trí con trỏ chuột.

### Bước 4: Kiểm Tra Âm Thanh Đồng Bộ (A/V Sync)
- [ ] Khi bấm Play: Nghe thấy âm thanh tổng hợp tần số thấp (FM synth) phát ra loa.
- [ ] Âm thanh phát mượt mà, không bị giật, rè hay tiếng nổ bụp (crackle).
- [ ] Khi bấm Pause: Âm thanh ngắt ngay lập tức, không bị trôi (echo/lag).

### Bước 5: Kiểm Tra Tương Tác 4 Phân Vùng
- [ ] **Source Bin (Trái):** Danh sách media mẫu hiển thị rõ nét, ô tìm kiếm gõ chữ mượt mà.
- [ ] **Timeline (Dưới):** Hiển thị các track V1, V2 với các clip mẫu màu cam neon và xanh dương.
- [ ] **Click chọn một Clip:** Viền clip sáng lên và thanh tiêu đề Inspector hiển thị `Đang chọn: [tên clip]`.
- [ ] **Chuyển tab Inspector sang `Node Graph (DAG)`:** Lưới đồ thị Snarl xuất hiện sẵn sàng cho việc nối dây socket.

---

## 🛠️ PHẦN 3: XỬ LÝ KHI PHÁT HIỆN LỖI (TROUBLESHOOTING)

Nếu một trong các bước trên thất bại:
1. **Nếu `run_e2e_audit.py` báo lỗi Clippy:** Xem log chi tiết dòng nào có chứa `.unwrap()` hoặc `.expect()` và chuyển sang `match` / `if let` an toàn.
2. **Nếu file nào chạm ngưỡng 700 dòng:** BẮT BUỘC dừng lại báo cáo Người Dùng để xin ý kiến quyết định phương án (không tự ý tái cấu trúc).
3. **Nếu có lỗi logic Runtime:** Kích hoạt ngay quy trình [/debug-workflow](file:///mnt/Data/darkworks/.agents/workflows/debug-workflow.md) để phân tích nguyên nhân gốc rễ (RCA) trước khi sửa code.
