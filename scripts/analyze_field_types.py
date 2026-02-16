#!/usr/bin/env python3
"""
分析 LDT/CSV 文件中实际使用的数据类型统计

扫描 DATA/CSV 目录下所有 CSV 文件，统计每种字段类型的使用频率。
"""

import csv
import os
from collections import Counter
from pathlib import Path


def get_field_type(header: str) -> str:
    """
    从 CSV 头部提取字段类型。

    格式: "字段名:类型" 或 "字段名"
    """
    if ':' in header:
        return header.split(':', 1)[1].strip().lower()
    return 'unknown'


def analyze_csv_files(csv_dir: Path) -> tuple[Counter, Counter, int]:
    """
    分析目录下所有 CSV 文件的字段类型。

    Returns:
        type_counter: 每种类型出现的次数
        file_type_counter: 每个文件包含的类型数量
        total_fields: 总字段数
    """
    type_counter = Counter()
    file_type_counter = Counter()
    total_fields = 0
    csv_files = list(csv_dir.glob('*.csv'))

    print(f"扫描 {len(csv_files)} 个 CSV 文件...")
    print()

    for csv_file in sorted(csv_files):
        try:
            with open(csv_file, 'r', encoding='utf-8') as f:
                reader = csv.reader(f)
                headers = next(reader, [])

                file_types = set()
                for header in headers:
                    field_type = get_field_type(header)
                    type_counter[field_type] += 1
                    file_types.add(field_type)
                    total_fields += 1

                # 记录该文件包含的类型
                for ft in file_types:
                    file_type_counter[ft] += 1

        except Exception as e:
            print(f"  警告: 读取 {csv_file.name} 失败: {e}")

    return type_counter, file_type_counter, total_fields


def main():
    # CSV 目录路径
    script_dir = Path(__file__).parent
    csv_dir = script_dir.parent / 'DATA' / 'CSV'

    if not csv_dir.exists():
        print(f"错误: CSV 目录不存在: {csv_dir}")
        return 1

    # 分析文件
    type_counter, file_type_counter, total_fields = analyze_csv_files(csv_dir)

    # 输出结果
    print("=" * 60)
    print("字段类型统计 (按出现次数排序)")
    print("=" * 60)
    print(f"{'类型':<12} {'出现次数':>10} {'占比':>10} {'涉及文件数':>12}")
    print("-" * 60)

    for field_type, count in type_counter.most_common():
        percentage = (count / total_fields) * 100
        file_count = file_type_counter[field_type]
        print(f"{field_type:<12} {count:>10} {percentage:>9.2f}% {file_count:>12}")

    print("-" * 60)
    print(f"{'总计':<12} {total_fields:>10}")
    print()

    # 类型分类统计
    print("=" * 60)
    print("类型分类")
    print("=" * 60)

    # 按类型分组
    numeric_types = {'int32', 'int64'}
    float_types = {'float32'}
    string_types = {'string', 'alias'}
    special_types = {'fid', 'bool', 'na'}

    categories = [
        ('整数类型 (int32, int64)', numeric_types),
        ('浮点类型 (float32)', float_types),
        ('字符串类型 (string, alias)', string_types),
        ('特殊类型 (fid, bool, na)', special_types),
    ]

    for name, types in categories:
        count = sum(type_counter[t] for t in types if t in type_counter)
        pct = (count / total_fields) * 100 if total_fields > 0 else 0
        print(f"{name}: {count} ({pct:.1f}%)")

    print()

    return 0


if __name__ == '__main__':
    exit(main())
