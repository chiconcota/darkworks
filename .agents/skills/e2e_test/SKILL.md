---
name: e2e_test
description: "Chạy quy trình kiểm thử End-to-End (E2E) tự động và trực quan cho Darkworks qua run_e2e_audit.py"
---

# KỸ NĂNG KIỂM THỬ ĐẦU-CUỐI: /e2e_test

Khi người dùng yêu cầu kiểm thử tính năng mới hoặc chạy E2E audit, thực hiện theo các bước sau:

## 1. KÍCH HOẠT MASTER HARNESS TỰ ĐỘNG
Chạy lệnh kiểm toán 7 cổng chất lượng:
```bash
python3 .agents/harness/run_e2e_audit.py
```

## 2. ĐÁNH GIÁ KẾT QUẢ 7 CỔNG
1. **VA-API Hardware Node:** Xác nhận nhận diện `/dev/dri/renderD128`.
2. **Cargo Check Workspace:** Xác nhận code liên kết không lỗi type.
3. **Workspace Test Suite:** Xác nhận 100% unit tests & integration tests pass.
4. **Clippy Zero-Warning Audit:** Xác nhận không có `unwrap()`, `expect()`, `panic!()`.
5. **File Size Limit Audit:** Xác nhận mọi file đạt chuẩn an toàn < 700 dòng.
6. **Binary Release Build:** Đảm bảo `cargo build --release` thành công.
7. **App Smoke Test:** Xác nhận app khởi chạy ngầm 2.5s thành công không crash.

## 3. HƯỚNG DẪN TEST TRỰC QUAN CHO NGƯỜI DÙNG
Nếu toàn bộ cổng PASS, hướng dẫn người dùng chạy app trực tiếp:
```bash
cargo run --release
```
Và đối chiếu theo checklist tại [e2e-test-workflow.md](file:///mnt/Data/darkworks/.agents/workflows/e2e-test-workflow.md).
