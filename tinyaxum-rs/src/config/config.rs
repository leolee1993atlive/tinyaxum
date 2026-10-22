use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub db: DbConfig,
    pub redis: RedisConfig,
    pub jwt: JwtConfig,
    pub duckdb: DuckdbConfig,
    pub encryption: EncryptionConfig,
    #[serde(default)]
    pub log: LogConfig,
    #[serde(default)]
    pub pool: PoolConfig,
    #[serde(default)]
    pub dashboard_cache: DashboardCacheConfig,
    #[serde(default)]
    pub cors: super::cors::CorsConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: i32,
    #[serde(default)]
    pub static_dir: Option<String>,
}

//
// 支持的 db_type: "mysql" | "postgres" | "sqlite"
// - mysql/postgres: 使用 url/port/db_name/username/password
// - sqlite: 仅使用 db_name 作为数据库文件路径，
//   例如 "data.db"、"file:data.db" 或 ":memory:"（内存模式）。
//   其余字段（url/port/username/password）可留空或省略。
#[derive(Debug, Deserialize)]
pub struct DbConfig {
    pub db_type: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub port: i32,
    pub db_name: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub port: i32,
    pub password: Option<String>,
    pub db: i32,
}

#[derive(Debug, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    /// Access token 有效期（秒）
    pub access_ttl: i64,
    /// Refresh token 有效期（秒）。
    ///
    /// **安全取舍说明**：Task 3 移除了 Redis token 存储，refresh token
    /// 不再具备服务端撤销能力。泄露的 refresh token 在整个 TTL 内可被
    /// 反复使用来换取新的 access token。建议将此值收紧至 7-14 天（
    /// 604800-1209600 秒），并在 config.toml 中显式配置。
    pub refresh_ttl: i64,
}

#[derive(Debug, Deserialize)]
pub struct DuckdbConfig {
    /// 传输任务每表 DuckDB 文件根目录
    #[serde(default = "default_data_dir")]
    pub data_dir: String,
}

fn default_data_dir() -> String {
    "data".to_string()
}

#[derive(Debug, Deserialize)]
pub struct EncryptionConfig {
    pub key: String,
}

#[derive(Debug, Deserialize)]
pub struct LogConfig {
    #[serde(default = "default_log_dir")]
    pub dir: String,
    #[serde(default = "default_log_prefix")]
    pub prefix: String,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            dir: default_log_dir(),
            prefix: default_log_prefix(),
        }
    }
}

fn default_log_dir() -> String {
    "logs".to_string()
}

fn default_log_prefix() -> String {
    "hdata-analysis".to_string()
}

#[derive(Debug, Deserialize)]
pub struct PoolConfig {
    #[serde(default = "default_db_max_conn")]
    pub db_max_connections: u32,
    #[serde(default = "default_db_min_conn")]
    pub db_min_connections: u32,
    #[serde(default = "default_redis_max_conn")]
    pub redis_max_connections: usize,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            db_max_connections: default_db_max_conn(),
            db_min_connections: default_db_min_conn(),
            redis_max_connections: default_redis_max_conn(),
        }
    }
}

fn default_db_max_conn() -> u32 {
    10
}
fn default_db_min_conn() -> u32 {
    5
}
fn default_redis_max_conn() -> usize {
    16
}

#[derive(Debug, Deserialize)]
pub struct DashboardCacheConfig {
    #[serde(default = "default_dash_ttl_days")]
    pub ttl_days: u64,
    #[serde(default = "default_dash_max_rows")]
    pub max_rows: i64,
}

impl Default for DashboardCacheConfig {
    fn default() -> Self {
        Self {
            ttl_days: default_dash_ttl_days(),
            max_rows: default_dash_max_rows(),
        }
    }
}

fn default_dash_ttl_days() -> u64 {
    30
}

fn default_dash_max_rows() -> i64 {
    2000
}

#[cfg(test)]
mod tests {
    use ::config::{Config, File, FileFormat};

    use super::*;

    const MINIMAL: &str = r#"
[server]
host = "localhost"
port = 3000

[db]
db_type = "sqlite"
db_name = "data.db"

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
key = "0123456789abcdef0123456789abcdef"
"#;

    fn parse(toml: &str) -> AppConfig {
        Config::builder()
            .add_source(File::from_str(toml, FileFormat::Toml))
            .build()
            .expect("build")
            .try_deserialize()
            .expect("deserialize")
    }

    #[test]
    fn pool_default_matches_serde_field_defaults() {
        let p = PoolConfig::default();
        assert_eq!(p.db_max_connections, 10);
        assert_eq!(p.db_min_connections, 5);
        assert_eq!(p.redis_max_connections, 16);
    }

    #[test]
    fn missing_pool_section_still_gets_real_defaults() {
        let cfg = parse(MINIMAL);
        assert_eq!(cfg.pool.db_max_connections, 10);
        assert_eq!(cfg.pool.db_min_connections, 5);
        assert_eq!(cfg.pool.redis_max_connections, 16);
    }
}
