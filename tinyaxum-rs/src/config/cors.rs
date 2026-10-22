use std::time::Duration;

use axum::http::{
    Method,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
};
use serde::Deserialize;
use tower_http::cors::{AllowOrigin, CorsLayer};

/// `[cors]` 配置段。
///
/// 渗透测试高频检查项：把请求 Origin 改成第三方地址，看响应是否反射
/// `Access-Control-Allow-Origin`。本模块的行为契约：
/// - `enabled = false`（或整段缺省）：不挂 CORS 层，任何响应都不带 CORS 头；
/// - `enabled = true`：仅当请求 Origin 命中 [`Self::allowed_origin_patterns`]
///   才回显该 Origin；未命中（含 Origin 非法）绝不回显——这正是
///   "反射型 CORS 漏洞"的分界线。
///
/// Cookie 认证要求跨域响应带 `Access-Control-Allow-Credentials: true`，
/// 此时 ACAO 不允许为 `*`，predicate 按命中回显精确 Origin 天然满足。
#[derive(Clone, Debug, Deserialize, Default)]
pub struct CorsConfig {
    #[serde(default)]
    pub enabled: bool,
    /// Origin 白名单模式，形如 `scheme://host[:port]`，host 与 port 段均可通配：
    /// `http://localhost:*`（任意端口）、`https://*.example.com`（任意子域）、
    /// `*`（显式全放行）。模式不写端口时对端口不限。
    #[serde(default)]
    pub allowed_origin_patterns: Vec<String>,
}

impl CorsConfig {
    /// 未启用或未配任何模式时返回 `None`：路由完全不挂 CORS 层。
    pub fn cors_layer(&self) -> Option<CorsLayer> {
        if !self.enabled || self.allowed_origin_patterns.is_empty() {
            return None;
        }
        let patterns = self.allowed_origin_patterns.clone();
        Some(
            CorsLayer::new()
                .allow_origin(AllowOrigin::predicate(move |origin, _| {
                    origin
                        .to_str()
                        .map(|o| origin_allowed(&patterns, o))
                        .unwrap_or(false)
                }))
                .allow_credentials(true)
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                    Method::PATCH,
                ])
                .allow_headers([CONTENT_TYPE, AUTHORIZATION, ACCEPT])
                .max_age(Duration::from_secs(3600)),
        )
    }
}

/// Origin 是否命中任一模式。空 Origin（同源请求/非浏览器）与
/// 结构非法的 Origin（浏览器不会产生）一律不放行。
pub fn origin_allowed(patterns: &[String], origin: &str) -> bool {
    if origin.is_empty() {
        return false;
    }
    let Some(parts) = split_authority(origin, false) else {
        return false;
    };
    patterns.iter().any(|p| {
        if p == "*" {
            // 显式全放行（Origin 结构合法性已在上面验证过）
            return true;
        }
        split_authority(p, true).is_some_and(|pat| {
            pat.scheme == parts.scheme
                && wildcard_match(pat.host, parts.host)
                && match pat.port {
                    // 模式端口为 `*`：任意端口（含浏览器省略的缺省端口）
                    Some("*") => true,
                    // 模式写了具体端口：端口必须完全相等
                    Some(n) => parts.port == Some(n),
                    // 模式未写端口：只约束 scheme+host
                    None => true,
                }
        })
    })
}

struct AuthorityParts<'a> {
    scheme: &'a str,
    host: &'a str,
    port: Option<&'a str>,
}

/// `scheme://host[:port]` 三段拆分。
///
/// - `wildcard=true`（模式）：host/port 段允许 `*`；
/// - `wildcard=false`（真实 Origin）：scheme 必须全小写、不得含 `*`；
///   路径、查询、用户信息、非法端口等浏览器不会放进 Origin 的内容，
///   一律返回 `None`。
fn split_authority(s: &str, wildcard: bool) -> Option<AuthorityParts<'_>> {
    let (scheme, rest) = s.split_once("://")?;
    if scheme.is_empty() || rest.is_empty() {
        return None;
    }
    if !wildcard
        && (!scheme.bytes().all(|b| b.is_ascii_lowercase())
            || rest.contains('*'))
    {
        return None;
    }
    if rest.contains(['/', '?', '#', '@']) {
        return None;
    }
    // 端口拆分：IPv6 字面量先剥方括号
    let (host, port) = if let Some(close) = rest.strip_prefix('[') {
        let (h, after) = close.split_once(']')?;
        match after.strip_prefix(':') {
            Some(p) => (h, Some(p)),
            None if after.is_empty() => (h, None),
            None => return None,
        }
    } else {
        match rest.rsplit_once(':') {
            Some((h, p)) if valid_port(p, wildcard) => (h, Some(p)),
            // 有冒号但端口段非法或为空——整串不认，防止 `*` 跨进 host 位
            Some(_) => return None,
            None => (rest, None),
        }
    };
    if host.is_empty() {
        return None;
    }
    if port.is_some_and(|p| !valid_port(p, wildcard)) {
        return None;
    }
    Some(AuthorityParts { scheme, host, port })
}

fn valid_port(p: &str, wildcard: bool) -> bool {
    !p.is_empty()
        && p.bytes()
            .all(|b| b.is_ascii_digit() || (wildcard && b == b'*'))
}

