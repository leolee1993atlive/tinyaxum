//! 从外部 MySQL / PostgreSQL 数据源读取表与列元数据（用于刷新缓存）

use anyhow::{Context, Result};
use mysql::prelude::Queryable;

use crate::{
    biz::datasource::model::data_source::DataSource,
    common::model::field_semantic_type::FieldSemanticType,
};

/// information_schema.COLUMNS 行:
/// (表名, 列名, 列注释, 列类型, 数据类型, 列键, 序号)
type ColumnRow = (String, String, Option<String>, String, String, String, i32);

#[derive(Debug, Clone)]
pub struct IntrospectedColumn {
    pub column_name: String,
    pub comment_label: Option<String>,
    pub source_physical_type: Option<String>,
    pub field_type: String,
    pub is_primary_key: bool,
    pub ordinal_position: i32,
}

#[derive(Debug, Clone)]
pub struct IntrospectedTable {
    pub table_name: String,
    pub comment_label: Option<String>,
    pub columns: Vec<IntrospectedColumn>,
}

pub fn introspect_mysql(ds: &DataSource) -> Result<Vec<IntrospectedTable>> {
    let opts = mysql::OptsBuilder::new()
        .ip_or_hostname(Some(ds.db_host.clone()))
        .tcp_port(ds.db_port)
        .user(Some(ds.db_username.clone()))
        .pass(Some(ds.db_password.clone()))
        .db_name(Some(ds.db_name.clone()));

    let mut conn = mysql::Conn::new(opts).context("连接 MySQL 数据源失败")?;
    let schema = ds.db_name.clone();

    let tables: Vec<(String, Option<String>)> = conn.exec_map(
        "SELECT TABLE_NAME, TABLE_COMMENT FROM information_schema.TABLES \
         WHERE TABLE_SCHEMA = ? AND TABLE_TYPE = 'BASE TABLE' ORDER BY TABLE_NAME",
        (schema.as_str(),),
        |(name, comment): (String, Option<String>)| (name, comment),
    )?;

    let all_columns: Vec<ColumnRow> = conn.exec_map(
        "SELECT TABLE_NAME, COLUMN_NAME, COLUMN_COMMENT, COLUMN_TYPE, DATA_TYPE, COLUMN_KEY, ORDINAL_POSITION \
         FROM information_schema.COLUMNS \
         WHERE TABLE_SCHEMA = ? \
         ORDER BY TABLE_NAME, ORDINAL_POSITION",
        (schema.as_str(),),
        |(tbl, col, col_comment, col_type, data_type, col_key, ordinal): ColumnRow| {
            (tbl, col, col_comment, col_type, data_type, col_key, ordinal)
        },
    )?;

    let mut cols_by_table: std::collections::HashMap<
        String,
        Vec<IntrospectedColumn>,
    > = std::collections::HashMap::new();
    for (
        tbl,
        column_name,
        comment_label,
        col_type,
        data_type,
        col_key,
        ordinal,
    ) in all_columns
    {
        cols_by_table
            .entry(tbl)
            .or_default()
            .push(IntrospectedColumn {
                column_name,
                comment_label: empty_to_none(comment_label),
                source_physical_type: Some(col_type),
                field_type: FieldSemanticType::from_mysql_data_type(&data_type)
                    .as_str()
                    .to_string(),
                is_primary_key: col_key.eq_ignore_ascii_case("PRI"),
                ordinal_position: ordinal,
            });
    }

    let mut out = Vec::with_capacity(tables.len());
    for (table_name, table_comment) in tables {
        let columns = cols_by_table.remove(&table_name).unwrap_or_default();
        out.push(IntrospectedTable {
            table_name,
            comment_label: empty_to_none(table_comment),
            columns,
        });
    }

    Ok(out)
}

