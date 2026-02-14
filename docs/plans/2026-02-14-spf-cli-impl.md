# SPF CLI 工具实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现 LaTale SPF 资源解包/打包 CLI 工具，支持精确往返（解包后未编辑再打包 MD5 一致）。

**Architecture:** 使用 memmap2 将 SPF 文件映射到内存，通过 bytemuck 零拷贝转换数据结构。读取时从文件末尾逆向解析，写入时按文件名排序顺序生成。

**Tech Stack:** Rust, clap, memmap2, bytemuck, anyhow, walkdir, indicatif

---

## Task 1: 项目骨架搭建

**Files:**
- Create: `src/lib.rs`
- Create: `src/main.rs`
- Create: `src/common/mod.rs`
- Create: `src/spf/mod.rs`

**Step 1: 创建 src/lib.rs**

```rust
pub mod common;
pub mod spf;
```

**Step 2: 创建 src/main.rs**

```rust
use clap::Parser;

#[derive(Parser)]
#[command(name = "latale-spf")]
#[command(about = "LaTale SPF resource pack/unpack tool")]
struct Cli {
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> anyhow::Result<()> {
    let _cli = Cli::parse();
    println!("latale-spf - LaTale SPF resource tool");
    Ok(())
}
```

**Step 3: 创建 src/common/mod.rs**

```rust
// Common utilities for latale-tools
```

**Step 4: 创建 src/spf/mod.rs**

```rust
pub mod types;
pub mod reader;
pub mod writer;
pub mod registry;

pub use types::*;
pub use reader::SpfReader;
pub use writer::SpfWriter;
pub use registry::SpfRegistry;
```

**Step 5: 验证编译**

Run: `cargo build`
Expected: 成功编译，无错误

**Step 6: Commit**

```bash
git add src/lib.rs src/main.rs src/common/mod.rs src/spf/mod.rs
git commit -m "feat: set up project skeleton"
```

---

## Task 2: SPF 数据结构定义

**Files:**
- Create: `src/spf/types.rs`

**Step 1: 定义 ResId 结构**

```rust
use bytemuck::{Pod, Zeroable};

/// SPF 版本号（文件末尾 4 字节）
pub type SpfVersion = i32;

/// RESID：32 位资源 ID
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Zeroable, Pod)]
#[repr(transparent)]
pub struct ResId(pub u32);

impl ResId {
    /// 从 FILE_ID 和 INSTANCE_ID 创建 ResId
    pub fn new(file_id: u8, instance_id: u32) -> Self {
        Self(((file_id as u32) << 24) | (instance_id & 0x00FF_FFFF))
    }

    /// 获取 FILE_ID（高 8 位）
    pub fn file_id(self) -> u8 {
        (self.0 >> 24) as u8
    }

    /// 获取 INSTANCE_ID（低 24 位）
    pub fn instance_id(self) -> u32 {
        self.0 & 0x00FF_FFFF
    }
}

impl std::fmt::Debug for ResId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResId")
            .field("file_id", &self.file_id())
            .field("instance_id", &self.instance_id())
            .finish()
    }
}
```

**Step 2: 验证编译**

Run: `cargo build`
Expected: 成功

**Step 3: 定义 FInfo 结构**

```rust
/// 文件信息结构（140 字节）
#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct FInfo {
    /// 文件名（C 字符串，128 字节）
    pub file_name: [u8; 128],
    /// 文件在 SPF 中的偏移量
    pub offset: i32,
    /// 文件大小（字节）
    pub size: i32,
    /// 资源 ID
    pub res_id: ResId,
}

const _: () = assert!(std::mem::size_of::<FInfo>() == 140);

impl FInfo {
    /// 获取文件名字符串（去除尾部的 null 字符）
    pub fn file_name_str(&self) -> &str {
        let end = self.file_name.iter().position(|&b| b == 0).unwrap_or(128);
        // SAFETY: SPF 文件名应该是有效的 ASCII
        unsafe { std::str::from_utf8_unchecked(&self.file_name[..end]) }
    }
}
```

**Step 4: 验证编译**

Run: `cargo build`
Expected: 成功

**Step 5: 定义 SpfHeader 结构**

```rust
/// SPF 文件头（40 字节）
#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct SpfHeader {
    /// 索引表总大小（所有 FInfo 的字节总和）
    pub header_size: i32,
    /// SPF 文件 ID
    pub file_id: i32,
    /// 描述信息
    pub desc: [u8; 32],
}

const _: () = assert!(std::mem::size_of::<SpfHeader>() == 40);

impl SpfHeader {
    /// 获取描述字符串
    pub fn desc_str(&self) -> &str {
        let end = self.desc.iter().position(|&b| b == 0).unwrap_or(32);
        unsafe { std::str::from_utf8_unchecked(&self.desc[..end]) }
    }
}
```

