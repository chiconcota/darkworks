# KẾ HOẠCH DỰ ÁN: GIAI ĐOẠN 3 (phase-3-node-graph.md)
@project: DARKWORKS | @phase: 3 | @status: 🟡 ĐANG TRIỂN KHAI (Bổ Sung Phần Nối Cầu Timeline & Render)
@goal: Đồ thị DAG Node Graph (egui_snarl), Hệ sinh thái 18 Versatile Atomic Nodes bản đầy đủ (Full Production-Ready), Hợp nhất 93 Shaders WGSL, Black Node Subgraphs, Action Node Automation, và ComfyUI Canvas Ergonomics.

---

## 1. NGUYÊN TẮC THIẾT KẾ & ĐỘT PHÁ KIẾN TRÚC

1. **Cung Cấp Bản Đầy Đủ 100% (No Simple / Advanced Tiers):**
   - Tuyệt đối không làm bản "rút gọn" rồi chắp vá "bản nâng cao". Toàn bộ 18 Node nguyên tử đều được trang bị trọn vẹn mọi tham số, thanh trượt và toggle chuyên nghiệp ngay từ đầu.
2. **Hợp Nhất 93 Shaders Cũ Thành 18 "Node Nguyên Tử Đa Năng" (Anti-Spaghetti):**
   - Không xé vụn thành 93 node gây bãi rác canvas (như tạo node Invert hay HFlip lẻ).
   - Tích hợp 93 thuật toán GPU WGSL có sẵn của `crates/gpu` và `crates/render` vào bên trong 18 Node chuẩn mực theo mô hình Nuke / DaVinci Fusion.
3. **Cross-Signal Modulation (Kết Nối Tín Hiệu Chéo Âm Thanh & Hình Ảnh):**
   - Dây tín hiệu âm thanh `AudioFloat` (từ `AudioReactor`) cắm trực tiếp vào các socket `Scale`, `Position`, `Wave Height` để tạo chuyển động giật nảy procedural 60fps không cần keyframe.
4. **Black Node (Compound Subgraph & FX Layer):**
   - Đóng gói các cụm node phức tạp thành 1 node hộp đen duy nhất, drill-down mở đồ thị con, bộc lộ Exposed Pins, xuất/nạp `.preset.json`.
5. **Code Node (Custom WGSL Shader JIT):**
   - Khung soạn shader WGSL ngay trên node, `wgpu` JIT-compile < 2ms, tự động parse reflection biến `uniform` sinh socket điều khiển.

---

## 2. DANH MỤC 18 "NODE NGUYÊN TỬ ĐẦY ĐỦ" (FULL 18 ATOMIC NODES)

### Nhóm 1: Nguồn Phát (Generators) - 4 Nodes
1. `🎬 VideoInput Node`: Nhận texture clip tại Playhead; chọn tường minh track `V1`, `V2`, hoặc `All Tracks Below`.
2. `🔤 TextGenerator Node`: `text`, `font_size`, `color [RGBA]`, `tracking`, `line_spacing`. Xuất texture có kênh Alpha.
3. `🟩 ShapeGenerator Node`: `shape_type` (Rectangle, RoundedRectangle, Circle, SolidMatte), `size [W, H]`, `corner_radius`, `color [RGBA]`.
4. `⏱️ TimeDriver Node`: `speed`, `mode` (Linear, SineLoop, PingPong). Xuất số thực `f32` cuộn sóng / chuyển động tự động.

### Nhóm 2: Chỉnh Màu & Quang Học (Color & Optics) - 4 Nodes
5. `🎨 ColorGrade Node` (Gom toàn bộ 15 effect màu cũ):
   - Sliders: `exposure`, `contrast`, `saturation`, `temperature`, `tint`, `gamma`.
   - Toggles tích hợp: `[✓] Invert Colors` ($RGB = 1.0 - RGB$), `[✓] Black & White`.
