use crate::utils::com::{export_doc, get_app_and_doc, ComGuard};

const PNG_FILE_TYPE: u32 = 802;

// 内部导出逻辑，export_type 用于区分导出模式：1 = 整个文档，2 = 当前选区。
fn export_cover_internal(file_path: &str, export_type: u32, ver: &str) -> Option<()> {
    let _guard = ComGuard::init();
    let (app, doc) = get_app_and_doc(ver)?;

    let exp_opt = app.create_export_options()?;
    exp_opt.set_size_x(600);
    exp_opt.set_size_y(600);
    exp_opt.set_maintain_aspect(true);
    exp_opt.set_transparent(true);
    let palette = app.create_palette_options()?;

    export_doc(&doc, file_path, PNG_FILE_TYPE, export_type, &exp_opt, &palette)
}

pub fn export_selection_cover(cover_src: &str, ver: &str) {
    let _ = export_cover_internal(cover_src, 2, ver);
}

pub fn export_cover(cover_src: String, ver: &str) -> Option<()> {
    export_cover_internal(&cover_src, 1, ver)
}