**Step 6: 添加常量定义**

```rust
/// SPF 文件常量
pub const SPF_VERSION: SpfVersion = 0;
/// 最大文件名长度
pub const MAX_FILE_NAME: usize = 128;
/// 描述信息长度
pub const DESC_SIZE: usize = 32;
```

**Step 7: 验证完整编译**

Run: `cargo build`
Expected: 成功，无警告

**Step 8: Commit**

```bash
git add src/spf/types.rs
git commit -m "feat(spf): define SPF data structures (ResId, FInfo, SpfHeader)"
```

---

## Task 3: SPF 注册表

**Files:**
- Create: `src/spf/registry.rs`

**Step 1: 定义 SpfRegistry 结构和映射表**

```rust
/// SPF 文件注册信息
#[derive(Debug, Clone, Copy)]
pub struct SpfRegistry {
    /// SPF 文件 ID
    pub file_id: u8,
    /// SPF 名称（不含扩展名）
    pub name: &'static str,
    /// 内部路径前缀
    pub path_prefix: &'static str,
}

impl SpfRegistry {
    /// 所有 SPF 映射表
    /// 基于 docs/08_spf_resource_system.md 中的 FILE_ID 对照表
    pub const ALL: &'static [SpfRegistry] = &[
        SpfRegistry { file_id: 0, name: "TESTPACK", path_prefix: "TEST" },
        SpfRegistry { file_id: 2, name: "HOSHIM", path_prefix: "CHAR/HOSHIM" },
        SpfRegistry { file_id: 3, name: "ROWID", path_prefix: "CHAR/ROWID" },
        SpfRegistry { file_id: 5, name: "MAKO1298", path_prefix: "CHAR/MAKO1298" },
        SpfRegistry { file_id: 6, name: "METALGENI", path_prefix: "CHAR/METALGENI" },
        SpfRegistry { file_id: 7, name: "DALBONG", path_prefix: "CHAR/DALBONG" },
        SpfRegistry { file_id: 8, name: "RYUMS", path_prefix: "CHAR/RYUMS" },
        SpfRegistry { file_id: 9, name: "BANX", path_prefix: "CHAR/BANX" },
        SpfRegistry { file_id: 10, name: "BARY", path_prefix: "CHAR/BARY" },
        SpfRegistry { file_id: 12, name: "CLAIRE", path_prefix: "CHAR/CLAIRE" },
        SpfRegistry { file_id: 13, name: "CVOICE", path_prefix: "CVOICE" },
        // 更多 SPF 可根据需要添加
    ];

    /// 根据 SPF 名称查找注册信息
    pub fn find_by_name(name: &str) -> Option<&'static SpfRegistry> {
        // 去除可能的扩展名
        let name = name.trim_end_matches(".SPF").trim_end_matches(".spf");
        Self::ALL.iter().find(|r| r.name.eq_ignore_ascii_case(name))
    }

    /// 根据 FILE_ID 查找注册信息
    pub fn find_by_file_id(file_id: u8) -> Option<&'static SpfRegistry> {
        Self::ALL.iter().find(|r| r.file_id == file_id)
    }
}
```

**Step 2: 验证编译**

Run: `cargo build`
Expected: 成功

**Step 3: Commit**

```bash
git add src/spf/registry.rs
git commit -m "feat(spf): add SPF registry with FILE_ID mappings"
```

---

## Task 4: SPF 读取器

**Files:**
- Create: `src/spf/reader.rs`

**Step 1: 定义 SpfReader 结构**

