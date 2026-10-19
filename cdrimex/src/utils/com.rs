use cdrsdk::app::IvgApplication;
use cdrsdk::document::IvgDocument;
use cdrsdk::structs::{IvgStructExportOptions, IvgStructPaletteOptions};
use windows::Win32::System::Com;
use wincom::Variant;

/// 轻量 RAII 守卫，自动管理 COM 生命周期。
pub(crate) struct ComGuard;

impl ComGuard {
    pub(crate) fn init() -> Self {
        unsafe {
            let _ = Com::CoInitialize(None);
        }
        Self
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        unsafe {
            Com::CoUninitialize();
        }
    }
}

/// 连接指定版本的 CorelDRAW 并取得当前活动文档。
pub(crate) fn get_app_and_doc(ver: &str) -> Option<(IvgApplication, IvgDocument)> {
    let app = IvgApplication::new(ver)?;
    let doc = app.active_document()?;
    Some((app, doc))
}

/// 以 5 参数形式调用 CorelDRAW 的 `Export` 方法。
///
/// `range`：1 = 整个文档，2 = 当前选区。
pub(crate) fn export_doc(
    doc: &IvgDocument,
    path: &str,
    file_type: u32,
    range: u32,
    options: &IvgStructExportOptions,
    palette: &IvgStructPaletteOptions,
) -> Option<()> {
    let args = vec![
        Variant::from_str(path),
        Variant::from_i64(file_type as i64),
        Variant::from_i64(range as i64),
        options.as_variant(),
        palette.as_variant(),
    ];
    match doc.raw().invoke_method("Export", args) {
        Ok(_) => Some(()),
        Err(e) => {
            eprintln!("Export failed for {path}: {e:?}");
            None
        }
    }
}