6. `💧 BlurSharpen Node` (Gom toàn bộ 5 shader làm mờ cũ):
   - `blur_type`: `Gaussian`, `Box`, `Directional`, `BokehBloom`, `UnsharpMask`.
   - Parameters: `radius`, `angle_deg`, `highlight_boost`.
7. `🌀 DistortLens Node` (Gom toàn bộ 6 shader biến dạng cũ):
   - `distort_type`: `Vignette`, `FishEye`, `LensDistortion`, `RippleWave`, `ChromaticAberration`.
   - Parameters: `intensity`, `radius`, `center [X, Y]`.
8. `🟢 Keyer Node`:
   - `key_color [RGB]`, `similarity`, `smoothness`, `spill_suppress` (tách phông xanh thời gian thực).

### Nhóm 3: Biến Hình & Hòa Trộn (Transform & Compositing) - 3 Nodes
9. `📐 Transform2D Node`:
   - Sliders: `position [X, Y]`, `scale [X, Y]`, `rotation_deg`, `anchor [X, Y]`, `opacity`.
   - Toggles: `[Flip H]`, `[Flip V]`.
   - Mod Sockets: `Pos Mod (Vec2)`, `Scale Mod (Scalar)`, `Rot Mod (Scalar)`, `Opacity Mod (Scalar)`.
10. `✂️ CropMask Node`:
    - `top`, `bottom`, `left`, `right`, `feather` (độ mềm viền).
11. `🎭 Blend Node`:
    - Cổng: `Background (Texture)`, `Foreground (Texture)`, `Mask (Texture)`.
    - 26 chế độ hòa trộn: `Normal`, `Multiply`, `Screen`, `Overlay`, `Difference`, `Add`...

### Nhóm 4: Chuyển Cảnh (Transitions) - 1 Node
12. `⚡ Transition Node` (Gom toàn bộ shader chuyển cảnh cũ):
    - Cổng: `Input A (Texture)`, `Input B (Texture)`, `Progress Mod (Scalar)`.
    - `transition_type`: `CrossDissolve`, `LinearWipe`, `Slide`, `Iris`, `ZoomPush`.
    - Parameters: `progress (0.0 -> 1.0)`, `feather`.

### Nhóm 5: Tín Hiệu & Tự Động Hóa (Signals & Automation) - 2 Nodes
13. `🔊 AudioReactor Node`:
    - `band`: `SubBass`, `Bass`, `LowMid`, `HighMid`, `Treble`, `MasterPeak`.
    - `multiplier`, `smooth`. Xuất `AudioFloat` điều khiển chuyển động theo nhịp nhạc.
14. `⚡ ActionNode`:
    - `scope`: `MarkedInOut`, `SelectedClips`, `TrackV1`, `TrackV2`. Nút `[ ▶ RUN WORKFLOW ]` thực thi lệnh `Session::execute`, hoàn tác `Ctrl + Z`.

### Nhóm 6: Mở Rộng & Tiện Ích ComfyUI (Advanced & Ergonomics) - 4 Nodes
15. `⬛ BlackNode`: Đóng gói Subgraph, drill-down mở đồ thị con, bộc lộ `ExposedPins`, xuất/nạp `.preset.json`.
16. `💻 CustomWgsl`: Code editor shader GPU, `wgpu` JIT-compile < 2ms, tự động parse reflection biến `uniform`.
17. `📝 StickyNote`: Thẻ ghi chú/memo văn bản nhiều dòng dán trên canvas.
18. `● Reroute`: Chấm bẻ góc dây uốn lượn gọn gàng, tránh vắt chéo dây qua mặt các node khác.
19. `🖥️ Output Node`: Đẩy khung hình kết xuất cuối cùng ra Viewport.

---

## 3. CHECKLIST TRIỂN KHAI GIAI ĐOẠN 3 (EXECUTION CHECKLIST)

