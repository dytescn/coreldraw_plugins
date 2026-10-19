use std::ffi::{c_char, c_int};
use serde::Deserialize;
use serde_json::json;

use crate::utils::cjson::{parse_json_input, write_error, write_success};
use crate::utils::export_file;

#[derive(Deserialize)]
struct FileInput {
    file_src: String,
    ver: String,
}

/// 导出文件（自动处理路径为空的情况）
#[unsafe(no_mangle)]
pub extern "C" fn post_export_file(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    let req: FileInput = match unsafe { parse_json_input(json_in) } {
        Ok(v) => v,
        Err((code, msg)) => return write_error(json_out, code, &msg),
    };

    if req.file_src.is_empty() || req.ver.is_empty() {
        return write_error(json_out, 400103, "file_src or ver is empty");
    }

    match export_file::export_file(req.file_src, &req.ver) {
        Some(()) => write_success(json_out, "success", json!({})),
        None => write_error(json_out, 400500, "export_file failed"),
    }
}

/// 复制 CDR 文件
#[unsafe(no_mangle)]
pub extern "C" fn post_export_cdr_file(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    let req: FileInput = match unsafe { parse_json_input(json_in) } {
        Ok(v) => v,
        Err((code, msg)) => return write_error(json_out, code, &msg),
    };

    if req.file_src.is_empty() || req.ver.is_empty() {
        return write_error(json_out, 400103, "file_src or ver is empty");
    }

    match export_file::export_cdr_file(req.file_src, &req.ver) {
        Some(()) => write_success(json_out, "success", json!({})),
        None => write_error(json_out, 400501, "export_cdr_file failed"),
    }
}


#[unsafe(no_mangle)]
pub extern "C" fn post_current_file_path(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    // 解析输入（只需要 ver）
    #[derive(Deserialize)]
    struct Input { ver: String }

    let req: Input = match unsafe { parse_json_input(json_in) } {
        Ok(v) => v,
        Err((code, msg)) => return write_error(json_out, code, &msg),
    };

    if req.ver.is_empty() {
        return write_error(json_out, 400103, "ver is empty");
    }

    match export_file::get_current_file_path(&req.ver) {
        Some(path) => write_success(json_out, "success", json!({ "file_path": path })),
        None => write_error(json_out, 400502, "get_current_file_path failed"),
    }
}
