---
trigger: always_on
---
# NHẬT KÝ TỰ CẢI THIỆN & QUY TẮC PHÒNG TRÁNH LỖI (improve.md)
@target: Toàn bộ AI Agents | @status: ALWAYS ACTIVE

> Tập tin này lưu giữ các quy tắc tự sửa lỗi, phản xạ hành vi bắt buộc và bài học xương máu.  
> File được tự động nạp vào não AI trong mọi phiên làm việc để đảm bảo không bao giờ lặp lại lỗi cũ.

---

## 🚨 NHỮNG THIẾT QUÂN LUẬT HÀNH VI CỐT LÕI (MANDATORY BEHAVIOR)

### 1. Luật Sắt Báo Lỗi: Hỏi Ý Kiến Trước Khi Đụng Vào Code
- **Quy tắc tuyệt đối:** Khi người dùng báo lỗi, **TRƯỚC KHI ĐỤNG VÀO CODE THÌ PHẢI HỎI Ý KIẾN NGƯỜI DÙNG. TUYỆT ĐỐI KHÔNG TỰ Ý SỬA!**
- **Quy trình chuẩn 3 bước:**
  1. Lắng nghe và phân tích nguyên nhân gốc rễ (Root Cause Analysis - RCA).
  2. Giải thích rõ: "Logic gãy ở đâu? Tại sao xảy ra? Tác động lan tỏa thế nào?".
  3. Trình bày phương án sửa bằng lời hoặc pseudocode, **chờ người dùng đồng ý mới được gõ code**.

### 2. Kỷ Luật Biên Dịch Rust & Luật "Never Crash" (FilmCraft Standard)
- **Cấm `.unwrap()`, `.expect()`, `panic!()`:** Trong toàn bộ codebase Rust của Darkworks, mọi hàm phải trả về `Result<T, E>` hoặc xử lý an toàn bằng `match` / `if let`. Tuyệt đối không để ứng dụng crash văng ra ngoài khi người dùng đang dựng video.
- **Tuân thủ phân tầng Cargo Layers:** Không được import chéo hoặc phụ thuộc vòng giữa các crate. Kiểm tra bằng `cargo xtask layers`.

### 3. Kỷ Luật Terminal & Tiến Trình Nền
- **Cấm lệnh tương tác chặn shell (Interactive Blockers):** Tuyệt đối không gõ các lệnh terminal chờ nhập liệu tương tác (như lệnh mở shell interactive, lệnh chạy server không ngắt, v.v.) làm treo session làm việc.
- **Tiến trình nặng:** Luôn chạy nền có timeout hoặc kiểm soát qua luồng nền (Background Job).

### 4. Kỷ Luật Ngân Sách Token & Không Cầm Đèn Chạy Trước Ô Tô
- **Chống Loop vô bổ:** Khi một giải pháp thất bại, dừng lại ngay để phân tích, cấm lặp lại hướng sửa lỗi cũ.
- **Không nhảy cóc giai đoạn:** Làm xong và nghiệm thu dứt điểm từng bước theo Roadmap của `system_map.md` (Player ➔ Timeline ➔ Node Graph ➔ Canvas ➔ AI). Không code trước những thứ chưa đến lượt.

### 5. Kỷ Luật Giới Hạn Dòng Code & Báo Cáo Người Dùng (< 700 Dòng)
- **Mức trần chuẩn mực & báo động:** **700 dòng code** mỗi file (`.rs`).
- **QUY TẮC BÁO CÁO BẮT BUỘC:** Khi file chạm mốc 700 dòng:
  - Báo cáo rõ ràng lên Người Dùng để Người Dùng quyết định phương án (Người Dùng có thể cho phép ngoại lệ nếu thấy hợp lý).
  - **CẤM TUYỆT ĐỐI:** Tự ý tối ưu hóa, gộp hàm hoặc tự tiện cắt xén tách file khi chưa xin ý kiến.
- Kiểm toán tự động qua tool `.agents/harness/check_file_size.py`.

---

## 📝 NHẬT KÝ BÀI HỌC KINH NGHIỆM ĐÃ TÍCH LŨY (LESSONS LEARNED)

- **[2026-11-19] Bẫy Font chữ mặc định của egui (Lỗi Tofu ô vuông tiếng Việt & Emoji):**
  - *Bài học:* Font mặc định của egui (`Ubuntu-Light`) chỉ hỗ trợ Latin cơ bản, không có tiếng Việt mở rộng và không có emoji màu. Phải nạp font chuẩn (`DejaVuSans.ttf`) vào `egui::FontDefinitions` và dùng Symbol Unicode monochrome thay cho emoji màu.
- **[2026-11-19] Bẫy nhầm lẫn giữa Desktop App và Web Agency:**
  - *Bài học:* Darkworks là ứng dụng phần mềm máy tính (Desktop NLE Video Editor) bằng Rust, không phải dự án làm web dịch vụ. Lược bỏ hoàn toàn các tư duy Agency (Account, Client Brief, Nghiệm thu web).
