//! Typed `product.open` for a verified local FoFline pack.
//!
//! Runtime builds the start URL. JS does not. Not Tauri. Not remote SPA.
//! Publish is chmod-only: production Runtime has `CAP_NET_ADMIN` only.
//! State root stays listing-closed (`0751`); pack files are `0644`.

use crate::state_store::atomic_write_file;
use serde_json::{json, Value};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub const TARGET_FOFLINE_LOCAL: &str = "fofline_local";
pub const LOCAL_START_URL: &str = "fofline-spa://localhost/index.html?eliteewFoflineClient=1";
const PRODUCT_DIR: &str = "product";
const LAST_LAUNCH: &str = "last_launch.json";
const PACK_DIR_MODE: u32 = 0o755;
const PACK_FILE_MODE: u32 = 0o644;
const STATE_TRAVERSE_MODE: u32 = 0o751;

pub fn open_block(state_root: &Path) -> Option<&'static str> {
    seat_block(state_root).or_else(|| pack_session_block(state_root))
}

fn seat_block(state_root: &Path) -> Option<&'static str> {
    crate::launcher_pin::access_block(state_root).or_else(|| {
        if crate::launcher_inner::observe(state_root).allow_fofline == Some(true) {
            None
        } else {
            Some("allow_fofline_required")
        }
    })
}

fn pack_session_block(state_root: &Path) -> Option<&'static str> {
    if !crate::spa_pack::is_ready(state_root, None) {
        Some("pack_not_ready")
    } else if !crate::launcher_oidc::session_can_open(state_root) {
        Some("oidc_session_required")
    } else {
        None
    }
}

pub fn open(state_root: &Path, params: Value) -> Result<Value, String> {
    if let Some(code) = seat_block(state_root) {
        return Err(code.into());
    }
    match params
        .get("target")
        .and_then(Value::as_str)
        .unwrap_or(TARGET_FOFLINE_LOCAL)
        .trim()
    {
        TARGET_FOFLINE_LOCAL => {}
        "connected_cloud" => return Err("remote_spa_forbidden".into()),
        "spa_deep_link" => return Err("deep_link_unavailable".into()),
        _ => return Err("unsupported_product_target".into()),
    }
    if let Some(code) = pack_session_block(state_root) {
        return Err(code.into());
    }
    crate::launcher_oidc::ensure_fresh_session(state_root)?;
    publish_for_shell(state_root)?;
    let spa = crate::spa_pack::observe(state_root);
    record_launch(state_root, spa.app_build_id.as_deref())?;
    Ok(json!({
        "product_open": "applied",
        "target": TARGET_FOFLINE_LOCAL,
        "spa_app_build_id": spa.app_build_id,
        "entrypoint_url": LOCAL_START_URL,
        "detail": "FoFline local pack opened; typed origin fofline-spa://localhost"
    }))
}

pub fn observe_flag(state_root: &Path, fingerprint: &str) -> &'static str {
    let pin = crate::launcher_pin::observe_flags(state_root, fingerprint);
    let inner = crate::launcher_inner::observe(state_root);
    if pin.unlocked && inner.allow_fofline == Some(true) && crate::spa_pack::is_ready(state_root, None)
    {
        "allowed"
    } else {
        "unavailable"
    }
}

fn record_launch(state_root: &Path, app_build_id: Option<&str>) -> Result<(), String> {
    let dir = state_root.join(PRODUCT_DIR);
    fs::create_dir_all(&dir).map_err(|error| format!("product_store:{error}"))?;
    let mut permissions = fs::metadata(&dir)
        .map_err(|error| format!("product_store:{error}"))?
        .permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&dir, permissions).map_err(|error| format!("product_store:{error}"))?;
    let body = json!({
        "target": TARGET_FOFLINE_LOCAL,
        "app_build_id": app_build_id,
        "entrypoint_url": LOCAL_START_URL,
        "launched_at": now_epoch(),
        "status": "applied"
    });
    atomic_write_file(
        &dir,
        LAST_LAUNCH,
        format!("{body}\n").as_bytes(),
        0o600,
    )
    .map_err(|error| format!("product_store:{error}"))?;
    Ok(())
}

pub(crate) fn ensure_shell_traverse(state_root: &Path) -> Result<(), String> {
    // systemd StateDirectoryMode, StateStore::open, and spa_pack used to force 0700
    // on the parents. Traverse-only lets the kiosk reach the pack; secrets stay 0600.
    set_mode(state_root, STATE_TRAVERSE_MODE)?;
    let pack = crate::spa_pack::pack_root(state_root);
    if pack.is_dir() {
        set_mode(&pack, STATE_TRAVERSE_MODE)?;
    }
    if crate::spa_pack::is_ready(state_root, None) {
        publish_tree(&crate::spa_pack::current_dir(state_root))?;
    }
    Ok(())
}

