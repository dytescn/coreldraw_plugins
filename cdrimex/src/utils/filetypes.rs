/// 将文件类型名转换为 CorelDRAW 导出过滤器编号。
pub(crate) fn cdr_file_type_to_u32(file_type_name: &str) -> u32 {
    match file_type_name.to_ascii_lowercase().as_str() {
        "pdf" => 1333,
        "jpg" | "jpeg" => 820,
        "png" => 802,
        _ => 0,
    }
}
