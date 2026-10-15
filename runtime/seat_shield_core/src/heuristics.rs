//! Real-time heuristics (observe-only). Peer window → risk_milli → chain_*.
//!
//! Kill-switch: `SeatPolicy.heuristics_enabled`. Never changes auto-act eligibility.

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::event::{Severity, ShieldEvent};
use crate::observe::ObservedConn;
use crate::policy::SeatPolicy;
use crate::timeutil::now_epoch;

pub const PEER_WINDOW_CAP: usize = 32;
pub const PEER_WINDOW_TTL_SECS: u64 = 300;
/// Cumulative connection sightings in the TTL window that trip reconnect burst.
pub const RECONNECT_BURST_THRESHOLD: u32 = 8;

#[must_use]
pub fn heuristics_active(policy: &SeatPolicy) -> bool {
    policy.heuristics_enabled
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerFlags {
    #[serde(default)]
    pub scan: bool,
    #[serde(default)]
    pub probe: bool,
    #[serde(default)]
    pub ioc_peer: bool,
    #[serde(default)]
    pub ioc_hash: bool,
    #[serde(default)]
    pub tmp_exec: bool,
    #[serde(default)]
    pub bad_remote_port: bool,
    #[serde(default)]
    pub watch_hit: bool,
    #[serde(default)]
    pub burst: bool,
}

impl PeerFlags {
    #[must_use]
    pub fn class_count(&self) -> u32 {
        [
            self.scan,
            self.probe,
            self.ioc_peer,
            self.ioc_hash,
            self.tmp_exec,
            self.bad_remote_port,
            self.watch_hit,
            self.burst,
        ]
        .iter()
        .filter(|f| **f)
        .count() as u32
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerEntry {
    pub ip: String,
    pub last_seen: u64,
    #[serde(default)]
    pub flags: PeerFlags,
    #[serde(default)]
    pub risk_milli: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_chain_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_chain_kind: Option<String>,
    /// Sum of connection rows observed for this peer inside the TTL window.
    #[serde(default)]
    pub conn_hits: u32,
    #[serde(default)]
    pub burst_emitted: bool,
    /// Compound kinds already emitted in this window (≤1 of each).
    #[serde(default)]
    pub chains_emitted: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerWindow {
    pub peers: Vec<PeerEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChainTeaser {
    pub ip: String,
    pub kind: String,
    pub chain_id: String,
    pub risk_milli: u16,
}

#[must_use]
pub fn compute_risk_milli(flags: &PeerFlags, has_active_chain: bool) -> u16 {
    let mut r = 0_u32;
    if flags.probe {
        r += 400;
    }
    if flags.ioc_peer {
        r += 250;
    }
    if flags.ioc_hash || flags.watch_hit {
        r += 300;
    }
    if flags.scan {
        r += 150;
    }
    if flags.tmp_exec {
        r += 100;
    }
    if flags.bad_remote_port {
        r += 50;
    }
    if flags.burst {
        r += 75;
    }
    if has_active_chain {
        r += 100;
    }
    r.min(1000) as u16
}

fn make_chain_id(ip: &str, kind: &str) -> String {
    let mut hasher = DefaultHasher::new();
    ip.hash(&mut hasher);
    kind.hash(&mut hasher);
    format!("c{:016x}", hasher.finish())
}

fn apply_kind_flag(flags: &mut PeerFlags, kind: &str) {
    match kind {
        "possible_port_scan" => flags.scan = true,
        "peer_probe_protected_service" => flags.probe = true,
        "ioc_peer_match" => flags.ioc_peer = true,
        "ioc_hash_match" | "file_scan_hit" => flags.ioc_hash = true,
        "suspicious_exec_location" => flags.tmp_exec = true,
        "network_suspicious_port" => flags.bad_remote_port = true,
        "file_watch_seen" => flags.watch_hit = true,
        "heuristic_reconnect_burst" => flags.burst = true,
        _ => {}
    }
}

impl PeerWindow {
    pub fn expire(&mut self, now: u64) {
        self.peers
            .retain(|p| now.saturating_sub(p.last_seen) < PEER_WINDOW_TTL_SECS);
    }

    pub fn clear(&mut self) {
        self.peers.clear();
    }

    #[must_use]
    pub fn risk_for(&self, ip: &str) -> u16 {
        self.peers
            .iter()
            .find(|p| p.ip == ip)
            .map(|p| p.risk_milli)
            .unwrap_or(0)
    }

    #[must_use]
    pub fn open_chains_count(&self) -> usize {
        self.peers
            .iter()
            .filter(|p| p.last_chain_id.is_some())
            .count()
    }

    #[must_use]
    pub fn chain_teasers(&self, limit: usize) -> Vec<ChainTeaser> {
        let mut rows: Vec<&PeerEntry> = self
            .peers
            .iter()
            .filter(|p| p.last_chain_id.is_some())
            .collect();
        rows.sort_by(|a, b| {
            b.risk_milli
                .cmp(&a.risk_milli)
                .then_with(|| a.ip.cmp(&b.ip))
        });
        rows.into_iter()
            .take(limit)
            .filter_map(|p| {
                Some(ChainTeaser {
                    ip: p.ip.clone(),
                    kind: p.last_chain_kind.clone()?,
                    chain_id: p.last_chain_id.clone()?,
                    risk_milli: p.risk_milli,
                })
            })
            .collect()
    }

    fn upsert(&mut self, ip: &str, now: u64) -> &mut PeerEntry {
        if let Some(idx) = self.peers.iter().position(|p| p.ip == ip) {
            self.peers[idx].last_seen = now;
            return &mut self.peers[idx];
        }
        if self.peers.len() >= PEER_WINDOW_CAP {
            // Drop lowest risk, then oldest.
            self.peers.sort_by(|a, b| {
                a.risk_milli
                    .cmp(&b.risk_milli)
                    .then_with(|| a.last_seen.cmp(&b.last_seen))
            });
            self.peers.remove(0);
        }
        self.peers.push(PeerEntry {
            ip: ip.to_string(),
            last_seen: now,
            flags: PeerFlags::default(),
            risk_milli: 0,
            last_chain_id: None,
            last_chain_kind: None,
            conn_hits: 0,
            burst_emitted: false,
            chains_emitted: Vec::new(),
        });
        let len = self.peers.len();
        &mut self.peers[len - 1]
    }
}

fn pick_chain(flags: &PeerFlags, has_live_conn: bool) -> Option<(&'static str, Severity)> {
    if flags.scan && flags.probe {
        return Some(("chain_scan_then_probe", Severity::Critical));
    }
    if flags.ioc_peer && flags.probe {
        return Some(("chain_ioc_peer_probe", Severity::Critical));
    }
    if (flags.ioc_hash || flags.watch_hit) && has_live_conn {
        return Some(("chain_watch_ioc_live", Severity::Critical));
    }
    if flags.tmp_exec && has_live_conn {
        return Some(("chain_tmp_exec_net", Severity::High));
    }
    if flags.class_count() >= 3 {
        return Some(("chain_multi_signal", Severity::High));
    }
    None
}

fn chain_event(ip: &str, kind: &str, severity: Severity, chain_id: &str, flags: &PeerFlags) -> ShieldEvent {
    let summary = match kind {
        "chain_scan_then_probe" => format!("Chain: port scan then protected probe from {ip}"),
        "chain_ioc_peer_probe" => format!("Chain: IOC peer plus protected probe from {ip}"),
        "chain_watch_ioc_live" => format!("Chain: hash/watch hit with live conn from {ip}"),
        "chain_tmp_exec_net" => format!("Chain: temp exec with live network from {ip}"),
        "chain_multi_signal" => format!(
            "Chain: {} signal classes on {ip}",
            flags.class_count()
        ),
        other => format!("Chain {other} on {ip}"),
    };
    ShieldEvent {
        kind: kind.into(),
        severity,
        summary,
        entity_ip: Some(ip.to_string()),
        entity_process: None,
        entity_port: None,
        explain_why: "Heuristic correlation across the peer window (observe-only).".into(),
        explain_action: "Review in Journal; block/hold only via existing confirm-gated acts."
            .into(),
        confidence_milli: 850,
        tags: vec!["seat".into(), "chain".into(), "heuristic".into()],
        sequence: 0,
        epoch: 0,
        chain_id: Some(chain_id.to_string()),
    }
}

fn burst_event(ip: &str, hits: u32) -> ShieldEvent {
    ShieldEvent {
        kind: "heuristic_reconnect_burst".into(),
        severity: Severity::Medium,
        summary: format!("Reconnect burst: {hits} conn sightings from {ip}"),
        entity_ip: Some(ip.to_string()),
        entity_process: None,
        entity_port: None,
        explain_why: format!(
            "Same peer accumulated ≥{RECONNECT_BURST_THRESHOLD} connection rows inside the peer window."
        ),
        explain_action: "Observe-only; does not arm blocks.".into(),
        confidence_milli: 700,
        tags: vec!["seat".into(), "heuristic".into(), "burst".into()],
        sequence: 0,
        epoch: 0,
        chain_id: None,
    }
}

/// Update peer window from this cycle's atomics + conns; emit burst/chain events.
/// Mutates `atomics` to attach `chain_id` when a compound fires for that peer.
#[must_use]
pub fn run_heuristics(
    policy: &SeatPolicy,
    conns: &[ObservedConn],
    atomics: &mut [ShieldEvent],
    window: &mut PeerWindow,
) -> Vec<ShieldEvent> {
    if !heuristics_active(policy) {
        window.clear();
        return Vec::new();
    }
    let now = now_epoch();
    window.expire(now);

    let mut conn_counts: HashMap<String, u32> = HashMap::new();
    for conn in conns {
        if crate::cycle::is_allowlisted(policy, &conn.remote_ip) {
            continue;
        }
        if let Ok(ip) = conn.remote_ip.parse::<std::net::IpAddr>() {
            if crate::cycle::is_loopback_or_unspecified(&ip) {
                continue;
            }
        }
        *conn_counts.entry(conn.remote_ip.clone()).or_default() += 1;
    }

    for event in atomics.iter() {
        let Some(ip) = event.entity_ip.as_deref() else {
            continue;
        };
        if crate::cycle::is_allowlisted(policy, ip) {
            continue;
        }
        if let Ok(parsed) = ip.parse::<std::net::IpAddr>() {
            if crate::cycle::is_loopback_or_unspecified(&parsed) {
                continue;
            }
        }
        let entry = window.upsert(ip, now);
        apply_kind_flag(&mut entry.flags, &event.kind);
    }

    let mut out = Vec::new();

    for (ip, count) in &conn_counts {
        let entry = window.upsert(ip, now);
        entry.conn_hits = entry.conn_hits.saturating_add(*count);
        if entry.conn_hits >= RECONNECT_BURST_THRESHOLD && !entry.burst_emitted {
            entry.flags.burst = true;
            entry.burst_emitted = true;
            out.push(burst_event(ip, entry.conn_hits));
        }
    }

    // Optional RTT contribution to risk (heuristics on only; display RTT always from conn).
    let mut rtt_bonus: HashMap<String, u16> = HashMap::new();
    for conn in conns {
        if let Some(ms) = conn.rtt_ms {
            if ms >= 250 {
                let e = rtt_bonus.entry(conn.remote_ip.clone()).or_insert(0);
                *e = (*e).max(25);
            }
        }
    }

    // Recompute risk and emit ≤1 new compound per peer this cycle.
    let ips: Vec<String> = window.peers.iter().map(|p| p.ip.clone()).collect();
    for ip in ips {
        let has_live = conn_counts.contains_key(&ip);
        let Some(entry) = window.peers.iter_mut().find(|p| p.ip == ip) else {
            continue;
        };
        let chosen = pick_chain(&entry.flags, has_live);
        let mut new_chain_id = None;
        if let Some((kind, severity)) = chosen {
            if !entry.chains_emitted.iter().any(|k| k == kind) {
                let cid = make_chain_id(&ip, kind);
                entry.chains_emitted.push(kind.to_string());
                entry.last_chain_id = Some(cid.clone());
                entry.last_chain_kind = Some(kind.to_string());
                new_chain_id = Some(cid.clone());
                out.push(chain_event(&ip, kind, severity, &cid, &entry.flags));
            }
        }
        let has_chain = entry.last_chain_id.is_some();
        let mut risk = compute_risk_milli(&entry.flags, has_chain) as u32;
        risk = risk.saturating_add(u32::from(
            rtt_bonus.get(&ip).copied().unwrap_or(0),
        ));
        entry.risk_milli = risk.min(1000) as u16;
        if let Some(cid) = new_chain_id {
            for event in atomics.iter_mut() {
                if event.entity_ip.as_deref() == Some(ip.as_str()) && event.chain_id.is_none() {
                    event.chain_id = Some(cid.clone());
                }
            }
        }
    }

    // Cap peers after updates.
    if window.peers.len() > PEER_WINDOW_CAP {
        window.peers.sort_by(|a, b| {
            b.risk_milli
                .cmp(&a.risk_milli)
                .then_with(|| b.last_seen.cmp(&a.last_seen))
        });
        window.peers.truncate(PEER_WINDOW_CAP);
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Severity;

    fn atomic(kind: &str, ip: &str) -> ShieldEvent {
        ShieldEvent {
            kind: kind.into(),
            severity: Severity::High,
            summary: kind.into(),
            entity_ip: Some(ip.into()),
            entity_process: None,
            entity_port: None,
            explain_why: "t".into(),
            explain_action: "t".into(),
            confidence_milli: 800,
            tags: vec![],
            sequence: 0,
            epoch: now_epoch(),
            chain_id: None,
        }
    }

    fn conn(ip: &str, local: u16) -> ObservedConn {
        ObservedConn {
            remote_ip: ip.into(),
            local_port: local,
            remote_port: 40000,
            process_name: "x".into(),
            pid: 1,
            exe_path: None,
            exe_sha256: None,
                    rtt_ms: None,
        }
    }

    #[test]
    fn disabled_clears_and_returns_empty() {
        let mut policy = SeatPolicy::default();
        policy.heuristics_enabled = false;
        let mut window = PeerWindow {
            peers: vec![PeerEntry {
                ip: "198.51.100.20".into(),
                last_seen: now_epoch(),
                flags: PeerFlags {
                    scan: true,
                    ..PeerFlags::default()
                },
                risk_milli: 150,
                last_chain_id: None,
                last_chain_kind: None,
                conn_hits: 0,
                burst_emitted: false,
                chains_emitted: Vec::new(),
            }],
        };
        let mut atomics = vec![
            atomic("possible_port_scan", "198.51.100.20"),
            atomic("peer_probe_protected_service", "198.51.100.20"),
        ];
        let out = run_heuristics(&policy, &[], &mut atomics, &mut window);
        assert!(out.is_empty());
        assert!(window.peers.is_empty());
    }

    #[test]
    fn scan_then_probe_emits_chain() {
        let policy = SeatPolicy::default();
        let mut window = PeerWindow::default();
        let mut atomics = vec![
            atomic("possible_port_scan", "198.51.100.20"),
            atomic("peer_probe_protected_service", "198.51.100.20"),
        ];
        let conns = vec![conn("198.51.100.20", 22)];
        let out = run_heuristics(&policy, &conns, &mut atomics, &mut window);
        assert!(
            out.iter().any(|e| e.kind == "chain_scan_then_probe"),
            "{out:?}"
        );
        let chain = out
            .iter()
            .find(|e| e.kind == "chain_scan_then_probe")
            .unwrap();
        assert!(chain.chain_id.is_some());
        assert!(atomics.iter().all(|e| e.chain_id == chain.chain_id));
        assert!(window.risk_for("198.51.100.20") >= 400 + 150);
    }

    #[test]
    fn allowlisted_peer_never_enters_window() {
        let mut policy = SeatPolicy::default();
        policy.allowlist = vec!["198.51.100.20".into()];
        let mut window = PeerWindow::default();
        let mut atomics = vec![
            atomic("possible_port_scan", "198.51.100.20"),
            atomic("peer_probe_protected_service", "198.51.100.20"),
        ];
        let out = run_heuristics(&policy, &[conn("198.51.100.20", 22)], &mut atomics, &mut window);
        assert!(out.is_empty());
        assert!(window.peers.is_empty());
    }

    #[test]
    fn risk_formula_caps() {
        let flags = PeerFlags {
            scan: true,
            probe: true,
            ioc_peer: true,
            ioc_hash: true,
            tmp_exec: true,
            bad_remote_port: true,
            watch_hit: true,
            burst: true,
        };
        assert_eq!(compute_risk_milli(&flags, true), 1000);
    }
}
