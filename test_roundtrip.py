#!/usr/bin/env python3
"""
SPF 打包/解包往返测试
测试流程：解包 SPF -> 重新打包 -> 比较 MD5
"""

import hashlib
import os
import shutil
import subprocess
import tempfile
from pathlib import Path

# 原始 SPF 文件的 MD5
ORIGINAL_MD5 = {
    "AJJIYA.SPF": "5b6fc10f9bdebe6ba9a5d24119dc167b",
    "BANX.SPF": "ff080fab06db2be0e9ecc487023f455d",
    "BARY.SPF": "2913b6047250a66c3757fc0ef788f146",
    "CLAIRE.SPF": "b13adba3f8a285e67be5f4009607bee9",
    "CVOICE.SPF": "e0fb0b10a1eb7f682925a54e6859ed67",
    "DALBONG.SPF": "641fefcae4f946585fea39da1e659dc0",
    "HOSHIM.SPF": "99f63dbe0864859b7ca2c56b8dbcc8ca",
    "JINSSAGA.SPF": "c93d843c5a611e20b02d8917ddf0b3ab",
    "MAKO1298.SPF": "840044d512f63d965092dd30874da1e9",
    "METALGENI.SPF": "31e73c80a985c24f5d79db8f4aa6902e",
    "ROWID.SPF": "1615d2c645c3f5712b563f8ea68be02d",
    "RYUMS.SPF": "291d8632e73b1d260772b4a9cedeadc5",
    "ZENNE.SPF": "ce0100abcdea5b2130cc2108ab3e1010",
}

# SPF 对应的编码
SPF_ENCODING = {
    "AJJIYA.SPF": "EUC-KR",
    "BANX.SPF": "GBK",
    "BARY.SPF": "GBK",
    "CLAIRE.SPF": "GBK",
    "CVOICE.SPF": "GBK",
    "DALBONG.SPF": "GBK",
    "HOSHIM.SPF": "GBK",
    "JINSSAGA.SPF": "GBK",
    "MAKO1298.SPF": "GBK",
    "METALGENI.SPF": "GBK",
    "ROWID.SPF": "GBK",
    "RYUMS.SPF": "GBK",
    "ZENNE.SPF": "GBK",
}


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


def test_spf_roundtrip(spf_file: Path, output_dir: Path, temp_dir: Path) -> dict:
    """测试单个 SPF 文件的往返"""
    spf_name = spf_file.name
    encoding = SPF_ENCODING.get(spf_name, "GBK")

    result = {
        "name": spf_name,
        "original_md5": ORIGINAL_MD5.get(spf_name, "unknown"),
        "new_md5": None,
        "file_count": 0,
        "success": False,
        "error": None,
    }

    print(f"\n{'='*60}")
    print(f"测试: {spf_name}")
    print(f"编码: {encoding}")
    print(f"{'='*60}")

    # 验证原始文件 MD5
    current_md5 = calculate_md5(spf_file)
    print(f"当前 MD5: {current_md5}")
    print(f"期望 MD5: {result['original_md5']}")

    if current_md5 != result["original_md5"]:
        result["error"] = f"原始文件 MD5 不匹配: {current_md5} != {result['original_md5']}"
        print(f"❌ {result['error']}")
        return result

    # 创建工作目录
    work_dir = temp_dir / spf_name.replace(".SPF", "")
    unpack_dir = work_dir / "unpacked"
    work_dir.mkdir(parents=True, exist_ok=True)

    # 步骤 1: 解包
    success, output = run_command(
        ["./target/release/latale-spf", "unpack", str(spf_file),
         "-o", str(unpack_dir)],
        "解包"
    )
    if not success:
        result["error"] = f"解包失败: {output}"
        print(f"❌ {result['error']}")
        return result

    # 获取文件数量
    for line in output.split("\n"):
        if "文件数量" in line:
            try:
                result["file_count"] = int(line.split(":")[1].strip())
            except:
                pass

    print(f"文件数量: {result['file_count']}")

    # 步骤 2: 重新打包
    new_spf = output_dir / spf_name
    success, output = run_command(
        ["./target/release/latale-spf", "pack", spf_name.replace(".SPF", ""),
         "-o", str(new_spf), "--data-dir", str(unpack_dir),
         "--encoding", encoding],
        "重新打包"
    )
    if not success:
        result["error"] = f"打包失败: {output}"
        print(f"❌ {result['error']}")
        return result

    # 步骤 3: 计算新文件的 MD5
    result["new_md5"] = calculate_md5(new_spf)
    print(f"新文件 MD5: {result['new_md5']}")

    # 步骤 4: 比较 MD5
    if result["new_md5"] == result["original_md5"]:
        result["success"] = True
        print(f"✅ MD5 匹配!")
    else:
        result["error"] = "MD5 不匹配"
        print(f"❌ MD5 不匹配!")

    return result


def main():
    print("SPF 打包/解包往返测试")
    print("="*60)

    # 检查可执行文件
    if not Path("./target/release/latale-spf").exists():
        print("错误: 找不到 ./target/release/latale-spf")
        print("请先运行: cargo build --release")
        return

    # 创建输出目录
    output_dir = Path("./output")
    output_dir.mkdir(exist_ok=True)

    # 创建临时目录
    temp_dir = Path("./temp_test")
    if temp_dir.exists():
        shutil.rmtree(temp_dir)
    temp_dir.mkdir()

    # 查找所有 SPF 文件
    spf_files = sorted(Path(".").glob("*.SPF"))

    if not spf_files:
        print("错误: 当前目录没有找到 SPF 文件")
        return

    print(f"找到 {len(spf_files)} 个 SPF 文件")

    # 测试结果
    results = []

    # 测试每个 SPF 文件
    for spf_file in spf_files:
        if spf_file.name not in ORIGINAL_MD5:
            print(f"\n跳过 {spf_file.name} (没有原始 MD5)")
            continue

        result = test_spf_roundtrip(spf_file, output_dir, temp_dir)
        results.append(result)

    # 清理临时目录
    if temp_dir.exists():
        shutil.rmtree(temp_dir)

    # 打印汇总
    print("\n" + "="*60)
    print("测试汇总")
    print("="*60)

    passed = sum(1 for r in results if r["success"])
    failed = len(results) - passed

    for r in results:
        status = "✅" if r["success"] else "❌"
        print(f"{status} {r['name']}: {r['file_count']} 文件")
        if r["error"]:
            print(f"   错误: {r['error']}")

    print()
    print(f"总计: {passed}/{len(results)} 通过")

    if failed > 0:
        print(f"\n失败的文件:")
        for r in results:
            if not r["success"]:
                print(f"  - {r['name']}")
                if r.get("new_md5"):
                    print(f"    原始 MD5: {r['original_md5']}")
                    print(f"    新的 MD5:  {r['new_md5']}")


if __name__ == "__main__":
    main()
