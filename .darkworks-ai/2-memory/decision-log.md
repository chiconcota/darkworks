# SỔ TAY QUYẾT ĐỊNH KIẾN TRÚC DOANH NGHIỆP: DARKWORKS (decision-log.md)
@project: DARKWORKS | @authority: Giám Đốc (Người Dùng) | @status: Bất Biến (Immutable Memory)

> Tài liệu này lưu trữ toàn bộ các quyết định chiến lược và kiến trúc kỹ thuật đã được Giám Đốc phê duyệt.  
> Nhân viên AI tuyệt đối không được tự ý lật lại hay hỏi lại những vấn đề đã được chốt trong file này.

---

## CÁC QUYẾT ĐỊNH NỀN TẢNG ĐÃ CHỐT:

### 1. Nền Tảng Công Nghệ (100% Native Rust)
- Toàn bộ phần mềm viết bằng Rust thuần, giao diện `egui`, render đồ họa `wgpu` (Vulkan/VA-API), âm thanh `cpal` (PipeWire/ALSA).
- Không dùng GStreamer cồng kềnh (tránh phụ thuộc GObject/GLib); ưu tiên giải mã phần cứng OS native kết hợp Ring-Buffer cache.

### 2. Khung Sườn Nền Tảng (Base Engine)
- Chọn **FilmCraft (`storytold/filmcraft`)** làm khung sườn nền móng: tận dụng sẵn hệ thống giải mã/mã hóa Rust, cơ chế Undo/Redo CoW (Copy-on-Write snapshots), cấu trúc lệnh `Session::execute()`, và MCP Server đã có sẵn.
- Tránh xa các dự án C++ cồng kềnh đời cũ (Kdenlive/Shotcut/MLT Framework) hoặc dự án C++ thiếu ổn định (Olive 0.2).

### 3. Kiến Trúc Hiệu Ứng (DAG Node Graph Thay Vì Linear Stack)
- Không dùng mô hình danh sách phẳng tuần tự (Effect Controls của Premiere).
- Nâng cấp thành **Mạng lưới đồ thị có hướng (DAG)** sử dụng `egui_snarl`: cho phép rẽ nhánh, trộn lớp song song, và lấy tín hiệu âm thanh (Audio Peak/Pitch) điều khiển chuyển động hình ảnh.
- Toàn bộ sơ đồ Node có thể xuất/nhập thành file `.preset.json` độc lập.

### 4. Đồ Họa 2D & Chữ (Canvas 2D Node Trên GPU)
- Khai tử ý tưởng nhúng trình duyệt Web (Chromium/CEF/WebKit) để tránh ngốn 1GB RAM và nghẽn băng thông chuyển pixel về CPU.
- Sử dụng **Vello + Cosmic-text** làm **Generator Node** bên trong Node Graph: vẽ vector, biểu đồ, chữ Karaoke nhảy màu trực tiếp bằng Compute Shader trên VRAM (Zero-copy).

### 5. Quản Lý Bộ Nhớ & Đồng Bộ Khung Hình
- Khóa cứng mức trần bộ nhớ RAM hệ thống `< 2GB (2048MB)` bằng Ring-Buffer LRU cache; giải phóng khung hình kịp thời, tuyệt đối không phình to 10GB như Blender VSE. Đủ sức chứa ~600 frame 1080p hoặc ~150 frame 4K mượt mà.
- Đồng bộ thời gian theo đơn vị Tick nguyên thủy (`filmcraft_time::Tick`), bảo đảm tính xác định theo từng khung hình (Deterministic Frame-by-Frame).

### 6. Tự Động Hóa & Buồng Lái AI Agent (Antigravity CLI)
- Trạng thái dự án lưu hoàn toàn dưới dạng văn bản mở: `project.json` (kèm File Watcher hot-reload).
- Mở cổng MCP Server nội bộ để Antigravity CLI tự động đọc transcript, cắt khoảng lặng (Silero VAD), tạo phụ đề (Whisper/Moonshine), và bơm code preset vào Timeline.

### 7. Chiến Lược "Bọc Ngoài & Thay Ruột" Cho FilmCraft (Modding Strategy)
- Không xóa thẳng tay hệ thống Effect của FilmCraft để tránh gây gãy compiler và mất 93 WGSL Shader có sẵn.
- Tái sử dụng toàn bộ 93 shader của FilmCraft thành các Node nguyên tử (Atomic Nodes) bên trong mạng lưới DAG Node Graph của Darkworks.
- Cửa sổ mã nguồn FilmCraft được clone nguyên bản tại `reference/filmcraft/` để làm tư liệu đối chiếu tham khảo và cách ly bằng `.gitignore`.

### 8. Chuẩn Hóa Cấu Trúc Buồng Lái & Bộ Nhớ (Zero-Trash & 3-Drawers)
- Buồng lái AI gom gọn về duy nhất `.agents/` (chứa `rules/`, `skills/`, `workflows/`, `harness/`).
- Bộ nhớ dự án nằm tại `.darkworks-ai/` chia làm 3 ngăn kéo chuẩn: `1-overview/`, `2-memory/`, `3-modules/`.
- Luật cốt tử #1 (`~/.gemini/GEMINI.md`): Tuyệt đối không tự ý sửa code khi User báo lỗi; bắt buộc phân tích RCA và xin phép trước.

### 9. Phân Tách Node Text & Node Shape/Vector Trên Node Graph (Vello GPU Backend)
- **Engine Tầng Dưới (`crates/canvas-renderer`):** Dùng chung engine Vello (GPU Vector Compute Shader) + Cosmic-text để render mọi đường cong Bézier, vector và font glyphs lên VRAM.
- **Tầng Giao Diện Editor (`crates/node-graph`):** Tách bạch triệt để UX thành các Node chuyên biệt, không gộp chung:
  - **Node Text / Subtitle (Generator Node):** Chuyên text string, font, size, alignment, karaoke color sync theo tick.
  - **Node Shape / Vector (Generator Node):** Chuyên vẽ hình khối cơ bản (Rectangle, Circle, Star), hỗ trợ import file `.svg` vector sắc nét.
  - **Node Canvas Group (Composite Node):** Đóng gói cụm nhiều layer text + shape (ví dụ: Lower-third tin tức) thành một preset JSON duy nhất.
