//! CorelDRAW 真实环境下的 FFI 冒烟测试。
//!
//! 运行前要求：
//! - CorelDRAW 已安装并可用 `CorelDRAW.Application.{CORELDRAW_VERSION}` 连接；
//! - `test_cdr/266.cdr` 存在。
//!
//! 通过 `CARGO_MANIFEST_DIR` 定位仓库根目录下的 `test_cdr/`，导出产物写入
//! `test_cdr/output/`。测试会打开样本文件并在结束时关闭（不保存）。
//!
//! 由于会操作 CorelDRAW 的当前实例，建议串行运行：
//! `cargo test --test ffi_e2e_tests -- --nocapture --test-threads=1`

use std::ffi::{c_char, CStr, CString};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use cdrsdk::app::IvgApplication;

use cdrimex::ffi::comps::post_export_select_comps;
use cdrimex::ffi::cover::{post_export_cover, post_export_selection_cover};
use cdrimex::ffi::file::{post_current_file_path, post_export_cdr_file, post_export_file};
use cdrimex::ffi::preview::{post_export_preview, post_export_preview_sum};

const MAX_OUTPUT_LEN: usize = 65536;

struct ComGuard;

impl ComGuard {
    fn init() -> Self {
        unsafe {
            let _ = windows::Win32::System::Com::CoInitialize(None);
        }
        Self
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        unsafe {
            windows::Win32::System::Com::CoUninitialize();
        }
    }
}

fn version() -> String {
    std::env::var("CORELDRAW_VERSION").unwrap_or_else(|_| "26".to_string())
}

fn test_cdr_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cdrimex should have a parent directory")
        .join("test_cdr")
}

fn output_dir() -> PathBuf {
    let dir = test_cdr_dir().join("output");
    std::fs::create_dir_all(&dir).expect("create output dir");
    dir
}

/// 调用 FFI 函数并解析输出 JSON。
fn call_json(
    input: serde_json::Value,
    f: extern "C" fn(*const c_char, *mut c_char) -> i32,
) -> serde_json::Value {
    let input = CString::new(input.to_string()).expect("json input must not contain NUL");
    let mut out = vec![0u8; MAX_OUTPUT_LEN];
    let ret = f(input.as_ptr(), out.as_mut_ptr() as *mut c_char);
    assert_eq!(ret, 0, "FFI returned unexpected status: {ret}");

    let text = unsafe { CStr::from_ptr(out.as_ptr() as *const c_char) }
        .to_string_lossy()
        .into_owned();
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("invalid JSON output: {e}\nraw: {text}"))
}

fn is_png(path: &Path) -> bool {
    let mut header = [0u8; 8];
    std::fs::File::open(path)
        .and_then(|mut f| f.read_exact(&mut header))
        .map(|_| header == *b"\x89PNG\r\n\x1a\n")
        .unwrap_or(false)
}

fn wait_for_document(app: &IvgApplication) -> cdrsdk::document::IvgDocument {
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        if let Some(doc) = app.active_document() {
            if doc
                .full_file_name()
                .map(|path| !path.is_empty())
                .unwrap_or(false)
            {
                return doc;
            }
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for CorelDRAW document"
        );
        std::thread::sleep(Duration::from_millis(500));
    }
}

