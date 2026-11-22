# MODULE KIẾN TRÚC: SCHEMA & DATA MODELS (schema/architecture.md)
@crate: `crates/schema` | @layer: L0 / Core Data Model | @status: 🟢 Hoàn thành Phase 1 + 2

## 1. VAI TRÒ & TRÁCH NHIỆM
Crate này định nghĩa toàn bộ mô hình dữ liệu cốt lõi của Darkworks. Cấu trúc dữ liệu cực nhẹ, thuần văn bản, hỗ trợ tuần tự hóa (serde JSON) để lưu ra `project.json` và `.preset.json`, phục vụ Hot-Reload và AI CLI.

## 2. PHÂN TÁCH SUB-MODULES (< 700 DÒNG)
- **[time.rs](file:///mnt/Data/darkworks/crates/schema/src/time.rs):**
  - Đơn vị thời gian nguyên thủy `Tick` (`TICKS_PER_SECOND = 254_016_000_000`) bảo đảm độ chính xác từng khung hình.
  - Struct `TimeRange` và các phép toán giao nhau, chứa nhau, timecode formatting.
- **[clip.rs](file:///mnt/Data/darkworks/crates/schema/src/clip.rs):**
  - `ClipSlice`: Đại diện cho một đoạn cắt media trên Timeline (`timeline_range`, `source_range`, `speed`, `is_muted`).
  - `Track`, `TrackKind` (Video, Audio, Canvas).
  - Thuật toán cắt dồn nguyên tử: `split_clip_at`, `split_at`, `clear_range`, `shift_from`, `ripple_delete_clip`, `trim_clip_in/out`.
- **[project.rs](file:///mnt/Data/darkworks/crates/schema/src/project.rs):**
  - Struct `Project`, `ProjectSettings`.
  - Mốc `mark_in` và `mark_out` chuẩn Lightworks.
  - Thuật toán `active_range`, `extract_range` (Ripple Delete vùng chọn), `lift_range` (Xóa rỗng giữ chỗ).
- **[error.rs](file:///mnt/Data/darkworks/crates/schema/src/error.rs):**
  - Định nghĩa `SchemaError` an toàn không panic.
