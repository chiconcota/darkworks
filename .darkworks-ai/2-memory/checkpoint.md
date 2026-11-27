# SỔ BÀN GIAO TIẾN ĐỘ CA KÍP: DARKWORKS (checkpoint.md)
@project: DARKWORKS | @phòng_ban: Toàn xưởng | @status: 🟢 Hoàn Tất `Ultra Keyer` Pro GPU + 14 `NodeFxOp` + Shared Eyedropper + Refactor `gpu_plan.rs`
@session_end: 2026-11-27 | @next_session_focus: Test thực tế các Atomic Nodes còn lại trên Node Graph & triển khai **Node & Preset Library Browser** (`Task 3.13`)

> Sổ ghi chép bàn giao công việc giữa các phiên làm việc của AI.  
> Khi mở phiên mới (`/start_session`), nhân viên đọc file này để nắm tiến độ dở dang trong 3 giây.

---

## 1. THÔNG TIN NHÁNH GIT & TRẠNG THÁI HIỆN TẠI
- **Nhánh Git:** `main` (đang có các thay đổi chưa commit của phiên 2026-11-27 — chờ ý kiến Người Dùng để commit/push).
- **Môi trường:** Linux x86_64, Vulkan (`AMD Radeon Graphics RADV RENOIR`) / VA-API, Pure Rust (`wgpu` + `egui` + `egui_snarl`).
- **Kết quả kiểm toán cuối phiên (SAU LẦN SỬA CUỐI — PASS 100%):**
  - `cargo check --workspace --all-targets`: **PASS** (`1m 15s`, 0 errors).
  - `cargo clippy --workspace --all-targets -- -D warnings`: **PASS** (`31.13s`, 0 warnings, 0 errors).
  - `cargo xtask layers`: **PASS** (`layers: ok`).
  - `python3 .agents/harness/audit.py`: **0 vi phạm `>= 700` dòng, 0 cảnh báo `> 650` dòng** (`924` files: `23` đạt chuẩn, `901` ngoại lệ Upstream/Vendor).
  - **Tests đã chạy:**
    - `cargo test -p filmcraft-render` (sau lần sửa cuối): **287 passed; 0 failed; 3 ignored** (`lib`), **6 passed** (`tests/it`), **1 passed** (`mixer_perf`) — gồm `video_input_matching_clip_track_compiles_to_gpu_layers` và `video_input_matching_clip_track_with_blend_v1_compiles_to_gpu_layers`.
    - Trước bước refactor cuối của `gpu_plan.rs` (các crate tương ứng không đổi sau bước đó): `darkworks-node-graph-core` (`4 passed`), `darkworks-node-graph` (`15 passed` + `1 passed`), `filmcraft-gpu` (`58 passed` lib + `1 passed` `export_pipeline`), `filmcraft-ui-egui --test it scripted::eyedropper` (`1 passed`, `813 filtered out`).

---

## 2. DANH SÁCH FILE THAY ĐỔI KÈM LIÊN KẾT MARKDOWN

