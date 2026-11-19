# SỔ BÀN GIAO TIẾN ĐỘ CA KÍP: DARKWORKS (checkpoint.md)
@project: DARKWORKS | @phòng_ban: Toàn xưởng | @status: Hoàn thành 100% Giai đoạn 1 & Sẵn sàng Giai đoạn 2 (Timeline NLE Core)
@session_end: 2026-11-19 | @next_session_focus: Giai đoạn 2 - Timeline NLE Core & Cắt ghép Clips

> Sổ ghi chép bàn giao công việc giữa các phiên làm việc của AI.  
> Khi mở phiên mới (/start_session), nhân viên đọc file này để nắm tiến độ dở dang trong 3 giây.

---

## 1. THÔNG TIN MÔI TRƯỜNG & TRẠNG THÁI HIỆN TẠI
- **Môi trường:** Linux x86_64, Rust toolchain, Git, Vulkan / VA-API.
- **Tình trạng:** Đã hoàn thành 100% Giai đoạn 1 (Media Player Engine & Viewport 60 FPS).
- **Video Test Nghiệm Thu:** `assets/samples/tropical_paradise_1hour.mp4` (Video 1 tiếng thực tế, 254.7MB, 108.001 frames).
- **Mã nguồn tham khảo:** Đã clone kho nguyên bản FilmCraft tại [reference/filmcraft](file:///mnt/Data/darkworks/reference/filmcraft) và cách ly an toàn trong [.gitignore](file:///mnt/Data/darkworks/.gitignore).

---

## 2. DANH SÁCH FILE ĐÃ THIẾT LẬP / CẬP NHẬT TRONG PHIÊN
- **Kiến trúc & Lõi Media (`crates/media`):**
  - [ffmpeg_source.rs](file:///mnt/Data/darkworks/crates/media/src/ffmpeg_source.rs): Triển khai `FfmpegMediaSource` với Background Prefetch Worker Thread qua streaming pipe FFmpeg độc lập, giải mã trước ~45 frames vào `FrameCache` (2GB cap).
  - [source.rs](file:///mnt/Data/darkworks/crates/media/src/source.rs): Bổ sung `notify_playhead` và `seek` vào `MediaSource` trait.
  - [cache.rs](file:///mnt/Data/darkworks/crates/media/src/cache.rs): Quản lý FrameCache LRU phân tầng 2 cấp (Mức 1: 1024MB, Mức trần: 2048MB), dọn dẹp vi mô $O(1)$ không giữ khóa Mutex.
  - [audio_player.rs](file:///mnt/Data/darkworks/crates/media/src/audio_player.rs): Đồng bộ âm thanh A/V Master Clock qua `cpal` và Monotonic Audio Feed Head.
- **Giao diện Người Dùng (`crates/ui`):**
  - [app.rs](file:///mnt/Data/darkworks/crates/ui/src/app.rs): Nạp FrameCache dùng chung, kết nối `active_source.notify_playhead(playhead)` và xử lý nạp asset khi click vào Bin.
  - [menu_bar.rs](file:///mnt/Data/darkworks/crates/ui/src/panels/menu_bar.rs): Hiển thị `RAM: XMB / YMB (2GB Cap)` với chỉ báo màu sắc thời gian thực.
  - [source_bin.rs](file:///mnt/Data/darkworks/crates/ui/src/panels/source_bin.rs): Hỗ trợ click vào Clip danh sách để switch `active_source`.
- **Tài liệu & Bộ nhớ:**
  - [phase-1-media-player.md](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/project-managers/phase-1-media-player.md): Đạt 100% DoD.
  - [system_map.md](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/system_map.md): Cập nhật trạng thái module và nhật ký thay đổi.
  - [decision-log.md](file:///mnt/Data/darkworks/.darkworks-ai/2-memory/decision-log.md): Bổ sung Quyết định #13, #14 và #15 về giải mã video 1 tiếng, luồng prefetch ngầm 60 FPS, và phân tầng cache 2 cấp (1GB/2GB).
  - [architecture.md](file:///mnt/Data/darkworks/.darkworks-ai/3-modules/media/architecture.md): Cập nhật kiến trúc FrameCache $O(1)$ và FfmpegMediaSource.

---

## 3. CÁC VẤN ĐỀ ĐÃ GIẢI QUYẾT & BÀI HỌC KINH NGHIỆM (RESOLVED)
1. **Khắc phục lỗi tụt FPS từ 5 - 11 FPS về 60 FPS mượt mà:** Chuyển việc giải mã sang `run_prefetch_loop` ngầm đọc streaming pipe tốc độ cao (~53x real-time), UI thread chỉ đọc frame từ RAM (<0.001ms).
2. **Khắc phục lỗi tụt 10 FPS khi chạm trần RAM:**
   - *Nguyên nhân:* Xả lũ hàng loạt 300MB và sắp xếp 2.000+ key trong khi giữ khóa `Mutex`.
   - *Khắc phục:* Chuyển sang thuật toán dọn dẹp vi mô $O(1)$ qua `VecDeque` và giải phóng vùng nhớ byte hoàn toàn NGOÀI khóa `Mutex`.
3. **Phân tầng ngân sách 2 cấp độ rõ ràng:** Mức 1 là 1GB (1024MB) cho video cache; Mức trần cứng là 2GB (2048MB) cho toàn bộ ứng dụng Darkworks (đủ chỗ cho AI Worker Whisper/VAD và Audio).
4. **Tuân thủ quy chuẩn:** Toàn bộ 23 file mã nguồn Rust đều đạt chuẩn an toàn < 700 dòng code, không unwrap/panic, 0 clippy warnings, vượt qua toàn bộ 7 cổng kiểm toán của [run_e2e_audit.py](file:///mnt/Data/darkworks/.agents/harness/run_e2e_audit.py).

---

## 4. KẾ HOẠCH BÀN GIAO CA TIẾP THEO (NEXT ACTIONS CHECKLIST)
- [x] **Giai đoạn 1 (Media Player):** Hoàn tất 100%.
- [ ] **Giai đoạn 2 (Timeline NLE Core):**
  - [ ] Bổ sung cấu trúc dữ liệu đa track (Multi-track Video/Audio tracks) trong `crates/schema`.
  - [ ] Thao tác cắt Clip (Razor / Split) theo phím tắt `C` hoặc icon kéo.
  - [ ] Thao tác kéo trượt Clip trên Timeline, xóa dồn (Ripple Delete) bằng phím `Delete` / `Backspace`.
  - [ ] Cơ chế File Watcher hot-reload `project.json`.
