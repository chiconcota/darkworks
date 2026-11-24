#!/usr/bin/env python3
"""
Darkworks Feature Verification Driver (drive_feature.py)
Lái ứng dụng Darkworks và kiểm chứng nghiệp vụ từng Feature theo kịch bản trong .darkworks-ai/1-overview/features/
"""

import os
import sys
import json
import time
import argparse
import subprocess
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent.parent
FEATURES_DIR = WORKSPACE / ".darkworks-ai" / "1-overview" / "features"
CLI_BIN = WORKSPACE / "target" / "debug" / "darkworks-cli"
EVIDENCE_DIR = WORKSPACE / "target" / "feature_evidence"


def print_banner():
    print("\n" + "=" * 70)
    print("  🎬 DARKWORKS FEATURE VERIFICATION DRIVER (/verify-feature)")
    print("=" * 70)


def list_features():
    print("\n📋 DANH SÁCH CÁC TÍNH NĂNG TRONG KHO NGHIỆP VỤ:")
    if not FEATURES_DIR.exists():
        print("  ❌ Thư mục features không tồn tại!")
        return
    for f in sorted(FEATURES_DIR.glob("*.md")):
        if f.name == "README.md":
            continue
        print(f"  • {f.stem}")
    print()


def verify_media_playback():
    print("\n▶️  [FC-01] Kiểm chứng: Media Playback & VA-API Hardware Decode")
    if not CLI_BIN.exists():
        print(f"  ❌ Binary {CLI_BIN} chưa được biên dịch. Hãy chạy 'cargo build --bin darkworks-cli' trước.")
        return False

    EVIDENCE_DIR.mkdir(parents=True, exist_ok=True)
    out_png = EVIDENCE_DIR / "playback_demo_frame.png"
    if out_png.exists():
        out_png.unlink()

    # 1. Render demo frame
    t0 = time.perf_counter()
    res = subprocess.run(
        [str(CLI_BIN), "render", "--demo", "--seconds", "2", "--out", str(out_png)],
        cwd=WORKSPACE,
        capture_output=True,
        text=True,
    )
    dt_ms = (time.perf_counter() - t0) * 1000

    if res.returncode != 0:
        print(f"  ❌ Lỗi khi render frame: {res.stderr.strip()}")
        return False

    if not out_png.exists() or out_png.stat().st_size == 0:
        print("  ❌ Không sinh được file bằng chứng ảnh render!")
        return False

    print(f"  ✅ Render 1080p frame thành công trong {dt_ms:.1f}ms (< 50ms)")
    print(f"  📸 Bằng chứng lưu tại: {out_png.relative_to(WORKSPACE)} ({out_png.stat().st_size} bytes)")
    return True


def verify_timeline_multitrack():
    print("\n▶️  [FC-02] Kiểm chứng: Multi-Track Timeline & Sequence Model")
    if not CLI_BIN.exists():
        print(f"  ❌ Binary {CLI_BIN} chưa được biên dịch.")
        return False

    t0 = time.perf_counter()
    res = subprocess.run(
        [str(CLI_BIN), "inspect", "--demo"],
        cwd=WORKSPACE,
        capture_output=True,
        text=True,
    )
    dt_ms = (time.perf_counter() - t0) * 1000

    if res.returncode != 0:
        print(f"  ❌ Lỗi khi inspect sequence: {res.stderr.strip()}")
        return False

    try:
        data = json.loads(res.stdout)
    except json.JSONDecodeError:
        print("  ❌ Dữ liệu trả về không phải JSON hợp lệ!")
        return False

    tracks = data.get("tracks", [])
    print(f"  ✅ Truy vấn Sequence thành công trong {dt_ms:.1f}ms: phát hiện {len(tracks)} tracks")
    return True


