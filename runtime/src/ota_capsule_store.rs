//! Capsule-gated OTA store — staging / verify / current.
//!
//! Artifact-role Ed25519 over `{v,payload,sig}`. Entitlement key must not
//! verify these docs. Cutover lives in `ota_cutover` (E4).

use crate::state_store::atomic_write_file;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

pub const OTA_DIR: &str = "ota-capsule";
pub const STAGING: &str = "staging";
pub const CURRENT: &str = "current";
pub const PREVIOUS: &str = "previous";
pub const MANIFEST_NAME: &str = "manifest.json";
pub const KIND: &str = "empiieros_ota_capsule_v1";
pub const ARTIFACT_PK_FILE: &str = "artifact.pk.hex";
pub const ARTIFACT_PK_DIR: &str = "ota";
/// Production blob href prefix (edge nginx `location /v1/blobs/`).
pub const BLOB_HREF_PREFIX: &str = "/empiieros-ota/v1/blobs/";

const MAX_FILES: usize = 64;
/// Tiny inline fixtures only — production OS binaries use href + blob dir.
const MAX_INLINE_FILE_BYTES: usize = 256 * 1024;
const MAX_INLINE_TOTAL_BYTES: usize = 1024 * 1024;
/// Blob-ref path (matches CP ingest / edge tmpfs caps).
const MAX_BLOB_FILE_BYTES: usize = 256 * 1024 * 1024;
const MAX_BLOB_TOTAL_BYTES: usize = 512 * 1024 * 1024;

pub fn root_dir(state: &Path) -> PathBuf {
    state.join(OTA_DIR)
}

pub fn staging_dir(state: &Path) -> PathBuf {
    root_dir(state).join(STAGING)
}

pub fn current_dir(state: &Path) -> PathBuf {
    root_dir(state).join(CURRENT)
}

pub fn is_current_ready(state: &Path) -> bool {
    current_dir(state).join(MANIFEST_NAME).is_file()
}

pub fn resolve_artifact_pk(state: &Path) -> Result<[u8; 32], String> {
    if let Ok(raw) = std::env::var("EMPIIEROS_OTA_ARTIFACT_PK") {
        return parse_pk_hex(&raw).map_err(|_| "artifact_role_mismatch".to_string());
    }
    let path = state.join(ARTIFACT_PK_DIR).join(ARTIFACT_PK_FILE);
    if !path.is_file() {
        return Err("artifact_pk_missing".into());
    }
    let raw = fs::read_to_string(&path).map_err(|_| "artifact_pk_missing".to_string())?;
    parse_pk_hex(&raw).map_err(|_| "artifact_role_mismatch".to_string())
}

/// True when env or `{state}/ota/artifact.pk.hex` can resolve a 32-byte PK.
pub fn artifact_pk_present(state: &Path) -> bool {
    resolve_artifact_pk(state).is_ok()
}

/// Plant verify-only artifact PK at `{state}/ota/artifact.pk.hex` (`0600`).
/// Never accepts or writes an SK. CP ceremony only.
#[allow(dead_code)] // public plant API; CP/lab scripts + tests call it
pub fn plant_artifact_pk(state: &Path, pk_hex: &str) -> Result<(), String> {
    let pk = parse_pk_hex(pk_hex).map_err(|_| "artifact_role_mismatch".to_string())?;
    let dir = state.join(ARTIFACT_PK_DIR);
    fs::create_dir_all(&dir).map_err(|error| format!("ota_pk_dir:{error}"))?;
    set_dir_mode_0700(&dir)?;
    atomic_write_file(&dir, ARTIFACT_PK_FILE, format!("{}\n", hex::encode(pk)).as_bytes(), 0o600)
        .map_err(|error| format!("ota_pk_plant:{error}"))?;
    Ok(())
}

