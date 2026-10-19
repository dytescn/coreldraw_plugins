//! 对外 FFI 接口层。
//!
//! 这里放置通过 `#[no_mangle] pub extern "C"` 暴露给宿主程序
//! （CorelDRAW 插件调用方）的函数，保持 ABI 稳定。
//!
//! FFI 导出函数按 C ABI 约定接收裸指针，指针有效性由调用方保证，
//! 因此这里统一关闭该 lint。
#![allow(clippy::not_unsafe_ptr_arg_deref)]

pub mod file;
pub mod comps;
pub mod cover;
pub mod preview;