### Schema & UI Node Graph (`crates/node-graph-core`, `crates/node-graph`, `crates/ui-egui`)
- [crates/node-graph-core/src/node_fx_types.rs](file:///mnt/Data/darkworks/crates/node-graph-core/src/node_fx_types.rs) *(mới, `279` dòng)*, [node.rs](file:///mnt/Data/darkworks/crates/node-graph-core/src/node.rs) (`532` dòng) & [lib.rs](file:///mnt/Data/darkworks/crates/node-graph-core/src/lib.rs): Tách shared enums/defaults; mở rộng `DarkNode::Keyer` lên Pro (`ChromaKey`/`LumaKey`, `Composite`/`AlphaChannel`/`ColorChannel`, `pedestal`, `highlight`, `shadow`, `choke`, `soften`, `matte_contrast`, `invert_matte`, `desaturate`, `spill_luma`, `luma_threshold`, `luma_cutoff`, tương thích ngược `#[serde(default)]`).
- [crates/node-graph/src/viewer.rs](file:///mnt/Data/darkworks/crates/node-graph/src/viewer.rs), [node_controls.rs](file:///mnt/Data/darkworks/crates/node-graph/src/node_controls.rs), [node_controls_fx.rs](file:///mnt/Data/darkworks/crates/node-graph/src/node_controls_fx.rs) (`405` dòng), [node.rs](file:///mnt/Data/darkworks/crates/node-graph/src/node.rs), [lib.rs](file:///mnt/Data/darkworks/crates/node-graph/src/lib.rs), [tests/graph_tests.rs](file:///mnt/Data/darkworks/crates/node-graph/tests/graph_tests.rs): Bổ sung pin `Garbage Matte` / `Matte Out`, UI `Keyer` lọc thông minh theo `ChromaKey`/`LumaKey` + nhóm thu gọn Cleanup/Spill.
- [crates/ui-egui/src/panels/monitor.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/monitor.rs), [state.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/state.rs), [effect_controls.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/effect_controls.rs), [node_graph.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/node_graph.rs), [node_graph_bridge.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/node_graph_bridge.rs), [node_graph_menus.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/node_graph_menus.rs): Tích hợp **Shared Eyedropper** trên Program Monitor cho cả 4 node chọn màu (`Keyer` tự bypass khi hút màu, `ColorGrade`, `TextGenerator`, `ShapeGenerator`) cho cả bound clip lẫn scratchpad.

### GPU Compute & Render Planner (`crates/gpu`, `crates/render`)
- [crates/gpu/src/node_fx.wgsl](file:///mnt/Data/darkworks/crates/gpu/src/node_fx.wgsl) *(mới, `503` dòng)*, [crates/gpu/src/node_fx.rs](file:///mnt/Data/darkworks/crates/gpu/src/node_fx.rs) *(mới, `190` dòng)*, [fx.rs](file:///mnt/Data/darkworks/crates/gpu/src/fx.rs), [lib.rs](file:///mnt/Data/darkworks/crates/gpu/src/lib.rs), [fx_tests.rs](file:///mnt/Data/darkworks/crates/gpu/src/fx_tests.rs): Triển khai 14 `NodeFxOp` trên GPU (`ChromaKeyPro` 3-texture ping-pong `OP_NODE_KEYER_MATTE`/`CHOKE`/`FINISH`, `LumaKey`, `RadialZoom`, 9 chế độ `DistortLens`: `Pixelate`, `ChromaticAberration`, `RippleWave`, `LensDistort`, `Twirl`, `FindEdges`, `NoiseGrain`, `TurbulentDisplace`).
- [crates/render/src/nodes/node_fx_ops.rs](file:///mnt/Data/darkworks/crates/render/src/nodes/node_fx_ops.rs) *(mới, `536` dòng)*, [node_fx_distort_cpu.rs](file:///mnt/Data/darkworks/crates/render/src/nodes/node_fx_distort_cpu.rs) *(mới, `279` dòng)*, [gpufx.rs](file:///mnt/Data/darkworks/crates/render/src/gpufx.rs), [effects.rs](file:///mnt/Data/darkworks/crates/render/src/effects.rs), [lib.rs](file:///mnt/Data/darkworks/crates/render/src/lib.rs), [nodes.rs](file:///mnt/Data/darkworks/crates/render/src/nodes.rs), [node_graph.rs](file:///mnt/Data/darkworks/crates/render/src/node_graph.rs), [node_graph_tests.rs](file:///mnt/Data/darkworks/crates/render/src/node_graph_tests.rs): Hợp nhất CPU/GPU parity cho `NodeFxOp`, mở khóa GPU cho FilmCraft `ultra_key` (`Relaxed`/`Aggressive`/`Soften`/`Choke`/`AlphaChannel`/`ColorChannel`), hỗ trợ `(out_node, out_pin)` và `Garbage Matte` / `Matte Out`.
- [crates/render/src/nodes/gpu_plan.rs](file:///mnt/Data/darkworks/crates/render/src/nodes/gpu_plan.rs) (`369` dòng), [gpu_plan_tests.rs](file:///mnt/Data/darkworks/crates/render/src/nodes/gpu_plan_tests.rs) *(mới, `392` dòng)*, [color.rs](file:///mnt/Data/darkworks/crates/render/src/nodes/color.rs) (`383` dòng), [blur_distort.rs](file:///mnt/Data/darkworks/crates/render/src/nodes/blur_distort.rs) (`450` dòng), [transform_blend.rs](file:///mnt/Data/darkworks/crates/render/src/nodes/transform_blend.rs) (`290` dòng), [signals.rs](file:///mnt/Data/darkworks/crates/render/src/nodes/signals.rs) (`452` dòng), [plan.rs](file:///mnt/Data/darkworks/crates/render/src/plan.rs): Hỗ trợ `Add as FX Clip` (`AdjustmentLayer` -> `PlanStep::Adjust`), `Blend(V1/All Below)` (kể cả khi `V1` trống), `VideoInput` trùng track (`V2` trên track `V2`), và tách phần biên dịch `FxOp` về `color.rs` / `blur_distort.rs` / `transform_blend.rs` để `gpu_plan.rs` gọn ở `369` dòng.

### Tài Liệu & Kế Hoạch (`.darkworks-ai/`)
- [ultra-keyer-pro-upgrade.md](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/project-managers/ultra-keyer-pro-upgrade.md), [master-plan-a-z.md](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/project-managers/master-plan-a-z.md), [node-graph/architecture.md](file:///mnt/Data/darkworks/.darkworks-ai/3-modules/node-graph/architecture.md), [decision-log.md](file:///mnt/Data/darkworks/.darkworks-ai/2-memory/decision-log.md) (#41), [system_map.md](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/system_map.md).

---

## 3. CÁC LỖI ĐÃ GIẢI QUYẾT TRONG PHIÊN (RESOLVED ISSUES)
1. **Nâng cấp `Ultra Keyer` lên chuẩn Pro & Tăng tốc GPU toàn bộ hiệu ứng Node Graph:** `ChromaKeyPro` 3-pass + `LumaKey` + `RadialZoom` + 9 chế độ `DistortLens` chạy Compute Shader trên `wgpu` (`node_fx.wgsl`), mở khóa luôn GPU cho các tùy chọn nâng cao của FilmCraft `ultra_key`.
2. **Shared Eyedropper trên Program Monitor:** Hỗ trợ hút màu trực tiếp cho `Keyer` (auto-bypass khi hút), `ColorGrade`, `TextGenerator`, `ShapeGenerator` ở cả chế độ gắn clip lẫn scratchpad.
3. **Sửa fallback CPU khi dùng `VideoInput("V2")` + `Blend(V1)` trên track `V2`:** Cập nhật `is_self_input` và `single_below_track_name` (kể cả khi `V1` trống), xác minh bằng unit tests mới trong `gpu_plan_tests.rs`.
4. **Rút gọn `gpu_plan.rs` từ 705 xuống 369 dòng:** Tách per-node GPU lowering về `color.rs`, `blur_distort.rs`, `transform_blend.rs` theo Cách 1 — toàn bộ file `crates/render/src/nodes/*.rs` và repo Darkworks đều đạt chuẩn `< 700` dòng.

---

## 4. KẾ HOẠCH BÀN GIAO CA KẾ TIẾP (NEXT ACTIONS CHECKLIST)
- [ ] **1. Test thực tế các Atomic Nodes còn lại trên giao diện & video 4K/1080p:**
  - Kiểm thử từng nhóm: `ColorGrade` (Lumetri tonal + Split Tone), `BlurSharpen` (6 kiểu gồm `RadialZoom` và `BokehBloom`), `DistortLens` (13 kiểu), `CropMask`, `Transform2D`, `Blend`, `AudioReactor`, `TimeDriver`, `TextGenerator`, `ShapeGenerator`, `Reroute`, `CustomWgsl`.
  - Theo dõi trường hợp Người Dùng phát hiện trên `Node Graph FX` (Properties panel hiện `Position: NaN X NaN Y` / `Anchor point: NaN X NaN Y` khi tạo overlay FX Clip — kiểm tra RCA nếu Người Dùng yêu cầu ở ca sau).
- [ ] **2. Theo dõi các tồn đọng đã ghi nhận cho Phase 4:**
  - `DAG Multi-Texture Pool` trên GPU (đồ thị đa nhánh phức tạp không fallback CPU) và `Transition` node trên GPU (`gpu_plan.rs` / `node_fx.wgsl`).
  - Triển khai **Node & Preset Library Browser** (`Task 3.13`) và chuyển sang Giai đoạn 4 (Vello Canvas 2D + Keyframe Node Graph).
