use std::path::PathBuf;

/// 安装器用的子命令。没有命中任何一个就正常启动服务器。
#[derive(Debug, PartialEq)]
pub enum Cli {
    Serve,
    WriteConfig { config: PathBuf, answers: PathBuf },
    CheckConfig { config: PathBuf },
}

pub fn parse<I: IntoIterator<Item = String>>(args: I) -> anyhow::Result<Cli> {
    let mut it = args.into_iter();
    it.next(); // argv[0]

    let mut config: Option<PathBuf> = None;
    let mut answers: Option<PathBuf> = None;
    let mut check = false;
    let mut write = false;

    while let Some(arg) = it.next() {
        // 注意：这里不能把取值抽成闭包 —— 闭包会可变借用 `it`，
        // 与外层 `while let` 的 `it.next()` 冲突，编译不过。
        match arg.as_str() {
            "--write-config" => {
                write = true;
                config = Some(PathBuf::from(it.next().ok_or_else(|| {
                    anyhow::anyhow!("--write-config 缺少路径参数")
                })?));
            }
            "--check-config" => {
                check = true;
                config = Some(PathBuf::from(it.next().ok_or_else(|| {
                    anyhow::anyhow!("--check-config 缺少路径参数")
                })?));
            }
            "--answers" => {
                answers = Some(PathBuf::from(it.next().ok_or_else(|| {
                    anyhow::anyhow!("--answers 缺少路径参数")
                })?));
            }
            other => anyhow::bail!("未知参数: {other}"),
        }
    }

    match (write, check) {
        (true, true) => {
            anyhow::bail!("--write-config 与 --check-config 不能同时使用")
        }
        (true, false) => Ok(Cli::WriteConfig {
            config: config.expect("--write-config 已带路径"),
            answers: answers.ok_or_else(|| {
                anyhow::anyhow!("--write-config 需要 --answers")
            })?,
        }),
        (false, true) => Ok(Cli::CheckConfig {
            config: config.expect("--check-config 已带路径"),
        }),
        (false, false) => Ok(Cli::Serve),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(args: &[&str]) -> anyhow::Result<Cli> {
        let mut v = vec!["tinyaxum.exe".to_string()];
        v.extend(args.iter().map(|s| s.to_string()));
        parse(v)
    }

    #[test]
    fn no_args_serves() {
        assert_eq!(p(&[]).unwrap(), Cli::Serve);
    }

    #[test]
    fn parses_write_config() {
        let got =
            p(&["--write-config", "c.toml", "--answers", "a.toml"]).unwrap();
        assert_eq!(
            got,
            Cli::WriteConfig {
                config: PathBuf::from("c.toml"),
                answers: PathBuf::from("a.toml"),
            }
        );
    }

    #[test]
    fn write_config_argument_order_does_not_matter() {
        let got =
            p(&["--answers", "a.toml", "--write-config", "c.toml"]).unwrap();
        assert_eq!(
            got,
            Cli::WriteConfig {
                config: PathBuf::from("c.toml"),
                answers: PathBuf::from("a.toml"),
            }
        );
    }

    #[test]
    fn parses_check_config() {
        let got = p(&["--check-config", "c.toml"]).unwrap();
        assert_eq!(
            got,
            Cli::CheckConfig {
                config: PathBuf::from("c.toml")
            }
        );
    }

    #[test]
    fn write_config_without_answers_errors() {
        let err = p(&["--write-config", "c.toml"]).unwrap_err().to_string();
        assert!(err.contains("--answers"), "实际: {err}");
    }

    #[test]
    fn missing_value_errors() {
        assert!(p(&["--check-config"]).is_err());
    }

    #[test]
    fn unknown_flag_errors() {
        let err = p(&["--nope"]).unwrap_err().to_string();
        assert!(err.contains("--nope"), "实际: {err}");
    }
}
