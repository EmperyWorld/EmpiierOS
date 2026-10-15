//! Capsule OTA cutover — P4 destination table plant (E4).
//!
//! Contract: `session/OTA-CAPSULE.md` · `docs/plans/TZ-CAPSULE-OTA.md` §Cutover.
//! Atomic `dest.new` → rename. Writes `ota-capsule/apply.json` and
//! `seat_shield/ota_baseline.json`. Lab: `EMPIIEROS_OTA_CUTOVER_ROOT` remaps
//! absolute destinations under a temp root (unit tests / dry-run).

use crate::command_runner;
use crate::ota_capsule_store::{self, MANIFEST_NAME, OTA_DIR};
use crate::state_store::atomic_write_file;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::Duration;

const APPLY_JSON: &str = "apply.json";
const UNIT_RUNTIME: &str = "empiieros-runtime.service";
const UNIT_SHELL: &str = "empiieros-shell-gui.service";
const UNIT_WATCH: &str = "empiieros-seat-shield-watch.service";
const WATCH_REL: &str = "helpers/seat-shield-watch";
const RESTART_TIMEOUT: Duration = Duration::from_secs(30);

/// Map allowlisted capsule relative path → (absolute dest, mode).
pub fn map_destination(rel: &str) -> Result<(PathBuf, u32), String> {
    let t = rel.trim();
    if !ota_capsule_store::is_path_safe(t) {
        return Err("path_escape".into());
    }
    let (abs, mode) = match t {
        "runtime/empiieros-runtime" => ("/usr/local/bin/empiieros-runtime", 0o755),
        "runtime/empiieros-shell-gui" => ("/usr/local/bin/empiieros-shell-gui", 0o755),
        "helpers/empiieros-air-enroll" => ("/usr/local/sbin/empiieros-air-enroll", 0o755),
        "helpers/empiieros-portal-up" => ("/usr/local/sbin/empiieros-portal-up", 0o755),
        "helpers/empiieros-spa-fetch" => ("/usr/local/sbin/empiieros-spa-fetch", 0o755),
        "helpers/empiieros-ota-fetch" => ("/usr/local/sbin/empiieros-ota-fetch", 0o755),
        "helpers/empiieros-wg" => ("/usr/local/sbin/empiieros-wg", 0o755),
        "helpers/empiieros-audio-volume" => ("/usr/local/sbin/empiieros-audio-volume", 0o755),
        "helpers/empiieros-wifi-radio" => ("/usr/local/sbin/empiieros-wifi-radio", 0o755),
        "helpers/empiieros-wifi-restore" => ("/usr/local/sbin/empiieros-wifi-restore", 0o755),
        "helpers/empiieros-time-sync" => ("/usr/local/sbin/empiieros-time-sync", 0o755),
        "helpers/seat-shield-fw" => ("/usr/libexec/empiieros-seat-shield-fw", 0o755),
        "helpers/seat-shield-watch" => ("/usr/libexec/empiieros-seat-shield-watch", 0o755),
        "helpers/seat-shield-probe" => ("/usr/libexec/empiieros-seat-shield-probe", 0o755),
        other if other.starts_with("shell/") => {
            let rest = &other["shell/".len()..];
            if rest.is_empty() || rest.contains("..") || rest.starts_with('/') {
                return Err("path_escape".into());
            }
            let dest = format!("/opt/empiieros/shell/{rest}");
            return Ok((rebase_dest(PathBuf::from(dest))?, 0o644));
        }
        _ => return Err("path_escape".into()),
    };
    Ok((rebase_dest(PathBuf::from(abs))?, mode))
}

fn rebase_dest(abs: PathBuf) -> Result<PathBuf, String> {
    if let Ok(root) = std::env::var("EMPIIEROS_OTA_CUTOVER_ROOT") {
        let root = PathBuf::from(root);
        // Strip leading `/` so join stays under lab root.
        let rel = abs.strip_prefix("/").unwrap_or(abs.as_path());
        return Ok(root.join(rel));
    }
    if !abs.is_absolute() {
        return Err("path_escape".into());
    }
    Ok(abs)
}

