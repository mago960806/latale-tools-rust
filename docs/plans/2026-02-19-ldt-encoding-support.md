# LDT Encoding Support Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add multi-encoding support to the LDT tool for handling files from different regions (BIG5, GBK, Shift-JIS, EUC-KR).

**Architecture:** Add an encoding field to LdtReader and LdtWriter structs, modify string encoding/decoding to use this field instead of hardcoded GBK. CLI adds `--encoding` parameter (default GBK) passed to all read/write operations.

**Tech Stack:** Rust, encoding_rs, clap

---

### Task 1: Add shared encoding utility

**Files:**
- Create: `src/common/encoding.rs`
- Modify: `src/common/mod.rs`

**Step 1: Create encoding utility module**

Create `src/common/encoding.rs`:

```rust
//! Encoding utilities for LaTale file formats
//!
//! Provides encoding detection and conversion for different regional versions.

/// Default encoding for LaTale files (Chinese version)
pub const DEFAULT_ENCODING: &str = "GBK";

/// Get encoding from name string
///
/// Supported encodings:
/// - `GBK` / `GB2312` / `GB18030` → GBK (Chinese, default)
/// - `BIG5` / `BIG-5` → BIG5 (Taiwan)
/// - `EUC-KR` / `EUCKR` / `KOREAN` → EUC-KR (Korean)
/// - `SHIFT_JIS` / `SHIFTJIS` / `SJIS` / `CP932` / `JAPANESE` → Shift-JIS (Japanese)
/// - `UTF-8` / `UTF8` → UTF-8
pub fn encoding_from_name(name: &str) -> &'static encoding_rs::Encoding {
    match name.to_uppercase().as_str() {
        "UTF-8" | "UTF8" => encoding_rs::UTF_8,
        "BIG5" | "BIG-5" => encoding_rs::BIG5,
        "EUC-KR" | "EUCKR" | "KOREAN" => encoding_rs::EUC_KR,
        "GBK" | "GB2312" | "GB18030" => encoding_rs::GBK,
        "SHIFT_JIS" | "SHIFTJIS" | "SJIS" | "CP932" | "JAPANESE" => encoding_rs::SHIFT_JIS,
        _ => encoding_rs::GBK,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoding_from_name_gbk() {
        assert_eq!(encoding_from_name("GBK"), encoding_rs::GBK);
        assert_eq!(encoding_from_name("gbk"), encoding_rs::GBK);
        assert_eq!(encoding_from_name("GB2312"), encoding_rs::GBK);
        assert_eq!(encoding_from_name("GB18030"), encoding_rs::GBK);
    }

    #[test]
    fn test_encoding_from_name_big5() {
        assert_eq!(encoding_from_name("BIG5"), encoding_rs::BIG5);
        assert_eq!(encoding_from_name("big5"), encoding_rs::BIG5);
        assert_eq!(encoding_from_name("BIG-5"), encoding_rs::BIG5);
    }

    #[test]
    fn test_encoding_from_name_euc_kr() {
        assert_eq!(encoding_from_name("EUC-KR"), encoding_rs::EUC_KR);
        assert_eq!(encoding_from_name("euckr"), encoding_rs::EUC_KR);
        assert_eq!(encoding_from_name("KOREAN"), encoding_rs::EUC_KR);
    }

    #[test]
    fn test_encoding_from_name_shift_jis() {
        assert_eq!(encoding_from_name("SHIFT_JIS"), encoding_rs::SHIFT_JIS);
        assert_eq!(encoding_from_name("sjis"), encoding_rs::SHIFT_JIS);
        assert_eq!(encoding_from_name("CP932"), encoding_rs::SHIFT_JIS);
        assert_eq!(encoding_from_name("JAPANESE"), encoding_rs::SHIFT_JIS);
    }

    #[test]
    fn test_encoding_from_name_utf8() {
        assert_eq!(encoding_from_name("UTF-8"), encoding_rs::UTF_8);
        assert_eq!(encoding_from_name("utf8"), encoding_rs::UTF_8);
    }

    #[test]
    fn test_encoding_from_name_unknown_defaults_to_gbk() {
        assert_eq!(encoding_from_name("unknown"), encoding_rs::GBK);
        assert_eq!(encoding_from_name(""), encoding_rs::GBK);
    }
}
```

