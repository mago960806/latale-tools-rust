# LDT 工具实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 创建独立的 `latale-ldt` CLI 工具，支持 LDT 数据库文件的读取、写入和 CSV 格式双向转换。

**Architecture:** 模块化设计，与 SPF 工具类似。`ldt/types.rs` 定义二进制结构，`reader.rs` 使用 mmap 零拷贝读取，`writer.rs` 写入，`csv.rs` 处理格式转换。独立的 `latale-ldt` binary。

**Tech Stack:** Rust, clap (CLI), memmap2 (零拷贝), bytemuck (二进制转换), csv crate (CSV 读写), anyhow (错误处理)

---

## Task 1: 添加依赖和创建 ldt 模块结构

**Files:**
- Modify: `Cargo.toml`
- Create: `src/ldt/mod.rs`
- Modify: `src/lib.rs`
- Create: `src/bin/latale-ldt.rs` (空文件占位)

**Step 1: 添加 csv 依赖到 Cargo.toml**

在 `[dependencies]` 部分添加：

```toml
csv = "1.3"
```

**Step 2: 创建 ldt 模块**

创建 `src/ldt/mod.rs`：

```rust
mod types;
mod reader;
mod writer;
mod csv;

pub use types::*;
pub use reader::LdtReader;
pub use writer::LdtWriter;
```

**Step 3: 更新 lib.rs**

修改 `src/lib.rs`：

```rust
pub mod spf;
pub mod ldt;
```

**Step 4: 创建 CLI 入口占位**

创建 `src/bin/latale-ldt.rs`：

```rust
fn main() {
    println!("latale-ldt - LDT database tool");
}
```

**Step 5: 添加 binary 到 Cargo.toml**

在 `[[bin]]` 部分后添加：

```toml
[[bin]]
name = "latale-ldt"
path = "src/bin/latale-ldt.rs"
```

**Step 6: 验证编译**

```bash
cargo build
```

Expected: 编译成功

**Step 7: Commit**

```bash
git add Cargo.toml src/lib.rs src/ldt/mod.rs src/bin/latale-ldt.rs
git commit -m "feat: add ldt module structure and latale-ldt binary"
```

---

## Task 2: 实现类型定义 (types.rs)

**Files:**
- Create: `src/ldt/types.rs`

**Step 1: 创建类型定义文件**

创建 `src/ldt/types.rs`：

