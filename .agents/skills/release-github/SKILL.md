---
name: release-github
description: Quy trình kiểm tra phiên bản SemVer, biên dịch binary release và phát hành bản phát hành trên GitHub (What's New / Improvements / Fixes)
---

# RELEASE WORKFLOW — DARKWORKS (/release-github)
@status: ACTIVE | @purpose: Quy trình đóng gói, chuẩn hóa Changelog & Phát hành phiên bản mới

Tài liệu này hướng dẫn quy trình phát hành phiên bản mới của **Darkworks** lên GitHub Releases theo cấu trúc chuẩn quốc tế (**What's New / Improvements / Fixes**).

---

## 1. ĐIỀU KIỆN TIÊN QUYẾT (PREREQUISITES)
Trước khi kích hoạt quy trình phát hành:
1. **Nâng số phiên bản (SemVer):** Cập nhật số phiên bản chính xác tại `Cargo.toml` gốc của workspace và các crate liên quan (ví dụ: `version = "0.1.0"`).
2. **Kiểm tra biên dịch & Kiểm thử:** Toàn bộ test suite phải vượt qua:
   ```bash
   cargo check --workspace --all-targets
   cargo test --workspace
   cargo clippy --workspace --all-targets -- -D warnings
   ```
3. **Ghi log hệ thống (`system_map.md`):** Cập nhật nhật ký tại `Recent Logs` của `.darkworks-ai/1-overview/system_map.md`.
4. **Git sạch:** Toàn bộ thay đổi đã được commit trên nhánh làm việc.

---

## 2. CHUẨN ĐỊNH DẠNG CHANGELOG 3 TẦNG

```markdown
# Release Notes - Darkworks {tag}
**Release Date:** {YYYY-MM-DD}  
**Tag:** `{tag}`

---

### ✨ What's New
- **[Tên Tính Năng Mới] ({Tên Crate}):** Mô tả tính năng mới (ví dụ: Node Canvas2D, MCP Server tools, VAD Auto Cut).

### ⚡ Improvements
- **[Tên Cải Tiến/Tối Ưu] ({Tên Crate}):** Tối ưu hiệu năng, giảm mức ăn RAM, nâng cấp FPS render, tái cấu trúc mã nguồn.

### 🐛 Fixes
- **[Tên Lỗi Đã Khắc Phục] ({Tên Crate}):** Mô tả lỗi, nguyên nhân gốc rễ (RCA) và cách sửa chữa.
```

---

## 3. QUY TRÌNH PHÁT HÀNH CHI TIẾT

### Bước 1: Biên dịch bản Release
```bash
cargo build --release --bin darkworks
```

### Bước 2: Tạo Git Tag & Push
```bash
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push origin main
git push origin vX.Y.Z
```

### Bước 3: Phát hành trên GitHub Release
1. Truy cập trang Releases trên GitHub Repository.
2. Dán nội dung Changelog 3 tầng đã chuẩn bị vào phần mô tả.
3. Đính kèm binary thực thi (`target/release/darkworks` hoặc file đóng gói nén tar.gz/deb/AppImage nếu có).
4. Nhấp nút **Publish release**.
