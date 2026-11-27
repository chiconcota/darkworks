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

### 26. Kiến Trúc Node Graph Mở Rộng: Text Generator, Black Node & Action Node (2026-11-22)
- **Bối cảnh & Quyết định:** Khai tử danh sách tuyến tính `Vec<EffectInstance>` cũ của FilmCraft/Premiere, thay thế bằng đồ thị DAG (`egui_snarl`). Mở rộng kiến trúc đồ thị với 3 thành phần cốt lõi:
  1. **Text Generator Node:** Đưa Text Node thành Node nguyên tử nền tảng ngay trong Phase 3 (dựa trên `crates/text`), sinh texture RGBA có alpha để hòa trộn qua Blend Node đè lên video.
  2. **Black Node (Compound / Subgraph FX Layer):**
     - Đóng gói cụm nhiều Atomic Nodes (Text + Glow + Blur + Color Grade...) thành 1 Node duy nhất trên canvas, chống rối đồ thị (Anti-Spaghetti).
     - Hỗ trợ Drill-down (nhấp đúp mở đồ thị con) và bộc lộ các chân cắm quan trọng (`Exposed Pins`).
     - Định dạng lưu trữ chuẩn của file `.preset.json`.
     - Đóng vai trò như một FX Layer / Adjustment Block trên Timeline khi áp dụng cho toàn bộ các layer video bên dưới.
  3. **Action Node (Workflow Automation & Batch Runner):**
     - Trạm tự động hóa quy trình dựng video trực quan.
     - Tiếp nhận phạm vi áp dụng (Target Scope: Selected Clips, Track V1/V2, hoặc Vùng `[Mark In, Mark Out]`).
     - Có nút bấm `[ ▶ Run Workflow ]` gọi trực tiếp vào hệ thống lệnh nguyên tử `Session::execute` để chỉnh sửa/áp hiệu ứng hàng loạt trên timeline bên dưới.
     - Hỗ trợ hoàn tác 100% bằng `Ctrl + Z` thông qua snapshot Copy-on-Write.

### 27. Bộ Công Cụ Tổ Chức Đồ Thị Trực Quan Kiểu ComfyUI (Canvas Ergonomics & Organization) (2026-11-22)
- **Bối cảnh & Quyết định:** Để đồ thị Node Graph không chỉ mạnh về tính toán mà còn trực quan, sạch sẽ và thân thiện với con người khi dự án phình to, Darkworks tích hợp bộ công cụ tổ chức canvas lấy cảm hứng từ ComfyUI, Blender và Unreal Engine:
  1. **Visual Group / Backdrop Box (Khung phân vùng màu):** Khung hình chữ nhật có màu nền mờ và tiêu đề lớn; khi kéo thanh header của Group, toàn bộ các node con bên trong sẽ di chuyển đồng bộ.
  2. **Sticky Note Node (Ghi chú / Memo):** Node ghi chú không có socket, hiển thị khung text ghi nhớ, chú thích workflow hoặc nhắc việc (TODO).
  3. **Reroute Node (Nút bẻ góc dây):** Node chấm tròn nhỏ pass-through giúp bẻ góc dây nối uốn lượn ngăn nắp, tránh dây vắt chéo che khuất mặt node.
  4. **Canvas Utilities:** Lưới nền Dot-Grid, cơ chế Snap-to-Grid (hít lưới thẳng hàng), phím tắt căn lề tự động (Auto-align), và Minimap ở góc màn hình.

### 28. Kiến Trúc "Code Node" Mở Rộng Vô Hạn Khi Chạm Giới Hạn (Custom WGSL & Script) (2026-11-22)
- **Bối cảnh & Quyết định:** Để giải quyết bài toán "khi chạm tới giới hạn của các node có sẵn thì làm sao?", Darkworks thiết lập cơ chế Code Node cho phép người dùng hoặc AI Agent tạo hiệu ứng mới ngay lập tức mà không cần build lại app:
  1. **Custom WGSL Shader Node (GPU JIT):**
     - Nhúng khung soạn code WGSL trực tiếp trên thân Node.
     - `wgpu` JIT-compile trực tiếp trên VRAM trong < 2ms, Hot-reload 60fps tức thì.
     - Tự động phân tích các biến `uniform` khai báo trong shader để tạo ra các socket tham số kéo dây ra ngoài.
     - Sẵn sàng đón đầu AI Agent (Antigravity CLI): người dùng yêu cầu bằng lời ➔ AI sinh code WGSL bơm thẳng vào node.
  2. **Script Action Node (CPU Procedural Automation):**
     - Chạy script Lua qua `crates/plugin-host` (`mlua` / LuaJIT) cho các logic tự động hóa timeline phức tạp.
  3. **Đóng gói thành Preset:** Code Node có thể đóng gói thành Black Node và xuất file `.preset.json` để chia sẻ cho cộng đồng sử dụng qua các thanh trượt trực quan mà không bị lộ code.


