use std::ffi::{c_char, c_int};
use serde::Deserialize;
use serde_json::json;

use crate::utils::cjson::{parse_json_input, write_error, write_success};
use crate::utils::export_comps;

#[derive(Deserialize)]
struct CompsInput {
    file_src: String,
    file_type_name: String,
    ver: String,
}

/// 导出选中的对象为指定文件类型（核心函数返回 `()`，直接视为成功）
#[unsafe(no_mangle)]
pub extern "C" fn post_export_select_comps(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    let req: CompsInput = match unsafe { parse_json_input(json_in) } {
        Ok(v) => v,
        Err((code, msg)) => return write_error(json_out, code, &msg),
    };

    if req.file_src.is_empty() || req.file_type_name.is_empty() || req.ver.is_empty() {
        return write_error(json_out, 400103, "file_src, file_type_name or ver is empty");
    }

    export_comps::export_select_comps(&req.file_src, &req.file_type_name, &req.ver);
    write_success(json_out, "success", json!({}))
}