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

### 10. Giới Hạn Dòng Code (< 700 Dòng) & Quy Trình Báo Cáo Người Dùng
- **Mức trần chuẩn mực & báo động:** **700 dòng code** mỗi file `.rs`.
- **Quy tắc tuyệt đối:** Khi file chạm mốc 700 dòng, Agent phải dừng lại báo cáo chi tiết cho Người Dùng để xin ý kiến quyết định phương án (tách module hoặc cấp phép ngoại lệ nếu thấy cần thiết). Tuyệt đối không tự tiện tối ưu hay cắt xén code.
- **Công cụ kiểm toán:** Tích hợp script kiểm tra tại `.agents/harness/check_file_size.py`.

### 11. Kiến Trúc Crate `crates/media`: Ring-Buffer Cache (Trần Cứng 2GB RAM) & Hardware Probe VA-API
- **Byte-Budgeted LRU Cache:** Quản lý bởi `FrameCache`, khóa trần 2048MB (2GB) theo đúng Khẩu quyết tối thượng #1. Tự động giải phóng các khung hình cũ nhất về 85% budget khi chạm ngưỡng.
- **Phần cứng Linux VA-API:** Tách biệt và bảo vệ an toàn qua `libloading` động (`libva.so.2` + `libva-drm.so.2` trên `/dev/dri/renderD128`).
- **Đồng bộ A/V Sync:** Khởi chạy `cpal` với đồng hồ nguyên thủy `Tick Clock`.
- **Không Panic:** Mọi luồng xử lý audio/video và FFI trả về `Result<T, MediaError>`. Cấm tuyệt đối unwrap/panic.
- **Giả lập kiểm thử độc lập:** Tích hợp `ProceduralMotion` 60fps và `SmpteBars` (1kHz tone) để test Viewport, scrubbing và audio playback tức thì.

### 12. Triệt Tiêu Lỗi Đứt Gãy Pha Âm Thanh (Audio Phase Discontinuity) & Khóa Đồng Bộ Audio Master Clock
- **Nguyên nhân gốc rễ (RCA):** Việc bù đệm âm thanh theo `playhead` hiện tại (trong quá khứ so với đuôi của ring buffer) gây hiện tượng giật lùi pha (phase jump) 60 lần/giây ➔ tạo tiếng rè rè liên tục.
- **Con trỏ Monotonic Audio Feed Head:** Quản lý bởi `audio_feed_head: Tick`. Khi nạp đệm, liên tục tịnh tiến `audio_feed_head` về tương lai, đảm bảo đường sóng âm liền mạch 100%.
- **Khóa Đồng bộ Audio Master Clock:** Playhead bám trực tiếp theo số khung hình audio phần cứng đã tiêu thụ (`audio_player.current_tick()`), loại bỏ độ trôi lệch tích lũy. Khi không có thiết bị âm thanh, tự động fallback an toàn về `dt` của egui.

### 13. Tích Hợp `FfmpegMediaSource`: Giải Mã Video MP4 1 Tiếng Thực Tế & Đệm Âm Thanh On-Demand
- **Giải mã Video Fast Seek:** Sử dụng FFmpeg sub-pipeline với tham số `-ss` trước `-i` để nhảy nhanh đến đúng keyframe (< 80ms) và trích xuất khung hình RGBA8 đưa vào `FrameCache` (2GB cap).
- **Bộ đệm Audio On-Demand:** Thay vì nạp toàn bộ 1 tiếng audio uncompressed (ngốn 1.2GB RAM), `audio_samples` trích xuất và đệm từng block 3 giây âm thanh PCM f32le, giữ RAM toàn bộ ứng dụng chỉ ở mức ~150MB - 300MB khi phát video 1 tiếng.
- **Nghiệm thu DoD Phase 1:** Hoàn thành 100% việc nạp và phát video 1 tiếng `tropical_paradise_1hour.mp4` trên Darkworks Viewport và Timeline.

### 14. Kiến Trúc Luồng Prefetch Ngầm (Background Streaming Prefetch Worker) Khôi Phục Vững Chắc 60 FPS
- **Nguyên nhân gốc rễ (RCA) tụt xuống 5 - 11 FPS:** Ở kiến trúc cũ, khi cache miss, vòng lặp Viewport trên UI Main Thread trực tiếp spawn tiến trình FFmpeg đơn lẻ (`ffmpeg -ss ... -frames:v 1`), gây nghẽn UI Thread 80 - 120ms mỗi frame ➔ FPS tụt thảm hại xuống 5 - 11 FPS.
- **Giải pháp Luồng Ngầm Streaming Pipe:**
  - Tách rời hoàn toàn việc giải mã khỏi luồng UI: `FfmpegMediaSource` khởi chạy một worker thread ngầm (`run_prefetch_loop`).
  - Worker thread duy trì 1 tiến trình ffmpeg streaming output raw RGBA pipe liên tục với tốc độ giải mã cực nhanh (~53x real-time), liên tục bơm khung hình vào `FrameCache` trước vị trí Playhead khoảng 45 khung hình (~1.5 giây).
  - Vòng lặp Viewport trên UI Thread chỉ việc lấy frame đã có sẵn trong `FrameCache` (< 0.001ms), khôi phục và giữ vững 60 FPS mượt mà tuyệt đối.
  - Hỗ trợ Fast Seek: Khi người dùng kéo thả (scrubbing) timeline hoặc nhảy vị trí, worker thread tự động ngắt pipe cũ và mở nhanh stream mới từ vị trí đích.

### 15. Phân Tầng Cache 2 Cấp (Mức 1: 1GB, Mức Trần: 2GB) & Thuật Toán Dọn Dẹp O(1) Triệt Tiêu Lỗi Tụt 10 FPS
- **Nguyên nhân gốc rễ (RCA) tụt 10 FPS khi chạm trần RAM:**
  - Khi cache chạm ngưỡng, thuật toán cũ xả lũ 15% budget cùng lúc (~300MB, ~350 frame) ngay trong khi đang giữ khóa `Mutex`.
  - Quá trình clone 2.000+ key chuỗi và sort hàng ngàn phần tử làm tắc nghẽn luồng UI Thread trong vài mili-giây, dẫn đến việc rớt 10 FPS.
- **Giải pháp 2 cấp độ bộ nhớ:**
  - **Mức 1 (1024MB = 1GB):** Ngân sách đệm video chuẩn khi phát video dài.
  - **Mức Trần (2048MB = 2GB):** Trần cứng bất biến của toàn bộ ứng dụng Darkworks, chừa không gian cho AI Worker (Whisper/VAD ~350MB) và Audio (~100MB). Các shader Node Graph và Canvas 2D chạy 100% trên VRAM nên không tiêu tốn RAM hệ thống.
- **Thuật toán Eviction vi mô O(1) & Deallocate Ngoài Khóa Mutex:**
  - Sử dụng `VecDeque<(K, u64)>` theo dõi thứ tự truy cập. Mỗi khi cache đầy, hệ thống chỉ giải phóng frame cũ nhất xa nhất trong quá khứ ($O(1)$) ngay khi nạp frame mới.
  - Con trỏ frame cũ được đẩy ra khỏi HashMap và giải phóng vùng nhớ (`drop`) hoàn toàn NGOÀI khóa `Mutex`, đảm bảo độ trễ truy xuất frame trên UI Thread luôn $< 0.0001\text{ms}$, giữ vững 60 FPS vĩnh viễn từ đầu đến cuối video 1 tiếng.
