#!/usr/bin/env python3
"""
[DEPRECATED / FORWARDER] check_file_size.py đã được đổi tên và nâng cấp thành audit.py.
File này được giữ lại để tương thích ngược với các workflow và script cũ.
"""
import os
import sys
from pathlib import Path

audit_script = Path(__file__).resolve().parent / "audit.py"
if audit_script.exists():
    os.execv(sys.executable, [sys.executable, str(audit_script)] + sys.argv[1:])
else:
    sys.exit("Không tìm thấy audit.py!")
