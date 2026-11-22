# BẢN ĐỒ TÍNH NĂNG TOÀN DỰ ÁN (FEATURE MAP INDEX)
@target: Darkworks NLE Video Editor | @architecture: FilmCraft Core Fork + Lightworks Timeline + Node Graph DAG

Bản đồ tính năng phân định rạch ròi 100% giữa **Lớp nền kỹ thuật kế thừa từ FilmCraft** và **Các tính năng cách mạng độc quyền do Darkworks phát triển**.

---

## 🏛️ BẢNG TỔNG QUAN PHÂN ĐỊNH (SYSTEM TAXONOMY MATRIX)

| Mã Phân Hệ | Tên Tính Năng | Nguồn Gốc / Bản Quyền | Trạng Thái | Vai Trò Trong Dự Án |
| :--- | :--- | :---: | :---: | :--- |
| `FC-01` | [Media Playback & VA-API Hardware Decode](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/features/filmcraft_media_playback.md) | **FilmCraft Foundation** | 🟢 Có sẵn | Demux đa định dạng, giải mã phần cứng Linux VA-API Zero-Copy, FrameCache < 2GB RAM |
| `FC-02` | [Multi-Track Timeline & NLE Core](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/features/filmcraft_timeline_multitrack.md) | **FilmCraft Foundation** | 🟢 Có sẵn | Trục thời gian đa track, Snapping, Ripple Edit, Snapshot Undo/Redo `Arc<Project>` |
| `FC-03` | [Audio Mixer & Master Sync Clock](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/features/filmcraft_audio_mixer.md) | **FilmCraft Foundation** | 🟢 Có sẵn | Mixer âm thanh đa kênh, VU Peak meters, A/V Sync với Audio Clock làm Master |
| `FC-04` | [Asset Browser & Project Management](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/features/filmcraft_project_assets.md) | **FilmCraft Foundation** | 🟢 Có sẵn | Quản lý bins, thư mục, metadata tags, import video/audio/hình ảnh |
| `FC-05` | [93 WGSL Shaders Library](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/features/filmcraft_shaders_library.md) | **FilmCraft Foundation** | 🟢 Có sẵn | Thư viện 93 shader xử lý hình ảnh, blend modes 26 chế độ, color transform |
| `DW-01` | [DAG Node Graph Engine & Presets](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/features/darkworks_node_graph.md) | ⚡ **Darkworks Core** | ⚪ Sắp tới | Thay thế Effect Controls tuyến tính bằng mạng đồ thị `egui_snarl`, đóng gói `.preset.json` |
| `DW-02` | [Procedural Motion (Audio Reactivity)](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/features/darkworks_audio_reactivity.md) | ⚡ **Darkworks Core** | ⚪ Sắp tới | Socket âm thanh (RMS Peak, Pitch) cắm dây trực tiếp điều khiển Scale/Shake/Glow video |
| `DW-03` | [Lightworks Mark & Park Editing](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/features/darkworks_lightworks_editing.md) | ⚡ **Darkworks Core** | 🟡 Đang cấy ghép | Phím tắt I/O/P, Extract (X), Lift (Z), Split (S), dải sáng Cyan Highlight toàn tracks |
| `DW-04` | [Vello Canvas 2D GPU Node](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/features/darkworks_vello_canvas.md) | ⚡ **Darkworks Core** | ⚪ Sắp tới | Vello + Cosmic-text vẽ Text Karaoke nhảy màu, vector SVG bằng Compute Shader (Zero RAM web) |
| `DW-05` | [AI Local Silence Cut & Subtitles](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/features/darkworks_ai_vad_silence.md) | ⚡ **Darkworks Core** | ⚪ Sắp tới | Silero VAD băm sạch khoảng lặng dưới 5s, Whisper offline tạo phụ đề tự động |

---

## 🧭 HƯỚNG DẪN ĐIỀU HƯỚNG (NAVIGATION)

- **Nếu kiểm toán nền tảng chạy video, âm thanh, cắt ghép cơ bản:** Xem các tài liệu bắt đầu bằng tiền tố `filmcraft_*`.
- **Nếu kiểm toán các tính năng đột phá, giao diện nối dây node, phím tắt Lightworks, đồ họa 2D:** Xem các tài liệu bắt đầu bằng tiền tố `darkworks_*`.
