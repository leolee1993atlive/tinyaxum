use std::{ops::Deref, sync::LazyLock};

use regex::Regex;
use sea_orm::{
    DbErr, RuntimeErr,
    sqlx::{
        Error as SqlxError, mysql::MySqlDatabaseError,
        postgres::PgDatabaseError,
    },
};

/// 将数据库错误翻译为中文标题 + 英文正文。
pub fn translate_db_error(err: &DbErr) -> String {
    let raw = err.to_string();
    // DbErr::Custom 会包装成 "Custom Error: {msg}"，需要剥离前缀
    let clean_raw = raw.strip_prefix("Custom Error: ").unwrap_or(&raw);
    if let Some(translated) = try_translate_by_code(err) {
        return translated;
    }
    translate_raw(clean_raw)
}

enum DbCode {
    MySql { number: u16, msg: String },
    Pg { state: String, msg: String },
}

fn extract_db_code(err: &DbErr) -> Option<DbCode> {
    // RuntimeErr::SqlxError 内部包装的是 Arc<sqlx::Error>，用 Deref 取
    // &sqlx::Error
    let sqlx_err: &SqlxError = match err {
        DbErr::Exec(RuntimeErr::SqlxError(e))
        | DbErr::Query(RuntimeErr::SqlxError(e))
        | DbErr::Conn(RuntimeErr::SqlxError(e)) => e.deref(),
        _ => return None,
    };
    let SqlxError::Database(db_err) = sqlx_err else {
        return None;
    };

    if let Some(my) = db_err.try_downcast_ref::<MySqlDatabaseError>() {
        return Some(DbCode::MySql {
            number: my.number(),
            msg: my.message().to_string(),
        });
    }
    if let Some(pg) = db_err.try_downcast_ref::<PgDatabaseError>() {
        return Some(DbCode::Pg {
            state: pg.code().to_string(),
            msg: pg.message().to_string(),
        });
    }
    None
}

fn try_translate_by_code(err: &DbErr) -> Option<String> {
    match extract_db_code(err)? {
        DbCode::MySql { number, msg } => {
            let title = match number {
                1146 => "表不存在",
                1054 => "字段不存在",
                1064 => "语法错误",
                1045 => "访问被拒绝",
                1049 => "数据库不存在",
                1062 => "数据重复",
                1452 => "外键约束失败",
                _ => return None,
            };
            Some(format!("{}：{}", title, msg))
        }
        DbCode::Pg { state, msg } => {
            let title = match state.as_str() {
                "42P01" => "表不存在",
                "42703" => "字段不存在",
                "42601" => "语法错误",
                "28P01" => "访问被拒绝",
                "23505" => "数据重复",
                "23503" => "外键约束失败",
                _ => return None,
            };
            Some(format!("{}：{}", title, msg))
        }
    }
}

fn join_title_body(title: &str, body: Option<&str>) -> String {
    match body.map(str::trim).filter(|s| !s.is_empty()) {
        Some(b) => format!("{} - {}", title, b),
        None => title.to_string(),
    }
}

/// 对原始数据库错误字符串做正则翻译。
/// 公共 API，供 sql_query_service 校验路径与其他需要"已 stringify 的错误"翻译的场景使用。
pub fn translate_raw(raw: &str) -> String {
    // 注意：更具体的模式必须放在通用模式之前！
    // CANNOT_COMPARE_RE 和 REF_COLUMN_RE 必须在 BINDER_ERROR_RE 之前
    if let Some(caps) = CANNOT_COMPARE_RE.captures(raw) {
        let title = format!(
            "绑定错误：无法比较类型 {} 和类型 {} 的值",
            &caps[1], &caps[2]
        );
        return join_title_body(&title, caps.get(3).map(|m| m.as_str()));
    }
    if let Some(caps) = REF_COLUMN_RE.captures(raw) {
        let title =
            format!("绑定错误：引用的字段 '{}' 在 FROM 子句中未找到", &caps[1]);
        return join_title_body(&title, caps.get(2).map(|m| m.as_str()));
    }
    if let Some(caps) = BINDER_ERROR_RE.captures(raw) {
        return format!("绑定错误：{}", &caps[1]);
    }
    if let Some(caps) = CATALOG_TABLE_RE.captures(raw) {
        let title = format!("目录错误：表 '{}' 不存在", &caps[1]);
        return join_title_body(&title, caps.get(2).map(|m| m.as_str()));
    }
    if let Some(caps) = CATALOG_COLUMN_RE.captures(raw) {
        let title = format!("目录错误：字段 '{}' 不存在", &caps[1]);
        return join_title_body(&title, caps.get(2).map(|m| m.as_str()));
    }
    if let Some(caps) = CATALOG_ERROR_RE.captures(raw) {
        return format!("目录错误：{}", &caps[1]);
    }
    if let Some(caps) = SYNTAX_ERROR_RE.captures(raw) {
        return format!("语法错误：{}", &caps[1]);
    }
    if let Some(caps) = INVALID_INPUT_RE.captures(raw) {
        return format!("输入错误：{}", &caps[1]);
    }
    if let Some(caps) = CONVERSION_ERROR_RE.captures(raw) {
        return format!("类型转换错误：{}", &caps[1]);
    }
    if let Some(caps) = TABLE_NOT_FOUND_RE.captures(raw) {
        let title = format!("表 '{}' 不存在", &caps[1]);
        return join_title_body(&title, caps.get(2).map(|m| m.as_str()));
    }
    if let Some(caps) = COLUMN_NOT_FOUND_RE.captures(raw) {
        let title = format!("字段 '{}' 不存在", &caps[1]);
        return join_title_body(&title, caps.get(2).map(|m| m.as_str()));
    }
    raw.to_string()
}