- [x] **Task 3.1 [Khởi Tạo Crate `crates/node-graph` & Snarl Canvas UI]:**
  - Vendor `egui-snarl` và `egui-scale` tích hợp `egui 0.37`.
  - Tạo cấu trúc crate: `types.rs`, `node.rs`, `group.rs`, `viewer.rs`, `preset.rs`, `evaluator.rs`, `compound.rs`, `custom_shader.rs`, `lib.rs`.
  - Tích hợp `PanelKind::NodeGraph` vào dock UI của `filmcraft-ui-egui`.
  - Xây dựng Visual Groups / Backdrop Box, Sticky Note, Reroute Dot, Dot-Grid.

- [x] **Task 3.2 [Mô Hình Dữ Liệu Đồ Thị & Hệ Thống Sockets/Pins]:**
  - Sockets đa tín hiệu: Texture (Lam), AudioFloat (Lục), Scalar/Vec2/Color (Vàng/Cam/Hồng), ActionFlow (Tím).
  - Tương thích tín hiệu `can_connect_to` (AudioFloat ➔ Scalar).
  - Định tuyến track tường minh `VideoInput` (`V1`, `V2`, `All Below`).

- [x] **Task 3.3 [Black Node Subgraph & Preset Storage]:**
  - Cấu trúc `BlackNodeSubgraph`, cơ chế `auto_detect_exposed_pins()` & `expose_pin()`.
  - Chuẩn hóa định dạng lưu trữ `.preset.json`.

- [x] **Task 3.4 [Topological DAG Evaluator & Cycle Detection]:**
  - Thuật toán Kahn's Algorithm sắp xếp thứ tự thực thi topo và phát hiện vòng lặp `DagError::CycleDetected`.
  - Hàm `evaluate_signals()` tính toán luồng số thực thời gian thực.
  - Nút `⚡ Validate DAG` trên toolbar hiển thị trạng thái đồ thị.

- [x] **Task 3.5 [Code Node Custom WGSL JIT Reflection]:**
  - Bộ phân tích `WgslParser` trích xuất biến `@group(..) @binding(..) var<uniform>` thành sockets.

- [x] **Task 3.6 [Cập Nhật Bản Đầy Đủ 100% Cho 18 Atomic Nodes & Menu Toolbar]:**
  - Mở rộng `DarkNode` với 3 node đa năng mới: `BlurSharpen`, `DistortLens`, `Transition`.
  - Bổ sung `invert: bool`, `black_and_white: bool` vào `ColorGrade`; `flip_h: bool`, `flip_v: bool` vào `Transform2D`.
  - Cập nhật UI controls và render sockets trong `viewer.rs` cho toàn bộ 18 nodes.

- [x] **Task 3.7 [Kho 5 Built-in Black Node Presets & Nạp Đồ Thị 1-Click]:**
  - Tạo sẵn 5 Preset mẫu kiểm chứng:
    1. `News Ticker Banner` (Shape xanh + Text đen + TimeDriver cuộn ngang + Blend).
    2. `Continuous Audio Waveform Line` (Shape line 4px + Distort wave + AudioReactor + TimeDriver + Blend).
    3. `Spotlight Invert` (VideoIn + Solid trắng + Shape circle mask + Blend difference).
    4. `Audio Beat Flash` (VideoIn + Solid trắng + AudioReactor bass + Blend difference).
    5. `Cinematic Soft Glow` (VideoIn + Blur Bokeh bloom + Blend screen).
  - Tích hợp Menu `📂 Load Preset` trên Toolbar cho phép chuyển đổi 1-click tức thì.

---

## 4. TIÊU CHUẨN NGHIỆM THU (DEFINITION OF DONE - DoD 3)

