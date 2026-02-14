# LaTale SPF CLI 工具设计方案

## 概述

LaTale 游戏资源解包/打包 CLI 工具，使用 Rust 实现，通过 memmap2 + bytemuck 实现高性能零拷贝文件读取。

**核心需求：**
- 解包和打包同等重要
- **精确往返**：解包后未编辑再打包，MD5 必须一致
- 当前只支持 SPF 格式

## 项目结构

```
src/
├── main.rs              # CLI 入口，解析 clap 参数
├── lib.rs               # 导出 spf 模块
├── common/
│   └── mod.rs           # 通用工具
└── spf/
    ├── mod.rs           # 导出 reader/writer/types
    ├── types.rs         # SPF 数据结构
    ├── reader.rs        # SPF 读取/解包逻辑
    ├── writer.rs        # SPF 写入/打包逻辑
    └── registry.rs      # FILE_ID ↔ SPF 名称 ↔ 路径前缀映射表
```

## 数据结构

```rust
// types.rs

use bytemuck::{Pod, Zeroable};

/// SPF 版本号（文件末尾 4 字节）
pub type SpfVersion = i32;

/// RESID：32 位资源 ID
#[derive(Clone, Copy, PartialEq, Eq, Zeroable, Pod)]
#[repr(transparent)]
pub struct ResId(pub u32);

impl ResId {
    pub fn new(file_id: u8, instance_id: u32) -> Self {
        Self(((file_id as u32) << 24) | (instance_id & 0xFFFFFF))
    }

    pub fn file_id(self) -> u8 {
        (self.0 >> 24) as u8
    }

    pub fn instance_id(self) -> u32 {
        self.0 & 0xFFFFFF
    }
}

/// 文件信息结构（140 字节）
#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct FInfo {
    pub file_name: [u8; 128],  // 文件名，C 字符串
    pub offset: i32,           // 文件在 SPF 中的偏移量
    pub size: i32,             // 文件大小
    pub res_id: ResId,         // 资源 ID
}

/// SPF 文件头（40 字节）
#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct SpfHeader {
    pub header_size: i32,      // 索引表总大小
    pub file_id: i32,          // SPF 文件 ID
    pub desc: [u8; 32],        // 描述信息
}

/// SPF 文件常量
pub const SPF_VERSION: SpfVersion = 0;
pub const MAX_FILE_NAME: usize = 128;
pub const DESC_SIZE: usize = 32;
```

## 读取器

```rust
// reader.rs

use memmap2::Mmap;
use std::fs::File;
use std::path::Path;

pub struct SpfReader {
    mmap: Mmap,
}

impl SpfReader {
    /// 打开 SPF 文件并映射到内存
    pub fn open(path: &Path) -> anyhow::Result<Self>;

    /// 获取 SPF 版本号（文件末尾 4 字节）
    pub fn version(&self) -> SpfVersion;

    /// 获取 SPF 文件头
    pub fn header(&self) -> &SpfHeader;

    /// 获取所有文件信息（FINFO 数组）
    pub fn file_infos(&self) -> &[FInfo];

    /// 获取指定文件的原始数据（零拷贝）
    pub fn get_file_data(&self, finfo: &FInfo) -> &[u8];

    /// 解包所有文件到指定目录
    pub fn unpack(&self, output_dir: &Path) -> anyhow::Result<()>;
}
```

**读取流程：**
1. mmap 整个 SPF 文件
2. 从文件末尾读取版本号 → SPF_HEADER → FINFO 索引表
3. 遍历 FINFO，按偏移量从数据区提取文件
4. 保持 FINFO 的原始顺序（对 MD5 一致性关键）

**所有读取都是零拷贝，直接返回 `&[u8]` 切片引用。**

## 写入器

```rust
// writer.rs

use std::path::Path;

pub struct SpfWriter {
    file_id: u8,
    files: Vec<(String, Vec<u8>)>,  // (文件名, 文件数据)
}

impl SpfWriter {
    /// 创建新的 SPF 写入器，指定 FILE_ID
    pub fn new(file_id: u8) -> Self;

    /// 添加文件
    pub fn add_file(&mut self, name: String, data: Vec<u8>);

    /// 从目录扫描并添加所有文件（按文件名排序）
    pub fn add_from_dir(&mut self, dir: &Path, prefix: &str) -> anyhow::Result<()>;

    /// 写入 SPF 文件
    pub fn write(&self, output_path: &Path) -> anyhow::Result<()>;
}
```

**写入流程：**
1. 文件按文件名排序后依次写入数据区
2. 计算每个文件的偏移量，构建 FINFO 数组
3. 写入 FINFO → SpfHeader → Version

## 注册表

```rust
// registry.rs

pub struct SpfRegistry {
    pub file_id: u8,
    pub name: &'static str,           // 如 "HOSHIM"
    pub path_prefix: &'static str,    // 如 "CHAR/HOSHIM"
}

impl SpfRegistry {
    /// 所有 SPF 映射表
    pub const ALL: &'static [SpfRegistry] = &[
        SpfRegistry { file_id: 0,  name: "TESTPACK",  path_prefix: "TEST" },
        SpfRegistry { file_id: 2,  name: "HOSHIM",    path_prefix: "CHAR/HOSHIM" },
        SpfRegistry { file_id: 3,  name: "ROWID",     path_prefix: "CHAR/ROWID" },
        SpfRegistry { file_id: 5,  name: "MAKO1298",  path_prefix: "CHAR/MAKO1298" },
        // ... 其他 SPF
    ];

    /// 根据 SPF 名称查找注册信息
    pub fn find_by_name(name: &str) -> Option<&'static SpfRegistry>;

    /// 根据 FILE_ID 查找注册信息
    pub fn find_by_file_id(file_id: u8) -> Option<&'static SpfRegistry>;
}
```

## CLI 接口

```rust
// main.rs

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "latale-spf")]
#[command(about = "LaTale SPF resource pack/unpack tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Unpack SPF file to directory
    Unpack {
        /// SPF file to unpack
        spf_file: String,
        /// Output directory (default: current directory)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Pack directory into SPF file
    Pack {
        /// SPF file name to create (determines FILE_ID and source path)
        spf_file: String,
        /// Output directory (default: current directory)
        #[arg(short, long)]
        output: Option<String>,
        /// Source DATA directory (default: ./DATA)
        #[arg(long)]
        data_dir: Option<String>,
    },
}
```

**使用示例：**
```bash
# 解包
latale-spf unpack HOSHIM.SPF              # → ./DATA/CHAR/HOSHIM/...
latale-spf unpack HOSHIM.SPF -o ./out/    # → ./out/DATA/CHAR/HOSHIM/...

# 打包
latale-spf pack HOSHIM.SPF                # 从 ./DATA/CHAR/HOSHIM/ 打包
latale-spf pack HOSHIM.SPF -o ./dist/     # 输出到 ./dist/HOSHIM.SPF
latale-spf pack HOSHIM.SPF --data-dir ./other/DATA  # 指定数据源
```

## MD5 一致性保证

为确保解包后未编辑再打包能产生相同 MD5：

1. **文件顺序**：打包时按文件名排序（假设原始打包也是按文件名排序）
2. **零拷贝读取**：直接读取原始字节，不做任何转换
3. **精确写入**：按原始结构精确写入每个字节
4. **RESID 重建**：FILE_ID 固定，INSTANCE_ID 按顺序（0, 1, 2...）生成
