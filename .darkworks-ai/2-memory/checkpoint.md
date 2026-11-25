# SỔ BÀN GIAO TIẾN ĐỘ CA KÍP: DARKWORKS (checkpoint.md)
@project: DARKWORKS | @phòng_ban: Toàn xưởng | @status: 🟡 Hoàn Thành Lưu/Xuất/Nhập `.preset.json` (Task 3.12) & Nâng Cấp Nhóm A Atomic Nodes — Còn Bug `Audio Reactor` & `Task 3.13`
@session_end: 2026-11-25 (Ca 2) | @next_session_focus: Điều tra RCA lỗi `Audio Reactor`, kiểm tra các node còn lại (`CustomWgsl`, `Reroute`, `viewer::connect`), và triển khai Node & Preset Library Browser (`Task 3.13`)

> Sổ ghi chép bàn giao công việc giữa các phiên làm việc của AI.  
> Khi mở phiên mới (`/start_session`), nhân viên đọc file này để nắm tiến độ dở dang trong 3 giây.

---

## 1. THÔNG TIN NHÁNH GIT & TRẠNG THÁI HIỆN TẠI
- **Nhánh Git:** `main` (đã commit toàn bộ thay đổi của phiên 2026-11-25).
- **Môi trường:** Linux x86_64, Vulkan / VA-API, Pure Rust (`wgpu` + `egui` + `egui_snarl`).
- **Kết quả kiểm toán cuối phiên (PASS 100%):**
  - `cargo check --workspace --all-targets`: **PASS** (`36.99s`, 0 errors).
  - `cargo clippy --workspace --all-targets -- -D warnings`: **PASS** (`39.79s`, 0 warnings, 0 errors).
  - `cargo xtask layers`: **PASS** (`layers: ok`).
  - `python3 .agents/harness/audit.py`: **0 vi phạm ngưỡng `>= 700` dòng** (kiểm toán `916` files: `21` đạt chuẩn, `0` cảnh báo `> 650` dòng, `895` ngoại lệ Upstream/Vendor).
  - Unit tests (`node-graph-core` 3/3, `node-graph` 15/15 + 1/1, `filmcraft-render node_graph::tests` 3/3): **PASS** ở lần chạy kiểm chứng sau lần sửa code cuối.

---

## 2. DANH SÁCH FILE THAY ĐỔI KÈM LIÊN KẾT MARKDOWN