### 29. Quy Hoạch AI Visual Inference Node (Face Swap / Swiper & AI Background Removal) Vào Phase 5 (2026-11-22)
- **Bối cảnh:** Người dùng đặt câu hỏi về khả năng mở rộng AI trong tương lai (tích hợp các mô hình AI xử lý hình ảnh như Swipers/Face Swap hoặc Tách nền AI).
- **Quyết định kiến trúc:**
  1. **Phân tách 2 hình thái tích hợp AI:**
     - **AI Audio & Automation (Whisper, Silero VAD):** Tích hợp dưới dạng `Action Node` hoặc tiến trình ngầm (chạy 1 lần, không render per-frame 60 FPS).
     - **AI Computer Vision (Face Swap / Swiper, Rembg / AI Matte, Depth-Map):** Tích hợp dưới dạng **AI Visual Node** trên canvas Node Graph: Nhận luồng `Texture` GPU ➔ Gọi inference `crates/ai-workers` (`ort` ONNX GPU) ➔ Xuất `Texture` đã qua xử lý (kèm kênh Alpha).
  2. **Bổ sung Task 5.3 vào Phase 5:**
     - Mở rộng năng lực của `crates/ai-workers` phục vụ cả Speech AI lẫn Visual AI.
     - Khả năng kết hợp tự nhiên trên Node Graph: Ghép `VideoInput` ➔ `AiRembgNode` ➔ `Blend` với nền mờ phía sau để tạo hiệu ứng xóa phông điện ảnh cực nhanh.


### 30. Hệ Sinh Thái 18 Versatile Atomic Nodes Bản Đầy Đủ & Kho 5 Built-in Presets Mẫu Cốt Lõi (2026-11-22)
- **Bối cảnh & Quyết định:** 
  - Thống nhất nguyên tắc: **Không chia bản Simple / Advanced** gây phức tạp phân mảnh. Toàn bộ 18 Node nguyên tử đều được cung cấp bản đầy đủ 100% (Full Production-Ready) ngay từ đầu.
  - Hợp nhất 93 Shaders WGSL cũ của FilmCraft thành 18 Node nguyên tử đa năng theo chuẩn Nuke/Fusion (tránh sinh 93 node lẻ gây bãi rác canvas):
    1. *Generators (4):* `VideoInput`, `TextGenerator`, `ShapeGenerator`, `TimeDriver`.
    2. *Color & Optics (4):* `ColorGrade` (tích hợp Invert & B&W), `BlurSharpen` (5 loại blur), `DistortLens` (6 loại biến dạng), `Keyer` (Ultra Keyer phông xanh kèm spill suppress).
    3. *Transform & Compositing (3):* `Transform2D` (kèm Flip H/V & Anchor), `CropMask` (kèm feather), `Blend` (26 chế độ hòa trộn).
    4. *Transitions (1):* `Transition` (30 kiểu chuyển cảnh).
    5. *Signals & Automation (2):* `AudioReactor` (lấy RMS Peak tần số âm thanh lái thông số hình ảnh 60 FPS), `ActionNode` (tự động hóa).
    6. *Advanced & Ergonomics (4):* `BlackNode` (Compound Subgraph), `CustomWgsl` (JIT shader), `StickyNote`, `Reroute Dot`, kèm `Output`.
  - **Kho 5 Built-in Presets Mẫu Cốt Lõi:** Tích hợp sẵn nạp 1-click trên Toolbar:
    1. `News Ticker Banner`: Dải tin tức nền xanh bo góc + Text tiêu đề + TimeDriver cuộn ngang + Blend.
    2. `Continuous Audio Waveform Line`: Shape line 4px + Distort wave + AudioReactor + TimeDriver + Blend Screen (đường sóng nhạc liên tục 60 FPS).
    3. `Spotlight Invert`: VideoIn + Solid trắng + Shape circle mask + Blend difference (vùng tròn âm bản di chuyển).
    4. `Audio Beat Invert Flash`: VideoIn + Solid trắng + AudioReactor bass + Blend difference (chớp âm bản theo tiếng trống kick).
    5. `Cinematic Soft Glow`: VideoIn + Blur Bokeh bloom + Blend screen.

