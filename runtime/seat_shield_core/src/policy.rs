//! Seat policy — Runtime/floor authoritative; Shell never supplies enforcement blobs.

use serde::{Deserialize, Serialize};

/// MVP default: SSH only. Expanding requires contract + allowlist change.
///
/// VNC/RDP/WinRM are not this seat. Do not seed them as defaults.
pub const DEFAULT_PROTECTED_PORTS: &[u16] = &[22];

pub const SUSPICIOUS_EXEC_PREFIXES: &[&str] = &["/tmp/", "/var/tmp/", "/dev/shm/"];

/// Remote ports that raise `network_suspicious_port` (event only, never auto-act).
pub const SUSPICIOUS_REMOTE_PORTS: &[u16] = &[4444, 5555, 6667, 31337, 54321, 12345];

/// Monitoring scanner-skip prefixes (Google / DNS / CDN). Copied, not imported.
pub const DEFAULT_SCANNER_SKIP_PREFIXES: &[&str] = &[
    "142.250.", "142.251.", "172.217.", "74.125.", "216.58.", "1.1.1.", "1.0.0.", "8.8.", "9.9.9.",
    "149.112.", "208.67.",
];

pub const DEFAULT_CYCLE_INTERVAL_SECS: u64 = 30;
pub const DEFAULT_MAX_AUTO_ACTS_PER_CYCLE: u32 = 5;
pub const DEFAULT_SCANNER_PORT_THRESHOLD: u32 = 6;

pub const MAX_ALLOWLIST_ENTRIES: usize = 256;
pub const MAX_PROTECTED_PORTS: usize = 64;
pub const MAX_SCANNER_SKIP_PREFIXES: usize = 64;

fn default_cycle_interval_secs() -> u64 {
    DEFAULT_CYCLE_INTERVAL_SECS
}

fn default_max_auto_acts() -> u32 {
    DEFAULT_MAX_AUTO_ACTS_PER_CYCLE
}

fn default_scanner_threshold() -> u32 {
    DEFAULT_SCANNER_PORT_THRESHOLD
}

fn default_skip_prefixes() -> Vec<String> {
    DEFAULT_SCANNER_SKIP_PREFIXES
        .iter()
        .map(|p| (*p).to_string())
        .collect()
}

fn default_heuristics_enabled() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeatPolicy {
    pub allowlist: Vec<String>,
    pub protected_ports: Vec<u16>,
    pub allow_private_on_protected: bool,
    pub act_enabled: bool,
    pub suppression_ttl_secs: u64,
    /// Cycle may apply `BlockPeer` for protected-port probes only. Default off.
    #[serde(default)]
    pub auto_act_protected: bool,
    #[serde(default = "default_max_auto_acts")]
    pub max_auto_acts_per_cycle: u32,
    #[serde(default = "default_cycle_interval_secs")]
    pub cycle_interval_secs: u64,
    #[serde(default = "default_scanner_threshold")]
    pub scanner_port_threshold: u32,
    #[serde(default = "default_skip_prefixes")]
    pub scanner_skip_ip_prefixes: Vec<String>,
    /// Auto-hold only after `ioc_hash_match`. Default off. Needs `act_enabled`.
    #[serde(default)]
    pub auto_quarantine_hash: bool,
    /// Real-time heuristics / chains (observe-only). Default on; no floor confirm.
    #[serde(default = "default_heuristics_enabled")]
    pub heuristics_enabled: bool,
}

impl Default for SeatPolicy {
    fn default() -> Self {
        Self {
            allowlist: Vec::new(),
            protected_ports: DEFAULT_PROTECTED_PORTS.to_vec(),
            allow_private_on_protected: true,
            act_enabled: true,
            suppression_ttl_secs: 120,
            auto_act_protected: true,
            max_auto_acts_per_cycle: DEFAULT_MAX_AUTO_ACTS_PER_CYCLE,
            cycle_interval_secs: DEFAULT_CYCLE_INTERVAL_SECS,
            scanner_port_threshold: DEFAULT_SCANNER_PORT_THRESHOLD,
            scanner_skip_ip_prefixes: default_skip_prefixes(),
            auto_quarantine_hash: true,
            heuristics_enabled: true,
        }
    }
}

/// Factory default before 2026-09-22 product policy (observe-only on disk).
pub fn legacy_observe_only_factory() -> SeatPolicy {
    SeatPolicy {
        allowlist: Vec::new(),
        protected_ports: DEFAULT_PROTECTED_PORTS.to_vec(),
        allow_private_on_protected: true,
        act_enabled: false,
        suppression_ttl_secs: 120,
        auto_act_protected: false,
        max_auto_acts_per_cycle: DEFAULT_MAX_AUTO_ACTS_PER_CYCLE,
        cycle_interval_secs: DEFAULT_CYCLE_INTERVAL_SECS,
        scanner_port_threshold: DEFAULT_SCANNER_PORT_THRESHOLD,
        scanner_skip_ip_prefixes: default_skip_prefixes(),
        auto_quarantine_hash: false,
        heuristics_enabled: true,
    }
}

