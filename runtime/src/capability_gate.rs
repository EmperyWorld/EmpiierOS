//! Snapshot rows whose availability is the operation's first precondition.
//!
//! Each row calls the same `block` the IPC operation returns before it reads
//! the request body. Helper binaries for air, portal, and SPA are a route,
//! not a precondition. `time.sync` follows `time_control::helper_ready`
//! (binary, unit, and `systemctl`). A masked timesyncd binary is not a gate.

use std::path::Path;

pub struct Row {
    pub id: &'static str,
    pub availability: &'static str,
    pub source: &'static str,
    pub reason: &'static str,
}

pub fn time_sync_row() -> Row {
    time_sync(crate::time_control::helper_ready())
}

pub fn time_sync(helper: bool) -> Row {
    Row {
        id: "time.sync",
        availability: if helper { "available" } else { "unavailable" },
        source: "operator-wall-clock",
        reason: if helper {
            "allowlisted_time_sync_helper_observed"
        } else {
            "time_sync_helper_not_installed"
        },
    }
}

pub fn product_open(state_root: Option<&Path>) -> Row {
    gated(
        "product.open",
        "launcher-product-local",
        "empiieros_product_open_local_pack",
        state_root,
        crate::launcher_product::open_block,
    )
}

pub fn launcher_status() -> Row {
    ready(
        "launcher.status",
        "launcher-observe",
        "empiieros_launcher_observe_only",
    )
}

pub fn capsule_import(state_root: Option<&Path>) -> Row {
    gated(
        "launcher.capsule_import",
        "launcher-capsule-v3",
        "empiieros_launcher_capsule_v3",
        state_root,
        crate::launcher_pin::access_block,
    )
}

pub fn air_activate(state_root: Option<&Path>) -> Row {
    gated(
        "launcher.air_activate",
        "launcher-air-wss",
        "empiieros_launcher_air_ewae1",
        state_root,
        crate::launcher_air::precondition,
    )
}

pub fn pin_issue() -> Row {
    Row {
        id: "launcher.pin_issue",
        availability: "unavailable",
        source: "secure-reset-ledger",
        reason: "empiieros_launcher_pin_installer_only",
    }
}

pub fn pin_unlock(state_root: Option<&Path>) -> Row {
    gated(
        "launcher.pin_unlock",
        "launcher-first-boot-pin",
        "empiieros_launcher_pin_unlock",
        state_root,
        crate::launcher_pin::unlock_block,
    )
}

pub fn channel_up(state_root: Option<&Path>) -> Row {
    gated(
        "launcher.channel_up",
        "launcher-portal-wg",
        "empiieros_launcher_portal_os_plane",
        state_root,
        crate::launcher_portal::bring_up_block,
    )
}

pub fn hostmap_apply(state_root: Option<&Path>) -> Row {
    gated(
        "launcher.hostmap_apply",
        "launcher-portal-hostmap",
        "empiieros_launcher_portal_hostmap",
        state_root,
        crate::portal_hostmap::hostmap_block,
    )
}

pub fn spa_sync(state_root: Option<&Path>) -> Row {
    gated(
        "launcher.spa_sync",
        "launcher-spa-pack",
        "empiieros_launcher_spa_bootstrap",
        state_root,
        crate::launcher_spa::precondition,
    )
}

pub fn edge_health(state_root: Option<&Path>) -> Row {
    gated(
        "launcher.edge_health",
        "launcher-spa-health",
        "empiieros_launcher_edge_health",
        state_root,
        crate::launcher_spa::precondition,
    )
}

pub fn oidc_start(state_root: Option<&Path>) -> Row {
    gated(
        "launcher.oidc_start",
        "launcher-native-oidc",
        "empiieros_launcher_native_oidc",
        state_root,
        crate::launcher_oidc::start_block,
    )
}

pub fn oidc_logout(state_root: Option<&Path>) -> Row {
    gated(
        "launcher.oidc_logout",
        "launcher-native-oidc",
        "empiieros_launcher_native_oidc_logout",
        state_root,
        crate::launcher_pin::access_block,
    )
}

pub fn ota_status(state_root: Option<&Path>) -> Row {
    gated(
        "launcher.ota_status",
        "launcher-ota-capsule",
        "empiieros_launcher_ota_gates",
        state_root,
        crate::launcher_ota::status_block,
    )
}

pub fn ota_request(state_root: Option<&Path>) -> Row {
    gated(
        "launcher.ota_request",
        "launcher-ota-capsule",
        "empiieros_launcher_ota_request_gated",
        state_root,
        crate::launcher_ota::mutation_block,
    )
}

pub fn ota_apply(state_root: Option<&Path>) -> Row {
    gated(
        "launcher.ota_apply",
        "launcher-ota-capsule",
        "cutover_planted_e4",
        state_root,
        crate::launcher_ota::mutation_block,
    )
}

fn gated(
    id: &'static str,
    source: &'static str,
    ready_reason: &'static str,
    state_root: Option<&Path>,
    block: fn(&Path) -> Option<&'static str>,
) -> Row {
    let reason = match state_root {
        Some(root) => block(root),
        None => Some("pin_not_issued"),
    };
    row(id, source, ready_reason, reason)
}