#[test]
fn all_ffi_interfaces_work() {
    let _com = ComGuard::init();
    let ver = version();

    let src = test_cdr_dir().join("0308.cdr");
    assert!(src.exists(), "missing test document: {}", src.display());
    let src_str = src.to_string_lossy().into_owned();

    let out = output_dir();
    let out_str = |name: &str| out.join(name).to_string_lossy().into_owned();

    let app = IvgApplication::new(&ver)
        .unwrap_or_else(|| panic!("CorelDRAW Application.{ver} is not available"));
    assert!(
        app.open_document(src_str.clone()),
        "open_document failed: {src_str}"
    );
    let doc = wait_for_document(&app);
    let opened_path = doc.full_file_name().unwrap_or_default();
    assert!(
        std::fs::canonicalize(&opened_path).ok() == std::fs::canonicalize(&src).ok(),
        "active document mismatch: opened={opened_path}, expected={src_str}"
    );

    // 1. post_current_file_path
    let resp = call_json(
        serde_json::json!({ "ver": ver }),
        post_current_file_path,
    );
    assert_eq!(resp["code"], 200, "post_current_file_path: {resp}");
    assert_eq!(resp["data"]["file_path"], opened_path);

    // 2. post_export_cdr_file
    let cdr_copy = out_str("export_cdr_file.cdr");
    let resp = call_json(
        serde_json::json!({ "file_src": cdr_copy, "ver": ver }),
        post_export_cdr_file,
    );
    assert_eq!(resp["code"], 200, "post_export_cdr_file: {resp}");
    assert!(Path::new(&cdr_copy).exists(), "copied cdr not found");
    assert_eq!(
        std::fs::metadata(&cdr_copy).unwrap().len(),
        std::fs::metadata(&src).unwrap().len(),
        "copied cdr size differs"
    );

    // 3. post_export_file
    let exported_cdr = out_str("export_file.cdr");
    let resp = call_json(
        serde_json::json!({ "file_src": exported_cdr, "ver": ver }),
        post_export_file,
    );
    assert_eq!(resp["code"], 200, "post_export_file: {resp}");
    assert!(Path::new(&exported_cdr).exists(), "exported cdr not found");

    // 4. post_export_cover（600x600 PNG）
    let cover_png = out_str("cover.png");
    let resp = call_json(
        serde_json::json!({ "cover_src": cover_png, "ver": ver }),
        post_export_cover,
    );
    assert_eq!(resp["code"], 200, "post_export_cover: {resp}");
    assert!(is_png(Path::new(&cover_png)), "cover.png is not a valid PNG");

    // 5. post_export_preview_sum
    let resp = call_json(
        serde_json::json!({ "ver": ver }),
        post_export_preview_sum,
    );
    assert_eq!(resp["code"], 200, "post_export_preview_sum: {resp}");
    let page_count = resp["data"]["page_count"]
        .as_u64()
        .expect("page_count should be a number");
    assert!(page_count >= 1, "unexpected page count: {page_count}");

    // 6. post_export_preview（2000x2000 PNG，每页一张）
    let preview_prefix = out.join("preview_").to_string_lossy().into_owned();
    let resp = call_json(
        serde_json::json!({ "preview_src": preview_prefix, "ver": ver }),
        post_export_preview,
    );
    assert_eq!(resp["code"], 200, "post_export_preview: {resp}");
    assert_eq!(resp["data"]["page_count"].as_u64(), Some(page_count));
    for i in 1..=page_count {
        let page_png = out.join(format!("preview_{i}_preview.png"));
        assert!(
            is_png(&page_png),
            "preview page {i} is not a valid PNG: {}",
            page_png.display()
        );
    }

    // 创建选区，供后两个导出接口使用。
    // Document.Shapes 在 CorelDRAW 26 上不可用，改从活动页面获取。
    let active_page = doc.active_page().expect("document has no active page");
    let shapes = active_page.shapes();
    let shape_count = shapes.as_ref().and_then(|s| s.count()).unwrap_or(0);
    eprintln!("shapes: available={}, count={shape_count}", shapes.is_some());
    assert!(
        shape_count > 0,
        "document has no shapes (doc.shapes() available: {})",
        shapes.is_some()
    );
    let shapes = shapes.expect("document has no shapes collection");
    let shapes_range = shapes.all_range().expect("shapes.all_range failed");
    assert!(
        shapes_range.create_selection(),
        "create_selection failed"
    );

    // 7. post_export_select_comps（导出选中对象为 PNG）
    let comps_png = out_str("select_comps.png");
    let resp = call_json(
        serde_json::json!({
            "file_src": comps_png,
            "file_type_name": "png",
            "ver": ver,
        }),
        post_export_select_comps,
    );
    assert_eq!(resp["code"], 200, "post_export_select_comps: {resp}");
    assert!(
        is_png(Path::new(&comps_png)),
        "select_comps.png is not a valid PNG"
    );

    // 8. post_export_selection_cover（选中对象封面，600x600 PNG）
    let selection_cover = out_str("selection_cover.png");
    let resp = call_json(
        serde_json::json!({ "cover_src": selection_cover, "ver": ver }),
        post_export_selection_cover,
    );
    assert_eq!(resp["code"], 200, "post_export_selection_cover: {resp}");
    assert!(
        is_png(Path::new(&selection_cover)),
        "selection_cover.png is not a valid PNG"
    );

    doc.close_without_saving();
}
