# latale-ldt 使用说明

`latale-ldt` 用于查看 LDT 数据表，以及在 LDT 与 CSV 之间双向转换。它既支持单个文件，也支持目录批量转换。

## 准备工作

Windows 发布包中的文件名包含版本和平台，例如：

```text
latale-ldt-0.0.2-windows-x64.exe
```

可以直接使用完整文件名，也可以将它改名为 `latale-ldt.exe`。以下示例使用改名后的文件名。

```powershell
.\latale-ldt.exe --help
.\latale-ldt.exe --version
```

路径中包含空格时，必须使用双引号。

## 编码选择

编码用于解释 LDT 中的字符串：

| 参数值 | 常见用途 |
|---|---|
| `GBK` | 私服资源 |
| `BIG5` | 台服资源 |
| `EUC-KR` | 韩服资源 |
| `SHIFT_JIS` | 日文资源 |
| `UTF-8` | UTF-8 资源 |

默认编码是 `GBK`。编码选择错误时，文字可能乱码或转换失败。

## 查看 LDT 信息

显示文件信息、字段定义和前 5 行：

```powershell
.\latale-ldt.exe info .\DATA\LDT\ITEM.LDT
```

指定编码和预览行数：

```powershell
.\latale-ldt.exe info .\DATA\LDT\ITEM.LDT --encoding BIG5 --rows 20
```

只查看基本信息与字段定义，不预览数据：

```powershell
.\latale-ldt.exe info .\DATA\LDT\ITEM.LDT --rows 0
```

`info` 的行预览会显示主键和前 5 个字段；完整数据请转换为 CSV 后查看。

## 单文件转换

### LDT 转 CSV

```powershell
.\latale-ldt.exe convert .\DATA\LDT\ITEM.LDT --encoding GBK
```

CLI 未指定输出路径时，会写入：

```text
DATA\CSV\ITEM.csv
```

指定输出文件：

```powershell
.\latale-ldt.exe convert .\DATA\LDT\ITEM.LDT `
  --encoding BIG5 `
  --output .\export\ITEM.csv
```

指定输出目录：

```powershell
.\latale-ldt.exe convert .\DATA\LDT\ITEM.LDT --output .\export
```

### CSV 转 LDT

转换方向由输入文件扩展名自动判断：

```powershell
.\latale-ldt.exe convert .\DATA\CSV\ITEM.csv --encoding GBK
```

未指定输出路径时，会写入：

```text
DATA\LDT\ITEM.LDT
```

指定输出文件：

```powershell
.\latale-ldt.exe convert .\ITEM.csv --output .\ITEM.LDT --encoding GBK
```

## 目录批量转换

批量将目录中的 LDT 转为 CSV：

```powershell
.\latale-ldt.exe convert .\DATA\LDT --output .\DATA\CSV --encoding GBK
```

批量将目录中的 CSV 转为 LDT：

```powershell
.\latale-ldt.exe convert .\DATA\CSV --output .\DATA\LDT --encoding GBK
```

未提供输入参数时，默认读取 `DATA\LDT`：

```powershell
.\latale-ldt.exe convert --encoding GBK
```

批量转换的规则：

- 只处理输入目录中的直接文件，不递归处理子目录。
- 同一个输入目录不能同时包含 LDT 和 CSV；请分目录处理。
- 每个文件独立转换，结束时显示成功与失败数量。
- 输入目录和输出目录不能相同。

## CSV 格式

CSV 第一行同时保存字段名和字段类型：

```csv
ID:int32,Name:string,Enabled:bool,Price:int32,Ratio:float32
1,Short Sword,true,100,1.25
```

支持的类型：

| 类型 | 说明 |
|---|---|
| `na` | 空或未使用字段 |
| `string` | 字符串 |
| `bool` | 布尔值 |
| `int32` | 32 位整数 |
| `float32` | 浮点数 |
| `fid` | SPF ID 与行 ID，例如 `"3,42"` |
| `alias` | 别名字符串 |
| `int64` | 64 位整数 |

第一列必须是主键列。CSV 转 LDT 时，数据库 ID 当前固定写为 `0`。

建议使用支持 UTF-8 CSV 的编辑器。字段中含逗号、换行或双引号时，应让 CSV 编辑器保留标准引号转义。

## 常见问题

### 转换后出现乱码

重新确认原始 LDT 所属服务器，并通过 `--encoding` 指定正确编码。台服通常使用 `BIG5`，韩服使用 `EUC-KR`，私服通常使用 `GBK`。

### 目录中同时存在 LDT 和 CSV

工具无法自动决定批量转换方向。请把两种文件放到不同目录后再次运行。

### 想把结果放在输入文件旁边

CLI 请明确指定输出路径：

```powershell
.\latale-ldt.exe convert .\work\ITEM.LDT --output .\work\ITEM.csv
```

桌面 GUI 的单文件转换默认会直接保存到输入文件所在目录。
