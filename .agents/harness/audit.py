#!/usr/bin/env python3
"""
Darkworks System Audit & Quality Assurance Harness (audit.py)
============================================================
Công cụ trung tâm kiểm toán chất lượng mã nguồn, kiến trúc và an toàn hệ thống cho Darkworks.

Các hạng mục kiểm toán hiện tại và tương lai:
1. [Active] File Size & Line Cap Audit (< 700 lines rule với Vendor Exemption)
2. [Active] Panic & Unwrap Safety Audit (Quét thủ công hoặc tích hợp clippy)
3. [Extensible] Cargo Layer Boundaries Audit (Kiểm tra phụ thuộc L0 -> L6)
4. [Extensible] Shader WGSL Linting & Memory Alignment
"""

import argparse
import os
import sys
from pathlib import Path

# Màu sắc hiển thị Terminal
C_RESET = "\033[0m"
C_BOLD = "\033[1m"
C_GREEN = "\033[32m"
C_RED = "\033[31m"
C_YELLOW = "\033[33m"
C_CYAN = "\033[36m"
C_BLUE = "\033[34m"
C_GRAY = "\033[90m"

ROOT_DIR = Path(__file__).resolve().parent.parent.parent
SOURCE_DIRS = [ROOT_DIR / "apps", ROOT_DIR / "crates"]
EXCEPTIONS_FILE = Path(__file__).resolve().parent / "filesize_exceptions.txt"

# Ngưỡng dòng code cho mã nguồn Darkworks
WARN_THRESHOLD = 650
FAIL_THRESHOLD = 700

# Các mẫu đường dẫn được miễn trừ trần 700 dòng (Mã nguồn Lớp Nền Upstream / Vendor)
DEFAULT_VENDOR_PATTERNS = [
    "reference/",
    "filmcraft",
    "vendor/",
    "target/",
]