**Step 2: Update common/mod.rs to export encoding module**

Modify `src/common/mod.rs`:

```rust
//! Common utilities used across the project

mod encoding;

pub use encoding::{encoding_from_name, DEFAULT_ENCODING};

// ... rest of the file unchanged ...
```

**Step 3: Run tests to verify**

Run: `cargo test --lib common::encoding`
Expected: All tests pass

**Step 4: Commit**

```bash
git add src/common/encoding.rs src/common/mod.rs
git commit -m "feat(common): add shared encoding utility module"
```

---

### Task 2: Update SPF to use shared encoding utility

**Files:**
- Modify: `src/spf/types.rs`
- Modify: `src/spf/mod.rs`

**Step 1: Update spf/types.rs to use common::encoding_from_name**

In `src/spf/types.rs`, remove the local `encoding_from_name` function (lines 113-123) and add import:

```rust
// At top of file, add:
use crate::common::encoding_from_name;

// Remove the local encoding_from_name function (lines 113-123)
```

**Step 2: Update spf/mod.rs if needed**

Check if `encoding_from_name` is re-exported in `src/spf/mod.rs`. If so, change it to re-export from common instead, or remove the re-export if not needed externally.

**Step 3: Run tests to verify**

Run: `cargo test --lib spf`
Expected: All tests pass

**Step 4: Commit**

```bash
git add src/spf/types.rs src/spf/mod.rs
git commit -m "refactor(spf): use shared encoding utility from common module"
```

---

### Task 3: Update LdtReader to support encoding

**Files:**
- Modify: `src/ldt/reader.rs`

**Step 1: Add encoding field to LdtReader struct**

In `src/ldt/reader.rs`, modify the struct:

```rust
pub struct LdtReader {
    mmap: Mmap,
    header: LdtHeader,
    encoding: &'static encoding_rs::Encoding,
}
```

**Step 2: Update open method to accept encoding parameter**

```rust
impl LdtReader {
    /// Open an LDT file and map it to memory
    pub fn open(path: &Path, encoding: &'static encoding_rs::Encoding) -> Result<Self> {
        let file = File::open(path)
            .with_context(|| format!("Failed to open LDT file: {}", path.display()))?;

        // SAFETY: File mapping is safe, we only read
        let mmap = unsafe { Mmap::map(&file) }
            .with_context(|| format!("Failed to mmap LDT file: {}", path.display()))?;

        let len = mmap.len();
        if len < HEADER_SIZE {
            bail!(
                "LDT file too small: {} bytes (minimum: {})",
                len,
                HEADER_SIZE
            );
        }

        // Read header from the beginning
        let header: LdtHeader = bytemuck::pod_read_unaligned(&mmap[0..HEADER_SIZE]);

        // Validate header
        if header.num_fields < 0 || header.num_fields > MAX_FIELDS as i32 {
            bail!("Invalid field count: {}", header.num_fields);
        }
        if header.num_rows < 0 {
            bail!("Invalid row count: {}", header.num_rows);
        }

        Ok(Self { mmap, header, encoding })
    }

    /// Get the encoding used by this reader
    pub fn encoding(&self) -> &'static encoding_rs::Encoding {
        self.encoding
    }

    // ... other methods unchanged until read_field_value ...
}
```

**Step 3: Update read_field_value to use self.encoding**

In `read_field_value` method, around line 203-204, change:

```rust
// Before:
let (s, _, _) = encoding_rs::GBK.decode(data);

// After:
let (s, _, _) = self.encoding.decode(data);
```

**Step 4: Run tests to verify compilation**