```rust
use crate::spf::{FInfo, SpfHeader, SpfVersion, SPF_VERSION};
use anyhow::{bail, Context, Result};
use memmap2::Mmap;
use std::fs::File;
use std::path::Path;

/// SPF 文件读取器
pub struct SpfReader {
    mmap: Mmap,
    header: SpfHeader,
    version: SpfVersion,
}

impl SpfReader {
    /// 打开 SPF 文件并映射到内存
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path)
            .with_context(|| format!("Failed to open SPF file: {}", path.display()))?;

        // SAFETY: 文件映射是安全的，我们只读取不写入
        let mmap = unsafe { Mmap::map(&file) }
            .with_context(|| format!("Failed to mmap SPF file: {}", path.display()))?;

        let len = mmap.len();
        if len < std::mem::size_of::<SpfVersion>() + std::mem::size_of::<SpfHeader>() {
            bail!("SPF file too small: {} bytes", len);
        }

        // 从文件末尾读取版本号（最后 4 字节）
        let version_offset = len - std::mem::size_of::<SpfVersion>();
        let version: SpfVersion = bytemuck::pod_read_unaligned(&mmap[version_offset..]);

        // 读取 SPF 头（版本号前 40 字节）
        let header_offset = version_offset - std::mem::size_of::<SpfHeader>();
        let header: SpfHeader = bytemuck::pod_read_unaligned(&mmap[header_offset..]);

        // 验证版本号
        if version != SPF_VERSION {
            bail!("Unsupported SPF version: {} (expected {})", version, SPF_VERSION);
        }

        Ok(Self { mmap, header, version })
    }
}
```

**Step 2: 添加访问器方法**

```rust
impl SpfReader {
    /// 获取 SPF 版本号
    pub fn version(&self) -> SpfVersion {
        self.version
    }

    /// 获取 SPF 文件头
    pub fn header(&self) -> &SpfHeader {
        &self.header
    }

    /// 获取文件数量
    pub fn file_count(&self) -> usize {
        self.header.header_size as usize / std::mem::size_of::<FInfo>()
    }

    /// 获取所有文件信息（FINFO 数组）
    pub fn file_infos(&self) -> &[FInfo] {
        let len = self.mmap.len();
        let header_size = std::mem::size_of::<SpfHeader>();
        let version_size = std::mem::size_of::<SpfVersion>();
        let finfo_size = std::mem::size_of::<FInfo>();

        let index_start = len - version_size - header_size - self.header.header_size as usize;
        let count = self.file_count();

        // SAFETY: FInfo 是 Pod 类型，字节布局保证正确
        bytemuck::cast_slice(&self.mmap[index_start..index_start + count * finfo_size])
    }

    /// 获取指定文件的原始数据（零拷贝）
    pub fn get_file_data(&self, finfo: &FInfo) -> &[u8] {
        let start = finfo.offset as usize;
        let end = start + finfo.size as usize;
        &self.mmap[start..end]
    }
}
```

**Step 3: 添加解包方法**

```rust
impl SpfReader {
    /// 解包所有文件到指定目录
    pub fn unpack(&self, output_dir: &Path) -> Result<()> {
        use std::fs;
        use std::io::Write;

        let finfos = self.file_infos();

        for finfo in finfos {
            let file_name = finfo.file_name_str();
            let output_path = output_dir.join(file_name);

            // 创建父目录
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            }

            // 写入文件数据
            let data = self.get_file_data(finfo);
            let mut file = fs::File::create(&output_path)
                .with_context(|| format!("Failed to create file: {}", output_path.display()))?;
            file.write_all(data)
                .with_context(|| format!("Failed to write file: {}", output_path.display()))?;
        }

        Ok(())
    }
}
```

**Step 4: 验证编译**

Run: `cargo build`
Expected: 成功

**Step 5: Commit**

```bash
git add src/spf/reader.rs
git commit -m "feat(spf): implement SpfReader with zero-copy mmap"
```

---

## Task 5: SPF 写入器

**Files:**
- Create: `src/spf/writer.rs`

**Step 1: 定义 SpfWriter 结构**

```rust
use crate::spf::{FInfo, ResId, SpfHeader, SpfVersion, SPF_VERSION, DESC_SIZE};
use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// SPF 文件写入器
pub struct SpfWriter {
    file_id: u8,
    desc: [u8; 32],
    /// 文件数据，使用 BTreeMap 保证按文件名排序
    files: BTreeMap<String, Vec<u8>>,
}

impl SpfWriter {
    /// 创建新的 SPF 写入器
    pub fn new(file_id: u8) -> Self {
        Self {
            file_id,
            desc: [0u8; DESC_SIZE],
            files: BTreeMap::new(),
        }
    }

    /// 设置描述信息
    pub fn set_desc(&mut self, desc: &str) {
        let bytes = desc.as_bytes();
        let len = bytes.len().min(DESC_SIZE - 1);
        self.desc[..len].copy_from_slice(&bytes[..len]);
    }

    /// 添加文件
    pub fn add_file(&mut self, name: String, data: Vec<u8>) {
        self.files.insert(name, data);
    }
}
```

**Step 2: 实现从目录读取**

