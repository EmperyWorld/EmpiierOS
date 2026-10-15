//! Pinned absolute-path floor helper actuator.
//!
//! Calls `seat-shield-fw` (or future `empiieros-seat-shield-fw`) with argv only.
//! Never builds shell strings. Never accepts relative helper paths.

use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::Deserialize;

use crate::act::FirewallActuator;
use crate::command_runner;
use crate::error::{ShieldError, ShieldResult};
use crate::fw;

const HELPER_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Deserialize)]
struct HelperResponse {
    ok: bool,
    #[serde(default)]
    peers: Vec<String>,
    #[serde(default)]
    removed: bool,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct HelperActuator {
    helper_path: PathBuf,
    state_dir: PathBuf,
    /// Allowlisted flag/value pairs inserted before `--state-dir` (e.g. nft backend).
    prefix: Vec<String>,
}

impl HelperActuator {
    pub fn new(helper_path: impl Into<PathBuf>, state_dir: impl Into<PathBuf>) -> ShieldResult<Self> {
        Self::with_prefix(helper_path, state_dir, Vec::new())
    }

    pub fn with_prefix(
        helper_path: impl Into<PathBuf>,
        state_dir: impl Into<PathBuf>,
        prefix: Vec<String>,
    ) -> ShieldResult<Self> {
        let helper_path = helper_path.into();
        let state_dir = state_dir.into();
        if !helper_path_is_safe(&helper_path) {
            return Err(ShieldError::Actuator(
                "helper_path must be absolute and free of NUL".into(),
            ));
        }
        if !state_dir.is_absolute() {
            return Err(ShieldError::Actuator(
                "helper state_dir must be absolute".into(),
            ));
        }
        fw::validate_helper_prefix(&prefix).map_err(ShieldError::Actuator)?;
        if !helper_path.exists() {
            return Err(ShieldError::NotAvailable(format!(
                "helper binary missing: {}",
                helper_path.display()
            )));
        }
        Ok(Self {
            helper_path,
            state_dir,
            prefix,
        })
    }

    fn invoke(&self, args: &[&str]) -> ShieldResult<HelperResponse> {
        let helper = self.helper_path.to_str().ok_or_else(|| {
            ShieldError::Actuator("helper path is not valid UTF-8".into())
        })?;
        let state = self.state_dir.to_str().ok_or_else(|| {
            ShieldError::Actuator("state_dir is not valid UTF-8".into())
        })?;
        let mut argv: Vec<&str> = Vec::with_capacity(self.prefix.len() + 2 + args.len());
        for token in &self.prefix {
            argv.push(token.as_str());
        }
        argv.push("--state-dir");
        argv.push(state);
        argv.extend_from_slice(args);
        let result = command_runner::run(helper, &argv, HELPER_TIMEOUT)?;
        if result.timed_out {
            return Err(ShieldError::Actuator("helper timed out".into()));
        }
        if !result.success {
            return Err(ShieldError::Actuator(format!(
                "helper exit={:?} stderr={}",
                result.exit_code,
                result.stderr.trim()
            )));
        }
        let line = result
            .stdout
            .lines()
            .find(|l| !l.trim().is_empty())
            .ok_or_else(|| ShieldError::Actuator("helper produced no JSON stdout".into()))?;
        let parsed: HelperResponse =
            serde_json::from_str(line).map_err(|e| ShieldError::Parse {
                context: "helper_json",
                detail: e.to_string(),
            })?;
        if !parsed.ok {
            return Err(ShieldError::Actuator(
                parsed
                    .error
                    .unwrap_or_else(|| "helper returned ok=false".into()),
            ));
        }
        Ok(parsed)
    }
}

fn validate_peer(peer_ip: &str) -> ShieldResult<()> {
    peer_ip
        .parse::<IpAddr>()
        .map_err(|_| ShieldError::InvalidParams(format!("invalid peer_ip: {peer_ip}")))?;
    Ok(())
}

impl FirewallActuator for HelperActuator {
    fn backend_name(&self) -> &'static str {
        "helper"
    }

    fn is_available(&self) -> bool {
        self.helper_path.is_absolute() && self.helper_path.exists()
    }

    fn block_peer(&mut self, peer_ip: &str, reason: &str) -> ShieldResult<()> {
        validate_peer(peer_ip)?;
        let reason = sanitize_reason(reason);
        let _ = self.invoke(&["block", "--ip", peer_ip, "--reason", &reason])?;
        Ok(())
    }

    fn unblock_peer(&mut self, peer_ip: &str) -> ShieldResult<bool> {
        validate_peer(peer_ip)?;
        let resp = self.invoke(&["unblock", "--ip", peer_ip])?;
        Ok(resp.removed)
    }

    fn list_blocked(&self) -> ShieldResult<Vec<String>> {
        let resp = self.invoke(&["list"])?;
        Ok(resp.peers)
    }
}

fn sanitize_reason(reason: &str) -> String {
    reason
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, ' ' | '.' | '-' | '_' | ':') {
                c
            } else {
                '_'
            }
        })
        .take(200)
        .collect()
}

/// Resolve helper path for docs/tests.
#[must_use]
pub fn helper_path_is_safe(path: &Path) -> bool {
    path.is_absolute() && !path.to_string_lossy().contains('\0')
}