/// Apply staged `os_component` capsule to allowlisted destinations.
pub fn apply_current(state_root: &Path) -> Result<Value, String> {
    let current = ota_capsule_store::current_dir(state_root);
    let manifest_path = current.join(MANIFEST_NAME);
    if !manifest_path.is_file() {
        return Err("ota_not_staged".into());
    }
    let raw = fs::read_to_string(&manifest_path).map_err(|e| format!("ota_manifest:{e}"))?;
    let doc: Value = serde_json::from_str(&raw).map_err(|_| "capsule_signature_required".to_string())?;
    let pk = ota_capsule_store::resolve_artifact_pk(state_root)?;
    let payload = reverify_payload(&doc, &pk)?;
    let channel = payload
        .get("channel")
        .and_then(Value::as_str)
        .unwrap_or("");
    if channel != "os_component" {
        // FoFline pack promote is a separate channel — must not rewrite /usr.
        return Err("os_before_payload".into());
    }
    let files = payload
        .get("files")
        .and_then(Value::as_array)
        .ok_or_else(|| "payload_incomplete".to_string())?;
    if files.is_empty() || files.len() > 64 {
        return Err("payload_incomplete".into());
    }

    let build = payload
        .get("app_build_id")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let mut digests: BTreeMap<String, String> = BTreeMap::new();
    let mut applied_paths: Vec<String> = Vec::new();

    for entry in files {
        let rel = entry
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| "payload_incomplete".to_string())?;
        let want = entry
            .get("sha256")
            .and_then(Value::as_str)
            .ok_or_else(|| "payload_incomplete".to_string())?
            .trim()
            .to_ascii_lowercase();
        let (dest, mode) = map_destination(rel)?;
        let src = current.join(rel);
        if !src.is_file() {
            return Err("ota_staging_incomplete".into());
        }
        let bytes = fs::read(&src).map_err(|e| format!("ota_read:{e}"))?;
        let got = hex::encode(Sha256::digest(&bytes));
        if got != want {
            return Err("capsule_signature_required".into());
        }
        install_atomic(&dest, &bytes, mode)?;
        let dest_key = dest_key_for_baseline(&dest)?;
        digests.insert(dest_key, got);
        applied_paths.push(rel.to_string());
    }

    write_apply_json(state_root, build, channel, &digests)?;
    seat_shield_core::write_ota_baseline(state_root, &digests)
        .map_err(|e| format!("ota_baseline_write:{e}"))?;

    let restarted = restart_allowlisted(&applied_paths)?;
    let reload_shell = applied_paths.iter().any(|p| p.starts_with("shell/"));
    Ok(json!({
        "ota_applied": true,
        "ota_channel": channel,
        "ota_app_build_id": build,
        "ota_files": applied_paths.len(),
        "ota_restarted": restarted,
        "ota_reload_shell": reload_shell,
        "ota_baseline": "written",
        "detail": "os_component cutover applied; baseline written"
    }))
}

/// Status chip for glass: idle / ready / applied / os_before_payload.
///
/// `applied` only when the last cutover matches the staged capsule (or there
/// is no newer stage). A new Request after a prior apply must show `ready`.
pub fn apply_status_chip(state_root: &Path) -> &'static str {
    match staged_channel(state_root).as_deref() {
        Some("fofline_update") => return "os_before_payload",
        Some("os_component") => {
            let staged = staged_build_id(state_root);
            let applied = applied_build_id(state_root);
            match (staged.as_deref(), applied.as_deref()) {
                (Some(s), Some(a)) if s == a => return "applied",
                (Some(_), _) => return "ready",
                _ => {}
            }
        }
        _ => {}
    }
    if applied_build_id(state_root).is_some() {
        return "applied";
    }
    "idle"
}

pub fn apply_json_present(state_root: &Path) -> bool {
    state_root.join(OTA_DIR).join(APPLY_JSON).is_file()
}

