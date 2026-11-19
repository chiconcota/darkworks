---
description: Quy trình 5 bước điều tra, chẩn đoán nguyên nhân gốc rễ (RCA) và sửa lỗi ngoại khoa trong hệ thống Darkworks
---

### Step 0 — Memory Check (Nạp ký ức)
- Đọc `.darkworks-ai/2-memory/checkpoint.md` và `.darkworks-ai/2-memory/decision-log.md`.
- Xác định Crate lỗi ➔ Chỉ đọc tài liệu trong `.darkworks-ai/3-modules/[tên-crate]/`.

### Step 1 — Comprehend & Clarify (CHƯA ĐƯỢC CHẠM VÀO CODE)
- Trả lời 2 câu hỏi: "Logic gãy ở đâu?" và "Tại sao nó xảy ra?".
- Nếu thiếu dữ liệu, dừng lại và đề xuất thêm log (ví dụ `tracing::debug!`).

### Step 2 — Isolate & Impact Assessment
- Xác định: File, Struct/Function, Line number.
- Liệt kê tác động lan tỏa (Side Effects / Blast Radius) đến các crate khác trong Cargo Workspace.

### Step 3 — Surgical Solution Proposal
- Trình bày giải pháp bằng ngôn ngữ tự nhiên hoặc pseudocode.
- **BẮT BUỘC:** Chờ Người Dùng đồng ý mới được gõ code!

### Step 4 — Execution & Verification
- Sửa chính xác khối mã lỗi (targeted diff), không rewrite cả file.
- Tuân thủ luật Never Crash (`Result<T, E>`), không dùng `.unwrap()`.
- Chạy `cargo check --workspace --all-targets` và `cargo test -p [crate]`.
- Nâng PATCH version nếu sửa crate.
