//! Production Capsule v3 import for the EmpiierOS launcher.
//!
//! Harvested from kinship Capsule v3 open (not Tauri IPC). Inner entitlement
//! and channel are persisted in Runtime. Activation tokens and WG conf never
//! enter the response.

use crate::capsule_v3::{self, OpenSummary};
use crate::launcher_inner;
use serde_json::{json, Value};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

const MAX_CAPSULE_BYTES: u64 = 2 * 1024 * 1024;
const RECIPIENT_SK_FILE: &str = "recipient.sk.hex";

pub fn import(state_root: &Path, params: Value) -> Result<Value, String> {
    crate::launcher_pin::require_unlocked(state_root)?;
    let path = params
        .get("path")
        .and_then(Value::as_str)
        .ok_or_else(|| "path_required".to_string())?;
    let file = allowlisted_capsule_path(state_root, path)?;
    let recipient_sk = ensure_recipient_sk(state_root)?;
    let issuer_pk = load_issuer_pk(state_root)?;
    let meta = fs::symlink_metadata(&file).map_err(|error| format!("read_failed:{error}"))?;
    if meta.file_type().is_symlink() || !meta.file_type().is_file() {
        return Err("path_not_regular_file".into());
    }
    if meta.len() > MAX_CAPSULE_BYTES {
        return Err("capsule_too_large".into());
    }
    let blob = fs::read(&file).map_err(|error| format!("read_failed:{error}"))?;
    bind_opened(
        state_root,
        &blob,
        file.file_name(),
        "file",
        recipient_sk,
        issuer_pk,
        file.parent(),
    )
}

pub(crate) fn bind_verified_blob(
    state_root: &Path,
    blob: &[u8],
    source: &str,
) -> Result<Value, String> {
    let recipient_sk = ensure_recipient_sk(state_root)?;
    let issuer_pk = load_issuer_pk(state_root)?;
    if blob.len() as u64 > MAX_CAPSULE_BYTES {
        return Err("capsule_too_large".into());
    }
    bind_opened(
        state_root,
        blob,
        Some(std::ffi::OsStr::new("air.ecap")),
        source,
        recipient_sk,
        issuer_pk,
        None,
    )
}

fn bind_opened(
    state_root: &Path,
    blob: &[u8],
    name: Option<&std::ffi::OsStr>,
    source: &str,
    recipient_sk: [u8; 32],
    issuer_pk: [u8; 32],
    sibling_dir: Option<&Path>,
) -> Result<Value, String> {
    let inner = capsule_v3::open_v3_inner(blob, recipient_sk, issuer_pk)?;
    persist_bind(
        state_root,
        &inner.summary,
        name,
        blob.len() as u64,
        source,
    )?;
    let view = launcher_inner::persist(state_root, &inner, blob)?;
    let ota_artifact_pk = crate::ota_capsule_store::plant_from_license(
        state_root,
        inner.entitlement.as_ref(),
        sibling_dir,
    )?;
    let source = if source == "air" { "air" } else { "file" };
    let result = json!({
        "imported": true,
        "bound": true,
        "source": source,
        "request_id_redacted": inner.summary.request_id_redacted,
        "product_id": inner.summary.product_id,
        "license_id_redacted": inner.summary.license_id_redacted,
        "bytes": blob.len() as u64,
        "name_redacted": redact_name(name),
        "entitlement_present": view.entitlement_present,
        "channel_present": view.channel_present,
        "channel_valid": view.channel_valid,
        "channel_up": false,
        "channel_code": view.channel_code,
        "allow_offline": view.allow_offline,
        "allow_online": view.allow_online,
        "allow_fofline": view.allow_fofline,
        "ota_artifact_pk": ota_artifact_pk,
        "detail": "capsule bound; inner entitlement/channel persisted; channel not started; product.open stays unavailable"
    });
    let text = result.to_string();
    if text.contains("activation_token")
        || text.contains("wg_conf")
        || text.contains("PrivateKey")
        || text.contains("eliteew_desktop_capsule_payload")
    {
        return Err("inner_secret_redaction_failed".into());
    }
    Ok(result)
}

