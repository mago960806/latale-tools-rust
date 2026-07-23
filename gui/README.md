# LaTale Tools Desktop

基于 Tauri 2、Svelte 5 和现有 `latale-tools` Rust 库的固定尺寸桌面界面。

## 功能

- SPF：查看包信息和文件列表、识别加密状态、验证、解包；ROWID 包可解包并生成 `latale.db`。
- LDT：查看全部表格数据，执行 LDT 和 CSV 双向转换。
- STG：查看 Stage/Group/Map 数量，执行 STG 和 JSON 双向转换。
- SPF 打包：从注册表选择资源类型，支持明文或加密打包。
- DATA 批量打包：已预留资源类型多选界面，执行队列将在后续接入。

## 本地开发

需要 Node.js、npm、Rust 以及当前平台对应的 Tauri 系统依赖。

```bash
cd gui
npm install
npm run tauri dev
```

仅检查或构建前端：

```bash
npm run check
npm run build
```

检查 Rust 后端：

```bash
cd src-tauri
cargo check
```

## Windows 安装包

Windows 平台配置会生成：

- 独立运行的 Windows `EXE`；
- NSIS `Setup.exe`，面向普通用户；
- WiX `MSI`，面向批量和企业部署。

在 Windows 构建机中运行：

```powershell
cd gui
npm ci
npm run tauri build
```

安装程序会关联 `.spf`、`.ldt`、`.stg`。`installer-hooks.nsh` 另外注册：

- SPF：验证、解包；
- LDT：转换；
- STG：转换。

这些右键功能使用当前用户注册表，不需要管理员权限。Windows 11 默认可能将经典菜单项放在“显示更多选项”中。

### 使用 GitHub Actions 构建

推送代码后，在 GitHub 仓库中打开 `Actions`，选择 `Build Windows Release`，点击
`Run workflow`。构建完成后，在任务页面的 `Artifacts` 区域下载
`latale-tools-windows-x64`。

推送 `v*` 格式的标签也会自动构建。例如：

```bash
git tag v0.0.3
git push origin v0.0.3
```

下载的构建产物包括桌面版独立 EXE、NSIS 安装程序、MSI 安装包，以及
`latale-spf`、`latale-ldt`、`latale-stg` 三个 CLI 工具和对应使用说明。
文件名统一使用小写字母和连字符。安装后的桌面程序文件名为
`latale-tools.exe`；应用和安装界面仍显示为 “LaTale Tools”。

## 目录

```text
gui/
├── src/                         # Svelte 前端
├── src-tauri/
│   ├── src/lib.rs               # Rust/Tauri 命令
│   ├── windows/                 # NSIS 安装和卸载钩子
│   ├── tauri.conf.json
│   └── tauri.windows.conf.json
└── package.json
```