- **[2026-11-19] Bẫy xóa sạch Effect của FilmCraft:**
  - *Bài học:* Không xóa thẳng tay các file shader và struct của FilmCraft làm gãy compiler. Áp dụng chiến lược "Bọc ngoài & Thay ruột" để tái sử dụng toàn bộ 93 Shader WGSL có sẵn của họ thành các Node nguyên tử trong hệ thống Node Graph của Darkworks.
- **[2026-11-20] Bẫy tự chế tạo lại bánh xe (Reinventing the Wheel) thay vì tận dụng nền tảng FilmCraft:**
  - *Bài học:* Darkworks là bản nâng cấp NLE dựa trên nền tảng FilmCraft (thay Linear Effect Stack bằng Node Graph và cấy ghép Timeline Lightworks). Tuyệt đối không sa đà vào việc tự viết lại toàn bộ Player, Hardware decode hay Audio sync từ con số 0. Kế thừa triệt để các crate nền tảng vững chắc của FilmCraft để dồn toàn lực vào giá trị cốt lõi: **Đồ thị DAG Node Graph (`egui_snarl`)**.
- **[2026-11-20] Bẫy ép chuẩn trần < 700 dòng lên mã nguồn Upstream / Vendor:**
  - *Bài học:* Mã nguồn nền tảng upstream thường chứa những file lớn (> 700 dòng) đã được kiểm thử ổn định. Việc đập nhỏ file gốc của họ là phản tác dụng, gây lỗi ngầm và mất khả năng merge upstream. Cần cơ chế ngoại lệ rõ ràng (`audit.py` Vendor Allowlist) để chỉ siết trần 700 dòng đối với mã nguồn do Darkworks phát triển mới.
- **[2026-11-22] Nhận diện thương hiệu & Đồng bộ Icon Hệ thống Desktop:**
  - *Bài học:* Khi kế thừa nền tảng từ dự án upstream, ngoài việc đổi tên chuỗi code, cần rà soát đồng bộ các file từ điển dịch thuật (`i18n/*.tsv`), hộp thoại hệ thống native (`rfd`), tên project demo mặc định, và trọn bộ icon app (`app_icon()` nạp `.png` chuẩn, `dev_icon.rs` và `~/.local/share/icons/hicolor/`) để thanh Taskbar/Dock OS hiển thị đúng nhận diện thương hiệu Darkworks.
- **[2026-11-22] Bẫy phân tầng Simple/Advanced giả tạo & Hợp nhất Shader thành Atomic Nodes:**
  - *Bài học:* Không chia bản rút gọn (Simple) rồi chắp vá bản nâng cao (Advanced) gây phân mảnh kiến trúc và sinh code thừa thãi. Không xé vụn 93 shader thành 93 node lẻ gây bãi rác Spaghetti trên Canvas. Cung cấp ngay bản đầy đủ (Full Production-Ready) 100% cho 18 Node nguyên tử đa năng gom trọn các shader cùng họ vào dropdown/toggle (ví dụ: Invert & B&W nằm trong ColorGrade, Flip H/V nằm trong Transform2D), giúp kiến trúc DAG cực kỳ tinh gọn, hiệu năng cao và giải quyết triệt để các bài toán thực tế.
- **[2026-11-24] Bẫy thư viện bên thứ ba tự ý xóa đối tượng ngầm không qua Undo (egui-snarl Delete bypass):**
  - *Bài học:* Thư viện bên thứ ba (`egui-snarl`) có thể cài sẵn logic xóa phần tử tự phát khi nhấn `Delete` / `Backspace` bên trong vòng lặp render mà không có hook callback. Điều này khiến đối tượng bị xóa sổ khỏi bộ nhớ trước khi hệ thống ứng dụng kịp chụp snapshot `Undo`, làm hỏng ngăn xếp lịch sử và khiến nút Undo bị xám. Giải pháp: Luôn rà soát và vô hiệu hóa các đoạn code xóa ngầm trong thư viện vendored, trao 100% quyền kiểm soát vòng đời xóa cho Darkworks (gọi `push_undo()`), đồng thời khóa cứng ID phân cấp của widget (`SnarlWidget::new().id(...)`) để dữ liệu vùng chọn luôn nhất quán.
- **[2026-11-25] Bẫy `Layout::left_to_right` của `egui_snarl` trong `show_body` (Lỗi widget đè lên nhau trên Node):**
  - *Bài học:* `vendor/egui-snarl` khởi tạo `body_ui` với `Layout::left_to_right(Align::Min)`. Nếu dùng `ui.scope(...)` bên trong `show_body` thì mọi `Grid`, `Checkbox`, `CollapsingHeader` phía dưới sẽ bị xếp ngang nối đuôi nhau và đè chồng lên dòng `Color:` khi chạm `set_max_width`. Giải pháp: Luôn bọc gốc `render_node_body` bằng `ui.vertical(...)` để khóa bố cục dọc an toàn.
