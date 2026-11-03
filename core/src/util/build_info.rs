//! 编译期的发布元数据，`--version` 输出与启动日志共用。

/// 烤进 agent 二进制的构建元数据。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildInfo {
    pub version: &'static str,
    pub build: &'static str,
    pub commit_sha: &'static str,
    pub description: &'static str,
}

impl BuildInfo {
    /// 读取 release CI（或本地发布脚本）注入的元数据；未注入的值退回
    /// 稳定的本地构建缺省。
    ///
    /// rustc 会把 `option_env!` 记进 dep-info，环境变量变化自动触发重编，
    /// 因此不需要 build.rs。
    #[must_use]
    pub const fn current() -> Self {
        Self::from_values(
            option_env!("QUASAR_VERSION"),
            option_env!("QUASAR_BUILD"),
            option_env!("QUASAR_COMMIT_SHA"),
            option_env!("QUASAR_DESCRIPTION"),
        )
    }

    const fn from_values(
        version: Option<&'static str>,
        build: Option<&'static str>,
        commit_sha: Option<&'static str>,
        description: Option<&'static str>,
    ) -> Self {
        Self {
            version: match version {
                Some(value) => value,
                None => env!("CARGO_PKG_VERSION"),
            },
            build: match build {
                Some(value) => value,
                None => "unknown",
            },
            commit_sha: match commit_sha {
                Some(value) => value,
                None => "unknown",
            },
            description: match description {
                Some(value) => value,
                None => "",
            },
        }
    }

    /// 服务诊断串：`<name>-<version>(<build> <sha>)`，空白值的兜底
    /// （dev / -1 / unknown）与 central 的 service_description 口径一致。
    #[must_use]
    pub fn service_description(self, service_name: &str) -> String {
        let version = if self.version.trim().is_empty() {
            "dev"
        } else {
            self.version
        };
        let build = if self.build.trim().is_empty() {
            "-1"
        } else {
            self.build
        };
        let commit_sha = if self.commit_sha.trim().is_empty() {
            "unknown"
        } else {
            self.commit_sha
        };
        format!("{service_name}-{version}({build} {commit_sha})")
    }
}

#[cfg(test)]
mod tests {
    use super::BuildInfo;

    const APP_NAME: &str = "rs-starter-template";

    #[test]
    fn complete_release_metadata_is_preserved() {
        let info = BuildInfo::from_values(
            Some("v1.2.3"),
            Some("2026-08-13"),
            Some("abc12345"),
            Some("修复 release 信息"),
        );

        assert_eq!(info.version, "v1.2.3");
        assert_eq!(info.build, "2026-08-13");
        assert_eq!(info.commit_sha, "abc12345");
        assert_eq!(info.description, "修复 release 信息");
        assert_eq!(
            info.service_description(APP_NAME),
            format!(
                "{}-{}({} {})",
                APP_NAME, info.version, info.build, info.commit_sha
            )
        );
    }

    #[test]
    fn missing_compile_values_use_stable_local_build_fallbacks() {
        let info = BuildInfo::from_values(None, None, None, None);

        assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(info.build, "unknown");
        assert_eq!(info.commit_sha, "unknown");
        assert_eq!(info.description, "");
        assert_eq!(
            info.service_description(APP_NAME),
            format!(
                "{}-{}(unknown unknown)",
                APP_NAME,
                env!("CARGO_PKG_VERSION")
            )
        );
    }

    #[test]
    fn diagnostic_output_applies_blank_value_fallbacks() {
        let info = BuildInfo::from_values(Some(" \t"), Some(""), Some("  "), Some(""));

        assert_eq!(
            info.service_description(APP_NAME),
            format!("{}-dev(-1 unknown)", APP_NAME)
        );
    }
}