```rust
use bytemuck::{Pod, Zeroable};

/// 最大字段数量
pub const MAX_FIELDS: usize = 128;

/// 字段名最大长度（字节）
pub const FIELD_NAME_SIZE: usize = 64;

/// 文件头大小
pub const HEADER_SIZE: usize = 8708;

/// LDT 字段类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum FieldType {
    /// 无效字段
    NA = 0,
    /// 字符串（最大 8192 字节）
    String = 1,
    /// 布尔值
    TF = 2,
    /// 32位整数
    Num = 3,
    /// 浮点数（百分比）
    Per = 4,
    /// 外键引用
    FID = 5,
    /// 别名（最大 4096 字节）
    Alias = 6,
    /// 64位整数
    Num64 = 7,
}

impl FieldType {
    /// 从 i32 转换，无效值返回 NA
    pub fn from_i32(value: i32) -> Self {
        match value {
            0 => Self::NA,
            1 => Self::String,
            2 => Self::TF,
            3 => Self::Num,
            4 => Self::Per,
            5 => Self::FID,
            6 => Self::Alias,
            7 => Self::Num64,
            _ => Self::NA,
        }
    }

    /// 获取 CSV 类型名称
    pub fn csv_type_name(&self) -> &'static str {
        match self {
            Self::NA => "na",
            Self::String => "string",
            Self::TF => "bool",
            Self::Num => "int32",
            Self::Per => "float32",
            Self::FID => "fid",
            Self::Alias => "alias",
            Self::Num64 => "int64",
        }
    }

    /// 从 CSV 类型名称解析
    pub fn from_csv_type_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "na" => Some(Self::NA),
            "string" => Some(Self::String),
            "bool" => Some(Self::TF),
            "int32" => Some(Self::Num),
            "float32" => Some(Self::Per),
            "fid" => Some(Self::FID),
            "alias" => Some(Self::Alias),
            "int64" => Some(Self::Num64),
            _ => None,
        }
    }

    /// 是否是变长类型
    pub fn is_variable_length(&self) -> bool {
        matches!(self, Self::String | Self::Alias | Self::FID)
    }
}

/// LDT 字段值
#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    NA,
    String(String),
    TF(bool),
    Num(i32),
    Per(f32),
    FID(i32),
    Alias(String),
    Num64(i64),
}

impl FieldValue {
    /// 获取字段类型
    pub fn field_type(&self) -> FieldType {
        match self {
            Self::NA => FieldType::NA,
            Self::String(_) => FieldType::String,
            Self::TF(_) => FieldType::TF,
            Self::Num(_) => FieldType::Num,
            Self::Per(_) => FieldType::Per,
            Self::FID(_) => FieldType::FID,
            Self::Alias(_) => FieldType::Alias,
            Self::Num64(_) => FieldType::Num64,
        }
    }

    /// 转换为 CSV 字符串
    pub fn to_csv_string(&self) -> String {
        match self {
            Self::NA => String::new(),
            Self::String(s) => s.clone(),
            Self::TF(b) => b.to_string(),
            Self::Num(n) => n.to_string(),
            Self::Per(f) => f.to_string(),
            Self::FID(id) => id.to_string(),
            Self::Alias(s) => s.clone(),
            Self::Num64(n) => n.to_string(),
        }
    }

    /// 从 CSV 字符串解析
    pub fn from_csv_string(s: &str, ty: FieldType) -> Self {
        if s.is_empty() {
            return Self::NA;
        }
        match ty {
            FieldType::NA => Self::NA,
            FieldType::String => Self::String(s.to_string()),
            FieldType::TF => Self::TF(s.eq_ignore_ascii_case("true")),
            FieldType::Num => Self::Num(s.parse().unwrap_or(0)),
            FieldType::Per => Self::Per(s.parse().unwrap_or(0.0)),
            FieldType::FID => Self::FID(s.parse().unwrap_or(0)),
            FieldType::Alias => Self::Alias(s.to_string()),
            FieldType::Num64 => Self::Num64(s.parse().unwrap_or(0)),
        }
    }
}

/// 字段定义
#[derive(Debug, Clone)]
pub struct FieldDef {
    /// 字段名
    pub name: String,
    /// 字段类型
    pub field_type: FieldType,
}

/// LDT 文件头（8708 字节）
#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct LdtHeader {
    /// 数据库 ID
    pub db_id: i32,
    /// 字段数量
    pub num_fields: i32,
    /// 数据行数
    pub num_rows: i32,
    /// 字段名称数组（128 * 64 = 8192 字节）
    pub field_names: [[u8; FIELD_NAME_SIZE]; MAX_FIELDS],
    /// 字段类型数组（128 * 4 = 512 字节）
    pub field_types: [i32; MAX_FIELDS],
}

const _: () = assert!(std::mem::size_of::<LdtHeader>() == HEADER_SIZE);

impl LdtHeader {
    /// 获取字段名（去除尾部 null）
    pub fn field_name(&self, index: usize) -> String {
        if index >= MAX_FIELDS {
            return String::new();
        }
        let bytes = &self.field_names[index];
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(FIELD_NAME_SIZE);
        String::from_utf8_lossy(&bytes[..end]).into_owned()
    }

    /// 获取字段类型
    pub fn field_type(&self, index: usize) -> FieldType {
        if index >= MAX_FIELDS {
            return FieldType::NA;
        }
        FieldType::from_i32(self.field_types[index])
    }

    /// 获取所有字段定义
    pub fn field_defs(&self) -> Vec<FieldDef> {
        let count = self.num_fields as usize;
        let mut defs = Vec::with_capacity(count);
        for i in 0..count {
            defs.push(FieldDef {
                name: self.field_name(i),
                field_type: self.field_type(i),
            });
        }
        defs
    }
}

/// 数据行
#[derive(Debug, Clone)]
pub struct Row {
    /// 主键
    pub primary_key: i64,
    /// 字段值
    pub values: Vec<FieldValue>,
}
```

**Step 2: 验证编译**

```bash
cargo build
```

Expected: 编译成功

**Step 3: Commit**

```bash
git add src/ldt/types.rs
git commit -m "feat(ldt): add type definitions (FieldType, FieldValue, LdtHeader, Row)"
```

---

## Task 3: 实现 LdtReader (reader.rs)

**Files:**
- Create: `src/ldt/reader.rs`

**Step 1: 创建 reader 文件**

创建 `src/ldt/reader.rs`：

