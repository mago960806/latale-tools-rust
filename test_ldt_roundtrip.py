#!/usr/bin/env python3
"""
LDT 转换往返测试
测试流程：LDT → CSV → LDT，比较 MD5
"""

import hashlib
import shutil
import subprocess
import sys
from pathlib import Path


def calculate_md5(file_path: Path) -> str:
    """计算文件的 MD5 哈希"""
    hash_md5 = hashlib.md5()
    with open(file_path, "rb") as f:
        for chunk in iter(lambda: f.read(8192), b""):
            hash_md5.update(chunk)
    return hash_md5.hexdigest()


def run_command(cmd: list[str], description: str) -> tuple[bool, str]:
    """运行命令并返回结果"""
    print(f"  {description}...")
    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=3600,  # 1 小时超时
        )
        if result.returncode != 0:
            return False, f"错误: {result.stderr}"
        return True, result.stdout
    except subprocess.TimeoutExpired:
        return False, "超时"
    except Exception as e:
        return False, str(e)


def calculate_original_md5(ldt_dir: Path) -> dict[str, str]:
    """计算原始 LDT 文件的 MD5"""
    md5_map = {}
    ldt_files = sorted(ldt_dir.glob("*.LDT"))
    total = len(ldt_files)

    print(f"计算原始 MD5 ({total} 个文件)...")
    for i, ldt_file in enumerate(ldt_files, 1):
        md5_map[ldt_file.name] = calculate_md5(ldt_file)
        if i % 50 == 0 or i == total:
            print(f"  进度: {i}/{total}")

    return md5_map


def main():
    print("LDT 转换往返测试")
    print("=" * 60)

    # 路径配置
    project_root = Path(__file__).parent
    ldt_dir = project_root / "DATA" / "LDT"
    csv_dir = project_root / "DATA" / "CSV"
    output_dir = project_root / "DATA" / "LDT_OUTPUT"
    binary = project_root / "target" / "release" / "latale-ldt"

    # 检查可执行文件
    if not binary.exists():
        print(f"错误: 找不到 {binary}")
        print("请先运行: cargo build --release")
        return 1

    # 检查源目录
    if not ldt_dir.exists():
        print(f"错误: LDT 目录不存在: {ldt_dir}")
        return 1

    ldt_files = list(ldt_dir.glob("*.LDT"))
    if not ldt_files:
        print(f"错误: LDT 目录中没有文件: {ldt_dir}")
        return 1

    print(f"LDT 文件数量: {len(ldt_files)}")
    print()

    # 步骤 1: 计算原始 MD5
    original_md5 = calculate_original_md5(ldt_dir)
    print()

    # 步骤 2: 清理并准备输出目录
    print("准备输出目录...")
    if csv_dir.exists():
        shutil.rmtree(csv_dir)
    csv_dir.mkdir(parents=True, exist_ok=True)

    if output_dir.exists():
        shutil.rmtree(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    print()

    # 步骤 3: LDT → CSV
    print("[步骤 1] LDT → CSV")
    print("-" * 60)
    success, output = run_command(
        [str(binary), "convert", str(ldt_dir), "-o", str(csv_dir)],
        "批量转换 LDT 到 CSV"
    )
    if not success:
        print(f"失败: {output}")
        return 1
    print("完成")
    print()

    # 步骤 4: CSV → LDT
    print("[步骤 2] CSV → LDT")
    print("-" * 60)
    success, output = run_command(
        [str(binary), "convert", str(csv_dir), "-o", str(output_dir)],
        "批量转换 CSV 到 LDT"
    )
    if not success:
        print(f"失败: {output}")
        return 1
    print("完成")
    print()

    # 步骤 5: 对比 MD5
    print("[步骤 3] MD5 对比")
    print("-" * 60)

    results = []
    passed = 0
    failed = 0

    for ldt_name in sorted(original_md5.keys()):
        original = original_md5[ldt_name]
        new_file = output_dir / ldt_name

        result = {
            "name": ldt_name,
            "original_md5": original,
            "new_md5": None,
            "success": False,
            "error": None,
        }

        if not new_file.exists():
            result["error"] = "文件不存在"
            failed += 1
        else:
            result["new_md5"] = calculate_md5(new_file)
            if result["new_md5"] == original:
                result["success"] = True
                passed += 1
            else:
                result["error"] = "MD5 不匹配"
                failed += 1

        results.append(result)

    # 输出结果
    print(f"对比完成: {passed}/{len(results)} 通过")
    print()

    # 汇总报告
    print("=" * 60)
    print("测试汇总")
    print("=" * 60)

    for r in results:
        status = "PASS" if r["success"] else "FAIL"
        print(f"[{status}] {r['name']}")
        if r["error"]:
            print(f"       错误: {r['error']}")
            if r.get("new_md5"):
                print(f"       原始: {r['original_md5']}")
                print(f"       新的: {r['new_md5']}")

    print()
    print(f"总计: {passed}/{len(results)} 通过")

    if failed > 0:
        print(f"\n失败的文件 ({failed} 个):")
        for r in results:
            if not r["success"]:
                print(f"  - {r['name']}")
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
