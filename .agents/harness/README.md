# DARKWORKS HARNESS & CODE QUALITY TOOLS

Trong Darkworks, toàn bộ quá trình kiểm toán mã nguồn, kiểm tra ranh giới Cargo layer và đảm bảo an toàn bộ nhớ được tự động hóa thông qua các bộ công cụ sau:

### 0. Bộ Kiểm Toán E2E Tự Động 1-Click (Master E2E Harness):
```bash
python3 .agents/harness/run_e2e_audit.py
```
> Tự động thực thi toàn bộ 7 cổng kiểm soát: Hardware Probe, Cargo Check, Test Suite, Clippy Zero-Warning, File Size Limit (< 700 dòng), Release Build và App Smoke Test.

### 1. Kiểm tra biên dịch toàn bộ workspace:
```bash
cargo check --workspace --all-targets
```

### 2. Kiểm tra quy chuẩn mã nguồn & Never Crash:
```bash
cargo clippy --workspace --all-targets -- -D warnings
```

### 3. Chạy toàn bộ unit test & integration test:
```bash
cargo test --workspace
```

### 4. Kiểm toán độ dài file mã nguồn (< 700 dòng code):
```bash
python3 .agents/harness/check_file_size.py
```
