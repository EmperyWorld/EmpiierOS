//! Capsule-gated OTA — license + hardware + live OIDC + portal VIP.
//!
//! Production ops: `launcher.ota_*`. Not bootstrap `launcher.spa_sync`.
//! Not historical `update.*` (disabled in secure-reset core).
//! Apply cutover planted (E4): `os_component` → P4 destinations + baseline.
//! Runtime keeps `IPAddressDeny=any`; VIP HTTPS is the OTA helper only.

use crate::https_vip;
use crate::launcher_capsule;
use crate::launcher_inner;
use crate::launcher_oidc;
use crate::launcher_pin;
use crate::launcher_status;
use crate::ota_capsule_store;
use crate::ota_cutover;
use crate::portal_hostmap;
use crate::state_store::atomic_write_file;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

const HELPER: &str = "/usr/local/sbin/empiieros-ota-fetch";
const UNIT: &str = "empiieros-ota-fetch.service";
const REQUEST_DIR: &str = "/run/empiieros";
const REQUEST_NAME: &str = "ota-fetch.request";
const RESULT_NAME: &str = "ota-fetch.result";
const CONFIRM_APPLY: &str = "APPLY_OTA";
const HELPER_TIMEOUT_SECS: u64 = 900;
const CAPSULE_JSON_MAX: u64 = 2 * 1024 * 1024;
const BLOB_FILE_MAX: u64 = 256 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct GateView {
    pub pin_unlocked: bool,
    pub license_ok: bool,
    pub allow_fofline: bool,
    pub device_keys_ok: bool,
    pub hardware_bound: bool,
    pub oidc_session: bool,
    pub portal_up: bool,
    pub hostmap_applied: bool,
    pub artifact_pk: bool,
    pub ota_helper: bool,
    pub staged: bool,
    pub first_block: Option<&'static str>,
}

pub fn status_block(state_root: &Path) -> Option<&'static str> {
    crate::launcher_pin::access_block(state_root)
}

pub fn mutation_block(state_root: &Path) -> Option<&'static str> {
    if let Some(code) = crate::launcher_pin::access_block(state_root) {
        return Some(code);
    }
    observe_gates(state_root).first_block
}

pub fn status(state_root: &Path, _params: Value) -> Result<Value, String> {
    if let Some(code) = status_block(state_root) {
        return Err(code.into());
    }
    let view = observe_gates(state_root);
    let gates_ok = view.first_block.is_none();
    let apply_chip = ota_cutover::apply_status_chip(state_root);
    Ok(json!({
        "ota_gate": view.first_block.unwrap_or("ok"),
        "pin_unlocked": view.pin_unlocked,
        "license_ok": view.license_ok,
        "allow_fofline": view.allow_fofline,
        "device_keys_ok": view.device_keys_ok,
        "hardware_bound": view.hardware_bound,
        "oidc_session": if view.oidc_session { "present" } else { "missing" },
        "portal_up": view.portal_up,
        "portal_hostmap": if view.hostmap_applied { "applied" } else { "missing" },
        "artifact_pk": if view.artifact_pk { "present" } else { "missing" },
        "ota_helper": if view.ota_helper { "present" } else { "missing" },
        "ota_staged": view.staged,
        "ota_apply": apply_chip,
        "ota_applied_build_id": ota_cutover::applied_build_id(state_root),
        "ota_staged_build_id": ota_cutover::staged_build_id(state_root),
        "ota_next_action": ota_cutover::next_action(state_root, gates_ok),
        "detail": "capsule OTA gates only — secrets stay in Runtime"
    }))
}

pub fn request(state_root: &Path, _params: Value) -> Result<Value, String> {
    require_all_gates(state_root)?;
    if helper_installed() && !helper_process() {
        return activate_via_helper(
            state_root,
            "attest",
            Duration::from_secs(HELPER_TIMEOUT_SECS),
        );
    }
    if helper_process() {
        return execute_attest_and_fetch(state_root);
    }
    // Edge mint is planted (E2c); seat still needs the VIP-only helper binary.
    Err("ota_helper_required".into())
}