/// Verify-only PK from a license entitlement (`ota_artifact_pk`) and/or a
/// sibling `artifact.pk.hex` next to the `.ecap`. Never an SK. Both present
/// must match. Neither present → `"missing"` (license still binds).
pub fn plant_from_license(
    state: &Path,
    entitlement: Option<&Value>,
    sibling_dir: Option<&Path>,
) -> Result<&'static str, String> {
    let from_ent = license_artifact_pk(entitlement)?;
    let from_sib = match sibling_dir {
        Some(dir) => sibling_artifact_pk(dir)?,
        None => None,
    };
    let pk = match (from_ent, from_sib) {
        (Some(a), Some(b)) if a != b => return Err("artifact_role_mismatch".into()),
        (Some(pk), _) | (None, Some(pk)) => pk,
        (None, None) => return Ok("missing"),
    };
    plant_artifact_pk(state, &hex::encode(pk))?;
    Ok("planted")
}

/// Plant from `{state}/capsule_inner/entitlement.json` when the seat
/// already bound a license (Air spent) but living Runtime had no plant.
/// Already-present PK is left alone. Never an SK.
pub fn plant_from_persisted_inner(state: &Path) -> Result<&'static str, String> {
    if artifact_pk_present(state) {
        return Ok("present");
    }
    let path = state.join("capsule_inner").join("entitlement.json");
    if !path.is_file() {
        return Ok("missing");
    }
    let raw = fs::read_to_string(&path).map_err(|error| format!("ota_pk_inner:{error}"))?;
    let doc: Value = serde_json::from_str(&raw).map_err(|_| "artifact_role_mismatch".to_string())?;
    plant_from_license(state, Some(&doc), None)
}

fn license_artifact_pk(entitlement: Option<&Value>) -> Result<Option<[u8; 32]>, String> {
    let Some(doc) = entitlement else {
        return Ok(None);
    };
    let raw = doc
        .get("ota_artifact_pk")
        .or_else(|| {
            doc.get("payload")
                .and_then(Value::as_object)
                .and_then(|obj| obj.get("ota_artifact_pk"))
        })
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty());
    let Some(hex) = raw else {
        return Ok(None);
    };
    parse_pk_hex(hex)
        .map(Some)
        .map_err(|_| "artifact_role_mismatch".to_string())
}

fn sibling_artifact_pk(dir: &Path) -> Result<Option<[u8; 32]>, String> {
    let path = dir.join(ARTIFACT_PK_FILE);
    if !path.is_file() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).map_err(|_| "artifact_role_mismatch".to_string())?;
    parse_pk_hex(&raw)
        .map(Some)
        .map_err(|_| "artifact_role_mismatch".to_string())
}

/// Verify signed OTA capsule doc and promote into `{state}/ota-capsule/current/`.
/// Inline `content_base64` fixtures only — production uses [`verify_and_stage_with_blobs`].
#[allow(dead_code)] // public API for fixtures / lab callers without blob_dir
pub fn verify_and_stage(state: &Path, doc: &Value) -> Result<Value, String> {
    verify_and_stage_with_blobs(state, doc, None)
}