pub fn introspect_postgresql(
    ds: &DataSource,
) -> Result<Vec<IntrospectedTable>> {
    use postgres::{Config, NoTls};

    let mut cfg = Config::new();
    cfg.host(&ds.db_host)
        .port(ds.db_port)
        .user(&ds.db_username)
        .password(&ds.db_password)
        .dbname(&ds.db_name)
        .connect_timeout(std::time::Duration::from_secs(10));

    tracing::info!(
        db_host = %ds.db_host,
        db_port = ds.db_port,
        db_name = %ds.db_name,
        "PostgreSQL 元数据采集连接中"
    );

    let mut client =
        cfg.connect(NoTls).context("连接 PostgreSQL 数据源失败")?;

    // 显式 schema：数据源配置优先，缺省 public
    let schema = ds.db_schema.as_deref().unwrap_or("public").to_string();

    tracing::info!(schema = %schema, "PostgreSQL 元数据采集 schema");

    let tables: Vec<String> = client
        .query(
            "SELECT tablename FROM pg_catalog.pg_tables \
             WHERE schemaname = $1 ORDER BY tablename",
            &[&schema],
        )
        .context("读取 PostgreSQL 表列表失败")?
        .into_iter()
        .map(|row| row.get::<_, String>(0))
        .collect();

    let table_comments: std::collections::HashMap<String, Option<String>> =
        client
            .query(
                "SELECT c.relname::text, obj_description(c.oid)::text \
                 FROM pg_catalog.pg_class c \
                 JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace \
                 WHERE n.nspname = $1 AND c.relkind = 'r'",
                &[&schema],
            )
            .context("读取 PostgreSQL 表注释失败")?
            .into_iter()
            .map(|row| {
                let name: String = row.get(0);
                let comment: Option<String> = row.get(1);
                (name, comment)
            })
            .collect();

    let all_columns = client
        .query(
            "SELECT
                 c.relname::text AS table_name,
                 a.attname::text,
                 pg_catalog.format_type(a.atttypid, a.atttypmod),
                 pg_catalog.col_description(a.attrelid, a.attnum),
                 EXISTS(
                     SELECT 1 FROM pg_constraint con
                     WHERE con.conrelid = c.oid
                       AND con.contype = 'p'
                       AND a.attnum = ANY(con.conkey)
                 ) AS is_pk,
                 a.attnum::int AS ordinal_position
             FROM pg_attribute a
             JOIN pg_class c ON c.oid = a.attrelid
             JOIN pg_namespace n ON n.oid = c.relnamespace
             WHERE n.nspname = $1
               AND a.attnum > 0 AND NOT a.attisdropped
               AND c.relkind = 'r'
             ORDER BY c.relname, a.attnum",
            &[&schema],
        )
        .context("读取 PostgreSQL 全部列失败")?;

    let mut cols_by_table: std::collections::HashMap<
        String,
        Vec<IntrospectedColumn>,
    > = std::collections::HashMap::new();
    for row in &all_columns {
        let table_name: String = row.get(0);
        let column_name: String = row.get(1);
        let fmt_type: String = row.get(2);
        let col_comment: Option<String> = row.get(3);
        let is_primary_key: bool = row.get(4);
        let ordinal_position: i32 = row.get(5);
        let data_type = fmt_type
            .split_whitespace()
            .next()
            .unwrap_or(fmt_type.as_str())
            .to_string();
        cols_by_table
            .entry(table_name)
            .or_default()
            .push(IntrospectedColumn {
                column_name,
                comment_label: empty_to_none(col_comment),
                source_physical_type: Some(fmt_type),
                field_type: FieldSemanticType::from_pg_data_type(&data_type)
                    .as_str()
                    .to_string(),
                is_primary_key,
                ordinal_position,
            });
    }

    let mut out = Vec::with_capacity(tables.len());
    for table_name in tables {
        let table_comment = table_comments.get(&table_name).cloned().flatten();
        let columns = cols_by_table.remove(&table_name).unwrap_or_default();
        out.push(IntrospectedTable {
            table_name,
            comment_label: empty_to_none(table_comment),
            columns,
        });
    }

    Ok(out)
}

fn empty_to_none(s: Option<String>) -> Option<String> {
    s.filter(|t| !t.trim().is_empty())
}
