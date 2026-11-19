---
trigger: always_on
---
# DARKWORKS SYSTEM RULES (darkworks-system-rules.md)
@target: Darkworks NLE Video Editor | @architecture: Rust Cargo Workspace (L0 -> L6) | @engine: wgpu + egui + vello

## 0. KHẨU QUYẾT TỐI THƯỢNG (THE PRIME DIRECTIVES)

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

## 1. PHÂN CÁCH TRÁCH NHIỆM GIỮA CÁC CRATES (CRATE BOUNDARY)
Trước khi code, Agent BẮT BUỘC nhận diện mã nguồn thuộc Crate nào để không lẫn lộn ranh giới:
- **`crates/schema`:** Chứa định nghĩa struct cho `project.json`, `preset.json`, `ClipSlice`. Cực nhẹ, không chứa logic nặng.
- **`crates/media`:** Lõi giải mã video phần cứng OS native (VA-API), quản lý Ring-Buffer, đồng bộ A/V Sync với `cpal`.
- **`crates/node-graph`:** Bộ khung `egui_snarl`, quản lý shader WGSL, duyệt đồ thị DAG, xuất/nạp `.preset.json`.
- **`crates/canvas-renderer`:** Vello + Cosmic-text: Generator Node vẽ chữ Karaoke, hình khối, vector 2D trên GPU.
- **`crates/automation`:** Cổng MCP Server (`rmcp`) và CLI dispatcher (`Session::execute`).
- **`crates/ai-workers`:** Luồng ngầm chạy Silero VAD (tìm khoảng lặng) và Whisper/Moonshine nhận diện giọng nói.
- **`crates/ui`:** Giao diện 4 khu vực vẽ bằng `egui` (Source Bin, Viewport, Inspector/Node Panel, Timeline).

---

## 2. QUY TRÌNH PHÁT TRIỂN CỦA AGENT (AI WORKFLOW PROTOCOL)
- **Step 1 (Context):** Đọc `.darkworks-ai/1-overview/system_map.md` để nắm trạng thái hệ thống.
- **Step 2 (Isolate Memory):** Chỉ đọc tài liệu module liên quan trong `.darkworks-ai/3-modules/[tên-crate]/`.
- **Step 3 (Schema First):** Thiết kế struct, trait và public API trước khi viết phần thân hàm.
- **Step 4 (Documentation Loop):** Cập nhật tài liệu module trong `3-modules/` và ghi nhật ký vào `.darkworks-ai/2-memory/decision-log.md`.

---

## 3. GIỚI HẠN DÒNG CODE & BÁO CÁO NGƯỜI DÙNG (FILE SIZE LIMIT RULE)
- **Mức chuẩn khuyến nghị:** Dưới 500 dòng code mỗi file (`.rs`).
- **Trần cứng báo động:** **700 dòng code**.
- **THIẾT QUÂN LUẬT BÁO CÁO:** Khi bất kỳ file mã nguồn nào chạm ngưỡng 700 dòng:
  - **CẤM TUYỆT ĐỐI:** Tự ý tối ưu hóa, gộp hàm hoặc tự tiện cắt xén tách file khi chưa xin ý kiến.
  - **BẮT BUỘC:** Báo cáo chi tiết lên Người Dùng để xin ý kiến quyết định phương án (Người Dùng có toàn quyền cho phép ngoại lệ vượt 700 dòng nếu thấy hợp lý hoặc chỉ định cách tái cấu trúc).
- **Công cụ kiểm toán:** Chạy `python3 .agents/harness/check_file_size.py` trước mọi phiên bàn giao (/end_session).

---

## 4. QUY TẮC ĐÁNH DẤU PHIÊN BẢN (SEMVER)
- Tuân thủ chuẩn `MAJOR.MINOR.PATCH`:
  - **PATCH (x.y.Z):** Sửa lỗi nhỏ, hotfix, tối ưu hiệu năng không đổi API.
  - **MINOR (x.Y.z):** Thêm Node mới, thêm Crate mới, mở rộng API tương thích ngược.
  - **MAJOR (X.y.z):** Tái cấu trúc kiến trúc làm thay đổi định dạng `project.json`.
- Mỗi khi release, cập nhật phiên bản trong `Cargo.toml`, ghi chú vào `decision-log.md` và `system_map.md`.