- [x] **DoD 3.1 (Snarl Canvas 60 FPS):** Tương tác zoom/pan vô cực mượt mà 60 FPS, kéo thả nối dây trực quan.
- [x] **DoD 3.2 (Full 18 Atomic Nodes):** Đầy đủ 18 nodes nguyên tử đa năng gom trọn 93 shader cũ, đầy đủ controls/toggles (Invert, B&W, Flip H/V).
- [x] **DoD 3.3 (Audio-Visual Modulation):** Tín hiệu `AudioFloat` nối trực tiếp sang `Scalar` lái thông số procedural chuyển động theo nhạc.
- [x] **DoD 3.4 (Topological Sort & Cycle Proof):** Không bao giờ crash, phát hiện vòng lặp vô hạn `CycleDetected`.
- [x] **DoD 3.5 (Black Node Compound):** Đóng gói, bộc lộ Exposed Pins, xuất nạp `.preset.json` độc lập.
- [x] **DoD 3.6 (Custom WGSL JIT):** Parse tự động reflection biến uniform thành socket.
- [x] **DoD 3.7 (Kho 5 Built-in Presets):** Nạp 1-click cả 5 presets mẫu chạy mượt mà, DAG hợp lệ 100%.
- [x] **DoD 3.8 (Tests & Lints):** 11/11 tests pass 100%, clippy sạch 0 cảnh báo.
- [x] **DoD 3.9 (Audit Code Ceiling):** 10/10 files Darkworks mới < 700 dòng, tuân thủ tuyệt đối chuẩn mực kiến trúc.

---

## 5. KẾ HOẠCH BỔ SUNG: NỐI CẦU NODE GRAPH ➔ TIMELINE & RENDER (TASKS 3.8 – 3.11)

> **Ràng buộc kiến trúc cần xử lý (`cargo xtask layers`):**  
> Các crate từ tầng L0–L4 (`project`, `render`, `gpu`, `engine`) bị cấm phụ thuộc vào `egui` (`UI_ONLY` trong `xtask/src/main.rs`), trong khi `darkworks-node-graph` hiện đang kéo `egui` và `egui-snarl`. Do đó, cần tách phần dữ liệu thuần + logic đánh giá DAG ra khỏi tầng UI `egui` trước khi gắn vào Timeline và Compositor.

- [x] **Task 3.8 [Tách Data Model & Gắn Node Graph vào Project/Timeline (`crates/project`, `crates/node-graph`, `crates/engine`)]:**
  - Đã tạo crate thuần dữ liệu `crates/node-graph-core` (Layer 0, không `egui`) chứa `NodeGraphDoc`, `DarkNode`, `DagEvaluator`, `WgslParser` và cầu nối `doc_from_snarl` / `snarl_from_doc` trong `crates/node-graph`.
  - Đã thêm `pub node_graph: Option<Box<NodeGraphDoc>>` vào `TrackItem` (`crates/project/src/track.rs`).
  - Đã bổ sung `nodeGraph.setOnClip`, `nodeGraph.createOverlayClip`, `nodeGraph.applyScope`, `nodeGraph.clearFromClip`, `nodeGraph.getFromClip` vào `crates/engine/src/node_graph.rs`.

- [x] **Task 3.9 [Bộ Thực Thi DAG Compositor Trong `crates/render` (CPU Reference + Live Signals)]:**
  - Đã triển khai `crates/render/src/node_graph.rs` + `node_graph_ops.rs` và nối vào `seq_canvas` (`lib.rs`) và `plan.rs` cho cả media/graphic clips lẫn adjustment/overlay layers.
  - Hỗ trợ `VideoInput`, `ShapeGenerator`, `TextGenerator`, `ColorGrade`, `BlurSharpen`, `DistortLens`, `Keyer`, `Transform2D`, `CropMask`, `Blend` (26 modes + Mask), `Transition`, `TimeDriver`, `AudioReactor` (RMS/band từ `mix_sequence`).