### 31. Hệ Thống Visual Group Đa Năng: Gom Nhóm Nhanh `Ctrl + G`, Kéo Thả Đồng Bộ & Khắc Phục Lỗi Chồng Đè (2026-11-23)
- **Bối cảnh & Vấn đề giải quyết:**
  - Người dùng phản ánh khi tạo 2 nhóm từ hộp thoại, cả 2 nhóm đều bị sinh ra tại tọa độ cố định `(120, 120)` đè nghẽn lên nhau gây lỗi rối chữ `gro[p 2121`.
  - Chưa có tính năng quét chọn nhiều node để gom nhóm nhanh mà phải ném thủ công từng node; chưa có khả năng di chuyển nhóm trên canvas.
- **Quyết định kiến trúc & Giải pháp:**
  1. **Phím tắt `Ctrl + G` (hoặc `Cmd + G`):**
     - Truy vấn danh sách node đang được chọn qua `egui_snarl::ui::get_selected_nodes`.
     - Tự động tính toán Bounding Box bao quanh tất cả các node được chọn cộng thêm khoảng đệm lề 35px thẩm mỹ.
     - Tự động sinh tên mặc định `Group {N}` nếu người dùng không đặt tên.
  2. **Bảng màu xoay vòng tự động (Auto-Rotating Palette):**
     - Triển khai `VisualGroup::default_color(idx)` xoay vòng qua 6 màu sắc chuẩn ComfyUI/Darkworks (Tím, Xanh dương, Xanh ngọc, Xanh lục, Cam, Đỏ thẫm) độ trong suốt 35%.
     - Nếu không có node nào chọn, vị trí nhóm mới tự động lệch so le (+50px mỗi nhóm), triệt tiêu hoàn toàn lỗi chồng đè `gro[p 2121`.
  3. **Tương tác trực tiếp trên Canvas (Direct Manipulation):**
     - **Header Bar (30px trên cùng):** Nhấp giữ chuột trái kéo thanh tiêu đề sẽ di chuyển cả khung Group VÀ đồng thời tịnh tiến tất cả các Node con nằm bên trong theo đúng độ dời `delta`.
     - **Tay cầm co giãn góc dưới phải (Resize Grip 20x20px):** Cho phép kéo mở rộng/thu hẹp biên độ khung Group mà không làm dịch chuyển các node bên trong.
     - **Khóa Panning nền Canvas:** Bắt cờ `is_dragging_group` truyền vào `DarkworksViewer::current_transform` để khóa đứng yên vị trí nền canvas, tránh bị trôi màn hình khi kéo group.
  4. **Tài liệu đặc tả UX/UI toàn diện:**
     - Lập bảng ma trận 20 tính năng UX/UI chuẩn chuyên nghiệp tại `.darkworks-ai/3-modules/node-graph/visual_group_ux_specs.md` sẵn sàng cho phiên phát triển tiếp theo.


