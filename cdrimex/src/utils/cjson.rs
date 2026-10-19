use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::ffi::{c_char, c_int, CStr};

/// 输出缓冲区最大长度（包括结尾 null）
/// 调用者必须保证 `json_out` 至少有 `MAX_OUTPUT_LEN` 字节可用空间。
pub const MAX_OUTPUT_LEN: usize = 65536;

/// 将 C 字符串指针转换为 &str，处理 null 和 UTF-8 错误
///
/// # Safety
/// `ptr` 必须指向以 null 结尾的有效 C 字符串，且在其生命周期内保持有效。
pub unsafe fn cstr_to_str<'a>(ptr: *const c_char) -> Result<&'a str, (i32, String)> {
    if ptr.is_null() {
        return Err((400100, "json_in is null".to_string()));
    }
    unsafe { CStr::from_ptr(ptr).to_str() }.map_err(|_| (400101, "invalid utf-8".to_string()))
}

/// 解析 JSON 输入为指定类型 T
///
/// # Safety
/// `ptr` 必须指向以 null 结尾的有效 C 字符串。
pub unsafe fn parse_json_input<T: DeserializeOwned>(ptr: *const c_char) -> Result<T, (i32, String)> {
    let body = unsafe { cstr_to_str(ptr)? };
    serde_json::from_str::<T>(body)
        .map_err(|e| (400102, format!("invalid json: {}", e)))
}

/// 将 JSON 值写入 C 缓冲区，并确保 null 结尾
/// 成功返回 0；`json_out` 为 null 返回 -1；内容过长返回 -2。
fn write_json_out(json_out: *mut c_char, value: Value) -> c_int {
    let out_str = value.to_string();
    let bytes = out_str.as_bytes();
    unsafe {
        if json_out.is_null() {
            return -1;
        }
        // 检查是否超出缓冲区（包括结尾 null）
        if bytes.len() + 1 > MAX_OUTPUT_LEN {
            return -2;
        }
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), json_out as *mut u8, bytes.len());
        *json_out.add(bytes.len()) = 0;
    }
    0
}

/// 返回错误 JSON
/// 错误响应格式：{ "code": 错误码, "data": null, "msg": 错误信息 }
pub fn write_error(json_out: *mut c_char, code: i32, msg: &str) -> c_int {
    write_json_out(
        json_out,
        json!({"code": code, "data": null, "msg": msg}),
    )
}

/// 返回成功 JSON
/// 成功响应格式：{ "code": 200, "data": 数据对象, "msg": 提示信息 }
pub fn write_success(json_out: *mut c_char, msg: &str, data: Value) -> c_int {
    write_json_out(
        json_out,
        json!({ "code": 200, "data": data, "msg": msg}),
    )
}