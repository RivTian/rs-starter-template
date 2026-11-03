//! CLI 解析
//!
//! 兼容语义（对等纪律）：旧二进制只认第一个参数的 `--version|-v|-V`，
//! 其余参数一律忽略——保持忽略（部署脚本可能带着历史遗留参数），
//! 新增 `--config <path>` 覆盖默认配置路径

use std::path::PathBuf;

/// CLI 动作。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliAction {
    Run { config: Option<PathBuf> },
    PrintVersion,
}

/// 解析参数（不含 argv[0]）。
pub fn parse(args: impl IntoIterator<Item = String>) -> CliAction {
    let args: Vec<String> = args.into_iter().collect();

    // 旧行为：只看第一个参数是否为版本请求
    if matches!(
        args.first().map(String::as_str),
        Some("--version" | "-v" | "-V")
    ) {
        return CliAction::PrintVersion;
    }

    let mut config = None;
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--config" | "-c" => {
                config = it.next().map(PathBuf::from);
                if config.is_none() {
                    eprintln!("rs-starter-template: {arg} 需要一个路径参数，已忽略");
                }
            }
            other => {
                if let Some(path) = other.strip_prefix("--config=") {
                    config = Some(PathBuf::from(path));
                }
                // 其余参数静默忽略（旧行为对等）
            }
        }
    }
    CliAction::Run { config }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn version_flags_win_in_first_position() {
        for flag in ["--version", "-v", "-V"] {
            assert_eq!(parse(args(&[flag])), CliAction::PrintVersion);
        }
        // 旧行为：仅第一个参数位生效
        assert_eq!(
            parse(args(&["run", "--version"])),
            CliAction::Run { config: None }
        );
    }

    #[test]
    fn config_flag_both_forms() {
        assert_eq!(
            parse(args(&["--config", "/tmp/a.toml"])),
            CliAction::Run {
                config: Some(PathBuf::from("/tmp/a.toml"))
            }
        );
        assert_eq!(
            parse(args(&["--config=/tmp/b.toml"])),
            CliAction::Run {
                config: Some(PathBuf::from("/tmp/b.toml"))
            }
        );
    }

    #[test]
    fn unknown_args_are_ignored_like_the_old_binary() {
        assert_eq!(
            parse(args(&["--whatever", "positional"])),
            CliAction::Run { config: None }
        );
        assert_eq!(parse(Vec::new()), CliAction::Run { config: None });
    }
}