/// One-time migration for seats that still carry sparse pre-S4 factory JSON on disk.
///
/// A deliberate disarm writes full `policy.json` including S4 keys; that must not re-arm.
pub fn needs_product_armed_migration(raw_policy_json: &str, policy: &SeatPolicy) -> bool {
    policy == &legacy_observe_only_factory()
        && (!raw_policy_json.contains("\"auto_act_protected\"")
            || !raw_policy_json.contains("\"heuristics_enabled\""))
}

impl SeatPolicy {
    /// Validate and normalize after a policy.set op.
    pub fn validated(mut self) -> Result<Self, String> {
        if self.allowlist.len() > MAX_ALLOWLIST_ENTRIES {
            return Err(format!(
                "allowlist exceeds max {MAX_ALLOWLIST_ENTRIES} entries"
            ));
        }
        for ip in &self.allowlist {
            if ip.parse::<std::net::IpAddr>().is_err() {
                return Err(format!("allowlist entry is not a valid IP: {ip}"));
            }
        }
        if self.protected_ports.is_empty() {
            return Err("protected_ports must not be empty".into());
        }
        if self.protected_ports.len() > MAX_PROTECTED_PORTS {
            return Err(format!(
                "protected_ports exceeds max {MAX_PROTECTED_PORTS}"
            ));
        }
        self.protected_ports.sort_unstable();
        self.protected_ports.dedup();
        self.suppression_ttl_secs = self.suppression_ttl_secs.clamp(10, 3600);
        self.cycle_interval_secs = self.cycle_interval_secs.clamp(10, 300);
        self.max_auto_acts_per_cycle = self.max_auto_acts_per_cycle.clamp(1, 32);
        self.scanner_port_threshold = self.scanner_port_threshold.clamp(2, 64);
        if self.scanner_skip_ip_prefixes.len() > MAX_SCANNER_SKIP_PREFIXES {
            return Err(format!(
                "scanner_skip_ip_prefixes exceeds max {MAX_SCANNER_SKIP_PREFIXES}"
            ));
        }
        for prefix in &self.scanner_skip_ip_prefixes {
            if prefix.is_empty()
                || prefix.len() > 64
                || prefix.chars().any(|c| c.is_whitespace() || c == '/')
            {
                return Err(format!("invalid scanner_skip_ip_prefixes entry: {prefix}"));
            }
        }
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_protected_ports_are_ssh_only() {
        let policy = SeatPolicy::default();
        assert_eq!(policy.protected_ports, vec![22]);
        for workstation in [3389_u16, 5900, 5901, 5985, 5986] {
            assert!(
                !policy.protected_ports.contains(&workstation),
                "port {workstation} must not be an implicit default"
            );
        }
        assert!(policy.act_enabled);
        assert!(policy.auto_act_protected);
        assert!(policy.auto_quarantine_hash);
        assert_eq!(policy.cycle_interval_secs, DEFAULT_CYCLE_INTERVAL_SECS);
        assert_eq!(
            policy.max_auto_acts_per_cycle,
            DEFAULT_MAX_AUTO_ACTS_PER_CYCLE
        );
        assert_eq!(policy.scanner_port_threshold, DEFAULT_SCANNER_PORT_THRESHOLD);
        assert!(policy
            .scanner_skip_ip_prefixes
            .iter()
            .any(|p| p == "8.8."));
    }

    #[test]
    fn sparse_legacy_json_triggers_armed_migration() {
        let raw = r#"{
            "allowlist": [],
            "protected_ports": [22],
            "allow_private_on_protected": true,
            "act_enabled": false,
            "suppression_ttl_secs": 120
        }"#;
        let policy: SeatPolicy = serde_json::from_str(raw).expect("legacy policy");
        assert!(needs_product_armed_migration(raw, &policy));
    }

    #[test]
    fn explicit_disarm_json_does_not_re_arm() {
        let policy = legacy_observe_only_factory();
        let raw = serde_json::to_string_pretty(&policy).expect("policy json");
        assert!(!needs_product_armed_migration(&raw, &policy));
    }

    #[test]
    fn legacy_policy_json_defaults_s4_knobs_fail_closed() {
        let raw = r#"{
            "allowlist": [],
            "protected_ports": [22],
            "allow_private_on_protected": true,
            "act_enabled": false,
            "suppression_ttl_secs": 120
        }"#;
        let policy: SeatPolicy = serde_json::from_str(raw).expect("legacy policy");
        assert!(!policy.auto_act_protected);
        assert!(!policy.auto_quarantine_hash);
        assert!(policy.heuristics_enabled);
        assert_eq!(policy.max_auto_acts_per_cycle, 5);
        assert_eq!(policy.cycle_interval_secs, 30);
    }

    #[test]
    fn validated_clamps_cycle_interval() {
        let policy = SeatPolicy {
            cycle_interval_secs: 1,
            ..SeatPolicy::default()
        }
        .validated()
        .expect("valid");
        assert_eq!(policy.cycle_interval_secs, 10);
        let high = SeatPolicy {
            cycle_interval_secs: 9_999,
            ..SeatPolicy::default()
        }
        .validated()
        .expect("valid");
        assert_eq!(high.cycle_interval_secs, 300);
    }
}
