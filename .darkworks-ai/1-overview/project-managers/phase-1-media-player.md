# KẾ HOẠCH DỰ ÁN: GIAI ĐOẠN 1 (phase-1-media-player.md)
@project: DARKWORKS | @phase: 1 | @status: 🟢 COMPLETED
@goal: Load video 1 tiếng, phát mượt mà 60fps, âm thanh đồng bộ và khóa cứng trần RAM < 2GB.

---

## 1. MỤC TIÊU CỐT LÕI (CORE DELIVERABLES)
1. **Khởi tạo cửa sổ ứng dụng 4 khu vực (`crates/ui`):** Dùng `egui + eframe + wgpu`.
2. **Khảo sát & Tích hợp Lõi Media (`crates/media`):** Tận dụng bộ giải mã phần cứng VA-API từ `reference/filmcraft/crates/media` và `platform`.
3. **Cơ chế Ring-Buffer LRU Cache:** Giới hạn cứng bộ nhớ đệm khung hình `< 2GB RAM (2048MB)`. Không bao giờ tích trữ frame tràn lan như Blender VSE.
4. **Đồng bộ Âm thanh & Hình ảnh (A/V Sync):** Dùng `cpal` phát âm thanh chuẩn qua PipeWire/ALSA, hình ảnh bám theo nhịp quét màn hình.

---

## 2. CHECKLIST CÁC ĐẦU VIỆC CỤ THỂ

- [x] **Task 1.1:** Khởi tạo Cargo Workspace gốc của Darkworks, cấu hình lints `#![deny(clippy::unwrap_used, clippy::expect_used)]` và đồng bộ GitHub.
- [x] **Task 1.2:** Dựng Crate `crates/ui` + `apps/darkworks` mở cửa sổ `egui + wgpu` có 4 phân vùng cơ bản (Source Bin, Viewport, Inspector, Timeline rỗng) kèm font DejaVu Sans tiếng Việt và ký tự Symbol chuẩn.
- [x] **Task 1.3:** Khởi tạo `crates/media`, tách module phát hiện và giải mã video phần cứng OS Native (Linux VA-API / NVDEC).
- [x] **Task 1.4:** Triển khai Ring-Buffer LRU cache (`FrameCache`), giới hạn cứng dung lượng RAM < 512MB (Trần cứng 2GB).
- [x] **Task 1.5:** Render khung hình video giải mã lên Viewport thông qua `VideoFrame` -> RGBA8 GPU texture.
- [x] **Task 1.6:** Tích hợp `cpal` phát audio track đồng bộ với khung hình video khi bấm Spacebar (Play/Pause).

---

## 3. TIÊU CHUẨN NGHIỆM THU GIAI ĐOẠN 1 (DEFINITION OF DONE)
- [x] Mở ứng dụng Darkworks phát được 1 file video MP4 dài 1 tiếng (Tích hợp `tropical_paradise_1hour.mp4`).
- [x] Mức ăn RAM đo được không vượt quá **2GB (2048MB)**.
- [x] Bấm Spacebar để Play / Pause tức thì, âm thanh khớp khẩu hình tuyệt đối.
- [x] Tua thanh Playhead (Scrubbing) phản hồi nhanh, không bị đơ giật giao diện.
- [x] Mã nguồn vượt qua kiểm tra `cargo check` và `cargo clippy` không có cảnh báo.