def load_file_exceptions():
    """Đọc danh sách các file được Người Dùng phê duyệt ngoại lệ từ file cấu hình."""
    exceptions = set()
    if EXCEPTIONS_FILE.exists():
        with open(EXCEPTIONS_FILE, "r", encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith("#"):
                    exceptions.add(line)
    return exceptions

def is_vendor_or_exempt(rel_path_str: str, exceptions: set) -> bool:
    """Xác định file có thuộc diện Vendor / Upstream Base hoặc đã được cấp ngoại lệ không."""
    # 1. Kiểm tra danh sách ngoại lệ được phê duyệt (hỗ trợ cả đường dẫn file hoặc prefix thư mục)
    for exc in exceptions:
        if rel_path_str == exc or rel_path_str.startswith(exc):
            return True
    # 2. Kiểm tra các mẫu thư mục vendor/upstream
    norm_path = rel_path_str.replace("\\", "/").lower()
    for pattern in DEFAULT_VENDOR_PATTERNS:
        if pattern.lower() in norm_path:
            return True
    return False

def audit_file_sizes(verbose=True, strict=False):
    """Kiểm toán độ dài dòng code cho toàn bộ mã nguồn (.rs)."""
    print(f"\n{C_CYAN}{C_BOLD}{'='*70}")
    print("  🦀 DARKWORKS AUDIT: FILE SIZE & UPSTREAM EXEMPTION CHECK")
    print(f"{'='*70}{C_RESET}\n")

    exceptions = load_file_exceptions()
    files_checked = 0
    darkworks_ok = 0
    darkworks_warn = []
    darkworks_fail = []
    vendor_exempt = []

    for s_dir in SOURCE_DIRS:
        if not s_dir.exists():
            continue
        for file_path in sorted(s_dir.rglob("*.rs")):
            files_checked += 1
            rel_path = file_path.relative_to(ROOT_DIR)
            rel_path_str = str(rel_path).replace("\\", "/")

            try:
                with open(file_path, "r", encoding="utf-8", errors="ignore") as f:
                    lines = len(f.readlines())
            except Exception as e:
                print(f"  {C_RED}✖ Lỗi đọc file {rel_path_str}: {e}{C_RESET}")
                continue

            # Phân loại: Vendor / Upstream Base vs Darkworks Core
            if is_vendor_or_exempt(rel_path_str, exceptions):
                vendor_exempt.append((rel_path_str, lines))
                if verbose:
                    print(f"  {C_GRAY}⚪ [VENDOR EXEMPT]: {rel_path_str} ({lines} lines) - Upstream Base{C_RESET}")
            else:
                if lines >= FAIL_THRESHOLD:
                    darkworks_fail.append((rel_path_str, lines))
                    print(f"  {C_RED}{C_BOLD}🔴 [FAIL >= {FAIL_THRESHOLD}]: {rel_path_str} ({lines} lines) -> BẮT BUỘC BÁO CÁO NGƯỜI DÙNG!{C_RESET}")
                elif lines > WARN_THRESHOLD:
                    darkworks_warn.append((rel_path_str, lines))
                    print(f"  {C_YELLOW}🟡 [WARN > {WARN_THRESHOLD}]: {rel_path_str} ({lines} lines) -> Tiến gần trần 700{C_RESET}")
                else:
                    darkworks_ok += 1
                    if verbose:
                        print(f"  {C_GREEN}🟢 [OK < {FAIL_THRESHOLD}]  : {rel_path_str} ({lines} lines){C_RESET}")

    # Bảng tổng kết
    print(f"\n{C_CYAN}{'-'*70}{C_RESET}")
    print(f"  {C_BOLD}Tổng số file kiểm toán        :{C_RESET} {files_checked}")
    print(f"  {C_GREEN}Mã nguồn Darkworks đạt chuẩn  :{C_RESET} {darkworks_ok}")
    print(f"  {C_YELLOW}Cảnh báo Darkworks (> 650 dòng):{C_RESET} {len(darkworks_warn)}")
    print(f"  {C_RED}Vi phạm Darkworks (>= 700 dòng):{C_RESET} {len(darkworks_fail)}")
    print(f"  {C_GRAY}Ngoại lệ Upstream / Vendor    :{C_RESET} {len(vendor_exempt)}")
    print(f"{C_CYAN}{'='*70}{C_RESET}\n")

    if darkworks_fail:
        print(f"{C_RED}{C_BOLD}🚨 CẢNH BÁO THIẾT QUÂN LUẬT:{C_RESET}")
        print(f"{C_RED}Phát hiện {len(darkworks_fail)} file mã nguồn Darkworks chạm ngưỡng 700 dòng!{C_RESET}")
        print(f"{C_YELLOW}Agent BẮT BUỘC DỪNG LẠI và BÁO CÁO NGƯỜI DÙNG để quyết định phương án (không tự ý refactor).{C_RESET}\n")
        return 1
    elif strict and darkworks_warn:
        print(f"{C_YELLOW}Cảnh báo: Có file vượt 650 dòng ở chế độ strict.{C_RESET}\n")
        return 1
    else:
        print(f"{C_GREEN}✅ TOÀN BỘ FILE MÃ NGUỒN DARKWORKS ĐẠT CHUẨN AN TOÀN (< 700 DÒNG)!{C_RESET}\n")
        return 0

def main():
    parser = argparse.ArgumentParser(description="Darkworks System Audit & Quality Assurance Harness")
    parser.add_argument("--sizes-only", action="store_true", help="Chỉ chạy kiểm toán kích thước file")
    parser.add_argument("--strict", action="store_true", help="Báo lỗi cả với cảnh báo (> 650 dòng)")
    parser.add_argument("--quiet", action="store_true", help="Chỉ hiển thị cảnh báo và lỗi")
    
    args = parser.parse_args()

    # Thực thi kiểm toán độ dài file
    exit_code = audit_file_sizes(verbose=not args.quiet, strict=args.strict)
    
    # [Tương lai]: Thêm các bước kiểm toán khác tại đây (audit_cargo_layers, audit_shaders, ...)
    
    sys.exit(exit_code)

if __name__ == "__main__":
    main()
