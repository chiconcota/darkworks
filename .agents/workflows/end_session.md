---
description: Kết thúc phiên làm việc Darkworks (Kiểm tra compile Rust, Cập nhật 3 ngăn kéo, Niêm phong Checkpoint bàn giao)
---

Bước 1. **Kiểm toán mã nguồn Rust (Sanity & Compile Check):**
   - Chạy `cargo check --workspace --all-targets` và `cargo clippy --workspace --all-targets -- -D warnings`.
   - Đảm bảo không có lỗi biên dịch và không lọt `.unwrap()`, `panic!()` vào code production.

Bước 2. **Cập nhật Crate Documentation (Kiến trúc cục bộ):**
   - Mở thư mục `.darkworks-ai/3-modules/[tên-crate]/architecture.md`.
   - Cập nhật struct mới, hàm mới hoặc luồng render mới vừa code xong.

Bước 3. **Lập Biên Bản Bàn Giao Ca Trực (`.darkworks-ai/2-memory/checkpoint.md`):**
   - Ghi đè toàn bộ nội dung mới vào `checkpoint.md` theo đủ 4 mục chuẩn:
     1. **Thông tin Nhánh Git & Trạng thái hiện tại.**
     2. **Danh sách file thay đổi kèm liên kết markdown.**
     3. **Các lỗi đã giải quyết trong phiên (Resolved Issues).**
     4. **Kế hoạch bàn giao ca kế tiếp (Next Actions Checklist):** Danh sách công việc cụ thể để Agent phiên sau vào là làm được ngay.

Bước 4. **Ghi Sổ Quyết Định & Cập Nhật Bản Đồ Hệ Thống:**
   - Cập nhật `.darkworks-ai/2-memory/decision-log.md` nếu có quyết định kiến trúc mới.
   - Cập nhật `.darkworks-ai/1-overview/system_map.md` (chuyển trạng thái module từ 🔴 ➔ 🟡 ➔ 🟢 và ghi nhật ký ngắn).

Bước 5. **Đánh giá phiên làm việc & Bài học kinh nghiệm:**
   - Dự thảo bài học kinh nghiệm mới (nếu có) và xin phép Người Dùng trước khi cập nhật vào `.agents/rules/improve.md`.
   - Hỏi ý kiến Người Dùng về việc `git commit` và `git push`.
