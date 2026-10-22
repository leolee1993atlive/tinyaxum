use serde::Deserialize;

/// 安装向导收集到的用户输入。由 Inno 写成 `{tmp}\answers.toml` 传入。
///
/// 走文件而非命令行参数：其中含数据库密码，命令行参数在 Windows 上
/// 任何进程都能从进程列表读到。
#[derive(Debug, Deserialize)]
pub struct Answers {
    pub server: AnswersServer,
    pub db: AnswersDb,
    pub redis: AnswersRedis,
}

#[derive(Debug, Deserialize)]
pub struct AnswersServer {
    pub port: i32,
}

#[derive(Debug, Deserialize)]
pub struct AnswersDb {
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
pub struct AnswersRedis {
    pub url: String,
    pub port: i32,
    #[serde(default)]
    pub password: String,
    pub db: i32,
}

impl Answers {
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let text = std::fs::read_to_string(path).map_err(|e| {
            anyhow::anyhow!("读取 answers 文件失败 ({}): {e}", path.display())
        })?;
        toml::from_str(&text)
            .map_err(|e| anyhow::anyhow!("解析 answers 文件失败: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
[server]
port = 3000

[db]
db_type = "mysql"
url = "127.0.0.1"
port = 3306
db_name = "bl-hdata"
username = "root"
password = "P@ss"

[redis]
url = "localhost"
port = 6379
password = ""
db = 1
"#;

    #[test]
    fn parses_full_answers() {
        let a: Answers = toml::from_str(SAMPLE).unwrap();
        assert_eq!(a.server.port, 3000);
        assert_eq!(a.db.db_type, "mysql");
        assert_eq!(a.db.password, "P@ss");
        assert_eq!(a.redis.db, 1);
    }

    #[test]
    fn sqlite_answers_omit_network_fields() {
        let a: Answers = toml::from_str(
            r#"
[server]
port = 3000

[db]
db_type = "sqlite"
db_name = "data.db"

[redis]
url = "localhost"
port = 6379
db = 1
"#,
        )
        .unwrap();
        assert_eq!(a.db.db_type, "sqlite");
        assert_eq!(a.db.url, "");
        assert_eq!(a.db.port, 0);
        assert_eq!(a.redis.password, "");
    }
}