```rust
use crate::ldt::{FieldDef, FieldValue, FieldType, LdtHeader, Row, HEADER_SIZE};
use anyhow::{bail, Context, Result};
use memmap2::Mmap;
use std::fs::File;
use std::path::Path;

/// LDT 文件读取器
pub struct LdtReader {
    mmap: Mmap,
    header: LdtHeader,
    /// 数据区起始偏移
    data_offset: usize,
}

impl LdtReader {
    /// 打开 LDT 文件
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path)
            .with_context(|| format!("Failed to open LDT file: {}", path.display()))?;

        let mmap = unsafe { Mmap::map(&file) }
            .with_context(|| format!("Failed to mmap LDT file: {}", path.display()))?;

        let len = mmap.len();
        if len < HEADER_SIZE {
            bail!("LDT file too small: {} bytes (need at least {})", len, HEADER_SIZE);
        }

        // 读取文件头
        let header: LdtHeader = bytemuck::pod_read_unaligned(&mmap[0..HEADER_SIZE]);

        Ok(Self {
            mmap,
            header,
            data_offset: HEADER_SIZE,
        })
    }

    /// 获取文件头
    pub fn header(&self) -> &LdtHeader {
        &self.header
    }

    /// 获取字段定义列表
    pub fn field_defs(&self) -> Vec<FieldDef> {
        self.header.field_defs()
    }

    /// 获取数据行数
    pub fn row_count(&self) -> usize {
        self.header.num_rows as usize
    }

    /// 获取数据库名称（从文件名推断）
    pub fn database_name(&self) -> String {
        format!("DB_{}", self.header.db_id)
    }

    /// 读取所有数据行
    pub fn read_rows(&self) -> Result<Vec<Row>> {
        let mut rows = Vec::with_capacity(self.row_count());
        let mut offset = self.data_offset;
        let field_defs = self.field_defs();

        for _ in 0..self.row_count() {
            let (row, new_offset) = self.read_row(offset, &field_defs)?;
            rows.push(row);
            offset = new_offset;
        }

        Ok(rows)
    }

    /// 读取单行数据
    fn read_row(&self, offset: usize, field_defs: &[FieldDef]) -> Result<(Row, usize)> {
        let mut current_offset = offset;

        // 读取主键（8 字节）
        if current_offset + 8 > self.mmap.len() {
            bail!("Unexpected end of file while reading primary key");
        }
        let primary_key: i64 = bytemuck::pod_read_unaligned(
            &self.mmap[current_offset..current_offset + 8]
        );
        current_offset += 8;

        // 读取字段值
        let mut values = Vec::with_capacity(field_defs.len());
        for def in field_defs {
            let (value, new_offset) = self.read_field_value(current_offset, def.field_type)?;
            values.push(value);
            current_offset = new_offset;
        }

        Ok((Row { primary_key, values }, current_offset))
    }

    /// 读取单个字段值
    fn read_field_value(&self, offset: usize, ty: FieldType) -> Result<(FieldValue, usize)> {
        match ty {
            FieldType::NA => Ok((FieldValue::NA, offset)),
            FieldType::Num => {
                if offset + 4 > self.mmap.len() {
                    bail!("Unexpected end of file while reading Num");
                }
                let value: i32 = bytemuck::pod_read_unaligned(&self.mmap[offset..offset + 4]);
                Ok((FieldValue::Num(value), offset + 4))
            }
            FieldType::Per => {
                if offset + 4 > self.mmap.len() {
                    bail!("Unexpected end of file while reading Per");
                }
                let value: f32 = bytemuck::pod_read_unaligned(&self.mmap[offset..offset + 4]);
                Ok((FieldValue::Per(value), offset + 4))
            }
            FieldType::TF => {
                if offset + 4 > self.mmap.len() {
                    bail!("Unexpected end of file while reading TF");
                }
                let value: i32 = bytemuck::pod_read_unaligned(&self.mmap[offset..offset + 4]);
                Ok((FieldValue::TF(value != 0), offset + 4))
            }
            FieldType::Num64 => {
                if offset + 8 > self.mmap.len() {
                    bail!("Unexpected end of file while reading Num64");
                }
                let value: i64 = bytemuck::pod_read_unaligned(&self.mmap[offset..offset + 8]);
                Ok((FieldValue::Num64(value), offset + 8))
            }
            FieldType::String | FieldType::Alias | FieldType::FID => {
                // 变长字符串：4 字节长度 + 数据
                if offset + 4 > self.mmap.len() {
                    bail!("Unexpected end of file while reading string length");
                }
                let len: i32 = bytemuck::pod_read_unaligned(&self.mmap[offset..offset + 4]);
                let len = len as usize;

                if offset + 4 + len > self.mmap.len() {
                    bail!("Unexpected end of file while reading string data");
                }

                let data = &self.mmap[offset + 4..offset + 4 + len];
                let s = String::from_utf8_lossy(data).into_owned();

                let value = match ty {
                    FieldType::String => FieldValue::String(s),
                    FieldType::Alias => FieldValue::Alias(s),
                    FieldType::FID => FieldValue::FID(s.parse().unwrap_or(0)),
                    _ => unreachable!(),
                };

                Ok((value, offset + 4 + len))
            }
        }
    }
}
```

