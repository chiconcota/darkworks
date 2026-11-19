#!/usr/bin/env python3
"""
Darkworks Harness: File Size Auditor (check_file_size.py)

Quy tắc:
- Trần chuẩn mực: < 700 dòng code mỗi file (.rs).
- Thiết quân luật: Khi file chạm 700 dòng, Agent phải dừng lại báo cáo Người Dùng
  để quyết định phương án (không tự động tối ưu/tái cấu trúc).
"""

import sys
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parent.parent.parent
SOURCE_DIRS = [ROOT_DIR / "apps", ROOT_DIR / "crates"]

WARN_THRESHOLD = 650
FAIL_THRESHOLD = 700

def audit_file_sizes():
    files_checked = 0
    warnings = []
    failures = []

    print("=" * 65)
    print("  🦀 DARKWORKS FILE SIZE AUDIT (< 700 Lines Directives)")
    print("=" * 65)

    for s_dir in SOURCE_DIRS:
        if not s_dir.exists():
            continue
        for file_path in sorted(s_dir.rglob("*.rs")):
            files_checked += 1
            rel_path = file_path.relative_to(ROOT_DIR)
            with open(file_path, "r", encoding="utf-8", errors="ignore") as f:
                lines = len(f.readlines())

            if lines >= FAIL_THRESHOLD:
                failures.append((rel_path, lines))
                print(f"  🔴 [FAIL >= 700]: {rel_path} ({lines} lines) -> BẮT BUỘC BÁO CÁO NGƯỜI DÙNG!")
            elif lines > WARN_THRESHOLD:
                warnings.append((rel_path, lines))
                print(f"  🟡 [WARN > 650]: {rel_path} ({lines} lines) -> Tiến gần trần 700")
            else:
                print(f"  🟢 [OK < 700]  : {rel_path} ({lines} lines)")

    print("-" * 65)
    print(f"Tổng số file kiểm toán: {files_checked}")
    print(f"Cảnh báo (> 650 dòng) : {len(warnings)}")
    print(f"Vi phạm (>= 700 dòng) : {len(failures)}")
    print("=" * 65)

    if failures:
        print("\n🚨 CẢNH BÁO THIẾT QUÂN LUẬT:")
        print("Có file chạm ngưỡng 700 dòng! Agent DỪNG LẠI và BÁO CÁO NGƯỜI DÙNG để quyết định.")
        return 1
    else:
        print("\n✅ TOÀN BỘ FILE MÃ NGUỒN ĐẠT CHUẨN AN TOÀN (< 700 DÒNG)!")
        return 0

if __name__ == "__main__":
    sys.exit(audit_file_sizes())
