//! Build provenance read from the `VERGEN_*` variables emitted by `build.rs`.

use {{crate_name}}_core::build_info::BuildInfo;

const IDEMPOTENT: &str = "VERGEN_IDEMPOTENT_OUTPUT";

fn clean(value: Option<&'static str>) -> &'static str {
    match value {
        Some(v) if !v.is_empty() && v != IDEMPOTENT => v,
        _ => "unknown",
    }
}

fn short_sha(sha: &'static str) -> &'static str {
    sha.get(..12).unwrap_or(sha)
}

/// Facts about this binary.
#[must_use]
pub fn current() -> BuildInfo {
    BuildInfo {
        name: env!("CARGO_PKG_NAME"),
        version: env!("CARGO_PKG_VERSION"),
        git_sha: short_sha(clean(option_env!("VERGEN_GIT_SHA"))),
        git_describe: clean(option_env!("VERGEN_GIT_DESCRIBE")),
        build_timestamp: clean(option_env!("VERGEN_BUILD_TIMESTAMP")),
        rustc: clean(option_env!("VERGEN_RUSTC_SEMVER")),
        profile: if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        },
    }
}