fn ready(id: &'static str, source: &'static str, reason: &'static str) -> Row {
    row(id, source, reason, None)
}

fn row(
    id: &'static str,
    source: &'static str,
    ready_reason: &'static str,
    block: Option<&'static str>,
) -> Row {
    Row {
        id,
        availability: if block.is_none() {
            "available"
        } else {
            "unavailable"
        },
        source,
        reason: block.unwrap_or(ready_reason),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("empiieros-capgate-{}-{nanos}", std::process::id()));
        fs::create_dir_all(&root).expect("temp root");
        root
    }

    fn assert_blocked(row: &Row, op: Result<serde_json::Value, String>) {
        assert_eq!(row.availability, "unavailable");
        assert_eq!(op.expect_err("operation should refuse"), row.reason);
    }

    #[test]
    fn time_sync_follows_the_helper_and_not_timesyncd() {
        let ready = time_sync(true);
        assert_eq!(ready.availability, "available");
        assert_eq!(ready.source, "operator-wall-clock");
        assert_eq!(ready.reason, "allowlisted_time_sync_helper_observed");
        let missing = time_sync(false);
        assert_eq!(missing.availability, "unavailable");
        assert_eq!(missing.source, "operator-wall-clock");
        assert_eq!(missing.reason, "time_sync_helper_not_installed");
        assert!(!ready.reason.contains("timesyncd"));
        assert!(!missing.reason.contains("timesyncd"));
        assert!(!ready.source.contains("timesyncd"));
        let row = time_sync_row();
        assert_eq!(
            row.availability == "available",
            crate::time_control::helper_ready()
        );
        if !crate::time_control::helper_ready() {
            let err = crate::time_control::execute(json!({
                "action": "set",
                "datetime": "2026-10-03 12:00"
            }))
            .expect_err("missing helper");
            assert!(err.starts_with("time_backend_unavailable"));
        }
    }

    #[test]
    fn rows_match_the_operation_before_the_request_body() {
        let root = temp_root();
        let some = Some(root.as_path());
        assert_eq!(product_open(None).reason, "pin_not_issued");
        assert_eq!(spa_sync(None).reason, "pin_not_issued");
        assert_eq!(ota_request(None).reason, "pin_not_issued");
        assert_blocked(
            &product_open(some),
            crate::launcher_product::open(&root, json!({})),
        );
        assert_blocked(&spa_sync(some), crate::launcher_spa::sync(&root, json!({})));
        assert_eq!(spa_sync(some).reason, "pin_not_issued");
        assert_blocked(
            &air_activate(some),
            crate::launcher_air::activate(&root, json!({})),
        );
        assert_blocked(
            &ota_request(some),
            crate::launcher_ota::request(&root, json!({})),
        );
        assert_blocked(
            &ota_status(some),
            crate::launcher_ota::status(&root, json!({})),
        );

        crate::launcher_pin::plant_unlocked(&root);
        assert_blocked(
            &product_open(some),
            crate::launcher_product::open(&root, json!({})),
        );
        assert_eq!(product_open(some).reason, "allow_fofline_required");
        assert_blocked(&spa_sync(some), crate::launcher_spa::sync(&root, json!({})));
        assert_eq!(spa_sync(some).reason, "allow_fofline_required");
        assert_blocked(
            &ota_request(some),
            crate::launcher_ota::request(&root, json!({})),
        );
        assert_eq!(ota_request(some).reason, "license_required");
        assert_eq!(ota_status(some).availability, "available");
        assert!(crate::launcher_ota::status(&root, json!({})).is_ok());
        assert_blocked(
            &channel_up(some),
            crate::launcher_portal::bring_up(&root, json!({})),
        );
        assert_eq!(channel_up(some).reason, "channel_missing");
        assert_blocked(
            &oidc_start(some),
            crate::launcher_oidc::start(&root, json!({})),
        );
        assert_eq!(oidc_start(some).reason, "allow_fofline_required");

        if !crate::launcher_inner::portal_iface_up() {
            assert_blocked(
                &hostmap_apply(some),
                crate::launcher_portal::apply_hostmap(&root, json!({})),
            );
            assert_eq!(hostmap_apply(some).reason, "portal_iface_down");
        }

        let pin_path = root.join("seat").join("first_boot_pin.json");
        let mut record: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&pin_path).expect("pin record"))
                .expect("json");
        record["fingerprint"] = json!("fp_v2_other-device");
        fs::write(
            &pin_path,
            serde_json::to_vec_pretty(&record).expect("encode"),
        )
        .expect("write");
        assert_blocked(
            &product_open(some),
            crate::launcher_product::open(&root, json!({})),
        );
        assert_eq!(product_open(some).reason, "fingerprint_mismatch");
        assert_blocked(
            &pin_unlock(some),
            crate::launcher_pin::unlock(&root, json!({})),
        );
        assert_eq!(pin_unlock(some).reason, "fingerprint_mismatch");

        let _ = fs::remove_dir_all(root);
    }
}
