//! EmpiierOS Seat Shield — Runtime bridge.
//!
//! S2: Unavailable actuator (observe-only honesty).
//! S3: Helper actuator when pinned floor binary is present on disk.

use std::path::{Path, PathBuf};

use seat_shield_core::{
    handle_runtime_line, is_seat_shield_op, ActuatorBackend, ServiceConfig, ShieldService,
};

use crate::protocol::RuntimeResponse;

pub const HELPER_LIBEXEC: &str = "/usr/libexec/empiieros-seat-shield-fw";
pub const HELPER_LOCAL_SBIN: &str = "/usr/local/sbin/empiieros-seat-shield-fw";
pub const PROBE_LIBEXEC: &str = "/usr/libexec/empiieros-seat-shield-probe";
pub const PROBE_LOCAL_SBIN: &str = "/usr/local/sbin/empiieros-seat-shield-probe";

pub fn handles(op: &str) -> bool {
    is_seat_shield_op(op)
}

fn nft_helper_prefix() -> Vec<String> {
    vec![
        "--backend".into(),
        "nft".into(),
        "--nft-bin".into(),
        "/usr/sbin/nft".into(),
        "--nft-family".into(),
        "inet".into(),
        "--nft-table".into(),
        "seat_shield".into(),
        "--nft-set".into(),
        "blocked4".into(),
        "--nft-set6".into(),
        "blocked6".into(),
    ]
}

/// Resolve pinned helper binary. Missing → observe-only Unavailable.
pub fn resolve_helper_bin() -> Option<PathBuf> {
    #[cfg(test)]
    {
        if let Ok(raw) = std::env::var("EMPIIEROS_SEAT_SHIELD_FW") {
            let path = PathBuf::from(raw);
            if path.is_absolute() && path.is_file() {
                return Some(path);
            }
        }
    }
    for candidate in [HELPER_LIBEXEC, HELPER_LOCAL_SBIN] {
        let path = PathBuf::from(candidate);
        if path.is_file() {
            return Some(path);
        }
    }
    None
}

/// Resolve inet_diag probe helper. Missing → `/proc` observe only.
pub fn resolve_probe_bin() -> Option<PathBuf> {
    #[cfg(test)]
    {
        if let Ok(raw) = std::env::var("EMPIIEROS_SEAT_SHIELD_PROBE") {
            let path = PathBuf::from(raw);
            if path.is_absolute() && path.is_file() {
                return Some(path);
            }
        }
    }
    for candidate in [PROBE_LIBEXEC, PROBE_LOCAL_SBIN] {
        let path = PathBuf::from(candidate);
        if path.is_file() {
            return Some(path);
        }
    }
    None
}

#[cfg(test)]
pub(crate) static HELPER_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn open_service_with_prefix(
    state_root: &Path,
    prefix_override: Option<Vec<String>>,
) -> Result<ShieldService, String> {
    let probe_helper_path = resolve_probe_bin();
    match resolve_helper_bin() {
        Some(helper_path) => {
            let helper_prefix = prefix_override.unwrap_or_else(nft_helper_prefix);
            ShieldService::open(ServiceConfig {
                state_root: state_root.to_path_buf(),
                actuator: ActuatorBackend::Helper,
                helper_path: Some(helper_path),
                helper_prefix,
                enable_proc_observe: true,
                probe_helper_path,
            })
            .map_err(|e| e.to_string())
        }
        None => ShieldService::open(ServiceConfig {
            state_root: state_root.to_path_buf(),
            actuator: ActuatorBackend::Unavailable,
            helper_path: None,
            helper_prefix: Vec::new(),
            enable_proc_observe: true,
            probe_helper_path,
        })
        .map_err(|e| e.to_string()),
    }
}

pub fn dispatch(state_root: &Path, request_bytes: &[u8]) -> RuntimeResponse {
    dispatch_inner(state_root, request_bytes, None)
}