/// Build id from the last successful cutover (`ota-capsule/apply.json`).
pub fn applied_build_id(state_root: &Path) -> Option<String> {
    if !apply_json_present(state_root) {
        return None;
    }
    let path = state_root.join(OTA_DIR).join(APPLY_JSON);
    let raw = fs::read_to_string(path).ok()?;
    let doc: Value = serde_json::from_str(&raw).ok()?;
    doc.get("app_build_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

/// Build id from the staged capsule manifest (`ota-capsule/current/`).
pub fn staged_build_id(state_root: &Path) -> Option<String> {
    let path = ota_capsule_store::current_dir(state_root).join(MANIFEST_NAME);
    let raw = fs::read_to_string(path).ok()?;
    let doc: Value = serde_json::from_str(&raw).ok()?;
    let payload = doc.get("payload")?;
    payload
        .get("app_build_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

/// Glass next-step token: fix_gates | check | install | current.
pub fn next_action(state_root: &Path, gates_ok: bool) -> &'static str {
    if !gates_ok {
        return "fix_gates";
    }
    match apply_status_chip(state_root) {
        "ready" => "install",
        "os_before_payload" => "install",
        "applied" => "current",
        _ => "check",
    }
}

fn staged_channel(state_root: &Path) -> Option<String> {
    let path = ota_capsule_store::current_dir(state_root).join(MANIFEST_NAME);
    let raw = fs::read_to_string(path).ok()?;
    let doc: Value = serde_json::from_str(&raw).ok()?;
    let payload = doc.get("payload")?;
    payload
        .get("channel")
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn dest_key_for_baseline(dest: &Path) -> Result<String, String> {
    // Baseline always stores production absolute paths (not lab rebase).
    if let Ok(root) = std::env::var("EMPIIEROS_OTA_CUTOVER_ROOT") {
        let root = PathBuf::from(root);
        if let Ok(rel) = dest.strip_prefix(&root) {
            return Ok(format!("/{}", rel.to_string_lossy()));
        }
    }
    Ok(dest.to_string_lossy().into_owned())
}

fn install_atomic(dest: &Path, bytes: &[u8], mode: u32) -> Result<(), String> {
    let parent = dest.parent().ok_or_else(|| "path_escape".to_string())?;
    fs::create_dir_all(parent).map_err(|e| format!("ota_dest_dir:{e}"))?;
    let name = dest
        .file_name()
        .ok_or_else(|| "path_escape".to_string())?
        .to_string_lossy();
    let new_path = parent.join(format!("{name}.new"));
    {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .mode(mode)
            .open(&new_path)
            .map_err(|e| format!("ota_dest_new:{e}"))?;
        file.write_all(bytes)
            .map_err(|e| format!("ota_dest_write:{e}"))?;
        file.sync_all().map_err(|e| format!("ota_dest_fsync:{e}"))?;
    }
    fs::set_permissions(&new_path, fs::Permissions::from_mode(mode))
        .map_err(|e| format!("ota_dest_mode:{e}"))?;
    fs::rename(&new_path, dest).map_err(|e| format!("ota_dest_rename:{e}"))?;
    Ok(())
}

fn write_apply_json(
    state_root: &Path,
    build: &str,
    channel: &str,
    digests: &BTreeMap<String, String>,
) -> Result<(), String> {
    let dir = state_root.join(OTA_DIR);
    fs::create_dir_all(&dir).map_err(|e| format!("ota_apply_dir:{e}"))?;
    let mut files = Map::new();
    for (k, v) in digests {
        files.insert(k.clone(), Value::String(v.clone()));
    }
    let body = json!({
        "v": 1,
        "channel": channel,
        "app_build_id": build,
        "files": files,
    });
    let raw = serde_json::to_vec_pretty(&body).map_err(|e| format!("ota_apply_json:{e}"))?;
    atomic_write_file(&dir, APPLY_JSON, &raw, 0o600).map_err(|e| format!("ota_apply_write:{e}"))?;
    Ok(())
}

/// Units whose **binaries** changed. `shell/` assets do not restart GUI.
fn units_to_restart(applied: &[String]) -> Vec<&'static str> {
    let mut out = Vec::new();
    if applied.iter().any(|p| p == "runtime/empiieros-shell-gui") {
        out.push(UNIT_SHELL);
    }
    if applied.iter().any(|p| p == WATCH_REL) {
        out.push(UNIT_WATCH);
    }
    if applied.iter().any(|p| p == "runtime/empiieros-runtime") {
        out.push(UNIT_RUNTIME);
    }
    out
}

fn restart_allowlisted(applied: &[String]) -> Result<Vec<String>, String> {
    if std::env::var("EMPIIEROS_OTA_CUTOVER_ROOT").is_ok() {
        // Lab rebase: never touch host units.
        return Ok(vec![]);
    }
    if !Path::new("/usr/bin/systemctl").is_file() {
        return Ok(vec![]);
    }
    let mut done = Vec::new();
    for unit in units_to_restart(applied) {
        if !unit_exists(unit) {
            continue;
        }
        if unit == UNIT_RUNTIME || unit == UNIT_SHELL {
            // Self/glass restart before the IPC newline is
            // `invalid runtime response framing`. Defer past the reply.
            if schedule_try_restart(unit).is_ok() {
                done.push(format!("{unit}#deferred"));
            }
            continue;
        }
        try_restart(unit)?;
        done.push(unit.into());
    }
    Ok(done)
}

fn schedule_try_restart(unit: &str) -> Result<(), String> {
    if !Path::new("/usr/bin/systemd-run").is_file() {
        return Err("ota_restart_schedule".into());
    }
    let result = command_runner::run(
        "/usr/bin/systemd-run",
        &[
            "--collect",
            "--quiet",
            "--on-active=3s",
            "/usr/bin/systemctl",
            "try-restart",
            unit,
        ],
        Duration::from_secs(8),
    )
    .map_err(|e| format!("ota_restart_schedule:{e}"))?;
    if !result.success {
        return Err(format!("ota_restart_schedule:{unit}"));
    }
    Ok(())
}

fn unit_exists(unit: &str) -> bool {
    command_runner::run(
        "/usr/bin/systemctl",
        &["cat", unit],
        Duration::from_secs(5),
    )
    .map(|r| r.success)
    .unwrap_or(false)
}

fn try_restart(unit: &str) -> Result<(), String> {
    let result = command_runner::run(
        "/usr/bin/systemctl",
        &["try-restart", unit],
        RESTART_TIMEOUT,
    )
    .map_err(|e| format!("ota_restart:{e}"))?;
    if !result.success {
        return Err(format!("ota_restart_failed:{unit}"));
    }
    Ok(())
}

fn reverify_payload(doc: &Value, pk: &[u8; 32]) -> Result<Value, String> {
    // Re-use store verify by staging round-trip is heavy; duplicate minimal check.
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};
    if doc.get("v").and_then(Value::as_u64) != Some(1) {
        return Err("capsule_signature_required".into());
    }
    let payload = doc
        .get("payload")
        .cloned()
        .ok_or_else(|| "capsule_signature_required".to_string())?;
    let sig_hex = doc
        .get("sig")
        .and_then(Value::as_str)
        .ok_or_else(|| "capsule_signature_required".to_string())?;
    let verifying =
        VerifyingKey::from_bytes(pk).map_err(|_| "artifact_role_mismatch".to_string())?;
    let sorted = sort_value(payload.clone());
    let msg = serde_json::to_vec(&sorted).map_err(|e| format!("canonical_json:{e}"))?;
    let sig_bytes = hex::decode(sig_hex.trim()).map_err(|_| "capsule_signature_required".to_string())?;
    if sig_bytes.len() != 64 {
        return Err("capsule_signature_required".into());
    }
    let mut sig_arr = [0u8; 64];
    sig_arr.copy_from_slice(&sig_bytes);
    verifying
        .verify(&msg, &Signature::from_bytes(&sig_arr))
        .map_err(|_| "capsule_signature_required".to_string())?;
    Ok(payload)
}

fn sort_value(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<String> = map.keys().cloned().collect();
            keys.sort();
            let mut out = Map::new();
            for k in keys {
                if let Some(v) = map.get(&k) {
                    out.insert(k, sort_value(v.clone()));
                }
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(items.into_iter().map(sort_value).collect()),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn temp_root(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("empiieros-cutover-{label}-{nanos}"));
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn plant_pk(state: &Path, pk: &[u8; 32]) {
        ota_capsule_store::plant_artifact_pk(state, &hex::encode(pk)).unwrap();
    }

    fn sign_doc(sk: &[u8; 32], payload: Value) -> Value {
        let sorted = sort_value(payload.clone());
        let msg = serde_json::to_vec(&sorted).unwrap();
        let sig = SigningKey::from_bytes(sk).sign(&msg);
        json!({
            "v": 1,
            "payload": payload,
            "sig": hex::encode(sig.to_bytes())
        })
    }

    #[test]
    fn units_to_restart_ignores_shell_assets() {
        assert!(units_to_restart(&["shell/index.html".into()]).is_empty());
        assert_eq!(
            units_to_restart(&["runtime/empiieros-runtime".into()]),
            vec![UNIT_RUNTIME]
        );
        assert_eq!(
            units_to_restart(&["runtime/empiieros-shell-gui".into()]),
            vec![UNIT_SHELL]
        );
        assert_eq!(
            units_to_restart(&[WATCH_REL.into()]),
            vec![UNIT_WATCH]
        );
    }

    #[test]
    fn map_rejects_unmapped_runtime_fixture() {
        assert_eq!(
            map_destination("runtime/fixture.txt").unwrap_err(),
            "path_escape"
        );
        assert!(map_destination("runtime/empiieros-runtime").is_ok());
        assert!(map_destination("runtime/empiieros-shell-gui").is_ok());
        assert!(map_destination("shell/index.html").is_ok());
        assert_eq!(map_destination("shell/").unwrap_err(), "path_escape");
    }

    #[test]
    fn apply_os_component_under_lab_root_writes_baseline() {
        let _guard = ENV_LOCK.lock().unwrap();
        let state = temp_root("state");
        let floor = temp_root("floor");
        std::env::set_var("EMPIIEROS_OTA_CUTOVER_ROOT", &floor);
        let sk = [0x42u8; 32];
        let pk = SigningKey::from_bytes(&sk).verifying_key().to_bytes();
        plant_pk(&state, &pk);
        let body = b"runtime-bin-e4";
        let payload = json!({
            "kind": ota_capsule_store::KIND,
            "channel": "os_component",
            "version": "0.0.1-e4",
            "app_build_id": "e4-cutover-1",
            "files": [{
                "path": "runtime/empiieros-runtime",
                "sha256": hex::encode(Sha256::digest(body)),
                "content_base64": base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    body
                ),
            }]
        });
        let doc = sign_doc(&sk, payload);
        ota_capsule_store::verify_and_stage(&state, &doc).unwrap();
        let out = apply_current(&state).unwrap();
        assert_eq!(out["ota_applied"], true);
        assert_eq!(out["ota_baseline"], "written");
        assert_eq!(out["ota_reload_shell"], false);
        assert_eq!(out["ota_restarted"], json!([]));
        let dest = floor.join("usr/local/bin/empiieros-runtime");
        assert_eq!(fs::read(&dest).unwrap(), body);
        let mode = fs::metadata(&dest).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o755);
        assert!(state.join(OTA_DIR).join(APPLY_JSON).is_file());
        assert_eq!(
            seat_shield_core::observe_ota_baseline(&state).as_str(),
            "ok"
        );
        // Baseline keys are production absolute paths.
        let baseline_raw =
            fs::read_to_string(state.join("seat_shield").join("ota_baseline.json")).unwrap();
        assert!(baseline_raw.contains("/usr/local/bin/empiieros-runtime"));
        assert_eq!(apply_status_chip(&state), "applied");
        assert_eq!(applied_build_id(&state).as_deref(), Some("e4-cutover-1"));
        assert_eq!(staged_build_id(&state).as_deref(), Some("e4-cutover-1"));
        assert_eq!(next_action(&state, true), "current");
        std::env::remove_var("EMPIIEROS_OTA_CUTOVER_ROOT");
        let _ = fs::remove_dir_all(&state);
        let _ = fs::remove_dir_all(&floor);
    }

    #[test]
    fn restage_newer_build_shows_ready_not_stuck_applied() {
        let _guard = ENV_LOCK.lock().unwrap();
        let state = temp_root("restage");
        let floor = temp_root("floor-restage");
        std::env::set_var("EMPIIEROS_OTA_CUTOVER_ROOT", &floor);
        let sk = [0x44u8; 32];
        let pk = SigningKey::from_bytes(&sk).verifying_key().to_bytes();
        plant_pk(&state, &pk);
        let body_a = b"runtime-bin-a";
        let payload_a = json!({
            "kind": ota_capsule_store::KIND,
            "channel": "os_component",
            "version": "0.0.1-a",
            "app_build_id": "build-a",
            "files": [{
                "path": "runtime/empiieros-runtime",
                "sha256": hex::encode(Sha256::digest(body_a)),
                "content_base64": base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    body_a
                ),
            }]
        });
        ota_capsule_store::verify_and_stage(&state, &sign_doc(&sk, payload_a)).unwrap();
        assert_eq!(apply_status_chip(&state), "ready");
        apply_current(&state).unwrap();
        assert_eq!(apply_status_chip(&state), "applied");

        let body_b = b"runtime-bin-b";
        let payload_b = json!({
            "kind": ota_capsule_store::KIND,
            "channel": "os_component",
            "version": "0.0.1-b",
            "app_build_id": "build-b",
            "files": [{
                "path": "runtime/empiieros-runtime",
                "sha256": hex::encode(Sha256::digest(body_b)),
                "content_base64": base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    body_b
                ),
            }]
        });
        ota_capsule_store::verify_and_stage(&state, &sign_doc(&sk, payload_b)).unwrap();
        assert_eq!(staged_build_id(&state).as_deref(), Some("build-b"));
        assert_eq!(applied_build_id(&state).as_deref(), Some("build-a"));
        assert_eq!(apply_status_chip(&state), "ready");
        assert_eq!(next_action(&state, true), "install");
        std::env::remove_var("EMPIIEROS_OTA_CUTOVER_ROOT");
        let _ = fs::remove_dir_all(&state);
        let _ = fs::remove_dir_all(&floor);
    }

    #[test]
    fn fofline_channel_refuses_os_paths() {
        let _guard = ENV_LOCK.lock().unwrap();
        let state = temp_root("fofline");
        let floor = temp_root("floor-ff");
        std::env::set_var("EMPIIEROS_OTA_CUTOVER_ROOT", &floor);
        let sk = [0x43u8; 32];
        let pk = SigningKey::from_bytes(&sk).verifying_key().to_bytes();
        plant_pk(&state, &pk);
        let body = b"pack-only";
        let payload = json!({
            "kind": ota_capsule_store::KIND,
            "channel": "fofline_update",
            "version": "0.0.1-ff",
            "app_build_id": "ff-1",
            "files": [{
                "path": "runtime/empiieros-runtime",
                "sha256": hex::encode(Sha256::digest(body)),
                "content_base64": base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    body
                ),
            }]
        });
        let doc = sign_doc(&sk, payload);
        ota_capsule_store::verify_and_stage(&state, &doc).unwrap();
        assert_eq!(apply_current(&state).unwrap_err(), "os_before_payload");
        assert!(!floor.join("usr/local/bin/empiieros-runtime").exists());
        std::env::remove_var("EMPIIEROS_OTA_CUTOVER_ROOT");
        let _ = fs::remove_dir_all(&state);
        let _ = fs::remove_dir_all(&floor);
    }
}
