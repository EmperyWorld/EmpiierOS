//! HTTPS navigation policy — lockstep with EliteEW EBrowser nav_policy.
//!
//! Authoritative harvest:
//! `powered_by_emperyworld/.../ebrowser_nav_policy.rs`
//!
//! This module is pure: no global ACTIVE_POLICY mutex (Runtime owns freshness).

use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use url::Url;

/// Default policy TTL (seconds) — matches EBrowser `POLICY_TTL_SECONDS`.
pub const POLICY_TTL_SECONDS: u64 = 300;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyMode {
    Open,
    Allowlist,
    Denylist,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicySnapshot {
    pub mode: PolicyMode,
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
    /// Unix epoch millis when the seat stamped the policy (receive clock).
    pub fetched_at_ms: u64,
    #[serde(default = "default_ttl")]
    pub ttl_seconds: u64,
    /// True when no explicit seat/admin row exists (empty allowlist deny-all).
    #[serde(default)]
    pub is_default: bool,
    /// Overlay UserMedia. Default true so existing VIP Meet does not regress.
    #[serde(default = "default_media_on")]
    pub camera: bool,
    #[serde(default = "default_media_on")]
    pub microphone: bool,
}

fn default_media_on() -> bool {
    true
}

fn default_ttl() -> u64 {
    POLICY_TTL_SECONDS
}

/// Missing-row default: allowlist + empty allow → deny all HTTPS.
#[must_use]
pub fn default_missing_policy(now_ms: u64) -> PolicySnapshot {
    PolicySnapshot {
        mode: PolicyMode::Allowlist,
        allow: Vec::new(),
        deny: Vec::new(),
        fetched_at_ms: now_ms,
        ttl_seconds: POLICY_TTL_SECONDS,
        is_default: true,
        camera: true,
        microphone: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DenyReason {
    Expired,
    Scheme,
    Userinfo,
    Port,
    MissingHost,
    AllowlistMiss,
    DenylistHit,
    DenylistEmpty,
    InvalidUrl,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavDecision {
    Allow,
    Deny(DenyReason),
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PolicySaveError {
    #[error("denylist_empty_forbidden")]
    DenylistEmptyForbidden,
}

/// Validate policy before persist (EBrowser save lock).
pub fn validate_policy_for_save(policy: &PolicySnapshot) -> Result<(), PolicySaveError> {
    if policy.mode == PolicyMode::Denylist && policy.deny.is_empty() {
        return Err(PolicySaveError::DenylistEmptyForbidden);
    }
    Ok(())
}

/// Evaluate navigation against a policy snapshot (caller's clock via `now_ms`).
#[must_use]
pub fn evaluate_navigation(url: &Url, policy: &PolicySnapshot, now_ms: u64) -> NavDecision {
    let ttl = policy.ttl_seconds.max(1).saturating_mul(1000);
    if now_ms > policy.fetched_at_ms.saturating_add(ttl) {
        return NavDecision::Deny(DenyReason::Expired);
    }

    let scheme = url.scheme();
    if scheme == "about" {
        let ok = url.as_str() == "about:blank" || url.as_str().starts_with("about:blank?");
        return if ok {
            NavDecision::Allow
        } else {
            NavDecision::Deny(DenyReason::Scheme)
        };
    }
    if matches!(
        scheme,
        "javascript" | "data" | "blob" | "file" | "http"
    ) {
        return NavDecision::Deny(DenyReason::Scheme);
    }
    if scheme != "https" {
        return NavDecision::Deny(DenyReason::Scheme);
    }
    if !url.username().is_empty() || url.password().is_some() {
        return NavDecision::Deny(DenyReason::Userinfo);
    }
    if let Some(port) = url.port() {
        if port != 443 {
            return NavDecision::Deny(DenyReason::Port);
        }
    }
    let Some(host) = url.host_str() else {
        return NavDecision::Deny(DenyReason::MissingHost);
    };
    let host_l = host.to_ascii_lowercase();

    match policy.mode {
        PolicyMode::Open => NavDecision::Allow,
        PolicyMode::Allowlist => {
            if policy.allow.is_empty() {
                return NavDecision::Deny(DenyReason::AllowlistMiss);
            }
            if host_matches_any(&host_l, &policy.allow) {
                NavDecision::Allow
            } else {
                NavDecision::Deny(DenyReason::AllowlistMiss)
            }
        }
        PolicyMode::Denylist => {
            if policy.deny.is_empty() {
                return NavDecision::Deny(DenyReason::DenylistEmpty);
            }
            if host_matches_any(&host_l, &policy.deny) {
                NavDecision::Deny(DenyReason::DenylistHit)
            } else {
                NavDecision::Allow
            }
        }
    }
}

/// Parse URL then evaluate; invalid URL → deny.
#[must_use]
pub fn evaluate_navigation_str(raw: &str, policy: &PolicySnapshot, now_ms: u64) -> NavDecision {
    match Url::parse(raw) {
        Ok(url) => evaluate_navigation(&url, policy, now_ms),
        Err(_) => NavDecision::Deny(DenyReason::InvalidUrl),
    }
}

#[must_use]
pub fn now_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn host_matches_any(host: &str, entries: &[String]) -> bool {
    for raw in entries {
        let mut entry = raw.trim().to_ascii_lowercase();
        if entry.is_empty() {
            continue;
        }
        if let Some(stripped) = entry.strip_prefix('.') {
            entry = stripped.to_string();
        }
        if ipv4_in_cidr(host, &entry) || ipv6_in_cidr(host, &entry) {
            return true;
        }
        if host == entry || host.ends_with(&format!(".{entry}")) {
            return true;
        }
    }
    false
}

fn ipv4_in_cidr(host: &str, entry: &str) -> bool {
    let (net_s, prefix) = match entry.split_once('/') {
        Some((n, p)) => match p.parse::<u32>() {
            Ok(v) => (n, v),
            Err(_) => return false,
        },
        None => (entry, 32u32),
    };
    if prefix > 32 {
        return false;
    }
    if net_s.contains(':') {
        return false;
    }
    let Ok(host_ip) = host.parse::<Ipv4Addr>() else {
        return false;
    };
    let Ok(net_ip) = net_s.parse::<Ipv4Addr>() else {
        return false;
    };
    let host_n = u32::from(host_ip);
    let net_n = u32::from(net_ip);
    let mask = if prefix == 0 {
        0u32
    } else {
        (!0u32) << (32 - prefix)
    };
    (host_n & mask) == (net_n & mask)
}

fn ipv6_in_cidr(host: &str, entry: &str) -> bool {
    let (net_s, prefix) = match entry.split_once('/') {
        Some((n, p)) => match p.parse::<u32>() {
            Ok(v) => (n, v),
            Err(_) => return false,
        },
        None => (entry, 128u32),
    };
    if prefix > 128 {
        return false;
    }
    let host_clean = host.trim().trim_start_matches('[').trim_end_matches(']');
    let net_clean = net_s.trim().trim_start_matches('[').trim_end_matches(']');
    if !net_clean.contains(':') {
        return false;
    }
    let Ok(host_ip) = host_clean.parse::<Ipv6Addr>() else {
        return false;
    };
    let Ok(net_ip) = net_clean.parse::<Ipv6Addr>() else {
        return false;
    };
    let host_n = u128::from(host_ip);
    let net_n = u128::from(net_ip);
    let mask = if prefix == 0 {
        0u128
    } else {
        (!0u128) << (128 - prefix)
    };
    (host_n & mask) == (net_n & mask)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(mode: PolicyMode, allow: &[&str], deny: &[&str]) -> PolicySnapshot {
        PolicySnapshot {
            mode,
            allow: allow.iter().map(|s| (*s).to_string()).collect(),
            deny: deny.iter().map(|s| (*s).to_string()).collect(),
            fetched_at_ms: now_epoch_ms(),
            ttl_seconds: POLICY_TTL_SECONDS,
            is_default: false,
            camera: true,
            microphone: true,
        }
    }

    fn allow(url: &str, policy: &PolicySnapshot) {
        let now = now_epoch_ms();
        assert_eq!(
            evaluate_navigation_str(url, policy, now),
            NavDecision::Allow,
            "expected allow for {url}"
        );
    }

    fn deny(url: &str, policy: &PolicySnapshot, reason: DenyReason) {
        let now = now_epoch_ms();
        assert_eq!(
            evaluate_navigation_str(url, policy, now),
            NavDecision::Deny(reason),
            "expected deny({reason:?}) for {url}"
        );
    }

    #[test]
    fn open_https_ok_http_denied() {
        let p = snap(PolicyMode::Open, &[], &[]);
        allow("https://example.com/x", &p);
        deny("http://example.com/x", &p, DenyReason::Scheme);
    }

    #[test]
    fn shared_deny_schemes() {
        let p = snap(PolicyMode::Open, &[], &[]);
        deny("javascript:alert(1)", &p, DenyReason::Scheme);
        deny("data:text/html,hi", &p, DenyReason::Scheme);
        deny("file:///etc/passwd", &p, DenyReason::Scheme);
        deny("blob:https://example.com/x", &p, DenyReason::Scheme);
        allow("about:blank", &p);
    }

    #[test]
    fn userinfo_and_non_443_denied() {
        let p = snap(PolicyMode::Open, &[], &[]);
        deny("https://user:pass@example.com/", &p, DenyReason::Userinfo);
        deny("https://example.com:8443/", &p, DenyReason::Port);
    }

    #[test]
    fn allowlist_suffix_and_cidr() {
        let p = snap(PolicyMode::Allowlist, &["example.com", "10.0.0.0/8"], &[]);
        allow("https://sub.example.com/", &p);
        deny("https://evil.com/", &p, DenyReason::AllowlistMiss);
        allow("https://10.1.2.3/", &p);
    }

    #[test]
    fn allowlist_ipv6_cidr() {
        let p = snap(PolicyMode::Allowlist, &["2001:db8::/32"], &[]);
        allow("https://[2001:db8::1]/", &p);
        deny("https://[2001:db9::1]/", &p, DenyReason::AllowlistMiss);
    }

    #[test]
    fn expired_fail_closed() {
        let mut p = snap(PolicyMode::Open, &[], &[]);
        p.fetched_at_ms = now_epoch_ms().saturating_sub(400_000);
        deny("https://example.com/", &p, DenyReason::Expired);
    }

    #[test]
    fn empty_denylist_fail_closed() {
        let p = snap(PolicyMode::Denylist, &[], &[]);
        deny("https://example.com/", &p, DenyReason::DenylistEmpty);
        assert_eq!(
            validate_policy_for_save(&p),
            Err(PolicySaveError::DenylistEmptyForbidden)
        );
    }

    #[test]
    fn empty_allowlist_fail_closed() {
        let p = snap(PolicyMode::Allowlist, &[], &[]);
        deny("https://example.com/", &p, DenyReason::AllowlistMiss);
    }

    #[test]
    fn default_missing_policy_denies_https() {
        let p = default_missing_policy(now_epoch_ms());
        assert!(p.is_default);
        assert_eq!(p.mode, PolicyMode::Allowlist);
        deny("https://example.com/", &p, DenyReason::AllowlistMiss);
        allow("about:blank", &p);
    }

    #[test]
    fn denylist_blocks_listed_allows_other() {
        let p = snap(PolicyMode::Denylist, &[], &["evil.com"]);
        deny("https://evil.com/", &p, DenyReason::DenylistHit);
        deny("https://sub.evil.com/", &p, DenyReason::DenylistHit);
        allow("https://example.com/", &p);
    }

    #[test]
    fn leading_dot_suffix_entry() {
        let p = snap(PolicyMode::Allowlist, &[".example.com"], &[]);
        allow("https://a.example.com/", &p);
        allow("https://example.com/", &p);
    }
}
