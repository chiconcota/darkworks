# SỔ BÀN GIAO TIẾN ĐỘ CA KÍP: DARKWORKS (checkpoint.md)
@project: DARKWORKS | @phòng_ban: Toàn xưởng | @status: 🟢 Nâng Cấp `Transform2D` 100% GPU (`ResampleArea`, `Uniform`, `Skew`) + `Blend` Nối 1 Dây GPU + Quy Hoạch `CropMask`/`Transform 3D`
@session_end: 2026-11-28 | @next_session_focus: Triển khai UX `Transition` Node + `TimeDriver (Clip01)` + mặc định 2s cho FX Clip chứa Transition, và tiếp tục test các Atomic Nodes còn lại

> Sổ ghi chép bàn giao công việc giữa các phiên làm việc của AI.  
> Khi mở phiên mới (`/start_session`), nhân viên đọc file này để nắm tiến độ dở dang trong 3 giây.

---

## 1. THÔNG TIN NHÁNH GIT & TRẠNG THÁI HIỆN TẠI
- **Nhánh Git:** `main` (đang có các thay đổi chưa commit của phiên 2026-11-27 và 2026-11-28 — chờ ý kiến Người Dùng để commit/push).
- **Môi trường:** Linux x86_64, Vulkan (`AMD Radeon Graphics RADV RENOIR`) / VA-API, Pure Rust (`wgpu` + `egui` + `egui_snarl`).
- **Kết quả kiểm toán cuối phiên (SAU LẦN SỬA MÃ NGUỒN CUỐI — PASS 100%):**
  - `cargo check --workspace --all-targets`: **PASS** (`11.47s`, 0 errors).
  - `cargo clippy --workspace --all-targets -- -D warnings`: **PASS** (0 warnings, 0 errors).
  - `cargo xtask layers`: **PASS** (`layers: ok`).
  - `python3 .agents/harness/audit.py`: **0 vi phạm `>= 700` dòng, 0 cảnh báo `> 650` dòng** (`924` files: `23` đạt chuẩn, `901` ngoại lệ Upstream/Vendor).
  - **Tests đã chạy sau lần sửa `.rs` cuối:**
    - `darkworks-node-graph`: **15 passed** (`tests/graph_tests.rs`) + **1 passed** (`tests/history_tests.rs`).
    - `darkworks-node-graph-core`: **4 passed** (`lib`).
    - `filmcraft-gpu`: **58 passed; 0 failed; 4 ignored** (`lib`, gồm `node_fx_pro_keyer_and_distort_match_cpu_exactly` kiểm chứng `resample_area` khớp 1:1 GPU/CPU) + **1 passed** (`export_pipeline`).
    - `filmcraft-render`: **289 passed; 0 failed; 3 ignored** (`lib`, gồm `single_wired_blend_stays_on_gpu_layers` & `transform_small_scale_scale_mod_and_skew_stay_on_gpu`) + **6 passed** (`tests/it`) + **1 passed** (`mixer_perf`).

---

## 2. DANH SÁCH FILE THAY ĐỔI TRONG CA 2026-11-28 (KÈM LIÊN KẾT MARKDOWN)
*(Lưu ý: trên `main` còn các thay đổi chưa commit của ca 2026-11-27).*

