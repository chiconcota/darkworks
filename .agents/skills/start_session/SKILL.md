---
name: start_session
description: Bắt đầu ca làm việc của Darkworks (Nạp Bộ Nhớ Checkpoint, System Map, Khảo Sát Crate & Sẵn Sàng Nhận Lệnh)
---

# BẮT ĐẦU CA LÀM VIỆC (/start_session)
@trigger: Khi bắt đầu ca làm việc mới hoặc chuyển đổi ngữ cảnh lớn

> Mục tiêu: Nạp nhanh tiến độ dở dang từ ca trước, xác định Crate mục tiêu, kiểm tra Git branch và sẵn sàng nhận lệnh mà không tốn token thừa.

---

## BƯỚC 1: NẠP BỘ NHỚ HỆ THỐNG (CONTEXT LOADING)
Đọc tuần tự các tệp bộ nhớ sau:

1. **Sổ Bàn Giao Ca Trước (BẮT BUỘC ĐỌC ĐẦU TIÊN):**
   - Đọc `.darkworks-ai/2-memory/checkpoint.md` để nắm các việc đã làm xong, bug đang dở dang và nhánh Git hiện tại.
2. **Bản Đồ Kiến Trúc Hệ Thống:**
   - Đọc `.darkworks-ai/1-overview/system_map.md` để xác định trạng thái các crate (🔴 / 🟡 / 🟢) và giai đoạn Roadmap hiện tại.
3. **Sổ Tay Quyết Định Cốt Lõi:**
   - Đọc phần đầu của `.darkworks-ai/2-memory/decision-log.md` (nắm các quyết định kiến trúc đã chốt bất biến).
4. **Nhật Ký Tự Cải Thiện & Sửa Sai:**
   - Đọc `.agents/rules/improve.md` để nạp các lỗi hành vi cần tránh và quy tắc an toàn.

---

## BƯỚC 2: XÁC ĐỊNH CRATE MỤC TIÊU & NHÁNH GIT (FOCUS ISOLATION)
1. **Kiểm tra Git:** Kiểm tra nhánh hiện tại (`git branch` / `git status`).
2. **Cô lập ngữ cảnh:** Xác định rõ Crate nào trong `crates/` sẽ được can thiệp trong ca này (`schema`, `media`, `node-graph`, `canvas-renderer`, `automation`, `ai-workers`, `ui`).
3. **Nạp tài liệu cục bộ:** Chỉ đọc tài liệu kiến trúc của Crate đó trong `.darkworks-ai/3-modules/[tên-crate]/architecture.md`. Tuyệt đối không đọc tràn lan các crate không liên quan.

---

## BƯỚC 3: BÁO CÁO SẴN SÀNG (READY CHECK)
Phản hồi ngắn gọn và rõ ràng:
- Đã nạp xong bộ nhớ `checkpoint.md`, `system_map.md` và `improve.md`.
- Trạng thái Git: Nhánh `[tên-nhánh]`.
- Crate mục tiêu: `[tên-crate]`.
- Việc cần làm tiếp theo theo kế hoạch bàn giao: `[nhiệm-vụ]`.
- Sẵn sàng nhận lệnh!
