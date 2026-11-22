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


### 16. Định Hướng Chiến Lược Hướng 1: Nền Tảng FilmCraft + Timeline Lightworks + Node Graph
- **Kế thừa nền tảng vững chắc:** Tận dụng 100% năng lực giải mã phần cứng VA-API, A/V clock sync `cpal`, `FrameCache` LRU < 2GB và cơ chế quản lý dự án của FilmCraft để tiết kiệm hàng tháng code lại bánh xe.
- **Cấy ghép Lightworks UX:** Đưa trọn vẹn triết lý Mark & Park đỉnh cao của Lightworks vào Timeline (`I, O, P, X, Z, S`, dải Cyan Highlight xuyên track, 2 ngoặc `[` `]`, Gap đen chuẩn NLE).
- **Trọng tâm cách mạng:** Cắt bỏ hoàn toàn khối `Vec<EffectInstance>` tuyến tính cũ kỹ của FilmCraft, thay bằng Đồ thị DAG Node Graph (`egui_snarl`) và 93 Shader WGSL nguyên tử.

### 17. Nâng Cấp Công Cụ Kiểm Toán `audit.py` & Cơ Chế Ngoại Lệ Lớp Nền (Vendor Exemption)
- Đổi tên và tái cấu trúc `check_file_size.py` thành công cụ trung tâm `audit.py` dạng mô-đun mở rộng.
- Thiết lập cơ chế phân định minh bạch:
  - Mã nguồn mới do Darkworks phát triển: Bắt buộc tuân thủ trần nghiêm ngặt `< 700 dòng`.
  - Mã nguồn lớp nền Upstream / Vendor của FilmCraft: Được cấp quyền ngoại lệ (`⚪ [VENDOR EXEMPT]`) qua file `filesize_exceptions.txt` để bảo toàn tính nguyên bản và tránh sinh lỗi ngầm.
- Tách `crates/ui/src/panels/timeline_menu.rs` (100 dòng) độc lập khỏi `timeline.rs` (634 dòng), giữ toàn bộ 32/32 files của Darkworks luôn xanh đạt chuẩn `< 650 dòng`.

### 18. Lịch Sử Thao Tác Con Người (`HistoryManager`) & Menu Ngữ Cảnh Chuột Phải
- **Hệ thống Undo/Redo tức thì:** Tích hợp `HistoryManager` (`crates/ui/src/history.rs`) lưu tối đa 50 snapshots `Project` trong RAM với chi phí < 0.001ms.
- **Phân định phím tắt chuẩn:** Phím `Z` đơn lẻ là lệnh **Lift (Xóa rỗng giữ chỗ)**; tổ hợp `Ctrl + Z` là lệnh **Undo (Hoàn tác)**; `Ctrl + Y` hoặc `Ctrl + Shift + Z` là lệnh **Redo (Làm lại)**.
- **Thao tác nhanh cho con người:** Bổ sung phím `Delete` / `Backspace` xóa clip được chọn và Menu Chuột Phải (`response.context_menu`) cho Clip (Split, Ripple Delete, Lift, Delete, Toggle Mute).

### 19. Tái Thiết Lập Workspace Darkworks Natively Trên Lớp Nền FilmCraft (2026-11-21)
- **Khắc phục triệt để lỗi lệch hướng:** Xóa bỏ hoàn toàn 3 crate mini chắp vá (`crates/schema`, `crates/media` dùng ffmpeg subprocess pipe, `crates/ui`).
- **Lớp nền chuẩn mực:** Đưa trực tiếp 33 crates của FilmCraft vào làm hạ tầng cốt lõi (`crates/*`).
- **Ứng dụng chính Darkworks:** Đổi tên và cấu hình `apps/darkworks` (Desktop NLE) và `apps/darkworks-cli` (Headless CLI / MCP server).
- **Kiểm chứng thực tế:** Biên dịch 100% thành công, render frame 1080p bằng `darkworks-cli render` đạt tốc độ 32.5 ms. Sẵn sàng cho Giai đoạn cấy ghép Node Graph DAG (`egui_snarl`) và trải nghiệm Lightworks.

