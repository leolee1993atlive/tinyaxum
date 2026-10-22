use anyhow::anyhow;
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};

use crate::config::config::DbConfig;

/// DSN 的 userinfo 段需要转义的字符集。保守起见：非字母数字一律转义，
/// 只放行 URL 中无歧义的 `-` `.` `_` `~`（RFC 3986 unreserved）。
const USERINFO: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

fn enc(s: &str) -> String {
    utf8_percent_encode(s, USERINFO).to_string()
}

/// 由 [db] 配置拼出 sea-orm 用的 DSN（含 TCP keepalive 参数）。
///
/// 生产启动路径（`app::init_app`）与安装器的 `--check-config` 共用此函数，
/// 保证向导里测通的配置与服务实际使用的连接串完全一致。
pub fn build_db_dsn(db: &DbConfig) -> anyhow::Result<String> {
    let dsn = match db.db_type.as_str() {
        "mysql" | "postgres" => format!(
            "{}://{}:{}@{}:{}/{}",
            db.db_type,
            enc(&db.username),
            enc(&db.password),
            db.url,
            db.port,
            db.db_name,
        ),
        "sqlite" => {
            // SQLite 使用 db.db_name 作为数据库路径，
            // 支持 "data.db"、"file:data.db"、":memory:" 等形式
            let path = db.db_name.trim();
            if path.is_empty() {
                "file:data.db".to_string()
            } else if path == ":memory:" || path.starts_with("file:") {
                path.to_string()
            } else {
                format!("file:{}", path)
            }
        }
        unsupported => {
            return Err(anyhow!("不支持的数据库类型: {}", unsupported));
        }
    };

    // 为 DSN 添加 TCP keepalive 参数，防止连接因空闲被数据库或防火墙关闭
    let dsn = match db.db_type.as_str() {
        "postgres" | "postgresql" => format!(
            "{}?keepalives=1&keepalives_idle=30&keepalives_interval=10&keepalives_count=5&connect_timeout=10&statement_timeout=30000",
            dsn
        ),
        "mysql" => {
            // MySQL 连接参数使用不同格式
            if dsn.contains('?') {
                format!("{}&tcp_keepalive=30000", dsn)
            } else {
                format!("{}?tcp_keepalive=30000", dsn)
            }
        }
        _ => dsn,
    };

    Ok(dsn)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db(db_type: &str, password: &str) -> DbConfig {
        DbConfig {
            db_type: db_type.to_string(),
            url: "127.0.0.1".to_string(),
            port: 3306,
            db_name: "tinyaxum".to_string(),
            username: "root".to_string(),
            password: password.to_string(),
        }
    }

    #[test]
    fn mysql_dsn_has_keepalive() {
        let dsn = build_db_dsn(&db("mysql", "root")).unwrap();
        assert_eq!(
            dsn,
            "mysql://root:root@127.0.0.1:3306/tinyaxum?tcp_keepalive=30000"
        );
    }

    #[test]
    fn postgres_dsn_has_keepalive() {
        let dsn = build_db_dsn(&db("postgres", "pw")).unwrap();
        assert!(dsn.starts_with("postgres://root:pw@127.0.0.1:3306/tinyaxum?"));
        assert!(dsn.contains("keepalives=1"));
    }

    #[test]
    fn password_special_chars_are_percent_encoded() {
        let dsn = build_db_dsn(&db("mysql", "P@ssw0rd:/#?")).unwrap();
        assert!(
            dsn.starts_with("mysql://root:P%40ssw0rd%3A%2F%23%3F@127.0.0.1:"),
            "密码未转义, 实际: {dsn}"
        );
    }

    #[test]
    fn sqlite_dsn_variants() {
        let mut c = db("sqlite", "");
        c.db_name = "data.db".to_string();
        assert_eq!(build_db_dsn(&c).unwrap(), "file:data.db");

        c.db_name = ":memory:".to_string();
        assert_eq!(build_db_dsn(&c).unwrap(), ":memory:");

        c.db_name = "file:other.db".to_string();
        assert_eq!(build_db_dsn(&c).unwrap(), "file:other.db");

        c.db_name = "".to_string();
        assert_eq!(build_db_dsn(&c).unwrap(), "file:data.db");
    }

    #[test]
    fn unsupported_db_type_errors() {
        assert!(build_db_dsn(&db("oracle", "x")).is_err());
    }
}
