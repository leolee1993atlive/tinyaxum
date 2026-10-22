//! DuckDB 初始化诊断工具
//!
//! 背景:正式环境 `Connection::open` 在 `duckdb_create_config` 处 panic
//! (assert state == 0)。`duckdb_create_config` 的 C 实现用 `catch(...)`
//! 把真实异常吞掉了,所以正常日志永远看不到原因。
//!
//! 本工具改用 `duckdb_open_ext(":memory:", ..., NULL, &error)`——它做的是
//! 同一套 `DBConfig` 构造 + 设 `duckdb_api`,但 `catch` 会把真实异常消息
//! 通过 `error` 出参回传(见 duckdb 源码 main/capi/duckdb-c.cpp)。
//!
//! 用法:
//!   开发机:  cargo build --release --bin duckdb_diag
//!   产物:    target/release/duckdb_diag(.exe)
//!   把这个 exe 拷到生产机,在控制台直接运行,把输出整段发回。

use std::{
    ffi::{CStr, CString},
    os::raw::{c_char, c_void},
    ptr,
};

use duckdb::ffi;

#[cfg(windows)]
unsafe extern "C" {}

fn main() {
    unsafe {
        // 1) 确认静态链接进来的 DuckDB 到底是哪个版本
        let ver = ffi::duckdb_library_version();
        if !ver.is_null() {
            println!(
                "DuckDB library version = {}",
                CStr::from_ptr(ver).to_string_lossy()
            );
        }

        // 2) 复现 DBConfig 构造,但用能回传错误消息的入口
        let path = CString::new(":memory:").unwrap();
        let mut db: ffi::duckdb_database = ptr::null_mut();
        let mut err: *mut c_char = ptr::null_mut();

        let state = ffi::duckdb_open_ext(
            path.as_ptr(),
            &mut db,
            ptr::null_mut(),
            &mut err,
        );

        if state == ffi::DuckDBSuccess {
            println!("结果: OK —— DuckDB 内存库初始化成功,这台机器没问题。");
            ffi::duckdb_close(&mut db);
        } else if !err.is_null() {
            println!(
                "结果: 失败(state={state})。被吞掉的真实异常 = {}",
                CStr::from_ptr(err).to_string_lossy()
            );
            ffi::duckdb_free(err as *mut c_void);
        } else {
            println!("结果: 失败(state={state}),但没有错误消息。");
        }
    }
}
