# MODULE KIẾN TRÚC: NODE GRAPH & GPU COMPOSITOR (node-graph/architecture.md)
@crate: `crates/node-graph-core` (L0) + `crates/node-graph` (L5) + `crates/render/src/node_graph.rs` & `nodes/` (L2) + `crates/gpu/src/{node_fx.rs, node_fx.wgsl, custom_wgsl.rs}` (L3)
@engine: egui_snarl + wgpu + crates/text | @status: 🟢 Hoàn Thành `Ultra Keyer` Pro + 14 `NodeFxOp` GPU Compute Shaders + Shared Eyedropper + Linear/`Blend`/`FX Clip` GPU Planner (Phiên sau test toàn diện các Atomic Nodes khác)

## 1. VAI TRÒ & TRÁCH NHIỆM
Đây là trái tim đồ họa phân biệt Darkworks với FilmCraft và Premiere truyền thống. Module này thay thế hoàn toàn danh sách tuyến tính `Vec<EffectInstance>` bằng mạng lưới đồ thị có hướng không chu trình (Directed Acyclic Graph - DAG), hỗ trợ các container đóng gói (Black Node), trạm tự động hóa quy trình (Action Node), công cụ tổ chức canvas ComfyUI Style, và khả năng mở rộng vô hạn qua Code Node (Custom WGSL JIT Shader).

## 2. KIẾN TRÚC & NGUYÊN LÝ HOẠT ĐỘNG
- **Node Graph UI:** Sử dụng `egui_snarl` để render bảng nối dây trực quan (zoom & pan vô cực, kéo dây Bézier, ghim sockets, đổi thông số).
- **Socket & Data Routing:**
  - Socket Hình ảnh (🔵 Xanh lam): truyền `wgpu::Texture` (RGBA8, Zero-copy GPU).
  - Socket Âm thanh (🟢 Xanh lá): truyền giá trị biên độ sóng âm `f32` (RMS Peak / Pitch từ `crates/audio-dsp`).
  - Socket Thông số (🟡 Vàng): truyền `f32`, `Vec2`, `Color` (dùng cho Scale, Position, Opacity, Procedural Motion).
  - Socket Luồng lệnh (🟣 Tím): kích hoạt luồng tự động hóa của Action Node.
  - Tương thích tín hiệu `NodeDataType::AudioFloat` có thể nối vào `NodeDataType::Scalar` để lái chuyển động procedural theo nhạc.

