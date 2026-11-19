# AGENT SELF-IMPROVEMENT LOG (self-improve.md)
@project: DARKWORKS | @status: ACTIVE | @last_update: 2026-11-19

> Sổ ghi chép sự cố và tự cải thiện hành vi của Agent trong dự án **Darkworks**.
> Các quy tắc hành vi bắt buộc vĩnh viễn được quy định tại [.agents/rules/improve.md](file:///mnt/Data/darkworks/.agents/rules/improve.md).

---

## 🚨 DANH SÁCH LỖI HÀNH VI CỐT TỬ CẦN TRÁNH (ACTIVE)

### MISTAKE-001: Tự ý sửa code khi User báo lỗi (Vi phạm Rule #1)
- **Lỗi:** Khi User thông báo có lỗi hoặc trục trặc, AI vội vã dùng tool sửa code mà chưa giải thích và chưa xin phép.
- **Biện pháp:** Dừng lại ngay lập tức! Bắt buộc phân tích Root Cause Analysis (RCA) ➔ Giải thích "Logic gãy ở đâu? Tại sao xảy ra? Tác động thế nào?" ➔ Đề xuất giải pháp và **chờ Người Dùng đồng ý** mới được gõ code.

### MISTAKE-002: Lệnh terminal tương tác làm treo tiến trình
- **Lỗi:** Gõ các lệnh interactive chờ nhập liệu (interactive shell, REPL không ngắt) khiến session bị đóng băng.
- **Biện pháp:** Chỉ chạy các lệnh batch / non-interactive với timeout kiểm soát.

### MISTAKE-003: Cầm đèn chạy trước ô tô & Scope Creep
- **Lỗi:** Tự ý nhảy cóc sang code Node Graph hay AI khi Lõi Video Player và Timeline của Giai đoạn 1 chưa hoàn thiện và chưa được nghiệm thu.
- **Biện pháp:** Tuyệt đối bám sát lộ trình từng bước trong [phase-1-media-player.md](file:///mnt/Data/darkworks/.darkworks-ai/1-overview/project-managers/phase-1-media-player.md). Làm dứt điểm từng task.

### MISTAKE-004: Vi phạm luật Never Crash của Rust
- **Lỗi:** Dùng `.unwrap()`, `.expect()`, `panic!()` trong mã nguồn xử lý media/render để code cho nhanh.
- **Biện pháp:** Bắt buộc kích hoạt `#![deny(clippy::unwrap_used)]`. Mọi hàm phải trả về `Result<T, E>` và bọc lỗi bằng `thiserror`.

---

## 🟢 BÀI HỌC KINH NGHIỆM ĐÃ TÍCH LŨY (RESOLVED)
- **[2026-11-19] Tách biệt tư duy Desktop App vs Web Agency:** Đã loại bỏ hoàn toàn các cấu trúc agency (Account, Client-intake, Assembly) để tập trung 100% vào kỷ luật kỹ thuật của NLE Video Editor.
- **[2026-11-19] Chiến lược bảo toàn 93 Shader của FilmCraft:** Áp dụng chiến lược "Bọc ngoài & Thay ruột" thay vì xóa thẳng tay làm gãy compiler.
