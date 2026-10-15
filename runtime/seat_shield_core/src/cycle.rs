//! Observe/decide cycle — pure policy; no host I/O.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::net::IpAddr;

use crate::event::{Severity, ShieldEvent, SuppressionCache};
use crate::feed::IocFeed;
use crate::observe::ObservedConn;
use crate::ota_observe::is_ota_portal_vip;
use crate::policy::{SeatPolicy, SUSPICIOUS_EXEC_PREFIXES, SUSPICIOUS_REMOTE_PORTS};
use crate::timeutil::now_epoch;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActKind {
    PreviewBlock,
    BlockPeer,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActDecision {
    pub act: ActKind,
    pub peer_ip: String,
    pub reason: String,
    pub allow_private: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CycleOutcome {
    pub events: Vec<ShieldEvent>,
    pub acts: Vec<ActDecision>,
    pub suppressed_duplicates: u32,
}

fn parse_ip(raw: &str) -> Option<IpAddr> {
    raw.parse().ok()
}

pub(crate) fn is_allowlisted(policy: &SeatPolicy, ip: &str) -> bool {
    policy.allowlist.iter().any(|entry| entry == ip)
}

pub(crate) fn is_loopback_or_unspecified(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => v4.is_loopback() || v4.is_unspecified(),
        IpAddr::V6(v6) => v6.is_loopback() || v6.is_unspecified(),
    }
}

pub(crate) fn is_private_or_link_local(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => v4.is_private() || v4.is_link_local(),
        IpAddr::V6(v6) => {
            (v6.segments()[0] & 0xfe00) == 0xfc00 || (v6.segments()[0] & 0xffc0) == 0xfe80
        }
    }
}

fn suspicious_exec(path: &str) -> bool {
    SUSPICIOUS_EXEC_PREFIXES
        .iter()
        .any(|prefix| path.starts_with(prefix))
}

fn stamp(mut event: ShieldEvent) -> ShieldEvent {
    if event.epoch == 0 {
        event.epoch = now_epoch();
    }
    event
}

fn event_protected_probe(conn: &ObservedConn) -> ShieldEvent {
    stamp(ShieldEvent {
        kind: "peer_probe_protected_service".into(),
        severity: Severity::Critical,
        summary: format!(
            "Peer {} probed protected local port {}",
            conn.remote_ip, conn.local_port
        ),
        entity_ip: Some(conn.remote_ip.clone()),
        entity_process: Some(conn.process_name.clone()),
        entity_port: Some(conn.local_port),
        explain_why: "Remote peer connected to a seat-protected local service port.".into(),
        explain_action: "Review peer; apply allowlisted seat.shield.act only when armed.".into(),
        confidence_milli: 920,
        tags: vec![
            "seat".into(),
            "protected_service".into(),
            "network".into(),
        ],
        sequence: 0,
        epoch: 0,
        chain_id: None,
    })
}

fn event_suspicious_exec(conn: &ObservedConn, path: &str) -> ShieldEvent {
    stamp(ShieldEvent {
        kind: "suspicious_exec_location".into(),
        severity: Severity::High,
        summary: format!("Executable from temporary path: {path}"),
        entity_ip: Some(conn.remote_ip.clone()),
        entity_process: Some(conn.process_name.clone()),
        entity_port: Some(conn.local_port),
        explain_why: "Process image resolved under a temporary filesystem prefix.".into(),
        explain_action: "Investigate process tree; quarantine is a later floor wave.".into(),
        confidence_milli: 880,
        tags: vec!["seat".into(), "filesystem".into(), "process".into()],
        sequence: 0,
        epoch: 0,
        chain_id: None,
    })
}

fn event_ioc_peer(conn: &ObservedConn, feed_version: &str) -> ShieldEvent {
    stamp(ShieldEvent {
        kind: "ioc_peer_match".into(),
        severity: Severity::High,
        summary: format!("Peer {} matches sealed IOC feed", conn.remote_ip),
        entity_ip: Some(conn.remote_ip.clone()),
        entity_process: Some(conn.process_name.clone()),
        entity_port: Some(conn.local_port),
        explain_why: format!("Remote IP is listed in sealed feed version {feed_version}."),
        explain_action: "Investigate; act only when seat.shield is armed.".into(),
        confidence_milli: 900,
        tags: vec!["seat".into(), "ioc".into(), "network".into()],
        sequence: 0,
        epoch: 0,
        chain_id: None,
    })
}

