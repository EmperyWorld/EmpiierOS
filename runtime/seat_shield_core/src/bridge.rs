//! Bridge helpers for EmpiierOS Runtime paste (S2).
//!
//! `ShieldResponse` is wire-compatible with Runtime NDJSON v1
//! (`protocol_version`, `id`, `ok`, `status`, `data`, `error`).
//! Prefer forwarding `encode_line()` directly from `ShieldService::handle_bytes`.

use crate::protocol::ShieldResponse;
use crate::service::ShieldService;

/// Ops owned by Seat Shield (observe + gated acts).
pub const SEAT_SHIELD_OPS: &[&str] = &[
    "seat.shield.status",
    "seat.shield.events",
    "seat.shield.journal",
    "seat.shield.policy",
    "seat.shield.policy.set",
    "seat.shield.cycle",
    "seat.shield.observe",
    "seat.shield.feed_status",
    "seat.shield.feed.install",
    "seat.shield.act",
    "seat.shield.reset",
    "seat.shield.scan_path",
    "seat.shield.quarantine",
    "seat.shield.quarantine.list",
    "seat.shield.quarantine.restore",
];

#[must_use]
pub fn is_seat_shield_op(op: &str) -> bool {
    SEAT_SHIELD_OPS.contains(&op)
}

/// Handle one NDJSON request line; returns one NDJSON response line (with `\n`).
pub fn handle_runtime_line(service: &mut ShieldService, request_line: &[u8]) -> Vec<u8> {
    service.handle_bytes(request_line).encode_line()
}

/// Map shield status string onto Runtime-ish failure class (documentation aid).
#[must_use]
pub fn runtime_status_class(resp: &ShieldResponse) -> &'static str {
    if resp.ok {
        return "success";
    }
    match resp.status.as_str() {
        "not_available" | "unavailable" => "unavailable",
        "invalid_params" | "invalid_request" | "missing_confirmation" => "invalid",
        "denied" => "denied",
        _ => "failed",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::act::ActuatorBackend;
    use crate::service::ServiceConfig;
    use serde_json::Value;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn recognizes_ops() {
        assert!(is_seat_shield_op("seat.shield.status"));
        assert!(is_seat_shield_op("seat.shield.journal"));
        assert!(is_seat_shield_op("seat.shield.scan_path"));
        assert!(is_seat_shield_op("seat.shield.quarantine.list"));
        assert!(!is_seat_shield_op("security.integrity.status"));
    }

    #[test]
    fn handle_line_is_runtime_shaped() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("seat_bridge_{stamp}"));
        let mut svc = ShieldService::open(ServiceConfig {
            state_root: root.clone(),
            actuator: ActuatorBackend::Unavailable,
            helper_path: None,
            helper_prefix: Vec::new(),
            enable_proc_observe: false,
                    probe_helper_path: None,
        })
        .unwrap();
        let line = handle_runtime_line(
            &mut svc,
            br#"{"id":"b1","op":"seat.shield.status","params":{}}"#,
        );
        assert_eq!(line.last(), Some(&b'\n'));
        let value: Value = serde_json::from_slice(&line[..line.len() - 1]).unwrap();
        assert_eq!(value["protocol_version"], 1);
        assert_eq!(value["ok"], true);
        assert_eq!(value["id"], "b1");
        assert!(value["data"]["product"].is_string());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn garbage_ndjson_does_not_panic_or_exec() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("seat_bridge_fuzz_{stamp}"));
        let mut svc = ShieldService::open(ServiceConfig {
            state_root: root.clone(),
            actuator: ActuatorBackend::Unavailable,
            helper_path: None,
            helper_prefix: Vec::new(),
            enable_proc_observe: false,
                    probe_helper_path: None,
        })
        .unwrap();
        let junk: &[&[u8]] = &[
            b"",
            b"{",
            b"\n",
            b"{\"id\":\"x\"}",
            b"{\"id\":\"x\",\"op\":\"seat.shield.act\",\"params\":{\"action\":\"block_peer\",\"peer_ip\":\"1.2.3.4\"}}",
            b"{\"id\":\"x\",\"op\":\"not.a.real.op\",\"params\":{}}",
            &[0u8; 256],
        ];
        for line in junk {
            let out = handle_runtime_line(&mut svc, line);
            assert!(!out.is_empty(), "response must be a line");
            assert_eq!(out.last(), Some(&b'\n'));
        }
        let _ = std::fs::remove_dir_all(root);
    }
}
