// DuckDB 错误路径回归：语法错误 / Binder 错误必须以 Err 返回，不允许
// C++ 异常穿透 FFI 导致进程 abort。
//
// 背景：Windows 下用默认 duckdb-bundled（cc 自编译）构建时，DuckDB 的
// Parser/Binder 异常会以 foreign exception 形式穿透 Rust 帧，触发
// "fatal runtime error: Rust cannot catch foreign exceptions, aborting"，
// 任何一条坏 SQL 都能打死整个进程（validate 边输入边校验时尤其致命）。
// 官方预编译 libduckdb（build-windows.ps1 / DUCKDB_LIB_DIR + --no-default-features）
// 没有此问题。本测试在两种链路下都应通过；若 bundled 链路下 abort，
// 说明又退化到了自编译构建。
#[test]
fn parser_error_returns_err_not_abort() {
    let conn = duckdb::Connection::open_in_memory().unwrap();
    // 悬空 FROM：validate 剥掉片段引用后曾产生的形状
    let r = conn.execute("EXPLAIN SELECT count(1) AS count1\nFROM ", []);
    assert!(r.is_err(), "悬空 FROM 应返回错误: {:?}", r);
}

#[test]
fn plain_syntax_error_returns_err() {
    let conn = duckdb::Connection::open_in_memory().unwrap();
    let r = conn.execute("EXPLAIN SELEC 1", []);
    assert!(r.is_err());
}

#[test]
fn binder_error_missing_table_returns_err() {
    let conn = duckdb::Connection::open_in_memory().unwrap();
    let r = conn.execute("EXPLAIN SELECT count(1) FROM vmq_inpatient_info", []);
    assert!(r.is_err(), "缺表应返回 Binder 错误: {:?}", r);
}

#[test]
fn valid_explain_ok() {
    let conn = duckdb::Connection::open_in_memory().unwrap();
    let r =
        conn.execute("EXPLAIN SELECT count(1) AS count1\nFROM (SELECT 1)", []);
    assert!(r.is_ok(), "合法 SQL 不应报错: {:?}", r);
}
