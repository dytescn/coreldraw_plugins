use crate::utils::com::{export_doc, get_app_and_doc, ComGuard};

const PNG_FILE_TYPE: u32 = 802;

pub fn export_preview(preview_src: String, ver: &str) -> Option<usize> {
    let _guard = ComGuard::init();
    let (app, doc) = get_app_and_doc(ver)?;

    let exp_opt = app.create_export_options()?;
    exp_opt.set_size_x(2000);
    exp_opt.set_size_y(2000);
    exp_opt.set_maintain_aspect(true);
    exp_opt.set_transparent(true);
    let palette = app.create_palette_options()?;

    let active_page = doc.active_page()?;
    let pages = doc.pages()?;
    let page_count = pages.count()?;

    // `IvgPages::item` 在 CorelDRAW 26 上不可用（Item 是带参数的属性），
    // 改用 First + Next 链式遍历。
    let mut current = pages.first();
    for i in 1..=page_count {
        let Some(cur_page) = current else { break; };
        let src = format!("{}{}_preview.png", preview_src, i);
        cur_page.activate();
        // 按原逻辑忽略单页导出返回值
        let _ = export_doc(&doc, &src, PNG_FILE_TYPE, 1, &exp_opt, &palette);
        current = cur_page.next();
    }
    active_page.activate();

    Some(page_count as usize)
}

pub fn export_preview_sum(ver: &str) -> Option<usize> {
    let _guard = ComGuard::init();
    let (_, doc) = get_app_and_doc(ver)?;
    Some(doc.pages()?.count()? as usize)
}
