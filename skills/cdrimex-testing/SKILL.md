---
name: cdrimex-testing
description: cdrimex 测试规范：参数校验测试 + 真实 CorelDRAW 端到端冒烟测试；test_cdr 样本与输出目录、串行运行、选区创建、PNG 校验、已知环境坑。编写或运行 cdrimex 测试时使用。
---

# cdrimex 测试规范

## 测试布局

- 参数校验测试：`cdrimex/tests/ffi_*_tests.rs`，只覆盖空指针、非法 JSON、空字段，不依赖 CorelDRAW。
- 真实环境冒烟测试：`cdrimex/tests/ffi_e2e_tests.rs`，覆盖 8 个接口的正常路径。
- 不要放在 `src/tests/`：只有 `cdrimex/tests/` 下的文件才会作为集成测试自动执行。

## 测试样本

- 样本目录：仓库根 `test_cdr/`。
- 默认使用 `0308.cdr`（62 个图形），不要用 `266.cdr`（打开后 `shapes` 为空）。
- 导出产物写入 `test_cdr/output/`，该目录已在 `.gitignore` 中忽略。
- 路径通过 `env!("CARGO_MANIFEST_DIR")` 的上级目录定位，避免依赖当前工作目录：

```rust
fn test_cdr_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("test_cdr")
}
```

## 运行方式

```bash
cd cdrimex
cargo test
cargo test --test ffi_e2e_tests -- --nocapture --test-threads=1
```

- E2E 测试会操作本机正在运行的 CorelDRAW 实例，必须串行。
- 版本通过 `CORELDRAW_VERSION` 覆盖，默认 `26`。
- 测试结束调用 `doc.close_without_saving()`，避免残留打开的文档。

## E2E 覆盖顺序

1. 打开 `test_cdr/0308.cdr`，轮询等待活动文档出现且 `full_file_name()` 非空（最多 60 秒）。
2. `post_current_file_path` → 校验原路径。
3. `post_export_cdr_file` → 校验文件存在且大小一致。
4. `post_export_file` → 校验导出文件存在。
5. `post_export_cover` → 校验 PNG 签名。
6. `post_export_preview_sum` → 校验 `page_count >= 1`。
7. `post_export_preview` → 校验每页 PNG 文件。
8. 创建选区后调用 `post_export_select_comps`、`post_export_selection_cover`。

## 创建选区

`Document.Shapes` 在 CorelDRAW 26 不可用，必须从活动页面获取：

```rust
let page = doc.active_page().expect("no active page");
let shapes = page.shapes().expect("no shapes");
let range = shapes.all_range().expect("all_range failed");
assert!(range.create_selection());
```

## 断言约定

- FFI 返回状态必须为 `0`。
- 成功响应 `code == 200`，失败场景断言具体错误码。
- 图片文件至少校验 PNG 签名 `\x89PNG\r\n\x1a\n`。
- 尺寸不做硬编码断言：`maintain_aspect = true` 时 600×600 选项会按页面比例缩放（例如得到 600×200）；preview 的 2000×2000 同理。

## 常见失败

| 现象 | 原因 / 处理 |
| --- | --- |
| `preview` 无文件 | `IvgPages::item` 在 26 上不可用，改用 `first()` + `next()` |
| `document has no shapes` | `Document.Shapes` 不可用，改用 `active_page().shapes()`；或换有内容的样本 |
| `Export failed` 未打印但无文件 | 检查 5 参数 `Export` 是否真正被调用；在 `export_doc` 中保留错误日志 |
| E2E 互相干扰 | 加 `--test-threads=1`，并确保上次失败后残留文档已关闭 |
