---
name: verify-feature
description: "Kiểm thử chấp nhận nghiệp vụ theo từng tính năng người dùng (Feature Acceptance Verification) bằng cách lái ứng dụng thực tế và đối chiếu kịch bản trong .darkworks-ai/1-overview/features/"
---

# KỸ NĂNG KIỂM THỬ NGHIỆP VỤ TÍNH NĂNG: /verify-feature
@target: Darkworks NLE Video Editor | @level: Feature Acceptance Gate (Tầng 2)

> **Phân biệt quan trọng:**  
> - `/e2e_test`: Cổng kỹ thuật toàn cục (Technical Gate - compile, clippy, leak RAM, trần file, smoke test).  
> - `/verify-feature`: Cổng nghiệp vụ tính năng (Feature Acceptance Gate - lái app theo hành vi người dùng, bắt bằng chứng đúng chuẩn).

---

## 1. QUY TRÌNH 4 BƯỚC KIỂM THỬ TÍNH NĂNG

### Bước 1: Tra cứu kịch bản tính năng (Feature Contract)
Đọc file đặc tả tính năng tương ứng trong kho lưu trữ nghiệp vụ:
- Thư mục: `.darkworks-ai/1-overview/features/`
- Danh mục tra cứu: [features/README.md](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/features/README.md)
- Ví dụ:
  - Lightworks Editing ➔ `darkworks_lightworks_editing.md`
  - Media Playback VA-API ➔ `filmcraft_media_playback.md`
  - Node Graph DAG ➔ `darkworks_node_graph.md`
  - Audio Reactivity ➔ `darkworks_audio_reactivity.md`
  - Auto Cut Silence ➔ `darkworks_ai_vad_silence.md`

### Bước 2: Chạy bộ điều khiển tự động (Harness Driver)
Chạy script lái app theo kịch bản:
```bash
python3 .agents/harness/drive_feature.py --feature <tên-feature>
```
Script sẽ khởi chạy `darkworks-cli` hoặc tương tác qua giao thức control socket để mô phỏng chính xác chuỗi thao tác của người dùng.

### Bước 3: Thu thập bằng chứng (Capture Evidence)
Kiểm tra kết quả trả về từ driver:
- Khung hình render kết quả (`.png`).
- Dữ liệu trạng thái sequence / clips / markers (`.json`).
- Mã thoát (exit code 0) và thời gian phản hồi.

### Bước 4: Đối chiếu tiêu chuẩn nghiệm thu & Báo cáo
Đối chiếu với phần **Gotchas** và **Sub-features** trong file đặc tả để kết luận:
- `PASS`: Tính năng hoạt động mượt mà, đúng mong đợi người dùng.
- `FAIL`: Logic gãy hoặc chưa hỗ trợ, trình bày RCA và phương án xử lý.
