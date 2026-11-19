# SỔ BÀN GIAO TIẾN ĐỘ CA KÍP: DARKWORKS (checkpoint.md)
@project: DARKWORKS | @phòng_ban: Toàn xưởng | @status: Hoàn tất khởi tạo & Sẵn sàng code Giai đoạn 1
@session_end: 2026-11-19 | @next_session_focus: Khởi tạo Cargo Workspace & Crate crates/ui

> Sổ ghi chép bàn giao công việc giữa các phiên làm việc của AI.  
> Khi mở phiên mới (/start_session), nhân viên đọc file này để nắm tiến độ dở dang trong 3 giây.

---

## 1. THÔNG TIN MÔI TRƯỜNG & TRẠNG THÁI HIỆN TẠI
- **Môi trường:** Linux x86_64, Rust toolchain, Git.
- **Tình trạng:** Đã hoàn thành 100% việc thiết lập buồng lái AI Agent và tài liệu kiến trúc cho dự án Darkworks.
- **Mã nguồn tham khảo:** Đã clone kho nguyên bản FilmCraft tại [reference/filmcraft](file:///mnt/Data/darkworks/reference/filmcraft) và cách ly an toàn trong [.gitignore](file:///mnt/Data/darkworks/.gitignore).

---

## 2. DANH SÁCH FILE ĐÃ THIẾT LẬP / CẬP NHẬT TRONG PHIÊN
- **Cấu hình & Quy chế:**
  - [AGENTS.md](file:///mnt/Data/darkworks/AGENTS.md) và [.agents/AGENTS.md](file:///mnt/Data/darkworks/.agents/AGENTS.md): Bản quy chế & chỉ dẫn toàn diện của Darkworks.
  - [.gitignore](file:///mnt/Data/darkworks/.gitignore): Loại trừ `target/` và `/reference/`.
- **Buồng lái [.agents/](file:///mnt/Data/darkworks/.agents):**
  - Rules: [rust-architect.md](file:///mnt/Data/darkworks/.agents/rules/rust-architect.md), [darkworks-system-rules.md](file:///mnt/Data/darkworks/.agents/rules/darkworks-system-rules.md), [docs-management.md](file:///mnt/Data/darkworks/.agents/rules/docs-management.md), [improve.md](file:///mnt/Data/darkworks/.agents/rules/improve.md).
  - Skills: `start_session`, `end_session`, `debug-workflow`, `release-github`.
  - Workflows: `start_session.md`, `end_session.md`, `debug-workflow.md`, `release-github.md`.
  - Harness: [README.md](file:///mnt/Data/darkworks/.agents/harness/README.md) cẩm nang lệnh kiểm toán Rust.
- **Bộ nhớ [.darkworks-ai/](file:///mnt/Data/darkworks/.darkworks-ai):**
  - [1-overview/system_map.md](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/system_map.md): Bản đồ kiến trúc hệ thống Darkworks.
  - [1-overview/project-managers/phase-1-media-player.md](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/project-managers/phase-1-media-player.md): Kế hoạch chi tiết Giai đoạn 1.
  - [2-memory/decision-log.md](file:///mnt/Data/darkworks/.darkworks-ai/2-memory/decision-log.md): 8 quyết định nền tảng đã chốt.
  - [2-memory/self-improve.md](file:///mnt/Data/darkworks/.darkworks-ai/2-memory/self-improve.md): Sổ ghi lỗi hành vi.
  - [3-modules/](file:///mnt/Data/darkworks/.darkworks-ai/3-modules): 7 file architecture.md cho `schema`, `media`, `node-graph`, `canvas-renderer`, `automation`, `ai-workers`, `ui`.

---

## 3. CÁC VẤN ĐỀ ĐÃ GIẢI QUYẾT & BÀI HỌC KINH NGHIỆM (RESOLVED)
1. **Xóa bỏ hoàn toàn tàn dư Agency làm web:** Đã loại bỏ các file `company-rules.md`, `client-intake`, `assembly-delivery`, `wp-architect.md`, `skaaa-nocode-system.md` và thay bằng bộ quy chuẩn Native Rust.
2. **Chuẩn hóa cây thư mục:** Gom sạch về 1 thư mục `.agents/` và cấu trúc bộ nhớ 3 ngăn kéo `.darkworks-ai/` (`1-overview`, `2-memory`, `3-modules`).
3. **Định vị chính xác Rule #1:** Đã xác nhận file Global Rule PC tại [~/.gemini/GEMINI.md](file:///home/chiconcota/.gemini/GEMINI.md).
4. **Chiến lược giữ 93 Shader FilmCraft:** Đã chốt phương án "Bọc ngoài & Thay ruột" biến effect thành Atomic Node thay vì xóa thô bạo.

---

## 4. KẾ HOẠCH BÀN GIAO CA TIẾP THEO (NEXT ACTIONS CHECKLIST)
Khi mở phiên kế tiếp (`/start_session`), Agent và Người Dùng sẽ bắt đầu triển khai code **Giai đoạn 1**:
- [ ] **Bước 1:** Khởi tạo Git repository cho `darkworks` (nếu chưa có).
- [ ] **Bước 2:** Khởi tạo `Cargo.toml` gốc cho workspace.
- [ ] **Bước 3:** Tạo crate `crates/ui` và dựng cửa sổ giao diện 4 khu vực đầu tiên bằng `egui + wgpu`.
- [ ] **Bước 4:** Tham chiếu sang `reference/filmcraft/crates/media` để chuẩn bị đưa lõi giải mã video phần cứng vào `crates/media`.