/// Verify + stage. When entries use `href` under [`BLOB_HREF_PREFIX`], `blob_dir`
/// must contain files named by the entry `sha256` hex (helper download tree).
pub fn verify_and_stage_with_blobs(
    state: &Path,
    doc: &Value,
    blob_dir: Option<&Path>,
) -> Result<Value, String> {
    let pk = resolve_artifact_pk(state)?;
    let payload = verify_signed_payload(doc, &pk)?;
    let kind = payload
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or("");
    if kind != KIND {
        return Err("artifact_role_mismatch".into());
    }
    let channel = payload
        .get("channel")
        .and_then(Value::as_str)
        .unwrap_or("");
    if channel != "os_component" && channel != "fofline_update" {
        return Err("artifact_role_mismatch".into());
    }
    let files = payload
        .get("files")
        .and_then(Value::as_array)
        .ok_or_else(|| "payload_incomplete".to_string())?;
    if files.is_empty() {
        return Err("payload_incomplete".into());
    }
    if files.len() > MAX_FILES {
        return Err("path_escape".into());
    }

    let staging = staging_dir(state);
    let _ = fs::remove_dir_all(&staging);
    fs::create_dir_all(&staging).map_err(|error| format!("ota_staging:{error}"))?;
    set_dir_mode_0700(&staging)?;

    let mut total = 0usize;
    let mut any_href = false;
    for entry in files {
        let rel = entry
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| "payload_incomplete".to_string())?;
        if !is_path_safe(rel) {
            return Err("path_escape".into());
        }
        let want_hash = entry
            .get("sha256")
            .and_then(Value::as_str)
            .ok_or_else(|| "payload_incomplete".to_string())?;
        let want_norm = want_hash.trim().to_ascii_lowercase();
        if !is_sha256_hex(&want_norm) {
            return Err("payload_incomplete".into());
        }
        let href_entry = entry_is_href(entry);
        if href_entry {
            any_href = true;
        }
        let bytes = resolve_entry_bytes(entry, &want_norm, blob_dir)?;
        let max_file = if href_entry {
            MAX_BLOB_FILE_BYTES
        } else {
            MAX_INLINE_FILE_BYTES
        };
        if bytes.len() > max_file {
            return Err("path_escape".into());
        }
        total = total.saturating_add(bytes.len());
        let max_total = if any_href {
            MAX_BLOB_TOTAL_BYTES
        } else {
            MAX_INLINE_TOTAL_BYTES
        };
        if total > max_total {
            return Err("path_escape".into());
        }
        let got = hex::encode(Sha256::digest(&bytes));
        if got != want_norm {
            return Err("capsule_signature_required".into());
        }
        write_relative(&staging, rel, &bytes)?;
    }

    let manifest_bytes = serde_json::to_vec_pretty(doc).map_err(|e| format!("ota_manifest:{e}"))?;
    atomic_write_file(&staging, MANIFEST_NAME, &manifest_bytes, 0o600)
        .map_err(|error| format!("ota_manifest:{error}"))?;

    promote_staging(state)?;
    let build = payload
        .get("app_build_id")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    // Observe-only Shield event — fail-soft so stage still succeeds.
    let _ = seat_shield_core::emit_ota_capsule_staged(state, build, channel);
    Ok(json!({
        "ota_staged": true,
        "ota_channel": channel,
        "ota_app_build_id": build,
        "ota_apply": "ready",
        "detail": "verified artifact staged; typed APPLY_OTA runs cutover"
    }))
}

fn entry_is_href(entry: &Value) -> bool {
    entry
        .get("href")
        .and_then(Value::as_str)
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false)
}

fn resolve_entry_bytes(
    entry: &Value,
    want_sha256: &str,
    blob_dir: Option<&Path>,
) -> Result<Vec<u8>, String> {
    let b64 = entry
        .get("content_base64")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let href = entry
        .get("href")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty());
    match (b64, href) {
        (Some(raw), None) => decode_base64(raw),
        (None, Some(href)) => {
            let name = parse_blob_href(href)?;
            if name != want_sha256 {
                return Err("path_escape".into());
            }
            let dir = blob_dir.ok_or_else(|| "payload_incomplete".to_string())?;
            let path = dir.join(&name);
            if !path.is_file() {
                return Err("payload_incomplete".into());
            }
            fs::read(&path).map_err(|_| "payload_incomplete".to_string())
        }
        (Some(_), Some(_)) => Err("payload_incomplete".into()),
        (None, None) => Err("payload_incomplete".into()),
    }
}

/// Accept only `/empiieros-ota/v1/blobs/<64 lowercase hex>` (no `..`, query, fragment).
pub fn parse_blob_href(href: &str) -> Result<String, String> {
    let t = href.trim();
    if !t.starts_with(BLOB_HREF_PREFIX) {
        return Err("path_escape".into());
    }
    let rest = &t[BLOB_HREF_PREFIX.len()..];
    if rest.is_empty()
        || rest.contains('/')
        || rest.contains("..")
        || rest.contains('\0')
        || rest.contains('?')
        || rest.contains('#')
        || rest.contains(':')
    {
        return Err("path_escape".into());
    }
    let name = rest.to_ascii_lowercase();
    if !is_sha256_hex(&name) {
        return Err("path_escape".into());
    }
    Ok(name)
}