fn event_ioc_hash(conn: &ObservedConn, digest: &str, feed_version: &str) -> ShieldEvent {
    stamp(ShieldEvent {
        kind: "ioc_hash_match".into(),
        severity: Severity::Critical,
        summary: format!("Exec hash match for {}", conn.process_name),
        entity_ip: Some(conn.remote_ip.clone()),
        entity_process: Some(conn.process_name.clone()),
        entity_port: Some(conn.local_port),
        explain_why: format!("SHA-256 matches sealed feed version {feed_version} (digest redacted length={}).", digest.len()),
        explain_action: "Isolate seat investigation; quarantine later floor wave.".into(),
        confidence_milli: 990,
        tags: vec!["seat".into(), "ioc".into(), "hash".into()],
        sequence: 0,
        epoch: 0,
        chain_id: None,
    })
}

fn event_suspicious_remote_port(conn: &ObservedConn) -> ShieldEvent {
    stamp(ShieldEvent {
        kind: "network_suspicious_port".into(),
        severity: Severity::Medium,
        summary: format!(
            "Suspicious remote port {} toward {}",
            conn.remote_port, conn.remote_ip
        ),
        entity_ip: Some(conn.remote_ip.clone()),
        entity_process: Some(conn.process_name.clone()),
        entity_port: Some(conn.remote_port),
        explain_why: "Connection targets a known suspicious remote port pattern.".into(),
        explain_action: "Event only. S4 does not auto-block behavioral signals.".into(),
        confidence_milli: 720,
        tags: vec!["seat".into(), "network".into(), "behavioral".into()],
        sequence: 0,
        epoch: 0,
        chain_id: None,
    })
}

fn event_possible_port_scan(ip: &str, distinct_local_ports: usize) -> ShieldEvent {
    let ports_u16 = u16::try_from(distinct_local_ports).unwrap_or(u16::MAX);
    let confidence = 700_u16.saturating_add(ports_u16.saturating_mul(30)).min(990);
    stamp(ShieldEvent {
        kind: "possible_port_scan".into(),
        severity: Severity::High,
        summary: format!(
            "Possible scan from {ip}: {distinct_local_ports} distinct local ports"
        ),
        entity_ip: Some(ip.to_string()),
        entity_process: None,
        entity_port: None,
        explain_why: "One remote endpoint touched many distinct local ports in one cycle.".into(),
        explain_action: "Event only. S4 does not auto-block scanner signals.".into(),
        confidence_milli: confidence,
        tags: vec!["seat".into(), "network".into(), "behavioral".into()],
        sequence: 0,
        epoch: 0,
        chain_id: None,
    })
}

fn scanner_skip(policy: &SeatPolicy, ip: &str) -> bool {
    policy
        .scanner_skip_ip_prefixes
        .iter()
        .any(|prefix| ip.starts_with(prefix.as_str()))
}

fn propose_block(policy: &SeatPolicy, conn: &ObservedConn, allow_private: bool) -> ActDecision {
    if policy.act_enabled {
        ActDecision {
            act: ActKind::BlockPeer,
            peer_ip: conn.remote_ip.clone(),
            reason: format!(
                "Protected service intrusion: local_port={}",
                conn.local_port
            ),
            allow_private,
        }
    } else {
        ActDecision {
            act: ActKind::PreviewBlock,
            peer_ip: conn.remote_ip.clone(),
            reason: format!(
                "preview: protected port {} probe from {}",
                conn.local_port, conn.remote_ip
            ),
            allow_private,
        }
    }
}

#[must_use]
pub fn run_cycle(policy: &SeatPolicy, conns: &[ObservedConn]) -> CycleOutcome {
    run_cycle_with_feed(policy, conns, None)
}

