# SỔ BÀN GIAO TIẾN ĐỘ CA KÍP: DARKWORKS (checkpoint.md)
@project: DARKWORKS | @phòng_ban: Toàn xưởng | @status: 🟢 Sẵn Sàng Cho Phiên Nâng Cấp Chuyên Sâu `Ultra Keyer` Pro
@session_end: 2026-11-26 | @next_session_focus: Triển khai toàn bộ kế hoạch [ultra-keyer-pro-upgrade.md](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/project-managers/ultra-keyer-pro-upgrade.md) (nâng cấp `DarkNode::Keyer` lên chuẩn Pro NLE 100% GPU)

> Sổ ghi chép bàn giao công việc giữa các phiên làm việc của AI.  
> Khi mở phiên mới (`/start_session`), nhân viên đọc file này để nắm tiến độ dở dang trong 3 giây.

---

## 1. THÔNG TIN NHÁNH GIT & TRẠNG THÁI HIỆN TẠI
- **Nhánh Git:** `main` (`ahead 1`, có các thay đổi chưa commit của phiên 2026-11-26 — chờ ý kiến Người Dùng để commit/push).
- **Môi trường:** Linux x86_64, Vulkan (`AMD Radeon Graphics RADV RENOIR`) / VA-API, Pure Rust (`wgpu` + `egui` + `egui_snarl`).
- **Kết quả kiểm toán cuối phiên (PASS 100%):**
  - `cargo check --workspace --all-targets`: **PASS** (`54.63s`, 0 errors).
  - `cargo clippy --workspace --all-targets -- -D warnings`: **PASS** (`47.47s`, 0 warnings, 0 errors).
  - `cargo xtask layers`: **PASS** (`layers: ok`).
  - `python3 .agents/harness/audit.py`: **0 vi phạm ngưỡng `>= 700` dòng** (`919` files: `21` đạt chuẩn, `1` cảnh báo `> 650` dòng ở [crates/node-graph-core/src/node.rs](file:///mnt/Data/darkworks/crates/node-graph-core/src/node.rs) — 654 dòng, `897` ngoại lệ Upstream/Vendor).
  - **Tests đã chạy sau lần sửa code liên quan:**
    - `filmcraft-render`: `282 passed; 0 failed; 3 ignored` (`lib`), `6 passed` (`tests/it`), `1 passed` (`mixer_perf`) — đã chạy sau khi tách [node_graph_tests.rs](file:///mnt/Data/darkworks/crates/render/src/node_graph_tests.rs).
    - `darkworks-node-graph-core` (`3/3`), `darkworks-node-graph` (`15/15` + `1/1`), `filmcraft-gpu` (`57 passed; 0 failed; 4 ignored` + `1` `export_pipeline`, parity `ultra_key` GPU thật `max rel diff` `6.88e-7` / `3.67e-7`, `0.000% flipped`), `filmcraft-golden` (`cpu::vfx_ultra_key` `1/1`): **PASS** trước khi tách file test sang `node_graph_tests.rs`.

---

## 2. DANH SÁCH FILE THAY ĐỔI KÈM LIÊN KẾT MARKDOWN

### Shader GPU, Planner & Node Graph Render (`crates/gpu`, `crates/render`)
- [crates/gpu/src/fx.wgsl](file:///mnt/Data/darkworks/crates/gpu/src/fx.wgsl) & [crates/gpu/src/fx.rs](file:///mnt/Data/darkworks/crates/gpu/src/fx.rs): Thêm `OP_CHROMA_KEY = 37` chạy Y'CbCr (Bt.709) + 1px neighbor choke kẹp biên (`clamp`) + spill suppression (`G <= max(R, B)`).
- [crates/gpu/src/fx_tests.rs](file:///mnt/Data/darkworks/crates/gpu/src/fx_tests.rs): Bổ sung 2 case `ultra_key` vào `gpu_effects_match_cpu_exactly`.
- [crates/render/src/gpufx.rs](file:///mnt/Data/darkworks/crates/render/src/gpufx.rs): Thêm `FxOp::ChromaKey`, đưa `"ultra_key"` vào `GPU_EFFECTS` (mở GPU cho FilmCraft `ultra_key`), hàm CPU `chroma_key_matte` / `chroma_key_cpu`.
- [crates/render/src/nodes/gpu_plan.rs](file:///mnt/Data/darkworks/crates/render/src/nodes/gpu_plan.rs) *(mới)* & [crates/render/src/plan.rs](file:///mnt/Data/darkworks/crates/render/src/plan.rs): Biên dịch nhánh Node Graph tuyến tính (`VideoInput("Auto"|"Clip") -> Keyer/ColorGrade/BlurSharpen/DistortLens/CropMask/Transform2D -> Output`) trực tiếp thành `FramePlan::Layers` + `LayerFx` chạy 100% GPU; bổ sung 3 unit tests.
- [crates/render/src/nodes.rs](file:///mnt/Data/darkworks/crates/render/src/nodes.rs), [color.rs](file:///mnt/Data/darkworks/crates/render/src/nodes/color.rs), [blur_distort.rs](file:///mnt/Data/darkworks/crates/render/src/nodes/blur_distort.rs), [signals.rs](file:///mnt/Data/darkworks/crates/render/src/nodes/signals.rs), [lib.rs](file:///mnt/Data/darkworks/crates/render/src/lib.rs): Tối ưu CPU fallback (`take_or_clone_texture`, bỏ `snapshot()` thừa, tái dùng `below_image`, nâng cấp `BlurSharpen`/`DistortLens`, đồng bộ `ApplyKeyer` dùng `FxOp::ChromaKey`).
- [crates/render/src/node_graph.rs](file:///mnt/Data/darkworks/crates/render/src/node_graph.rs) (`416` dòng) & [crates/render/src/node_graph_tests.rs](file:///mnt/Data/darkworks/crates/render/src/node_graph_tests.rs) *(mới, `296` dòng)*: Dùng `take_or_clone_texture` + `pool::recycle_f32` và tách khối `#[cfg(test)]` sang file riêng.

### Node Schema, UI Controls & Kế Hoạch Phiên Sau
- [crates/node-graph-core/src/lib.rs](file:///mnt/Data/darkworks/crates/node-graph-core/src/lib.rs) & [crates/node-graph-core/src/node.rs](file:///mnt/Data/darkworks/crates/node-graph-core/src/node.rs): Bổ sung `BlurDimensions`, `BlurType::RadialZoom`, `DistortType::{Twirl, FindEdges, NoiseGrain, TurbulentDisplace}`, `Reroute` (`Any` default).
- [crates/node-graph/src/lib.rs](file:///mnt/Data/darkworks/crates/node-graph/src/lib.rs), [node.rs](file:///mnt/Data/darkworks/crates/node-graph/src/node.rs), [node_controls.rs](file:///mnt/Data/darkworks/crates/node-graph/src/node_controls.rs), [node_controls_fx.rs](file:///mnt/Data/darkworks/crates/node-graph/src/node_controls_fx.rs) *(mới)*, [preset.rs](file:///mnt/Data/darkworks/crates/node-graph/src/preset.rs), [viewer.rs](file:///mnt/Data/darkworks/crates/node-graph/src/viewer.rs), [tests/graph_tests.rs](file:///mnt/Data/darkworks/crates/node-graph/tests/graph_tests.rs), [crates/ui-egui/src/panels/node_graph_menus.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/node_graph_menus.rs): Tách `node_controls_fx.rs`, lọc tham số UI theo chế độ `DistortLens` / `BlurSharpen`, kiểm tra nối dây `viewer::connect`, `CustomWgsl` template mới.
- [ultra-keyer-pro-upgrade.md](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/project-managers/ultra-keyer-pro-upgrade.md) *(mới)*, [node-graph/architecture.md](file:///mnt/Data/darkworks/.darkworks-ai/3-modules/node-graph/architecture.md), [decision-log.md](file:///mnt/Data/darkworks/.darkworks-ai/2-memory/decision-log.md) (#40), [system_map.md](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/system_map.md).

---

## 3. CÁC LỖI ĐÃ GIẢI QUYẾT TRONG PHIÊN (RESOLVED ISSUES)
1. **RCA & Sửa lỗi `AudioReactor` + `ColorGrade` (`Exp` / Preset 4):** Khớp hệ số phơi sáng tuyến tính + `Exp Mod * 2.5`, xác nhận `AudioBeatFlash` hoạt động rõ rệt.
2. **Sửa 3 tồn đọng Node Graph:** `Reroute` đổi kiểu trực tiếp (`Any` default), `CustomWgsl` từ menu có sẵn `fn fs_main`, và `DarkworksViewer::connect` chặn kéo dây sai kiểu ngay trên Canvas.
3. **Hoàn thiện `BlurSharpen` & `DistortLens` + UI lọc tham số:** Ẩn các ô `Radius`/`Center X/Y` không liên quan ở `Posterize`, `ChromaticAberration`...; bổ sung `RadialZoom`, `dimensions`, `repeat_edge`, `bokeh_sides`, `Twirl`, `FindEdges`, `NoiseGrain`, `TurbulentDisplace`; tách `node_controls_fx.rs`.
4. **Sửa lag 87% CPU & viền xanh ở `Ultra Keyer` + FilmCraft `ultra_key`:** Đưa Y'CbCr Bt.709 + 1px choke kẹp biên + spill suppression lên GPU `OP_CHROMA_KEY = 37` và biên dịch nhánh tuyến tính `Apply to Clip` qua `gpu_plan.rs` (Người Dùng đã xác nhận test thực tế hết lag khi nối `Video Input (Auto) -> Ultra Keyer -> Final Output` bằng `Apply to Clip`).
5. **Tách `node_graph_tests.rs`:** Rút [node_graph.rs](file:///mnt/Data/darkworks/crates/render/src/node_graph.rs) từ 712 xuống 416 dòng (`< 700` dòng).

---

## 4. KẾ HOẠCH BÀN GIAO CA KẾ TIẾP (NEXT ACTIONS CHECKLIST)
> 🎯 **Trọng tâm duy nhất phiên sau:** Thực hiện theo [ultra-keyer-pro-upgrade.md](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/project-managers/ultra-keyer-pro-upgrade.md) để hoàn thiện node **`🟢 Ultra Keyer`** lên chuẩn Pro NLE.

- [ ] **1. Tách enum/struct phụ khỏi [node.rs](file:///mnt/Data/darkworks/crates/node-graph-core/src/node.rs) (đang 654 dòng) & mở rộng `DarkNode::Keyer` (`#[serde(default)]`):**
  - Thêm `Mode` (`ChromaKey` / `LumaKey`), `Output` (`Composite` / `AlphaChannel` / `ColorChannel`), `Pedestal`, `Highlight`, `Shadow`, `Choke`, `Soften`, `Matte Contrast`, `Desaturate`, `Spill Luma`, `Invert Matte` (+ tùy chọn pin `Garbage Matte`).
- [ ] **2. Nâng cấp Shader WGSL (`fx.wgsl` / `fx.rs` / `gpufx.rs`):**
  - Hỗ trợ đầy đủ tham số mới trên GPU (gồm `choke`/`soften`, `AlphaChannel`, `LumaKey`, bù sáng `spill_luma`), đồng thời mở khóa GPU cho các preset `Relaxed` / `Aggressive` của FilmCraft `ultra_key`.
- [ ] **3. Đưa `Add as FX Clip` (`AdjustmentLayer`) & Graph 4 node (`V1 + Blend`) lên GPU (`plan.rs` / `gpu_plan.rs`):**
  - Biên dịch `AdjustmentLayer` tuyến tính thành `PlanStep::Adjust` và mẫu `Blend(VideoInput("All Below" | "V1"))` thành `PlanLayer { fx, blend, opacity }`.
- [ ] **4. UI lọc theo Mode (`node_controls_fx.rs`) & Eyedropper trên Program Monitor (`monitor.rs`):**
  - Nhóm gọn `Matte Cleanup` / `Spill` và cho phép bấm nút hút màu trên node rồi click thẳng vào vùng xanh trong Program Monitor.
