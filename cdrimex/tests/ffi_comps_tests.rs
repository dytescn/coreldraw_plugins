use std::ffi::{CString, c_char, CStr};
use cdrimex::ffi::comps::post_export_select_comps;

fn cstr(s: &str) -> *const c_char {
    CString::new(s).unwrap().into_raw()
}

unsafe fn read_json_out(ptr: *mut c_char) -> String {
    CStr::from_ptr(ptr).to_string_lossy().into_owned()
}

fn output_buffer() -> (*mut c_char, Vec<u8>) {
    let mut buffer = vec![0u8; 65536];
    let ptr = buffer.as_mut_ptr() as *mut c_char;
    (ptr, buffer)
}

#[test]
fn test_export_select_comps_null_input() {
    let (out, _buf) = output_buffer();
    let _: i32 = post_export_select_comps(std::ptr::null(), out);
    let output = unsafe { read_json_out(out) };
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["code"], 400100);
}

#[test]
fn test_export_select_comps_invalid_json() {
    let input = cstr("{invalid json}");
    let (out, _buf) = output_buffer();
    let _ = post_export_select_comps(input, out);
    let output = unsafe { read_json_out(out) };
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["code"], 400102);
}

#[test]
fn test_export_select_comps_empty_fields() {
    let input = cstr(r#"{"file_src":"","file_type_name":"","ver":""}"#);
    let (out, _buf) = output_buffer();
    let _ = post_export_select_comps(input, out);
    let output = unsafe { read_json_out(out) };
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["code"], 400103);
    assert_eq!(parsed["msg"], "file_src, file_type_name or ver is empty");
}