fn allowlisted_capsule_path(state_root: &Path, raw: &str) -> Result<PathBuf, String> {
    if !raw.starts_with('/') || raw.contains("..") {
        return Err("path_not_allowed".into());
    }
    let path = PathBuf::from(raw);
    let allowed = raw.starts_with("/media/")
        || raw.starts_with("/run/media/")
        || raw.starts_with("/var/lib/empiieros-runtime/capsules/")
        || path.starts_with(state_root.join("capsules"));
    if !allowed {
        return Err("path_not_allowed".into());
    }
    Ok(path)
}

pub(crate) fn load_recipient_sk(state_root: &Path) -> Option<[u8; 32]> {
    let raw = fs::read_to_string(state_root.join("device_keys").join(RECIPIENT_SK_FILE)).ok()?;
    decode32(&raw)
}

/// Create `{state}/device_keys/recipient.sk.hex` when the file is absent.
/// An existing invalid file is fail-closed (`device_keys_invalid`) — do not rotate.
pub(crate) fn ensure_recipient_sk(state_root: &Path) -> Result<[u8; 32], String> {
    let dir = state_root.join("device_keys");
    let path = dir.join(RECIPIENT_SK_FILE);
    if path.exists() {
        return load_recipient_sk(state_root).ok_or_else(|| "device_keys_invalid".to_string());
    }
    fs::create_dir_all(&dir).map_err(|error| format!("device_keys_store_failed:{error}"))?;
    let mut dir_permissions = fs::metadata(&dir)
        .map_err(|error| format!("device_keys_store_failed:{error}"))?
        .permissions();
    dir_permissions.set_mode(0o700);
    fs::set_permissions(&dir, dir_permissions)
        .map_err(|error| format!("device_keys_store_failed:{error}"))?;
    let mut sk = [0u8; 32];
    getrandom::getrandom(&mut sk).map_err(|error| format!("rng_failed:{error}"))?;
    fs::write(&path, hex::encode(sk)).map_err(|error| format!("device_keys_store_failed:{error}"))?;
    let mut permissions = fs::metadata(&path)
        .map_err(|error| format!("device_keys_store_failed:{error}"))?
        .permissions();
    permissions.set_mode(0o600);
    fs::set_permissions(&path, permissions)
        .map_err(|error| format!("device_keys_store_failed:{error}"))?;
    load_recipient_sk(state_root).ok_or_else(|| "device_keys_missing".to_string())
}

pub(crate) fn load_issuer_pk(state_root: &Path) -> Result<[u8; 32], String> {
    capsule_v3::load_issuer_verifying_pk(Some(state_root))
}

pub(crate) fn persist_bind(
    state_root: &Path,
    summary: &OpenSummary,
    name: Option<&std::ffi::OsStr>,
    bytes: u64,
    source: &str,
) -> Result<(), String> {
    let dir = state_root.join("capsule_bind");
    fs::create_dir_all(&dir).map_err(|error| format!("bind_store_failed:{error}"))?;
    let path = dir.join("bound.json");
    let source = if source == "air" { "air" } else { "file" };
    let body = json!({
        "bound": true,
        "source": source,
        "request_id_redacted": summary.request_id_redacted,
        "product_id": summary.product_id,
        "license_id_redacted": summary.license_id_redacted,
        "name_redacted": redact_name(name),
        "bytes": bytes,
        "note": "launcher bind; inner entitlement/channel in capsule_inner — not Start"
    });
    fs::write(&path, format!("{body}\n")).map_err(|error| format!("bind_store_failed:{error}"))?;
    let mut permissions = fs::metadata(&path)
        .map_err(|error| format!("bind_store_failed:{error}"))?
        .permissions();
    permissions.set_mode(0o600);
    fs::set_permissions(&path, permissions)
        .map_err(|error| format!("bind_store_failed:{error}"))?;
    Ok(())
}

fn redact_name(name: Option<&std::ffi::OsStr>) -> String {
    let raw = name.and_then(|value| value.to_str()).unwrap_or("capsule");
    if raw.chars().all(|item| item.is_ascii_alphanumeric() || item == '.' || item == '-' || item == '_')
        && raw.len() <= 64
    {
        raw.to_string()
    } else {
        "capsule".into()
    }
}