```rust
impl SpfWriter {
    /// 从目录扫描并添加所有文件
    /// prefix 是 SPF 内部路径前缀（如 "CHAR/HOSHIM"）
    pub fn add_from_dir(&mut self, data_dir: &Path, prefix: &str) -> Result<()> {
        use walkdir::WalkDir;

        let prefix_path = data_dir.join(prefix);
        if !prefix_path.exists() {
            anyhow::bail!("Directory not found: {}", prefix_path.display());
        }

        for entry in WalkDir::new(&prefix_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let full_path = entry.path();
            let relative = full_path.strip_prefix(data_dir)
                .context("Failed to strip prefix")?;

            let name = relative.to_string_lossy().replace('\\', "/");
            let data = std::fs::read(full_path)
                .with_context(|| format!("Failed to read: {}", full_path.display()))?;

            self.add_file(name, data);
        }

        Ok(())
    }
}
```

**Step 3: 实现写入方法**

```rust
impl SpfWriter {
    /// 写入 SPF 文件
    pub fn write(&self, output_path: &Path) -> Result<()> {
        let file = File::create(output_path)
            .with_context(|| format!("Failed to create: {}", output_path.display()))?;
        let mut writer = BufWriter::new(file);

        let finfo_size = std::mem::size_of::<FInfo>();
        let file_count = self.files.len();
        let header_size = (file_count * finfo_size) as i32;

        // 1. 写入文件数据区，同时计算偏移量
        let mut finfos: Vec<FInfo> = Vec::with_capacity(file_count);
        let mut current_offset: i32 = 0;

        for (name, data) in &self.files {
            // 构建文件名数组
            let mut file_name = [0u8; 128];
            let name_bytes = name.as_bytes();
            let len = name_bytes.len().min(127);
            file_name[..len].copy_from_slice(&name_bytes[..len]);

            // 构建 FINFO
            let finfo = FInfo {
                file_name,
                offset: current_offset,
                size: data.len() as i32,
                res_id: ResId::new(self.file_id, finfos.len() as u32),
            };
            finfos.push(finfo);

            // 写入数据
            writer.write_all(data)
                .context("Failed to write file data")?;
            current_offset += data.len() as i32;
        }

        // 2. 写入 FINFO 索引表
        for finfo in &finfos {
            let bytes: &[u8] = bytemuck::bytes_of(finfo);
            writer.write_all(bytes)
                .context("Failed to write FINFO")?;
        }

        // 3. 写入 SPF 头
        let header = SpfHeader {
            header_size,
            file_id: self.file_id as i32,
            desc: self.desc,
        };
        let header_bytes: &[u8] = bytemuck::bytes_of(&header);
        writer.write_all(header_bytes)
            .context("Failed to write SPF header")?;

        // 4. 写入版本号
        let version_bytes: &[u8] = bytemuck::bytes_of(&SPF_VERSION);
        writer.write_all(version_bytes)
            .context("Failed to write version")?;

        writer.flush().context("Failed to flush")?;

        Ok(())
    }
}
```

**Step 4: 验证编译**

Run: `cargo build`
Expected: 成功

**Step 5: Commit**

```bash
git add src/spf/writer.rs
git commit -m "feat(spf): implement SpfWriter for packing files"
```

---

## Task 6: CLI 命令实现

**Files:**
- Modify: `src/main.rs`

**Step 1: 完整的 CLI 实现**

