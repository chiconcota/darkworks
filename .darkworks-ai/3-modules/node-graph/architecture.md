# MODULE KIẾN TRÚC: NODE GRAPH & GPU COMPOSITOR (node-graph/architecture.md)
@crate: `crates/node-graph-core` (L0) + `crates/node-graph` (L5) + `crates/render/src/node_graph.rs` (L2) + `crates/gpu/src/custom_wgsl.rs` (L3)
@engine: egui_snarl + wgpu + crates/text | @status: 🟡 Hoàn Thành Core DAG + Timeline/Render Bridge (Còn Task 3.12 Kho Preset `.preset.json` Thật)

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
  1. `VideoInput`: Lấy texture khung hình video gốc từ clip tại Playhead (chọn nguồn Track V1, V2 hoặc All Below).
  2. `TextGenerator`: Sinh chữ động, xuất texture RGBA kèm kênh Alpha trong suốt (`font_size`, `tracking`, `line_spacing`).
  3. `ShapeGenerator`: Vẽ hình khối chữ nhật bo góc, hình tròn, dải màu Solid Matte làm nền cho banner, lower-third, mask.
  4. `TimeDriver`: Bộ đếm thời gian, tốc độ cuộn ngang Ticker/Marquee, sóng dao động LFO (Linear, SineLoop, PingPong).
  5. `ColorGrade`: Chỉnh sáng, độ tương phản, nhiệt độ màu, độ bão hòa, tint, gamma, Color Tint (`tint_color` Color Picker + `tint_amount`), và 2 toggle chuyên dụng `[✓] Invert Colors`, `[✓] Black & White`.
  6. `BlurSharpen`: Gom 5 shader làm mờ (Gaussian, Box, Directional, Bokeh Bloom, Unsharp Mask) kèm `radius`, `angle_deg`, `highlight_boost`.
  7. `DistortLens`: Gom 6 shader biến dạng (Vignette, FishEye, Lens Distortion, Ripple Wave, Chromatic Aberration) kèm `intensity`, `radius`, `center`.
  8. `Keyer`: Tách phông xanh Ultra Keyer thời gian thực kèm `key_color` (Color Picker) và `spill_suppress`.
  9. `Transform2D`: Zoom, Pan, Rotate, Anchor, Opacity, và 2 toggle `[Flip H]`, `[Flip V]`.
  10. `CropMask`: Cắt xén 4 cạnh và làm mềm viền `feather`.
  11. `Blend`: Trộn 2 lớp hình ảnh với đầy đủ 27 chế độ hòa trộn của `filmcraft_render::Blend` (`BlendMode::ALL`, giữ `Add` tương thích ngược) và 3 cổng `Background`, `Foreground`, `Mask`.
  12. `Transition`: Chuyển cảnh 30 hiệu ứng (CrossDissolve, LinearWipe, Slide, Iris, ZoomPush) kèm `progress` và `feather`.
  13. `AudioReactor`: Đọc RMS peak từ audio track đẩy sang socket thông số (Scale/Shake/Glow/Wave).
  14. `ActionNode`: Trạm tự động hóa quy trình, nút `[ ▶ RUN WORKFLOW ]` gọi lệnh `Session::execute`, hoàn tác `Ctrl + Z`.
  15. `BlackNode`: Đóng gói cụm nodes thành 1 hộp đen, bộc lộ `Exposed Pins`, xuất/nạp `.preset.json`.
  16. `CustomWgsl`: Nhúng khung soạn code WGSL, wgpu JIT-compile < 2ms, tự động parse reflection biến `uniform`.
  17. `StickyNote`: Tấm thẻ ghi chú văn bản dán trên canvas.
  18. `Reroute`: Nút chấm tròn bẻ góc dây vuông vắn, tránh vắt chéo dây qua mặt các node khác.
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

- **Tách Crate L0 (`crates/node-graph-core`) & Cầu Nối Compositor (`crates/render` + `crates/gpu`):**
  - `crates/node-graph-core` (Layer 0, không phụ thuộc `egui`) chứa `NodeGraphDoc`, `DarkNode`, `DagEvaluator`, `WgslParser`.
  - `crates/project::TrackItem` lưu `pub node_graph: Option<Box<NodeGraphDoc>>`; `crates/engine/src/node_graph.rs` cung cấp các lệnh `nodeGraph.{setOnClip, createOverlayClip, applyScope, clearFromClip, getFromClip}`.
  - `crates/render/src/node_graph.rs` điều phối `eval_item_graph` tại `Tick` hiện tại và chia nhánh xử lý theo nhóm node vào `crates/render/src/nodes/` (`generators.rs`, `shape.rs`, `text.rs`, `color_blur.rs`, `blur_distort.rs`, `transform_blend.rs`, `signals.rs`).
  - `crates/gpu/src/custom_wgsl.rs` biên dịch `CustomWgsl` fragment shader an toàn qua `push_error_scope` (cache tối đa 32 pipelines) và nối vào render via `register_wgsl_runner`.

- **Topological DAG Evaluator & Cycle Proof:**
  - Thuật toán Kahn's Algorithm sắp xếp thứ tự thực thi và phát hiện vòng lặp `DagError::CycleDetected`.
  - Hàm `evaluate_signals()` tính toán luồng số thực thời gian thực.

## 3. CẤU TRÚC PHÂN TẦNG SOURCE CODE
```text
crates/node-graph-core/    # Layer 0 (không egui): NodeGraphDoc, DarkNode, DagEvaluator, WgslParser
crates/node-graph/         # Layer 5 (egui_snarl UI): doc.rs bridge, node_controls.rs, viewer.rs, preset.rs, history.rs, group.rs
crates/render/src/
├── node_graph.rs          # DAG execution dispatcher (eval_item_graph)
└── nodes/                 # Xử lý từng nhóm node: generators, shape, text, color_blur, blur_distort, transform_blend, signals
crates/gpu/src/custom_wgsl.rs  # wgpu JIT pipeline cho CustomWgsl
crates/engine/src/node_graph.rs # Lệnh Session::execute (nodeGraph.*)
```
