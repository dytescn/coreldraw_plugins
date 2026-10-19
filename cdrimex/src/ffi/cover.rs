use std::ffi::{c_char, c_int};
use serde::Deserialize;
use serde_json::json;

use crate::utils::cjson::{parse_json_input, write_error, write_success};
use crate::utils::export_cover;

#[derive(Deserialize)]
struct CoverInput {
    cover_src: String,
    ver: String,
}

/// 导出单张封面（PNG 格式，600x600）
#[unsafe(no_mangle)]
pub extern "C" fn post_export_cover(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    let req: CoverInput = match unsafe { parse_json_input(json_in) } {
        Ok(v) => v,
        Err((code, msg)) => return write_error(json_out, code, &msg),
    };

    if req.cover_src.is_empty() || req.ver.is_empty() {
        return write_error(json_out, 400103, "cover_src or ver is empty");
    }

    match export_cover::export_cover(req.cover_src, &req.ver) {
        Some(()) => write_success(json_out, "success", json!({})),
        None => write_error(json_out, 400600, "export_cover failed"),
    }
}

/// 导出选中对象的封面（核心函数返回 `()`，直接视为成功）
#[unsafe(no_mangle)]
pub extern "C" fn post_export_selection_cover(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    let req: CoverInput = match unsafe { parse_json_input(json_in) } {
        Ok(v) => v,
        Err((code, msg)) => return write_error(json_out, code, &msg),
    };

    if req.cover_src.is_empty() || req.ver.is_empty() {
        return write_error(json_out, 400103, "cover_src or ver is empty");
    }

    export_cover::export_selection_cover(&req.cover_src, &req.ver);
    write_success(json_out, "success", json!({}))
}
