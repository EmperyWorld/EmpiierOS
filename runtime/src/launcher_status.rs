//! Observe-only EmpiierOS launcher status.
//!
//! Fingerprint algorithm harvested from kinship `device_identity.rs`
//! (`fp_v2` / hostname + machine-id + DMI UUID). Not a Tauri command.
//! First-boot PIN flags come from `launcher_pin`. Capsule bind is observed
//! from `capsule_bind/bound.json`.

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

pub struct FingerprintInputs {
    pub hostname: String,
    pub machine_id: String,
    pub hardware_uuid: String,
}

pub fn fingerprint_v2(inputs: &FingerprintInputs) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"eliteew-desktop-client|v2|");
    hasher.update(inputs.hostname.as_bytes());
    hasher.update(b"|");
    hasher.update(inputs.machine_id.as_bytes());
    hasher.update(b"|");
    hasher.update(inputs.hardware_uuid.as_bytes());
    let digest = hasher.finalize();
    format!("fp_v2_{}", hex_encode(&digest[..16]))
}

pub fn live_fingerprint_v2() -> String {
    fingerprint_v2(&live_inputs())
}

pub fn observe(state_root: &Path) -> Value {
    let inputs = live_inputs();
    let fingerprint = fingerprint_v2(&inputs);
    let pin = crate::launcher_pin::observe_flags(state_root, &fingerprint);
    let bind = load_bind(state_root);
    let inner = crate::launcher_inner::observe(state_root);
    let spa = crate::launcher_spa::observe(state_root);
    let assets = crate::spa_assets::observe(state_root);
    json!({
        "vehicle": "empiieros-shell-gui",
        "tauri": false,
        "fingerprint": fingerprint,
        "fingerprint_scheme": "fp_v2",
        "fingerprint_bound": pin.bound,
        "pin_issued": pin.issued,
        "pin_unlocked": pin.unlocked,
        "pin_locked": pin.locked,
        "capsule_imported": bind.bound,
        "product_id": bind.product_id,
        "capsule_source": bind.source,
        "entitlement_present": inner.entitlement_present,
        "channel_present": inner.channel_present,
        "channel_valid": inner.channel_valid,
        "channel_up": inner.channel_up,
        "channel_code": inner.channel_code,
        "allow_offline": inner.allow_offline,
        "allow_online": inner.allow_online,
        "allow_fofline": inner.allow_fofline,
        "spa_state": spa.state,
        "spa_code": spa.code,
        "spa_app_build_id": spa.app_build_id,
        "spa_assets_ready": assets.ready,
        "spa_assets_total": assets.total,
        "spa_assets_code": assets.code,
        "product_open": crate::launcher_product::observe_flag(state_root, &fingerprint),
        "oidc_session": crate::launcher_oidc::session_flag(state_root),
        "portal_hostmap": crate::portal_hostmap::observe_default(),
        "wall_clock_ok": crate::oidc_dpop_iat::wall_clock_is_sane(),
        "detail": "launcher status; capsule bind, inner flags, channel/sysfs, spa pack flags, portal host-map, wall clock, native OIDC session flag; product.open is allowed only for a ready local pack after login"
    })
}

struct BindView {
    bound: bool,
    product_id: String,
    source: String,
}

fn load_bind(state_root: &Path) -> BindView {
    let empty = BindView {
        bound: false,
        product_id: String::new(),
        source: String::new(),
    };
    let raw = fs::read_to_string(state_root.join("capsule_bind").join("bound.json")).ok();
    let Some(value) = raw.and_then(|text| serde_json::from_str::<Value>(&text).ok()) else {
        return empty;
    };
    let bound = value.get("bound").and_then(Value::as_bool).unwrap_or(false);
    let product_id = value
        .get("product_id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_' || *c == '.')
        .take(64)
        .collect();
    let source = match value.get("source").and_then(Value::as_str) {
        Some("air") => "air",
        Some("file") => "file",
        _ => "",
    }
    .to_string();
    BindView {
        bound,
        product_id,
        source,
    }
}

pub(crate) fn live_inputs() -> FingerprintInputs {
    FingerprintInputs {
        hostname: read_trimmed("/etc/hostname")
            .or_else(|| std::env::var("HOSTNAME").ok())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "unknown-host".into()),
        machine_id: read_trimmed("/etc/machine-id").unwrap_or_else(|| "no-machine-id".into()),
        hardware_uuid: hardware_uuid(),
    }
}

fn hardware_uuid() -> String {
    for candidate in [
        "/sys/class/dmi/id/product_uuid",
        "/sys/class/dmi/id/board_serial",
        "/sys/class/dmi/id/product_serial",
    ] {
        if let Some(value) = read_trimmed(candidate) {
            if !value.eq_ignore_ascii_case("none")
                && !value.eq_ignore_ascii_case("unknown")
                && !value.eq_ignore_ascii_case("to be filled by o.e.m.")
            {
                return value;
            }
        }
    }
    "no-hw-uuid".into()
}

fn read_trimmed(path: &str) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{fingerprint_v2, load_bind, FingerprintInputs};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn fingerprint_v2_is_stable_and_prefixed() {
        let inputs = FingerprintInputs {
            hostname: "eliteew-ts".into(),
            machine_id: "abc".into(),
            hardware_uuid: "uuid-1".into(),
        };
        let first = fingerprint_v2(&inputs);
        let second = fingerprint_v2(&inputs);
        assert_eq!(first, second);
        assert!(first.starts_with("fp_v2_"), "{first}");
        assert_eq!(first.len(), "fp_v2_".len() + 32);
        let other = fingerprint_v2(&FingerprintInputs {
            hostname: "other".into(),
            machine_id: "abc".into(),
            hardware_uuid: "uuid-1".into(),
        });
        assert_ne!(first, other);
    }

    #[test]
    fn bind_view_reads_air_product() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("empiieros-bind-{nanos}"));
        fs::create_dir_all(root.join("capsule_bind")).unwrap();
        fs::write(
            root.join("capsule_bind/bound.json"),
            r#"{"bound":true,"product_id":"eliteew-fofline","source":"air"}
"#,
        )
        .unwrap();
        let view = load_bind(&root);
        assert!(view.bound);
        assert_eq!(view.product_id, "eliteew-fofline");
        assert_eq!(view.source, "air");
        let status = crate::launcher_status::observe(&root);
        assert_eq!(status["entitlement_present"], false);
        assert_eq!(status["channel_present"], false);
        assert_eq!(status["channel_valid"], false);
        assert_eq!(status["channel_up"], false);
        assert_eq!(status["channel_code"], "missing");
        assert_eq!(status["product_open"], "unavailable");
        fs::remove_dir_all(root).ok();
    }
}
