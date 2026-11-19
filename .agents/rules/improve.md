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

---

## 📝 NHẬT KÝ BÀI HỌC KINH NGHIỆM ĐÃ TÍCH LŨY (LESSONS LEARNED)

- **[2026-11-19] Bẫy nhầm lẫn giữa Desktop App và Web Agency:**
  - *Bài học:* Darkworks là ứng dụng phần mềm máy tính (Desktop NLE Video Editor) bằng Rust, không phải dự án làm web dịch vụ. Lược bỏ hoàn toàn các tư duy Agency (Account, Client Brief, Nghiệm thu web).
- **[2026-11-19] Bẫy xóa sạch Effect của FilmCraft:**
  - *Bài học:* Không xóa thẳng tay các file shader và struct của FilmCraft làm gãy compiler. Áp dụng chiến lược "Bọc ngoài & Thay ruột" để tái sử dụng toàn bộ 93 Shader WGSL có sẵn của họ thành các Node nguyên tử trong hệ thống Node Graph của Darkworks.
