use std::path::Path;

use rand::{RngExt, distr::Alphanumeric};
use toml_edit::{DocumentMut, value};

use crate::config::answers::Answers;

const TEMPLATE: &str = include_str!("../../templates/config.toml.tmpl");

/// `encryption.key` 必须恰好 32 字符：`crypto_util` 把它当 AES-256 的
/// 原始密钥字节用。改这个常量前先读 `util/crypto_util.rs`。
const ENCRYPTION_KEY_LEN: usize = 32;
const JWT_SECRET_LEN: usize = 64;

fn random_ascii(len: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

/// 按向导输入生成或更新 `config.toml`。
///
/// - 文件不存在：由内嵌模板生成完整配置，密钥用 CSPRNG 现场生成。
/// - 文件已存在：只做外科更新。
pub fn write_config(path: &Path, answers: &Answers) -> anyhow::Result<()> {
    let text = if path.exists() {
        update_existing(&std::fs::read_to_string(path)?, answers)?
    } else {
        render_fresh(answers)?
    };
    std::fs::write(path, text)
        .map_err(|e| anyhow::anyhow!("写入配置失败 ({}): {e}", path.display()))
}

/// 单次扫描模板做 `@NAME@` 替换。
///
/// 不能用链式 `.replace()`：那样先填入的用户值会成为后续 replace 的输入，
/// 值里若含另一个占位符的字面量就会被二次改写。例如数据库密码正好是
/// `@ENCRYPTION_KEY@` 时，链式替换会把它连同真正的占位符一起换成同一个
/// 随机密钥 —— 密码被悄悄改写，且与加密密钥撞成同一个值。
fn render_template(tmpl: &str, vars: &[(&str, String)]) -> String {
    let bytes = tmpl.as_bytes();
    let mut out = String::with_capacity(tmpl.len() + 256);
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'@' {
            let mut j = i + 1;
            while j < bytes.len()
                && (bytes[j].is_ascii_uppercase()
                    || bytes[j].is_ascii_digit()
                    || bytes[j] == b'_')
            {
                j += 1;
            }
            // j > i + 1 排除空名 "@@"
            if j > i + 1 && j < bytes.len() && bytes[j] == b'@' {
                let name = &tmpl[i + 1..j];
                if let Some((_, v)) = vars.iter().find(|(k, _)| *k == name) {
                    out.push_str(v);
                    i = j + 1;
                    continue;
                }
            }
        }
        // 非占位符：原样搬运一个字符(模板含中文注释,不能按字节推进)
        let ch = tmpl[i..].chars().next().expect("i 在字符边界上");
        out.push(ch);
        i += ch.len_utf8();
    }

    out
}