fn is_sha256_hex(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

fn promote_staging(state: &Path) -> Result<(), String> {
    let staging = staging_dir(state);
    let current = current_dir(state);
    let previous = root_dir(state).join(PREVIOUS);
    if !staging.join(MANIFEST_NAME).is_file() {
        return Err("ota_staging_incomplete".into());
    }
    if current.is_dir() {
        let _ = fs::remove_dir_all(&previous);
        fs::rename(&current, &previous).map_err(|error| format!("ota_previous:{error}"))?;
    }
    fs::rename(&staging, &current).map_err(|error| format!("ota_promote:{error}"))?;
    set_dir_mode_0700(&current)?;
    Ok(())
}

fn verify_signed_payload(doc: &Value, pubkey: &[u8; 32]) -> Result<Value, String> {
    if !looks_signed(doc) {
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
        VerifyingKey::from_bytes(pubkey).map_err(|_| "artifact_role_mismatch".to_string())?;
    let msg = canonical_bytes(&payload)?;
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

fn looks_signed(doc: &Value) -> bool {
    doc.get("v").and_then(Value::as_u64) == Some(1)
        && doc.get("payload").map(Value::is_object).unwrap_or(false)
        && doc
            .get("sig")
            .and_then(Value::as_str)
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false)
}

fn canonical_bytes(payload: &Value) -> Result<Vec<u8>, String> {
    let sorted = sort_value(payload.clone());
    serde_json::to_vec(&sorted).map_err(|e| format!("canonical_json:{e}"))
}

fn sort_value(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<String> = map.keys().cloned().collect();
            keys.sort();
            let mut out = serde_json::Map::new();
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

pub fn is_path_safe(path: &str) -> bool {
    let t = path.trim();
    if t.is_empty() || t.starts_with('/') || t.contains("..") || t.contains('\0') || t.contains(':')
    {
        return false;
    }
    t.starts_with("runtime/") || t.starts_with("shell/") || t.starts_with("helpers/")
}

fn parse_pk_hex(raw: &str) -> Result<[u8; 32], String> {
    let bytes = hex::decode(raw.trim()).map_err(|_| "pubkey_invalid".to_string())?;
    if bytes.len() != 32 {
        return Err("pubkey_invalid".into());
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

fn decode_base64(raw: &str) -> Result<Vec<u8>, String> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(raw.trim())
        .map_err(|_| "payload_incomplete".to_string())
}

fn write_relative(root: &Path, rel: &str, bytes: &[u8]) -> Result<(), String> {
    let dest = root.join(rel);
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("ota_file:{error}"))?;
        set_dir_mode_0700(parent)?;
    }
    fs::write(&dest, bytes).map_err(|error| format!("ota_file:{error}"))?;
    let mut perms = fs::metadata(&dest)
        .map_err(|error| format!("ota_file:{error}"))?
        .permissions();
    perms.set_mode(0o600);
    fs::set_permissions(&dest, perms).map_err(|error| format!("ota_file:{error}"))?;
    Ok(())
}

fn set_dir_mode_0700(path: &Path) -> Result<(), String> {
    let mut perms = fs::metadata(path)
        .map_err(|error| format!("ota_dir:{error}"))?
        .permissions();
    perms.set_mode(0o700);
    fs::set_permissions(path, perms).map_err(|error| format!("ota_dir:{error}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("empiieros-ota-store-{nanos}"));
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn plant_pk(root: &Path, pk: &[u8; 32]) {
        let dir = root.join(ARTIFACT_PK_DIR);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(ARTIFACT_PK_FILE), hex::encode(pk)).unwrap();
    }

    fn sign(payload: Value, sk: [u8; 32]) -> Value {
        let msg = canonical_bytes(&payload).unwrap();
        let sig = SigningKey::from_bytes(&sk).sign(&msg);
        json!({ "v": 1, "payload": payload, "sig": hex::encode(sig.to_bytes()) })
    }

    #[test]
    fn path_escape_rejected() {
        assert!(!is_path_safe("../etc/passwd"));
        assert!(!is_path_safe("/abs"));
        assert!(!is_path_safe("evil/bin"));
        assert!(is_path_safe("runtime/empiieros-runtime"));
        assert!(is_path_safe("helpers/empiieros-ota-fetch"));
    }

    #[test]
    fn missing_pk_fail_closed() {
        let root = temp_root();
        let doc = json!({"v": 1, "payload": {"kind": KIND}, "sig": "aa"});
        assert_eq!(
            verify_and_stage(&root, &doc).unwrap_err(),
            "artifact_pk_missing"
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn plant_artifact_pk_roundtrip() {
        let root = temp_root();
        assert!(!artifact_pk_present(&root));
        let sk = [0x76u8; 32];
        let pk = SigningKey::from_bytes(&sk).verifying_key().to_bytes();
        plant_artifact_pk(&root, &hex::encode(pk)).unwrap();
        assert!(artifact_pk_present(&root));
        assert_eq!(resolve_artifact_pk(&root).unwrap(), pk);
        let mode = fs::metadata(root.join(ARTIFACT_PK_DIR).join(ARTIFACT_PK_FILE))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn plant_from_license_sibling_and_entitlement() {
        let root = temp_root();
        let sk = [0x77u8; 32];
        let pk = SigningKey::from_bytes(&sk).verifying_key().to_bytes();
        let hex = hex::encode(pk);
        let media = root.join("media");
        fs::create_dir_all(&media).unwrap();
        assert_eq!(plant_from_license(&root, None, None).unwrap(), "missing");
        assert!(!artifact_pk_present(&root));
        fs::write(media.join(ARTIFACT_PK_FILE), format!("{hex}\n")).unwrap();
        assert_eq!(
            plant_from_license(&root, None, Some(&media)).unwrap(),
            "planted"
        );
        assert_eq!(resolve_artifact_pk(&root).unwrap(), pk);
        let ent = json!({ "license_id": "lic-1", "ota_artifact_pk": hex });
        assert_eq!(
            plant_from_license(&root, Some(&ent), Some(&media)).unwrap(),
            "planted"
        );
        let other = json!({ "ota_artifact_pk": "aa".repeat(32) });
        assert_eq!(
            plant_from_license(&root, Some(&other), Some(&media)).unwrap_err(),
            "artifact_role_mismatch"
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn plant_from_persisted_inner_reads_payload_field() {
        let root = temp_root();
        let hex = "ab".repeat(32);
        let inner = root.join("capsule_inner");
        fs::create_dir_all(&inner).unwrap();
        assert_eq!(plant_from_persisted_inner(&root).unwrap(), "missing");
        fs::write(
            inner.join("entitlement.json"),
            json!({ "v": 1, "payload": { "ota_artifact_pk": hex }, "sig": "00" }).to_string(),
        )
        .unwrap();
        assert_eq!(plant_from_persisted_inner(&root).unwrap(), "planted");
        assert!(artifact_pk_present(&root));
        assert_eq!(plant_from_persisted_inner(&root).unwrap(), "present");
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn fixture_verifies_and_stages() {
        let root = temp_root();
        let sk = [0x71u8; 32];
        let pk = SigningKey::from_bytes(&sk).verifying_key().to_bytes();
        plant_pk(&root, &pk);
        let body = b"ota-fixture-v1";
        let payload = json!({
            "kind": KIND,
            "channel": "os_component",
            "version": "0.0.1-test",
            "app_build_id": "ota-fixture-1",
            "files": [{
                "path": "runtime/fixture.txt",
                "sha256": hex::encode(Sha256::digest(body)),
                "content_base64": base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    body
                )
            }]
        });
        let doc = sign(payload, sk);
        let out = verify_and_stage(&root, &doc).unwrap();
        assert_eq!(out["ota_staged"], true);
        assert_eq!(out["ota_app_build_id"], "ota-fixture-1");
        assert_eq!(out["ota_apply"], "ready");
        assert!(is_current_ready(&root));
        let stored = fs::read(current_dir(&root).join("runtime/fixture.txt")).unwrap();
        assert_eq!(stored, body);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn wrong_key_role_rejected() {
        let root = temp_root();
        let sk = [0x71u8; 32];
        let other = [0x72u8; 32];
        let pk = SigningKey::from_bytes(&sk).verifying_key().to_bytes();
        plant_pk(&root, &pk);
        let body = b"x";
        let payload = json!({
            "kind": KIND,
            "channel": "os_component",
            "version": "0.0.1",
            "app_build_id": "x",
            "files": [{
                "path": "helpers/empiieros-ota-fetch",
                "sha256": hex::encode(Sha256::digest(body)),
                "content_base64": base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    body
                )
            }]
        });
        let doc = sign(payload, other);
        assert_eq!(
            verify_and_stage(&root, &doc).unwrap_err(),
            "capsule_signature_required"
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn blob_href_escape_rejected() {
        assert!(parse_blob_href(&format!(
            "{BLOB_HREF_PREFIX}{}",
            "ab".repeat(32)
        ))
        .is_ok());
        assert_eq!(
            parse_blob_href("../etc/passwd").unwrap_err(),
            "path_escape"
        );
        assert_eq!(
            parse_blob_href("/fofline-updates/v1/blobs/aa").unwrap_err(),
            "path_escape"
        );
        assert_eq!(
            parse_blob_href(&format!("{BLOB_HREF_PREFIX}../x")).unwrap_err(),
            "path_escape"
        );
        assert_eq!(
            parse_blob_href(&format!("{BLOB_HREF_PREFIX}not-hex")).unwrap_err(),
            "path_escape"
        );
        assert_eq!(
            parse_blob_href(&format!("{BLOB_HREF_PREFIX}{}/evil", "ab".repeat(32))).unwrap_err(),
            "path_escape"
        );
    }

    #[test]
    fn blob_ref_stages_from_blob_dir() {
        let root = temp_root();
        let sk = [0x73u8; 32];
        let pk = SigningKey::from_bytes(&sk).verifying_key().to_bytes();
        plant_pk(&root, &pk);
        let body = b"empiieros-e1a-blob-ref\n";
        let digest = hex::encode(Sha256::digest(body));
        let blobs = root.join("blobs");
        fs::create_dir_all(&blobs).unwrap();
        fs::write(blobs.join(&digest), body).unwrap();
        let payload = json!({
            "kind": KIND,
            "channel": "os_component",
            "version": "0.0.1-blob",
            "app_build_id": "e1a-blob",
            "files": [{
                "path": "helpers/empiieros-ota-fetch",
                "sha256": digest,
                "href": format!("{BLOB_HREF_PREFIX}{digest}")
            }]
        });
        let doc = sign(payload, sk);
        let out = verify_and_stage_with_blobs(&root, &doc, Some(&blobs)).unwrap();
        assert_eq!(out["ota_staged"], true);
        assert_eq!(out["ota_app_build_id"], "e1a-blob");
        assert_eq!(
            fs::read(current_dir(&root).join("helpers/empiieros-ota-fetch")).unwrap(),
            body
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn blob_href_digest_mismatch_rejected() {
        let root = temp_root();
        let sk = [0x74u8; 32];
        let pk = SigningKey::from_bytes(&sk).verifying_key().to_bytes();
        plant_pk(&root, &pk);
        let body = b"body";
        let digest = hex::encode(Sha256::digest(body));
        let other = hex::encode(Sha256::digest(b"other"));
        let blobs = root.join("blobs");
        fs::create_dir_all(&blobs).unwrap();
        fs::write(blobs.join(&digest), body).unwrap();
        let payload = json!({
            "kind": KIND,
            "channel": "os_component",
            "files": [{
                "path": "helpers/empiieros-ota-fetch",
                "sha256": digest,
                "href": format!("{BLOB_HREF_PREFIX}{other}")
            }]
        });
        let doc = sign(payload, sk);
        assert_eq!(
            verify_and_stage_with_blobs(&root, &doc, Some(&blobs)).unwrap_err(),
            "path_escape"
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn blob_href_without_blob_dir_incomplete() {
        let root = temp_root();
        let sk = [0x75u8; 32];
        let pk = SigningKey::from_bytes(&sk).verifying_key().to_bytes();
        plant_pk(&root, &pk);
        let body = b"need-dir";
        let digest = hex::encode(Sha256::digest(body));
        let payload = json!({
            "kind": KIND,
            "channel": "os_component",
            "files": [{
                "path": "helpers/empiieros-ota-fetch",
                "sha256": digest,
                "href": format!("{BLOB_HREF_PREFIX}{digest}")
            }]
        });
        let doc = sign(payload, sk);
        assert_eq!(
            verify_and_stage(&root, &doc).unwrap_err(),
            "payload_incomplete"
        );
        fs::remove_dir_all(root).ok();
    }
}
