//! OS-plane FoFline SPA pack sync. Runtime stays `IPAddressDeny=any`.
//!
//! Helper talks only to VIP `10.88.89.1`. Not Tauri. Not `product.open`.

use crate::command_runner;
use crate::launcher_inner;
use crate::spa_pack::{self, SpaView, STATE_DENIED, STATE_READY};
use crate::state_store::atomic_write_file;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::time::Duration;

const HELPER: &str = "/usr/local/sbin/empiieros-spa-fetch";
const UNIT: &str = "empiieros-spa-fetch.service";
const REQUEST_DIR: &str = "/run/empiieros";
const REQUEST_NAME: &str = "spa-fetch.request";
const RESULT_NAME: &str = "spa-fetch.result";

pub fn observe(state_root: &Path) -> SpaView {
    let inner = launcher_inner::observe(state_root);
    if inner.allow_fofline != Some(true) {
        return SpaView {
            state: STATE_DENIED.into(),
            code: "allow_fofline_required".into(),
            app_build_id: None,
        };
    }
    spa_pack::observe(state_root)
}

pub fn precondition(state_root: &Path) -> Option<&'static str> {
    if let Some(code) = crate::launcher_pin::access_block(state_root) {
        return Some(code);
    }
    if launcher_inner::observe(state_root).allow_fofline != Some(true) {
        return Some("allow_fofline_required");
    }
    if !launcher_inner::portal_iface_up() {
        return Some("channel_not_up");
    }
    None
}

pub fn sync(state_root: &Path, _params: Value) -> Result<Value, String> {
    if let Some(code) = precondition(state_root) {
        return Err(code.into());
    }
    if helper_installed() && !helper_process() {
        return activate_via_helper_action(state_root, "sync", Duration::from_secs(900));
    }
    execute_sync(state_root)
}

pub fn edge_health(state_root: &Path, _params: Value) -> Result<Value, String> {
    if let Some(code) = precondition(state_root) {
        return Err(code.into());
    }
    if helper_installed() && !helper_process() {
        return activate_via_helper_action(state_root, "health", Duration::from_secs(30));
    }
    execute_health()
}

pub fn helper_main() -> Result<(), String> {
    let request_path = Path::new(REQUEST_DIR).join(REQUEST_NAME);
    let outcome = match helper_exchange(&request_path) {
        Ok(value) => json!({ "ok": true, "result": value }),
        Err(error) => json!({ "ok": false, "error": error }),
    };
    let _ = fs::remove_file(&request_path);
    atomic_write_file(
        Path::new(REQUEST_DIR),
        RESULT_NAME,
        format!("{outcome}\n").as_bytes(),
        0o600,
    )
    .map_err(|error| format!("spa_result_write:{error}"))?;
    Ok(())
}

fn helper_exchange(request_path: &Path) -> Result<Value, String> {
    let raw = fs::read_to_string(request_path).map_err(|error| format!("spa_request_read:{error}"))?;
    let request: Value =
        serde_json::from_str(&raw).map_err(|error| format!("spa_request_json:{error}"))?;
    let state = request
        .get("state_dir")
        .and_then(Value::as_str)
        .ok_or_else(|| "spa_request_state_dir".to_string())?;
    if let Some(code) = precondition(Path::new(state)) {
        return Err(code.into());
    }
    match request.get("action").and_then(Value::as_str).unwrap_or("sync") {
        "health" => execute_health(),
        "oidc_token" => crate::launcher_oidc::execute_token(Path::new(state)),
        "oidc_par" => crate::launcher_oidc::execute_par(Path::new(state)),
        "oidc_refresh" => crate::launcher_oidc::execute_refresh(Path::new(state)),
        _ => execute_sync(Path::new(state)),
    }
}

fn helper_process() -> bool {
    matches!(
        std::env::var("EMPIIEROS_SPA_HELPER").ok().as_deref(),
        Some("1") | Some("true") | Some("yes")
    )
}

fn helper_installed() -> bool {
    Path::new(HELPER).is_file()
        && Path::new("/etc/systemd/system").join(UNIT).is_file()
        && Path::new("/usr/bin/systemctl").is_file()
}

fn activate_via_helper_action(
    state_root: &Path,
    action: &str,
    timeout: Duration,
) -> Result<Value, String> {
    let body = json!({
        "state_dir": state_root.display().to_string(),
        "action": action
    });
    atomic_write_file(
        Path::new(REQUEST_DIR),
        REQUEST_NAME,
        format!("{body}\n").as_bytes(),
        0o600,
    )
    .map_err(|error| format!("spa_request_write_failed:{error}"))?;
    let _ = fs::remove_file(Path::new(REQUEST_DIR).join(RESULT_NAME));
    let started = command_runner::run("/usr/bin/systemctl", &["start", UNIT], timeout)
        .map_err(|error| format!("spa_helper_spawn_failed:{error}"))?;
    if !started.success {
        return Err(format!("spa_helper_start_failed: {}", started.stderr.trim()));
    }
    let raw = fs::read_to_string(Path::new(REQUEST_DIR).join(RESULT_NAME))
        .map_err(|_| "spa_helper_result_missing".to_string())?;
    if raw.contains("PrivateKey") || raw.contains("wg_conf") {
        return Err("spa_secret_redaction_failed".into());
    }
    let parsed: Value =
        serde_json::from_str(raw.trim()).map_err(|error| format!("spa_helper_result_json:{error}"))?;
    if parsed.get("ok") == Some(&json!(true)) {
        return parsed
            .get("result")
            .cloned()
            .ok_or_else(|| "spa_helper_result_missing".to_string());
    }
    Err(parsed
        .get("error")
        .and_then(Value::as_str)
        .unwrap_or("spa_sync_failed")
        .to_string())
}

