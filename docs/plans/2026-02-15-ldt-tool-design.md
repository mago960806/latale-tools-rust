# LDT 数据库转换工具设计

## 概述

LDT 工具用于 LaTale 游戏数据库文件的读取、写入和格式转换。支持 LDT 二进制格式与 CSV 文本格式之间的双向转换，便于使用文本比较工具进行数据合并。

## CLI 命令

### `latale-ldt convert [INPUT] [-o OUTPUT]`

智能转换命令，根据输入类型自动判断转换方向。

**参数**：
- `INPUT` - 输入文件或目录（可选，默认 `DATA/LDT`）
- `-o, --output <PATH>` - 输出路径（可选，根据输入类型有不同默认值）

**行为**：

| 输入类型 | 默认输出 | 说明 |
|---------|---------|------|
| 单个 `.LDT` 文件 | `DATA/CSV/{name}.csv` | LDT → CSV |
| 单个 `.csv` 文件 | `DATA/LDT/{name}.LDT` | CSV → LDT |
| 目录（含 `.LDT` 文件） | `DATA/CSV/` | 批量 LDT → CSV |
| 目录（含 `.csv` 文件） | `DATA/LDT/` | 批量 CSV → LDT |
| 无参数 | `DATA/LDT` → `DATA/CSV` | 使用默认值 |

**安全机制**：
- 目录中同时存在 `.LDT` 和 `.csv` 时报错
- 输出路径与输入路径相同时报错

### `latale-ldt info <LDT_FILE>`

显示 LDT 数据库结构信息。

**输出内容**：
- 数据库 ID
- 字段数量
- 数据行数
- 字段列表（名称 + 类型）

## CSV 格式规范

### 文件结构

```csv
# database: ITEM.LDT
# rows: 150
ItemID:int64,Name:string,Price:int32,Rate:float32,Enabled:bool,IconRef:fid
10001,"青铜剑",1500,1.0,true,5001
10002,"铁剑",3000,1.2,true,5002
```

### 头部注释

- `# database: <name>` - 原始数据库名称
- `# rows: <count>` - 数据行数

### 列头格式

```
<字段名>:<类型>
```

### 类型映射

| LDT 类型 | CSV 类型 | 说明 |
|----------|---------|------|
| `fldNum` | `int32` | 32位有符号整数 |
| `fldNum64` | `int64` | 64位有符号整数 |
| `fldPer` | `float32` | 单精度浮点数 |
| `fldTF` | `bool` | 布尔值 |
| `fldString` | `string` | 字符串（最大 8192 字节） |
| `fldAlias` | `alias` | 别名（最大 4096 字节） |
| `fldFID` | `fid` | 外键引用 |

## LDT 二进制格式

### 文件头（8716 字节）

| 偏移量 | 大小 | 字段 | 说明 |
|--------|------|------|------|
| 0x00 | 4 | `nDB_ID` | 数据库 ID |
| 0x04 | 4 | `nFields` | 字段数量（最多 128） |
| 0x08 | 4 | `nData` | 数据行数 |
| 0x0C | 8192 | `FieldNAM[128]` | 字段名称数组（每个 64 字节） |
| 0x200C | 512 | `FieldTYP[128]` | 字段类型数组（每个 4 字节） |

### 文件尾（64 字节）

每个 LDT 文件末尾有一个固定的 64 字节 footer：
- `END` (3 字节)
- 空格填充 (61 字节，0x20)

### 数据行结构

每行数据：
1. 主键（4 字节，int32）
2. 各字段数据（按字段定义顺序）

### 字段数据存储

| 类型 | 存储格式 |
|------|---------|
| `int32` | 4 字节 little-endian |
| `int64` | 8 字节 little-endian |
| `float32` | 4 字节 IEEE 754 little-endian |
| `bool` | 4 字节（0=false，非0=true） |
| `string`/`alias`/`fid` | 2 字节长度（u16）+ 变长数据（GBK 编码，无结束符） |

## 代码架构

```
src/
├── main.rs              # CLI 入口（clap）
├── lib.rs               # 库根
└── ldt/
    ├── mod.rs           # 模块导出
    ├── types.rs         # LdtHeader, FieldType, FieldValue, Row
    ├── reader.rs        # LdtReader（零拷贝读取）
    ├── writer.rs        # LdtWriter（写入 LDT）
    └── csv.rs           # CSV 导入/导出
```

### 模块职责

- **types.rs**: 定义 LDT 二进制结构、字段类型枚举、字段值枚举
- **reader.rs**: 使用 mmap 零拷贝读取 LDT 文件，提供迭代器访问数据行
- **writer.rs**: 将数据行写入 LDT 文件
- **csv.rs**: CSV 格式的读写，类型转换

## 错误处理

使用 `anyhow` 进行错误处理，主要错误场景：
- 文件不存在或无法读取
- 文件格式不正确（文件头损坏）
- 字段类型不匹配
- CSV 解析错误
- 数据验证失败（如字符串超长）
