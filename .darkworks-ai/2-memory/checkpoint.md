# SỔ BÀN GIAO TIẾN ĐỘ CA KÍP: DARKWORKS (checkpoint.md)
@project: DARKWORKS | @phòng_ban: Toàn xưởng | @status: 🟢 Hoàn Thành `TimeDriver` (`Clip01`/`Duration`), `Add as FX Clip` Thông Minh & `Transition` Dùng Chung FilmCraft `apply_scaled` (Phương Án A)
@session_end: 2026-11-28 (Ca 2) | @next_session_focus: Test thực tế `Transition` (`TimeDriver::Clip01` & `Duration`) + các Atomic Nodes còn lại trước khi chuyển sang `Task 3.9` (Node & Preset Library Browser)

> Sổ ghi chép bàn giao công việc giữa các phiên làm việc của AI.  
> Khi mở phiên mới (`/start_session`), nhân viên đọc file này để nắm tiến độ dở dang trong 3 giây.

---

## 1. THÔNG TIN NHÁNH GIT & TRẠNG THÁI HIỆN TẠI
- **Nhánh Git:** `main` (`main...origin/main [ahead 1]`, đang có các thay đổi chưa commit của ca 2026-11-27 và 2026-11-28 — chờ ý kiến Người Dùng để commit/push).
- **Môi trường:** Linux x86_64, Vulkan (`AMD Radeon Graphics RADV RENOIR`) / VA-API, Pure Rust (`wgpu` + `egui` + `egui_snarl`).
- **Kết quả kiểm toán cuối phiên (SAU LẦN SỬA `.rs` CUỐI — PASS 100%):**
  - `cargo check --workspace --all-targets`: **PASS** (`22.53s`, 0 errors).
  - `cargo clippy --workspace --all-targets -- -D warnings`: **PASS** (`25.27s`, 0 warnings, 0 errors).
  - `cargo xtask layers`: **PASS** (`layers: ok`).
  - `python3 .agents/harness/audit.py`: **0 vi phạm `>= 700` dòng, 0 cảnh báo `> 650` dòng** (`924` files: `23` đạt chuẩn, `901` ngoại lệ Upstream/Vendor).
  - **Tests đã chạy sau lần sửa `.rs` cuối:**
    - `filmcraft-render` (`node_graph` filter): **6 passed; 0 failed** (`287 filtered out`, gồm `transition_uses_filmcraft_engine_without_shrunken_border_artifacts`), `tests/it`: 0 filtered (`6 filtered out`), `mixer_perf`: 0 filtered (`1 filtered out`).
    - `darkworks-node-graph-core`: **6 passed** (gồm `time_driver_clip01_normalizes_by_clip_duration_and_clamps` & `overlay_hint_reads_linear_duration_through_reroute_and_tracks`).
    - `darkworks-node-graph`: **15 passed** (`tests/graph_tests.rs`) + **1 passed** (`tests/history_tests.rs`).
    - `filmcraft-engine` (`node_graph` filter): **2 passed** (gồm `transition_overlay_respects_timedriver_markzone_and_overlap`).

---

## 2. DANH SÁCH FILE THAY ĐỔI TRONG CA 2026-11-28 (CA 2 — KÈM LIÊN KẾT MARKDOWN)
*(Lưu ý: trên `main` còn các thay đổi chưa commit từ ca 2026-11-27 và ca 1 2026-11-28).*

