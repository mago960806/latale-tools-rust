# latale-spf 使用说明

`latale-spf` 用于查看、验证、解包和打包 LaTale 的 SPF 资源包。它能够自动识别新版加密 SPF；查看、验证和解包时不需要额外提供密钥或参数。

## 准备工作

Windows 发布包中的文件名包含版本和平台，例如：

```text
latale-spf-0.0.2-windows-x64.exe
```

可以直接使用完整文件名，也可以将它改名为 `latale-spf.exe`。以下示例使用改名后的文件名。

在资源文件所在目录打开 PowerShell：

```powershell
.\latale-spf.exe --help
.\latale-spf.exe --version
```

路径中包含空格时，必须使用双引号：

```powershell
.\latale-spf.exe info "D:\LaTale Client\ROWID.SPF"
```

## 查看 SPF 信息

```powershell
.\latale-spf.exe info ROWID.SPF
```

输出包括版本号、加密状态、文件编号、注册名称、文件名编码、文件数量和总大小。

增加 `-l` 或 `--list` 可显示包内全部文件：

```powershell
.\latale-spf.exe info ROWID.SPF --list
```

## 验证完整性

```powershell
.\latale-spf.exe verify ROWID.SPF
```

加密包会先自动解密索引再验证。验证通过时显示“文件完整无损”；发现偏移、大小或索引问题时，程序会列出问题并返回非零退出码。

## 解包 SPF

解包到当前 PowerShell 所在目录：

```powershell
.\latale-spf.exe unpack ROWID.SPF
```

指定输出目录：

```powershell
.\latale-spf.exe unpack ROWID.SPF --output D:\latale-unpacked
```

只查看将要写出的文件，不实际解包：

```powershell
.\latale-spf.exe unpack ROWID.SPF --dry-run
```

如果 SPF 已加密，程序会自动解密资源数据。输出目录会按照包内路径创建，例如 `DATA\LDT\ITEM.LDT`。

## 打包 SPF

最常见的目录结构如下：

```text
D:\latale-work\
└─ DATA\
   └─ LDT\
      ├─ ITEM.LDT
      └─ MONSTER.LDT
```

打包未加密的 `ROWID.SPF`：

```powershell
.\latale-spf.exe pack ROWID --data-dir D:\latale-work --version 2026072301
```

默认输出到当前目录的 `ROWID.SPF`。指定输出文件：

```powershell
.\latale-spf.exe pack ROWID `
  --data-dir D:\latale-work `
  --version 2026072301 `
  --output D:\release\ROWID.SPF
```

打包加密 SPF：

```powershell
.\latale-spf.exe pack ROWID `
  --data-dir D:\latale-work `
  --version 2026072301 `
  --encrypt
```

仅检查输入目录和待打包文件：

```powershell
.\latale-spf.exe pack ROWID --data-dir D:\latale-work --dry-run
```

常用参数：

| 参数 | 说明 |
|---|---|
| `--data-dir <目录>` | DATA 资源根目录的上一级目录，默认是当前目录 |
| `-o, --output <文件>` | 输出 SPF 文件 |
| `--version <数字>` | 写入 SPF 的版本号 |
| `--encoding <编码>` | 文件名编码；不填写时使用资源类型注册值 |
| `--encrypt` | 使用新版 ChaCha20 加密；默认不加密 |
| `--dry-run` | 列出文件但不生成 SPF |

## 支持的资源类型

`pack` 的第一个参数不是任意文件名，而是注册的资源类型：

| 名称 | 文件编号 | 主要输入目录 |
|---|---:|---|
| `AJJIYA` | 1 | `DATA/ANITABLE` |
| `HOSHIM` | 2 | `DATA/FX` |
| `ROWID` | 3 | `DATA/LDT` |
| `JINSSAGA` | 4 | `DATA/BGFORMAT` |
| `MAKO1298` | 5 | `DATA/BACKGROUND` |
| `METALGENI` | 6 | `DATA/SOUND` |
| `DALBONG` | 7 | `DATA/BGM` |
| `RYUMS` | 8 | `DATA/TERRAIN` |
| `BANX` | 9 | `DATA/GLOBALRES` 及已注册子目录 |
| `BARY` | 10 | `DATA/CURSOR`、`DATA/INTERFACE`、`DATA/LOADING` 等 |
| `ZENNE` | 11 | `DATA/CHAR` 下的已注册角色与怪物目录 |
| `CLAIRE` | 12 | `DATA/PROLOGUE`、`DATA/LOGO` |
| `CVOICE` | 13 | `DATA/CP_IMAGE`、`DATA/STORY/STORY1` 至 `STORY8` |

打包器只读取每个注册目录中的直接文件，不递归扫描未注册子目录，并按注册目录顺序写入 SPF。

## 常见问题

### 提示“未知的 SPF 名称”

请使用上表中的名称，例如 `ROWID`，不要使用自定义名称。

### 提示“读取源文件失败”

检查 `--data-dir` 是否指向 `DATA` 目录的上一级。例如文件位于 `D:\work\DATA\LDT` 时，应使用 `--data-dir D:\work`。

### 如何确认打包结果

建议依次运行：

```powershell
.\latale-spf.exe info .\ROWID.SPF --list
.\latale-spf.exe verify .\ROWID.SPF
```
