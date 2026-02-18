# LDT 工具编码支持设计

## 概述

为 LDT 工具添加多编码支持，使其能够处理不同地区的 LDT 文件（台湾 BIG5、中国 GBK、日本 Shift-JIS、韩国 EUC-KR 等）。

## 需求

| 场景 | 行为 |
|------|------|
| `info` 命令 | `--encoding` 指定解码编码（默认 GBK），正确显示字段名和数据 |
| `convert` LDT→CSV | `--encoding` 指定 LDT 的解码编码（默认 GBK），CSV 输出 UTF-8 |
| `convert` CSV→LDT | `--encoding` 指定 LDT 的编码（默认 GBK），从 UTF-8 CSV 读取 |
| 批量转换 | 所有文件使用统一编码 |

**编码策略**：
- LDT 文件：使用指定编码（默认 GBK）
- CSV 文件：始终使用 UTF-8

## CLI 变更

```
latale-ldt info <file> [--rows N] [--encoding <ENC>]
latale-ldt convert <input> [-o <output>] [--encoding <ENC>]
```

支持的编码名称（复用 SPF 的 `encoding_from_name` 函数）：
- `GBK` / `GB2312` / `GB18030` → GBK（默认）
- `BIG5` / `BIG-5` → BIG5（台湾）
- `EUC-KR` / `EUCKR` / `KOREAN` → EUC-KR（韩国）
- `SHIFT_JIS` / `SJIS` / `CP932` / `JAPANESE` → Shift-JIS（日本）
- `UTF-8` / `UTF8` → UTF-8

## 核心模块变更

### LdtReader

```rust
pub struct LdtReader {
    mmap: Mmap,
    header: LdtHeader,
    encoding: &'static encoding_rs::Encoding,  // 新增
}

impl LdtReader {
    pub fn open(path: &Path, encoding: &'static encoding_rs::Encoding) -> Result<Self> {
        // ... 原有逻辑 ...
        Ok(Self { mmap, header, encoding })
    }

    fn read_field_value(&self, offset: usize, field_type: FieldType) -> Result<(FieldValue, usize)> {
        // String/Alias/FID 类型解码时使用 self.encoding 替代硬编码 GBK
        let (s, _, _) = self.encoding.decode(data);
    }
}
```

### LdtWriter

```rust
pub struct LdtWriter {
    db_id: i32,
    field_defs: Vec<FieldDef>,
    rows: Vec<Row>,
    encoding: &'static encoding_rs::Encoding,  // 新增
}

impl LdtWriter {
    pub fn new(db_id: i32, encoding: &'static encoding_rs::Encoding) -> Self {
        Self {
            db_id,
            field_defs: Vec::new(),
            rows: Vec::new(),
            encoding,
        }
    }

    fn write_string<W: Write>(&self, writer: &mut W, s: &str) -> Result<()> {
        // 使用 self.encoding 替代硬编码 GBK
        let (bytes, _, _) = self.encoding.encode(s);
    }
}
```

### 输出信息增强

在 `info` 和 `convert` 的输出中显示当前使用的编码。

## 影响范围

- `src/ldt/reader.rs` - 添加 encoding 字段，修改 open 和 read_field_value
- `src/ldt/writer.rs` - 添加 encoding 字段，修改 new 和 write_gbk_string（重命名为 write_encoded_string）
- `src/ldt/mod.rs` - 可能需要导出 encoding_from_name 或相关函数
- `src/cli/latale-ldt.rs` - 添加 --encoding 参数，传递编码到 reader/writer
- 测试文件 - 更新测试以适配新 API
