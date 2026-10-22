use std::time::Duration;

// use anyhow::{Context, Result, anyhow};
use crate::biz::datasource::model::data_source::DataSource;

pub async fn test_data_source(data_source: &DataSource) -> (bool, String) {
    let ds = data_source.clone();
    let result = tokio::time::timeout(
        Duration::from_secs(10),
        tokio::task::spawn_blocking(move || test_data_source_sync(&ds)),
    )
    .await;

    match result {
        Ok(Ok(res)) => res,
        Ok(Err(e)) => {
            tracing::error!(error = %e, "测试数据源任务异常");
            (false, "测试数据源任务执行失败".to_string())
        }
        Err(_) => (false, "测试数据源超时".to_string()),
    }
}

pub fn test_data_source_sync(data_source: &DataSource) -> (bool, String) {
    match data_source.db_type.to_lowercase().as_str() {
        "mysql" => test_mysql(data_source),
        "postgresql" | "postgres" => test_postgresql(data_source),
        db_type => (false, format!("不支持的数据库类型: {}", db_type)),
    }
}

fn test_mysql(data_source: &DataSource) -> (bool, String) {
    let opts = mysql::OptsBuilder::new()
        .ip_or_hostname(Some(data_source.db_host.clone()))
        .tcp_port(data_source.db_port)
        .user(Some(data_source.db_username.clone()))
        .pass(Some(data_source.db_password.clone()))
        .db_name(Some(data_source.db_name.clone()));

    match mysql::Conn::new(opts) {
        Ok(_) => {
            tracing::info!(
                db_name = %data_source.db_name,
                "成功创建 MySQL 连接"
            );
            (true, "Connection created successfully".to_string())
        }
        Err(e) => {
            tracing::error!(
                error = %e,
                db_name = %data_source.db_name,
                "创建 MySQL 连接失败"
            );
            (false, "Connection creation failed".to_string())
        }
    }
}

fn test_postgresql(data_source: &DataSource) -> (bool, String) {
    let mut cfg = postgres::Config::new();
    cfg.host(&data_source.db_host)
        .port(data_source.db_port)
        .user(&data_source.db_username)
        .password(&data_source.db_password)
        .dbname(&data_source.db_name);

    match cfg.connect(postgres::NoTls) {
        Ok(_) => {
            tracing::info!(
                db_name = %data_source.db_name,
                "成功创建 PostgreSQL 连接"
            );
            (true, "Connection created successfully".to_string())
        }
        Err(e) => {
            tracing::error!(
                error = %e,
                db_name = %data_source.db_name,
                "创建 PostgreSQL 连接失败"
            );
            (false, "Connection creation failed".to_string())
        }
    }
}