fn decode32(raw: &str) -> Option<[u8; 32]> {
    let bytes = hex::decode(raw.trim()).ok()?;
    if bytes.len() != 32 {
        return None;
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::{allowlisted_capsule_path, import};
    use crate::capsule_v3::{self, seal_v3_for_test};
    use serde_json::json;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    const DEV_SIGNING_PK_HEX: &str =
        "a09aa5f47a6759802ff955f8dc2d2a14a5c99d23be97f864127ff9383455a4f0";
    const DEV_SIGNING_SK_HEX: &str =
        "2222222222222222222222222222222222222222222222222222222222222222";
    const DEV_RECIPIENT_SK_HEX: &str =
        "1111111111111111111111111111111111111111111111111111111111111111";

    fn temp_root() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("empiieros-capsule-{nanos}"));
        fs::create_dir_all(root.join("capsules")).unwrap();
        fs::create_dir_all(root.join("device_keys")).unwrap();
        fs::create_dir_all(root.join("capsule")).unwrap();
        root
    }

    #[test]
    fn path_allowlist_rejects_escape() {
        let root = temp_root();
        assert!(allowlisted_capsule_path(&root, "/etc/passwd").is_err());
        assert!(allowlisted_capsule_path(&root, "/media/../etc/passwd").is_err());
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn import_roundtrip_persists_bind_without_payload() {
        let root = temp_root();
        let recipient_sk = hex::decode(DEV_RECIPIENT_SK_HEX).unwrap();
        let mut sk = [0u8; 32];
        sk.copy_from_slice(&recipient_sk);
        let signing_sk = hex::decode(DEV_SIGNING_SK_HEX).unwrap();
        let mut ssk = [0u8; 32];
        ssk.copy_from_slice(&signing_sk);
        let blob = seal_v3_for_test(
            &json!({
                "kind": "eliteew_desktop_capsule_payload",
                "mode": "fofline",
                "allow_fofline": true,
                "allow_online": false,
                "wg_conf": "[Interface]\nPrivateKey=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=\n",
                "activation_token": "tok_must_stay_out_of_store",
                "entitlement": {"license_id": "lic-file-1"}
            }),
            sk,
            ssk,
            "2099-01-01T00:00:00Z",
            None,
        );
        fs::write(root.join("device_keys/recipient.sk.hex"), DEV_RECIPIENT_SK_HEX).unwrap();
        fs::write(root.join("capsule/signing.pk.hex"), DEV_SIGNING_PK_HEX).unwrap();
        crate::launcher_pin::plant_unlocked(&root);
        let capsule = root.join("capsules/current.ecap");
        fs::write(&capsule, &blob).unwrap();
        let result = import(
            &root,
            json!({ "path": capsule.to_str().unwrap() }),
        )
        .unwrap();
        assert_eq!(result["imported"], true);
        assert_eq!(result["bound"], true);
        assert_eq!(result["entitlement_present"], true);
        assert_eq!(result["channel_present"], true);
        assert_eq!(result["allow_fofline"], true);
        assert_eq!(result["ota_artifact_pk"], "missing");
        assert!(!result.to_string().contains("eliteew_desktop_capsule_payload"));
        assert!(!result.to_string().contains("activation_token"));
        assert!(!result.to_string().contains("wg_conf"));
        assert!(!result.to_string().contains("PrivateKey"));
        let channel = fs::read_to_string(root.join("capsule_inner/channel.json")).unwrap();
        assert!(channel.contains("PrivateKey"));
        assert!(!channel.contains("activation_token"));
        let flags_meta = fs::metadata(root.join("capsule_inner/flags.json")).unwrap();
        assert_eq!(flags_meta.permissions().mode() & 0o777, 0o600);
        let sealed = root.join("capsules/current.ecap");
        assert_eq!(fs::read(&sealed).unwrap(), blob);
        let status = crate::launcher_status::observe(&root);
        assert_eq!(status["capsule_imported"], true);
        assert_eq!(status["entitlement_present"], true);
        assert_eq!(status["channel_present"], true);
        assert_eq!(status["channel_valid"], false);
        assert_eq!(status["channel_up"], false);
        assert_eq!(status["channel_code"], "present_invalid");
        assert_eq!(status["allow_fofline"], true);
        assert!(!status.to_string().contains("PrivateKey"));
        assert!(!status.to_string().contains("10.88.89.1"));
        if std::env::var("EMPIIEROS_WRITE_CAPSULE_FIXTURE").ok().as_deref() == Some("1") {
            let dest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/metal/lab-capsule");
            fs::create_dir_all(&dest).unwrap();
            fs::write(dest.join("recipient.sk.hex"), DEV_RECIPIENT_SK_HEX).unwrap();
            fs::write(dest.join("signing.pk.hex"), DEV_SIGNING_PK_HEX).unwrap();
            fs::write(dest.join("current.ecap"), blob).unwrap();
        }
        let _ = capsule_v3::load_signing_pk;
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn import_validates_portal_conf_without_starting_tunnel() {
        let root = temp_root();
        let recipient_sk = hex::decode(DEV_RECIPIENT_SK_HEX).unwrap();
        let mut sk = [0u8; 32];
        sk.copy_from_slice(&recipient_sk);
        let signing_sk = hex::decode(DEV_SIGNING_SK_HEX).unwrap();
        let mut ssk = [0u8; 32];
        ssk.copy_from_slice(&signing_sk);
        let wg = "[Interface]\n\
PrivateKey = AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE=\n\
Address = 10.88.89.101/32\n\
DNS = 10.88.89.1\n\
MTU = 1380\n\
\n\
[Peer]\n\
PublicKey = AgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgI=\n\
Endpoint = 93.77.165.185:51823\n\
AllowedIPs = 10.88.89.1/32\n\
PersistentKeepalive = 25\n";
        let blob = seal_v3_for_test(
            &json!({
                "kind": "eliteew_desktop_capsule_payload",
                "mode": "fofline",
                "allow_fofline": true,
                "wg_conf": wg,
                "entitlement": {"license_id": "lic-portal-1"}
            }),
            sk,
            ssk,
            "2099-01-01T00:00:00Z",
            None,
        );
        fs::write(root.join("device_keys/recipient.sk.hex"), DEV_RECIPIENT_SK_HEX).unwrap();
        fs::write(root.join("capsule/signing.pk.hex"), DEV_SIGNING_PK_HEX).unwrap();
        crate::launcher_pin::plant_unlocked(&root);
        let capsule = root.join("capsules/valid.ecap");
        fs::write(&capsule, &blob).unwrap();
        let result = import(&root, json!({ "path": capsule.to_str().unwrap() })).unwrap();
        assert_eq!(result["channel_present"], true);
        assert_eq!(result["channel_valid"], true);
        assert_eq!(result["channel_up"], false);
        assert_eq!(result["channel_code"], "valid_idle");
        assert!(!result.to_string().contains("PrivateKey"));
        assert!(!result.to_string().contains("93.77.165.185"));
        let status = crate::launcher_status::observe(&root);
        assert_eq!(status["channel_valid"], true);
        assert_eq!(status["channel_up"], false);
        assert_eq!(status["channel_code"], "valid_idle");
        assert!(!status.to_string().contains("PrivateKey"));
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn import_plants_sibling_artifact_pk() {
        let root = temp_root();
        let recipient_sk = hex::decode(DEV_RECIPIENT_SK_HEX).unwrap();
        let mut sk = [0u8; 32];
        sk.copy_from_slice(&recipient_sk);
        let signing_sk = hex::decode(DEV_SIGNING_SK_HEX).unwrap();
        let mut ssk = [0u8; 32];
        ssk.copy_from_slice(&signing_sk);
        let blob = seal_v3_for_test(
            &json!({
                "kind": "eliteew_desktop_capsule_payload",
                "mode": "fofline",
                "allow_fofline": true,
                "entitlement": {"license_id": "lic-ota-pk"}
            }),
            sk,
            ssk,
            "2099-01-01T00:00:00Z",
            None,
        );
        fs::write(root.join("device_keys/recipient.sk.hex"), DEV_RECIPIENT_SK_HEX).unwrap();
        fs::write(root.join("capsule/signing.pk.hex"), DEV_SIGNING_PK_HEX).unwrap();
        crate::launcher_pin::plant_unlocked(&root);
        let capsule = root.join("capsules/with-pk.ecap");
        fs::write(&capsule, &blob).unwrap();
        let pk = "ab".repeat(32);
        fs::write(root.join("capsules/artifact.pk.hex"), format!("{pk}\n")).unwrap();
        let result = import(&root, json!({ "path": capsule.to_str().unwrap() })).unwrap();
        assert_eq!(result["imported"], true);
        assert_eq!(result["ota_artifact_pk"], "planted");
        assert!(!result.to_string().contains(&pk));
        let planted = fs::read_to_string(root.join("ota/artifact.pk.hex")).unwrap();
        assert_eq!(planted.trim(), pk);
        fs::remove_dir_all(root).ok();
    }
}
