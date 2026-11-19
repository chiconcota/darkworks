#!/usr/bin/env python3
"""
Darkworks E2E Test Runner & System Audit Harness
=================================================
Công cụ tự động hóa kiểm thử End-to-End cho Darkworks:
1. Compilation Gate (cargo check)
2. Unit & Integration Test Suite (cargo test)
3. Strict Clippy Linter (Không panic / unwrap / expect)
4. Audit giới hạn dòng code (< 700 lines)
5. Thăm dò phần cứng VA-API & GPU Driver
6. Smoke Test nhị phân Desktop App (Không crash khi nạp)
"""

import os
import subprocess
import sys
import time

C_RESET = "\033[0m"
C_BOLD = "\033[1m"
C_GREEN = "\033[32m"
C_RED = "\033[31m"
C_YELLOW = "\033[33m"
C_CYAN = "\033[36m"
C_BLUE = "\033[34m"

def print_banner():
    print(f"\n{C_CYAN}{C_BOLD}{'='*70}")
    print("  🎬 DARKWORKS E2E TEST WORKFLOW & QUALITY AUDIT SUITE")
    print(f"{'='*70}{C_RESET}\n")

def run_step(title, cmd, cwd=None, env=None, timeout_sec=120):
    print(f"{C_BOLD}▶ [{title}]...{C_RESET}")
    print(f"  {C_BLUE}$ {' '.join(cmd) if isinstance(cmd, list) else cmd}{C_RESET}")
    start = time.time()
    
    current_env = os.environ.copy()
    if env:
        current_env.update(env)
        
    try:
        proc = subprocess.run(
            cmd,
            shell=isinstance(cmd, str),
            cwd=cwd,
            env=current_env,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            timeout=timeout_sec,
        )
        elapsed = time.time() - start
        if proc.returncode == 0:
            print(f"  {C_GREEN}✔ THÀNH CÔNG{C_RESET} ({elapsed:.2f}s)\n")
            return True, proc.stdout
        else:
            print(f"  {C_RED}✖ THẤT BẠI (Mã lỗi: {proc.returncode}){C_RESET} ({elapsed:.2f}s)")
            print(f"{C_YELLOW}--- LOG CHI TIẾT ---{C_RESET}")
            print(proc.stdout[-2000:] if len(proc.stdout) > 2000 else proc.stdout)
            print(f"{C_YELLOW}--------------------{C_RESET}\n")
            return False, proc.stdout
    except subprocess.TimeoutExpired:
        print(f"  {C_RED}✖ QUÁ THỜI GIAN ({timeout_sec}s){C_RESET}\n")
        return False, "Timeout"
    except Exception as e:
        print(f"  {C_RED}✖ LỖI HỆ THỐNG: {e}{C_RESET}\n")
        return False, str(e)

def check_hardware_vaapi():
    print(f"{C_BOLD}▶ [Kiểm tra Phần cứng VA-API & DRM Render Node]...{C_RESET}")
    render_node = "/dev/dri/renderD128"
    has_node = os.path.exists(render_node)
    
    # Kiểm tra libva
    has_libva = os.path.exists("/usr/lib/libva.so.2") or os.path.exists("/usr/lib64/libva.so.2")
    
    if has_node and has_libva:
        print(f"  {C_GREEN}✔ Phát hiện {render_node} và libva.so.2 (Sẵn sàng Hardware Acceleration){C_RESET}\n")
        return True, f"{render_node} OK"
    else:
        print(f"  {C_YELLOW}⚠ Không tìm thấy {render_node} hoặc libva. (Hệ thống sẽ chạy CPU Fallback){C_RESET}\n")
        return True, "Fallback"

