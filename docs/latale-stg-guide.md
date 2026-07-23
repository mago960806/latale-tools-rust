# latale-stg 使用说明

`latale-stg` 用于查看 LaTale 的 STG 场景数据，并在 STG 与可编辑的 JSON 之间双向转换。

## 准备工作

Windows 发布包中的文件名包含版本和平台，例如：

```text
latale-stg-0.0.2-windows-x64.exe
```

可以直接使用完整文件名，也可以将它改名为 `latale-stg.exe`。以下示例使用改名后的文件名。

```powershell
.\latale-stg.exe --help
.\latale-stg.exe --version
```

路径中包含空格时，必须使用双引号。

## 编码选择

编码用于解释 STG 中的 Stage、Group 和 Map 字符串：

| 参数值 | 常见用途 |
|---|---|
| `GBK` | 私服资源 |
| `BIG5` | 台服资源 |
| `EUC-KR` | 韩服资源 |
| `SHIFT_JIS` | 日文资源 |
| `UTF-8` | UTF-8 资源 |

默认编码是 `GBK`。同一个编码必须用于 STG → JSON 和 JSON → STG。

## 查看 STG 结构

显示全部 Stage、Group 和 Map：

```powershell
.\latale-stg.exe info .\cn\STAGENEW.STG --encoding GBK
```

限制输出数量，便于快速检查：

```powershell
.\latale-stg.exe info .\tw\STAGENEW.STG `
  --encoding BIG5 `
  --stages 3 `
  --groups 2 `
  --maps 2
```

参数含义：

| 参数 | 说明 |
|---|---|
| `--stages <数量>` | 只显示前 N 个 Stage |
| `--groups <数量>` | 每个 Stage 只显示前 N 个 Group |
| `--maps <数量>` | 每个 Group 只显示前 N 个 Map |
| `--encoding <编码>` | STG 字符串编码 |

未提供 STG 文件时，默认读取 `cn\STAGENEW.STG`。

## STG 转 JSON

在输入文件旁生成 `STAGENEW.JSON`：

```powershell
.\latale-stg.exe convert .\cn\STAGENEW.STG --encoding GBK
```

指定输出文件：

```powershell
.\latale-stg.exe convert .\tw\STAGENEW.STG `
  --encoding BIG5 `
  --output .\export\STAGENEW.JSON
```

指定输出目录：

```powershell
.\latale-stg.exe convert .\cn\STAGENEW.STG --output .\export
```

## JSON 转 STG

转换方向由输入文件扩展名自动判断：

```powershell
.\latale-stg.exe convert .\export\STAGENEW.JSON `
  --encoding GBK `
  --output .\rebuilt\STAGENEW.STG
```

如果不指定输出路径，会在 JSON 所在目录生成同名 `.STG` 文件。

## 编辑 JSON 时的注意事项

JSON 保留 STG 的层级结构：

```text
StageList
└─ GroupList
   └─ MapList
```

修改 JSON 时需要注意：

- `StageCount` 必须等于 `StageList` 的实际数量。
- 每个 Stage 的 `GroupCount` 必须等于其 `GroupList` 数量。
- 每个 Group 的 `MapCount` 必须等于其 `MapList` 数量。
- STG 字符串字段在二进制中最多占 64 字节；不要只按字符数量判断。
- JSON 必须保持合法格式，字符串中的反斜杠和双引号需要正确转义。
- 建议保留原始 STG 备份，不要直接覆盖唯一副本。

## 推荐的修改流程

```powershell
# 1. 转为 JSON
.\latale-stg.exe convert .\cn\STAGENEW.STG `
  --encoding GBK `
  --output .\work\STAGENEW.JSON

# 2. 编辑并检查 JSON

# 3. 写回新的 STG
.\latale-stg.exe convert .\work\STAGENEW.JSON `
  --encoding GBK `
  --output .\work\STAGENEW.STG

# 4. 再次读取，确认结构
.\latale-stg.exe info .\work\STAGENEW.STG `
  --encoding GBK `
  --stages 3 `
  --groups 2 `
  --maps 2
```

## 常见问题

### 文字乱码或提示解码失败

检查 `--encoding` 是否与资源所属服务器一致。台服通常使用 `BIG5`，韩服使用 `EUC-KR`，私服通常使用 `GBK`。

### JSON 写回 STG 失败

首先检查 JSON 语法，然后检查 `StageCount`、`GroupCount` 和 `MapCount` 是否与实际列表长度一致。

### 输出文件在哪里

- STG → JSON：默认放在输入 STG 旁边。
- JSON → STG：默认放在输入 JSON 旁边。
- `--output` 没有扩展名时按目录处理；有扩展名时按完整文件路径处理。
