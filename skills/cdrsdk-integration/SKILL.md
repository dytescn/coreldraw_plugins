---
name: cdrsdk-integration
description: CorelDRAW COM 集成规范（cdrsdk 0.2.0）：模块路径、API 命名、Export 调用、页面与图形遍历、COM 生命周期，以及 CorelDRAW 26 上已确认的兼容性坑。修改 cdrimex 的 utils/export_* 时使用。
---

# cdrsdk 0.2.0 集成规范

## 版本与入口

- CorelDRAW 版本通过 ProgID `CorelDRAW.Application.{ver}` 连接，本项目使用 `ver = "26"`（CorelDRAW 2025/26.x）。
- 版本号可通过环境变量 `CORELDRAW_VERSION` 覆盖；默认 `26`。
- 入口：

```rust
use cdrsdk::app::IvgApplication;
use cdrsdk::document::IvgDocument;

let app = IvgApplication::new(ver)?;          // Option<Self>
let doc = app.active_document()?;            // Option<IvgDocument>
```

## API 对照（旧 0.1.x → 0.2.0）

| 旧写法 | 0.2.0 写法 |
| --- | --- |
| `cdrsdk::sdk::application::IvgApplication` | `cdrsdk::app::IvgApplication` |
| `cdrsdk::sdk::document::IvgDocument` | `cdrsdk::document::IvgDocument` |
| `app.get_active_document()` | `app.active_document()` |
| `doc.get_fullfilename()` | `doc.full_file_name() -> Option<String>` |
| `doc.put_fullfilename(p)` | `doc.save_as(p) -> bool`（无直接 setter） |
| `app.create_struct_export_options()` | `app.create_export_options() -> Option<IvgStructExportOptions>` |
| `app.create_struct_palette_options()` | `app.create_palette_options() -> Option<IvgStructPaletteOptions>` |
| `doc.get_activepage()` | `doc.active_page()` |
| `doc.get_pages()` | `doc.pages()` |
| `pages.get_count()` | `pages.count() -> Option<i64>` |
| `pages.get_item(i)` | **不要用 `pages.item(i)`**，见下方兼容性坑 |
| `doc.export(path, type, range, exp, pale)` | 用 `doc.raw().invoke_method("Export", args)` 保留 5 参数语义 |
| `filetypes::cdr_file_type_to_u32` | 自行维护映射：`pdf=1333`、`jpg/jpeg=820`、`png=802` |

## Export 调用

`cdrsdk 0.2.0` 的高层 `export`/`export_ex` 没有“整文档 / 选区”参数。需要保留原行为时直接走底层：

```rust
use wincom::Variant;

let args = vec![
    Variant::from_str(path),
    Variant::from_i64(file_type as i64),
    Variant::from_i64(range as i64),   // 1 = 整个文档，2 = 当前选区
    exp_opt.as_variant(),
    palette.as_variant(),
];
doc.raw().invoke_method("Export", args)?;
```

`wincom` 必须作为直接依赖（`wincom = "0.2.0"`），否则无法构造 `Variant`。

## CorelDRAW 26 上已确认的兼容性坑

1. **`IvgPages::item(i)` 返回 `None`**
   - 原因：`Item` 在 COM 中是带参数的属性，`invoke_method("Item")` 不可靠。
   - 解决：用 `pages.first()` 取第一页，再用 `page.next()` 链式遍历：

```rust
let page_count = pages.count()?;
let mut current = pages.first();
for i in 1..=page_count {
    let Some(page) = current else { break };
    page.activate();
    // export ...
    current = page.next();
}
```

2. **`Document.Shapes` 返回 `None`**
   - 原因：CorelDRAW 26 的文档对象上该属性不可用。
   - 解决：从活动页面取图形：`doc.active_page()?.shapes()`。

3. **`save_as` 会实际落盘**
   - 0.2.0 没有“只改路径”的 setter。`export_file` 在文档已有路径时先 `fs::copy`，再调用 `save_as`；接受这一次额外写入。

## COM 生命周期

每个导出函数都用 RAII 守卫，保证 `CoInitialize` / `CoUninitialize` 成对：

```rust
struct ComGuard;
impl ComGuard {
    fn init() -> Self {
        unsafe { let _ = windows::Win32::System::Com::CoInitialize(None); }
        Self
    }
}
impl Drop for ComGuard {
    fn drop(&mut self) {
        unsafe { windows::Win32::System::Com::CoUninitialize(); }
    }
}
```

在 `utils/com.rs` 中复用 `ComGuard`、`get_app_and_doc`、`export_doc`，不要在多个 `export_*` 文件里重复实现。

## 错误处理

- 核心导出函数统一返回 `Option<T>`，由 FFI 层转换为 JSON 错误码。
- 不要用 `unwrap()` 处理 COM 结果；`get_active_document().unwrap()` 会在无文档时 panic。
- 导出失败时至少 `eprintln!` 记录路径和错误，便于定位。
