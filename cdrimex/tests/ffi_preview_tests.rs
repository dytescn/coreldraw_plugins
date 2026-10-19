use std::ffi::{CString, c_char, CStr};
use cdrimex::ffi::preview::{post_export_preview, post_export_preview_sum};

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
fn test_export_preview_null_input() {
    let (out, _buf) = output_buffer();
    let _ = post_export_preview(std::ptr::null(), out);
    let output = unsafe { read_json_out(out) };
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["code"], 400100);
}

#[test]
fn test_export_preview_empty_fields() {
    let input = cstr(r#"{"preview_src":"","ver":""}"#);
    let (out, _buf) = output_buffer();
    let _ = post_export_preview(input, out);
    let output = unsafe { read_json_out(out) };
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["code"], 400103);
    assert_eq!(parsed["msg"], "preview_src or ver is empty");
}

#[test]
fn test_export_preview_sum_null_input() {
    let (out, _buf) = output_buffer();
    let _ = post_export_preview_sum(std::ptr::null(), out);
    let output = unsafe { read_json_out(out) };
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["code"], 400100);
}

#[test]
fn test_export_preview_sum_invalid_json() {
    let input = cstr("{invalid json}");
    let (out, _buf) = output_buffer();
    let _ = post_export_preview_sum(input, out);
    let output = unsafe { read_json_out(out) };
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["code"], 400102);
}

#[test]
fn test_export_preview_sum_empty_ver() {
    let input = cstr(r#"{"ver":""}"#);
    let (out, _buf) = output_buffer();
    let _: i32 = post_export_preview_sum(input, out);
    let output = unsafe { read_json_out(out) };
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["code"], 400103);
    assert_eq!(parsed["msg"], "ver is empty");
}


#[test]
#[ignore] // 依赖 CorelDRAW 环境，默认跳过；可手动运行
fn test_export_preview_sum_normal_input() {
    let input = cstr(r#"{"ver":"26"}"#);
    let (out, _buf) = output_buffer();
    let _ = post_export_preview_sum(input, out);
    let output = unsafe { read_json_out(out) };

    // 打印完整的 JSON 字符串
    println!("完整 JSON 输出: {}", output);

    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    // 打印解析后的 JSON（pretty 格式）
    println!("解析后的 JSON:\n{}", serde_json::to_string_pretty(&parsed).unwrap());

    assert!(
        parsed["code"] == 200 || parsed["code"] == 400801,
        "Unexpected code: {}",
        parsed["code"]
    );

    if parsed["code"] == 200 {
        assert!(parsed["data"]["page_count"].is_number());
    }
}