use std::{path::Path, time::Duration};

use ::config::{Config, File};
use sea_orm::{ConnectOptions, Database};

use crate::config::{
    config::AppConfig,
    dsn::{build_db_dsn, build_redis_url},
};

/// `crypto_util` 把 encryption.key 当 AES-256 的原始密钥字节用，必须 32 字节。
/// 启动期只判 `is_empty()`（见 `app.rs`），长度错了要到第一次加密数据源密码
/// 时才炸 —— 所以在这里提前拦下。
fn check_encryption_key(config: &AppConfig) -> anyhow::Result<()> {
    let len = config.encryption.key.len();
    if len != 32 {
        anyhow::bail!(
            "[encryption] key 必须恰好 32 个字符(当前 {len} 个)。\
             它被当作 AES-256 的原始密钥使用，长度不符会导致保存数据源时失败。"
        );
    }
    Ok(())
}

/// 校验一份 config.toml：能否解析、密钥长度、DB 与 Redis 是否连得上。
///
/// 供安装向导在「下一步」时静默调用。成功返回 Ok，失败返回的错误由
/// `main` 打到 stderr 并以非 0 退出。
pub async fn check_config(path: &Path) -> anyhow::Result<()> {
    let config: AppConfig = Config::builder()
        .add_source(File::from(path))
        .build()
        .map_err(|e| anyhow::anyhow!("读取配置失败: {e}"))?
        .try_deserialize()
        .map_err(|e| anyhow::anyhow!("解析配置失败: {e}"))?;

    check_encryption_key(&config)?;

    let dsn = build_db_dsn(&config.db)?;
    let mut opt = ConnectOptions::new(&dsn);
    opt.max_connections(1)
        .min_connections(1)
        .connect_timeout(Duration::from_secs(10))
        .acquire_timeout(Duration::from_secs(10))
        .sqlx_logging(false);
    Database::connect(opt).await.map_err(|e| {
        anyhow::anyhow!(
            "数据库连接失败 ({} {}:{}/{}): {e}",
            config.db.db_type,
            config.db.url,
            config.db.port,
            config.db.db_name
        )
    })?;

    let redis_url = build_redis_url(&config.redis);
    let mut redis_cfg = deadpool_redis::Config::from_url(redis_url);
    redis_cfg.pool = Some(deadpool_redis::PoolConfig {
        max_size: 1,
        timeouts: deadpool_redis::Timeouts {
            wait: Some(Duration::from_secs(5)),
            create: Some(Duration::from_secs(5)),
            recycle: Some(Duration::from_secs(5)),
        },
        ..Default::default()
    });
    let pool = redis_cfg
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .map_err(|e| anyhow::anyhow!("创建 Redis 连接池失败: {e}"))?;
    let mut conn = pool.get().await.map_err(|e| {
        anyhow::anyhow!(
            "Redis 连接失败 ({}:{} db={}): {e}",
            config.redis.url,
            config.redis.port,
            config.redis.db
        )
    })?;
    deadpool_redis::redis::cmd("PING")
        .query_async::<String>(&mut conn)
        .await
        .map_err(|e| anyhow::anyhow!("Redis PING 失败: {e}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use ::config::{Config, File, FileFormat};

    use super::*;

    fn cfg_with_key(key: &str) -> AppConfig {
        let text = format!(
            r#"
[server]
host = "localhost"
port = 3000

[db]
db_type = "sqlite"
db_name = ":memory:"

[redis]
url = "localhost"
port = 6379
db = 1

[jwt]
secret = "s"
access_ttl = 3600
refresh_ttl = 1800000

[duckdb]
data_dir = "data"

[encryption]
key = "{key}"
"#
        );
        Config::builder()
            .add_source(File::from_str(&text, FileFormat::Toml))
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap()
    }

    #[test]
    fn accepts_32_char_key() {
        let c = cfg_with_key("0123456789abcdef0123456789abcdef");
        assert!(check_encryption_key(&c).is_ok());
    }

    #[test]
    fn rejects_short_key() {
        let c = cfg_with_key("tooshort");
        let err = check_encryption_key(&c).unwrap_err().to_string();
        assert!(err.contains("32"), "错误信息应点明 32, 实际: {err}");
    }

    #[test]
    fn rejects_64_char_key() {
        let c = cfg_with_key(
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        );
        assert!(check_encryption_key(&c).is_err());
    }

    #[test]
    fn rejects_empty_key() {
        let c = cfg_with_key("");
        assert!(check_encryption_key(&c).is_err());
    }
}