**Step 2: 验证编译**

```bash
cargo build
```

Expected: 编译成功

**Step 3: Commit**

```bash
git add src/ldt/reader.rs
git commit -m "feat(ldt): add LdtReader with mmap-based zero-copy reading"
```

---

## Task 4: 实现 LdtWriter (writer.rs)

**Files:**
- Create: `src/ldt/writer.rs`

**Step 1: 创建 writer 文件**

创建 `src/ldt/writer.rs`：

```rust
use crate::ldt::{FieldDef, FieldType, FieldValue, LdtHeader, Row, MAX_FIELDS, FIELD_NAME_SIZE, HEADER_SIZE};
use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// LDT 文件写入器
pub struct LdtWriter {
    db_id: i32,
    field_defs: Vec<FieldDef>,
    rows: Vec<Row>,
}

impl LdtWriter {
    /// 创建新的 LDT 写入器
    pub fn new(db_id: i32) -> Self {
        Self {
            db_id,
            field_defs: Vec::new(),
            rows: Vec::new(),
        }
    }

    /// 添加字段定义
    pub fn add_field(&mut self, name: &str, field_type: FieldType) {
        self.field_defs.push(FieldDef {
            name: name.to_string(),
            field_type,
        });
    }

    /// 添加数据行
    pub fn add_row(&mut self, row: Row) {
        self.rows.push(row);
    }

    /// 获取字段定义
    pub fn field_defs(&self) -> &[FieldDef] {
        &self.field_defs
    }

    /// 获取行数
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// 从字段定义和行数据创建 Writer
    pub fn from_data(db_id: i32, field_defs: Vec<FieldDef>, rows: Vec<Row>) -> Self {
        Self {
            db_id,
            field_defs,
            rows,
        }
    }

    /// 写入 LDT 文件
    pub fn write(&self, path: &Path) -> Result<()> {
        // 确保父目录存在
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        let file = File::create(path)
            .with_context(|| format!("Failed to create LDT file: {}", path.display()))?;
        let mut writer = BufWriter::new(file);

        // 1. 构建并写入文件头
        let header = self.build_header();
        let header_bytes: &[u8] = bytemuck::bytes_of(&header);
        writer.write_all(header_bytes)
            .context("Failed to write LDT header")?;

        // 2. 写入数据行
        for row in &self.rows {
            self.write_row(&mut writer, row)?;
        }

        writer.flush()
            .context("Failed to flush LDT file")?;

        Ok(())
    }

    /// 构建文件头
    fn build_header(&self) -> LdtHeader {
        let mut header = LdtHeader {
            db_id: self.db_id,
            num_fields: self.field_defs.len() as i32,
            num_rows: self.rows.len() as i32,
            field_names: [[0u8; FIELD_NAME_SIZE]; MAX_FIELDS],
            field_types: [0i32; MAX_FIELDS],
        };

        for (i, def) in self.field_defs.iter().enumerate() {
            // 写入字段名
            let name_bytes = def.name.as_bytes();
            let len = name_bytes.len().min(FIELD_NAME_SIZE - 1);
            header.field_names[i][..len].copy_from_slice(&name_bytes[..len]);

            // 写入字段类型
            header.field_types[i] = def.field_type as i32;
        }

        header
    }

    /// 写入单行数据
    fn write_row<W: Write>(&self, writer: &mut W, row: &Row) -> Result<()> {
        // 写入主键
        writer.write_all(&row.primary_key.to_le_bytes())
            .context("Failed to write primary key")?;

        // 写入字段值
        for (i, value) in row.values.iter().enumerate() {
            let expected_type = self.field_defs.get(i)
                .map(|d| d.field_type)
                .unwrap_or(FieldType::NA);
            self.write_field_value(writer, value, expected_type)?;
        }

        Ok(())
    }

    /// 写入单个字段值
    fn write_field_value<W: Write>(&self, writer: &mut W, value: &FieldValue, expected_type: FieldType) -> Result<()> {
        match value {
            FieldValue::NA => {
                // NA 类型不写入数据
            }
            FieldValue::Num(n) => {
                writer.write_all(&n.to_le_bytes())
                    .context("Failed to write Num")?;
            }
            FieldValue::Per(f) => {
                writer.write_all(&f.to_le_bytes())
                    .context("Failed to write Per")?;
            }
            FieldValue::TF(b) => {
                let v: i32 = if *b { 1 } else { 0 };
                writer.write_all(&v.to_le_bytes())
                    .context("Failed to write TF")?;
            }
            FieldValue::Num64(n) => {
                writer.write_all(&n.to_le_bytes())
                    .context("Failed to write Num64")?;
            }
            FieldValue::String(s) => {
                let bytes = s.as_bytes();
                writer.write_all(&(bytes.len() as i32).to_le_bytes())
                    .context("Failed to write string length")?;
                writer.write_all(bytes)
                    .context("Failed to write string data")?;
            }
            FieldValue::Alias(s) => {
                let bytes = s.as_bytes();
                writer.write_all(&(bytes.len() as i32).to_le_bytes())
                    .context("Failed to write alias length")?;
                writer.write_all(bytes)
                    .context("Failed to write alias data")?;
            }
            FieldValue::FID(id) => {
                // FID 存储为字符串（长度 + 数据）
                let s = id.to_string();
                let bytes = s.as_bytes();
                writer.write_all(&(bytes.len() as i32).to_le_bytes())
                    .context("Failed to write FID length")?;
                writer.write_all(bytes)
                    .context("Failed to write FID data")?;
            }
        }

        Ok(())
    }
}
```