/// 整串锚定的通配符匹配，仅 `*` 一个元字符（匹配任意字符序列，含空串）。
///
/// 只在 scheme/host 单段内使用（结构化拆分保证 `*` 不会跨段）。
/// 不引入 regex 依赖：feat 分支的 vendor workspace 加 crate 需要重新
/// vendor，为一个 Origin 匹配不值得。
fn wildcard_match(pattern: &str, text: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = text.chars().collect();
    let (mut pi, mut ti) = (0usize, 0usize);
    // 最近一次 `*` 的位置与它开始尝试匹配的 text 下标（回溯锚点）
    let (mut star_pi, mut star_ti) = (usize::MAX, 0usize);

    while ti < t.len() {
        if pi < p.len() && p[pi] == '*' {
            star_pi = pi;
            star_ti = ti;
            pi += 1;
        } else if pi < p.len() && p[pi] == t[ti] {
            pi += 1;
            ti += 1;
        } else if star_pi != usize::MAX {
            // 当前字符失配：让上一个 `*` 多吞一个字符后重试
            pi = star_pi + 1;
            star_ti += 1;
            ti = star_ti;
        } else {
            return false;
        }
    }
    while pi < p.len() && p[pi] == '*' {
        pi += 1;
    }
    pi == p.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn allowed(patterns: &[&str], origin: &str) -> bool {
        origin_allowed(
            &patterns
                .iter()
                .map(|s| (*s).to_string())
                .collect::<Vec<_>>(),
            origin,
        )
    }

    #[test]
    fn exact_pattern_matches_verbatim() {
        assert!(allowed(&["http://localhost:5173"], "http://localhost:5173"));
        assert!(!allowed(
            &["http://localhost:5173"],
            "http://localhost:5174"
        ));
    }

    #[test]
    fn port_wildcard_matches_any_port() {
        let p = &["http://localhost:*", "http://127.0.0.1:*"];
        assert!(allowed(p, "http://localhost:5173"));
        assert!(allowed(p, "http://127.0.0.1:3000"));
        // `*` 端口也放行缺省端口形式（浏览器对 80/443 省略端口）
        assert!(allowed(p, "http://localhost"));
        // 端口位通配绝不等于 host 位通配
        assert!(!allowed(p, "http://localhost.evil.com:5173"));
        assert!(!allowed(p, "http://localhost:5173.evil.com"));
    }

    #[test]
    fn host_wildcard_matches_subdomains_not_apex() {
        let p = &["https://*.example.com"];
        assert!(allowed(p, "https://app.example.com"));
        assert!(allowed(p, "https://a.b.example.com"));
        assert!(!allowed(p, "https://example.com"));
    }

    #[test]
    fn weird_origins_are_rejected_structurally() {
        let p = &["http://localhost:*", "https://*.example.com", "*"];
        // Origin 不应有路径/查询
        assert!(!allowed(p, "https://evil.com/u?x=.example.com"));
        // 用户信息段
        assert!(!allowed(p, "https://evil.com@example.com"));
        // 真实 Origin 不含通配符字面量
        assert!(!allowed(p, "https://*.example.com"));
        // 非 http(s) scheme
        assert!(!allowed(p, "file:///etc/passwd"));
        assert!(!allowed(p, "null"));
    }

    #[test]
    fn evil_origin_never_matches() {
        let p = &["http://localhost:*", "http://127.0.0.1:*"];
        assert!(!allowed(p, "https://evil.com"));
        assert!(!allowed(p, "http://localhost:5173.evil.com"));
    }

    #[test]
    fn scheme_is_part_of_the_match() {
        let p = &["http://localhost:*"];
        assert!(!allowed(p, "https://localhost:5173"));
    }

    #[test]
    fn pattern_without_port_constrains_host_only() {
        let p = &["http://localhost"];
        assert!(allowed(p, "http://localhost:5173"));
        assert!(allowed(p, "http://localhost"));
        assert!(!allowed(p, "http://localhost.evil.com"));
    }

    #[test]
    fn ipv6_loopback_with_and_without_port() {
        let p = &["http://[::1]:*"];
        assert!(allowed(p, "http://[::1]:5173"));
        assert!(allowed(p, "http://[::1]"));
        assert!(!allowed(p, "http://[::2]:5173"));
    }

    #[test]
    fn full_wildcard_is_explicit_opt_in() {
        assert!(allowed(&["*"], "https://anything.example.org"));
        assert!(!allowed(&["*"], ""));
        assert!(!allowed(&["*"], "https://evil.com@example.com"));
    }

    #[test]
    fn empty_origin_is_rejected() {
        assert!(!allowed(&["http://localhost:*"], ""));
    }

    #[test]
    fn cors_layer_gated_by_enabled_and_patterns() {
        assert!(CorsConfig::default().cors_layer().is_none());
        assert!(
            CorsConfig {
                enabled: true,
                allowed_origin_patterns: vec![],
            }
            .cors_layer()
            .is_none()
        );
        assert!(
            CorsConfig {
                enabled: true,
                allowed_origin_patterns: vec!["http://localhost:*".to_string()],
            }
            .cors_layer()
            .is_some()
        );
        // enabled=false 即使配了模式也不挂层
        assert!(
            CorsConfig {
                enabled: false,
                allowed_origin_patterns: vec!["http://localhost:*".to_string()],
            }
            .cors_layer()
            .is_none()
        );
    }
}
