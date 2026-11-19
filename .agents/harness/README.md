# DARKWORKS HARNESS & CODE QUALITY TOOLS

Trong Darkworks, toàn bộ quá trình kiểm toán mã nguồn, kiểm tra ranh giới Cargo layer và đảm bảo an toàn bộ nhớ được thực hiện thông qua bộ công cụ chuẩn của Rust:

1. **Kiểm tra biên dịch toàn bộ workspace:**
   ```bash
   cargo check --workspace --all-targets
   ```

2. **Kiểm tra quy chuẩn mã nguồn & Never Crash:**
   ```bash
   cargo clippy --workspace --all-targets -- -D warnings
   ```

3. **Chạy toàn bộ unit test & integration test:**
   ```bash
   cargo test --workspace
   ```

4. **Kiểm tra tính toàn vẹn phân tầng Cargo (L0 -> L6):**
   ```bash
   cargo xtask layers
   ```
