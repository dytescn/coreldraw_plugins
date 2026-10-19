use std::ffi::{c_char, c_int};
use serde::Deserialize;
use serde_json::json;

use crate::utils::cjson::{parse_json_input, write_error, write_success};
use crate::utils::export_preview;

#[derive(Deserialize)]
struct PreviewInput {
    preview_src: String,
    ver: String,
}

#[derive(Deserialize)]
struct PreviewSumInput {
    ver: String,
}

/// 导出所有页面的预览图（PNG 格式，2000x2000）
#[unsafe(no_mangle)]
pub extern "C" fn post_export_preview(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    let req: PreviewInput = match unsafe { parse_json_input(json_in) } {
        Ok(v) => v,
        Err((code, msg)) => return write_error(json_out, code, &msg),
    };

    if req.preview_src.is_empty() || req.ver.is_empty() {
        return write_error(json_out, 400103, "preview_src or ver is empty");
    }

    match export_preview::export_preview(req.preview_src, &req.ver) {
        Some(page_count) => write_success(json_out, "success", json!({ "page_count": page_count })),
        None => write_error(json_out, 400800, "export_preview failed"),
    }
}

/// 获取文档总页数（不导出）
#[unsafe(no_mangle)]
pub extern "C" fn post_export_preview_sum(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    let req: PreviewSumInput = match unsafe { parse_json_input(json_in) } {
        Ok(v) => v,
        Err((code, msg)) => return write_error(json_out, code, &msg),
    };

    if req.ver.is_empty() {
        return write_error(json_out, 400103, "ver is empty");
    }

    match export_preview::export_preview_sum(&req.ver) {
        Some(page_count) => write_success(json_out, "success", json!({ "page_count": page_count })),
        None => write_error(json_out, 400801, "export_preview_sum failed"),
    }
}