### Mã nguồn Rust (Preset `.preset.json`, Nhóm A Atomic Nodes, Signals & UI)
- [crates/node-graph-core/src/preset.rs](file:///mnt/Data/darkworks/crates/node-graph-core/src/preset.rs): `NodeGraphPresetDoc` (đầy đủ `nodes`, `pos`, `custom_title`, `wires`, `groups` + tương thích ngược preset cũ) và `chain_node_graphs` (nối tiếp DAG trước `Final Output`).
- [crates/node-graph-core/src/node.rs](file:///mnt/Data/darkworks/crates/node-graph-core/src/node.rs) & [evaluator.rs](file:///mnt/Data/darkworks/crates/node-graph-core/src/evaluator.rs): Nâng cấp Nhóm A Atomic Nodes (`ColorGrade` Lumetri + `Exp Mod`, `BlurSharpen` `threshold` + `Radius Mod`, `DistortLens` thêm `Mirror`/`Offset`/`Posterize`/`Pixelate` + `feather`, `ShapeGenerator` `position`/`stroke`/`feather`/`Pos Mod`, `TextGenerator` `font_family`/`bold`/`italic`/`align`/`position`/`stroke`/`Pos Mod`, `Transition` `direction`/`ZoomPush`/`DipBlack`/`DipWhite`/`BlurDissolve`, `TimeDriver` & `AudioReactor` `track_name` + `Vec2 Out`).
- [crates/node-graph/src/node_controls.rs](file:///mnt/Data/darkworks/crates/node-graph/src/node_controls.rs), [viewer.rs](file:///mnt/Data/darkworks/crates/node-graph/src/viewer.rs), [preset.rs](file:///mnt/Data/darkworks/crates/node-graph/src/preset.rs), [Cargo.toml](file:///mnt/Data/darkworks/crates/node-graph/Cargo.toml): Khóa `ui.vertical` chống đè chữ, `ComboBox` chọn Track cho `VideoInput`/`AudioReactor`, `ComboBox` chọn Font hệ thống từ `filmcraft-text` cho `TextGenerator`, sửa `Scale X/Y` trên `Transform2D`, sửa padding nhãn pin trái/phải.
- [crates/render/src/nodes/signals.rs](file:///mnt/Data/darkworks/crates/render/src/nodes/signals.rs), [node_graph.rs](file:///mnt/Data/darkworks/crates/render/src/node_graph.rs), [lib.rs](file:///mnt/Data/darkworks/crates/render/src/lib.rs), và các file [color.rs](file:///mnt/Data/darkworks/crates/render/src/nodes/color.rs), [blur_distort.rs](file:///mnt/Data/darkworks/crates/render/src/nodes/blur_distort.rs), [shape.rs](file:///mnt/Data/darkworks/crates/render/src/nodes/shape.rs), [text.rs](file:///mnt/Data/darkworks/crates/render/src/nodes/text.rs), [transform_blend.rs](file:///mnt/Data/darkworks/crates/render/src/nodes/transform_blend.rs): Thực thi đầy đủ Nhóm A, `resolve_video_input` đọc đúng `V1..V4` / `All Below` / `Auto` (kể cả FX Clip, chống lặp), `compute_audio_rms_map` đọc riêng từng track (`Master`/`Linked`/`A1..A4`), và sửa FX Clip overlay không xóa video bên dưới.
- [crates/engine/src/presets.rs](file:///mnt/Data/darkworks/crates/engine/src/presets.rs): Bổ sung `PresetLibrary::data_dir()`.
- [crates/ui-egui/src/panels/node_graph_presets.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/node_graph_presets.rs), [node_graph.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/node_graph.rs), [node_graph_menus.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/node_graph_menus.rs), [panels.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels.rs): Hộp thoại `💾 Save Preset` (lưu vào `<data_dir>/Node Presets/` dùng chung mọi project), `💾 Export .preset.json`, `📥 Import Preset` và chế độ **Replace / Chain**.

### Kế hoạch, Feature Map & Bộ nhớ
- [phase-3-node-graph.md](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/project-managers/phase-3-node-graph.md), [master-plan-a-z.md](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/project-managers/master-plan-a-z.md), [darkworks_node_presets_keyframes_migration.md](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/features/darkworks_node_presets_keyframes_migration.md), [decision-log.md](file:///mnt/Data/darkworks/.darkworks-ai/2-memory/decision-log.md) (#39), [node-graph/architecture.md](file:///mnt/Data/darkworks/.darkworks-ai/3-modules/node-graph/architecture.md), [ui/architecture.md](file:///mnt/Data/darkworks/.darkworks-ai/3-modules/ui/architecture.md), [system_map.md](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/system_map.md).

---

## 3. CÁC LỖI ĐÃ GIẢI QUYẾT TRONG PHIÊN (RESOLVED ISSUES)
1. **Triển khai Lưu / Xuất / Nhập `.preset.json` đầy đủ (`Task 3.12` / `DW-06`):** Giữ nguyên 100% dây nối, tọa độ, tên tùy chỉnh và `VisualGroup`; lưu dùng chung mọi project tại `<data_dir>/Node Presets/` và hỗ trợ Replace/Chain.
2. **Nâng cấp Nhóm A Atomic Nodes:** Hoàn thiện `ColorGrade` (Lumetri tonal + split tone + `Exp Mod`), `BlurSharpen` (Box thật + `threshold` + `Radius Mod`), `DistortLens` (`Mirror`, `Offset`, `Posterize`, `Pixelate`, `feather`), `ShapeGenerator` & `TextGenerator` (`position`, `stroke`, `feather`, `Font` ComboBox, `Bold/Italic`, `Align`, `Pos Mod`), `Transition` (`direction`, `ZoomPush` thật, `DipBlack/White`, `BlurDissolve`), `TimeDriver`/`AudioReactor` (`Vec2 Out`).
3. **`VideoInput` không chọn được V2 & FX Clip overlay xóa video V1:** Thêm ComboBox chọn track trên `VideoInput`, nâng cấp `resolve_video_input` đọc đúng track (kể cả FX Clip có chống đệ quy) và sửa `crates/render/src/lib.rs` composite FX Clip không có `VideoInput` đè lên `below`.
4. **Lỗi đè chữ `Color:` + `Bold/Italic/Font Family` trên `TextGenerator` & `Scale` trên `Transform2D`:** Đổi `render_node_body` sang `ui.vertical` (khắc phục `Layout::left_to_right` của `egui_snarl`), bỏ padding dư ở `show_input`, tách `show_output`, thêm `Scale X/Y`.

---

## 4. KẾ HOẠCH BÀN GIAO CA KẾ TIẾP (NEXT ACTIONS CHECKLIST)
> ⚠️ **BẮT BUỘC TUÂN THỦ LUẬT #1:** Người Dùng vừa báo lỗi **`Audio Reactor` có vẻ fail khi test thực tế**. Khi mở phiên tiếp theo, **PHẢI phân tích RCA và hỏi ý kiến Người Dùng trước khi sửa code!**

- [ ] **1. Điều tra & sửa lỗi `Audio Reactor` (ƯU TIÊN SỐ 1 — Hỏi User trước khi sửa code):**
  - Kiểm tra kỹ vì sao `Audio Reactor` chưa chạy / ra RMS = `0` hoặc không tác động đúng khi test trên app (`compute_audio_rms_map` trong [signals.rs](file:///mnt/Data/darkworks/crates/render/src/nodes/signals.rs), `crate::audio::mix_sequence` / `track_input`, `sources` truyền vào `render_seq`, hoặc việc `eval_item_graph` bị cache frame tĩnh của `render`/viewport khi audio thay đổi!).
  - Gợi ý kiểm tra: khi clip video có `node_graph` chứa `AudioReactor` hoặc `TimeDriver`, kiểm tra xem cache khung hình ở Viewport/Engine (`playback` / `render_cache` / `frame_key`) có biết frame thay đổi theo thời gian hay đang cache theo nguồn tĩnh không, và kiểm tra `start` / `win_frames` / `sources` khi lấy mẫu audio từ `FfmpegMediaSource` hoặc clip audio!
- [ ] **2. Tiếp tục kiểm tra từng node còn lại & sửa 3 điểm tồn đọng đã soi thấy:**
  - `CustomWgsl` từ toolbar menu ([node_graph_menus.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/node_graph_menus.rs#L140-L147)): đoạn snippet mặc định thiếu `fn fs_main`.
  - `Reroute`: thêm chọn kiểu `Any` / `Scalar` / `Vec2` / `Texture` để bẻ góc được dây tín hiệu vàng/cam.
  - `DarkworksViewer::connect` ([viewer.rs](file:///mnt/Data/darkworks/crates/node-graph/src/viewer.rs)): chặn kéo dây sai kiểu (`NodeDataType::can_connect_to`) ngay trên canvas.
  - Kiểm tra tiếp các node `BlurSharpen`, `DistortLens`, `Blend`, `Transition`, `Keyer`, `CropMask` trên app theo yêu cầu của User.
- [ ] **3. Nghiệm thu `DW-06` (`Test 6.1 – 6.3`) & Triển khai `Task 3.13` (Node & Preset Library Browser):**
  - Xây dựng **Node & Preset Library Browser** (tìm kiếm real-time, lọc Category/Favorites, lưới Thumbnail xem trước tự render qua Node Graph trên `preview_image`, Quick Search `Tab`, và kéo thả thẳng vào Node Graph hoặc Clip trên Timeline).
  - Chuyển đổi bộ 18 `lumetri_presets` + Micro-Presets sang `.preset.json`.
