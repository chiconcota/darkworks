# SỔ BÀN GIAO TIẾN ĐỘ CA KÍP: DARKWORKS (checkpoint.md)
@project: DARKWORKS | @phòng_ban: Toàn xưởng | @status: 🟡 Hoàn Thành Cầu Nối Node Graph ⇄ Timeline/Render & Sửa Lỗi UI/Live-Sync — Sẵn Sàng Task 3.12 (`DW-06`: Save Preset & Custom Preset)
@session_end: 2026-11-25 | @next_session_focus: Triển khai tính năng Save Preset (`.preset.json` thật) & Thư viện Custom Preset (`DW-06` / Task 3.12)

> Sổ ghi chép bàn giao công việc giữa các phiên làm việc của AI.  
> Khi mở phiên mới (`/start_session`), nhân viên đọc file này để nắm tiến độ dở dang trong 3 giây.

---

## 1. THÔNG TIN NHÁNH GIT & TRẠNG THÁI HIỆN TẠI
- **Nhánh Git:** `main` (chưa commit các thay đổi của phiên 2026-11-25 — chờ Người Dùng quyết định commit).
- **Môi trường:** Linux x86_64, Vulkan / VA-API, Pure Rust (`wgpu` + `egui` + `egui_snarl`).
- **Kết quả kiểm toán cuối phiên (PASS 100%):**
  - `cargo check --workspace --all-targets`: **PASS** (`41.61s`, 0 errors).
  - `cargo clippy --workspace --all-targets -- -D warnings`: **PASS** (0 warnings, 0 errors).
  - `cargo xtask layers`: **PASS** (tuân thủ nghiêm ngặt phân tầng L0–L6, L0–L4 không kéo `egui`).
  - `python3 .agents/harness/audit.py`: **0 vi phạm ngưỡng `>= 700` dòng** (kiểm toán `913` files: `19` đạt chuẩn, `1` cảnh báo `> 650` dòng, `893` ngoại lệ Upstream/Vendor).

---

## 2. DANH SÁCH FILE THAY ĐỔI KÈM LIÊN KẾT MARKDOWN