pub fn apply(state_root: &Path, params: Value) -> Result<Value, String> {
    require_all_gates(state_root)?;
    let confirm = params
        .get("confirm")
        .and_then(Value::as_str)
        .unwrap_or("");
    if confirm != CONFIRM_APPLY {
        return Err("confirmation_required".into());
    }
    if !ota_capsule_store::is_current_ready(state_root) {
        return Err("ota_not_staged".into());
    }
    ota_cutover::apply_current(state_root)
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
    .map_err(|error| format!("ota_result_write:{error}"))?;
    Ok(())
}

fn helper_exchange(request_path: &Path) -> Result<Value, String> {
    let raw = fs::read_to_string(request_path).map_err(|error| format!("ota_request_read:{error}"))?;
    let request: Value =
        serde_json::from_str(&raw).map_err(|error| format!("ota_request_json:{error}"))?;
    let state = request
        .get("state_dir")
        .and_then(Value::as_str)
        .ok_or_else(|| "ota_request_state_dir".to_string())?;
    let action = request
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("attest");
    let root = Path::new(state);
    require_all_gates(root)?;
    match action {
        "attest" => execute_attest_and_fetch(root),
        "stage" => {
            let doc = request
                .get("capsule")
                .cloned()
                .ok_or_else(|| "capsule_signature_required".to_string())?;
            let blob_dir = request
                .get("blob_dir")
                .and_then(Value::as_str)
                .map(PathBuf::from);
            ota_capsule_store::verify_and_stage_with_blobs(root, &doc, blob_dir.as_deref())
        }
        _ => Err("ota_action_unknown".into()),
    }
}

/// POST attest → grant → GET current + blobs → verify_and_stage. Helper-only.
fn execute_attest_and_fetch(state_root: &Path) -> Result<Value, String> {
    let attest = build_attest_request(state_root)?;
    let body =
        serde_json::to_string(&attest).map_err(|error| format!("ota_attest_json:{error}"))?;
    let (status, raw) = https_vip::post_json(https_vip::OTA_ATTEST_PATH, &body)?;
    if status == 404 {
        return Err(attest_error_code(&raw));
    }
    if status != 200 {
        return Err(format!("ota_attest_http_{status}"));
    }
    let capsule_href = parse_attest_grant(&raw)?;
    let capsule_raw = https_vip::get_text(&capsule_href, CAPSULE_JSON_MAX)
        .map_err(|error| map_http_error("ota_capsule", &error))?;
    let doc: Value =
        serde_json::from_str(&capsule_raw).map_err(|_| "capsule_signature_required".to_string())?;
    let spool = blob_spool_dir()?;
    download_blobs_for_doc(&doc, &spool)?;
    let staged = ota_capsule_store::verify_and_stage_with_blobs(state_root, &doc, Some(&spool))?;
    let _ = fs::remove_dir_all(&spool);
    Ok(staged)
}

fn attest_error_code(raw: &str) -> String {
    if let Ok(v) = serde_json::from_str::<Value>(raw) {
        if v.get("error").and_then(Value::as_str) == Some("no_capsule") {
            return "no_capsule".into();
        }
    }
    if raw.contains("no_capsule") {
        return "no_capsule".into();
    }
    "no_capsule".into()
}

/// Extract `capsule_href` from a portal-path grant. Fail-closed on escape.
pub fn parse_attest_grant(raw: &str) -> Result<String, String> {
    let v: Value = serde_json::from_str(raw.trim()).map_err(|_| "ota_grant_invalid".to_string())?;
    if v.get("ok").and_then(Value::as_bool) != Some(true) {
        return Err("ota_grant_denied".into());
    }
    let grant = v.get("grant").and_then(Value::as_str).unwrap_or("");
    if grant != "portal_path_v1" {
        return Err("ota_grant_denied".into());
    }
    let href = v
        .get("capsule_href")
        .and_then(Value::as_str)
        .ok_or_else(|| "ota_grant_invalid".to_string())?
        .trim();
    if href.contains("..") || href.contains('\\') || href.contains('\0') {
        return Err("path_escape".into());
    }
    if !href.starts_with(https_vip::OTA_PREFIX) {
        return Err("path_escape".into());
    }
    if href != https_vip::OTA_CURRENT_PATH {
        // Day-1 lock: only the well-known current capsule path.
        return Err("path_escape".into());
    }
    Ok(href.to_string())
}

