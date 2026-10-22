use std::fs;

use bcrypt::{DEFAULT_COST, hash};
use config::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Config::builder()
        .add_source(config::File::with_name("config"))
        .build()?;

    let db_type = settings.get_string("db.db_type")?;

    match db_type.to_lowercase().as_str() {
        "postgresql" | "postgres" => run_postgres(&settings)?,
        "mysql" => run_mysql(&settings)?,
        "sqlite" => run_sqlite(&settings)?,
        other => return Err(format!("不支持的 db_type: {other}").into()),
    }

    println!("数据库初始化完成。");
    Ok(())
}

// ─── PostgreSQL ──────────────────────────────────────────────────────────────

fn run_postgres(settings: &Config) -> Result<(), Box<dyn std::error::Error>> {
    use postgres::{Client, NoTls};

    let host = settings.get_string("db.url")?;
    let port = settings.get_int("db.port")?;
    let username = settings.get_string("db.username")?;
    let password = settings.get_string("db.password")?;
    let db_name = settings.get_string("db.db_name")?;

    let conn_str = format!(
        "host={host} port={port} user={username} password={password} dbname={db_name}"
    );

    println!("正在连接到 {host}:{port}/{db_name} ...");
    let mut client = Client::connect(&conn_str, NoTls)?;
    println!("已连接。");

    let sql = fs::read_to_string("db/migrations/postgresql/0001_init.sql")?;
    client.batch_execute(&sql)?;
    println!("迁移 postgresql/0001_init.sql 已执行。");

    let row = client.query_one(
        "SELECT EXISTS(SELECT 1 FROM sys_user WHERE username = 'admin')",
        &[],
    )?;
    let exists: bool = row.get(0);

    if !exists {
        let hashed = hash("admin123", DEFAULT_COST)?;
        client.execute(
            "INSERT INTO sys_user
                (username, password, nick_name, active,
                 version, created_by, created_date, last_modified_by, last_modified_date)
             VALUES ($1, $2, $3, 'Y', 1, 0, NOW(), 0, NOW())",
            &[&"admin", &hashed, &"Administrator"],
        )?;
        println!("默认管理员用户已创建 (密码: admin123)");
        println!("重要: 首次登录后请修改管理员密码!");
    } else {
        println!("管理员用户已存在，跳过。");
    }

    Ok(())
}

// ─── MySQL ───────────────────────────────────────────────────────────────────

fn run_mysql(settings: &Config) -> Result<(), Box<dyn std::error::Error>> {
    use mysql::prelude::Queryable;

    let host = settings.get_string("db.url")?;
    let port = settings.get_int("db.port")?;
    let username = settings.get_string("db.username")?;
    let password = settings.get_string("db.password")?;
    let db_name = settings.get_string("db.db_name")?;

    println!("正在连接到 {host}:{port}/{db_name} ...");
    let url = format!("mysql://{username}:{password}@{host}:{port}/{db_name}");
    let pool = mysql::Pool::new(url.as_str())?;
    let mut conn = pool.get_conn()?;
    println!("已连接。");

    let sql = fs::read_to_string("db/migrations/mysql/0001_init.sql")?;
    for stmt in split_sql_statements(&sql) {
        conn.query_drop(&stmt)?;
    }
    println!("迁移 mysql/0001_init.sql 已执行。");

    let exists: bool = conn
        .query_first(
            "SELECT EXISTS(SELECT 1 FROM sys_user WHERE username = 'admin')",
        )?
        .unwrap_or(false);

    if !exists {
        let hashed = hash("admin123", DEFAULT_COST)?;
        conn.exec_drop(
            "INSERT INTO sys_user
                (username, password, nick_name, active,
                 version, created_by, created_date, last_modified_by, last_modified_date)
             VALUES (?, ?, ?, 'Y', 1, 0, NOW(), 0, NOW())",
            (&"admin", &hashed, &"Administrator"),
        )?;
        println!("默认管理员用户已创建 (密码: admin123)");
        println!("重要: 首次登录后请修改管理员密码!");
    } else {
        println!("管理员用户已存在，跳过。");
    }

    Ok(())
}

// ─── SQLite ──────────────────────────────────────────────────────────────────

fn run_sqlite(settings: &Config) -> Result<(), Box<dyn std::error::Error>> {
    use rusqlite::params;

    let path = settings.get_string("db.db_name")?;

    println!("正在打开 SQLite 数据库 {path} ...");
    let conn = rusqlite::Connection::open(&path)?;
    println!("已连接。");

    let sql = fs::read_to_string("db/migrations/sqlite/0001_init.sql")?;
    conn.execute_batch(&sql)?;
    println!("迁移 sqlite/0001_init.sql 已执行。");

    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sys_user WHERE username = 'admin')",
        [],
        |row| row.get(0),
    )?;

    if !exists {
        let hashed = hash("admin123", DEFAULT_COST)?;
        conn.execute(
            "INSERT INTO sys_user
                (username, password, nick_name, active,
                 version, created_by, created_date, last_modified_by, last_modified_date)
             VALUES (?1, ?2, ?3, 'Y', 1, 0, CURRENT_TIMESTAMP, 0, CURRENT_TIMESTAMP)",
            params!["admin", hashed, "Administrator"],
        )?;
        println!("默认管理员用户已创建 (密码: admin123)");
        println!("重要: 首次登录后请修改管理员密码!");
    } else {
        println!("管理员用户已存在，跳过。");
    }

    Ok(())
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

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
