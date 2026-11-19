# DARKWORKS: WORKSPACE AGENT CONFIGURATION (AGENTS.md)
Tập tin này tích hợp toàn bộ các quy chuẩn kiến trúc, thiết quân luật kỹ thuật và quy trình vận hành ca kíp của AI Agent trong dự án **Darkworks (GPU-Accelerated Node-Driven NLE Video Editor)**.

---

# PHẦN 1: THIẾT QUÂN LUẬT KIẾN TRÚC & QUY TẮC PHÁT TRIỂN

## 1. KHẨU QUYẾT TỐI THƯỢNG (THE PRIME DIRECTIVES)

1. **Khóa Trần Bộ Nhớ (RAM Cap < 2GB):**
   - Tuyệt đối không nhồi hàng ngàn frame đã giải mã vào RAM hệ thống như Blender VSE.
   - Luôn sử dụng bộ đệm vòng (Ring-Buffer LRU cache) giới hạn cứng < 2GB. Giải mã phần cứng OS native (VA-API trên Linux) đẩy thẳng khung hình lên `wgpu::Texture` (Zero-copy).

2. **Dự Án Thuần Văn Bản (Plain-Text State & Hot-Reload):**
   - Toàn bộ trạng thái Timeline, Clips và sơ đồ hiệu ứng lưu ở định dạng mở: `project.json` và `.preset.json`.
   - Cơ chế File Watcher lắng nghe thời gian thực: AI Agent hoặc công cụ CLI ngoài sửa `project.json` ➔ Timeline trên giao diện lập tức cập nhật mà không cần khởi động lại app.

3. **Mạng Lưới Đồ Thị DAG Node Graph (Thay Cho Linear Stack):**
   - Thay thế hoàn toàn danh sách hiệu ứng tuần tự (Linear Stack của Premiere/FilmCraft gốc) bằng đồ thị có hướng không chu trình (DAG) sử dụng `egui_snarl`.
   - Hỗ trợ rẽ nhánh, tái sử dụng luồng hình ảnh, kết nối tín hiệu âm thanh (Audio RMS Peak / Pitch) điều khiển thông số video theo thời gian thực (Procedural Motion).
   - Đóng gói toàn bộ cụm Node thành file `.preset.json` độc lập.

4. **Canvas 2D Node Thuần GPU (Không Nhúng Web Browser):**
   - Khai tử ý tưởng nhúng Chromium/WebKit để tránh ngốn 1GB RAM và nghẽn băng thông đọc pixel về CPU.
   - Dùng **Vello + Cosmic-text** làm một **Generator Node** bên trong Node Graph: vẽ vector, biểu đồ, infographic, phụ đề Karaoke nhảy màu trực tiếp bằng Compute Shader trên VRAM.

5. **AI-First & Điều Khiển Qua Antigravity CLI (MCP Server):**
   - Tích hợp cổng MCP Server (`rmcp`) nội bộ và giao diện dòng lệnh `darkworks-cli`.
   - Toàn bộ thao tác cắt ghép, trích xuất transcript, tạo sub, và bơm hiệu ứng đều được điều khiển thông qua các hàm lệnh chuẩn (`Session::execute`).

6. **Thời Gian Xác Định Bằng Tick Nguyên Thủy:**
   - Tuyệt đối không dùng số thực `f64` giây cho thời gian Timeline. Sử dụng số nguyên `Tick` (`1s = 254_016_000_000 ticks`) để bảo đảm độ chính xác khung hình (Deterministic Frame-by-Frame).

---

## 2. PHÂN TẦNG VÀ RANH GIỚI CRATE (CARGO LAYERS)
Trước khi code, Agent BẮT BUỘC nhận diện mã nguồn thuộc Crate nào để không lẫn lộn ranh giới:
- **`crates/schema`:** Chứa định nghĩa struct cho `project.json`, `preset.json`, `ClipSlice`. Cực nhẹ, không chứa logic nặng.
- **`crates/media`:** Lõi giải mã video phần cứng OS native (VA-API), quản lý Ring-Buffer, đồng bộ A/V Sync với `cpal`.
- **`crates/node-graph`:** Bộ khung `egui_snarl`, quản lý shader WGSL, duyệt đồ thị DAG, xuất/nạp `.preset.json`.
- **`crates/canvas-renderer`:** Vello + Cosmic-text: Generator Node vẽ chữ Karaoke, hình khối, vector 2D trên GPU.
- **`crates/automation`:** Cổng MCP Server (`rmcp`) và CLI dispatcher (`Session::execute`).
- **`crates/ai-workers`:** Luồng ngầm chạy Silero VAD (tìm khoảng lặng) và Whisper/Moonshine nhận diện giọng nói.
- **`crates/ui`:** Giao diện 4 khu vực vẽ bằng `egui` (Source Bin, Viewport, Inspector/Node Panel, Timeline).

---

## 3. THIẾT QUÂN LUẬT AN TOÀN BỘ NHỚ & ỔN ĐỊNH (NEVER CRASH)
- **Cấm tuyệt đối Panic trong Production:**
  - Kích hoạt `#![deny(clippy::unwrap_used, clippy::expect_used)]` trên toàn bộ workspace.
  - CẤM gọi `.unwrap()`, `.expect()`, `panic!()` trong code xử lý video, audio hoặc render.
  - Mọi hàm có rủi ro thất bại BẮT BUỘC trả về `Result<T, E>`.