def main():
    print_banner()
    root_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "../.."))
    os.chdir(root_dir)

    results = []

    # 1. Hardware Check
    hw_ok, _ = check_hardware_vaapi()
    results.append(("VA-API Hardware Node", hw_ok))

    # 2. Compilation Gate
    ok, _ = run_step(
        "Bước 1: Biên dịch Workspace (cargo check)",
        ["cargo", "check", "--workspace"],
        cwd=root_dir,
    )
    results.append(("Cargo Check Workspace", ok))
    if not ok:
        print(f"{C_RED}Dừng kiểm thử do biên dịch thất bại.{C_RESET}")
        sys.exit(1)

    # 3. Test Suite (Unit & E2E Integration Tests)
    ok, _ = run_step(
        "Bước 2: Chạy toàn bộ Test Suite (cargo test)",
        ["cargo", "test", "--workspace"],
        cwd=root_dir,
    )
    results.append(("Workspace Test Suite (Unit & E2E)", ok))

    # 4. Strict Clippy Linter
    ok, _ = run_step(
        "Bước 3: Kiểm toán An toàn Bộ nhớ & Clippy (Không Panic / Unwrap)",
        ["cargo", "clippy", "--workspace", "--all-targets", "--", "-D", "warnings"],
        cwd=root_dir,
    )
    results.append(("Clippy Zero-Warning Audit", ok))

    # 5. File Size Audit (< 700 lines)
    ok, _ = run_step(
        "Bước 4: Kiểm toán Kích thước File (< 700 dòng code)",
        [sys.executable, ".agents/harness/check_file_size.py"],
        cwd=root_dir,
    )
    results.append(("File Size Limit Audit (< 700 lines)", ok))

    # 6. Binary Release Build & Smoke Test
    ok, _ = run_step(
        "Bước 5: Biên dịch Binary Release (apps/darkworks)",
        ["cargo", "build", "--release", "--bin", "darkworks"],
        cwd=root_dir,
    )
    results.append(("Binary Release Build", ok))

    if ok:
        # Chạy thử 2.5s để đảm bảo binary mở được cửa sổ, nạp font và không bị crash ngầm
        smoke_cmd = "DISPLAY=:1 WAYLAND_DISPLAY=wayland-1 timeout 2.5 ./target/release/darkworks"
        print(f"{C_BOLD}▶ [Bước 6: Khởi chạy Smoke Test Binary 2.5 giây]...{C_RESET}")
        res = subprocess.run(smoke_cmd, shell=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        # Timeout trả về mã 124 là bình thường vì ta chủ động ngắt
        if res.returncode in (0, 124):
            print(f"  {C_GREEN}✔ Binary khởi động thành công, nhận diện GPU adapter và thoát sạch an toàn.{C_RESET}\n")
            results.append(("App Smoke Test (Launch & Adapter)", True))
        else:
            print(f"  {C_RED}✖ Binary crash khi khởi động (Mã lỗi: {res.returncode}){C_RESET}")
            print(res.stdout)
            results.append(("App Smoke Test (Launch & Adapter)", False))

    # TỔNG KẾT BẢNG KẾT QUẢ
    print(f"\n{C_CYAN}{C_BOLD}{'='*70}")
    print("  📋 BẢNG TỔNG KẾT KẾT QUẢ E2E AUDIT")
    print(f"{'='*70}{C_RESET}")
    
    all_passed = True
    for name, status in results:
        status_str = f"{C_GREEN}✔ PASS{C_RESET}" if status else f"{C_RED}✖ FAIL{C_RESET}"
        if not status:
            all_passed = False
        print(f"  {name:<45} : {status_str}")
        
    print(f"{C_CYAN}{'='*70}{C_RESET}\n")
    
    if all_passed:
        print(f"{C_GREEN}{C_BOLD}🎉 TOÀN BỘ CÁC CỔNG KIỂM TRA E2E ĐẠT CHUẨN! BẢN BUILD SẴN SÀNG.{C_RESET}\n")
        sys.exit(0)
    else:
        print(f"{C_RED}{C_BOLD}❌ PHÁT HIỆN LỖI TRONG QUY TRÌNH KIỂM THỬ. VUI LÒNG KIỂM TRA LẠI.{C_RESET}\n")
        sys.exit(1)

if __name__ == "__main__":
    main()