**Step 2: 验证编译**

```bash
cargo build
```

Expected: 编译成功

**Step 3: Commit**

```bash
git add src/ldt/writer.rs
git commit -m "feat(ldt): add LdtWriter for writing LDT files"
```

---

## Task 5: 实现 CSV 转换 (csv.rs)

**Files:**
- Create: `src/ldt/csv.rs`

**Step 1: 创建 csv 转换文件**

创建 `src/ldt/csv.rs`：

```rust
use crate::ldt::{FieldDef, FieldType, FieldValue, LdtWriter, Row};
use anyhow::{bail, Context, Result};
use std::io::{Read, Write};
use std::path::Path;

/// CSV 配置
pub struct CsvConfig {
    /// 数据库名称（用于注释头）
    pub database_name: String,
}

/// 将 LDT 数据导出为 CSV
pub fn export_to_csv<W: Write>(
    writer: &mut W,
    field_defs: &[FieldDef],
    rows: &[Row],
    config: &CsvConfig,
) -> Result<()> {
    // 写入头部注释
    writeln!(writer, "# database: {}", config.database_name)?;
    writeln!(writer, "# rows: {}", rows.len())?;

    // 写入列头
    let headers: Vec<String> = field_defs
        .iter()
        .map(|def| format!("{}:{}", def.name, def.field_type.csv_type_name()))
        .collect();
    {
        let mut csv_writer = csv::Writer::from_writer(writer);
        csv_writer.write_record(&headers)?;
        csv_writer.flush()?;
    }

    // 写入数据行
    let mut csv_writer = csv::Writer::from_writer(writer);
    for row in rows {
        let mut record = Vec::with_capacity(row.values.len() + 1);

        // 主键作为第一列（不写入，主键在 Row 中单独存储）
        // 根据设计，主键是 8 字节整数，不在字段列表中
        // 但 CSV 中需要体现，所以作为第一列

        // 实际上，根据 LDT 格式，主键是每行的第一个 8 字节
        // 这里我们把它作为隐含列处理，或者在字段定义中包含它
        // 让我们重新设计：主键不单独列出，而是作为普通字段

        // 等等，看设计文档，主键是每行数据的一部分
        // 但在字段定义中没有明确列出
        // 让我们假设主键字段名为 "ID" 或由第一个字段表示

        // 重新理解：LDT 行 = 主键(8字节) + 字段值
        // 主键不在 field_defs 中，需要单独处理
        // CSV 中我们把主键作为第一列，类型为 int64

        // 先写主键
        record.push(row.primary_key.to_string());

        // 写入字段值
        for value in &row.values {
            record.push(value.to_csv_string());
        }

        csv_writer.write_record(&record)?;
    }

    csv_writer.flush()?;
    Ok(())
}

/// CSV 导入结果
pub struct CsvImportResult {
    /// 数据库 ID
    pub db_id: i32,
    /// 字段定义（包含主键作为第一个字段）
    pub field_defs: Vec<FieldDef>,
    /// 数据行
    pub rows: Vec<Row>,
}

/// 从 CSV 导入数据
pub fn import_from_csv<R: Read>(reader: &mut R) -> Result<CsvImportResult> {
    let mut content = String::new();
    reader.read_to_string(&mut content)?;

    let mut lines = content.lines();

    // 解析头部注释
    let mut database_name = String::new();
    let mut _expected_rows: usize = 0;

    for line in &mut lines {
        let line = line.trim();
        if line.starts_with('#') {
            if let Some(rest) = line.strip_prefix("# database:") {
                database_name = rest.trim().to_string();
            } else if let Some(rest) = line.strip_prefix("# rows:") {
                _expected_rows = rest.trim().parse().unwrap_or(0);
            }
        } else if !line.is_empty() {
            // 遇到非注释行，回退处理
            break;
        }
    }

    // 收集剩余内容
    let remaining: String = lines.collect::<Vec<_>>().join("\n");
    let header_line = remaining.lines().next()
        .context("CSV has no header line")?;

    // 解析列头
    let headers: Vec<&str> = csv::Reader::from_reader(header_line.as_bytes())
        .headers()
        .context("Failed to read CSV headers")?
        .iter()
        .collect();

    let mut field_defs = Vec::with_capacity(headers.len());

    for (i, header) in headers.iter().enumerate() {
        let parts: Vec<&str> = header.split(':').collect();
        let (name, type_name) = if parts.len() == 2 {
            (parts[0], parts[1])
        } else if parts.len() == 1 {
            // 无类型标注，根据位置推断
            if i == 0 {
                (parts[0], "int64") // 第一列默认为主键
            } else {
                (parts[0], "string")
            }
        } else {
            bail!("Invalid column header: {}", header);
        };

        let field_type = FieldType::from_csv_type_name(type_name)
            .with_context(|| format!("Unknown type: {}", type_name))?;

        field_defs.push(FieldDef {
            name: name.to_string(),
            field_type,
        });
    }

    // 解析数据行
    let mut csv_reader = csv::Reader::from_reader(remaining.as_bytes());
    let mut rows = Vec::new();

    for result in csv_reader.records() {
        let record = result.context("Failed to read CSV record")?;

        if record.len() != field_defs.len() {
            bail!(
                "Column count mismatch: expected {}, got {}",
                field_defs.len(),
                record.len()
            );
        }

        // 第一列是主键
        let primary_key: i64 = record.get(0)
            .context("Missing primary key column")?
            .parse()
            .context("Invalid primary key value")?;

        // 解析字段值（从第二列开始）
        let mut values = Vec::with_capacity(field_defs.len() - 1);
        for (i, value_str) in record.iter().skip(1).enumerate() {
            let field_type = field_defs.get(i + 1)
                .map(|d| d.field_type)
                .unwrap_or(FieldType::NA);
            values.push(FieldValue::from_csv_string(value_str, field_type));
        }

        rows.push(Row { primary_key, values });
    }

    // 从数据库名提取 db_id
    let db_id = database_name
        .strip_prefix("DB_")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    // 移除第一个字段定义（主键），因为它不在字段值中
    if !field_defs.is_empty() {
        field_defs.remove(0);
    }

    Ok(CsvImportResult {
        db_id,
        field_defs,
        rows,
    })
}

/// 从 CSV 文件创建 LdtWriter
pub fn csv_to_writer<R: Read>(reader: &mut R) -> Result<LdtWriter> {
    let result = import_from_csv(reader)?;
    Ok(LdtWriter::from_data(result.db_id, result.field_defs, result.rows))
}

/// 将 LdtReader 数据导出到 CSV 文件
pub fn ldt_to_csv<W: Write>(
    writer: &mut W,
    db_id: i32,
    field_defs: &[FieldDef],
    rows: &[Row],
    database_name: &str,
) -> Result<()> {
    let config = CsvConfig {
        database_name: database_name.to_string(),
    };

    // 重新构建字段定义，添加主键作为第一列
    let mut all_fields = vec![FieldDef {
        name: "ID".to_string(),
        field_type: FieldType::Num64,
    }];
    all_fields.extend(field_defs.iter().cloned());

    export_to_csv(writer, &all_fields, rows, &config)
}
```