### 32. Hoàn Thiện Bộ Công Cụ Tương Tác Visual Group UX Đạt Chuẩn ComfyUI & Blender (2026-11-24)
- **Bối cảnh & Quyết định kiến trúc:**
  - Tiếp tục nâng cấp Visual Group thành công cụ tổ chức đồ thị chuyên nghiệp, mượt mà và trực quan theo bảng đặc tả `visual_group_ux_specs.md`:
  1. **Inline Rename trên Header Bar (`F2` hoặc Nhấp đúp):**
     - Nhấp đúp trực tiếp lên Header Bar hoặc bấm phím tắt `F2` (khi rê chuột vào Group) để bật ô nhập liệu `egui::TextEdit::singleline` ngay trên tiêu đề.
     - Phím `Enter` hoặc click ra ngoài lưu tên mới; phím `Escape` hủy thao tác.
  2. **Menu Ngữ Cảnh Chuột Phải Đầy Đủ (Context Menu):**
     - Nhấp chuột phải lên thanh Header Bar để mở menu ngữ cảnh: Đổi tên (Inline Rename), Bảng màu 6 màu ComfyUI dịu mắt (Purple, Blue, Turquoise, Green, Orange, Crimson), Căn vừa Node con (Auto-Fit), Khóa / Mở khóa vị trí (Lock / Unlock), Rã nhóm (Ungroup), Xóa toàn bộ cụm.
  3. **Phím Tắt Rã Nhóm Nhanh `Ctrl + Shift + G` (Ungroup):**
     - Hủy khung nhóm Visual Group nhưng **giữ nguyên 100% tất cả Node và dây nối bên trong**.
     - Tự động ưu tiên rã nhóm chứa các node đang được quét chọn, hoặc nhóm vừa được rê chuột qua.
  4. **Căn Vừa Tự Động (Auto-Fit to Content):**
     - Nhấp đúp vào tay cầm co giãn góc dưới phải hoặc chọn "🎯 Auto-Fit" trong Menu để khung Group tự động tính lại Bounding Box bao khít toàn bộ các Node con (+35px padding lề và chừa 40px không gian Header).
  5. **Chế Độ Khóa Vị Trí (Lock Mode):**
     - Thêm trường `is_locked: bool` (hỗ trợ `#[serde(default)]` tương thích ngược tuyệt đối).
     - Khi bị khóa, thanh Header hiển thị biểu tượng 🔒, vô hiệu hóa việc kéo di chuyển và resize để bảo vệ sơ đồ khỏi bấm nhầm.
  6. **Kiến Trúc Tách Phân Hệ `node_graph_groups.rs` Độc Lập:**
     - Tách toàn bộ logic tương tác canvas, menu ngữ cảnh, inline rename và hộp thoại quản lý nhóm sang `crates/ui-egui/src/panels/node_graph_groups.rs` (354 dòng).
     - Giữ `node_graph.rs` (520 dòng) luôn xanh đạt chuẩn thiết quân luật `< 700 dòng`.
     - Bộ 14/14 tests tích hợp của `darkworks-node-graph` đạt **100% PASS**, clippy sạch 0 lỗi cảnh báo.
  7. **Khắc Phục Lỗi 01 & Lỗi Khóa Cứng Hai Chiều Khi Kéo Tách Nhóm / Resize Grip (2026-11-24):**
     - **Nguyên nhân gốc rễ (RCA):**
       - Lỗi lồng nhóm không tương tác: Hộp va chạm kiểm tra `over_any_node` dùng Bounding Box giả định thô `(220, 160)` của Node cha tạo vùng chết ảo chiếm trọn Header Bar của Group con bên dưới; đồng thời duyệt mảng theo thứ tự xuôi `0..N` khiến Group cha cướp sự kiện của Group con.
       - Lỗi không kéo tách 2 nhóm ra xa nhau được (Mutual Lock): Việc kiểm tra `old_rect.contains(center)` quá lỏng lẻo khiến hai nhóm chỉ giao cắt một phần (`nhóm 1` và `nhóm 2`) đều chứa tâm của nhau, dẫn đến việc kéo nhóm này thì nhóm kia cũng bị tịnh tiến theo, khóa chặt 2 nhóm vào nhau.
       - Lỗi tay cầm Resize khó bắt chuột: Tay cầm co giãn chỉ có kích thước 20x20px, quá bé khi hiển thị trên màn hình độ phân giải cao hoặc khi canvas ở tỷ lệ thu phóng (zoom).
     - **Giải pháp ngoại khoa toàn diện:**
       - Thu hẹp vùng kiểm tra Header Node (`over_node_header`) chỉ quét thanh tiêu đề ~26px.
       - Đảo ngược thứ tự duyệt tương tác (`.rev()`) để Group con (nằm trên / tạo sau) luôn có Z-order cao hơn và ưu tiên đón nhận hover/click trước.
       - Thay thế kiểm tra tâm bằng **Bao hàm nghiêm ngặt 100% (Strict Containment)**: `old_rect.contains(child.min) && old_rect.contains(child.max)`. Hai nhóm chỉ chồng đè một phần hoàn toàn độc lập, kéo tách rời nhau tự do.
       - Mở rộng vùng tương tác tay cầm co giãn `resize_rect` từ `20px` lên `32px` (tăng diện tích hit-box 2.5 lần), đồng thời vẽ 3 đường rãnh chéo tương phản cao chuẩn đồ họa chuyên nghiệp.
       - Bổ sung unit test `test_nested_visual_groups_detection_and_translation` bảo vệ hồi quy (14/14 tests PASS).


### 33. Đồng Bộ Phím Xóa Toàn Cục (`Delete` / `Backspace`) Cho Node Graph & Tự Động Bắt Tiêu Điểm (2026-11-24)
- **Bối cảnh & RCA:**
  - Người dùng quét chọn hàng loạt Node (`Shift + drag`) hoặc rê chuột vào Visual Group và nhấn `Delete` nhưng không có phản hồi.
  - RCA: `node_graph::show` ban đầu chưa cập nhật tiêu điểm `app.ui.focused = PanelKind::NodeGraph`, trong khi phím `Delete` bị bộ điều phối toàn cục `FilmcraftApp::handle_shortcuts` nuốt (`consume_chord`) từ đầu frame và điều hướng nhầm về Timeline.