fn blob_spool_dir() -> Result<PathBuf, String> {
    let dir = std::env::temp_dir().join(format!("empiieros-ota-blobs-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).map_err(|error| format!("ota_blob_spool:{error}"))?;
    Ok(dir)
}

fn download_blobs_for_doc(doc: &Value, spool: &Path) -> Result<(), String> {
    let files = doc
        .get("payload")
        .and_then(|p| p.get("files"))
        .and_then(Value::as_array)
        .ok_or_else(|| "payload_incomplete".to_string())?;
    for entry in files {
        let href = match entry.get("href").and_then(Value::as_str) {
            Some(h) if !h.trim().is_empty() => h.trim(),
            _ => continue,
        };
        let name = ota_capsule_store::parse_blob_href(href)?;
        let want = entry
            .get("sha256")
            .and_then(Value::as_str)
            .ok_or_else(|| "payload_incomplete".to_string())?
            .trim()
            .to_ascii_lowercase();
        if name != want {
            return Err("path_escape".into());
        }
        let dest = spool.join(&name);
        let downloaded = https_vip::get_file_hashed(href, &dest, BLOB_FILE_MAX)
            .map_err(|error| map_http_error("ota_blob", &error))?;
        if !downloaded.sha256_hex.eq_ignore_ascii_case(&want) {
            let _ = fs::remove_file(&dest);
            return Err("capsule_signature_required".into());
        }
    }
    Ok(())
}

fn map_http_error(prefix: &str, error: &str) -> String {
    if error == "spa_http_404" || error.ends_with("_404") {
        return "no_capsule".into();
    }
    format!("{prefix}:{error}")
}

/// Attest document for the edge (flags + device proof class). No secrets.
pub fn build_attest_request(state_root: &Path) -> Result<Value, String> {
    require_all_gates(state_root)?;
    let fingerprint = launcher_status::live_fingerprint_v2();
    let mut hasher = Sha256::new();
    hasher.update(fingerprint.as_bytes());
    let machine_fingerprint_hash = hex::encode(hasher.finalize());
    Ok(json!({
        "kind": "empiieros_ota_attest_v1",
        "fingerprint_scheme": "fp_v2",
        "machine_fingerprint_hash": machine_fingerprint_hash,
        "allow_fofline": true,
        "oidc_session": "present",
        "portal_hostmap": "applied",
        "note": "attest before update-capsule transfer; secrets omitted"
    }))
}

fn require_all_gates(state_root: &Path) -> Result<(), String> {
    match mutation_block(state_root) {
        Some(code) => Err(code.into()),
        None => Ok(()),
    }
}

fn observe_gates(state_root: &Path) -> GateView {
    let fingerprint = launcher_status::live_fingerprint_v2();
    let pin = launcher_pin::observe_flags(state_root, &fingerprint);
    let inner = launcher_inner::observe(state_root);
    let keys_path = state_root.join("device_keys").join("recipient.sk.hex");
    let (device_keys_ok, keys_block) = if !keys_path.exists() {
        (false, Some("device_keys_required"))
    } else if launcher_capsule::load_recipient_sk(state_root).is_none() {
        (false, Some("device_keys_invalid"))
    } else {
        (true, None)
    };
    let hardware_bound = pin.issued && pin.bound && pin.unlocked;
    let oidc = launcher_oidc::session_present(state_root);
    let portal_up = launcher_inner::portal_iface_up();
    let hostmap_applied = portal_hostmap::observe_default() == "applied";
    let plant = ota_capsule_store::plant_from_persisted_inner(state_root);
    let artifact_pk = ota_capsule_store::artifact_pk_present(state_root);
    let plant_mismatch = matches!(plant, Err(ref e) if e == "artifact_role_mismatch");
    let ota_helper = helper_installed();
    let staged = ota_capsule_store::is_current_ready(state_root);

    let mut first_block = None;
    if !pin.unlocked {
        first_block = Some("pin_not_unlocked");
    } else if !inner.entitlement_present {
        first_block = Some("license_required");
    } else if inner.allow_fofline != Some(true) {
        first_block = Some("allow_fofline_required");
    } else if let Some(code) = keys_block {
        first_block = Some(code);
    } else if !hardware_bound {
        first_block = Some("hardware_bind_required");
    } else if !oidc {
        first_block = Some("oidc_session_required");
    } else if !portal_up {
        first_block = Some("portal_required");
    } else if !hostmap_applied {
        first_block = Some("portal_hostmap_required");
    } else if plant_mismatch {
        first_block = Some("artifact_role_mismatch");
    } else if !artifact_pk {
        first_block = Some("artifact_pk_missing");
    } else if !ota_helper {
        first_block = Some("ota_helper_required");
    }

    GateView {
        pin_unlocked: pin.unlocked,
        license_ok: inner.entitlement_present,
        allow_fofline: inner.allow_fofline == Some(true),
        device_keys_ok,
        hardware_bound,
        oidc_session: oidc,
        portal_up,
        hostmap_applied,
        artifact_pk,
        ota_helper,
        staged,
        first_block,
    }
}

fn helper_process() -> bool {
    matches!(
        std::env::var("EMPIIEROS_OTA_HELPER").ok().as_deref(),
        Some("1") | Some("true") | Some("yes")
    )
}

fn helper_installed() -> bool {
    Path::new(HELPER).is_file()
        && Path::new("/etc/systemd/system").join(UNIT).is_file()
        && Path::new("/usr/bin/systemctl").is_file()
}

fn activate_via_helper(
    state_root: &Path,
    action: &str,
    timeout: Duration,
) -> Result<Value, String> {
    fs::create_dir_all(REQUEST_DIR).map_err(|error| format!("ota_run_dir:{error}"))?;
    let request = json!({
        "state_dir": state_root,
        "action": action,
    });
    atomic_write_file(
        Path::new(REQUEST_DIR),
        REQUEST_NAME,
        format!("{request}\n").as_bytes(),
        0o600,
    )
    .map_err(|error| format!("ota_request_write:{error}"))?;
    let _ = fs::remove_file(Path::new(REQUEST_DIR).join(RESULT_NAME));
    let started = crate::command_runner::run("/usr/bin/systemctl", &["start", UNIT], timeout)
        .map_err(|error| format!("ota_helper_start:{error}"))?;
    if !started.success {
        return Err(format!(
            "ota_helper_start:exit={}",
            started.exit_code.unwrap_or(-1)
        ));
    }
    let raw = fs::read_to_string(Path::new(REQUEST_DIR).join(RESULT_NAME))
        .map_err(|_| "ota_helper_result_missing".to_string())?;
    let parsed: Value =
        serde_json::from_str(&raw).map_err(|error| format!("ota_helper_result_json:{error}"))?;
    if parsed.get("ok").and_then(Value::as_bool) == Some(true) {
        return parsed
            .get("result")
            .cloned()
            .ok_or_else(|| "ota_helper_result_missing".to_string());
    }
    Err(parsed
        .get("error")
        .and_then(Value::as_str)
        .unwrap_or("ota_request_failed")
        .to_string())
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
        let root = std::env::temp_dir().join(format!("empiieros-ota-{nanos}"));
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn plant_license(root: &Path) {
        let dir = root.join("capsule_inner");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("flags.json"),
            r#"{"entitlement_present":true,"allow_fofline":true,"channel_present":true,"channel_valid":true,"channel_code":"valid_idle"}"#,
        )
        .unwrap();
    }

    fn plant_keys(root: &Path) {
        let dir = root.join("device_keys");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("recipient.sk.hex"),
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        )
        .unwrap();
    }

    fn plant_oidc(root: &Path) {
        launcher_oidc::plant_session(root);
    }

    #[test]
    fn status_and_request_refuse_without_unlock() {
        let root = temp_root();
        assert_eq!(status(&root, json!({})).unwrap_err(), "pin_not_issued");
        assert_eq!(request(&root, json!({})).unwrap_err(), "pin_not_issued");
        assert_eq!(
            apply(&root, json!({"confirm": "APPLY_OTA"})).unwrap_err(),
            "pin_not_issued"
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn gates_refuse_in_order_until_helper_or_portal() {
        let root = temp_root();
        launcher_pin::plant_unlocked(&root);
        assert_eq!(request(&root, json!({})).unwrap_err(), "license_required");
        plant_license(&root);
        assert_eq!(
            request(&root, json!({})).unwrap_err(),
            "device_keys_required"
        );
        plant_keys(&root);
        assert_eq!(
            request(&root, json!({})).unwrap_err(),
            "oidc_session_required"
        );
        plant_oidc(&root);
        let err = request(&root, json!({})).unwrap_err();
        assert!(
            err == "portal_required"
                || err == "portal_hostmap_required"
                || err == "artifact_pk_missing"
                || err == "ota_helper_required"
                || err == "secure_transport_required",
            "unexpected refuse: {err}"
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn apply_requires_exact_confirm_then_not_staged() {
        let root = temp_root();
        launcher_pin::plant_unlocked(&root);
        plant_license(&root);
        plant_keys(&root);
        plant_oidc(&root);
        let early = apply(&root, json!({})).unwrap_err();
        if early == "confirmation_required" {
            assert_eq!(
                apply(&root, json!({"confirm": "APPLY_OTA"})).unwrap_err(),
                "ota_not_staged"
            );
        } else {
            assert!(
                early == "portal_required"
                    || early == "portal_hostmap_required"
                    || early == "license_required"
                    || early == "oidc_session_required"
                    || early == "artifact_pk_missing",
                "unexpected: {early}"
            );
        }
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn status_reports_flags_after_unlock() {
        let root = temp_root();
        launcher_pin::plant_unlocked(&root);
        let view = status(&root, json!({})).unwrap();
        assert_eq!(view["pin_unlocked"], true);
        assert_eq!(view["ota_apply"], "idle");
        assert_eq!(view["ota_gate"], "license_required");
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn status_plants_artifact_pk_from_persisted_inner() {
        let root = temp_root();
        launcher_pin::plant_unlocked(&root);
        plant_license(&root);
        let hex = "cd".repeat(32);
        fs::write(
            root.join("capsule_inner").join("entitlement.json"),
            json!({ "v": 1, "payload": { "ota_artifact_pk": hex }, "sig": "00" }).to_string(),
        )
        .unwrap();
        let view = status(&root, json!({})).unwrap();
        assert_eq!(view["artifact_pk"], "present");
        assert_ne!(view["ota_gate"], "artifact_pk_missing");
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn parse_attest_grant_accepts_portal_path_current_only() {
        let ok = r#"{"ok":true,"grant":"portal_path_v1","capsule_href":"/empiieros-ota/v1/current.json"}"#;
        assert_eq!(
            parse_attest_grant(ok).unwrap(),
            "/empiieros-ota/v1/current.json"
        );
        assert_eq!(
            parse_attest_grant(r#"{"ok":false}"#).unwrap_err(),
            "ota_grant_denied"
        );
        assert_eq!(
            parse_attest_grant(
                r#"{"ok":true,"grant":"portal_path_v1","capsule_href":"/empiieros-ota/../etc/passwd"}"#
            )
            .unwrap_err(),
            "path_escape"
        );
        assert_eq!(
            parse_attest_grant(
                r#"{"ok":true,"grant":"portal_path_v1","capsule_href":"/fofline-updates/x"}"#
            )
            .unwrap_err(),
            "path_escape"
        );
        assert_eq!(
            parse_attest_grant(
                r#"{"ok":true,"grant":"portal_path_v1","capsule_href":"/empiieros-ota/v1/other.json"}"#
            )
            .unwrap_err(),
            "path_escape"
        );
    }

    #[test]
    fn e3_stage_from_env_capsule_and_blobs() {
        // Lab Measure: EMPIIEROS_OTA_E3_LIVE=1 + state/capsule/blobs/pk paths.
        if std::env::var("EMPIIEROS_OTA_E3_LIVE").ok().as_deref() != Some("1") {
            return;
        }
        let state = PathBuf::from(
            std::env::var("EMPIIEROS_OTA_E3_STATE").expect("EMPIIEROS_OTA_E3_STATE"),
        );
        let capsule = PathBuf::from(
            std::env::var("EMPIIEROS_OTA_E3_CAPSULE").expect("EMPIIEROS_OTA_E3_CAPSULE"),
        );
        let blobs = PathBuf::from(
            std::env::var("EMPIIEROS_OTA_E3_BLOBS").expect("EMPIIEROS_OTA_E3_BLOBS"),
        );
        let pk = std::env::var("EMPIIEROS_OTA_E3_PK").expect("EMPIIEROS_OTA_E3_PK");
        ota_capsule_store::plant_artifact_pk(&state, &pk).unwrap();
        let doc: Value = serde_json::from_str(&fs::read_to_string(&capsule).unwrap()).unwrap();
        let out =
            ota_capsule_store::verify_and_stage_with_blobs(&state, &doc, Some(&blobs)).unwrap();
        assert_eq!(out["ota_staged"], true);
        assert_eq!(out["ota_apply"], "ready");
        assert!(ota_capsule_store::is_current_ready(&state));
    }
}
