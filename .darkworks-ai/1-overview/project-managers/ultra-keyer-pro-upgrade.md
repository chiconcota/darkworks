# KẾ HOẠCH LÀM VIỆC PHIÊN SAU: HOÀN THIỆN ULTRA KEYER (ultra-keyer-pro-upgrade.md)
@project: DARKWORKS | @module: Ultra Keyer (`DarkNode::Keyer` + GPU `FxOp` + Monitor Eyedropper) | @status: 🔴 CHỜ TRIỂN KHAI

> **Mục tiêu phiên làm việc:** Tập trung nâng cấp duy nhất node **`🟢 Ultra Keyer`** (và nhánh GPU liên quan) từ bản cơ bản (~65%) lên bản **Pro NLE hoàn chỉnh 100%**, chạy mượt toàn bộ bằng GPU WGSL và giữ tương thích ngược với các project/preset hiện có.

---

## 1. HIỆN TRẠNG NỀN TẢNG (TỪ PHIÊN TRƯỚC)

### Đã hoàn thành & kiểm chứng
- **Lõi GPU Compute (`OP_CHROMA_KEY = 37`):** đã có tại [fx.wgsl](file:///mnt/Data/darkworks/crates/gpu/src/fx.wgsl), [fx.rs](file:///mnt/Data/darkworks/crates/gpu/src/fx.rs), [gpufx.rs](file:///mnt/Data/darkworks/crates/render/src/gpufx.rs) dùng tách sắc độ Y'CbCr (Bt.709), neighbor choke 1px (kẹp biên khung hình) và kìm kênh màu chủ đạo (`G <= max(R, B)`).
- **Nhánh GPU cho Clip tuyến tính:** `Apply to Clip` với `VideoInput (Auto) -> Keyer -> Output` đi qua [gpu_plan.rs](file:///mnt/Data/darkworks/crates/render/src/nodes/gpu_plan.rs) vào `FramePlan::Layers`, hết hẳn lag CPU.
- **Tách gọn test render:** [node_graph.rs](file:///mnt/Data/darkworks/crates/render/src/node_graph.rs) còn 416 dòng, test tách sang [node_graph_tests.rs](file:///mnt/Data/darkworks/crates/render/src/node_graph_tests.rs) (296 dòng).

### Khoảng trống còn lại (Phạm vi giải quyết ở phiên này)
1. Node `DarkNode::Keyer` mới có 4 tham số (`key_color`, `similarity`, `smoothness`, `spill_suppress`), chưa có `Pedestal`, `Highlight`, `Shadow`, `Choke`, `Soften`, `Matte Contrast`, `Spill Luma` / `Desaturate`, và chưa có chế độ **`Luma Key`** (tách nền đen/trắng).
2. Chưa có chế độ xem **`Output` (`Composite` / `Alpha Channel` trắng-đen / `Color Channel`)** để soi sạch nền khi tách phông thật.
3. Chưa có **Eyedropper (ống hút màu)** click trực tiếp lên Program Monitor.
4. Chưa có **Mask / Garbage Matte** (khoanh vùng giữ/xóa cưỡng bức) và **Matte Out**.
5. Vẫn rớt về CPU (lag) ở 2 tình huống:
   - Dùng `Add as FX Clip` (`AdjustmentLayer` có `node_graph`) chưa nối vào `PlanStep::Adjust`.
   - Graph nối 4 node `VideoInput("Auto") -> ... -> Blend(Bg = VideoInput("All Below" / "V1"), Mask = None) -> Output`.
   - Effect `ultra_key` gốc của FilmCraft khi chọn `Setting = Relaxed / Aggressive` hoặc `Soften > 0`.

---

## 2. LỘ TRÌNH THỰC HIỆN CHI TIẾT (4 BƯỚC THEO TẦNG CARGO)

### Bước 1 — Mở rộng Schema `DarkNode::Keyer` & Tương thích ngược (`L0: crates/node-graph-core`)
- **Lưu ý trần 700 dòng:** [node.rs](file:///mnt/Data/darkworks/crates/node-graph-core/src/node.rs) hiện đang **654 dòng** (`> 650` cảnh báo). Trước khi thêm struct/enum cho Keyer, tách enum `KeyerMode` / `KeyerOutput` / helper mặc định sang module con (ví dụ `crates/node-graph-core/src/node_fx_types.rs` hoặc tái sử dụng file con hợp lý) để không vượt 700 dòng.
- **Dữ liệu `DarkNode::Keyer` bổ sung (bắt buộc gắn `#[serde(default = ...)]` cho mọi field mới để không vỡ file `project.json` / `.preset.json` cũ):**
  - `mode: KeyerMode` (`ChromaKey` mặc định | `LumaKey` tách nền đen/trắng).
  - `output: KeyerOutput` (`Composite` | `AlphaChannel` | `ColorChannel`).
  - **Matte Generation:** `pedestal: f32` (mặc định `0.10`), `highlight: f32` (`0.10`), `shadow: f32` (`0.50`).
  - **Matte Cleanup:** `choke: f32` (`0.0`), `soften: f32` (`0.0`), `matte_contrast: f32` (`0.0`), `invert_matte: bool` (`false`).
  - **Spill Suppression (chế độ Chroma):** `desaturate: f32` (`0.25`), `spill_luma: f32` (`0.50`).
  - **Luma Key params (chỉ hiện khi `mode == LumaKey`):** `luma_threshold: f32`, `luma_cutoff: f32`.
- **Cổng nối (Pins) tại [viewer.rs](file:///mnt/Data/darkworks/crates/node-graph/src/viewer.rs):**
  - Giữ pin 0: `Image In (Texture)`.
  - Bổ sung pin 1 (tùy chọn không bắt buộc nối): `Mask / Garbage Matte (Texture)` (alpha = 0 ở mask hoặc vùng đen sẽ ép xóa/giữ theo cấu hình, mặc định không nối = toàn khung hình).
  - Bổ sung output pin 1 (tùy chọn): `Matte Out (Texture)` (hoặc xuất qua `AlphaChannel` để tiết kiệm nếu chưa cần thêm socket phức tạp — chốt lại với User trước khi thêm pin mới).

### Bước 2 — Nâng cấp Thuật toán GPU WGSL & CPU Reference (`L2: crates/render`, `L3: crates/gpu`)
1. **Nâng cấp `FxOp::ChromaKey` & bổ sung `FxOp::LumaKey` trong [gpufx.rs](file:///mnt/Data/darkworks/crates/render/src/gpufx.rs), [fx.rs](file:///mnt/Data/darkworks/crates/gpu/src/fx.rs), [fx.wgsl](file:///mnt/Data/darkworks/crates/gpu/src/fx.wgsl):**
   - Tích hợp đầy đủ `highlight`, `shadow`, `pedestal`, `matte_contrast`, `invert_matte`, `output` (`Composite` / `AlphaChannel` / `ColorChannel`) trong shader WGSL + hàm CPU `chroma_key_cpu`.
   - Thêm `spill_luma` (bù độ sáng khi giảm kênh xanh theo `l0 - l1`) và `desaturate` có kiểm soát để giữ màu da/tóc tự nhiên.
   - Hỗ trợ **`choke` & `soften` trên GPU** bằng chuỗi bước trong `fx::steps`:
     - Pass 1: tính matte thô -> lưu/tạo alpha plane hoặc áp dụng pass blur separable có sẵn (`OP_GAUSS_H` / `OP_GAUSS_V` hoặc pass choke chuyên dụng chỉ lọc kênh alpha) rồi mới áp composite + spill ở pass cuối.
   - Mở khóa `FxOp::eval` cho FilmCraft `"ultra_key"` trong [gpufx.rs](file:///mnt/Data/darkworks/crates/render/src/gpufx.rs#L476-L499) để cả `Setting = Relaxed / Aggressive` và `Soften / Choke > 0` đều chạy GPU.
2. **Xử lý `Garbage Matte` trong [node_graph.rs](file:///mnt/Data/darkworks/crates/render/src/node_graph.rs) và `gpu_plan.rs`:**
   - Nếu pin `Mask` không nối ➔ đi 100% GPU `LayerFx`. Nếu có nối `Mask` ➔ kết hợp mask trên CPU (hoặc biên dịch sang GPU nếu là mask tĩnh).

### Bước 3 — Đưa `Add as FX Clip` & Graph 4 Node `V1 + Blend` lên GPU (`L2: crates/render`)
1. **`Add as FX Clip` (`AdjustmentLayer` có Node Graph):**
   - Trong [plan.rs](file:///mnt/Data/darkworks/crates/render/src/plan.rs#L331-L405) & [gpu_plan.rs](file:///mnt/Data/darkworks/crates/render/src/nodes/gpu_plan.rs): nếu `AdjustmentLayer` mang đồ thị tuyến tính (`VideoInput("Auto" | "All Below") -> [ColorGrade/BlurSharpen/DistortLens/Keyer/CropMask...] -> Output`), cho `layered_at` trả `true` và biên dịch thành `PlanStep::Adjust { ops }` chạy trực tiếp trên GPU.
2. **Mẫu 4 node `VideoInput("Auto") -> ... -> Blend(Bg = VideoInput("All Below" | "V1"), Mask = None) -> Output`:**
   - Khi clip không phải track đáy và `VideoInput` ở pin Background của `Blend` trỏ đúng xuống nền dưới (và `Mask` không nối), biên dịch thẳng thành `LayerFx + PlanLayer.blend + PlanLayer.opacity` trên GPU thay vì rơi về CPU `render_sequence`.

### Bước 4 — Giao diện Node UI & Eyedropper trên Program Monitor (`L4: crates/node-graph`, `L5: crates/ui-egui`)
1. **UI chọn lọc theo Mode trong [node_controls_fx.rs](file:///mnt/Data/darkworks/crates/node-graph/src/node_controls_fx.rs):**
   - Dropdown `Mode` (`Chroma Key` / `Luma Key`), `Output` (`Composite` / `Alpha Channel` / `Color Channel`).
   - Ẩn các ô Chroma/Spill khi chọn `Luma Key`; ẩn `Threshold`/`Cutoff` khi chọn `Chroma Key`.
   - Nhóm gọn `Matte Cleanup` (`Choke`, `Soften`, `Contrast`) và `Spill` (`Spill`, `Desaturate`, `Luma Boost`) trong `CollapsingHeader` để node không bị quá cao trên canvas.
2. **Eyedropper (Ống hút màu từ Program Monitor):**
   - Khảo sát cơ chế armed eyedropper sẵn có của `effect_controls.rs` / `monitor.rs` (dùng cho FilmCraft `ultra_key`), mở rộng một hook chọn màu cho Node Graph Keyer để khi bấm nút hút màu trên node `Ultra Keyer`, click chuột lên Program Monitor sẽ cập nhật thẳng `key_color` vào clip đang mở (có Undo).

---

## 3. TIÊU CHÍ NGHIỆM THU & KIỂM TOÁN CUỐI PHIÊN SAU

- [ ] **Backward compatibility:** Đồ thị cũ chỉ có 4 field (`key_color`, `similarity`, `smoothness`, `spill_suppress`) deserialize thành công 100% không lỗi.
- [ ] **GPU Parity (`filmcraft-gpu`):** Toàn bộ test `fx_tests::gpu_effects_match_cpu_exactly` đạt `flips == 0%` và `max rel diff` trong ngưỡng chuẩn cho cả `ChromaKey` (`choke`, `soften`, `AlphaChannel`, `spill_luma`, `Aggressive` preset) lẫn `LumaKey`.
- [ ] **GPU Plan Coverage (`filmcraft-render`):**
  - `VideoInput("Auto") -> Keyer -> Output` (`Apply to Clip`) ➔ `FramePlan::Layers` (`LayerFx`).
  - `VideoInput("All Below") -> Keyer -> Output` (`Add as FX Clip`) ➔ `FramePlan::Composite` với `PlanStep::Adjust`.
  - `VideoInput("Auto") -> Keyer -> Blend(VideoInput("All Below")) -> Output` ➔ `FramePlan::Layers`.
- [ ] **Chất lượng hình ảnh thực tế:** Tách clip `318703.mp4` (ông già Noel) mượt 60fps ở cả `Apply to Clip` và `Ultra Key` của FilmCraft, soi `Alpha Channel` sạch nền, không còn viền xanh / xám viền da.
- [ ] **Kiểm toán Workspace:**
  - `cargo test -p darkworks-node-graph-core -p darkworks-node-graph -p filmcraft-render -p filmcraft-gpu`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo xtask layers`
  - `python3 .agents/harness/audit.py` (`0` vi phạm `>= 700` dòng, đặc biệt để ý `crates/node-graph-core/src/node.rs`).