**Step 2: 验证编译**

```bash
cargo build
```

Expected: 编译成功

**Step 3: Commit**

```bash
git add src/ldt/csv.rs
git commit -m "feat(ldt): add CSV import/export functionality"
```

---

## Task 6: 实现 CLI (latale-ldt.rs)

**Files:**
- Modify: `src/bin/latale-ldt.rs`

**Step 1: 实现完整 CLI**

修改 `src/bin/latale-ldt.rs`：

```rust
use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use latale_tools::ldt::{LdtReader, ldt_to_csv, csv_to_writer};
use std::path::PathBuf;

/// 默认输入目录
const DEFAULT_INPUT_DIR: &str = "DATA/LDT";
/// 默认输出目录
const DEFAULT_OUTPUT_DIR: &str = "DATA/CSV";

#[derive(Parser)]
#[command(name = "latale-ldt")]
#[command(about = "LaTale LDT 数据库转换工具", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 智能转换：LDT ↔ CSV（自动判断方向）
    Convert {
        /// 输入文件或目录
        input: Option<PathBuf>,
        /// 输出路径
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// 显示 LDT 文件信息
    Info {
        /// LDT 文件路径
        ldt_file: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Convert { input, output } => {
            cmd_convert(input.as_deref(), output.as_deref())?;
        }
        Commands::Info { ldt_file } => {
            cmd_info(&ldt_file)?;
        }
    }

    Ok(())
}

fn cmd_convert(input: Option<&std::path::Path>, output: Option<&std::path::Path>) -> Result<()> {
    let input = input.unwrap_or_else(|| std::path::Path::new(DEFAULT_INPUT_DIR));

    if input.is_file() {
        convert_single_file(input, output)?;
    } else if input.is_dir() {
        convert_directory(input, output)?;
    } else {
        bail!("Input path does not exist: {}", input.display());
    }

    Ok(())
}

fn convert_single_file(input: &std::path::Path, output: Option<&std::path::Path>) -> Result<()> {
    let extension = input.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    match extension.as_deref() {
        Some("ldt") => {
            // LDT → CSV
            let output_path = output.map(|p| p.to_path_buf()).unwrap_or_else(|| {
                std::path::Path::new(DEFAULT_OUTPUT_DIR)
                    .join(input.file_stem().unwrap())
                    .with_extension("csv")
            });

            println!("[转换] {} → {}", input.display(), output_path.display());
            convert_ldt_to_csv(input, &output_path)?;
            println!("[完成] 已生成 {}", output_path.display());
        }
        Some("csv") => {
            // CSV → LDT
            let output_path = output.map(|p| p.to_path_buf()).unwrap_or_else(|| {
                std::path::Path::new(DEFAULT_INPUT_DIR)
                    .join(input.file_stem().unwrap())
                    .with_extension("LDT")
            });

            println!("[转换] {} → {}", input.display(), output_path.display());
            convert_csv_to_ldt(input, &output_path)?;
            println!("[完成] 已生成 {}", output_path.display());
        }
        _ => {
            bail!("Unsupported file type: {}", input.display());
        }
    }

    Ok(())
}

fn convert_directory(input: &std::path::Path, output: Option<&std::path::Path>) -> Result<()> {
    // 检测目录中的文件类型
    let mut ldt_count = 0;
    let mut csv_count = 0;

    let entries = std::fs::read_dir(input)
        .with_context(|| format!("Failed to read directory: {}", input.display()))?;

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                match ext.to_lowercase().as_str() {
                    "ldt" => ldt_count += 1,
                    "csv" => csv_count += 1,
                    _ => {}
                }
            }
        }
    }

    if ldt_count > 0 && csv_count > 0 {
        bail!("目录中同时存在 .LDT 和 .csv 文件，无法确定转换方向。请使用单一类型文件的目录。");
    }

    if ldt_count == 0 && csv_count == 0 {
        bail!("目录中没有 .LDT 或 .csv 文件");
    }

    let output = output.unwrap_or_else(|| {
        if ldt_count > 0 {
            std::path::Path::new(DEFAULT_OUTPUT_DIR)
        } else {
            std::path::Path::new(DEFAULT_INPUT_DIR)
        }
    });

    // 检查输入输出是否相同
    let input_abs = std::fs::canonicalize(input).unwrap_or_else(|_| input.to_path_buf());
    let output_abs = if output.exists() {
        std::fs::canonicalize(output).unwrap_or_else(|_| output.to_path_buf())
    } else {
        output.to_path_buf()
    };

    if input_abs == output_abs {
        bail!("输入目录和输出目录相同，请指定不同的输出目录");
    }

    // 创建输出目录
    std::fs::create_dir_all(output)
        .with_context(|| format!("Failed to create output directory: {}", output.display()))?;

    if ldt_count > 0 {
        println!("[批量转换] LDT → CSV ({} 个文件)", ldt_count);
        let entries = std::fs::read_dir(input)
            .with_context(|| format!("Failed to read directory: {}", input.display()))?;

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().map(|e| e.to_lowercase()) == Some("ldt".to_string()) {
                let output_path = output
                    .join(path.file_stem().unwrap())
                    .with_extension("csv");
                print!("  {} → {} ... ", path.display(), output_path.display());
                convert_ldt_to_csv(&path, &output_path)?;
                println!("OK");
            }
        }
    } else {
        println!("[批量转换] CSV → LDT ({} 个文件)", csv_count);
        let entries = std::fs::read_dir(input)
            .with_context(|| format!("Failed to read directory: {}", input.display()))?;

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().map(|e| e.to_lowercase()) == Some("csv".to_string()) {
                let output_path = output
                    .join(path.file_stem().unwrap())
                    .with_extension("LDT");
                print!("  {} → {} ... ", path.display(), output_path.display());
                convert_csv_to_ldt(&path, &output_path)?;
                println!("OK");
            }
        }
    }

    println!("[完成] 输出目录: {}", output.display());
    Ok(())
}

fn convert_ldt_to_csv(input: &std::path::Path, output: &std::path::Path) -> Result<()> {
    let reader = LdtReader::open(input)
        .with_context(|| format!("Failed to open LDT file: {}", input.display()))?;

    let header = reader.header();
    let field_defs = header.field_defs();
    let rows = reader.read_rows()
        .context("Failed to read LDT data")?;

    // 确保输出目录存在
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    let mut file = std::fs::File::create(output)
        .with_context(|| format!("Failed to create CSV file: {}", output.display()))?;

    let db_name = input
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    ldt_to_csv(&mut file, header.db_id, &field_defs, &rows, db_name)
        .context("Failed to write CSV")?;

    Ok(())
}

fn convert_csv_to_ldt(input: &std::path::Path, output: &std::path::Path) -> Result<()> {
    let mut file = std::fs::File::open(input)
        .with_context(|| format!("Failed to open CSV file: {}", input.display()))?;

    let writer = csv_to_writer(&mut file)
        .context("Failed to parse CSV")?;

    // 确保输出目录存在
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    writer.write(output)
        .with_context(|| format!("Failed to write LDT file: {}", output.display()))?;

    Ok(())
}

fn cmd_info(ldt_file: &std::path::Path) -> Result<()> {
    let reader = LdtReader::open(ldt_file)
        .with_context(|| format!("无法打开 LDT 文件: {}", ldt_file.display()))?;

    let header = reader.header();

    println!("[文件信息] {}", ldt_file.display());
    println!("{}", "-".repeat(50));
    println!("数据库 ID:   {}", header.db_id);
    println!("字段数量:    {}", header.num_fields);
    println!("数据行数:    {}", header.num_rows);
    println!();
    println!("[字段列表]");
    println!("{}", "-".repeat(50));

    let field_defs = header.field_defs();
    for (i, def) in field_defs.iter().enumerate() {
        println!("  {:3}: {:<30} [{}]", i + 1, def.name, def.field_type.csv_type_name());
    }

    println!();

    Ok(())
}
```

