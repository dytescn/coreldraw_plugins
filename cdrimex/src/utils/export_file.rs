use crate::utils::com::{get_app_and_doc, ComGuard};

pub fn export_file(file_src: String, ver: &str) -> Option<()> {
    let _guard = ComGuard::init();
    let (_, doc) = get_app_and_doc(ver)?;

    let doc_file_path = doc.full_file_name().unwrap_or_default();
    if doc_file_path.is_empty() {
        return doc.save_as(file_src.as_str()).then_some(());
    }

    if let Err(e) = std::fs::copy(&doc_file_path, &file_src) {
        eprintln!("copy failed: {:?}", e);
        return None;
    }
    let _ = doc.save_as(file_src.as_str());
    Some(())
}

pub fn export_cdr_file(file_src: String, ver: &str) -> Option<()> {
    let _guard = ComGuard::init();
    let (_, doc) = get_app_and_doc(ver)?;

    let doc_file_path = doc.full_file_name()?;
    if doc_file_path.is_empty() {
        return None;
    }
    if let Err(e) = std::fs::copy(&doc_file_path, &file_src) {
        eprintln!("copy failed: {:?}", e);
        return None;
    }
    Some(())
}

pub fn get_current_file_path(ver: &str) -> Option<String> {
    let _guard = ComGuard::init();
    let (_, doc) = get_app_and_doc(ver)?;
    doc.full_file_name().filter(|path| !path.is_empty())
}