- **Hệ Sinh Thái 18 Versatile Atomic Nodes Bản Đầy Đủ (Gom Trọn 93 Shaders Cũ):**
  1. `VideoInput`: Lấy texture khung hình video từ `Auto (Clip)`, `All Below`, hoặc track tường minh `V1`, `V2`, `V3`, `V4` (hỗ trợ cả FX Clip có giới hạn `MAX_VIDEO_INPUT_DEPTH = 4` chống lặp).
  2. `TextGenerator`: Sinh chữ động có alpha (`font_size`, `font_family` ComboBox từ `filmcraft_text`, `bold`, `italic`, `align`, `position`, `tracking`, `line_spacing`, `stroke_width` + `stroke_color`, và socket `Pos Mod (Vec2)`).
  3. `ShapeGenerator`: Vẽ hình khối chữ nhật bo góc, hình tròn, Solid Matte kèm `position`, `stroke_width` + `stroke_color`, `feather` (làm mềm viền Mask), và socket `Pos Mod (Vec2)`.
  4. `TimeDriver`: Bộ đếm thời gian (`Linear`, `SineLoop`, `PingPong`), hỗ trợ `amplitude`, `offset`, `vec2_axis`, xuất cả `Time Value (Scalar)` lẫn `Vec2 Out (Vec2)`.
  5. `ColorGrade`: Chỉnh sáng/màu cơ bản (`exposure` + socket `Exp Mod`, `contrast`, `saturation`, `temperature`, `tint`, `gamma`, `tint_color`/`tint_amount`), `Invert Colors`, `Black & White`, và bộ Lumetri mở rộng (`highlights`, `shadows`, `whites`, `blacks`, `vibrance`, `faded_film`, `shadow_tint`, `highlight_tint`).
  6. `BlurSharpen`: Gom 6 thuật toán (`Gaussian`, `Box`, `Directional`, `BokehBloom`, `UnsharpMask`, `RadialZoom` — nay chạy GPU via `NodeFxOp::RadialZoom`) kèm `radius` + socket `Radius Mod (Scalar)`, `dimensions` (`Both`/`Horizontal`/`Vertical`), `repeat_edge`, `bokeh_sides` (Polygon shape 0..8), `center [X, Y]`, `angle_deg`, `highlight_boost`, `threshold`.
  7. `DistortLens`: Gom 13 hiệu ứng — cả 9 chế độ trước đây chạy CPU (`Pixelate`, `ChromaticAberration`, `RippleWave`, `FishEye`, `LensDistortion`, `Twirl`, `FindEdges`, `NoiseGrain`, `TurbulentDisplace`) nay chạy 100% GPU via `node_fx.wgsl`, cùng 4 chế độ gốc (`Vignette`, `Mirror`, `Offset`, `Posterize`); UI trong `node_controls_fx.rs` tự động ẩn các ô không liên quan theo từng chế độ.
  8. `Keyer` (`Ultra Keyer` Pro): Hỗ trợ 2 chế độ (`ChromaKey` Y'CbCr Bt.709 3-pass `Matte -> Choke -> Finish` / `LumaKey` Luminance threshold/cutoff), 3 chế độ output (`Composite`, `AlphaChannel`, `ColorChannel`), đầy đủ bộ lọc (`Pedestal`, `Highlight`, `Shadow`, `Choke`, `Soften`, `Matte Contrast`, `Invert Matte`, `Desaturate`, `Spill Luma`), pin `Garbage Matte` (in pin 1) + `Matte Out` (out pin 1), và **Shared Eyedropper** hút màu trực tiếp trên Program Monitor (kèm auto-bypass tức thì).
  9. `Transform2D`: Zoom (`Scale X/Y`), Pan, Rotate, Anchor, Opacity, và 2 toggle `[Flip H]`, `[Flip V]`.
  10. `CropMask`: Cắt xén 4 cạnh và làm mềm viền `feather`.
  11. `Blend`: Trộn 2 lớp hình ảnh với đầy đủ 27 chế độ hòa trộn của `filmcraft_render::Blend` (`BlendMode::ALL`, giữ `Add` tương thích ngược) và 3 cổng `Background`, `Foreground`, `Mask`.
  12. `Transition`: Chuyển cảnh 8 kiểu (`CrossDissolve`, `LinearWipe`, `Slide`, `Iris`, `ZoomPush`, `DipBlack`, `DipWhite`, `BlurDissolve`) kèm `direction` (`Left`/`Right`/`Up`/`Down`), `progress` và `feather`.
  13. `AudioReactor`: Đọc RMS theo dải tần từ `Master Mix`, `Linked Audio` (`item.link`), hoặc track `A1..A4`, xuất `Peak Value (Scalar)` & `Vec2 Out (Vec2)` (điều khiển tuyến tính stops phơi sáng `Exp Mod * 2.5`).
  14. `ActionNode`: Trạm tự động hóa quy trình, nút `[ ▶ RUN WORKFLOW ]` gọi lệnh `Session::execute`, hoàn tác `Ctrl + Z`.
  15. `BlackNode`: Đóng gói cụm nodes thành 1 hộp đen, bộc lộ `Exposed Pins`, xuất/nạp `.preset.json`.
  16. `CustomWgsl`: Nhúng khung soạn code WGSL (kèm mẫu `fn fs_main` mặc định từ menu), wgpu JIT-compile < 2ms, tự động parse reflection biến `uniform`.
  17. `StickyNote`: Tấm thẻ ghi chú văn bản dán trên canvas.
  18. `Reroute`: Nút chấm tròn hỗ trợ đổi kiểu `Any`/`Texture`/`Scalar`/`Vec2`/`AudioFloat` trực tiếp để bẻ góc mọi dây tín hiệu; `DarkworksViewer::connect` kiểm tra hợp lệ qua `NodeDataType::can_connect_to`.
  19. `Output`: Khung hình kết xuất cuối cùng đẩy ra Viewport.

- **Kho 5 Built-in Presets Mẫu Cốt Lõi:**
  1. `News Ticker Banner`: Shape bo góc + Text tiêu đề + TimeDriver cuộn ngang + Blend.
  2. `Continuous Audio Waveform Line`: Shape line 4px + Distort wave + AudioReactor + TimeDriver + Blend Screen (đường sóng nhạc 60 FPS).
  3. `Spotlight Invert`: VideoIn + Solid trắng + Shape circle mask + Blend difference.
  4. `Audio Beat Invert Flash`: VideoIn + Solid trắng + AudioReactor bass + Blend difference.
  5. `Cinematic Soft Glow`: VideoIn + Blur Bokeh bloom + Blend screen.

- **Tổ Chức Canvas ComfyUI Style & Visual Groups (Backdrop Boxes):**
  - Cấu trúc `VisualGroup`: Lưu id, title, min/max tọa độ, mã màu RGBA, và cờ trạng thái `is_locked`.
  - Các hàm tiện ích: `header_rect()` (thanh tiêu đề kéo thả 30px trên cùng), `resize_rect()` (tay cầm góc dưới phải 20x20px), `contains_node()` (kiểm tra node thuộc vùng nhóm), `default_color(idx)` (xoay vòng bảng màu 6 màu ComfyUI dịu mắt), `auto_fit()` (tự động co/dãn khung ôm khít các Node con), `toggle_lock()` (khóa/mở khóa vị trí chống kéo lệch).
  - Phím tắt `Ctrl + G`: Tự động tính Bounding Box bao quanh các Node đang chọn (+35px padding) và tạo Visual Group mới kèm tên mặc định `Group {N}`.
  - Phím tắt `Ctrl + Shift + G`: Rã nhóm nhanh (hủy khung nhóm nhưng giữ nguyên toàn bộ Node và dây nối bên trong).
  - Tương tác kéo thả trên Canvas: Nhấp kéo Header Bar di chuyển đồng bộ cả Group và toàn bộ Node thành viên bên trong; kéo Resize Grip góc dưới phải mở rộng/thu hẹp khung Group mà không dịch chuyển node. Khóa panning nền canvas khi đang kéo group.
  - Inline Rename (`F2` hoặc Nhấp đúp Header): Cho phép gõ đổi tên trực tiếp trên thanh tiêu đề của Group.
  - Context Menu Chuột Phải: Chuột phải vào Header Bar hiện menu: Đổi tên, Bảng màu 6 màu ComfyUI, Auto-Fit, Khóa/Mở khóa vị trí, Rã nhóm (Ungroup), Xóa sạch cụm.
  - Đồng bộ Phím Xóa Toàn Cục (`Delete` / `Backspace`): Tự động gán tiêu điểm `app.ui.focused = PanelKind::NodeGraph` khi tương tác canvas; hàm `delete_selected_or_hovered` và `has_selection` cho phép xóa hàng loạt các node đang chọn hoặc group đang hover, kết nối trực tiếp với bộ điều phối phím tắt `keys.rs`. Bổ sung nút `🗑 Delete` trực quan trên thanh toolbar.
  - Ngăn Xếp Lịch Sử Hoàn Tác / Làm Lại (`Undo` / `Redo`): Cấu trúc `GraphHistory` & `GraphSnapshot` (`crates/node-graph/src/history.rs`) lưu tối đa 50 bước, tự động chụp snapshot trước mọi thao tác sửa đổi (`Delete`, `node_to_remove`, `Ctrl + G`, `Ctrl + Shift + G`, `Clear All`, `Load Preset`); kết nối phím tắt `Ctrl + Z` và `Ctrl + Y` / `Ctrl + Shift + Z`, bổ sung 2 nút `↶ Undo` và `↷ Redo` trên thanh toolbar. Khắc phục triệt để lỗi mất snapshot khi xóa bằng cách vô hiệu hóa cơ chế xóa ngầm tự phát của `vendor/egui-snarl`, khóa cứng `snarl_canvas_id` với `SnarlWidget::new().id(...)` và tiêu thụ sự kiện phím `consume_key` cho `Ctrl + Delete` / `Ctrl + Backspace`.
  - Phân hệ độc lập: `crates/ui-egui/src/panels/node_graph_groups.rs`, `crates/ui-egui/src/panels/node_graph_menus.rs`, và cầu nối Timeline `crates/ui-egui/src/panels/node_graph_bridge.rs` (hỗ trợ `Apply to Clip`, `Add as FX Clip`, tách sang chế độ Nháp `📝 New Graph (Nháp)` chống live-sync ghi đè clip cũ, `🗑 Gỡ khỏi Clip`, `ActionNode` scope execution).
  - Bố cục điều khiển dọc gọn (`max_width(195.0)`) trong `crates/node-graph/src/node_controls.rs` tích hợp `ComboBox` cho `Blend`, `ShapeGenerator`, `AudioReactor`, `ActionNode` và Color Picker trực tiếp trên node.
  - Lộ trình mở rộng chi tiết xem tại `visual_group_ux_specs.md`.

- **Phân Hệ Lưu / Xuất / Nhập `.preset.json` (`crates/node-graph-core/src/preset.rs` & `crates/ui-egui/src/panels/node_graph_presets.rs`):**
  - `NodeGraphPresetDoc`: Lưu metadata (`name`, `category`, `version`, `description`) + trọn vẹn `NodeGraphDoc` (`nodes`, `pos`, `custom_title`, `wires`, `groups`), giữ tương thích ngược preset cũ.
  - Thư mục User Presets dùng chung mọi project tại `<data_dir>/Node Presets/`.
  - Hỗ trợ `chain_node_graphs` cho 2 chế độ áp Preset: **Thay thế (Replace)** hoặc **Nối tiếp trước `Final Output` (Chain)**.
- **Bố Cục UI & Render Nodes (`node_controls.rs`, `crates/render/src/`):**
  - `render_node_body` dùng `ui.vertical` (`set_max_width(210.0)`) khắc phục triệt để xung đột `Layout::left_to_right` của `egui_snarl`.
  - FX Clip (`AdjustmentLayer`) trong `crates/render/src/lib.rs` phân biệt đồ thị có `VideoInput` (`canvas.set(adj)`) và đồ thị overlay không có `VideoInput` (`blend::composite` đè lên `below`).
  - `crates/render/src/nodes/signals.rs` xử lý `resolve_video_input`, `compute_audio_rms_map` và các hàm `modulated_*`.

- **Topological DAG Evaluator & Cycle Proof:**
  - Thuật toán Kahn's Algorithm sắp xếp thứ tự thực thi và phát hiện vòng lặp `DagError::CycleDetected`.
  - Hàm `evaluate_signals()` tính toán luồng số thực thời gian thực.

## 3. CẤU TRÚC PHÂN TẦNG SOURCE CODE
```text
crates/node-graph-core/    # Layer 0 (không egui): node.rs (532d), node_fx_types.rs (279d), preset.rs, graph.rs, wgsl_parser.rs
crates/node-graph/         # Layer 5 (egui_snarl UI): doc.rs, node_controls.rs, node_controls_fx.rs (405d), viewer.rs, preset.rs, history.rs, group.rs
crates/render/src/
├── node_graph.rs          # DAG execution dispatcher (đường đa nhánh / Garbage Matte / Matte Out)
├── node_graph_tests.rs    # Unit tests cho eval_item_graph
└── nodes/
    ├── gpu_plan.rs        # Graph planner/scheduler (369d): Apply to Clip (Auto/Clip/track của clip), Blend(V1/All Below), AdjustmentLayer
    ├── gpu_plan_tests.rs  # Unit tests cho GPU planner (392d)
    ├── node_fx_ops.rs     # 14 NodeFxOp + CPU fallback thống nhất (536d)
    ├── node_fx_distort_cpu.rs # CPU parity helpers cho hiệu ứng biến dạng (279d)
    ├── color.rs           # ColorGrade & Keyer (CPU + push_*_gpu_op, 383d)
    ├── blur_distort.rs    # BlurSharpen & DistortLens (CPU + push_*_gpu_ops, 450d)
    ├── transform_blend.rs # Transform2D, CropMask, Blend, Transition (CPU + push_*_gpu_ops, 290d)
    └── shape.rs / text.rs / signals.rs
crates/gpu/src/            # node_fx.rs (190d) + node_fx.wgsl (503d), fx.rs + fx.wgsl, custom_wgsl.rs
crates/engine/src/node_graph.rs # Lệnh Session::execute (nodeGraph.*)
crates/ui-egui/src/panels/ # node_graph*, monitor.rs (Shared Eyedropper)
```
