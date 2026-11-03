//! `AppConfig` 与统一加载管线
//!
//! 管线（启动与热重载共用同一条）：
//!
//! ```text
//! 读文件（缺文件 → 落盘内嵌模板再读）
//!   → {{env.NAME:default}} 占位符展开
//!   → 反序列化（serde_path_to_error 定位字段路径）
//!   ── 以下为保留钩子，随首个真实业务字段落地 ──
//!   → resolve_paths（相对路径一律相对配置文件目录）
//!   → complete（跨字段推导）
//!   → validate（硬错误，教学式信息）
//!   → sanitize（软越界钳位 + warn）
//! ```
//!
//! 校验的两层分工（字段落地时遵循）：**validate 只拦「继续跑必然失败」的
//! 取值**（反正运行时也会失败，提前到加载期错误信息更好）；其余越界一律
//! 钳位 + warn（无人值守边缘盒，拒绝启动的代价远大于按保守值跑）。

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::env_expand::expand_env_placeholders;
use super::error::ConfigError;

/// 运行时配置。
///
/// 当前没有业务字段。采集、上报等配置将在出现首个真实消费者时加入。
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct AppConfig {}

/// 内嵌默认配置模板（缺文件时落盘；`load_or_init` 使用）。
///
/// 当前 [`AppConfig`] 无字段，模板为空；字段落地时模板与 `Default` 必须
/// 同步（`embedded_template_parses_and_equals_default` 钉住这一点）。
pub fn default_config_template() -> &'static str {
    include_str!("cfg_runtime.toml")
}

/// 加载（不落盘版本：文件必须存在）。启动与热重载共用。
pub fn load(path: &Path) -> Result<AppConfig, ConfigError> {
    let raw = std::fs::read_to_string(path).map_err(|source| ConfigError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let expanded = expand_env_placeholders(&raw)?;
    let deserializer =
        toml::de::Deserializer::parse(&expanded).map_err(|error| ConfigError::Parse {
            path_in_doc: "(toml 语法)".to_string(),
            message: error.message().to_string(),
        })?;

    serde_path_to_error::deserialize(deserializer).map_err(|error| ConfigError::Parse {
        path_in_doc: error.path().to_string(),
        message: error.inner().message().to_string(),
    })

    // TODO: 补充 resolve_paths, complete, validate, sanitize
}

/// 缺文件时先落盘内嵌模板再加载（`boot_strap` / `ConfigStore::load_or_init` 用）, 再走统一加载管线。
pub fn load_or_init(path: &Path) -> Result<AppConfig, ConfigError> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent).map_err(|source| ConfigError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    if !path.exists() {
        std::fs::write(path, default_config_template()).map_err(|source| ConfigError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        tracing::info!(path = %path.display(), "config file missing, default template written");
    }

    load(path)
}

impl AppConfig {
    // TODO: 补充 resolve_paths, complete, validate, sanitize
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn tmp_dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!(
            "starter_cfg_{tag}_{}_{}",
            std::process::id(),
            uuid_ish()
        ));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    /// 不引 uuid：core 无该依赖，测试只要够独特即可
    fn uuid_ish() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    }

    /// 内嵌模板必须能被本管线读通，且等价于 Default（模板即默认值的镜像；
    /// 模板改了默认值没跟上——或反过来——这条立刻红）
    #[test]
    fn embedded_template_parses_and_equals_default() {
        let dir = tmp_dir("tpl");
        let path = dir.join("runtime.toml");
        let cfg = load_or_init(&path).expect("模板必须可读");
        assert!(path.exists(), "缺文件应落盘模板");
        assert_eq!(cfg, AppConfig::default(), "模板取值必须与 Default 一致");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 升级兼容判据（占位）：旧部署的 snake_case 全量配置必须零改动可读。
    /// 当前无业务字段，样本为空；每落地一段配置就把对应旧片段补进来。
    #[test]
    fn legacy_snake_case_config_still_parses() {
        let dir = tmp_dir("legacy");
        let path = dir.join("runtime.toml");
        std::fs::write(
            &path, r#"
"#,
        )
        .unwrap();
        let _cfg = load(&path).expect("旧 snake_case 配置必须可读");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
