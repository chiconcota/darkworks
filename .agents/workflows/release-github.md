---
description: Quy trình kiểm tra phiên bản SemVer, biên dịch binary release và phát hành trên GitHub
---

1. **Điều kiện tiên quyết:**
   - Cập nhật số phiên bản SemVer tại `Cargo.toml`.
   - Chạy `cargo check --workspace --all-targets` và `cargo test --workspace`.
   - Viết changelog của phiên bản mới vào `system_map.md` (3 tầng: What's New / Improvements / Fixes).
   - Nhánh Git sạch, đã commit.

2. **Quy trình phát hành:**
   - Biên dịch bản release: `cargo build --release --bin darkworks`
   - Đánh Tag Git: `git tag -a vX.Y.Z -m "Release vX.Y.Z"`
   - Push lên remote: `git push origin main && git push origin vX.Y.Z`
   - Đăng tải Release trên GitHub kèm Changelog và file binary.