```rust
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use latale_tools::spf::{SpfReader, SpfRegistry, SpfWriter};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "latale-spf")]
#[command(about = "LaTale SPF resource pack/unpack tool")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Unpack SPF file to directory
    Unpack {
        /// SPF file to unpack
        spf_file: PathBuf,
        /// Output directory (default: current directory)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Pack directory into SPF file
    Pack {
        /// SPF file name to create (determines FILE_ID and source path)
        spf_file: String,
        /// Output directory (default: current directory)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Source DATA directory (default: ./DATA)
        #[arg(long)]
        data_dir: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Unpack { spf_file, output } => {
            cmd_unpack(&spf_file, output.as_deref())?;
        }
        Commands::Pack { spf_file, output, data_dir } => {
            cmd_pack(&spf_file, output.as_deref(), data_dir.as_deref())?;
        }
    }

    Ok(())
}

fn cmd_unpack(spf_file: &std::path::Path, output: Option<&std::path::Path>) -> Result<()> {
    let output_dir = output.unwrap_or_else(|| std::path::Path::new("."));

    println!("Opening {}...", spf_file.display());
    let reader = SpfReader::open(spf_file)
        .with_context(|| format!("Failed to open SPF: {}", spf_file.display()))?;

    println!("SPF version: {}", reader.version());
    println!("File ID: {}", reader.header().file_id);
    println!("Description: {}", reader.header().desc_str());
    println!("File count: {}", reader.file_count());

    println!("Unpacking to {}...", output_dir.display());
    reader.unpack(output_dir)
        .context("Failed to unpack SPF")?;

    println!("Done!");
    Ok(())
}

fn cmd_pack(spf_name: &str, output: Option<&std::path::Path>, data_dir: Option<&std::path::Path>) -> Result<()> {
    let registry = SpfRegistry::find_by_name(spf_name)
        .with_context(|| format!("Unknown SPF name: {}", spf_name))?;

    let data_dir = data_dir.unwrap_or_else(|| std::path::Path::new("DATA"));
    let output_dir = output.unwrap_or_else(|| std::path::Path::new("."));

    println!("Packing {} (FILE_ID={})", registry.name, registry.file_id);
    println!("Source: {}/{}", data_dir.display(), registry.path_prefix);

    let mut writer = SpfWriter::new(registry.file_id);
    writer.add_from_dir(data_dir, registry.path_prefix)
        .context("Failed to read source files")?;

    let output_path = output_dir.join(format!("{}.SPF", registry.name));
    println!("Output: {}", output_path.display());

    writer.write(&output_path)
        .context("Failed to write SPF")?;

    println!("Done!");
    Ok(())
}
```

**Step 2: 验证编译**

Run: `cargo build`
Expected: 成功

**Step 3: 测试基本功能**

Run: `cargo run -- --help`
Expected: 显示帮助信息

**Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat(cli): implement unpack and pack commands"
```

---

## Task 7: 测试 MD5 一致性

**Step 1: 构建发布版本**

Run: `cargo build --release`
Expected: 成功

**Step 2: 测试解包 CVOICE.SPF（较小的文件）**

Run: `./target/release/latale-spf unpack CVOICE.SPF -o ./test_output/`
Expected: 成功解包

**Step 3: 验证生成的文件**

Run: `ls ./test_output/DATA/CVOICE/ | head -10`
Expected: 显示解包的文件列表

**Step 4: 测试重新打包**

Run: `./target/release/latale-spf pack CVOICE.SPF -o ./test_packed/`
Expected: 成功打包

**Step 5: 比较 MD5**

Run: `md5 CVOICE.SPF && md5 ./test_packed/CVOICE.SPF`
Expected: 两个 MD5 值相同

**Step 6: 清理测试文件**

Run: `rm -rf ./test_output/ ./test_packed/`

---

## Task 8: 添加进度显示

**Files:**
- Modify: `src/spf/reader.rs`
- Modify: `src/spf/writer.rs`

**Step 1: 在 reader.rs 中添加进度条**

```rust
// 在 unpack 方法中添加 indicatif 进度条
use indicatif::{ProgressBar, ProgressStyle};

pub fn unpack(&self, output_dir: &Path) -> Result<()> {
    // ... 现有代码 ...

    let pb = ProgressBar::new(finfos.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
        .unwrap());

    for finfo in finfos {
        pb.set_message(finfo.file_name_str().to_string());
        // ... 写入文件 ...
        pb.inc(1);
    }

    pb.finish_with_message("Done");
    Ok(())
}
```

**Step 2: 在 writer.rs 中添加进度条**

```rust
// 在 add_from_dir 和 write 方法中添加进度条
```

**Step 3: 验证编译**

Run: `cargo build`
Expected: 成功

**Step 4: Commit**

```bash
git add src/spf/reader.rs src/spf/writer.rs
git commit -m "feat: add progress bars for unpack and pack operations"
```

---

## Task 9: 最终验证与清理

**Step 1: 运行完整测试**

Run: `cargo build --release && ./target/release/latale-spf --help`

**Step 2: 测试一个完整的 SPF 往返**

Run: 完整的解包-打包-MD5 验证流程

**Step 3: 确保代码质量**

Run: `cargo clippy -- -D warnings`
Expected: 无警告

**Step 4: 最终 commit**

```bash
git add -A
git commit -m "feat: complete SPF CLI tool implementation"
```

---

## Notes

1. **MD5 一致性关键点：**
   - 文件必须按文件名排序（BTreeMap 保证）
   - INSTANCE_ID 按顺序生成（0, 1, 2...）
   - 所有字节精确写入，不做转换

2. **潜在问题：**
   - 如果原始 SPF 不是按文件名排序，MD5 会不同
   - 需要用实际数据验证排序假设
