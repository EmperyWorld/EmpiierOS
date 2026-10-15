//! H4 entry points: feed untrusted bytes into parsers we already have.
//!
//! These functions must not exec the floor helper or call `nft`.

use crate::act::ActuatorBackend;
use crate::bridge::handle_runtime_line;
use crate::fw::nft::{parse_nft_set_json, parse_nft_set_text};
use crate::observe::proc::probe_proc_net_line;
use crate::service::{ServiceConfig, ShieldService};
use std::path::PathBuf;

/// Parse nft text-list output. Errors and empty sets are success for fuzz.
pub fn nft_text(raw: &str) {
    let _ = parse_nft_set_text(raw);
}

/// Parse nft JSON-list output.
pub fn nft_json(raw: &str) {
    let _ = parse_nft_set_json(raw);
}

/// Parse one `/proc/net/tcp{,6}` data line.
pub fn proc_net_line(raw: &str, ipv6: bool) {
    probe_proc_net_line(raw, ipv6);
}

/// One Runtime NDJSON line. Caller must pass `Unavailable` so no helper exec.
pub fn runtime_line(service: &mut ShieldService, raw: &[u8]) {
    let out = handle_runtime_line(service, raw);
    assert!(!out.is_empty(), "H4: handle_runtime_line must emit a line");
    assert_eq!(out.last(), Some(&b'\n'));
}

/// Open a service that cannot exec `seat-shield-fw`.
pub fn unavailable_service(state_root: PathBuf) -> ShieldService {
    ShieldService::open(ServiceConfig {
        state_root,
        actuator: ActuatorBackend::Unavailable,
        helper_path: None,
        helper_prefix: Vec::new(),
        enable_proc_observe: false,
                probe_helper_path: None,
        })
    .expect("H4 unavailable service")
}
