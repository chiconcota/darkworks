---
description: Bắt đầu ca làm việc của Darkworks (Nạp Bộ Nhớ Checkpoint, System Map, Khảo Sát Crate & Sẵn Sàng Nhận Lệnh)
---

1. **Khởi động bộ nhớ (Context Loading):**
   - Đọc file `.darkworks-ai/2-memory/checkpoint.md` LÀ BƯỚC ĐẦU TIÊN để nhận bàn giao trạng thái công việc, Git branch, và bug đang làm dở từ phiên trước.
   - Đọc file `.darkworks-ai/1-overview/system_map.md` để nắm cấu trúc tổng thể và tiến độ các Crate.
   - Đọc file `.darkworks-ai/2-memory/decision-log.md` (nắm các quyết định kiến trúc bất biến đã chốt).
   - Đọc file `.agents/rules/improve.md` để nạp danh sách lỗi hành vi cần tránh và quy tắc an toàn.

2. **Xác định tiêu điểm & Nhánh làm việc (Focus & Git Branch Decision):**
   - Kiểm tra nhánh Git hiện tại (`git branch` hoặc `git status`).
   - Cùng người dùng xác định Crate mục tiêu cần can thiệp trong `crates/`.
   - Quyết định làm việc trên nhánh `main` hay tạo nhánh tính năng mới (`feature/ten-tinh-nang`).

3. **Load Crate Memory (Context-Switching):**
   - Dựa vào Crate mục tiêu, đọc tài liệu trong thư mục `.darkworks-ai/3-modules/[tên-crate]/architecture.md`.
   - Tuyệt đối không đọc lộn tài liệu của Crate không liên quan để tránh tràn ngữ cảnh.

4. **Sẵn sàng (Ready Check):**
   - Phản hồi ngắn gọn: "Đã nạp kiến trúc Darkworks (system_map.md) và Sổ tự cải thiện (improve.md). Đang làm việc trên nhánh Git: [Tên Nhánh]. Đang cô lập vùng làm việc vào Crate: [Tên Crate]. Sẵn sàng nhận lệnh."