- **Tránh rò rỉ VRAM/RAM:**
  - Không phân bổ bộ nhớ mới (`alloc`) trong vòng lặp Render 60fps. Sử dụng bộ nhớ đệm tái sử dụng (Ring-Buffer, Staging Buffer).
  - Khung hình đã giải mã phải giải phóng kịp thời theo thuật toán LRU (Least Recently Used), khóa cứng mức trần RAM `< 512MB`.
- **Giới hạn dòng code & Báo cáo Người Dùng (< 700 dòng):**
  - Mọi file mã nguồn tuân thủ trần chuẩn mực tối đa **700 dòng**.
  - **BẮT BUỘC:** Khi file chạm ngưỡng 700 dòng, Agent phải dừng lại báo cáo Người Dùng để quyết định phương án; tuyệt đối không tự động tái cấu trúc hay tối ưu hóa bừa bãi. Người Dùng có thể cho phép ngoại lệ nếu thấy cần thiết.
  - Kiểm toán tự động qua tool `.agents/harness/check_file_size.py`.

---

## 4. QUẢN LÝ TÀI LIỆU 3 NGĂN KÉO (ZERO-TRASH DIRECTIVE)
Toàn bộ tài liệu dự án được quản lý trong `.darkworks-ai/`:
- **`1-overview/`:** `system_map.md` (Bản đồ kiến trúc tổng thể) và `project-managers/` (Kế hoạch từng Phase).
- **`2-memory/`:** `checkpoint.md` (Bàn giao ca kíp), `decision-log.md` (Quyết định cốt lõi) và `archive/`.
- **`3-modules/`:** Tài liệu kiến trúc của 7 crates cốt lõi (`schema`, `media`, `node-graph`, `canvas-renderer`, `automation`, `ai-workers`, `ui`).

---

## 5. QUY TẮC ĐÁNH DẤU PHIÊN BẢN (SEMVER)
- Tuân thủ chuẩn `MAJOR.MINOR.PATCH`:
  - **PATCH (x.y.Z):** Sửa lỗi nhỏ, hotfix, tối ưu hiệu năng không đổi API.
  - **MINOR (x.Y.z):** Thêm Node mới, thêm Crate mới, mở rộng API tương thích ngược.
  - **MAJOR (X.y.z):** Tái cấu trúc kiến trúc làm thay đổi định dạng `project.json`.

---
---

# PHẦN 2: QUY TRÌNH VẬN HÀNH CA KÍP (SYSTEM WORKFLOWS)

## 1. QUY TRÌNH MỞ PHIÊN (/start_session)
1. **Khởi động bộ nhớ:** Đọc `.darkworks-ai/2-memory/checkpoint.md`, `system_map.md`, `decision-log.md` và `.agents/rules/improve.md`.
2. **Xác định tiêu điểm:** Kiểm tra Git branch (`git status`), xác định Crate cần can thiệp.
3. **Load Crate Memory:** Đọc tài liệu liên đới tại `.darkworks-ai/3-modules/[tên-crate]/architecture.md`.
4. **Báo cáo sẵn sàng:** Xác nhận trạng thái và xin lệnh thực thi.

## 2. QUY TRÌNH ĐÓNG PHIÊN (/end_session)
1. **Kiểm toán mã nguồn:** Chạy `cargo check --workspace --all-targets` và `cargo clippy`.
2. **Cập nhật tài liệu (Tối đa 4 điểm chạm):**
   - Cập nhật `3-modules/[tên-crate]/architecture.md`.
   - Ghi sổ `decision-log.md`.
   - Cập nhật `system_map.md` (chuyển đổi trạng thái module và ghi log).
   - Niêm phong `checkpoint.md` (bàn giao tiến độ cho ca sau).
3. **Bài học kinh nghiệm:** Cập nhật `.agents/rules/improve.md` nếu có lỗi phát sinh sau khi được Người Dùng duyệt.
4. **Git Commit:** Xin ý kiến Người Dùng để commit và push.

## 3. QUY TRÌNH GỠ LỖI NGOẠI KHOA (/debug-workflow)
- **Step 0 — Memory Check:** Đọc `checkpoint.md`, cô lập crate lỗi.
- **Step 1 — Comprehend & Clarify (CHƯA ĐƯỢC CHẠM VÀO CODE):** Phân tích RCA (Logic gãy ở đâu? Tại sao xảy ra?).
- **Step 2 — Isolate & Impact Assessment:** Đánh giá blast radius và kiểm tra các bẫy bộ nhớ/deadlock/shader.
- **Step 3 — Surgical Proposal:** Trình bày giải pháp và **chờ Người Dùng đồng ý mới được sửa code**.
- **Step 4 — Execution & Verification:** Sửa mã chính xác (targeted diff), kiểm tra bằng `cargo test`, bump PATCH version.

## 4. QUY TRÌNH PHÁT HÀNH (/release-github)
1. Cập nhật version trong `Cargo.toml`.
2. Biên dịch binary release: `cargo build --release --bin darkworks`.
3. Tạo Git tag `vX.Y.Z` và push lên remote.
4. Phát hành GitHub Release kèm Changelog chuẩn 3 tầng (What's New / Improvements / Fixes).
