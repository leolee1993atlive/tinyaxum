use std::{fs, path::Path};

use bcrypt::{DEFAULT_COST, hash};
use config::Config;
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, Statement};

const USERINFO: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

fn enc(s: &str) -> String {
    utf8_percent_encode(s, USERINFO).to_string()
}

fn build_dsn(
    settings: &Config,
    db_type: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let dsn = match db_type {
        "postgresql" | "postgres" => format!(
            "postgres://{}:{}@{}:{}/{}",
            enc(&settings.get_string("db.username")?),
            enc(&settings.get_string("db.password")?),
            settings.get_string("db.url")?,
            settings.get_int("db.port")?,
            settings.get_string("db.db_name")?,
        ),
        "mysql" => format!(
            "mysql://{}:{}@{}:{}/{}",
            enc(&settings.get_string("db.username")?),
            enc(&settings.get_string("db.password")?),
            settings.get_string("db.url")?,
            settings.get_int("db.port")?,
            settings.get_string("db.db_name")?,
        ),
        "sqlite" => {
            let raw = settings.get_string("db.db_name")?;
            let path = raw.strip_prefix("file:").unwrap_or(&raw);
            if path == ":memory:" {
                "sqlite::memory:".to_string()
            } else {
                if let Some(parent) = Path::new(path).parent()
                    && !parent.as_os_str().is_empty()
                {
                    fs::create_dir_all(parent)?;
                }
                format!("sqlite://{}?mode=rwc", path)
            }
        }
        other => {
            return Err(format!("不支持的 db_type: {other}").into());
        }
    };
    Ok(dsn)
}

fn migration_path(db_type: &str) -> &'static str {
    match db_type {
        "postgresql" | "postgres" => "db/migrations/postgresql/0001_init.sql",
        "mysql" => "db/migrations/mysql/0001_init.sql",
        _ => "db/migrations/sqlite/0001_init.sql",
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Config::builder()
        .add_source(config::File::with_name("config"))
        .build()?;

    let db_type = settings.get_string("db.db_type")?.to_lowercase();
    let dsn = build_dsn(&settings, &db_type)?;

    println!("正在连接数据库 ...");
    let db: DatabaseConnection = Database::connect(&dsn).await?;
    println!("已连接。");

    let sql = fs::read_to_string(migration_path(&db_type))?;
    let backend = db.get_database_backend();
    for stmt in split_sql_statements(&sql) {
        db.execute_unprepared(&stmt).await?;
    }
    println!("迁移 {} 已执行。", migration_path(&db_type));

    let exists = db
        .query_one_raw(Statement::from_string(
            backend,
            "SELECT user_id FROM sys_user WHERE username = 'admin'",
        ))
        .await?
        .is_some();

    if !exists {
        let hashed = hash("admin123", DEFAULT_COST)?;
        let sql = format!(
            "INSERT INTO sys_user
                (username, password, nick_name, active,
                 version, created_by, created_date, last_modified_by, last_modified_date)
             VALUES ('admin', '{hashed}', 'Administrator', 'Y', 1, 0, CURRENT_TIMESTAMP, 0, CURRENT_TIMESTAMP)"
        );
        db.execute_unprepared(&sql).await?;
        println!("默认管理员用户已创建 (密码: admin123)");
        println!("重要: 首次登录后请修改管理员密码!");
    } else {
        println!("管理员用户已存在，跳过。");
    }

    println!("TinyAxum 数据库初始化完成。");
    Ok(())
}

/// Split a SQL file into individual statements by `;`.
/// Strips comment-only segments and blank entries.
fn split_sql_statements(sql: &str) -> Vec<String> {
    sql.split(';')
        .map(|s| s.trim().to_string())
        .filter(|s| {
            !s.is_empty()
                && !s
                    .lines()
                    .all(|l| l.trim().is_empty() || l.trim().starts_with("--"))
        })
        .collect()
}