Run: `cargo build --lib`
Expected: Compilation succeeds

Note: Tests will fail because LdtReader::open now requires encoding parameter. This is expected.

**Step 5: Commit**

```bash
git add src/ldt/reader.rs
git commit -m "feat(ldt): add encoding support to LdtReader"
```

---

### Task 4: Update LdtWriter to support encoding

**Files:**
- Modify: `src/ldt/writer.rs`

**Step 1: Add encoding field to LdtWriter struct**

In `src/ldt/writer.rs`:

```rust
pub struct LdtWriter {
    db_id: i32,
    field_defs: Vec<FieldDef>,
    rows: Vec<Row>,
    encoding: &'static encoding_rs::Encoding,
}
```

**Step 2: Update new method to accept encoding parameter**

```rust
impl LdtWriter {
    /// Create a new LDT writer with the given database ID and encoding
    pub fn new(db_id: i32, encoding: &'static encoding_rs::Encoding) -> Self {
        Self {
            db_id,
            field_defs: Vec::new(),
            rows: Vec::new(),
            encoding,
        }
    }

    /// Get the encoding used by this writer
    pub fn encoding(&self) -> &'static encoding_rs::Encoding {
        self.encoding
    }

    // ... other methods unchanged ...
}
```

**Step 3: Rename and update write_gbk_string to use encoding**

Rename `write_gbk_string` to `write_encoded_string` and use `self.encoding`:

```rust
/// Write an encoded string with length prefix
fn write_encoded_string<W: Write>(&self, writer: &mut W, s: &str) -> Result<()> {
    let (bytes, _, _) = self.encoding.encode(s);
    let bytes = bytes.as_ref();
    // Write length + content (no terminator)
    writer.write_all(&(bytes.len() as u16).to_le_bytes())?;
    writer.write_all(bytes)?;
    Ok(())
}
```

**Step 4: Update write_field_value to use new method name**

```rust
// In write_field_value, change write_gbk_string to write_encoded_string:

FieldValue::String(s) => {
    self.write_encoded_string(writer, s)?;
}

FieldValue::Alias(s) => {
    self.write_encoded_string(writer, s)?;
}

FieldValue::FID(spf_id, row_id) => {
    let s = format!("{},{}", spf_id, row_id);
    self.write_encoded_string(writer, &s)?;
}
```

**Step 5: Run tests to verify compilation**

Run: `cargo build --lib`
Expected: Compilation succeeds

**Step 6: Commit**

```bash
git add src/ldt/writer.rs
git commit -m "feat(ldt): add encoding support to LdtWriter"
```

---

### Task 5: Update LDT tests for encoding

**Files:**
- Modify: `src/ldt/reader.rs` (tests section)
- Modify: `src/ldt/writer.rs` (tests section)
- Modify: `src/ldt/csv.rs` (tests section)

**Step 1: Update csv.rs tests**

In `src/ldt/csv.rs` tests, no changes needed since CSV import/export doesn't use LdtReader/LdtWriter directly.

Run: `cargo test --lib ldt::csv`
Expected: Tests pass

**Step 2: Update writer.rs tests**

In `src/ldt/writer.rs` tests section, update `test_writer_basic`:

```rust
#[test]
fn test_writer_basic() {
    let mut writer = LdtWriter::new(1, encoding_rs::GBK);
    // ... rest unchanged ...
}
```

**Step 3: Run tests to verify**

Run: `cargo test --lib ldt`
Expected: All tests pass

**Step 4: Commit**

```bash
git add src/ldt/writer.rs
git commit -m "test(ldt): update tests for encoding parameter"
```

---

### Task 6: Update CLI for info command

**Files:**
- Modify: `src/cli/latale-ldt.rs`

**Step 1: Add encoding parameter to Info command**

