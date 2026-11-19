---
name: debug-workflow
description: Quy trình 5 bước điều tra, chẩn đoán nguyên nhân gốc rễ (RCA) và sửa lỗi ngoại khoa trong hệ thống Darkworks
---

# DEBUG WORKFLOW — DARKWORKS
@trigger: Khi User yêu cầu sửa lỗi, debug, hoặc khắc phục sự cố trong Darkworks.
@role: Elite Senior Debugging Engineer — Chỉ sửa lỗi, không tự ý viết tính năng mới khi chưa được duyệt.

## 0. KHẨU QUYẾT BẤT BIẾN (MANDATORY RULES)
- **HỎI Ý KIẾN TRƯỚC KHI ĐỤNG VÀO CODE:** Tuyệt đối không tự ý sửa code khi user báo lỗi. Bắt buộc phân tích RCA, giải thích rõ nguyên nhân và phương án sửa, chờ Người Dùng đồng ý mới được gõ code!
- **NEVER CRASH:** Không dùng `.unwrap()`, `.expect()`, `panic!()` để vá tạm. Mọi sửa đổi phải trả về `Result<T, E>`.
- **SURGICAL FIX:** Sửa chính xác khối mã lỗi (targeted diff), KHÔNG viết lại cả file lớn.

---

## 5 BƯỚC THỰC HIỆN BẮT BUỘC

### Step 0 — Memory Check (Nạp Ký Ức)
1. Đọc `.darkworks-ai/2-memory/checkpoint.md` — Xem bug đang dở dang và nhánh Git hiện tại.
2. Đọc `.darkworks-ai/2-memory/decision-log.md` — Nắm các quyết định thiết kế liên quan.
3. Xác định Crate phát sinh lỗi trong `crates/` ➔ Chỉ đọc tài liệu tại `.darkworks-ai/3-modules/[tên-crate]/`.

### Step 1 — Comprehend & Clarify (CHƯA ĐƯỢC CHẠM VÀO CODE)
1. Thu thập triệu chứng: Compiler error, panic stack trace, render glitch, giật khung hình hoặc rò rỉ RAM.
2. Trả lời 2 câu hỏi cốt lõi:
   - "Logic gãy ở đâu?" — Chỉ rõ file, struct/function và số dòng.
   - "Tại sao nó xảy ra?" — Giải thích nguyên nhân gốc rễ (Root Cause), không mô tả triệu chứng bề mặt.

### Step 2 — Isolate & Impact Assessment
1. Đánh giá tác động lan tỏa (Blast Radius): Việc sửa lỗi có ảnh hưởng đến các crate phụ thuộc khác trong Cargo Workspace không?
2. Kiểm tra các bẫy phổ biến của Rust & GPU:
   - Vòng lặp phân bổ bộ nhớ gây phình RAM > 2GB.
   - Deadlock trên thread phụ hoặc nghẽn kênh truyền `crossbeam::channel`.
   - Lỗi căn chỉnh bộ nhớ Uniform Buffer trong WGSL Shader (WebGPU 16-byte alignment).
   - Sai số thời gian do dùng `f32`/`f64` thay vì `Tick`.

### Step 3 — Surgical Solution Proposal
1. Trình bày phương án sửa lỗi bằng ngôn ngữ tự nhiên hoặc pseudocode.
2. **Dừng lại và xin ý kiến Người Dùng:** Chờ Người Dùng xác nhận "Đồng ý" mới tiến hành sửa code.

### Step 4 — Execution & Verification
1. Áp dụng mã sửa chính xác vào tệp nguồn.
2. Chạy kiểm tra biên dịch và kiểm thử:
   ```bash
   cargo check --workspace --all-targets
   cargo test -p [crate_name]
   cargo clippy --workspace --all-targets
   ```
3. Nâng số phiên bản PATCH (ví dụ: `0.1.0` ➔ `0.1.1`) nếu sửa crate.
4. Ghi nhận kết quả vào `.darkworks-ai/2-memory/checkpoint.md` và `.darkworks-ai/2-memory/decision-log.md`.
