# MODULE KIẾN TRÚC: USER INTERFACE (ui/architecture.md)
@crate: `crates/ui-egui` (ứng dụng chính `apps/darkworks`) | @layer: L6 / Presentation & Interaction | @status: 🟢 Hoàn thành Giai Đoạn 2 (Timeline Lightworks UX)

## 1. VAI TRÒ & TRÁCH NHIỆM
Chịu trách nhiệm hiển thị toàn bộ giao diện 60fps siêu mượt bằng `egui` + `wgpu`, hỗ trợ Dockable Panels, Dual Monitor (Source Monitor + Program Monitor), Audio Mixer đa kênh, và Trục thời gian Timeline chuẩn phong cách biên tập **Lightworks Mark & Park**.

## 2. CẤU TRÚC PHÂN VÙNG GIAO DIỆN
1. **Source Monitor & Program Monitor ([panels/monitor.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/monitor.rs)):**
   - Source Viewer: Xem và cắt in/out clip thô trong Media Bin.
   - Program Viewer: Khung xem trước render Zero-Copy từ GPU `wgpu::Texture`, bám sát theo Playhead của Sequence.
2. **Media Bin / Project Browser ([panels/project.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/project.rs)):**
   - Quản lý bins, media files (video, audio, hình ảnh), hỗ trợ kéo thả trực tiếp vào Timeline hoặc Source Monitor.
3. **Timeline Lightworks NLE ([panels/timeline/](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/timeline/)):**
   - Trục thời gian đa track (V1, V2 Video; A1, A2 Audio, Caption tracks).
   - Cơ chế Mark & Park chuẩn Lightworks:
     - `I` ➔ Mark In (`markers.markIn`)
     - `O` ➔ Mark Out (`markers.markOut`)
     - `P` ➔ Clear In/Out (`markers.clearInOut`)
     - `X` ➔ Extract (`sequence.extract` - Ripple Delete vùng chọn In/Out)
     - `Z` ➔ Lift (`sequence.lift` - Xóa rỗng vùng chọn để lại khoảng trống Gap)
     - `S` hoặc `C` ➔ Razor Split (`sequence.addEdit` - Cắt clip tại Playhead)
     - `Shift + S` ➔ Bật/tắt hít mép timeline (`sequence.snap`)
   - **Trực quan hóa dải Cyan Highlight ([ruler.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/timeline/ruler.rs) & [timeline.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/timeline.rs)):**
     - Dải phủ màu Cyan bán trong suốt `rgba(0, 210, 255, 35)` phủ xuyên suốt toàn bộ chiều dọc tất cả các Track.
     - Hai đường laser viền Cyan ở hai đầu mốc In và Out.
     - Hai mốc ngoặc vuông `[` (In) và `]` (Out) màu Cyan nổi bật trên thanh Ruler.
4. **Audio Track Mixer ([panels/mixer.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/mixer.rs)):**
   - Bộ trộn âm thanh đa kênh, faders, VU meters theo chuẩn LUFS / dBTP.
5. **Node Graph Panel & Timeline Bridge ([panels/node_graph.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/node_graph.rs), [panels/node_graph_bridge.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/node_graph_bridge.rs), [panels/node_graph_presets.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/node_graph_presets.rs)):**
   - Giao diện đồ thị `egui_snarl` với menu thêm 18 Atomic Nodes ([panels/node_graph_menus.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/node_graph_menus.rs)), Visual Groups ([panels/node_graph_groups.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/node_graph_groups.rs)), và Lưu/Xuất/Nhập `.preset.json` + Thư viện User Presets dùng chung mọi project (hỗ trợ Replace & Chain) tại [panels/node_graph_presets.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/node_graph_presets.rs).
   - Cầu nối đồng bộ xuống Timeline & Program Monitor: `📌 Apply to Clip`, `🎬 Add as FX Clip`, `📝 New Graph (Nháp)` (tách sang chế độ Nháp qua `detach_to_scratchpad()` tránh live-sync ghi đè Clip cũ khi `Load Preset` / `Clear All` / `Reset`), `🗑 Gỡ khỏi Clip`, và badge `◈` Cyan trên clip Timeline ([clips.rs](file:///mnt/Data/darkworks/crates/ui-egui/src/panels/timeline/clips.rs)).

## 3. CÁC HỆ THỐNG PHỤ TRỢ CỐT LÕI
- **Lịch sử Undo / Redo Toàn Cục:**
  - Cơ chế `Arc<Project>` Copy-on-Write từ `crates/engine`: mọi thao tác qua `Session::execute` đều tự động lưu snapshot, bấm `Ctrl + Z` (Undo) và `Ctrl + Y` (Redo) tức thì mà không tốn RAM.
- **Tách Biệt Độc Lập Khỏi FilmCraft Server:**
  - Đã loại bỏ hoàn toàn crate `craft-update`, `updates_ui` và các cờ check update để ứng dụng Darkworks chạy độc lập 100%.
  - Loại bỏ nút Discord trên thanh Header và toàn bộ submenu thừa của Help (giữ lại nhãn `Help` để gắn tài liệu sau).
- **Kiểm Thử Nghiệp Vụ:**
  - Hỗ trợ lái và kiểm thử trực tiếp qua skill `/verify-feature` và công cụ `.agents/harness/drive_feature.py`.