static BINDER_ERROR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)^Binder Error:\s*(.+)$").unwrap());
static CANNOT_COMPARE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?s)^Binder Error:\s*Cannot compare values of type (\w+) and type (\w+)(?:\s*-\s*(.*))?$",
    )
    .unwrap()
});
static CATALOG_TABLE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?s)^Catalog Error:\s*Table with name ['"]([^'"]+)['"] does not exist!(?:\s*-\s*(.*))?$"#,
    )
    .unwrap()
});
static CATALOG_COLUMN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?s)^Catalog Error:\s*Column with name ['"]([^'"]+)['"] does not exist!(?:\s*-\s*(.*))?$"#,
    )
    .unwrap()
});
static REF_COLUMN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?s)^Binder Error:\s*Referenced column ['"]([^'"]+)['"] not found in FROM clause!?(?:\s*-\s*(.*))?$"#,
    )
    .unwrap()
});
static CATALOG_ERROR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)^Catalog Error:\s*(.+)$").unwrap());
static SYNTAX_ERROR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)^Syntax Error:\s*(.+)$").unwrap());
static INVALID_INPUT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)^Invalid Input Error:\s*(.+)$").unwrap());
static CONVERSION_ERROR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)^Conversion Error:\s*(.+)$").unwrap());
static TABLE_NOT_FOUND_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?s)^[Tt]able with name ['"]([^'"]+)['"] does not exist!?(?:\s*-\s*(.*))?$"#,
    )
    .unwrap()
});
static COLUMN_NOT_FOUND_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?s)^[Cc]olumn with name ['"]([^'"]+)['"] does not exist!?(?:\s*-\s*(.*))?$"#,
    )
    .unwrap()
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_cannot_compare_error() {
        let input = "Binder Error: Cannot compare values of type TIMESTAMP and type VARCHAR - an explicit cast is required LINE 21: AND t1.in_date >= t2.dept_date";
        let result = translate_raw(input);
        assert_eq!(
            result,
            "绑定错误：无法比较类型 TIMESTAMP 和类型 VARCHAR 的值 - an explicit cast is required LINE 21: AND t1.in_date >= t2.dept_date"
        );
    }

    #[test]
    fn test_translate_catalog_table() {
        let input = "Catalog Error: Table with name 'users' does not exist!";
        let result = translate_raw(input);
        assert_eq!(result, "目录错误：表 'users' 不存在");
    }

    #[test]
    fn test_translate_syntax_error() {
        let input = "Syntax Error: syntax error at or near \"FROM\"";
        let result = translate_raw(input);
        assert!(result.starts_with("语法错误："));
        assert!(result.contains("FROM"));
    }

    #[test]
    fn test_translate_unknown_error() {
        let input = "Some random error that doesn't match any pattern";
        let result = translate_raw(input);
        assert_eq!(result, input);
    }

    #[test]
    fn test_translate_table_not_found_without_prefix() {
        let input = "Table with name 'orders' does not exist!";
        let result = translate_raw(input);
        assert_eq!(result, "表 'orders' 不存在");
    }

    #[test]
    fn test_translate_generic_binder_error() {
        let input = "Binder Error: some other binder issue";
        let result = translate_raw(input);
        assert_eq!(result, "绑定错误：some other binder issue");
    }

    #[test]
    fn test_translate_ref_column_error() {
        let input =
            "Binder Error: Referenced column 'foo' not found in FROM clause!";
        let result = translate_raw(input);
        assert_eq!(result, "绑定错误：引用的字段 'foo' 在 FROM 子句中未找到");
    }

    #[test]
    fn test_translate_db_error_public_api() {
        // Test with a generic DbErr::Custom that matches regex
        // Note: DbErr::Custom wraps message as "Custom Error: {msg}"
        let db_err = DbErr::Custom(
            "Syntax Error: syntax error at or near \"FROM\"".to_string(),
        );
        let result = translate_db_error(&db_err);
        assert_eq!(result, "语法错误：syntax error at or near \"FROM\"");
    }

    #[test]
    fn test_translate_db_error_unknown() {
        let db_err = DbErr::Custom("Some unknown error".to_string());
        let result = translate_db_error(&db_err);
        assert_eq!(result, "Some unknown error");
    }

    #[test]
    fn test_translate_cannot_compare_multiline() {
        let input = "Binder Error: Cannot compare values of type TIMESTAMP and type VARCHAR\n- an explicit cast is required LINE 21: AND t1.in_date >= t2.dept_date";
        let result = translate_raw(input);
        assert!(
            result
                .starts_with("绑定错误：无法比较类型 TIMESTAMP 和类型 VARCHAR")
        );
        assert!(result.contains("an explicit cast is required LINE 21"));
    }

    #[test]
    fn test_translate_catalog_table_with_body() {
        let input = "Catalog Error: Table with name 'users' does not exist! - Did you mean 'user'?";
        let result = translate_raw(input);
        assert_eq!(
            result,
            "目录错误：表 'users' 不存在 - Did you mean 'user'?"
        );
    }
}
