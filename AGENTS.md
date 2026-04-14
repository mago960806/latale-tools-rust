# AGENTS.md

This file provides guidance to Codex (Codex.ai/code) when working with code in this repository.

## 项目概述

LaTale Tools 是一组 Rust CLI 工具，用于处理 LaTale 游戏资源文件：

- `latale-spf`: 解包和打包 SPF 资源包。SPF 文件采用"头尾颠倒"的结构，文件元数据和索引存储在文件末尾。
- `latale-ldt`: LDT 数据表与 CSV 的双向转换。
- `latale-stg`: STG 关卡/地图数据与 JSON 的双向转换。

## 构建命令

```bash
# 构建发布版本
cargo build --release

# 运行 CLI
./target/release/latale-spf <command>
./target/release/latale-ldt <command>
./target/release/latale-stg <command>

# 可用命令：
#   info <spf_file>           - 显示 SPF 文件信息
#   verify <spf_file>         - 验证 SPF 完整性
#   unpack <spf_file>         - 解包 SPF 到目录
#   pack <spf_name>           - 打包目录为 SPF

# latale-stg 常用命令：
#   info [stg_file]           - 以树状结构显示 Stage/Group/Map
#   convert <input>           - STG ↔ JSON 双向转换
```

## 架构

```
src/
├── cli/                  # clap CLI 入口
│   ├── latale-spf.rs
│   ├── latale-ldt.rs
│   └── latale-stg.rs
├── lib.rs                # 库根
├── spf/                  # SPF 资源包读写
├── ldt/                  # LDT 数据表读写与 CSV 转换
├── stg/                  # STG 关卡数据读写与 JSON 转换
└── common/               # 通用工具
```

### 关键设计决策

1. **零拷贝读取**: 使用 `memmap2` 进行内存映射文件访问
2. **编码支持**: 文件名支持 GBK、EUC-KR 等编码（由 `encoding_rs` 处理）
3. **目录顺序保留**: `include_dirs` 定义精确顺序；每个目录内的文件按字母排序
4. **打包不递归**: `add_from_dir` 只读取直接文件，不递归子目录
5. **STG 命名**: STG JSON 字段使用 C++ 结构命名去掉类型前缀后的 PascalCase，如 `StageID`、`GroupList`、`MapName`。
6. **STG 字符串**: `char[64]` 字符串只解码第一个 `\0` 前的有效数据；`\0` 后填充或脏数据忽略，不生成 `Raw` 字段。

### SpfRegistry

每个 SPF 在 `registry.rs` 中有元数据：
- `file_id`: SPF 标识符（用于 RESID 编码）
- `name`: SPF 名称（不含扩展名）
- `version`: 版本号（如 2022091501）
- `encoding`: 文件名编码（如 "GBK"、"EUC-KR"）
- `include_dirs`: 要打包的目录，保持原始顺序

### SPF 二进制布局

```
[文件数据区] → [FINFO 索引表] → [SpfHeader (136 字节)] → [版本号 (4 字节)]
```

从文件末尾读取：版本号 → 文件头 → 索引表 → 文件数据

## 文档

SPF 格式规范: `docs/08_spf_resource_system.md`
STG 格式规范: `docs/STG_BINARY_STRUCTURE.md`
