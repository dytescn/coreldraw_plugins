---
name: cdrimex-ffi
description: cdrimex 的 FFI 导出接口规范：JSON 输入输出协议、错误码、64 KiB 输出缓冲、参数校验、裸指针安全、no_mangle 导出、lib.rs 重导出。新增或修改 src/ffi、src/utils/cjson.rs 时使用。
---

# cdrimex FFI 接口规范

## 接口签名

所有导出函数使用 C ABI，接收两个指针：

```rust
#[unsafe(no_mangle)]
pub extern "C" fn post_xxx(json_in: *const c_char, json_out: *mut c_char) -> c_int
```

- `json_in`：以 NUL 结尾的 UTF-8 JSON 字符串。
- `json_out`：调用方提供、至少 `65536` 字节的缓冲区。
- 返回值：写入成功 `0`；`json_out` 为空 `-1`；内容超过缓冲区 `-2`。

## JSON 协议

成功：

```json
{ "code": 200, "data": {}, "msg": "success" }
```

失败：

```json
{ "code": 400103, "data": null, "msg": "file_src or ver is empty" }
```

统一由 `utils/cjson.rs` 的 `write_success` / `write_error` 生成，禁止各接口自行拼接 JSON。

## 错误码

| 错误码 | 含义 |
| --- | --- |
| 400100 | `json_in` 为空指针 |
| 400101 | 输入不是合法 UTF-8 |
| 400102 | 输入不是合法 JSON / 缺少必填字段 |
| 400103 | 必填字段为空字符串 |
| 400500 | `export_file` 失败 |
| 400501 | `export_cdr_file` 失败 |
| 400502 | `get_current_file_path` 失败 |
| 400600 | `export_cover` 失败 |
| 400800 | `export_preview` 失败 |
| 400801 | `export_preview_sum` 失败 |

新增接口时沿用该区段规则，并在测试中覆盖：空指针、非法 JSON、空字段。

## 新增接口步骤

1. 在 `src/utils/export_*.rs` 实现核心逻辑，返回 `Option<T>`。
2. 在 `src/ffi/*.rs` 定义输入结构体（`#[derive(Deserialize)]`）和导出函数。
3. 用 `unsafe { parse_json_input(json_in) }` 解析输入；先校验空字段，再调用核心逻辑。
4. 在 `src/lib.rs` 添加 `pub use` 重导出（容易漏）。
5. 更新 `src/dytes.json` 的 `exports` 与 `src/mcp.json` 的 `tools`，并补齐对应的 `mcp/*.json` 工具定义。
6. 添加参数校验测试，并在真实 CorelDRAW 冒烟测试中覆盖正常路径。

## 裸指针与安全

- `cstr_to_str` 和 `parse_json_input` 标记为 `unsafe fn`，调用处必须写 `unsafe { ... }`。
- FFI 导出函数本身保持 `pub extern "C"`（不标 `unsafe`），由 C 调用方保证指针有效。
- `src/ffi/mod.rs` 中统一允许：

```rust
#![allow(clippy::not_unsafe_ptr_arg_deref)]
```

理由：FFI 边界按 C ABI 约定接收裸指针。

## 输出缓冲区

- 长度常量：`pub const MAX_OUTPUT_LEN: usize = 65536;`
- 写入前检查 `bytes.len() + 1 > MAX_OUTPUT_LEN`，并保证末尾 NUL。
- 测试缓冲区使用 `Vec<u8>` 的 `as_mut_ptr()`，不能把 `as_ptr()` 的 `*const` 强转成 `*mut`。

## 当前接口清单

| 导出函数 | 输入字段 | 输出 |
| --- | --- | --- |
| `post_export_file` | `file_src`, `ver` | `{}` |
| `post_export_cdr_file` | `file_src`, `ver` | `{}` |
| `post_current_file_path` | `ver` | `{ "file_path": "..." }` |
| `post_export_select_comps` | `file_src`, `file_type_name`, `ver` | `{}` |
| `post_export_cover` | `cover_src`, `ver` | `{}` |
| `post_export_selection_cover` | `cover_src`, `ver` | `{}` |
| `post_export_preview` | `preview_src`, `ver` | `{ "page_count": N }` |
| `post_export_preview_sum` | `ver` | `{ "page_count": N }` |

`preview_src` 是**前缀**：实际文件名为 `{preview_src}{i}_preview.png`。

## 已知行为

`post_export_select_comps` 和 `post_export_selection_cover` 的核心函数当前返回 `()`，即使内部失败也返回 `code = 200`。如需真实错误，需要先让核心函数返回 `Option<()>` 并同步修改 FFI 层与测试。