```rust
#[derive(Subcommand)]
enum Commands {
    /// 显示 LDT 文件信息
    Info {
        /// LDT 文件路径
        ldt_file: PathBuf,
        /// 显示前 N 行数据
        #[arg(short, long, default_value = "5")]
        rows: usize,
        /// 文件名编码 (GBK, BIG5, EUC-KR, SHIFT_JIS, UTF-8)
        #[arg(long, default_value = "GBK")]
        encoding: String,
    },
    // ...
}
```

**Step 2: Update main match for Info**

```rust
Commands::Info { ldt_file, rows, encoding } => {
    cmd_info(&ldt_file, rows, &encoding)?;
}
```

**Step 3: Update cmd_info function**

```rust
fn cmd_info(ldt_file: &std::path::Path, preview_rows: usize, encoding_name: &str) -> Result<()> {
    let encoding = latale_tools::common::encoding_from_name(encoding_name);

    let reader = LdtReader::open(ldt_file, encoding)
        .with_context(|| format!("无法打开 LDT 文件: {}", ldt_file.display()))?;

    // ... existing code ...

    // Add encoding info display after "文件大小":
    println!("文件编码:    {}", encoding_name);

    // ... rest unchanged ...
}
```

**Step 4: Run build to verify**

Run: `cargo build --release`
Expected: Build succeeds

**Step 5: Commit**

```bash
git add src/cli/latale-ldt.rs
git commit -m "feat(cli): add --encoding option to info command"
```

---

### Task 7: Update CLI for convert command

**Files:**
- Modify: `src/cli/latale-ldt.rs`

**Step 1: Add encoding parameter to Convert command**

```rust
#[derive(Subcommand)]
enum Commands {
    // ...
    /// 双向转换：LDT ↔ CSV（支持单文件和目录批量）
    Convert {
        /// 输入文件或目录（默认 DATA/LDT）
        input: Option<PathBuf>,
        /// 输出路径
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// 文件名编码 (GBK, BIG5, EUC-KR, SHIFT_JIS, UTF-8)
        #[arg(long, default_value = "GBK")]
        encoding: String,
    },
}
```

**Step 2: Update main match for Convert**

```rust
Commands::Convert { input, output, encoding } => {
    let input = input.as_deref();
    cmd_convert(input, output.as_deref(), &encoding)?;
}
```

**Step 3: Update cmd_convert function signature**

```rust
fn cmd_convert(input: Option<&std::path::Path>, output: Option<&std::path::Path>, encoding_name: &str) -> Result<()> {
    let encoding = latale_tools::common::encoding_from_name(encoding_name);

    let input = input.unwrap_or_else(|| std::path::Path::new(DEFAULT_LDT_DIR));

    if !input.exists() {
        bail!("输入路径不存在: {}", input.display());
    }

    if input.is_file() {
        convert_single_file(input, output, false, encoding)
    } else if input.is_dir() {
        convert_directory(input, output, encoding, encoding_name)
    } else {
        bail!("输入路径不是文件或目录: {}", input.display())
    }
}
```

**Step 4: Update convert_single_file function**

```rust
fn convert_single_file(input: &Path, output: Option<&Path>, silent: bool, encoding: &'static encoding_rs::Encoding) -> Result<()> {
    // ... existing code until the match ...

    match input_ext.as_str() {
        ext if ext == LDT_EXTENSION => {
            // ... output path logic unchanged ...
            convert_ldt_to_csv(input, &output_path, silent, encoding)?;
        }

        ext if ext == CSV_EXTENSION => {
            // ... output path logic unchanged ...
            convert_csv_to_ldt(input, &output_path, silent, encoding)?;
        }
        // ...
    }
    // ...
}
```

**Step 5: Update convert_directory function**

```rust
fn convert_directory(input: &Path, output: Option<&Path>, encoding: &'static encoding_rs::Encoding, encoding_name: &str) -> Result<()> {
    // ... existing code ...

    // Add encoding display after "文件数量":
    println!("文件编码:    {}", encoding_name);

    // ... existing loop, but pass encoding to convert_single_file ...
    match convert_single_file(file, Some(output_dir.as_path()), true, encoding) {
        // ...
    }

    // ...
}
```

