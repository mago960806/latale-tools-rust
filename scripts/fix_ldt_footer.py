#!/usr/bin/env python3
"""
LDT 文件修复脚本
检测并移除 END footer 之后的多余数据

LDT 文件格式：
- Header (8716 bytes)
- Row data
- Footer: "END" + 61 个空格 (64 bytes total)

正常情况下，文件应该在 footer 之后结束。
如果 footer 之后还有数据，需要移除。
"""

from pathlib import Path


# Footer 格式: "END" + 61 个空格
FOOTER_MARKER = b"END"
FOOTER_PADDING = b" " * 61
EXPECTED_FOOTER = FOOTER_MARKER + FOOTER_PADDING
FOOTER_SIZE = 64


def find_footer(data: bytes) -> int:
    """
    查找 END footer 的位置
    返回 footer 开始的偏移量，如果找不到返回 -1
    """
    pos = 0
    while True:
        pos = data.find(FOOTER_MARKER, pos)
        if pos == -1:
            return -1
        # 检查后面是否是 61 个空格
        if pos + FOOTER_SIZE <= len(data):
            if data[pos + 3:pos + FOOTER_SIZE] == FOOTER_PADDING:
                return pos
        pos += 1


def scan_and_fix(ldt_dir: Path, dry_run: bool = False) -> tuple[int, int]:
    """
    扫描并修复 LDT 文件

    Returns:
        (fixed_count, total_scanned)
    """
    ldt_files = sorted(ldt_dir.glob("*.LDT"))
    fixed_count = 0

    print(f"扫描 {len(ldt_files)} 个 LDT 文件...")
    print()

    for ldt_file in ldt_files:
        data = ldt_file.read_bytes()
        original_size = len(data)

        # 查找 footer
        footer_pos = find_footer(data)

        if footer_pos == -1:
            print(f"[警告] {ldt_file.name}: 未找到有效的 END footer")
            continue

        # 计算预期的文件结束位置
        expected_end = footer_pos + FOOTER_SIZE

        if expected_end == original_size:
            # 文件正常
            continue

        # footer 之后有多余数据
        extra_bytes = original_size - expected_end
        print(f"[异常] {ldt_file.name}:")
        print(f"       文件大小: {original_size} bytes")
        print(f"       Footer 位置: {footer_pos}")
        print(f"       预期结束: {expected_end}")
        print(f"       多余数据: {extra_bytes} bytes")

        if not dry_run:
            # 截断文件
            truncated_data = data[:expected_end]
            ldt_file.write_bytes(truncated_data)
            print(f"       [已修复] 移除了 {extra_bytes} bytes")
            fixed_count += 1
        else:
            print(f"       [模拟] 将移除 {extra_bytes} bytes")

        print()

    return fixed_count, len(ldt_files)


def main():
    import sys

    project_root = Path(__file__).parent.parent
    ldt_dir = project_root / "DATA" / "LDT"

    if not ldt_dir.exists():
        print(f"错误: LDT 目录不存在: {ldt_dir}")
        return 1

    # 检查是否有 --dry-run 参数
    dry_run = "--dry-run" in sys.argv or "-n" in sys.argv

    if dry_run:
        print("=== 模拟运行模式 ===")
        print()

    print("LDT Footer 修复工具")
    print("=" * 60)
    print(f"目录: {ldt_dir}")
    print()

    fixed, total = scan_and_fix(ldt_dir, dry_run)

    print("=" * 60)
    print(f"扫描完成: {total} 个文件")

    if dry_run:
        print(f"待修复: {fixed} 个文件")
        print()
        print("移除 --dry-run 参数以执行实际修复")
    else:
        print(f"已修复: {fixed} 个文件")

    return 0


if __name__ == "__main__":
    exit(main())