fn publish_for_shell(state_root: &Path) -> Result<(), String> {
    ensure_shell_traverse(state_root)
}

fn publish_tree(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| format!("pack_publish:{error}"))?;
    if metadata.file_type().is_symlink() {
        return Err("pack_symlink".into());
    }
    if metadata.is_dir() {
        set_mode(path, PACK_DIR_MODE)?;
        for entry in fs::read_dir(path).map_err(|error| format!("pack_publish:{error}"))? {
            publish_tree(&entry.map_err(|error| format!("pack_publish:{error}"))?.path())?;
        }
        return Ok(());
    }
    if metadata.is_file() {
        set_mode(path, PACK_FILE_MODE)?;
        return Ok(());
    }
    Err("pack_entry".into())
}

fn set_mode(path: &Path, mode: u32) -> Result<(), String> {
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).map_err(|error| format!("pack_publish:{error}"))
}

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("empiieros-product-{nanos}"));
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn plant_pack(root: &Path) {
        let current = crate::spa_pack::current_dir(root);
        fs::create_dir_all(&current).unwrap();
        fs::write(current.join("index.html"), "<!doctype html><title>pack</title>").unwrap();
        fs::write(
            current.join("pack-manifest.json"),
            r#"{"schema_version":1,"kind":"fofline-spa","app_build_id":"1786238708853-jn25ybw0v0"}"#,
        )
        .unwrap();
        crate::spa_pack::write_ready_state(root, "1786238708853-jn25ybw0v0").unwrap();
        let inner = root.join("capsule_inner");
        fs::create_dir_all(&inner).unwrap();
        fs::write(
            inner.join("flags.json"),
            r#"{"allow_fofline":true,"channel_present":true,"channel_valid":true}"#,
        )
        .unwrap();
    }

    #[test]
    fn open_requires_unlock_fofline_and_pack() {
        let root = temp_root();
        assert_eq!(open(&root, json!({})).unwrap_err(), "pin_not_issued");
        crate::launcher_pin::plant_unlocked(&root);
        assert_eq!(open(&root, json!({})).unwrap_err(), "allow_fofline_required");
        plant_pack(&root);
        assert_eq!(open(&root, json!({})).unwrap_err(), "oidc_session_required");
        crate::launcher_oidc::plant_session(&root);
        let opened = open(&root, json!({})).unwrap();
        assert_eq!(opened["product_open"], "applied");
        assert_eq!(opened["target"], TARGET_FOFLINE_LOCAL);
        assert_eq!(opened["entrypoint_url"], LOCAL_START_URL);
        assert!(opened["entrypoint_url"].as_str().unwrap().starts_with("fofline-spa://"));
        assert_ne!(opened["target"], "connected_cloud");
        assert_eq!(
            open(&root, json!({ "target": "connected_cloud" })).unwrap_err(),
            "remote_spa_forbidden"
        );
        let index = crate::spa_pack::current_dir(&root).join("index.html");
        assert_eq!(
            fs::metadata(&index).unwrap().permissions().mode() & 0o777,
            PACK_FILE_MODE
        );
        assert_eq!(
            fs::metadata(&root).unwrap().permissions().mode() & 0o777,
            STATE_TRAVERSE_MODE
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn ensure_repairs_0700_walk_and_keeps_secrets_closed() {
        let root = temp_root();
        crate::launcher_pin::plant_unlocked(&root);
        plant_pack(&root);
        let pin = root.join("seat").join("first_boot_pin.json");
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        fs::set_permissions(
            &crate::spa_pack::pack_root(&root),
            fs::Permissions::from_mode(0o700),
        )
        .unwrap();
        fs::set_permissions(
            &crate::spa_pack::current_dir(&root),
            fs::Permissions::from_mode(0o700),
        )
        .unwrap();
        ensure_shell_traverse(&root).unwrap();
        assert_eq!(
            fs::metadata(&root).unwrap().permissions().mode() & 0o777,
            STATE_TRAVERSE_MODE
        );
        assert_eq!(
            fs::metadata(&crate::spa_pack::pack_root(&root))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            STATE_TRAVERSE_MODE
        );
        assert_eq!(
            fs::metadata(&crate::spa_pack::current_dir(&root).join("index.html"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            PACK_FILE_MODE
        );
        assert_eq!(
            fs::metadata(&pin).unwrap().permissions().mode() & 0o777,
            0o600
        );
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        crate::RuntimeCore::new(root.clone()).expect("core");
        assert_eq!(
            fs::metadata(&root).unwrap().permissions().mode() & 0o777,
            STATE_TRAVERSE_MODE
        );
        fs::remove_dir_all(root).ok();
    }
}