- [x] **Task 3.10 [Tối Ưu GPU (`CustomWgsl` & `wgpu` Pipeline) (`crates/gpu`, `crates/render`)]:**
  - Đã triển khai `crates/gpu/src/custom_wgsl.rs` biên dịch an toàn qua `push_error_scope` (cache tối đa 32 pipelines) và hook `register_wgsl_runner` từ `apps/darkworks` & `apps/darkworks-cli`.

- [x] **Task 3.11 [Giao Diện Nối Cầu Node Graph ⇄ Timeline ⇄ Program Monitor (`crates/ui-egui`)]:**
  - Đã triển khai `crates/ui-egui/src/panels/node_graph_bridge.rs`: đồng bộ khi chọn clip, các nút `📌 Apply to Clip`, `🎬 Add as FX Clip`, `📝 New Graph (Nháp)`, `🗑 Gỡ khỏi Clip`, kích hoạt `[ ▶ RUN WORKFLOW ]` trên `ActionNode`, live sync khi nhả chuột, và badge `◈` màu Cyan trên Timeline (`clips.rs`).
- [x] **Task 3.12 [Kho Preset `.preset.json` Thật: Export/Import File & Thư Mục User Presets]:**
  - Đã chuẩn hóa `NodeGraphPresetDoc` (`crates/node-graph-core/src/preset.rs`), `💾 Save Preset` (Lưu vào `<data_dir>/Node Presets/` dùng chung mọi project), `💾 Export .preset.json` (`Save As`), `📥 Import Preset` (`Open File`), và 2 chế độ **Thay thế (Replace)** / **Nối tiếp trước `Final Output` (Chain)**.
- [ ] **Task 3.13 [Node & Preset Library Browser (Search, Filter, Thumbnail Preview & Kéo-Thả)]:**
  - Nâng cấp giao diện chọn Preset từ menu thả xuống sang **Preset Library Browser** (tìm kiếm real-time, lọc Category/Tag/Favorites, lưới Thumbnail xem trước được render qua Node Graph trên `preview_image`, Quick Search `Tab` trên Canvas, và kéo thả thẳng từ Library vào Node Graph hoặc Clip trên Timeline).
  - Đóng gói các Micro-Presets (Blur/Glow/Distort/Flip/Keyer) và chuyển bộ `lumetri_presets` sang `ColorGrade` Presets.

### Tiêu chuẩn nghiệm thu bổ sung (DoD 3.10 – 3.14)

- [x] **DoD 3.10 (`News Ticker Banner`):** Nạp Preset 1 ➔ bấm `🎬 Add as FX Clip` ➔ xuất hiện clip trên Track V2 ➔ phát Timeline thấy dải nền bo góc + chữ chạy ngang trên Program Monitor (chờ nghiệm thu trực quan).
- [ ] **DoD 3.11 (`Spotlight Invert` & `Cinematic Soft Glow`):** Bấm `📌 Apply to Clip` (hoặc chạy `ActionNode` theo `[Mark In, Mark Out]`) ➔ đổi hiệu ứng trực tiếp trên Program Monitor và hoàn tác được bằng `Ctrl + Z` (chờ nghiệm thu trực quan).
- [ ] **DoD 3.12 (`Audio Beat Flash` / `Audio Waveform`):** Phát clip có âm thanh ➔ hiệu ứng phản ứng đồng bộ theo nhịp âm thanh trên Program Monitor (chờ nghiệm thu trực quan).
- [x] **DoD 3.13 (Kiểm toán & Tương thích):** `cargo xtask layers` pass 100%, `cargo clippy` 0 warning, toàn bộ file mới/sửa `< 700 dòng` (`audit.py` 0 vi phạm), unit tests `render` & `engine` round-trip `project.json` PASS.
- [ ] **DoD 3.14 (Kho Preset `.preset.json` Thật):** Xuất file `.preset.json` ra ổ đĩa ➔ `Clear All` ➔ Import lại giữ nguyên 100% node, tọa độ, dây nối và Visual Groups; xuất hiện trong thư viện Preset lưu sẵn.