### Mã nguồn Rust (Node Graph Core, Render, GPU, Engine, Timeline Bridge & UI)
- [crates/node-graph-core/](file:///mnt/Data/darkworks/crates/node-graph-core/src/lib.rs): Crate Layer 0 thuần dữ liệu (`NodeGraphDoc`, `DarkNode`, `BlendMode::ALL` 27+1 modes, `DagEvaluator`, `WgslParser`).
- [crates/node-graph/src/doc.rs](file:///mnt/Data/darkworks/crates/node-graph/src/doc.rs): Cầu nối 2 chiều `doc_from_snarl` / `snarl_from_doc`.
- [crates/node-graph/src/node_controls.rs](file:///mnt/Data/darkworks/crates/node-graph/src/node_controls.rs): Bố cục điều khiển dọc gọn (`max_width(195.0)`), Color Picker trên `ColorGrade`/`TextGenerator`/`ShapeGenerator`/`Keyer`, `ComboBox` cho `Blend` (27 modes + `Add`), `ShapeGenerator`, `AudioReactor`, `ActionNode`.
- [crates/render/src/node_graph.rs](file:///mnt/Data/darkworks/crates/render/src/node_graph.rs) & [crates/render/src/nodes/](file:///mnt/Data/darkworks/crates/render/src/nodes.rs): Thực thi DAG compositor theo `Tick`, tách gọn từng nhóm node (`generators.rs`, `shape.rs`, `text.rs`, `color_blur.rs`, `blur_distort.rs`, `transform_blend.rs`, `signals.rs`).
- [crates/gpu/src/custom_wgsl.rs](file:///mnt/Data/darkworks/crates/gpu/src/custom_wgsl.rs): Biên dịch & chạy `CustomWgsl` fragment shader an toàn qua `push_error_scope` (cache tối đa 32 pipelines), hook vào `apps/darkworks` & `apps/darkworks-cli`.
- [crates/project/src/track.rs](file:///mnt/Data/darkworks/crates/project/src/track.rs) & [crates/engine/src/node_graph.rs](file:///mnt/Data/darkworks/crates/engine/src/node_graph.rs): Lưu `node_graph` trong `TrackItem` + lệnh `nodeGraph.{setOnClip, createOverlayClip, applyScope, clearFromClip, getFromClip}`.
- [crates/ui-egui/src/panels/node_graph.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/node_graph.rs), [node_graph_menus.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/node_graph_menus.rs), [node_graph_bridge.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/node_graph_bridge.rs): Đồng bộ selection, `Apply to Clip`, `Add as FX Clip`, `New Graph (Nháp)` (`detach_to_scratchpad`), `Gỡ khỏi Clip`, live-sync an toàn và badge `◈` Cyan trên [clips.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/timeline/clips.rs).

### Kế hoạch, Feature Map & Bộ nhớ
- [master-plan-a-z.md](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/project-managers/master-plan-a-z.md): Bổ sung Task 3.8 (Kho Preset `.preset.json` thật), Phase 4 (Task 4.5–4.7: Keyframe Node Graph + Transform Handles + Ẩn `Effect Controls`/`Video Effects` cũ), Phase 5 (Task 5.5: Audio/Transition Nodes + Xóa sạch `Effects`/`Effect Controls` cũ).
- [phase-3-node-graph.md](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/project-managers/phase-3-node-graph.md): Hoàn thành Tasks 3.8–3.11; thêm Task 3.12 (`DW-06`).
- [darkworks_node_presets_keyframes_migration.md](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/features/darkworks_node_presets_keyframes_migration.md), [features/README.md](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/features/README.md), [feature_map.md](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/feature_map.md): Bổ sung kịch bản kiểm thử cho `DW-06`, `DW-07`, `DW-08`.
- [system_map.md](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/system_map.md), [decision-log.md](file:///mnt/Data/darkworks/.darkworks-ai/2-memory/decision-log.md) (#36–#38), [node-graph/architecture.md](file:///mnt/Data/darkworks/.darkworks-ai/3-modules/node-graph/architecture.md), [ui/architecture.md](file:///mnt/Data/darkworks/.darkworks-ai/3-modules/ui/architecture.md).

---

## 3. CÁC LỖI ĐÃ GIẢI QUYẾT TRONG PHIÊN (RESOLVED ISSUES)
1. **Giao diện Node quá rộng & thiếu Color Picker trực tiếp:** Chuyển controls sang bố cục dọc gọn (`max_width(195.0)`), bổ sung Color Tint Picker (`tint_color`, `tint_amount`) ngay trên `ColorGrade`.
2. **Tách module `crates/render/src/nodes/`:** Giảm tải cho `node_graph_ops.rs` cũ bằng phân tách từng file xử lý theo nhóm chức năng node (`generators`, `shape`, `text`, `color_blur`, `blur_distort`, `transform_blend`, `signals`).
3. **Node `Blend` chỉ hiện `Normal` & thiếu `ComboBox` trên một số node:** Mở rộng `BlendMode` lên đủ 27 chế độ của `filmcraft_render::Blend` (+ `Add` tương thích ngược), thêm `ComboBox` cho `Blend`, `ShapeGenerator`, `AudioReactor`, `ActionNode`.
4. **Live-sync ghi đè nhầm FX Clip 1 khi nạp Preset / Clear All để tạo FX Clip 2:** Thêm `detach_to_scratchpad()` khi bấm `Load Preset` / `Clear All` / `Reset` (giữ `last_observed_selection` để không bị nạp đè lại ở frame sau), bổ sung nút `📝 New Graph (Nháp)` và đổi `⛓ Gỡ` ➔ `🗑 Gỡ khỏi Clip`.
5. **Làm rõ hiện trạng `Export Preset`:** Xác nhận nút `💾 Export Preset` hiện mới chỉ copy JSON thiếu `wires`/`pos` vào Clipboard chứ chưa lưu `.preset.json` thật; đã đưa vào kế hoạch ca sau (`Task 3.12` / `DW-06`).

---

## 4. KẾ HOẠCH BÀN GIAO CA KẾ TIẾP (NEXT ACTIONS CHECKLIST)
Khi mở phiên tiếp theo (`/start_session`), ưu tiên làm dứt điểm **Tính năng Save Preset & Custom Preset (`Task 3.12` / `DW-06`)**:
- [ ] **1. Chuẩn hóa định dạng `.preset.json` đầy đủ (`crates/node-graph-core` & `crates/node-graph`):**
  - Đóng gói metadata (`name`, `category`, `version`, `description`) kèm trọn vẹn `NodeGraphDoc` (`nodes`, `pos`, `custom_title`, `wires`, `groups`) để Export/Import giữ 100% dây nối & tọa độ.
  - Giữ tương thích ngược nếu đọc file preset cũ không có `wires`.
- [ ] **2. Nâng cấp `💾 Export Preset` & `📥 Import Preset…` dùng hộp thoại file (`crates/ui-egui`):**
  - Dùng `FileDialog::save_as` & `FileDialog::open_file` (từ `native_dialogs.rs`) để lưu/mở file `.preset.json` thật trên ổ đĩa (có thể giữ kèm nút phụ Copy JSON nếu cần).
  - Đảm bảo Import luôn đưa về chế độ Nháp (`detach_to_scratchpad()`), không ghi đè clip cũ.
- [ ] **3. Xây dựng Kho Custom / User Presets lưu sẵn 1-click:**
  - Cho phép lưu trực tiếp Preset tùy biến vào thư mục User Presets (ví dụ thư mục data của Darkworks) và tự động scan/hiển thị thành danh sách trong menu `Load Preset` (hoặc bảng Node Presets) để tái sử dụng 1-click.
  - Bổ sung lựa chọn **Thay thế (Replace)** hoặc **Nối tiếp trước `Final Output` (Chain)** khi áp Preset lên một Clip đã có Node Graph.
- [ ] **4. Nghiệm thu theo Feature Map:**
  - Chạy test theo kịch bản `DW-06` (`Test 6.1 – 6.3`) trong [darkworks_node_presets_keyframes_migration.md](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/features/darkworks_node_presets_keyframes_migration.md#L23-L52).