- **Quyết định kiến trúc & Giải pháp:**
  1. **Tự động bắt tiêu điểm:** Gán `app.ui.focused = PanelKind::NodeGraph` ngay khi con trỏ chuột tương tác (click, drag) trong vùng Node Graph.
  2. **Bắt chặn thông minh trong `keys.rs`:** Khi nhận lệnh `edit.clear` hoặc `edit.rippleDelete` (`Delete` / `Backspace`), nếu tiêu điểm là `NodeGraph` hoặc Node Graph có node/group đang chọn (`has_selection`), hệ thống ưu tiên gọi `delete_selected_or_hovered` xóa ngay lập tức cho Node Graph và chặn rơi xuống Timeline.
  3. **Nút xóa trực quan trên Toolbar:** Bổ sung nút `🗑 Delete` trên thanh công cụ trên cùng để hỗ trợ thao tác chuột nhanh.
  4. **Kiểm toán an toàn:** Duy trì toàn bộ file < 700 dòng (`node_graph.rs`: 682 dòng, `keys.rs`: 517 dòng), zero panics, 15/15 unit tests PASS.


### 34. Tích Hợp Ngăn Xếp Lịch Sử Undo / Redo (`GraphHistory`) Cho Node Graph (2026-11-24)
- **Bối cảnh & RCA:**
  - Người dùng lỡ xóa node hoặc rã group nhưng bấm `Ctrl + Z` không khôi phục lại được.
  - RCA: `NodeGraphState` lưu trạng thái đồ thị trực tiếp mà không có snapshot dự phòng; lệnh `edit.undo` (`Ctrl + Z`) từ đầu frame được FilmCraft điều hướng xuống Timeline.
- **Quyết định kiến trúc & Giải pháp:**
  1. **Module `GraphHistory` & `GraphSnapshot` (`crates/node-graph/src/history.rs`):** Lưu trữ ngăn xếp ảnh chụp trạng thái `Snarl<DarkNode>`, `groups`, `custom_titles` với độ sâu tối đa 50 bước (< 5MB RAM).
  2. **Tự động chụp Snapshot:** Gọi `state.push_undo()` trước mọi thao tác xóa (`Delete`, `node_to_remove`), gom nhóm (`Ctrl + G`), rã nhóm (`Ctrl + Shift + G`), dọn sạch (`Clear All`), nạp preset (`Load Preset`).
  3. **Điều phối Phím tắt `Ctrl + Z` & `Ctrl + Y`:** Bắt chặn trong `keys.rs` khi tiêu điểm ở Node Graph để gọi `node_graph::undo` / `node_graph::redo`.
  4. **Nút bấm trực quan:** Bổ sung 2 nút `↶ Undo` và `↷ Redo` trên thanh công cụ Node Graph (tự động bật/tắt theo `can_undo` / `can_redo`).
  5. **Tách module `node_graph_menus.rs`:** Đưa menu thêm node và nạp preset ra module riêng (247 dòng), giúp `node_graph.rs` giảm mạnh từ 682 xuống 577 dòng (< 700 dòng).


### 35. Khắc Phục Triệt Để Lỗi Node Mất Snapshot Undo Khi Xóa (`Ctrl + Delete` / `Delete`) (2026-11-24)
- **Bối cảnh & RCA:**
  - Người dùng xóa node bằng tổ hợp `Ctrl + Delete` (hoặc `Delete`), node biến mất nhưng nút `↶ Undo` bị xám và `Ctrl + Z` không hồi phục được node (trong khi Group vẫn Undo được).
  - RCA:
    1. `vendor/egui-snarl/src/ui.rs` (dòng 1252-1260) có cơ chế tự tiện xóa node khỏi `snarl` bên trong hàm render `snarl.show()` mà không hề gọi `push_undo()`. Cụ thể `i.key_pressed(Key::Delete)` luôn trả về true cả khi giữ Ctrl.
    2. `snarl_id` bị lệch định danh giữa Ui cha và `canvas_ui` con, khiến `get_selected_nodes` ở ngoài truy vấn sai ID và nhận danh sách rỗng.
    3. `keys.rs` bỏ lọt tổ hợp `Ctrl + Delete` / `Ctrl + Backspace`.
- **Quyết định kiến trúc & Giải pháp:**
  1. **Xóa bỏ cơ chế xóa ngầm trong `vendor/egui-snarl`:** Chuyển giao 100% quyền kiểm soát vòng đời xóa node cho Darkworks để đảm bảo luôn lưu snapshot Undo.
  2. **Chuẩn hóa ID với `SnarlWidget::new().id(...)`:** Đảm bảo `snarl_canvas_id` cố định nhất quán 100% giữa lúc render và lúc đọc danh sách chọn.
  3. **Bắt trọn `Ctrl + Delete` & `consume_key`:** Tiêu thụ chính xác các sự kiện phím trong cả `keys.rs` và `node_graph.rs`, ngăn ngừa hiện tượng lặp phím.
  4. **Xóa vùng chọn sau khi xóa:** Gọi `egui_snarl::ui::deselect_all_nodes` để làm sạch trạng thái chọn. Nút `↶ Undo` lập tức sáng đèn và `Ctrl + Z` hồi phục đầy đủ node cùng dây nối.