### 20. Cắt Bỏ Triệt Để Mô-đun Cập Nhật Tự Động (Updater Module Removal) Khỏi Darkworks (2026-11-21)
- **Nguyên nhân gốc rễ (RCA):** Lớp nền FilmCraft tích hợp sẵn module `craft-update` và `updates_ui` tự động truy vấn máy chủ GitHub/Storytold của FilmCraft lúc khởi động app. Vì Darkworks đánh dấu phiên bản `version = 0.1.0`, updater thấy `0.1.0 < 0.20.1` nên bật popup modal yêu cầu người dùng update FilmCraft.
- **Giải pháp giải phẫu ngoại khoa (Xóa bỏ tận gốc, không chỉ tắt cờ):**
  - Gỡ bỏ vĩnh viễn dependency `craft-update` khỏi root `Cargo.toml`, `apps/darkworks/Cargo.toml`, và `crates/ui-egui/Cargo.toml`.
  - Xóa file giao diện update `crates/ui-egui/src/updates_ui.rs` và test liên quan `crates/ui-egui/tests/flatpak_updates.rs`.
  - Cắt bỏ hoàn toàn runtime kiểm tra update và menu `"app.checkForUpdates"` trong `main.rs`, `lib.rs`, `app.rs`, `frame.rs`, và `menus.rs`.
  - Dọn sạch thư mục dư thừa `apps/filmcraft-web/`.
- **Kết quả nghiệm thu:** Biên dịch `cargo check --workspace` và `cargo build --bin darkworks` thành công 100%, modal popup hoàn toàn biến mất, ứng dụng khởi động độc lập tuyệt đối.

### 21. Chuẩn Hóa Kiến Trúc Kiểm Thử 2 Tầng: Phân Định Rạch Ròi Cổng Kỹ Thuật (`/e2e_test`) & Cổng Nghiệp Vụ (`/verify-feature`) (2026-11-21)
- **Bối cảnh & Vấn đề:** Kịch bản kiểm thử hành vi người dùng (Feature Map) sinh ra từ template mẫu ngoài bị nằm lạc lõng trong `.cursor/` và `.agents/skills/verify-darkworks/`, không tương thích với quy chuẩn 3 ngăn kéo (`docs-management.md`) và bị nhầm lẫn chức năng với `/e2e_test`.
- **Phân định 2 tầng kiểm thử chuẩn mực:**
  - **Tầng 1 - Cổng Kỹ Thuật Toàn Cục (`/e2e_test` qua `run_e2e_audit.py`):** Kiểm tra tính toàn vẹn code trước khi kết ca (compile, clippy, memory leak, trần dòng code, smoke test khởi động).
  - **Tầng 2 - Cổng Chấp Nhận Nghiệp Vụ (`/verify-feature` qua `drive_feature.py`):** Lái app thật theo đúng kịch bản hành vi người dùng (User Journey), trích xuất khung hình render, log trạng thái và bắt bằng chứng thực tế.
- **Sắp đặt cấu trúc Không Rác (Zero-Trash):**
  - Đưa toàn bộ 9 file kịch bản tính năng về Ngăn kéo 1: `.darkworks-ai/1-overview/features/` và lưu bản đồ tổng quan tại `.darkworks-ai/1-overview/feature_map.md`.
  - Tạo skill chuẩn `.agents/skills/verify-feature/SKILL.md` và script điều khiển `.agents/harness/drive_feature.py`.
  - Xóa bỏ hoàn toàn thư mục thừa `.cursor/` và `.agents/skills/verify-darkworks/`.
- **Nghiệm thu:** `drive_feature.py --feature playback` tự động kích hoạt `darkworks-cli` render khung hình 1080p và bắt bằng chứng tại `target/feature_evidence/playback_demo_frame.png` thành công.

### 22. Cấy Ghép Triết Lý Lightworks UX Vào Nền Tảng Darkworks (2026-11-21)
- **Kích hoạt Preset phím tắt Darkworks Default:**
  - `I` ➔ Mark In (`markers.markIn`)
  - `O` ➔ Mark Out (`markers.markOut`)
  - `P` ➔ Clear In/Out (`markers.clearInOut`)
  - `X` ➔ Extract (`sequence.extract` - Ripple Delete vùng chọn In/Out)
  - `Z` ➔ Lift (`sequence.lift` - Xóa rỗng vùng chọn để lại khoảng trống Gap)
  - `S` / `C` ➔ Razor Split (`sequence.addEdit` - Cắt clip tại Playhead)
  - `Shift+S` ➔ Snap Toggle (`sequence.snap` - Bật/tắt hít mép timeline)
- **Trực quan hóa dải sáng Cyan Highlight:**
  - Thước đo Ruler: Vẽ dải phủ màu Cyan `rgba(0, 210, 255, 90)` kết hợp 2 dấu ngoặc vuông `[` (In) và `]` (Out).
  - Thân Timeline: Phủ dải bán trong suốt màu Cyan `rgba(0, 210, 255, 35)` xuyên suốt toàn bộ chiều dọc tất cả các Track (V1, V2, A1, A2...) kèm 2 đường laser biên độ Cyan tại In và Out.