fn render_fresh(answers: &Answers) -> anyhow::Result<String> {
    // 模板里的字符串占位符带引号，值必须先按 TOML 基本字符串转义，
    // 否则密码里的 " 或 \ 会写出一份语法错误的 config.toml。
    let esc = |s: &str| s.replace('\\', r"\\").replace('"', r#"\""#);

    let vars = [
        ("PORT", answers.server.port.to_string()),
        ("DB_TYPE", esc(&answers.db.db_type)),
        ("DB_URL", esc(&answers.db.url)),
        ("DB_PORT", answers.db.port.to_string()),
        ("DB_NAME", esc(&answers.db.db_name)),
        ("DB_USERNAME", esc(&answers.db.username)),
        ("DB_PASSWORD", esc(&answers.db.password)),
        ("JWT_SECRET", random_ascii(JWT_SECRET_LEN)),
        ("ENCRYPTION_KEY", random_ascii(ENCRYPTION_KEY_LEN)),
    ];

    let text = render_template(TEMPLATE, &vars);

    // 语法自检：转义写漏时(密码里带 " 或 \)在这里就炸,别写出一份
    // 语法坏掉的 config.toml 让服务起不来。字段层面的正确性由
    // `fresh_config_is_valid_appconfig` 测试用真的 AppConfig 反序列化来守。
    text.parse::<DocumentMut>().map_err(|e| {
        anyhow::anyhow!("生成的 config.toml 不是合法 TOML: {e}")
    })?;

    Ok(text)
}

/// 升级路径：只改向导收集的那几个键，其余一律不动。
///
/// **绝不能碰 `[jwt].secret` 与 `[encryption].key`** —— 重写 encryption.key
/// 会让库里所有已加密的数据永久解不开。
fn update_existing(
    existing: &str,
    answers: &Answers,
) -> anyhow::Result<String> {
    let mut doc = existing
        .parse::<DocumentMut>()
        .map_err(|e| anyhow::anyhow!("现有 config.toml 不是合法 TOML: {e}"))?;

    doc["server"]["port"] = value(answers.server.port as i64);

    doc["db"]["db_type"] = value(answers.db.db_type.as_str());
    doc["db"]["url"] = value(answers.db.url.as_str());
    doc["db"]["port"] = value(answers.db.port as i64);
    doc["db"]["db_name"] = value(answers.db.db_name.as_str());
    doc["db"]["username"] = value(answers.db.username.as_str());
    doc["db"]["password"] = value(answers.db.password.as_str());

    Ok(doc.to_string())
}

#[cfg(test)]
mod tests {
    use ::config::{Config, File, FileFormat};

    use super::*;
    use crate::config::config::AppConfig;

    fn answers() -> Answers {
        toml::from_str(
            r#"
[server]
port = 8080

[db]
db_type = "mysql"
url = "10.0.0.5"
port = 3307
db_name = "tinyaxum"
username = "app"
password = "P@ss:w/rd"
"#,
        )
        .unwrap()
    }

    fn parse(text: &str) -> AppConfig {
        Config::builder()
            .add_source(File::from_str(text, FileFormat::Toml))
            .build()
            .expect("build")
            .try_deserialize()
            .expect("deserialize")
    }

    #[test]
    fn user_value_containing_a_placeholder_is_not_re_substituted() {
        // 链式 .replace() 会让这个密码被后一次 @ENCRYPTION_KEY@ 替换连带改写,
        // 结果是密码变成随机密钥、且与 encryption.key 撞成同一个值。
        let mut ans = answers();
        ans.db.password = "@ENCRYPTION_KEY@".to_string();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        write_config(&path, &ans).unwrap();
        let cfg = parse(&std::fs::read_to_string(&path).unwrap());

        assert_eq!(
            cfg.db.password, "@ENCRYPTION_KEY@",
            "用户值里的占位符字面量被二次替换了"
        );
        assert_ne!(cfg.encryption.key, cfg.db.password);
        assert_eq!(cfg.encryption.key.len(), 32);
    }

    #[test]
    fn unknown_placeholders_are_left_alone() {
        // 模板里没登记的 @X@ 应原样保留,不能被吞掉
        assert_eq!(
            render_template("a @NOPE@ b", &[("PORT", "1".to_string())]),
            "a @NOPE@ b"
        );
        // 邮箱之类的裸 @ 不能被误当成占位符起点
        assert_eq!(
            render_template("user@host @PORT@", &[("PORT", "1".to_string())]),
            "user@host 1"
        );
    }

    #[test]
    fn fresh_config_is_valid_appconfig() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        write_config(&path, &answers()).unwrap();

        let text = std::fs::read_to_string(&path).unwrap();
        let cfg = parse(&text);

        assert_eq!(cfg.server.port, 8080);
        assert_eq!(cfg.db.db_type, "mysql");
        assert_eq!(cfg.db.url, "10.0.0.5");
        assert_eq!(cfg.db.port, 3307);
        assert_eq!(cfg.db.db_name, "tinyaxum");
        assert_eq!(cfg.db.username, "app");
        assert_eq!(cfg.db.password, "P@ss:w/rd");
        assert_eq!(cfg.server.static_dir.as_deref(), Some("static"));
    }

    #[test]
    fn fresh_config_generates_32_char_encryption_key() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        write_config(&path, &answers()).unwrap();
        let cfg = parse(&std::fs::read_to_string(&path).unwrap());

        assert_eq!(
            cfg.encryption.key.chars().count(),
            32,
            "encryption.key 必须恰好 32 字符, 实际: {:?}",
            cfg.encryption.key
        );
        assert_eq!(cfg.encryption.key.len(), 32, "必须是 32 字节(纯 ASCII)");
        assert_ne!(cfg.encryption.key, "test-only-encryption-key-32chars");
    }

    #[test]
    fn fresh_config_generates_unique_secrets() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.toml");
        let b = dir.path().join("b.toml");
        write_config(&a, &answers()).unwrap();
        write_config(&b, &answers()).unwrap();

        let ca = parse(&std::fs::read_to_string(&a).unwrap());
        let cb = parse(&std::fs::read_to_string(&b).unwrap());

        assert_ne!(ca.encryption.key, cb.encryption.key);
        assert_ne!(ca.jwt.secret, cb.jwt.secret);
        assert!(!ca.jwt.secret.contains("replace-me"));
    }

    #[test]
    fn quotes_in_password_do_not_break_toml() {
        let mut ans = answers();
        ans.db.password = r#"a"b\c"#.to_string();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        write_config(&path, &ans).unwrap();

        let cfg = parse(&std::fs::read_to_string(&path).unwrap());
        assert_eq!(cfg.db.password, r#"a"b\c"#);
    }

    const EXISTING: &str = r#"
[server]
port = 3000
host = "localhost"
static_dir = "static"

[db]
# 这行注释必须活下来
db_type = "mysql"
url = "127.0.0.1"
port = 3306
db_name = "old"
username = "root"
password = "old-pw"

[jwt]
secret = "EXISTING-JWT-SECRET-DO-NOT-TOUCH"
access_ttl = 3600
refresh_ttl = 1800000

[encryption]
key = "test-only-encryption-key-32chars"

[pool]
db_max_connections = 99
db_min_connections = 7
"#;

    fn write_existing_then_update() -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, EXISTING).unwrap();
        write_config(&path, &answers()).unwrap();
        (dir, path)
    }

    #[test]
    fn upgrade_never_touches_encryption_key() {
        let (_d, path) = write_existing_then_update();
        let cfg = parse(&std::fs::read_to_string(&path).unwrap());
        assert_eq!(
            cfg.encryption.key, "test-only-encryption-key-32chars",
            "升级重写了 encryption.key —— 这会让所有已加密的数据永久解不开"
        );
    }

    #[test]
    fn upgrade_never_touches_jwt_secret() {
        let (_d, path) = write_existing_then_update();
        let cfg = parse(&std::fs::read_to_string(&path).unwrap());
        assert_eq!(cfg.jwt.secret, "EXISTING-JWT-SECRET-DO-NOT-TOUCH");
    }

    #[test]
    fn upgrade_preserves_hand_tuned_pool_section() {
        let (_d, path) = write_existing_then_update();
        let cfg = parse(&std::fs::read_to_string(&path).unwrap());
        assert_eq!(cfg.pool.db_max_connections, 99);
        assert_eq!(cfg.pool.db_min_connections, 7);
    }

    #[test]
    fn upgrade_preserves_comments() {
        let (_d, path) = write_existing_then_update();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(
            text.contains("# 这行注释必须活下来"),
            "toml_edit 应保留注释, 实际:\n{text}"
        );
    }

    #[test]
    fn upgrade_applies_wizard_fields() {
        let (_d, path) = write_existing_then_update();
        let cfg = parse(&std::fs::read_to_string(&path).unwrap());
        assert_eq!(cfg.server.port, 8080);
        assert_eq!(cfg.db.url, "10.0.0.5");
        assert_eq!(cfg.db.port, 3307);
        assert_eq!(cfg.db.db_name, "tinyaxum");
        assert_eq!(cfg.db.username, "app");
        assert_eq!(cfg.db.password, "P@ss:w/rd");
    }

    #[test]
    fn upgrade_preserves_unrelated_server_fields() {
        let (_d, path) = write_existing_then_update();
        let cfg = parse(&std::fs::read_to_string(&path).unwrap());
        assert_eq!(cfg.server.host, "localhost");
        assert_eq!(cfg.server.static_dir.as_deref(), Some("static"));
    }
}