### 36. Tách Lõi `darkworks-node-graph-core` (L0) & Nối Cầu Node Graph ➔ Timeline / Compositor (2026-11-25)
- **Bối cảnh & Ràng buộc:** `xtask layers` cấm mọi crate tầng L0–L4 (`project`, `render`, `gpu`, `engine`) phụ thuộc vào `egui`. Trong khi đó `darkworks-node-graph` kéo `egui` và `egui-snarl`, nên chưa thể gắn Node Graph xuống `TrackItem` hay thực thi trong `crates/render`.
- **Quyết định kiến trúc:**
  1. **Tách crate `crates/node-graph-core` (L0):** Chứa mô hình thuần dữ liệu `NodeGraphDoc`, `DarkNode`, `DagEvaluator`, `WgslParser` (chỉ phụ thuộc `serde`, `serde_json`, `thiserror`). `crates/node-graph` (L5) cung cấp cầu nối 2 chiều `doc_from_snarl` / `snarl_from_doc`.
  2. **Lưu trữ trong `TrackItem` & Lệnh `Session::execute`:** Thêm `pub node_graph: Option<Box<NodeGraphDoc>>` vào `TrackItem` (`crates/project`); bổ sung các lệnh `nodeGraph.{setOnClip, createOverlayClip, applyScope, clearFromClip, getFromClip}` (`crates/engine`) ăn theo `Arc<Project>` Undo/Redo và `project.json`.
  3. **DAG Compositor & `CustomWgsl` GPU:** `crates/render/src/node_graph.rs` (+ `node_graph_ops.rs`) đánh giá đồ thị theo frame tại `Tick` hiện tại (hỗ trợ cả media clips và adjustment/overlay layers, trích xuất RMS/band cho `AudioReactor`), kết hợp `crates/gpu/src/custom_wgsl.rs` chạy fragment shader an toàn qua `push_error_scope`.
  4. **Đồng bộ UI (`node_graph_bridge.rs` & `timeline/clips.rs`):** Thêm nút `📌 Apply to Clip`, `🎬 Add as FX Clip`, `⛓ Gỡ`, kích hoạt `[ ▶ RUN WORKFLOW ]` trên `ActionNode`, live sync khi nhả chuột và vẽ badge `◈` Cyan trên clip Timeline.

### 37. Bổ Sung Hệ Thống Keyframe & `KeyframeCurve` Node Vào Giai Đoạn 4 (2026-11-25)
- **Bối cảnh:** Giai đoạn 3 mới triển khai chuyển động procedural (`TimeDriver` + `AudioReactor`), chưa có keyframe theo mốc thời gian cho Node Graph. Tuy nhiên khi dựng motion graphics thực tế vẫn cần điều khiển chính xác theo từng mốc `Tick` (Ease-In/Out, giữ yên, trượt vào/ra).
- **Quyết định của Giám Đốc:** Bổ sung hạng mục Keyframe vào **Giai đoạn 4 (Phase 4)** song song với Vello Canvas 2D Node, gồm 2 lớp:
  1. **Keyframe trực tiếp trên tham số của `DarkNode`:** Đặt/xóa/nhảy mốc `◀ ◆ ▶` theo Playhead (`Tick`), hỗ trợ `Hold`, `Linear`, `EaseIn`, `EaseOut`, `EaseInOut` / Cubic Bezier, lưu tương thích ngược bằng `#[serde(default)]`.
  2. **Node `KeyframeCurve` (Spline Signal):** Xuất tín hiệu `Scalar` để 1 đường cong keyframe điều khiển đồng bộ nhiều Node qua socket `Mod`, đồng thời kết hợp được với `TimeDriver` và `AudioReactor` và đóng gói vào `.preset.json`.

### 38. Lộ Trình Thay Thế 100% `Effects` & `Effect Controls` FilmCraft Bằng Node Graph + Kho Preset `.preset.json` (2026-11-25)
- **Mục tiêu:** Thay toàn bộ hiệu ứng cũ của FilmCraft bằng Node Graph + Thư viện `.preset.json` lưu sẵn, sau đó xóa hẳn UI `Effects` và `Effect Controls` cũ khỏi Darkworks.
- **Lộ trình an toàn 3 bước (không làm mất tính năng giữa chừng):**
  1. **Phase 3 (Task 3.8 / 3.12):** Nâng cấp Export/Import file `.preset.json` thật theo `NodeGraphDoc` (đầy đủ `nodes`, `pos`, `wires`, `groups`) + Thư mục User Presets dùng 1-click (hỗ trợ Replace/Chain).
  2. **Phase 4 (Task 4.7):** Xong Keyframe Node Graph + nối Transform Handles trên Program Monitor vào `Transform2D` + đóng gói các Video/Color Presets ➔ **Ẩn** `Effect Controls` và `Video Effects` cũ khỏi UI mặc định.
  3. **Phase 5 (Task 5.5):** Bổ sung Audio DSP Nodes & Edit-Point Transition Bridge trên Timeline ➔ **Xóa sạch** `PanelKind::Effects` và `PanelKind::EffectControls` cũ khỏi giao diện Darkworks.

