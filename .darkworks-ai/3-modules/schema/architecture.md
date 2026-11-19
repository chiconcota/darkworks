# MODULE KIẾN TRÚC: SCHEMA & DATA MODELS (schema/architecture.md)
@crate: `crates/schema` (hoặc `core-schema`) | @layer: L0 / Core Data Model

## 1. VAI TRÒ & TRÁCH NHIỆM
Crate này chịu trách nhiệm định nghĩa toàn bộ cấu trúc dữ liệu cốt lõi của Darkworks. Cấu trúc dữ liệu phải cực nhẹ, có thể tuần tự hóa (serde JSON) để lưu ra file `project.json` và `.preset.json`, phục vụ Hot-Reload và AI CLI.

## 2. CẤU TRÚC DỮ LIỆU CHÍNH
- `Project`: Đại diện cho toàn bộ dự án (tracks, clips, sequence settings, fps).
- `ClipSlice`: Thông tin đoạn cắt video/audio trên Timeline (`source_in`, `source_out`, `timeline_start`, `duration_ticks`).
- `Track`: Phân loại Track V1, V2 (Video/Canvas) và A1, A2 (Audio).
- `Tick`: Đơn vị thời gian nguyên thủy chống sai số (`filmcraft_time::Tick`).
- `NodeGraphSchema`: Định nghĩa đồ thị Node cho từng Clip (`nodes`, `connections`, `exposed_inputs`).
- `Preset`: Định nghĩa cấu trúc lưu cụm Node thành file `.preset.json`.
