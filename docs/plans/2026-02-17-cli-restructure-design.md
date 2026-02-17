# CLI 目录结构重构设计

## 背景

项目目前有两个独立的 CLI 工具：
- `latale-spf` - SPF 资源打包/解包
- `latale-ldt` - LDT 数据库转换

当前目录结构不对称：
- `latale-spf` 入口在 `src/main.rs`
- `latale-ldt` 入口在 `src/bin/latale-ldt.rs`

未来可能添加 GUI，需要更清晰的组织结构。

## 目标

1. 将 `src/bin/` 重命名为 `src/cli/`，为未来 GUI 模块预留命名空间
2. 统一 CLI 入口文件位置
3. 保持独立可执行文件

## 设计

### 重构后目录结构

```
src/
├── lib.rs              # 库根（保持不变）
├── cli/                # CLI 入口目录（原 bin/）
│   ├── latale-spf.rs   # SPF CLI 入口
│   └── ldt-ldt.rs      # LDT CLI 入口
├── common/             # 通用工具（保持不变）
├── spf/                # SPF 模块（保持不变）
└── ldt/                # LDT 模块（保持不变）
```

### Cargo.toml 变更

```toml
[[bin]]
name = "latale-spf"
path = "src/cli/latale-spf.rs"

[[bin]]
name = "latale-ldt"
path = "src/cli/latale-ldt.rs"
```

## 实施步骤

1. 创建 `src/cli/` 目录
2. 移动 `src/main.rs` → `src/cli/latale-spf.rs`
3. 移动 `src/bin/latale-ldt.rs` → `src/cli/latale-ldt.rs`
4. 删除空的 `src/bin/` 目录
5. 更新 `Cargo.toml` 中的 bin path
6. 验证构建通过

## 影响范围

- Cargo.toml 配置
- 两个 CLI 入口文件位置
- 不影响库代码（spf/, ldt/, common/）