### 39. Kiến Trúc Node & Preset Library (Search/Filter/Thumbnail Preview & Micro-Presets) (2026-11-25)
- **Bối cảnh:** Menu thả xuống `📂 Load Preset` không đủ khả năng tìm kiếm, lọc nhóm, xem trước và kéo-thả khi thay thế toàn bộ `Effects` + `Effect Controls` cũ (số lượng preset có thể lên 50–200+).
- **Quyết định kiến trúc đã duyệt:**
  1. **Node & Preset Library Browser:** Nâng cấp UI chọn Preset thành bảng thư viện có tìm kiếm real-time, lọc Category/Favorites, Quick Search `Tab` trên Canvas, và kéo thả trực tiếp vào Node Graph hoặc thẳng lên Clip Timeline.
  2. **Thumbnail Preview thống nhất:** Tái sử dụng ảnh mẫu `lumetri_presets::preview_image` chạy qua bộ thực thi Node Graph (cache ảnh nhỏ) để hiển thị thumbnail trực quan cho cả Color Presets cũ lẫn Custom User Presets.
  3. **Micro-Presets & Migrate `lumetri_presets`:** Đóng gói các hiệu ứng phổ biến thành Micro-Presets `.preset.json` (1–2 Atomic Nodes cài sẵn) và chuyển đổi dần `lumetri_presets` sang `ColorGrade` Presets kèm lộ trình nâng cấp Atomic Nodes tương ứng.

### 40. Nâng Cấp Toàn Diện Node Quang Học, Biên Dịch Nhánh Tuyến Tính Lên GPU (`gpu_plan.rs` + `OP_CHROMA_KEY = 37`) & Phân Tách Module Controls/Tests (2026-11-26)
- **Bối cảnh & RCA:**
  - `DistortLens` trước đó luôn hiện các ô `Radius` / `Center X/Y` dù chế độ không dùng (như `Posterize`, `ChromaticAberration`) và thiếu `Twirl`, `FindEdges`, `NoiseGrain`, `TurbulentDisplace`; `BlurSharpen` thiếu `RadialZoom`, `dimensions`, `repeat_edge`, `bokeh_sides`.
  - Khi chạy `Ultra Keyer` (hoặc `ultra_key` của FilmCraft) trên Timeline có video nền 4K (`V1`) + clip xanh lá (`V2`), máy lag nặng (~87% CPU) và còn viền xanh vì: (a) `GPU_EFFECTS` thiếu `"ultra_key"`, (b) `plan.rs` đẩy mọi clip có `node_graph` xuống `render_sequence` toàn khung trên CPU, và (c) `apply_keyer` dùng khoảng cách RGB tuyến tính không tách Y'CbCr hay choke biên.
