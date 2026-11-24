# SỔ BÀN GIAO TIẾN ĐỘ CA KÍP: DARKWORKS (checkpoint.md)
@project: DARKWORKS | @phòng_ban: Toàn xưởng | @status: 🟢 Hoàn Thành 100% Bộ Công Cụ Visual Group UX & Sẵn Sàng Giai Đoạn 4
@session_end: 2026-11-24 | @next_session_focus: Khởi động Giai đoạn 4: Canvas 2D Generator Node (crates/canvas-renderer: Vello + Cosmic-text)

> Sổ ghi chép bàn giao công việc giữa các phiên làm việc của AI.  
> Khi mở phiên mới (/start_session), nhân viên đọc file này để nắm tiến độ dở dang trong 3 giây.

---

## 1. THÔNG TIN MÔI TRƯỜNG & TRẠNG THÁI HIỆN TẠI
- **Môi trường:** Linux x86_64, Rust 1.85 (Edition 2024), Vulkan / Linux VA-API (`/dev/dri/renderD128`).
- **Tình trạng:** **ĐÃ HOÀN THÀNH 100% TOÀN BỘ CÁC TÍNH NĂNG VISUAL GROUP UX & KHẮC PHỤC TRIỆT ĐỂ LỖI KÉO TÁCH NHÓM & RESIZE**:
  - **Đồng Bộ Phím Xóa Dùng Chung (`Delete` / `Backspace` & Toolbar Button):**
    - Kết nối tiêu điểm tự động `app.ui.focused = PanelKind::NodeGraph` khi click, rê chuột hoặc quét chọn trên canvas Node Graph.
    - Phân quyền điều phối trong `keys.rs`: khi bấm `Delete` hoặc `Backspace` (`edit.clear`), nếu có Node hoặc Group đang được chọn, hệ thống ưu tiên xóa ngay lập tức cho Node Graph mà không để rơi xuống Timeline.
    - Bổ sung nút bấm trực quan `🗑 Delete` trên thanh Toolbar Node Graph.
  - **Tích Hợp Ngăn Xếp Lịch Sử Hoàn Tác / Làm Lại (`Undo` / `Redo`):**
    - Trang bị cấu trúc `GraphHistory` & `GraphSnapshot` trong [crates/node-graph/src/history.rs](file:///mnt/Data/darkworks/crates/node-graph/src/history.rs) lưu tối đa 50 bước.
    - Tự động chụp snapshot trước mọi thao tác sửa đổi đồ thị (`Delete`, `node_to_remove`, `Ctrl + G`, `Ctrl + Shift + G`, `Clear All`, `Load Preset`).
    - Kết nối phím tắt `Ctrl + Z` (Undo) và `Ctrl + Y` / `Ctrl + Shift + Z` (Redo) trong `keys.rs` và `node_graph.rs`.
    - Bổ sung 2 nút bấm trực quan `↶ Undo` và `↷ Redo` trên thanh công cụ Node Graph.
  - **Khắc Phục Triệt Để Lỗi Dính Chùm Khi Kéo Hai Nhóm (Strict Containment):**
    - Thay thế điều kiện chứa tâm lỏng lẻo bằng cơ chế bao hàm nghiêm ngặt 100% (`contains_group`: cả `min` và `max` phải nằm trong `old_rect`).
    - Triệt tiêu hoàn toàn lỗi khóa cứng hai chiều (Mutual Lock); hai nhóm giao cắt (`nhóm 1` và `nhóm 2`) giờ đây hoàn toàn độc lập, kéo nhóm nào chỉ nhóm đó di chuyển, dễ dàng kéo tách xa nhau.
  - **Nâng Cấp Tay Cầm Co Giãn (Resize Grip 32x32px & 3 Rãnh Chéo):**
    - Mở rộng vùng tương tác `resize_rect` từ `20px` lên `32px` (tăng 2.5 lần diện tích bắt chuột), vẽ 3 rãnh chéo trắng đục sắc nét, bắt chuột cực nhạy ở mọi tỷ lệ zoom canvas.
  - **Khắc Phục Triệt Để Lỗi 01 (Nested Visual Groups Interaction & Translation):**
    - Nhận diện và sửa triệt để nguyên nhân group con nằm lồng trong group cha không tương tác được (bị vùng chết ảo 60px của node đè lên header và thứ tự duyệt cướp quyền).
    - Giới hạn vùng hover node chỉ tính thanh tiêu đề `over_node_header` (~26px), đảo ngược thứ tự duyệt canvas `.rev()` để group con có Z-order cao hơn.
    - Kéo group cha tự động tịnh tiến đồng bộ toàn bộ group con và node bên trong.
  - **Inline Rename trên Header Bar (`F2` hoặc Nhấp đúp):**
    - Nhấp đúp chuột trái trực tiếp lên thanh Header của Group hoặc nhấn phím tắt `F2` (khi rê chuột qua Group) để bật ô nhập liệu `TextEdit` đổi tên trực tiếp.
    - Nhấn `Enter` hoặc click ra ngoài lưu tên mới; nhấn `Escape` hủy thao tác.
  - **Menu Ngữ Cảnh Chuột Phải Đầy Đủ (Context Menu):**
    - Nhấp chuột phải trên thanh Header Bar mở menu: `✏️ Đổi tên (Inline Rename)`, `🎨 Bảng màu (Color Palette)` chọn 6 màu ComfyUI, `🎯 Căn vừa Node con (Auto-Fit)`, `🔒 Khóa / Mở khóa vị trí (Lock / Unlock)`, `🔓 Rã nhóm (Ungroup - Giữ Node)`, `🗑 Xóa toàn bộ (Group & Nodes)`.
  - **Phím Tắt Rã Nhóm Nhanh `Ctrl + Shift + G` (Ungroup):**
    - Hủy khung nhóm Visual Group nhưng **giữ nguyên 100% tất cả Node và dây nối bên trong**.
    - Tự động nhận diện group cần rã theo thứ tự ưu tiên: group chứa node đang chọn ➔ group vừa rê chuột ➔ group cuối cùng.
  - **Căn Vừa Tự Động (Auto-Fit to Content):**
    - Nhấp đúp vào tay cầm co giãn góc dưới phải hoặc chọn "🎯 Auto-Fit" trong Menu để khung Group tự động co/dãn ôm khít các Node bên trong kèm khoảng đệm 35px thẩm mỹ và chừa 40px không gian Header.
  - **Chế Độ Khóa Vị Trí (Lock Mode):**
    - Bổ sung trường `is_locked: bool` cho `VisualGroup` (`#[serde(default)]` tương thích ngược 100%).
    - Khi bị khóa, thanh Header hiển thị biểu tượng 🔒, ẩn tay cầm resize và khóa hoàn toàn kéo di chuyển để bảo vệ sơ đồ.
  - **Kiến trúc Tách Phân Hệ Chuẩn Mực (< 700 dòng):**
    - Tách logic tương tác sang [crates/ui-egui/src/panels/node_graph_groups.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/node_graph_groups.rs) (394 dòng) và [crates/ui-egui/src/panels/node_graph_menus.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/node_graph_menus.rs) (247 dòng).
    - Tạo module [crates/node-graph/src/history.rs](file:///mnt/Data/darkworks/crates/node-graph/src/history.rs) (84 dòng) quản lý ngăn xếp Undo/Redo.
    - Giữ [crates/ui-egui/src/panels/node_graph.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/node_graph.rs) (577 dòng) và [crates/ui-egui/src/app/keys.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/app/keys.rs) (526 dòng) luôn đạt chuẩn an toàn thiết quân luật `< 700 dòng`.
  - **Kết quả kiểm toán & kiểm thử:**
    - Toàn bộ 16/16 unit tests của `darkworks-node-graph` đạt **100% PASS** (bao gồm `test_graph_history_undo_redo`, `test_nested_visual_groups_detection_and_translation`, `test_visual_group_lock_and_palette`, `test_visual_group_auto_fit`).
    - Toàn bộ 4/4 kịch bản Feature verification (`python3 .agents/harness/drive_feature.py -f all`): **100% PASS**.
    - Kiểm toán trần dòng code `python3 .agents/harness/check_file_size.py`: **0 vi phạm**.
    - `cargo clippy --workspace --all-targets`: **0 warnings, 0 errors**.
- **Nhánh Git:** `main`.

---

## 2. DANH SÁCH FILE THAY ĐỔI TRONG PHIÊN
- [crates/node-graph/src/history.rs](file:///mnt/Data/darkworks/crates/node-graph/src/history.rs): Khởi tạo `GraphHistory` và `GraphSnapshot` quản lý ngăn xếp Undo/Redo tối đa 50 bước.
- [crates/node-graph/src/group.rs](file:///mnt/Data/darkworks/crates/node-graph/src/group.rs): Bổ sung `is_locked`, bảng màu `PALETTE`, `COLOR_NAMES`, `auto_fit()`, `toggle_lock()`, `set_color()`, và `set_color_index()`.
- [crates/node-graph/src/viewer.rs](file:///mnt/Data/darkworks/crates/node-graph/src/viewer.rs): Render icon 🔒 khi group bị khóa, ẩn tay cầm resize khi bị khóa, bổ sung `#[allow(clippy::too_many_arguments)]`.
- [vendor/egui-snarl/src/ui.rs](file:///mnt/Data/darkworks/vendor/egui-snarl/src/ui.rs): Vô hiệu hóa xóa ngầm không có Undo, export hàm `deselect_all_nodes`.
- [vendor/egui-snarl/src/ui/state.rs](file:///mnt/Data/darkworks/vendor/egui-snarl/src/ui/state.rs): Thêm hàm `deselect_all_nodes` xóa sạch vùng chọn trong bộ nhớ context.
- [crates/ui-egui/src/panels/node_graph.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/node_graph.rs): Khóa cứng `snarl_canvas_id` qua `SnarlWidget::new().id(...)`, consume phím `Ctrl + Delete` / `Delete`, gọi `push_undo()` và `deselect_all_nodes`.
- [crates/ui-egui/src/app/keys.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/app/keys.rs): Điều phối phím tắt xóa dùng chung `edit.clear` (`Delete` / `Backspace`), bắt `Ctrl + Delete` / `Ctrl + Backspace`, và `Ctrl + Z` / `Ctrl + Y` ưu tiên cho Node Graph.
- [crates/node-graph/tests/graph_tests.rs](file:///mnt/Data/darkworks/crates/node-graph/tests/graph_tests.rs): Các unit tests bảo vệ Visual Group (15 tests PASS).
- [crates/node-graph/tests/history_tests.rs](file:///mnt/Data/darkworks/crates/node-graph/tests/history_tests.rs): Unit test kiểm chứng hoàn tác/làm lại `test_graph_history_undo_redo` (PASS).
- [crates/ui-egui/src/panels.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels.rs): Khai báo `pub mod node_graph_groups;` và `pub mod node_graph_menus;`.
- [crates/ui-egui/src/panels/node_graph_menus.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/node_graph_menus.rs): Module riêng quản lý menu Thêm Node và Tải Preset (247 dòng).
- [crates/ui-egui/src/panels/node_graph_groups.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/node_graph_groups.rs): Module xử lý kéo thả, inline rename, context menu, auto-fit, rã nhóm và floating dialog (394 dòng).
- [.darkworks-ai/3-modules/node-graph/visual_group_ux_specs.md](file:///mnt/Data/darkworks/.darkworks-ai/3-modules/node-graph/visual_group_ux_specs.md): Cập nhật trạng thái hoàn thành cho các tính năng P0 & P1.
- [.darkworks-ai/3-modules/node-graph/architecture.md](file:///mnt/Data/darkworks/.darkworks-ai/3-modules/node-graph/architecture.md): Cập nhật kiến trúc Visual Group, phím xóa toàn cục và ngăn xếp Undo/Redo.
- [.darkworks-ai/2-memory/decision-log.md](file:///mnt/Data/darkworks/.darkworks-ai/2-memory/decision-log.md): Ghi sổ Quyết định kiến trúc #32, #33, #34 và #35.
- [.darkworks-ai/1-overview/system_map.md](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/system_map.md): Cập nhật nhật ký Recent Logs phiên 2026-11-24.

---

## 3. CÁC TÍNH NĂNG ĐÃ HOÀN TẤT TRIỆT ĐỂ (RESOLVED FEATURES)
1. **Inline Rename:** Nhấp đúp vào Header Bar hoặc bấm `F2` để sửa tên trực tiếp trên Canvas (Enter lưu, Esc hủy).
2. **Context Menu Chuột Phải:** Chuột phải trên Header Bar có đầy đủ menu đổi tên, bảng màu 6 màu, Auto-Fit, Khóa, Rã nhóm, Xóa.
3. **Phím tắt rã nhóm `Ctrl + Shift + G`:** Hủy khung Group mà không chạm vào Node hay dây nối.
4. **Auto-Fit to Content:** Nhấp đúp tay cầm hoặc chọn Auto-Fit để co dãn khung ôm khít node con.
5. **Chế độ Khóa nhóm (Lock Mode):** Ngăn ngừa kéo lệch và resize ngoài ý muốn.
6. **Đồng bộ Phím Xóa Toàn Cục (`Delete` & `Backspace`):** Tự động bắt tiêu điểm Node Graph, ưu tiên xóa hàng loạt node quét chọn hoặc group đang hover, thêm nút `🗑 Delete` trên toolbar.
7. **Tích hợp Ngăn Xếp Lịch Sử Hoàn Tác / Làm Lại (`Ctrl + Z` / `Ctrl + Y`):** Khôi phục chính xác 100% các node, dây nối và group vừa lỡ xóa; bổ sung 2 nút `↶ Undo` và `↷ Redo` trên thanh toolbar.
8. **Khắc Phục Triệt Để Lỗi Node Không Undo Được Khi Xóa Bằng `Ctrl + Delete` / `Delete`:**
   - Vô hiệu hóa cơ chế xóa ngầm tự phát của `vendor/egui-snarl` để trao 100% quyền kiểm soát xóa cho Darkworks kèm chụp snapshot Undo đầy đủ.
   - Chuẩn hóa `snarl_canvas_id` cố định nhất quán với `SnarlWidget::new().id(...)`, giúp truy vấn chính xác danh sách node chọn.
   - Bắt trọn cả `Delete`, `Backspace`, `Ctrl + Delete`, `Ctrl + Backspace` và tiêu thụ key (`consume_key`) tránh xung đột.
   - Gọi `egui_snarl::ui::deselect_all_nodes` dọn sạch vùng chọn sau khi xóa. Node xóa xong, nút `↶ Undo` lập tức sáng đèn và nhấn `Ctrl + Z` phục hồi nguyên trạng 100%.

---

## 4. KẾ HOẠCH HÀNH ĐỘNG CA TIẾP THEO (NEXT ACTIONS CHECKLIST)
Khi mở phiên tiếp theo (`/start_session`):
- [ ] **Khởi động & Kiểm tra trải nghiệm thực tế với Người Dùng:**
  - Kiểm tra thao tác phím `Delete`, `Ctrl + Z` (Undo) và `Ctrl + Y` (Redo) trên giao diện người dùng.
- [ ] **Giai đoạn 4: Tích hợp Canvas 2D Generator Node (`crates/canvas-renderer`):**
  - Khởi tạo crate `crates/canvas-renderer` tích hợp backend **Vello + Cosmic-text**.
  - Triển khai hàm render vector 2D và text glyphs trực tiếp lên `wgpu::Texture` bằng Compute Shader.
  - Tích hợp Text Node và Shape Node trong Node Graph kết nối trực tiếp với Canvas 2D Renderer.
  - Hỗ trợ đổi màu chữ Karaoke nhảy màu theo tick thời gian thực.
