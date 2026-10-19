# cdrimex 开发记录

> 日期：2026-10-19
> 分支：`cdrimex`（已推送到 `origin/cdrimex`，提交 `13d75b2`）
> 目标：CorelDRAW 插件 `cdrimex`，以 Rust `cdylib` 形式导出 JSON 接口，并通过 MCP 工具清单对外提供。

## 1. 项目初始化

- [x] 在仓库根创建 `skills/` 目录。
- [x] 在 `cdrimex/` 下执行 `cargo init --lib`，包名 `cdrimex`。
- [x] 配置为 `cdylib + rlib`，便于同时产出插件 DLL 和运行集成测试。

`cdrimex/Cargo.toml` 关键配置：

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
wincom = "0.2.0"
windows = { version = "0.52.0", features = [ ... ] }
```

## 2. 依赖问题处理

- [x] 确认 `cdrsdk 0.2.0` 可用。
- [x] 发现 `cdrsdk 0.2.0` 的传递依赖 `dyteslogs ^0.1.0` 已被 crates.io **yanked**，新解析会直接失败。
- [x] 在 `cdrimex/Cargo.lock` 中锁定 `dyteslogs 0.1.0`，Cargo 允许已锁定的 yanked 版本继续使用。
- [x] 移除对 `dyteslogs` 的直接依赖，日志改用 `eprintln!`。
- [x] 新增 `wincom = "0.2.0"` 直接依赖，用于构造底层 `Export` 调用所需的 `Variant`。

> 注意：不要删除 `Cargo.lock`，也不要执行会触发重新解析的 `cargo update`，否则 yanked 依赖问题会复现。

## 3. 模块结构

```
cdrimex/src/
├── lib.rs              # 声明模块，re-export 8 个 FFI 函数
├── dytes.json          # 插件发布清单
├── mcp.json            # MCP 工具列表（8 项）
├── mcp/                # 8 个 MCP 工具定义
├── ffi/
│   ├── mod.rs
│   ├── file.rs         # export_file / export_cdr_file / current_file_path
│   ├── comps.rs        # export_select_comps
│   ├── cover.rs        # export_cover / export_selection_cover
│   └── preview.rs      # export_preview / export_preview_sum
└── utils/
    ├── mod.rs
    ├── cjson.rs        # JSON 输入解析、统一响应、64 KiB 输出缓冲
    ├── com.rs          # COM 生命周期、应用/文档初始化、底层 Export
    ├── filetypes.rs    # 文件类型映射
    ├── export_file.rs
    ├── export_comps.rs
    ├── export_cover.rs
    └── export_preview.rs
```

## 4. cdrsdk 0.2.0 迁移

原代码按旧版 `cdrsdk` API 编写（`cdrsdk::sdk::...`），与 0.2.0 不兼容。已完成迁移：

- [x] `cdrsdk::sdk::application::IvgApplication` → `cdrsdk::app::IvgApplication`
- [x] `cdrsdk::sdk::document::IvgDocument` → `cdrsdk::document::IvgDocument`
- [x] `get_active_document()` → `active_document()`
- [x] `get_fullfilename()` → `full_file_name()`
- [x] `put_fullfilename()` → `save_as()`
- [x] `create_struct_export_options()` → `create_export_options()`
- [x] `create_struct_palette_options()` → `create_palette_options()`
- [x] `get_activepage()` / `get_pages()` / `get_count()` → `active_page()` / `pages()` / `count()`
- [x] 用 `utils/com.rs::export_doc` 保留 5 参数 `Export` 语义（路径、过滤器、范围、导出选项、调色板选项）
- [x] 文件类型映射：`pdf=1333`、`jpg/jpeg=820`、`png=802`

### CorelDRAW 26 兼容性修复

- [x] `IvgPages::item(i)` 在 CorelDRAW 26 上返回 `None`（`Item` 是带参数的属性，不能按方法调用）。
      已改为 `pages.first()` + `page.next()` 链式遍历。
- [x] `Document.Shapes` 在 CorelDRAW 26 上返回 `None`。
      已改为从 `doc.active_page().shapes()` 获取图形；测试创建选区也使用该路径。
- [x] 移除重复的 `ComGuard` 实现，统一到 `utils/com.rs`，保证 `CoInitialize` / `CoUninitialize` 成对。
- [x] `cjson` 中接收裸指针的 `cstr_to_str` / `parse_json_input` 标记为 `unsafe fn`，调用处补 `unsafe` 块。
- [x] `src/ffi/mod.rs` 统一允许 `clippy::not_unsafe_ptr_arg_deref`（FFI 边界约定）。

## 5. FFI 接口

共 8 个导出函数，统一签名：

```rust
#[unsafe(no_mangle)]
pub extern "C" fn post_xxx(json_in: *const c_char, json_out: *mut c_char) -> c_int
```

| 导出函数 | 输入字段 | 成功 data |
| --- | --- | --- |
| `post_export_file` | `file_src`, `ver` | `{}` |
| `post_export_cdr_file` | `file_src`, `ver` | `{}` |
| `post_current_file_path` | `ver` | `{ file_path }` |
| `post_export_select_comps` | `file_src`, `file_type_name`, `ver` | `{}` |
| `post_export_cover` | `cover_src`, `ver` | `{}` |
| `post_export_selection_cover` | `cover_src`, `ver` | `{}` |
| `post_export_preview` | `preview_src`, `ver` | `{ page_count }` |
| `post_export_preview_sum` | `ver` | `{ page_count }` |

响应格式：

```json
{ "code": 200, "data": {}, "msg": "success" }
{ "code": 400103, "data": null, "msg": "file_src or ver is empty" }
```

已实现错误码：`400100` 空指针、`400101` UTF-8 错误、`400102` JSON/字段解析失败、`400103` 必填字段为空、`400500`〜`400801` 各导出失败。

## 6. 测试

- [x] 4 组参数校验测试：`ffi_comps_tests`、`ffi_cover_tests`、`ffi_file_tests`、`ffi_preview_tests`。
- [x] 1 组真实 CorelDRAW 冒烟测试：`ffi_e2e_tests.rs`，覆盖全部 8 个接口。
- [x] 将测试从 `src/tests/` 移到 `cdrimex/tests/`，使集成测试能被 Cargo 自动发现。
- [x] 修复测试中 `Vec<u8>` 用 `as_ptr()` 转 `*mut` 的未定义行为，改用 `as_mut_ptr()`。
- [x] E2E 使用 `test_cdr/0308.cdr`（62 个图形）；`266.cdr` 无图形，不适合测选区接口。
- [x] E2E 产物写入 `test_cdr/output/`，测试结束调用 `close_without_saving()`。
- [x] 测试结果：**20 passed, 0 failed, 1 ignored**。
      - 忽略项：`test_export_preview_sum_normal_input`（原有 `#[ignore]`，依赖 CorelDRAW 环境）。