- **Quyết định kiến trúc & Giải pháp:**
  1. **UI Controls lọc theo Mode & Tách Module:** Tách `crates/node-graph/src/node_controls_fx.rs` khỏi `node_controls.rs` để giữ mỗi file dưới trần 700 dòng (`438` & `319` dòng); chỉ vẽ tham số phù hợp với từng `DistortType` / `BlurType`, bổ sung đủ 4 chế độ Distort + 3 tham số Blur mở rộng.
  2. **GPU `OP_CHROMA_KEY = 37` (Y'CbCr Bt.709 + Anti-Fringe):** Triển khai `FxOp::ChromaKey` trong `crates/gpu/src/fx.wgsl`, `fx.rs` và `crates/render/src/gpufx.rs` (kèm mở khóa FilmCraft `"ultra_key"` khi không dùng `soften`/CC phụ), sử dụng tách chroma Y'CbCr Bt.709, 1px neighbor choke kẹp tọa độ biên (`clamp`), và kìm kênh màu chủ đạo (`spill_suppress`).
  3. **Biên dịch nhánh tuyến tính Node Graph lên GPU (`crates/render/src/nodes/gpu_plan.rs`):** Khi Media Clip mang đồ thị tuyến tính `VideoInput("Auto"|"Clip") -> Keyer/ColorGrade/BlurSharpen/DistortLens/CropMask/Transform2D -> Output`, `plan.rs` giữ nguyên ở `FramePlan::Layers` và chạy GPU `LayerFx`. Đồng thời tối ưu đường fallback CPU (`take_or_clone_texture`, bỏ `snapshot()` thừa khi không đọc nền, tái sử dụng `below_image` và `pool::recycle_f32`), tách `node_graph_tests.rs` khỏi `node_graph.rs` (416 dòng).
  4. **Phạm vi bàn giao phiên tiếp theo (`ultra-keyer-pro-upgrade.md`):** Chuyên tâm hoàn thiện `Ultra Keyer` bản Pro (`AlphaChannel` preview, `Pedestal/Highlight/Shadow/Choke/Soften/Spill Luma`, `LumaKey`, Eyedropper trên Monitor, và đưa nốt `Add as FX Clip` + mẫu 4 node `V1+Blend` lên GPU).

### 41. Kiến Trúc `NodeFxOp` Tách Riêng (`node_fx.wgsl` / `node_fx.rs`), `Ultra Keyer` Pro GPU 3-Pass, Shared Eyedropper & Domain-Split `gpu_plan.rs` (2026-11-27)
- **Bối cảnh & RCA:**
  - Cần nâng cấp `DarkNode::Keyer` lên chuẩn Pro (`ChromaKey` / `LumaKey`, `Pedestal/Highlight/Shadow`, `Choke/Soften`, `AlphaChannel`/`ColorChannel`, `Garbage Matte` / `Matte Out`) và tăng tốc 9 chế độ `DistortLens` + `RadialZoom` lên GPU mà không làm phình to `crates/gpu/src/fx.wgsl` hay `crates/render/src/gpufx.rs`.
  - Khi người dùng đặt clip nền xanh ở track `V2` và chọn `VideoInput("V2") -> Keyer` (hoặc thêm `Blend(Bg = "V1")`), cả khi `V1` có khoảng trống hoặc đang phát, `gpu_plan.rs` trả `None` khiến luồng rớt về CPU (`87% CPU`) vì trước đó `is_self_input` chưa nhận diện track chứa chính clip (`V2`) và `single_below_track_name` trả `None` lúc `V1` trống.
  - Ngoài ra `crates/render/src/nodes/gpu_plan.rs` chạm `705` dòng (vượt trần `700` dòng).
- **Quyết định kiến trúc & Giải pháp:**
  1. **Tách riêng module Compute Shader cho Node Graph:** Tạo `crates/gpu/src/node_fx.wgsl` (`503` dòng) + `crates/gpu/src/node_fx.rs` (`190` dòng), `crates/render/src/nodes/node_fx_ops.rs` (`536` dòng) + `node_fx_distort_cpu.rs` (`279` dòng), và `crates/node-graph-core/src/node_fx_types.rs` (`279` dòng) giữ cho `fx.wgsl`/`gpufx.rs`/`node.rs` gọn nhẹ dưới trần 700 dòng.
  2. **14 `NodeFxOp` GPU + Parity FilmCraft `ultra_key`:** Toàn bộ `ChromaKeyPro` (3-texture ping-pong `Matte -> Choke -> Finish` hỗ trợ `Choke`/`Soften`, `AlphaChannel`/`ColorChannel`, `Pedestal/Highlight/Shadow/Spill Luma`), `LumaKey`, `RadialZoom`, và 9 chế độ `DistortLens` (`Pixelate`, `ChromaticAberration`, `RippleWave`, `LensDistort`, `Twirl`, `FindEdges`, `NoiseGrain`, `TurbulentDisplace`) đều chạy GPU Compute Shader; đồng thời mở khóa GPU cho cả `Relaxed` / `Aggressive` / `Soften` / `Choke` / `AlphaChannel` / `ColorChannel` của FilmCraft `ultra_key`.
  3. **Shared Eyedropper & Self-Track / `Blend(V1)` / `FX Clip` GPU:** Tái sử dụng chung 1 bộ Eyedropper trên Program Monitor cho cả 4 node chọn màu (`Keyer` có auto-bypass khi hút màu, `ColorGrade`, `TextGenerator`, `ShapeGenerator`); cập nhật `gpu_plan.rs` + `plan.rs` hỗ trợ `VideoInput` trùng track của clip (`V2` trên track `V2`), `Blend(Bg = "V1" | "All Below")` kể cả khi `V1` trống, và `AdjustmentLayer` (`Add as FX Clip`) qua `PlanStep::Adjust`.
  4. **Tách Domain Helpers khỏi `gpu_plan.rs` (Cách 1):** Giữ `gpu_plan.rs` (`369` dòng) chỉ làm Graph Planner/Scheduler; chuyển logic biên dịch `FxOp` của từng node về `color.rs` (`383` dòng), `blur_distort.rs` (`450` dòng) và `transform_blend.rs` (`290` dòng) để dễ mở rộng cho các node GPU mới mà không lặp lại tình trạng file vượt 700 dòng.
