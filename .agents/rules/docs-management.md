---
trigger: always_on
---
# DARKWORKS AI DOCUMENTATION MANAGEMENT (docs-management.md)
@target: Toàn bộ AI Agents | @trigger: /end_session hoặc bất kỳ cập nhật tài liệu nào

## 1. THIẾT QUÂN LUẬT KHÔNG RÁC (ZERO-TRASH DIRECTIVE)
Mọi tài liệu bắt buộc phải nằm gọn bên trong 3 ngăn kéo chuẩn của `.darkworks-ai/`:

```text
.darkworks-ai/
├── 1-overview/
│   ├── system_map.md           (Bản đồ kiến trúc tổng thể, Trạng thái Dự án, Roadmap)
│   └── project-managers/       (Kế hoạch và tiến độ chi tiết từng Phase)
├── 2-memory/
│   ├── decision-log.md         (Quyết định kiến trúc cốt lõi bất biến)
│   ├── checkpoint.md           (Bàn giao tiến độ ca kíp và Git branch cho phiên sau)
│   └── archive/                (Lưu trữ các quyết định và log cũ)
└── 3-modules/
    ├── schema/                 (Tài liệu kiến trúc data models, project.json)
    ├── media/                  (Tài liệu kiến trúc giải mã video, A/V sync)
    ├── node-graph/             (Tài liệu kiến trúc DAG Node Graph, WGSL Shaders)
    ├── canvas-renderer/        (Tài liệu kiến trúc Vello + Cosmic-text 2D)
    ├── automation/             (Tài liệu kiến trúc MCP Server, CLI commands)
    ├── ai-workers/             (Tài liệu kiến trúc Silero VAD, Whisper local)
    └── ui/                     (Tài liệu kiến trúc giao diện egui 4 phân vùng)
```

## 2. QUY TRÌNH GHI ĐÈ CHỐNG LỖI (ANTI-DUPLICATION PROTOCOL)
- Trước khi cập nhật bất kỳ tài liệu nào (`system_map.md`, `decision-log.md`...), AI BẮT BUỘC phải đọc nội dung hiện tại của file đó.
- **Nghiêm Cấm:** Tự ý tạo file mới trùng lặp nội dung với file trung tâm. Cập nhật trực tiếp nội dung vào các file đã có sẵn.

## 3. LUẬT NGUYÊN TẮC /END_SESSION
Khi gọi luồng `/end_session`, AI cập nhật tối đa 4 điểm chạm:
1. `1-overview/system_map.md`: Cập nhật trạng thái modules (🔴 ➔ 🟡 ➔ 🟢) và Recent Logs.
2. `2-memory/decision-log.md`: Ghi nhận các quyết định kiến trúc mới được duyệt.
3. `2-memory/checkpoint.md`: Bàn giao tiến độ đang làm dở và Git branch cho ca sau.
4. `3-modules/[tên-crate]/architecture.md`: Cập nhật kiến trúc nếu crate có thay đổi lớn.

## 4. HẠN CHẾ SINH FILE RÁC NGOÀI GỐC (ANTI-CLUTTER DIRECTIVE)
- Tuyệt đối KHÔNG tự ý tạo mới các file Markdown (`.md`) ở thư mục gốc của project hoặc ngoài 3 ngăn kéo của `.darkworks-ai/`.
- Nếu có nhu cầu tạo file mới ngoài cấu trúc 3 ngăn kéo, bắt buộc phải hỏi ý kiến Người Dùng trước.
