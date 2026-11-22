# SỔ BÀN GIAO TIẾN ĐỘ CA KÍP: DARKWORKS (checkpoint.md)
@project: DARKWORKS | @phòng_ban: Toàn xưởng | @status: 🟢 Hoàn thành 100% Giai Đoạn 2 (Timeline Lightworks UX, Rebranding & Tinh Chỉnh Giao Diện)
@session_end: 2026-11-22 | @next_session_focus: Giai đoạn 3 - Đồ thị DAG Node Graph (egui_snarl) & 93 Atomic Shaders

> Sổ ghi chép bàn giao công việc giữa các phiên làm việc của AI.  
> Khi mở phiên mới (/start_session), nhân viên đọc file này để nắm tiến độ dở dang trong 3 giây.

---

## 1. THÔNG TIN MÔI TRƯỜNG & TRẠNG THÁI HIỆN TẠI
- **Môi trường:** Linux x86_64, Rust 1.85 (Edition 2024), Vulkan / Linux VA-API (`/dev/dri/renderD128`).
- **Tình trạng:** **ĐÃ HOÀN TẤT TOÀN DIỆN GIAI ĐOẠN 2 & HOÀN THIỆN NHẬN DIỆN THƯƠNG HIỆU DARKWORKS**:
  - Loại bỏ hoàn toàn nút [Discord] trên thanh Header bar và các liên kết bên ngoài của FilmCraft/ArtCraft.
  - Tinh gọn Menu Bar: Giữ lại nhãn `Help` phẳng trên menu bar cho tài liệu tương lai, lược bỏ toàn bộ submenu thừa.
  - Chuẩn hóa toàn bộ chuỗi hiển thị thương hiệu: Đổi triệt để `FilmCraft` ➔ `Darkworks` trên mọi cửa sổ, hộp thoại khôi phục crash, lỗi panic, bộ lọc file (`*.fcproj`), menu native và tên dự án mẫu mặc định (`Darkworks – Demo Project`).
  - Thiết kế và tích hợp trọn bộ icon app chính thức của Darkworks (Biểu tượng chòm sao Node Graph Play Triangle Neon Cyan/Purple) cho cả eframe window và Taskbar/Dock desktop Linux (`dev.darkworks.editor.desktop`).
  - Kiểm toán `audit.py` 0 vi phạm trần dòng code (< 700 dòng).
  - Bộ test tự động `drive_feature.py -f all` đạt **100% PASS** cho cả 3 tính năng (FC-01, FC-02, DW-03).
  - `cargo check --workspace --all-targets` đạt **100% PASS**.
- **Nhánh Git:** `main`.

---

## 2. DANH SÁCH FILE THAY ĐỔI TRONG PHIÊN
- [crates/ui-egui/src/header.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/header.rs): Gỡ nút Discord, mở rộng không gian cho tiêu đề project.
- [crates/ui-egui/src/menus.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/menus.rs): Giữ nhãn `Help` phẳng không mở submenu; xóa menu path các lệnh help cũ.
- [crates/ui-egui/src/panels/file_dialogs.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/file_dialogs.rs): Chuẩn hóa câu thông báo khôi phục crash `Darkworks quit unexpectedly...`.
- [crates/ui-egui/src/app/frame.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/app/frame.rs): Cửa sổ báo lỗi UI `Darkworks hit an error`.
- [crates/ui-egui/src/panels/dialogs.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/dialogs.rs): Hộp thoại About Darkworks.
- [crates/engine/src/demo.rs](file:///mnt/Data/darkworks/crates/engine/src/demo.rs): Dự án mẫu mặc định mang tên `Darkworks – Demo Project`.
- [crates/project/src/settings.rs](file:///mnt/Data/darkworks/crates/project/src/settings.rs) & [crates/engine/src/project_tools.rs](file:///mnt/Data/darkworks/crates/engine/src/project_tools.rs): Renderer mặc định `Darkworks GPU Acceleration (wgpu)`.
- [apps/darkworks/src/main.rs](file:///mnt/Data/darkworks/apps/darkworks/src/main.rs): Cửa sổ eframe mang tên `Darkworks`, nạp icon `darkworks-256.png`.
- [apps/darkworks/src/dev_icon.rs](file:///mnt/Data/darkworks/apps/darkworks/src/dev_icon.rs): Đăng ký launcher desktop `dev.darkworks.editor.desktop`.
- [assets/app-icon/](file:///mnt/Data/darkworks/assets/app-icon/): Tạo trọn bộ icon ứng dụng Darkworks (16px đến 1024px).
- [.darkworks-ai/2-memory/decision-log.md](file:///mnt/Data/darkworks/.darkworks-ai/2-memory/decision-log.md): Bổ sung Quyết định #23, #24, #25.

---

## 3. KẾ HOẠCH HÀNH ĐỘNG CA TIẾP THEO (PHASE 3: ĐỒ THỊ DAG NODE GRAPH)
- [x] **Giai đoạn 1:** Tiếp nhận nền tảng FilmCraft, khóa trần RAM < 2GB (Hoàn thành 100%).
- [x] **Giai đoạn 2:** Cấy ghép Timeline Lightworks UX & Tinh chỉnh thương hiệu Darkworks (Hoàn thành 100%).
- [ ] **Giai đoạn 3 (Đồ thị DAG Node Graph & 93 WGSL Shaders - Trọng tâm cốt lõi):**
  - [ ] Khởi tạo crate `crates/node-graph` tích hợp `egui_snarl`.
  - [ ] Khai tử danh sách tuyến tính `Vec<EffectInstance>` cũ của FilmCraft/Premiere.
  - [ ] Xây dựng giao diện Node Graph Canvas tương tác mượt mà (kéo thả node, socket in/out, dây cong Bézier).
  - [ ] Đóng gói 93 Shader WGSL từ `crates/render` thành các Node nguyên tử (Transform, Blend, Color Grading, Keying, Blur, v.v.).
  - [ ] Hỗ trợ xuất/nạp đồ thị độc lập định dạng `.preset.json`.