**Step 2: 更新 ldt/mod.rs 导出**

修改 `src/ldt/mod.rs`：

```rust
mod types;
mod reader;
mod writer;
mod csv;

pub use types::*;
pub use reader::LdtReader;
pub use writer::LdtWriter;
pub use csv::{export_to_csv, import_from_csv, csv_to_writer, ldt_to_csv, CsvConfig, CsvImportResult};
```

**Step 3: 验证编译**

```bash
cargo build
```

Expected: 编译成功

**Step 4: Commit**

```bash
git add src/ldt/mod.rs src/bin/latale-ldt.rs
git commit -m "feat: implement latale-ldt CLI with convert and info commands"
```

---

## Task 7: 测试和验证

**Step 1: 构建发布版本**

```bash
cargo build --release
```

Expected: 编译成功

**Step 2: 测试 help 命令**

```bash
./target/release/latale-ldt --help
./target/release/latale-ldt convert --help
./target/release/latale-ldt info --help
```

Expected: 显示正确的帮助信息

**Step 3: 手动测试（如果有测试文件）**

如果有 LDT 测试文件：

```bash
./target/release/latale-ldt info DATA/LDT/ITEM.LDT
./target/release/latale-ldt convert DATA/LDT/ITEM.LDT
```

**Step 4: Final Commit**

```bash
git add -A
git commit -m "chore: build and verify latale-ldt tool"
```

---

## Summary

实现顺序：
1. ✅ Task 1: 模块结构和依赖
2. ✅ Task 2: 类型定义 (types.rs)
3. ✅ Task 3: LdtReader (reader.rs)
4. ✅ Task 4: LdtWriter (writer.rs)
5. ✅ Task 5: CSV 转换 (csv.rs)
6. ✅ Task 6: CLI 实现
7. ✅ Task 7: 测试验证

每个 Task 都有独立的 commit，遵循 TDD 原则和频繁提交的最佳实践。
