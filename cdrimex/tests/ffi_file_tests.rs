use std::ffi::{CString, c_char, CStr};
use cdrimex::ffi::file::{post_export_file, post_export_cdr_file};

// 辅助函数：创建 C 字符串指针
fn cstr(s: &str) -> *const c_char {
    CString::new(s).unwrap().into_raw()
}

// 辅助函数：从输出缓冲区读取 JSON 字符串
unsafe fn read_json_out(ptr: *mut c_char) -> String {
    CStr::from_ptr(ptr).to_string_lossy().into_owned()
}

// 辅助函数：准备输出缓冲区并返回其指针
fn output_buffer() -> (*mut c_char, Vec<u8>) {
    let mut buffer = vec![0u8; 65536];
    let ptr = buffer.as_mut_ptr() as *mut c_char;
    (ptr, buffer) // buffer 需要保持存活，否则指针悬垂
}

#[test]
fn test_export_file_null_input() {
    let (out, _buf) = output_buffer();
    let ret = post_export_file(std::ptr::null(), out);
    let output = unsafe { read_json_out(out) };
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(ret, 0);
    assert_eq!(parsed["code"], 400100);
    assert_eq!(parsed["msg"], "json_in is null");
}

#[test]
fn test_export_file_invalid_json() {
    let input = cstr("{invalid json}");
    let (out, _buf) = output_buffer();
    let _ = post_export_file(input, out);
    let output = unsafe { read_json_out(out) };
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["code"], 400102);
    assert!(parsed["msg"].as_str().unwrap().contains("invalid json"));
}

#[test]
fn test_export_file_missing_ver_field() {
    let input = cstr(r#"{"file_src":"C:\\test.cdr"}"#);
    let (out, _buf) = output_buffer();
    let _ = post_export_file(input, out);
    let output = unsafe { read_json_out(out) };
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["code"], 400102); // serde 解析失败
}

#[test]
fn test_export_file_empty_fields() {
    let input = cstr(r#"{"file_src":"","ver":""}"#);
    let (out, _buf) = output_buffer();
    let _ = post_export_file(input, out);
    let output = unsafe { read_json_out(out) };
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["code"], 400103);
    assert_eq!(parsed["msg"], "file_src or ver is empty");
}

#[test]
fn test_export_cdr_file_null_input() {
    let (out, _buf) = output_buffer();
    let _ = post_export_cdr_file(std::ptr::null(), out);
    let output = unsafe { read_json_out(out) };
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["code"], 400100);
}

#[test]
fn test_export_cdr_file_empty_fields() {
    let input = cstr(r#"{"file_src":"","ver":""}"#);
    let (out, _buf) = output_buffer();
    let _ = post_export_cdr_file(input, out);
    let output = unsafe { read_json_out(out) };
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["code"], 400103);
    assert_eq!(parsed["msg"], "file_src or ver is empty");
}