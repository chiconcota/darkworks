---
name: end_session
description: Kết thúc ca làm việc của Darkworks (Kiểm tra compile Rust, Cập nhật 3 ngăn kéo, Niêm phong Checkpoint bàn giao)
---

# KẾT THÚC CA LÀM VIỆC (/end_session)
@trigger: Khi kết thúc ca làm việc hoặc hoàn thành một cột mốc kỹ thuật

> Mục tiêu: Kiểm tra mã nguồn Rust không có lỗi biên dịch, tổng hợp các thay đổi kiến trúc và niêm phong tiến độ vào `.darkworks-ai/2-memory/checkpoint.md` để ca sau tiếp quản trong 3 giây.

---

## BƯỚC 1: KIỂM TOÁN MÃ NGUỒN (RUST AUDIT & SANITY CHECK)
Chạy các lệnh kiểm tra tự động trước khi đóng phiên:
```bash
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```
- Nếu có lỗi biên dịch hoặc cảnh báo clippy, giải quyết dứt điểm trước khi bàn giao.
- Xác nhận không có `.unwrap()` hay `panic!()` lọt vào mã nguồn mới.

---

## BƯỚC 2: CẬP NHẬT TỦ TÀI LIỆU (TỐI ĐA 4 ĐIỂM CHẠM)
1. **Cập nhật Crate Architecture:**
   - Mở `.darkworks-ai/3-modules/[tên-crate]/architecture.md` để ghi nhận các struct, hàm mới hoặc thay đổi luồng xử lý vừa hoàn thành.
2. **Ghi Sổ Quyết Định:**
   - Mở `.darkworks-ai/2-memory/decision-log.md` để chèn thêm các quyết định kỹ thuật mới (nếu có).
3. **Cập nhật Bản Đồ Hệ Thống:**
   - Mở `.darkworks-ai/1-overview/system_map.md` để cập nhật trạng thái crate (🔴 ➔ 🟡 ➔ 🟢) và ghi chép dòng nhật ký vắn tắt tại `Recent Logs`.
4. **Niêm Phong Sổ Bàn Giao (QUAN TRỌNG NHẤT):**
   - Mở và GHI ĐÈ vào `.darkworks-ai/2-memory/checkpoint.md`:
     - Trạng thái công việc hiện tại.
     - Các file đã tạo / chỉnh sửa.
     - Việc cụ thể cần làm tiếp theo khi mở ca sau.
     - Nhánh Git hiện tại.

---

## BƯỚC 3: ĐÁNH GIÁ PHIÊN & CẬP NHẬT BÀI HỌC KINH NGHIỆM
- Nếu trong ca phát sinh lỗi thao tác, nhầm lẫn kiến trúc hoặc giải pháp thất bại:
  - Dự thảo ngắn gọn bài học rút ra và hỏi ý kiến Người Dùng.
  - Khi được Người Dùng đồng ý, cập nhật bài học mới vào [.agents/rules/improve.md](file:///mnt/Data/darkworks/.agents/rules/improve.md).

---

## BƯỚC 4: XÁC NHẬN KẾT THÚC
- Hỏi ý kiến Người Dùng về việc `git commit` và `git push`.
- Thông báo rõ ràng: "Sổ bộ nhớ Darkworks đã được niêm phong an toàn. Ca làm việc kết thúc!"
