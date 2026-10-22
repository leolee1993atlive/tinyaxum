use std::ops::Deref;

use sea_orm::{
    DbErr, RuntimeErr,
    sqlx::{
        Error as SqlxError, mysql::MySqlDatabaseError,
        postgres::PgDatabaseError,
    },
};

/// 将数据库错误翻译为中文标题 + 英文正文。
pub fn translate_db_error(err: &DbErr) -> String {
    if let Some(translated) = try_translate_by_code(err) {
        return translated;
    }
    let raw = err.to_string();
    // DbErr::Custom 会包装成 "Custom Error: {msg}"，需要剥离前缀
    raw.strip_prefix("Custom Error: ")
        .unwrap_or(&raw)
        .to_string()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_db_error_unknown() {
        let db_err = DbErr::Custom("Some unknown error".to_string());
        let result = translate_db_error(&db_err);
        assert_eq!(result, "Some unknown error");
    }
}