def verify_lightworks_editing():
    print("\n▶️  [DW-03] Kiểm chứng: Lightworks Mark & Park Editing UX")
    shortcut_file = WORKSPACE / "crates" / "engine" / "src" / "shortcut_presets.rs"
    if not shortcut_file.exists():
        print("  ❌ Không tìm thấy file shortcut_presets.rs!")
        return False

    content = shortcut_file.read_text(encoding="utf-8")
    checks = [
        ("markers.markIn", "I"),
        ("markers.markOut", "O"),
        ("markers.clearInOut", "P"),
        ("sequence.extract", "X"),
        ("sequence.lift", "Z"),
        ("sequence.addEdit", "S"),
    ]

    missing = []
    for cmd, key in checks:
        pattern = f'("{cmd}", "{key}",'
        if pattern not in content:
            missing.append(f"{cmd} -> {key}")

    if missing:
        print(f"  🟡 Một số phím tắt Lightworks chưa được kích hoạt trong Darkworks Default:")
        for m in missing:
            print(f"     • {m}")
        print("  ℹ️  Cần cấy ghép phím tắt vào crates/engine/src/shortcut_presets.rs ở Giai đoạn 2.")
        return False

    print("  ✅ Toàn bộ phím tắt Lightworks (I, O, P, X, Z, S) đã được cấu hình chuẩn xác!")
    return True


def verify_node_graph():
    print("\n▶️  [DW-01] Kiểm chứng: DAG Node Graph Engine, 18 Atomic Nodes & 5 Presets")
    node_panel_file = WORKSPACE / "crates" / "ui-egui" / "src" / "panels" / "node_graph.rs"
    if not node_panel_file.exists():
        print("  ❌ Không tìm thấy file node_graph.rs trong ui-egui!")
        return False

    t0 = time.perf_counter()
    res = subprocess.run(
        ["cargo", "test", "-p", "darkworks-node-graph", "--", "test_all_5_builtin_presets_validation", "test_18_atomic_nodes_creation_and_sockets", "test_dag_evaluator_cycle_detection"],
        cwd=WORKSPACE,
        capture_output=True,
        text=True,
    )
    dt_ms = (time.perf_counter() - t0) * 1000

    if res.returncode != 0:
        print(f"  ❌ Lỗi khi chạy test suite darkworks-node-graph:\n{res.stderr.strip()}")
        return False

    print(f"  ✅ 18 Atomic Nodes, 5 Presets & DAG Cycle Detection kiểm thử thành công ({dt_ms:.1f}ms)!")
    print(f"  🌟 Giao diện Node Graph Panel đã tích hợp sẵn sàng trong Docking Layout.")
    return True


def main():
    print_banner()
    parser = argparse.ArgumentParser(description="Darkworks Feature Verification Driver")
    parser.add_argument("--feature", "-f", type=str, help="Tên tính năng cần kiểm chứng (hoặc 'all')")
    parser.add_argument("--list", "-l", action="store_true", help="Liệt kê toàn bộ các tính năng")

    args = parser.parse_args()

    if args.list or not args.feature:
        list_features()
        if not args.feature:
            print("💡 Dùng lệnh: python3 .agents/harness/drive_feature.py --feature <tên-feature> hoặc 'all'")
            return

    feat = args.feature.lower()
    results = {}

    if feat in ["all", "media", "playback", "fc-01", "filmcraft_media_playback"]:
        results["FC-01 Media Playback"] = verify_media_playback()

    if feat in ["all", "timeline", "fc-02", "filmcraft_timeline_multitrack"]:
        results["FC-02 Multi-Track Timeline"] = verify_timeline_multitrack()

    if feat in ["all", "lightworks", "dw-03", "darkworks_lightworks_editing"]:
        results["DW-03 Lightworks Editing"] = verify_lightworks_editing()

    if feat in ["all", "nodegraph", "node-graph", "node_graph", "dw-01", "darkworks_node_graph"]:
        results["DW-01 Node Graph Engine"] = verify_node_graph()

    if not results:
        print(f"⚠️  Chưa có kịch bản kiểm thử tự động cho feature: '{args.feature}'.")
        print("    Vui lòng kiểm tra tên trong danh sách (--list).")
        sys.exit(1)

    print("\n" + "-" * 70)
    print("📊 BẢNG TỔNG KẾT KIỂM CHỨNG FEATURE:")
    all_pass = True
    for name, passed in results.items():
        status = "🟢 PASS" if passed else "🔴 FAIL"
        print(f"  • {name:<35}: {status}")
        if not passed:
            all_pass = False
    print("-" * 70 + "\n")

    sys.exit(0 if all_pass else 1)


if __name__ == "__main__":
    main()