- [x] `cargo clippy --all-targets` 通过。

E2E 运行方式：

```bash
cd cdrimex
cargo test --test ffi_e2e_tests -- --nocapture --test-threads=1
```

实测导出尺寸（`maintain_aspect = true`，按页面比例缩放）：

| 产物 | 尺寸 |
| --- | --- |
| `cover.png` / `selection_cover.png` | 600×200 |
| `preview_1_preview.png` | 2000×667 |
| `select_comps.png` | 13429×4477 |

## 7. MCP 配置

- [x] 按 8 个 FFI 接口补齐 `cdrimex/src/mcp/` 下的工具定义。
- [x] 更新 `cdrimex/src/mcp.json`，工具列表从 7 项增加到 8 项。
- [x] 新增 `current_file_path.json`（对应 `post_current_file_path`）。
- [x] 更新 `cdrimex/src/dytes.json` 的 `exports`，增加 `./mcp/current_file_path.json`。
- [x] 修正描述：
      - `export_preview` 的 `preview_src` 是文件名前缀，输出为 `{preview_src}{页码}_preview.png`；
      - `export_select_comps` 支持 `png` / `jpg` / `jpeg` / `pdf`。
- [x] 校验：8 个工具定义全部存在、`name` 与文件名一致、`mcp.json` 与 `dytes.json` 的 `./mcp/*` 完全一致。

## 8. Skill 文档

已在仓库根 `skills/` 下生成 4 个项目规范：

- [x] `skills/cdrsdk-integration/SKILL.md` — cdrsdk 0.2.0 API 与 CorelDRAW 26 兼容性坑。
- [x] `skills/cdrimex-ffi/SKILL.md` — FFI 协议、错误码、裸指针安全、新增接口步骤。
- [x] `skills/cdrimex-testing/SKILL.md` — 参数测试与真实环境 E2E 规范。
- [x] `skills/cdrimex-build-release/SKILL.md` — 依赖锁定、构建、clippy、发布清单检查。

## 9. Git

- [x] 从 `main` 创建分支 `cdrimex`。
- [x] 提交 `13d75b2 feat: add cdrimex CorelDRAW plugin`：31 个文件，2027 行新增。
- [x] 推送到 `origin/cdrimex`，本地分支已跟踪远程。
- [x] `test_cdr/` 通过 `.git/info/exclude` 本地排除，文件保留但不进入版本控制。
- [x] `.gitignore` 保持与已提交版本一致，未新增 `test_cdr` 规则。

## 10. 待办 / 遗留问题

- [ ] 确认发布工具读取 `dytes.json` 的基准目录。当前 `dytes.json`、`mcp.json` 位于 `cdrimex/src/`，而 `exports` 中的 `./target/release/cdrimex.dll` 看起来以包根为基准；必要时把 `dytes.json`、`mcp.json`、`mcp/` 移到 `cdrimex/` 根目录。
- [ ] `dytes.json` 版本为 `1.0.1`，`Cargo.toml` 版本为 `0.1.0`，发布前统一。
- [ ] 建立发布流水线（`cargo build --release` + `dytes.json` 打包）。
- [ ] `skills/` 不在 Reasonix 默认技能发现路径，如需自动加载，配置 `[skills].paths = ["skills"]` 或移动到 `.reasonix/skills/`。
- [ ] `skills/` 尚未提交到 git。
- [ ] `dyteslogs 0.1.0` 仍处于 yanked 状态，等待 `cdrsdk` 上游更新或改用可用版本。
- [ ] `post_export_select_comps` / `post_export_selection_cover` 内部失败时仍返回 `code = 200`（核心函数返回 `()`），如需真实错误需调整函数签名。
- [ ] 被忽略的 `test_export_preview_sum_normal_input` 可按需启用。