#[must_use]
pub fn run_cycle_with_feed(
    policy: &SeatPolicy,
    conns: &[ObservedConn],
    feed: Option<&IocFeed>,
) -> CycleOutcome {
    let mut events = Vec::new();
    let mut acts = Vec::new();
    let mut local_ports_by_ip: HashMap<String, HashSet<u16>> = HashMap::new();

    for conn in conns {
        let Some(ip) = parse_ip(&conn.remote_ip) else {
            continue;
        };
        if is_loopback_or_unspecified(&ip) {
            continue;
        }
        if is_allowlisted(policy, &conn.remote_ip) {
            continue;
        }
        // Portal VIP: OTA helper path — never propose BlockPeer / PreviewBlock.
        if is_ota_portal_vip(&conn.remote_ip) {
            continue;
        }

        local_ports_by_ip
            .entry(conn.remote_ip.clone())
            .or_default()
            .insert(conn.local_port);
        if SUSPICIOUS_REMOTE_PORTS.contains(&conn.remote_port) {
            events.push(event_suspicious_remote_port(conn));
        }

        if let Some(feed) = feed {
            if feed.ips.iter().any(|x| x == &conn.remote_ip) {
                events.push(event_ioc_peer(conn, &feed.version));
            }
            if let Some(digest) = conn.exe_sha256.as_deref() {
                if feed
                    .hashes
                    .iter()
                    .any(|h| h.eq_ignore_ascii_case(digest))
                {
                    events.push(event_ioc_hash(conn, digest, &feed.version));
                }
            }
        }

        if let Some(path) = conn.exe_path.as_deref() {
            if suspicious_exec(path) {
                events.push(event_suspicious_exec(conn, path));
            }
        }

        if !policy.protected_ports.contains(&conn.local_port) {
            continue;
        }

        let private = is_private_or_link_local(&ip);
        if private && !policy.allow_private_on_protected {
            continue;
        }

        events.push(event_protected_probe(conn));
        acts.push(propose_block(policy, conn, private));
    }

    let threshold = usize::try_from(policy.scanner_port_threshold).unwrap_or(usize::MAX);
    for (ip, ports) in local_ports_by_ip {
        if ports.len() < threshold || scanner_skip(policy, &ip) {
            continue;
        }
        events.push(event_possible_port_scan(&ip, ports.len()));
    }

    let mut cache = SuppressionCache::new(policy.suppression_ttl_secs);
    let (events, suppressed_duplicates) = cache.filter(events);

    CycleOutcome {
        events,
        acts,
        suppressed_duplicates,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conn(ip: &str, local_port: u16) -> ObservedConn {
        ObservedConn {
            remote_ip: ip.into(),
            local_port,
            remote_port: 50123,
            process_name: "sshd".into(),
            pid: 1,
            exe_path: None,
            exe_sha256: None,
                    rtt_ms: None,
        }
    }

    #[test]
    fn whitelist_and_loopback_and_preview() {
        let armed = SeatPolicy {
            act_enabled: true,
            allowlist: vec!["198.51.100.7".into()],
            ..SeatPolicy::default()
        };
        assert!(run_cycle(&armed, &[conn("198.51.100.7", 22)]).acts.is_empty());
        assert!(run_cycle(&armed, &[conn("127.0.0.1", 22)]).acts.is_empty());
        let preview = run_cycle(&SeatPolicy::default(), &[conn("198.51.100.9", 22)]);
        assert_eq!(preview.acts[0].act, ActKind::PreviewBlock);
        let lan = run_cycle(
            &SeatPolicy {
                act_enabled: true,
                ..SeatPolicy::default()
            },
            &[conn("192.168.1.77", 22)],
        );
        assert_eq!(lan.acts[0].act, ActKind::BlockPeer);
        assert!(lan.acts[0].allow_private);
    }

    #[test]
    fn ota_portal_vip_never_proposes_block() {
        let armed = SeatPolicy {
            act_enabled: true,
            allow_private_on_protected: true,
            ..SeatPolicy::default()
        };
        let out = run_cycle(&armed, &[conn(crate::ota_observe::OTA_PORTAL_VIP, 22)]);
        assert!(out.acts.is_empty(), "VIP must not get BlockPeer/PreviewBlock");
        assert!(out.events.is_empty(), "VIP exempt from protected-probe events");
    }

    /// H02/H03/H06/H07/H10 gold — harvested intent from Monitoring pytest, not Python ports.
    #[test]
    fn gold_harvest_false_positive_guards() {
        let armed = SeatPolicy {
            act_enabled: true,
            allowlist: vec!["203.0.113.10".into()],
            allow_private_on_protected: false,
            ..SeatPolicy::default()
        };

        // H02: allowlist never auto-acted even on SSH.
        let wl = run_cycle(&armed, &[conn("203.0.113.10", 22)]);
        assert!(wl.events.is_empty());
        assert!(wl.acts.is_empty());

        // Loopback + unspecified ignored.
        assert!(run_cycle(&armed, &[conn("127.0.0.1", 22)]).acts.is_empty());
        assert!(run_cycle(&armed, &[conn("0.0.0.0", 22)]).acts.is_empty());
        assert!(run_cycle(&armed, &[conn("::1", 22)]).acts.is_empty());

        // Non-protected port → no protected-service act (H03 SSH-focused MVP).
        let http = run_cycle(&armed, &[conn("198.51.100.50", 80)]);
        assert!(http.acts.is_empty());
        assert!(!http
            .events
            .iter()
            .any(|e| e.kind == "peer_probe_protected_service"));

        // Private LAN without allow_private_on_protected → no act (H06).
        let lan_denied = run_cycle(&armed, &[conn("10.0.0.9", 22)]);
        assert!(lan_denied.acts.is_empty());

        // Same LAN when private allowed → BlockPeer (autonomous protected-service gold).
        let lan_ok = run_cycle(
            &SeatPolicy {
                act_enabled: true,
                allow_private_on_protected: true,
                ..SeatPolicy::default()
            },
            &[conn("192.168.1.77", 22)],
        );
        assert_eq!(lan_ok.acts.len(), 1);
        assert_eq!(lan_ok.acts[0].act, ActKind::BlockPeer);
        assert!(lan_ok
            .events
            .iter()
            .any(|e| e.kind == "peer_probe_protected_service"));

        // H07: observe-only → PreviewBlock only.
        let preview = run_cycle(
            &SeatPolicy::default(),
            &[conn("198.51.100.88", 22)],
        );
        assert_eq!(preview.acts[0].act, ActKind::PreviewBlock);

        // H10: suspicious exec prefixes.
        let mut tmp = conn("198.51.100.60", 443);
        tmp.exe_path = Some("/tmp/evil".into());
        tmp.process_name = "evil".into();
        let sus = run_cycle(&SeatPolicy::default(), &[tmp]);
        assert!(sus.events.iter().any(|e| e.kind == "suspicious_exec_location"));

        let mut shm = conn("198.51.100.61", 443);
        shm.exe_path = Some("/dev/shm/dropper".into());
        let sus2 = run_cycle(&SeatPolicy::default(), &[shm]);
        assert!(sus2
            .events
            .iter()
            .any(|e| e.kind == "suspicious_exec_location"));
    }

    #[test]
    fn gold_harvest_ioc_and_suppression() {
        use crate::feed::IocFeed;

        let feed = IocFeed {
            version: "gold-1".into(),
            ips: vec!["198.51.100.200".into()],
            hashes: vec![
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            ],
            digest: "gold-digest".into(),
        };
        let mut c = conn("198.51.100.200", 443);
        c.exe_sha256 =
            Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into());
        let out = run_cycle_with_feed(&SeatPolicy::default(), &[c.clone(), c], Some(&feed));
        assert!(out.events.iter().any(|e| e.kind == "ioc_peer_match"));
        assert!(out.events.iter().any(|e| e.kind == "ioc_hash_match"));
        // Duplicate conn same cycle → suppression may drop repeats.
        assert!(out.suppressed_duplicates >= 1 || out.events.len() >= 2);
    }

    #[test]
    fn gold_s4_behavioral_events_never_act() {
        let mut suspicious = conn("198.51.100.9", 443);
        suspicious.remote_port = 4444;
        let one = run_cycle(&SeatPolicy::default(), &[suspicious]);
        assert!(one
            .events
            .iter()
            .any(|e| e.kind == "network_suspicious_port"));
        assert!(one.acts.is_empty());

        let scan: Vec<ObservedConn> = (0_u16..6)
            .map(|offset| {
                let mut row = conn("198.51.100.50", 2000 + offset);
                row.remote_port = 80;
                row
            })
            .collect();
        let scanned = run_cycle(
            &SeatPolicy {
                act_enabled: true,
                ..SeatPolicy::default()
            },
            &scan,
        );
        assert_eq!(
            scanned
                .events
                .iter()
                .filter(|e| e.kind == "possible_port_scan")
                .count(),
            1
        );
        assert!(
            scanned.acts.is_empty(),
            "behavioral scan must not propose BlockPeer"
        );

        let benign: Vec<ObservedConn> = (0_u16..6)
            .map(|offset| conn("8.8.8.8", 2100 + offset))
            .collect();
        let skipped = run_cycle(&SeatPolicy::default(), &benign);
        assert!(!skipped
            .events
            .iter()
            .any(|e| e.kind == "possible_port_scan"));

        let armed = SeatPolicy {
            act_enabled: true,
            allowlist: vec!["198.51.100.50".into()],
            ..SeatPolicy::default()
        };
        let allow = run_cycle(&armed, &scan);
        assert!(allow.events.is_empty());
        assert!(allow.acts.is_empty());
    }
}