fn execute_sync(state_root: &Path) -> Result<Value, String> {
    let raw = crate::https_vip::get_text(spa_pack::MANIFEST_PATH, spa_pack::MANIFEST_MAX_BYTES)
        .map_err(|error| {
            let _ = spa_pack::write_error_state(state_root, "spa_manifest_fetch");
            error
        })?;
    let manifest = spa_pack::parse_spa_latest(&raw).map_err(|error| {
        let _ = spa_pack::write_error_state(state_root, &error);
        error
    })?;
    if spa_pack::is_ready(state_root, Some(&manifest.app_build_id)) {
        spa_pack::write_ready_state(state_root, &manifest.app_build_id)?;
        crate::launcher_product::ensure_shell_traverse(state_root)?;
        let assets = crate::spa_assets::sync_groups(state_root, &manifest.app_build_id)?;
        crate::launcher_product::ensure_shell_traverse(state_root)?;
        return Ok(flags(
            STATE_READY,
            "ready",
            Some(&manifest.app_build_id),
            "FoFline workspace already current",
            &assets,
        ));
    }
    let artifact = spa_pack::artifact_path(&manifest)?;
    let spool = spa_pack::spool_dir(state_root);
    fs::create_dir_all(&spool).map_err(|error| format!("spa_spool:{error}"))?;
    let archive = spool.join(format!("{}.tar.zst", manifest.app_build_id));
    let downloaded = crate::https_vip::get_file_hashed(
        &artifact,
        &archive,
        spa_pack::CORE_PACK_ARCHIVE_MAX_BYTES,
    )
    .map_err(|error| {
        let _ = fs::remove_file(&archive);
        let _ = spa_pack::write_error_state(state_root, &error);
        error
    })?;
    if !downloaded
        .sha256_hex
        .eq_ignore_ascii_case(manifest.sha256.trim())
    {
        let _ = fs::remove_file(&archive);
        let _ = spa_pack::write_error_state(state_root, "sha256_mismatch");
        return Err("sha256_mismatch".into());
    }
    let bytes = fs::read(&archive).map_err(|error| format!("spa_spool:{error}"))?;
    if let Err(error) = spa_pack::verify_tauri_minisign(&bytes, &manifest.signature) {
        let _ = fs::remove_file(&archive);
        let _ = spa_pack::write_error_state(state_root, &error);
        return Err(error);
    }
    drop(bytes);
    let staging = spa_pack::staging_dir(state_root);
    if let Err(error) = spa_pack::extract_tar_zst_file(&archive, &staging) {
        let _ = fs::remove_file(&archive);
        let _ = spa_pack::write_error_state(state_root, &error);
        return Err(error);
    }
    let _ = fs::remove_file(&archive);
    spa_pack::promote_staging(state_root)?;
    spa_pack::write_ready_state(state_root, &manifest.app_build_id)?;
    crate::launcher_product::ensure_shell_traverse(state_root)?;
    let assets = crate::spa_assets::sync_groups(state_root, &manifest.app_build_id)?;
    crate::launcher_product::ensure_shell_traverse(state_root)?;
    Ok(flags(
        STATE_READY,
        "ready",
        Some(&manifest.app_build_id),
        "FoFline workspace installed",
        &assets,
    ))
}

fn execute_health() -> Result<Value, String> {
    let raw = crate::https_vip::get_text(
        crate::https_vip::HEALTH_PATH,
        crate::https_vip::HEALTH_MAX_BYTES,
    )?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("edge_health_empty".into());
    }
    Ok(json!({
        "edge_health": "applied",
        "via": crate::https_vip::EDGE_VIP,
        "path": crate::https_vip::HEALTH_PATH,
        "bytes": raw.len(),
        "body_prefix": trimmed.chars().take(80).collect::<String>(),
        "detail": "edge /api/health via portal VIP; Runtime IPAddressDeny stays"
    }))
}

fn flags(
    state: &str,
    code: &str,
    app_build_id: Option<&str>,
    detail: &str,
    assets: &crate::spa_assets::AssetsView,
) -> Value {
    json!({
        "spa_state": state,
        "spa_code": code,
        "spa_app_build_id": app_build_id,
        "spa_assets_ready": assets.ready,
        "spa_assets_total": assets.total,
        "spa_assets_code": assets.code,
        "product_open": "unavailable",
        "detail": detail
    })
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
        let root = std::env::temp_dir().join(format!("empiieros-spa-sync-{nanos}"));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn sync_requires_unlock_fofline_and_portal() {
        let root = temp_root();
        assert_eq!(sync(&root, json!({})).unwrap_err(), "pin_not_issued");
        crate::launcher_pin::plant_unlocked(&root);
        assert_eq!(sync(&root, json!({})).unwrap_err(), "allow_fofline_required");
        assert_eq!(edge_health(&root, json!({})).unwrap_err(), "allow_fofline_required");
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn edge_health_requires_unlock() {
        let root = temp_root();
        assert_eq!(edge_health(&root, json!({})).unwrap_err(), "pin_not_issued");
        fs::remove_dir_all(root).ok();
    }
}
