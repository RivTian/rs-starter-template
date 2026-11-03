//! `{{env.NAME:default}}` 占位符展开
//!
//! 手写解析器而不是引模板引擎：需求只有一种形态的占位符，模板引擎带来的
//! 转义规则、控制流语法反而是攻击面。
//!
//! # 语法
//!
//! - `{{env.NAME}}`——环境变量必须已设置且非空，否则 [`ConfigError::EnvMissing`]；
//! - `{{env.NAME:default}}`——未设置或为空时回退 `default`（default 可为空串，
//!   表示「缺省时展开为空」）；
//! - 变量名限 `[A-Za-z_][A-Za-z0-9_]*`；不满足语法的 `{{` 一律原样保留
//!   （TOML 值里合法出现 `{{` 的场景不能被误伤）；
//! - 不递归：展开结果里的 `{{env.…}}` 不再二次展开（防环）。
//!
//! # 统一了什么
//!
//! 占位符让**任何**配置值都能从环境注入，不必为每个敏感字段单独开
//! `*_env` 口子——新部署直接写 `password = "{{env.PG_PASS:}}"` 即可。

use super::error::ConfigError;

/// 对整份配置文本做占位符展开。
pub fn expand_env_placeholders(input: &str) -> Result<String, ConfigError> {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;

    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let after_brace = &rest[start..];

        match parse_placeholder(after_brace) {
            Some((name, default, consumed)) => {
                match std::env::var(name) {
                    Ok(v) if !v.is_empty() => out.push_str(&v),
                    // 未设置或为空：有默认值用默认值，否则报错
                    _ => match default {
                        Some(d) => out.push_str(d),
                        None => {
                            return Err(ConfigError::EnvMissing {
                                name: name.to_string(),
                            });
                        }
                    },
                }
                rest = &after_brace[consumed..];
            }
            // 不是合法占位符：`{{` 原样保留，继续向后扫
            None => {
                out.push_str("{{");
                rest = &after_brace[2..];
            }
        }
    }
    out.push_str(rest);
    Ok(out)
}

/// 尝试从以 `{{` 开头的片段解析一个占位符。
///
/// 返回 `(变量名, 默认值, 消费的字节数)`；语法不符返回 `None`。
fn parse_placeholder(s: &str) -> Option<(&str, Option<&str>, usize)> {
    let body_start = 2; // 跳过 "{{"
    let body = s.get(body_start..)?;
    let end = body.find("}}")?;
    let body = &body[..end];

    let spec = body.strip_prefix("env.")?;
    let (name, default) = match spec.find(':') {
        Some(i) => (&spec[..i], Some(&spec[i + 1..])),
        None => (spec, None),
    };

    if name.is_empty() || !is_valid_var_name(name) {
        return None;
    }
    Some((name, default, body_start + end + 2))
}

fn is_valid_var_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试串行安全：环境变量按测试名隔离，避免并行测试互踩
    fn with_env<R>(name: &str, value: Option<&str>, f: impl FnOnce() -> R) -> R {
        // SAFETY: 测试进程内串行使用互不相同的变量名
        unsafe {
            match value {
                Some(v) => std::env::set_var(name, v),
                None => std::env::remove_var(name),
            }
        }
        let r = f();
        unsafe { std::env::remove_var(name) };
        r
    }

    #[test]
    fn expands_set_variable() {
        with_env("EXP_T1", Some("hello"), || {
            let s = expand_env_placeholders("a = \"{{env.EXP_T1}}\"").unwrap();
            assert_eq!(s, "a = \"hello\"");
        });
    }

    #[test]
    fn falls_back_to_default_when_unset_or_empty() {
        with_env("EXP_T2", None, || {
            let s = expand_env_placeholders("a = {{env.EXP_T2:42}}").unwrap();
            assert_eq!(s, "a = 42");
        });
        // 空值同样回退：`FOO=` 与未设置在配置语义上等价
        with_env("EXP_T3", Some(""), || {
            let s = expand_env_placeholders("a = \"{{env.EXP_T3:x}}\"").unwrap();
            assert_eq!(s, "a = \"x\"");
        });
        // 默认值可为空串
        with_env("EXP_T4", None, || {
            let s = expand_env_placeholders("a = \"{{env.EXP_T4:}}\"").unwrap();
            assert_eq!(s, "a = \"\"");
        });
    }

    #[test]
    fn missing_without_default_is_a_teaching_error() {
        with_env("EXP_T5", None, || {
            let e = expand_env_placeholders("a = {{env.EXP_T5}}").unwrap_err();
            let msg = e.to_string();
            assert!(msg.contains("EXP_T5"), "错误要指名变量: {msg}");
            assert!(msg.contains("默认值"), "错误要教怎么修: {msg}");
        });
    }

    /// 不满足语法的 `{{` 原样保留：TOML 值里的花括号不能被误伤
    #[test]
    fn non_placeholder_braces_survive() {
        for s in [
            "a = \"{{not_env}}\"",
            "a = \"{{env.}}\"",
            "a = \"{{env.1BAD}}\"",
            "a = \"{{ env.X }}\"", // 带空格不算占位符
            "a = \"{{\"",          // 无闭合
        ] {
            assert_eq!(expand_env_placeholders(s).unwrap(), s, "{s} 应原样保留");
        }
    }

    /// 同一行多个占位符、占位符与普通文本混排
    #[test]
    fn multiple_placeholders_in_one_pass() {
        with_env("EXP_T6", Some("u"), || {
            with_env("EXP_T7", Some("p"), || {
                let s = expand_env_placeholders(
                    "dsn = \"{{env.EXP_T6}}:{{env.EXP_T7}}@{{env.EXP_T8:h}}\"",
                )
                .unwrap();
                assert_eq!(s, "dsn = \"u:p@h\"");
            });
        });
    }

    /// 不递归：展开出来的内容不再二次展开
    #[test]
    fn expansion_is_not_recursive() {
        with_env("EXP_T9", Some("{{env.EXP_T9}}"), || {
            let s = expand_env_placeholders("a = \"{{env.EXP_T9}}\"").unwrap();
            assert_eq!(s, "a = \"{{env.EXP_T9}}\"");
        });
    }
}
