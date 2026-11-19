# MODULE KIẾN TRÚC: AUTOMATION & MCP SERVER (automation/architecture.md)
@crate: `crates/automation` (hoặc `filmcraft-cli` + MCP) | @layer: L5 / Agent Interface & IPC

## 1. VAI TRÒ & TRÁCH NHIỆM
Mở cổng giao tiếp cho các tác tử AI ngoài dòng lệnh (Antigravity CLI, Claude Code...) hoặc các script tự động hóa có thể điều khiển trực tiếp phần mềm thông qua chuẩn MCP (Model Context Protocol).

## 2. KIẾN TRÚC & TẬP LỆNH CHUẨN
- **Cổng MCP Server (`rmcp`):** Khởi chạy luồng ngầm mở Unix Socket (`/tmp/darkworks.sock`) hoặc stdin/stdout.
- **Tập Lệnh Exposed (Tools):**
  - `get_timeline_state()`: Đọc danh sách clip, track, thời lượng.
  - `get_transcript(clip_id)`: Lấy văn bản kèm timestamp của clip.
  - `cut_and_delete(ranges)`: Băm clip và xóa đoạn theo khoảng thời gian tick.
  - `insert_preset(clip_id, preset_json)`: Bơm sơ đồ Node hiệu ứng mới vào clip.
  - `render_headless(project_path, output_mp4)`: Chạy render ngầm không bật UI.
- **Command Dispatcher:** Tận dụng hệ thống `Session::execute(id, params)` có sẵn của FilmCraft để đảm bảo mọi lệnh AI đều được ghi vào lịch sử Undo/Redo an toàn.