### Core & UI `TimeDriver` (`crates/node-graph-core`, `crates/node-graph`)
- [crates/node-graph-core/src/node_fx_types.rs](file:///mnt/Data/darkworks/crates/node-graph-core/src/node_fx_types.rs) (`281` dòng): Thêm `TimeMode::Clip01`.
- [crates/node-graph-core/src/evaluator.rs](file:///mnt/Data/darkworks/crates/node-graph-core/src/evaluator.rs) (`409` dòng): Thêm `DagEvaluator::evaluate_signals_for_clip` chuẩn hóa `Clip01` theo `clip_duration_sec` + 2 unit tests.
- [crates/node-graph-core/src/doc.rs](file:///mnt/Data/darkworks/crates/node-graph-core/src/doc.rs) (`181` dòng) & [lib.rs](file:///mnt/Data/darkworks/crates/node-graph-core/src/lib.rs) (`28` dòng): Thêm `OverlayPlacementHint`, `resolve_source_node`, `overlay_hint()` phân tích `Transition` + `TimeDriver` + track nguồn qua `Reroute`.
- [crates/node-graph/src/node_controls.rs](file:///mnt/Data/darkworks/crates/node-graph/src/node_controls.rs) (`563` dòng): UI `TimeDriver` thêm mode `Clip 0→1 (theo Clip)` và ô `Duration (s)` đồng bộ 2 chiều với `Speed`.

### FX Clip Thông Minh & Render `Transition` (`crates/engine`, `crates/ui-egui`, `crates/render`)
- [crates/engine/src/node_graph.rs](file:///mnt/Data/darkworks/crates/engine/src/node_graph.rs) (`661` dòng): Cập nhật `create_overlay_clip` + `find_transition_overlap` ưu tiên `Mark In/Out` > `TimeDriver` > `Overlap` > `2.0s` mặc định `Transition` (`5.0s` cho node khác) + test `transition_overlay_respects_timedriver_markzone_and_overlap`.
- [crates/ui-egui/src/panels/node_graph_bridge.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/node_graph_bridge.rs) (`401` dòng): Bỏ gán cứng `seconds: 5.0` ở `🎬 Add as FX Clip`, hiển thị rõ `durationSec` & `reason`.
- [crates/render/src/node_graph.rs](file:///mnt/Data/darkworks/crates/render/src/node_graph.rs) (`480` dòng) & [gpu_plan.rs](file:///mnt/Data/darkworks/crates/render/src/nodes/gpu_plan.rs) (`389` dòng): Truyền `item.duration.seconds()` vào `evaluate_signals_for_clip`; truyền `opts.scale` vào `apply_transition`.
- [crates/render/src/nodes/transform_blend.rs](file:///mnt/Data/darkworks/crates/render/src/nodes/transform_blend.rs) (`308` dòng) & [node_graph_tests.rs](file:///mnt/Data/darkworks/crates/render/src/node_graph_tests.rs) (`381` dòng): Nâng cấp `apply_transition` (Phương án A) gọi chung `vtransition::find_transition` + `crate::transitions::apply_scaled` của FilmCraft cho cả 8 kiểu + test hồi quy.

### Kế hoạch & Bộ nhớ (`.darkworks-ai/`)
- [master-plan-a-z.md](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/project-managers/master-plan-a-z.md), [node-graph/architecture.md](file:///mnt/Data/darkworks/.darkworks-ai/3-modules/node-graph/architecture.md), [decision-log.md](file:///mnt/Data/darkworks/.darkworks-ai/2-memory/decision-log.md) (#43), [system_map.md](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/system_map.md).

---

## 3. CÁC LỖI ĐÃ GIẢI QUYẾT & ĐIỂM ĐÃ CHỐT TRONG PHIÊN
1. **Khắc phục lỗi `Add as FX Clip` dài hơn hiệu ứng & phải move/resize thủ công:** Thêm `TimeMode::Clip01`, ô `Duration (s)` trên `TimeDriver`, và tự căn vị trí/độ dài FX Clip theo `Mark [In, Out]` > `TimeDriver` > `Overlap` > `2.0s`.
2. **Khắc phục lỗi `Transition` bị "thô" (như `ZoomPush` thu nhỏ lộ viền xám ở `0.35x`, thiếu easing & motion blur):** Áp dụng **Phương án A**, đưa cả 8 kiểu của `DarkNode::Transition` sang gọi trực tiếp `crate::transitions::apply_scaled` của FilmCraft (`film_dissolve`, `linear_wipe`, `push`, `iris_round`, `motion_camera`, `dip_to_color`, `blur_dissolve`), hỗ trợ luôn chuyển cảnh 1 clip sang đen.
3. **Chốt giữ triết lý Node Graph cho `Transition`:** Không thêm chế độ `Auto` ngầm vào `Transition` để tránh trùng lặp với `TimeDriver (Clip 0→1)` và giữ minh bạch luồng dây trên canvas (việc khỏi phải nối dây thủ công sẽ do **Thư viện Preset `.preset.json` `Task 3.9`** giải quyết).
4. **Ghi chú vấn đề tồn đọng vào `master-plan-a-z.md`:** Wipe hình học phức tạp bằng Vello (`Task 4.3`), chuyển cảnh 3D thật / Optical Flow `Morph Cut` / tối ưu GPU cho `Transition` (`Task 7.2`).

---

## 4. KẾ HOẠCH BÀN GIAO CA KẾ TIẾP (NEXT ACTIONS CHECKLIST)
- [ ] **1. Nghiệm thu thực tế `Transition` + `TimeDriver` mới trên GUI:**
  - Chạy lại kịch bản: (a) `TimeDriver` `Clip 0→1` với `Markzone` hoặc 2 clip gối đầu (`V1` + `V2`), (b) `TimeDriver` `Duration = 2.0s`, (c) 1 clip nối `Input A` chuyển sang đen.
- [ ] **2. Test thực tế các Atomic Nodes còn lại:**
  - `ColorGrade`, `BlurSharpen` (6 kiểu), `DistortLens` (13 kiểu), `AudioReactor`, `TextGenerator`, `ShapeGenerator`, `Reroute`, `CustomWgsl` và đồ thị đa nhánh.
  - Chưa chuyển sang `Task 3.9` (Node & Preset Library Browser) cho tới khi Người Dùng xác nhận test xong toàn bộ các Node.
