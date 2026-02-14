# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

LaTale SPF Tools 是一个 Rust CLI 工具，用于解包和打包 LaTale 游戏资源文件（SPF 格式）。SPF 文件采用"头尾颠倒"的结构，文件元数据和索引存储在文件末尾。

## 构建命令

```bash
# 构建发布版本
cargo build --release

# 运行 CLI
./target/release/latale-spf <command>

# 可用命令：
#   info <spf_file>           - 显示 SPF 文件信息
#   verify <spf_file>         - 验证 SPF 完整性
#   unpack <spf_file>         - 解包 SPF 到目录
#   pack <spf_name>           - 打包目录为 SPF
```

## 架构

```
src/
├── main.rs              # CLI 入口，使用 clap
├── lib.rs                # 库根
├── spf/
│   ├── mod.rs            # 模块导出
│   ├── types.rs          # 二进制结构 (FInfo, SpfHeader, ResId)
│   ├── reader.rs         # SpfReader，使用 mmap 零拷贝读取
│   ├── writer.rs         # SpfWriter 用于打包文件
│   └── registry.rs       # SpfRegistry 包含 SPF 元数据
└── common/               # 通用工具
```

### 关键设计决策

1. **零拷贝读取**: 使用 `memmap2` 进行内存映射文件访问
2. **编码支持**: 文件名支持 GBK、EUC-KR 等编码（由 `encoding_rs` 处理）
3. **目录顺序保留**: `include_dirs` 定义精确顺序；每个目录内的文件按字母排序
4. **打包不递归**: `add_from_dir` 只读取直接文件，不递归子目录

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