- **Nghiệm thu Giai đoạn 2 (DoD 2):**
  - Chạy `python3 .agents/harness/drive_feature.py -f all` xác nhận 100% 3/3 features PASS (Media Playback, Multi-Track Timeline, Lightworks Editing).
  - Kiểm toán `python3 .agents/harness/audit.py` xác nhận 0 vi phạm dòng code. Bản build sẵn sàng tại `target/debug/darkworks`.

### 23. Tinh Chỉnh Giao Diện Header & Menu Bar (Loại Bỏ Discord & Submenu Help) (2026-11-22)
- **Loại bỏ nút Discord:** Gỡ bỏ hoàn toàn nút bấm Discord trên thanh Header bar (`crates/ui-egui/src/header.rs`), giải phóng không gian cho tiêu đề project.
- **Lược bỏ Submenu Help:**
  - Giữ lại chữ `Help` trên thanh Menu Bar làm placeholder cho hệ thống tài liệu Darkworks tương lai.
  - Xóa bỏ toàn bộ các mục con thừa kế của ArtCraft/FilmCraft (`help.discord`, `help.website`, `help.github`, `help.reportIssue`, v.v.).
  - Cập nhật bộ kiểm thử `links_ui` và `menus_ui` xác nhận không còn tồn tại các mục Help rác.
- **Nghiệm thu:** `cargo check --workspace` PASS, `cargo test -p filmcraft-ui-egui --test it` PASS, `audit.py` 0 vi phạm.

### 24. Chuẩn Hóa Thương Hiệu Người Dùng (FilmCraft ➔ Darkworks Rebranding) (2026-11-22)
- **Hộp thoại khôi phục & cảnh báo (`crates/ui-egui`):**
  - Đổi `"FilmCraft quit unexpectedly while “{name}” had unsaved changes."` ➔ `"Darkworks quit unexpectedly while “{name}” had unsaved changes."`
  - Đổi `"FilmCraft was closed while..."` ➔ `"Darkworks was closed while..."`
  - Đổi `"FilmCraft hit an error"` ➔ `"Darkworks hit an error"`
  - Cập nhật toàn bộ các file từ điển bản dịch `crates/ui-egui/src/i18n/*.tsv`.
- **Dự án mẫu mặc định & Cài đặt hệ thống (`crates/engine`, `crates/project`):**
  - Đổi tên dự án mẫu mặc định từ `"FilmCraft – Demo Project"` ➔ `"Darkworks – Demo Project"`.
  - Đổi Renderer mặc định sang `"Darkworks GPU Acceleration (wgpu)"` và `"Darkworks Software Only (CPU)"`.
- **Ứng dụng chính Desktop (`apps/darkworks`):**
  - Khởi chạy cửa sổ eframe với tiêu đề `"Darkworks"`.
  - Cập nhật tiêu đề hộp thoại cảnh báo hệ thống (`rfd::MessageDialog`) và bộ lọc file (`"Darkworks Project"`).
  - Cập nhật menu ứng dụng macOS native (`"Darkworks"`, `"About Darkworks"`) và file `.desktop` Linux.
- **Nghiệm thu:** `cargo check --workspace` PASS, `cargo test` PASS, `audit.py` 0 vi phạm (< 700 dòng), `drive_feature.py -f all` 100% PASS. Biên dịch binary thành công tại `target/debug/darkworks`.

### 25. Cập Nhật Icon Ứng Dụng Chính Thức (Darkworks App Icon & Desktop Integration) (2026-11-22)
- **Tạo bộ icon Darkworks đa kích thước:**
  - Kết xuất icon hình học Squircle 1024x1024 từ logo chòm sao Node Graph Play Triangle (Neon Cyan & Purple Flare) tại `assets/branding/darkworks-app-icon.html`.
  - Tự động sinh trọn bộ icon chuẩn `assets/app-icon/darkworks-{512, 256, 128, 64, 48, 32, 24, 16}.png` và `darkworks.ico`.
  - Cập nhật theme hicolor hệ thống `~/.local/share/icons/hicolor/` và desktop entry `dev.darkworks.editor.desktop`.
- **Nạp icon trong ứng dụng (`apps/darkworks/src/main.rs`):**
  - Sửa hàm `app_icon()` nạp trực tiếp `darkworks-256.png` và `darkworks-512.png`.
- **Nghiệm thu:** Icon trên thanh Taskbar / Dock Linux và tiêu đề window hiển thị chuẩn xác biểu tượng Darkworks. Biên dịch `cargo build --bin darkworks` thành công 100%.