**Step 6: Update convert_ldt_to_csv function**

```rust
fn convert_ldt_to_csv(input: &Path, output_path: &Path, silent: bool, encoding: &'static encoding_rs::Encoding) -> Result<()> {
    let start = Instant::now();

    // 读取 LDT
    let reader = LdtReader::open(input, encoding)
        .with_context(|| format!("无法打开 LDT 文件: {}", input.display()))?;

    // ... rest unchanged ...

    if !silent {
        // ... add encoding display in output if desired ...
    }

    Ok(())
}
```

**Step 7: Update convert_csv_to_ldt function**

```rust
fn convert_csv_to_ldt(input: &Path, output_path: &Path, silent: bool, encoding: &'static encoding_rs::Encoding) -> Result<()> {
    let start = Instant::now();

    // 读取 CSV
    let mut file = std::fs::File::open(input)
        .with_context(|| format!("无法打开 CSV 文件: {}", input.display()))?;
    let (db_id, field_defs, rows) = import_from_csv(&mut file).context("读取 CSV 文件失败")?;

    // 确保输出目录存在
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("无法创建输出目录: {}", parent.display()))?;
    }

    // 写入 LDT
    let mut writer = LdtWriter::new(db_id, encoding);
    writer.set_field_defs(&field_defs);
    writer.set_rows(&rows);
    writer.write(output_path).context("写入 LDT 文件失败")?;

    // ... rest unchanged ...
}
```

**Step 8: Run build to verify**

Run: `cargo build --release`
Expected: Build succeeds

**Step 9: Commit**

```bash
git add src/cli/latale-ldt.rs
git commit -m "feat(cli): add --encoding option to convert command"
```

---

### Task 8: Integration testing

**Files:**
- Test with actual LDT files

**Step 1: Build release**

Run: `cargo build --release`
Expected: Build succeeds

**Step 2: Test info command with default encoding**

Run: `./target/release/latale-ldt info DATA/LDT/Item.LDT`
Expected: Shows file info with GBK encoding

**Step 3: Test info command with explicit encoding**

Run: `./target/release/latale-ldt info DATA/LDT/Item.LDT --encoding BIG5`
Expected: Shows file info with BIG5 encoding display

**Step 4: Test convert LDT to CSV**

Run: `./target/release/latale-ldt convert DATA/LDT/Item.LDT -o /tmp/test.csv`
Expected: CSV file created successfully

**Step 5: Test convert CSV to LDT**

Run: `./target/release/latale-ldt convert /tmp/test.csv -o /tmp/test.LDT`
Expected: LDT file created successfully

**Step 6: Test roundtrip with different encoding**

Run:
```bash
./target/release/latale-ldt convert DATA/LDT/Item.LDT --encoding BIG5 -o /tmp/big5.csv
./target/release/latale-ldt convert /tmp/big5.csv --encoding BIG5 -o /tmp/big5.LDT
```
Expected: Both conversions succeed

**Step 7: Test batch conversion**

Run: `./target/release/latale-ldt convert DATA/LDT --encoding GBK -o /tmp/ldt_csv`
Expected: Batch conversion succeeds

**Step 8: Commit**

```bash
git add -A
git commit -m "test: verify encoding support integration"
```

---

### Task 9: Final verification and documentation

**Files:**
- Verify: All files
- Update: `CLAUDE.md` if needed

**Step 1: Run all tests**

Run: `cargo test`
Expected: All tests pass

**Step 2: Run clippy**

Run: `cargo clippy -- -D warnings`
Expected: No warnings

**Step 3: Test help output**

Run: `./target/release/latale-ldt --help`
Run: `./target/release/latale-ldt info --help`
Run: `./target/release/latale-ldt convert --help`
Expected: Help shows --encoding option

**Step 4: Final commit**

```bash
git add -A
git commit -m "chore: final verification for encoding support"
```