### Schema & UI Node Graph (`crates/node-graph-core`, `crates/node-graph`, `crates/ui-egui`)
- [crates/node-graph-core/src/node.rs](file:///mnt/Data/darkworks/crates/node-graph-core/src/node.rs) (`555` dòng): Thêm `uniform_scale`, `skew`, `skew_axis` (`#[serde(default)]` giữ tương thích ngược) cho `DarkNode::Transform2D` và constructor `DarkNode::default_transform()` (mặc định bật `uniform_scale: true`).
- [crates/node-graph/src/node_controls.rs](file:///mnt/Data/darkworks/crates/node-graph/src/node_controls.rs) (`539` dòng), [viewer.rs](file:///mnt/Data/darkworks/crates/node-graph/src/viewer.rs) (`598` dòng), [tests/graph_tests.rs](file:///mnt/Data/darkworks/crates/node-graph/tests/graph_tests.rs), [crates/ui-egui/src/panels/node_graph_menus.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/node_graph_menus.rs) (`153` dòng): Giao diện `Transform2D` có checkbox `Uniform (Lock X = Y)`, `Skew/Axis`, và dùng `default_transform()`.

### GPU Compute & Render Planner (`crates/gpu`, `crates/render`)
- [crates/gpu/src/node_fx.wgsl](file:///mnt/Data/darkworks/crates/gpu/src/node_fx.wgsl) (`526` dòng), [node_fx.rs](file:///mnt/Data/darkworks/crates/gpu/src/node_fx.rs) (`198` dòng), [fx_tests.rs](file:///mnt/Data/darkworks/crates/gpu/src/fx_tests.rs): Bổ sung `OP_NODE_RESAMPLE_AREA = 114` (`2×2..8×8` supersampling trên GPU cho thu nhỏ `< 0.5`) và test GPU/CPU parity `resample_area`.
- [crates/render/src/nodes/node_fx_ops.rs](file:///mnt/Data/darkworks/crates/render/src/nodes/node_fx_ops.rs) (`550` dòng), [node_fx_distort_cpu.rs](file:///mnt/Data/darkworks/crates/render/src/nodes/node_fx_distort_cpu.rs) (`327` dòng): Thêm `NodeFxOp::ResampleArea` và `resample_area_cpu` parity 1:1.
- [crates/render/src/nodes/transform_blend.rs](file:///mnt/Data/darkworks/crates/render/src/nodes/transform_blend.rs) (`338` dòng), [node_graph.rs](file:///mnt/Data/darkworks/crates/render/src/node_graph.rs) (`473` dòng): Tách hàm dùng chung `build_transform_affine` (hỗ trợ `uniform_scale`, `skew`, `skew_axis`), chọn `Resample` khi `minify <= 2.01` và `ResampleArea` khi `minify > 2.01`.
- [crates/render/src/nodes/gpu_plan.rs](file:///mnt/Data/darkworks/crates/render/src/nodes/gpu_plan.rs) (`382` dòng), [gpu_plan_tests.rs](file:///mnt/Data/darkworks/crates/render/src/nodes/gpu_plan_tests.rs) (`578` dòng): Hỗ trợ `Blend` nối 1 dây trên GPU (`Background`-only pass-through hoặc `Foreground`-only `AlphaAdjust`), bỏ chặn `static_chain_ok` để `Transform2D` chạy 100% GPU cả khi `|Scale| < 0.5` hoặc nối `Scale Mod`.

### Kế hoạch & Bộ nhớ (`.darkworks-ai/`)
- [master-plan-a-z.md](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/project-managers/master-plan-a-z.md), [node-graph/architecture.md](file:///mnt/Data/darkworks/.darkworks-ai/3-modules/node-graph/architecture.md), [decision-log.md](file:///mnt/Data/darkworks/.darkworks-ai/2-memory/decision-log.md) (#42), [system_map.md](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/system_map.md).

---

## 3. CÁC LỖI ĐÃ GIẢI QUYẾT & ĐIỂM ĐÃ CHỐT TRONG PHIÊN
1. **Sửa lỗi rớt CPU khi `Blend` chỉ nối 1 đầu (`Transform2D -> Keyer -> Blend -> Output`):** Bổ sung xử lý `Blend` nối 1 dây (không có `Mask`) trong `gpu_plan.rs`, xác minh bằng `single_wired_blend_stays_on_gpu_layers`.
2. **Nâng cấp `Transform2D` lên 100% GPU + `Uniform Scale` + `Skew/Axis`:** Giải quyết triệt để việc thu nhỏ `< 0.5` hoặc cắm `Scale Mod` bị rơi về CPU bằng `NodeFxOp::ResampleArea` (khớp 1:1 GPU/CPU).
3. **Làm rõ câu hỏi về `CropMask` & `Transform 3D`:**
   - Chốt phương án **Cách 1** cho `CropMask` (giữ 1 Node gồm `Edge Crop` + `Bezier / Pen Mask` nối `masks.rs`/`mask.wgsl` + `Matte Out`) vào **Phase 4 (`Task 4.7`)**.
   - Đưa Atomic Node **`Transform 3D`** (Lightworks 3D DVE / Homography) vào **Phase 7 (`Task 7.1`)** sau Milestone/Production.
4. **Phát hiện điểm UX khi test `Transition` Node:** `Add as FX Clip` mặc định tạo clip `5.0s` (hoặc theo Mark In/Out) trong khi `TimeDriver` chỉ chạy theo giây tuyệt đối (`time_sec * speed`), khiến việc căn chuyển cảnh 2s rất khó khớp khi kéo giãn/co clip. Đã đề xuất thêm `TimeMode::Clip01` (`0..1` theo độ dài Clip) + mặc định `2.0s` khi tạo FX Clip chứa `Transition` — **chờ chốt & triển khai ở đầu ca sau**.

---

## 4. KẾ HOẠCH BÀN GIAO CA KẾ TIẾP (NEXT ACTIONS CHECKLIST)
- [ ] **1. Làm UX `Transition` Node & `TimeDriver` (`Clip01`) ngay đầu ca sau:**
  - Chốt với Người Dùng rồi triển khai `TimeMode::Clip01` (chuẩn hóa `(t - item.start) / item.duration` trong `0.0..=1.0`, bất kể độ dài FX Clip trên Timeline) và mặc định `2.0s` khi `Add as FX Clip` cho đồ thị có `Transition` (hoặc tùy chọn nhanh trên UI).
  - Test thực tế đủ 8 kiểu `Transition` (`CrossDissolve`, `LinearWipe`, `Slide`, `Iris`, `ZoomPush`, `DipBlack`, `DipWhite`, `BlurDissolve`) với Clip A trên `V1`, Clip B gối đầu trên `V2`, và `Node Graph FX` trên `V3` (đo hiệu năng, cân nhắc GPU cho `Transition` nếu cần).
- [ ] **2. Test thực tế các Atomic Nodes còn lại:**
  - `ColorGrade`, `BlurSharpen` (6 kiểu), `DistortLens` (13 kiểu), `AudioReactor`, `TextGenerator`, `ShapeGenerator`, `Reroute`, `CustomWgsl` và đồ thị đa nhánh.
  - Vẫn giữ nguyên tắc chưa chuyển sang `Task 3.9` (Node & Preset Library Browser) cho tới khi Người Dùng xác nhận test xong toàn bộ các Node.
