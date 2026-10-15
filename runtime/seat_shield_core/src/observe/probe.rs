//! Prefer pinned `seat-shield-probe dump` (inet_diag); fail closed to caller → `/proc`.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::command_runner;
use crate::error::{ShieldError, ShieldResult};
use crate::observe::ObservedConn;

pub const PROBE_BACKEND: &str = "probe_inet_diag";
pub const PROBE_TTL_SECS: u64 = 5;
const PROBE_TIMEOUT: Duration = Duration::from_millis(1500);
const MAX_PROBE_SOCKS: usize = 2048;

#[must_use]
pub fn probe_path_is_safe(path: &Path) -> bool {
    path.is_absolute() && !path.to_string_lossy().contains('\0')
}

#[derive(Debug, Clone)]
pub struct ProbeConnSource {
    helper_path: PathBuf,
    cache: Option<(Instant, Vec<ObservedConn>)>,
    ttl: Duration,
}

impl ProbeConnSource {
    pub fn new(helper_path: impl Into<PathBuf>) -> ShieldResult<Self> {
        let helper_path = helper_path.into();
        if !probe_path_is_safe(&helper_path) {
            return Err(ShieldError::Actuator(
                "probe helper_path must be absolute and free of NUL".into(),
            ));
        }
        Ok(Self {
            helper_path,
            cache: None,
            ttl: Duration::from_secs(PROBE_TTL_SECS),
        })
    }

    #[must_use]
    pub fn helper_path(&self) -> &Path {
        &self.helper_path
    }

    /// Collect ESTAB TCP rows from probe dump. Uses in-process TTL cache.
    pub fn collect_fresh(&mut self) -> ShieldResult<Vec<ObservedConn>> {
        if let Some((at, rows)) = &self.cache {
            if at.elapsed() < self.ttl {
                return Ok(rows.clone());
            }
        }
        let result = command_runner::run(
            self.helper_path.to_str().ok_or_else(|| {
                ShieldError::Actuator("probe helper path is not UTF-8".into())
            })?,
            &["dump"],
            PROBE_TIMEOUT,
        )?;
        if result.timed_out {
            return Err(ShieldError::Actuator("probe helper timed out".into()));
        }
        if !result.success {
            return Err(ShieldError::Actuator(format!(
                "probe helper failed: {}",
                result.stderr.chars().take(120).collect::<String>()
            )));
        }
        let line = result
            .stdout
            .lines()
            .find(|l| !l.trim().is_empty())
            .ok_or_else(|| ShieldError::Actuator("probe helper produced no stdout".into()))?;
        let rows = parse_probe_snap(line)?;
        self.cache = Some((Instant::now(), rows.clone()));
        Ok(rows)
    }
}

/// Parse one `t=snap` line from `seat-shield-probe dump`.
pub fn parse_probe_snap(line: &str) -> ShieldResult<Vec<ObservedConn>> {
    let value: Value = serde_json::from_str(line).map_err(|e| ShieldError::Parse {
        context: "probe_snap_parse",
        detail: e.to_string(),
    })?;
    if value.get("t").and_then(Value::as_str) != Some("snap") {
        return Err(ShieldError::Parse {
            context: "probe_snap_parse",
            detail: "expected t=snap".into(),
        });
    }
    let socks = value
        .get("socks")
        .and_then(Value::as_array)
        .ok_or_else(|| ShieldError::Parse {
            context: "probe_snap_parse",
            detail: "socks array required".into(),
        })?;
    let mut out = Vec::new();
    for sock in socks.iter().take(MAX_PROBE_SOCKS) {
        let state = sock.get("state").and_then(Value::as_str).unwrap_or("");
        if state != "ESTAB" {
            continue;
        }
        let remote_ip = sock
            .get("dst")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if remote_ip.is_empty() || remote_ip == "0.0.0.0" || remote_ip == "::" {
            continue;
        }
        let local_port = sock.get("sport").and_then(Value::as_u64).unwrap_or(0);
        let remote_port = sock.get("dport").and_then(Value::as_u64).unwrap_or(0);
        if local_port > u64::from(u16::MAX) || remote_port > u64::from(u16::MAX) {
            continue;
        }
        let process_name = sock
            .get("comm")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        let pid = sock.get("pid").and_then(Value::as_i64).unwrap_or(0) as i32;
        let rtt_ms = sock
            .get("rtt_us")
            .and_then(Value::as_u64)
            .map(|us| (us / 1000) as u32);
        out.push(ObservedConn {
            remote_ip,
            local_port: local_port as u16,
            remote_port: remote_port as u16,
            process_name,
            pid,
            exe_path: None,
            exe_sha256: None,
            rtt_ms,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_snap_with_rtt() {
        let line = r#"{"t":"snap","backend":"inet_diag","socks":[{"proto":"tcp","state":"ESTAB","src":"10.0.0.1","sport":22,"dst":"198.51.100.20","dport":40000,"inode":1,"pid":7,"comm":"sshd","rtt_us":12000,"retrans":0}]}"#;
        let rows = parse_probe_snap(line).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].remote_ip, "198.51.100.20");
        assert_eq!(rows[0].local_port, 22);
        assert_eq!(rows[0].rtt_ms, Some(12));
        assert_eq!(rows[0].process_name, "sshd");
    }

    #[test]
    fn rejects_non_snap() {
        assert!(parse_probe_snap(r#"{"t":"hello"}"#).is_err());
    }
}