fn dispatch_inner(
    state_root: &Path,
    request_bytes: &[u8],
    prefix_override: Option<Vec<String>>,
) -> RuntimeResponse {
    let mut service = match open_service_with_prefix(state_root, prefix_override) {
        Ok(svc) => svc,
        Err(error) => {
            return RuntimeResponse::failure(
                "",
                "failed",
                "seat_shield_open_failed",
                error,
            );
        }
    };

    let encoded = handle_runtime_line(&mut service, request_bytes);
    let trimmed = encoded.strip_suffix(b"\n").unwrap_or(encoded.as_slice());
    match serde_json::from_slice::<RuntimeResponse>(trimmed) {
        Ok(response) => response,
        Err(error) => RuntimeResponse::failure(
            "",
            "internal_error",
            "seat_shield_response_parse_failed",
            error.to_string(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tmp_root() -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "empiieros-seat-shield-{}-{stamp}",
            std::process::id()
        ))
    }

    #[test]
    fn default_status_unavailable_when_helper_missing() {
        let _guard = super::HELPER_ENV_LOCK.lock().expect("helper env lock");
        // Ensure test env does not accidentally pin a helper.
        std::env::remove_var("EMPIIEROS_SEAT_SHIELD_FW");
        let root = tmp_root();
        let response = dispatch(
            &root,
            br#"{"id":"ss1","op":"seat.shield.status","params":{}}"#,
        );
        assert!(response.ok, "{response:?}");
        assert_eq!(response.data["mode"], "unavailable");
        assert_eq!(response.data["backend"], "unavailable");
        assert_eq!(response.data["integrity_twin"], "security.integrity");
        assert_eq!(response.data["integrity_twin_state"], "disabled");
        assert_eq!(response.data["act_enabled"], true);
        assert_eq!(response.data["auto_act_protected"], true);
        assert_eq!(response.data["auto_quarantine_hash"], true);
        assert_eq!(response.data["quarantine_count"], 0);
        assert_eq!(response.data["watch_pending"], 0);
        assert_eq!(response.data["last_watch_drained"], 0);
        assert_eq!(response.data["protected_ports"][0], 22);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn act_is_not_available_without_helper() {
        let _guard = super::HELPER_ENV_LOCK.lock().expect("helper env lock");
        std::env::remove_var("EMPIIEROS_SEAT_SHIELD_FW");
        let root = tmp_root();
        let enable = dispatch(
            &root,
            br#"{"id":"ss2","op":"seat.shield.policy.set","params":{"act_enabled":true,"confirmation":"ENABLE_ACTS"}}"#,
        );
        assert!(enable.ok, "{enable:?}");
        let act = dispatch(
            &root,
            br#"{"id":"ss3","op":"seat.shield.act","params":{"action":"block_peer","peer_ip":"198.51.100.20","confirmation":"BLOCK_PEER"}}"#,
        );
        assert!(!act.ok);
        assert_eq!(act.status, "not_available");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn cycle_emits_redacted_event() {
        let _guard = super::HELPER_ENV_LOCK.lock().expect("helper env lock");
        std::env::remove_var("EMPIIEROS_SEAT_SHIELD_FW");
        let root = tmp_root();
        let cycle = dispatch(
            &root,
            br#"{"id":"ss4","op":"seat.shield.cycle","params":{"connections":[{"remote_ip":"198.51.100.20","local_port":22,"process_name":"sshd","pid":1}]}}"#,
        );
        assert!(cycle.ok, "{cycle:?}");
        assert!(
            cycle.data["events"]
                .as_array()
                .is_some_and(|events| !events.is_empty()),
            "{cycle:?}"
        );
        assert!(
            cycle.data["applied_acts"]
                .as_array()
                .is_some_and(|acts| acts.is_empty()),
            "{cycle:?}"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn auto_act_protected_without_act_enabled_is_denied() {
        let _guard = super::HELPER_ENV_LOCK.lock().expect("helper env lock");
        std::env::remove_var("EMPIIEROS_SEAT_SHIELD_FW");
        let root = tmp_root();
        let disarm = dispatch(
            &root,
            br#"{"id":"ss5a","op":"seat.shield.policy.set","params":{"act_enabled":false}}"#,
        );
        assert!(disarm.ok, "{disarm:?}");
        let denied = dispatch(
            &root,
            br#"{"id":"ss5","op":"seat.shield.policy.set","params":{"auto_act_protected":true}}"#,
        );
        assert!(!denied.ok, "{denied:?}");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn scan_path_outside_roots_is_denied() {
        let _guard = super::HELPER_ENV_LOCK.lock().expect("helper env lock");
        std::env::remove_var("EMPIIEROS_SEAT_SHIELD_FW");
        let root = tmp_root();
        let denied = dispatch(
            &root,
            br#"{"id":"ss6","op":"seat.shield.scan_path","params":{"path":"/etc/passwd"}}"#,
        );
        assert!(!denied.ok, "{denied:?}");
        assert_eq!(denied.status, "failed");
        assert_eq!(
            denied.error.as_ref().map(|e| e.code.as_str()),
            Some("act_denied")
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn helper_state_backend_can_arm_and_block() {
        let _guard = super::HELPER_ENV_LOCK.lock().expect("helper env lock");
        let root = tmp_root();
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("seat_shield_core");
        // Pin target-dir under tmp so ambient CARGO_TARGET_DIR (packager) cannot
        // hide the binary from the path we pass via EMPIIEROS_SEAT_SHIELD_FW.
        let helper_target = root.join("helper-target");
        let helper = helper_target.join("debug/seat-shield-fw");
        let status = Command::new("cargo")
            .args([
                "build",
                "--manifest-path",
                manifest.join("Cargo.toml").to_str().unwrap(),
                "--target-dir",
                helper_target.to_str().unwrap(),
                "--bin",
                "seat-shield-fw",
                "-q",
            ])
            .env_remove("CARGO_TARGET_DIR")
            .status()
            .expect("cargo build helper");
        assert!(status.success());
        assert!(helper.is_file(), "missing {}", helper.display());

        std::env::set_var("EMPIIEROS_SEAT_SHIELD_FW", helper.to_str().unwrap());
        let prefix = vec!["--backend".into(), "state".into()];
        let enable = dispatch_inner(
            &root,
            br#"{"id":"h1","op":"seat.shield.policy.set","params":{"act_enabled":true,"confirmation":"ENABLE_ACTS"}}"#,
            Some(prefix.clone()),
        );
        assert!(enable.ok, "{enable:?}");
        let status = dispatch_inner(
            &root,
            br#"{"id":"h2","op":"seat.shield.status","params":{}}"#,
            Some(prefix.clone()),
        );
        assert_eq!(status.data["backend"], "helper");
        assert_eq!(status.data["mode"], "armed");
        let block = dispatch_inner(
            &root,
            br#"{"id":"h3","op":"seat.shield.act","params":{"action":"block_peer","peer_ip":"198.51.100.20","confirmation":"BLOCK_PEER"}}"#,
            Some(prefix.clone()),
        );
        assert!(block.ok, "{block:?}");
        assert_eq!(block.data["backend"], "helper");
        let after = dispatch_inner(
            &root,
            br#"{"id":"h4","op":"seat.shield.status","params":{}}"#,
            Some(prefix),
        );
        assert!(after.ok, "{after:?}");
        assert_eq!(after.data["blocked_count"], 1);
        let peers = after.data["blocked_peers"]
            .as_array()
            .expect("blocked_peers array");
        assert!(
            peers.iter().any(|p| p.as_str() == Some("198.51.100.20")),
            "{after:?}"
        );
        std::env::remove_var("EMPIIEROS_SEAT_SHIELD_FW");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn helper_auto_act_blocks_protected_probe_and_status_stays_inside_ttl() {
        let _guard = super::HELPER_ENV_LOCK.lock().expect("helper env lock");
        let root = tmp_root();
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("seat_shield_core");
        let helper_target = root.join("helper-target");
        let helper = helper_target.join("debug/seat-shield-fw");
        let status = Command::new("cargo")
            .args([
                "build",
                "--manifest-path",
                manifest.join("Cargo.toml").to_str().unwrap(),
                "--target-dir",
                helper_target.to_str().unwrap(),
                "--bin",
                "seat-shield-fw",
                "-q",
            ])
            .env_remove("CARGO_TARGET_DIR")
            .status()
            .expect("cargo build helper");
        assert!(status.success());
        std::env::set_var("EMPIIEROS_SEAT_SHIELD_FW", helper.to_str().unwrap());
        let prefix = vec!["--backend".into(), "state".into()];
        let enable = dispatch_inner(
            &root,
            br#"{"id":"a1","op":"seat.shield.policy.set","params":{"act_enabled":true,"auto_act_protected":true,"confirmation":"ENABLE_ACTS"}}"#,
            Some(prefix.clone()),
        );
        assert!(enable.ok, "{enable:?}");
        let cycle = dispatch_inner(
            &root,
            br#"{"id":"a2","op":"seat.shield.cycle","params":{"connections":[{"remote_ip":"198.51.100.20","local_port":22,"process_name":"sshd","pid":1}]}}"#,
            Some(prefix.clone()),
        );
        assert!(cycle.ok, "{cycle:?}");
        assert_eq!(cycle.data["note"], "protected auto-act applied");
        assert_eq!(cycle.data["applied_acts"][0]["peer_ip"], "198.51.100.20");
        let status1 = dispatch_inner(
            &root,
            br#"{"id":"a3","op":"seat.shield.status","params":{}}"#,
            Some(prefix.clone()),
        );
        assert_eq!(status1.data["blocked_count"], 1);
        assert_eq!(status1.data["auto_act_protected"], true);
        let epoch = status1.data["last_cycle_epoch"].as_u64().unwrap();
        let status2 = dispatch_inner(
            &root,
            br#"{"id":"a4","op":"seat.shield.status","params":{}}"#,
            Some(prefix),
        );
        assert_eq!(status2.data["last_cycle_epoch"].as_u64().unwrap(), epoch);
        std::env::remove_var("EMPIIEROS_SEAT_SHIELD_FW");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn status_drains_watch_journal_without_holding_clean_tmp() {
        let _guard = super::HELPER_ENV_LOCK.lock().expect("helper env lock");
        std::env::remove_var("EMPIIEROS_SEAT_SHIELD_FW");
        let root = tmp_root();
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let sample = PathBuf::from(format!("/tmp/seat_s5b_rt_{stamp}"));
        fs::write(&sample, b"os-s5b-clean").unwrap();
        let journal = root.join("seat_shield").join("watch.jsonl");
        fs::create_dir_all(journal.parent().expect("watch parent")).unwrap();
        fs::write(
            &journal,
            format!(
                "{}\n",
                serde_json::json!({
                    "epoch": 1,
                    "path": sample.to_string_lossy(),
                    "op": "create"
                })
            ),
        )
        .unwrap();
        let response = dispatch(
            &root,
            br#"{"id":"ss7","op":"seat.shield.status","params":{}}"#,
        );
        assert!(response.ok, "{response:?}");
        assert_eq!(response.data["watch_pending"], 0);
        assert!(sample.exists());
        let _ = fs::remove_file(&sample);
        let _ = fs::remove_dir_all(root);
    }
}
