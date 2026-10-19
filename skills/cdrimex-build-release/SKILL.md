---
name: cdrimex-build-release
description: cdrimex 构建与发布规范：cdylib 配置、依赖锁定（dyteslogs 被 yanked）、wincom 直接依赖、edition 与 unsafe no_mangle、clippy 门禁，以及 dytes.json/mcp.json 发布清单检查。构建、修复依赖或发布前使用。
---

# cdrimex 构建与发布规范

## Cargo 配置

```toml
[package]
name = "cdrimex"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]
name = "cdrimex"

[dependencies]
cdrsdk = "0.2.0"
libc = "0.2"
once_cell = "1.19"
serde = { version = "1.0", features = ["derive"] }
serde_derive = "1.0"
serde_json = "1.0"
wincom = "0.2.0"          # 底层 Export 调用需要 Variant
windows = { version = "0.52.0", features = [ ... ] }
```

- 同时产出 `cdylib`（插件 DLL）和 `rlib`（集成测试可用）。
- `edition = "2021"` 下可使用 `#[unsafe(no_mangle)]`（Rust 1.82+）。
- `wincom` 是直接依赖，不是仅传递依赖。

## 依赖锁定：dyteslogs 已被 yanked

`cdrsdk 0.2.0` 依赖 `dyteslogs ^0.1.0`，而 `dyteslogs 0.1.0` 在 crates.io 已被 yanked：

- 保留 `cdrimex/Cargo.lock` 中锁定的 `dyteslogs 0.1.0`，Cargo 允许已锁定版本继续使用。
- **不要删除 `Cargo.lock`**，也不要执行会触发重新解析的 `cargo update`。
- 当前代码已移除对 `dyteslogs` 的直接依赖，日志用 `eprintln!`。
- 长期方案：等待 `cdrsdk` 上游更新依赖，或改用可用的私有源。

## 构建与检查命令

```bash
cd cdrimex

# 编译（含测试目标）
cargo check --all-targets

# 代码质量门禁
cargo clippy --all-targets

# 全量测试
cargo test

# 发布构建
cargo build --release
```

发布产物：`cdrimex/target/release/cdrimex.dll`。

## clippy 约定

- `src/ffi/mod.rs` 统一允许 `not_unsafe_ptr_arg_deref`（FFI 边界）。
- `cjson` 的裸指针函数标记为 `unsafe fn`，调用处写 `unsafe` 块。
- 不新增 `#[allow]` 除非有 FFI/平台边界理由，并在注释中说明。

## 发布清单检查

`src/dytes.json` 声明的 `exports` 必须与磁盘文件一致：

```json
{
  "exports": {
    "./target/release/cdrimex.dll": "cdrimex.dll",
    "./mcp.json": "mcp.json",
    "./mcp/export_file.json": "export_file.json",
    "...": "..."
  }
}
```

当前仓库**缺少** `src/mcp/*.json`（7 个工具定义），发布前必须补齐：

- `export_file.json`
- `export_cdr_file.json`
- `export_cover.json`
- `export_selection_cover.json`
- `export_select_comps.json`
- `export_preview.json`
- `export_preview_sum.json`

`src/mcp.json` 的 `tools` 列表也必须与这些文件一一对应。

## 发布前检查项

- [ ] `cargo check --all-targets` 通过
- [ ] `cargo clippy --all-targets` 通过
- [ ] `cargo test` 通过（真实环境 E2E 需 CorelDRAW 26）
- [ ] `cargo build --release` 生成 `cdrimex.dll`
- [ ] `dytes.json` / `mcp.json` / `mcp/*.json` 完整且相互一致
- [ ] `Cargo.lock` 已提交，包含 `dyteslogs 0.1.0`
