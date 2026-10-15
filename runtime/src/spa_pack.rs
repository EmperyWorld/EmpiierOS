//! FoFline SPA pack verify/store (harvested from living `spa_pack`).
//!
//! Minisign + sha256, atomic staging → current. Not `product.open`.
//! Helper fetches bytes; this module owns verify/extract.

use base64::Engine;
use minisign_verify::{PublicKey, Signature};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{copy, Read};
use std::os::unix::fs::PermissionsExt;
use std::path::{Component, Path, PathBuf};
use tar::{Archive, EntryType};

pub const KIND_FOFLINE_SPA: &str = "fofline-spa";
pub const STATE_PACK_MISSING: &str = "pack_missing";
pub const STATE_READY: &str = "ready";
pub const STATE_ERROR: &str = "error";
pub const STATE_DENIED: &str = "denied";

pub const MANIFEST_PATH: &str = "/fofline-updates/spa-latest.json";
pub const PUBLIC_BASE: &str = "https://app.eliteew.ru/fofline-updates";

/// Same FoFline/Tauri updater minisign pubkey as living `tauri.conf.fofline.overlay.json`.
pub const FOFLINE_UPDATER_MINISIGN_PK: &str =
    "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDkzQjYwNUI3MzZDNUNEODMKUldTRHpjVTJ0d1cya3h5TmNmV29NYjhTMTJEZVEwTmJaZVh4eWRvb3BtaTUwVko2MnpibGxwOTkK";

pub const CORE_PACK_ARCHIVE_MAX_BYTES: u64 = 512 * 1024 * 1024;
/// Optional group archives stay well under the 1.5 GiB audio tree.
pub const GROUP_PACK_ARCHIVE_MAX_BYTES: u64 = 128 * 1024 * 1024;
pub const ARCHIVE_MAX_ENTRIES: u64 = 50_000;
pub const ARCHIVE_MAX_FILE_BYTES: u64 = 256 * 1024 * 1024;
pub const ARCHIVE_MAX_TOTAL_BYTES: u64 = 1024 * 1024 * 1024;
pub const ARCHIVE_MAX_COMPRESSION_RATIO: u64 = 100;
pub const MANIFEST_MAX_BYTES: u64 = 64 * 1024;

const PACK_DIR: &str = "fofline-spa";
const PACK_PARENT_MODE: u32 = 0o751;
const PACK_CURRENT_MODE: u32 = 0o755;
const SECRET_DIR_MODE: u32 = 0o700;

#[derive(Debug, Clone, Deserialize)]
pub struct SpaLatestManifest {
    pub schema_version: u32,
    pub kind: String,
    pub version: String,
    pub app_build_id: String,
    pub url: String,
    #[serde(default)]
    pub signature: String,
    pub sha256: String,
    #[serde(default)]
    pub include_audio: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalSpaPackState {
    pub schema_version: u32,
    pub state: String,
    pub app_build_id: Option<String>,
    #[serde(default)]
    pub last_error_code: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SpaView {
    pub state: String,
    pub code: String,
    pub app_build_id: Option<String>,
}

pub fn pack_root(state_root: &Path) -> PathBuf {
    state_root.join(PACK_DIR)
}

pub fn current_dir(state_root: &Path) -> PathBuf {
    pack_root(state_root).join("current")
}

pub fn staging_dir(state_root: &Path) -> PathBuf {
    pack_root(state_root).join("staging")
}

pub fn spool_dir(state_root: &Path) -> PathBuf {
    pack_root(state_root).join("download-spool")
}

pub fn observe(state_root: &Path) -> SpaView {
    let local = load_state(state_root);
    if local.state == STATE_READY && is_ready(state_root, local.app_build_id.as_deref()) {
        return SpaView {
            state: STATE_READY.into(),
            code: "ready".into(),
            app_build_id: local.app_build_id,
        };
    }
    if local.state == STATE_ERROR {
        return SpaView {
            state: STATE_ERROR.into(),
            code: local.last_error_code.unwrap_or_else(|| "error".into()),
            app_build_id: local.app_build_id,
        };
    }
    SpaView {
        state: STATE_PACK_MISSING.into(),
        code: "pack_missing".into(),
        app_build_id: None,
    }
}

pub fn parse_spa_latest(raw: &str) -> Result<SpaLatestManifest, String> {
    let manifest: SpaLatestManifest =
        serde_json::from_str(raw).map_err(|error| format!("spa_latest_parse:{error}"))?;
    if manifest.kind != KIND_FOFLINE_SPA {
        return Err("spa_kind".into());
    }
    if manifest.schema_version < 1 {
        return Err("spa_schema".into());
    }
    if !valid_app_build_id(&manifest.app_build_id) || manifest.version.trim().is_empty() {
        return Err("spa_app_build_id".into());
    }
    if manifest.sha256.trim().len() != 64
        || !manifest.sha256.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err("spa_sha256".into());
    }
    if manifest.signature.trim().is_empty() {
        return Err("signature_required".into());
    }
    if manifest.include_audio {
        return Err("audio_pack_forbidden".into());
    }
    allowlisted_artifact_url(&manifest)?;
    Ok(manifest)
}

pub fn allowlisted_artifact_url(manifest: &SpaLatestManifest) -> Result<String, String> {
    let expected = format!(
        "{PUBLIC_BASE}/artifacts/EliteEW_FoFline_SPA-{}.tar.zst",
        manifest.app_build_id
    );
    if manifest.url.trim() != expected {
        return Err("spa_url_forbidden".into());
    }
    if manifest.url.contains("desktop-client-updates") {
        return Err("spa_url_forbidden".into());
    }
    Ok(format!(
        "/fofline-updates/artifacts/EliteEW_FoFline_SPA-{}.tar.zst",
        manifest.app_build_id
    ))
}

pub fn artifact_path(manifest: &SpaLatestManifest) -> Result<String, String> {
    allowlisted_artifact_url(manifest)
}

pub fn verify_tauri_minisign(data: &[u8], signature_b64: &str) -> Result<(), String> {
    let pub_key_decoded = base64_to_utf8(FOFLINE_UPDATER_MINISIGN_PK)?;
    let public_key =
        PublicKey::decode(&pub_key_decoded).map_err(|error| format!("spa_pubkey:{error}"))?;
    let sig_decoded = base64_to_utf8(signature_b64)?;
    let signature =
        Signature::decode(&sig_decoded).map_err(|error| format!("spa_signature:{error}"))?;
    public_key
        .verify(data, &signature, true)
        .map_err(|_| "signature_invalid".to_string())?;
    Ok(())
}

#[cfg(test)]
fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

pub fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|error| format!("spa_sha_read:{error}"))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 32 * 1024];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|error| format!("spa_sha_read:{error}"))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

pub fn extract_tar_zst_file(archive: &Path, dest: &Path) -> Result<(), String> {
    let meta = fs::metadata(archive).map_err(|error| format!("spa_archive:{error}"))?;
    let compressed_len = meta.len();
    if compressed_len > CORE_PACK_ARCHIVE_MAX_BYTES {
        return Err("pack_too_large".into());
    }
    let file = File::open(archive).map_err(|error| format!("spa_archive:{error}"))?;
    let decoder = zstd::stream::read::Decoder::new(file)
        .map_err(|error| format!("spa_zstd:{error}"))?;
    extract_tar_reader(decoder, compressed_len, dest)?;
    if !dest.join("index.html").is_file() {
        return Err("spa_missing_index".into());
    }
    if !dest.join("pack-manifest.json").is_file() {
        return Err("spa_missing_manifest".into());
    }
    Ok(())
}

pub fn extract_group_tar_zst_file(archive: &Path, dest: &Path) -> Result<(), String> {
    let meta = fs::metadata(archive).map_err(|error| format!("spa_archive:{error}"))?;
    let compressed_len = meta.len();
    if compressed_len > GROUP_PACK_ARCHIVE_MAX_BYTES {
        return Err("pack_too_large".into());
    }
    let file = File::open(archive).map_err(|error| format!("spa_archive:{error}"))?;
    let decoder = zstd::stream::read::Decoder::new(file)
        .map_err(|error| format!("spa_zstd:{error}"))?;
    extract_tar_reader(decoder, compressed_len, dest)?;
    if !dest.join("files.json").is_file() && !dest.join("group-manifest.json").is_file() {
        return Err("group_pack_manifest_missing".into());
    }
    Ok(())
}

pub fn promote_staging(state_root: &Path) -> Result<PathBuf, String> {
    let staging = staging_dir(state_root);
    let current = current_dir(state_root);
    let root = pack_root(state_root);
    fs::create_dir_all(&root).map_err(|error| format!("spa_store:{error}"))?;
    set_dir_mode(&root, PACK_PARENT_MODE)?;
    let backup = root.join("previous");
    if backup.exists() {
        let _ = fs::remove_dir_all(&backup);
    }
    if current.exists() {
        fs::rename(&current, &backup).map_err(|error| format!("spa_backup:{error}"))?;
    }
    if let Err(error) = fs::rename(&staging, &current) {
        if backup.exists() && !current.exists() {
            let _ = fs::rename(&backup, &current);
        }
        return Err(format!("spa_promote:{error}"));
    }
    if backup.exists() {
        let _ = fs::remove_dir_all(&backup);
    }
    set_dir_mode(&current, PACK_CURRENT_MODE)?;
    Ok(current)
}

pub fn write_ready_state(state_root: &Path, app_build_id: &str) -> Result<(), String> {
    save_state(
        state_root,
        &LocalSpaPackState {
            schema_version: 1,
            state: STATE_READY.into(),
            app_build_id: Some(app_build_id.to_string()),
            last_error_code: None,
        },
    )
}

pub fn write_error_state(state_root: &Path, code: &str) -> Result<(), String> {
    let keep = is_ready(state_root, None);
    let prior = load_state(state_root);
    save_state(
        state_root,
        &LocalSpaPackState {
            schema_version: 1,
            state: if keep {
                STATE_READY.into()
            } else {
                STATE_ERROR.into()
            },
            app_build_id: if keep { prior.app_build_id } else { None },
            last_error_code: Some(code.into()),
        },
    )
}

pub fn is_ready(state_root: &Path, expected_id: Option<&str>) -> bool {
    let cur = current_dir(state_root);
    if !(cur.join("index.html").is_file() && cur.join("pack-manifest.json").is_file()) {
        return false;
    }
    if let Some(want) = expected_id {
        let Ok(raw) = fs::read_to_string(cur.join("pack-manifest.json")) else {
            return false;
        };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) else {
            return false;
        };
        return value.get("app_build_id").and_then(|item| item.as_str()) == Some(want);
    }
    true
}

fn extract_tar_reader<R: Read>(
    reader: R,
    compressed_len: u64,
    dest: &Path,
) -> Result<(), String> {
    if dest.exists() {
        fs::remove_dir_all(dest).map_err(|error| format!("spa_staging:{error}"))?;
    }
    fs::create_dir_all(dest).map_err(|error| format!("spa_staging:{error}"))?;
    set_dir_mode(dest, SECRET_DIR_MODE)?;
    let mut archive = Archive::new(reader);
    archive.set_overwrite(true);
    let mut entries = 0u64;
    let mut total_written = 0u64;
    for entry in archive
        .entries()
        .map_err(|error| format!("spa_tar:{error}"))?
    {
        let mut entry = entry.map_err(|error| format!("spa_tar:{error}"))?;
        entries += 1;
        if entries > ARCHIVE_MAX_ENTRIES {
            return Err("archive_too_many_entries".into());
        }
        let kind = entry.header().entry_type();
        match kind {
            EntryType::Regular | EntryType::Directory | EntryType::GNUSparse => {}
            _ => return Err("archive_forbidden_entry".into()),
        }
        let rel = entry
            .path()
            .map_err(|error| format!("spa_tar:{error}"))?
            .into_owned();
        if path_is_unsafe(&rel) {
            return Err("archive_unsafe_path".into());
        }
        if kind == EntryType::Directory {
            let dir = dest.join(&rel);
            fs::create_dir_all(&dir).map_err(|error| format!("spa_staging:{error}"))?;
            continue;
        }
        let size = entry.size();
        if size > ARCHIVE_MAX_FILE_BYTES {
            return Err("archive_file_too_large".into());
        }
        total_written = total_written.saturating_add(size);
        if total_written > ARCHIVE_MAX_TOTAL_BYTES {
            return Err("archive_total_too_large".into());
        }
        if compressed_len > 0 && total_written / compressed_len.max(1) > ARCHIVE_MAX_COMPRESSION_RATIO
        {
            return Err("archive_compression_bomb".into());
        }
        let out_path = dest.join(&rel);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).map_err(|error| format!("spa_staging:{error}"))?;
        }
        let mut out = File::create(&out_path).map_err(|error| format!("spa_staging:{error}"))?;
        copy(&mut entry, &mut out).map_err(|error| format!("spa_staging:{error}"))?;
    }
    Ok(())
}

fn path_is_unsafe(path: &Path) -> bool {
    path.is_absolute()
        || path
            .components()
            .any(|item| matches!(item, Component::ParentDir | Component::RootDir))
}

fn valid_app_build_id(raw: &str) -> bool {
    let Some((digits, rest)) = raw.split_once('-') else {
        return false;
    };
    !digits.is_empty()
        && digits.bytes().all(|b| b.is_ascii_digit())
        && !rest.is_empty()
        && rest.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
}

fn load_state(state_root: &Path) -> LocalSpaPackState {
    let path = pack_root(state_root).join("state.json");
    match fs::read_to_string(path) {
        Ok(raw) => serde_json::from_str(&raw).unwrap_or_else(|_| missing_state()),
        Err(_) => missing_state(),
    }
}

fn missing_state() -> LocalSpaPackState {
    LocalSpaPackState {
        schema_version: 1,
        state: STATE_PACK_MISSING.into(),
        app_build_id: None,
        last_error_code: None,
    }
}

fn save_state(state_root: &Path, state: &LocalSpaPackState) -> Result<(), String> {
    let root = pack_root(state_root);
    fs::create_dir_all(&root).map_err(|error| format!("spa_store:{error}"))?;
    set_dir_mode(&root, PACK_PARENT_MODE)?;
    crate::state_store::atomic_write_file(
        &root,
        "state.json",
        format!("{}\n", serde_json::to_string(state).map_err(|error| format!("spa_store:{error}"))?)
            .as_bytes(),
        0o600,
    )
    .map(|_| ())
    .map_err(|error| format!("spa_store:{error}"))
}

fn set_dir_mode(dir: &Path, mode: u32) -> Result<(), String> {
    let mut permissions = fs::metadata(dir)
        .map_err(|error| format!("spa_store:{error}"))?
        .permissions();
    permissions.set_mode(mode);
    fs::set_permissions(dir, permissions).map_err(|error| format!("spa_store:{error}"))
}

fn base64_to_utf8(b64: &str) -> Result<String, String> {
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(b64.trim())
        .map_err(|_| "spa_b64".to_string())?;
    String::from_utf8(decoded).map_err(|_| "spa_b64".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("empiieros-spa-{nanos}"));
        fs::create_dir_all(&root).unwrap();
        root
    }

    const GOOD: &str = r#"{
      "schema_version": 1,
      "kind": "fofline-spa",
      "version": "1786238708853-jn25ybw0v0",
      "app_build_id": "1786238708853-jn25ybw0v0",
      "url": "https://app.eliteew.ru/fofline-updates/artifacts/EliteEW_FoFline_SPA-1786238708853-jn25ybw0v0.tar.zst",
      "signature": "dW50cnVzdGVk",
      "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      "include_audio": false
    }"#;

    #[test]
    fn parse_accepts_living_shape() {
        let manifest = parse_spa_latest(GOOD).expect("good");
        assert_eq!(
            format!("{PUBLIC_BASE}/spa-latest.json"),
            format!("https://app.eliteew.ru{MANIFEST_PATH}")
        );
        assert_eq!(manifest.kind, KIND_FOFLINE_SPA);
        assert_eq!(
            artifact_path(&manifest).unwrap(),
            "/fofline-updates/artifacts/EliteEW_FoFline_SPA-1786238708853-jn25ybw0v0.tar.zst"
        );
    }

    #[test]
    fn parse_rejects_empty_signature() {
        let bad = GOOD.replace("dW50cnVzdGVk", "");
        assert_eq!(parse_spa_latest(&bad).unwrap_err(), "signature_required");
    }

    #[test]
    fn parse_rejects_desktop_channel() {
        let bad = GOOD.replace(
            "https://app.eliteew.ru/fofline-updates/artifacts/EliteEW_FoFline_SPA-1786238708853-jn25ybw0v0.tar.zst",
            "https://app.eliteew.ru/desktop-client-updates/artifacts/EliteEW_FoFline_SPA-1786238708853-jn25ybw0v0.tar.zst",
        );
        assert_eq!(parse_spa_latest(&bad).unwrap_err(), "spa_url_forbidden");
    }

    #[test]
    fn parse_rejects_audio() {
        let bad = GOOD.replace("\"include_audio\": false", "\"include_audio\": true");
        assert_eq!(parse_spa_latest(&bad).unwrap_err(), "audio_pack_forbidden");
    }

    #[test]
    fn sha256_known() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn extract_rejects_parent_dir() {
        let root = temp_root();
        let src = root.join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("index.html"), b"<html></html>").unwrap();
        fs::write(src.join("pack-manifest.json"), b"{\"app_build_id\":\"t\"}").unwrap();
        let archive = root.join("bad.tar.zst");
        let status = std::process::Command::new("bash")
            .arg("-lc")
            .arg(format!(
                "cd '{}' && tar --transform='s,index.html,../escape.html,' -cf - . | zstd -q -o '{}'",
                src.display(),
                archive.display()
            ))
            .status();
        if status.map(|s| s.success()).unwrap_or(false) {
            let dest = root.join("out");
            assert_eq!(
                extract_tar_zst_file(&archive, &dest).unwrap_err(),
                "archive_unsafe_path"
            );
        }
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn observe_missing_by_default() {
        let root = temp_root();
        let view = observe(&root);
        assert_eq!(view.state, STATE_PACK_MISSING);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn ready_store_is_traversable_and_state_stays_closed() {
        let root = temp_root();
        let staging = staging_dir(&root);
        fs::create_dir_all(&staging).unwrap();
        fs::write(staging.join("index.html"), "<!doctype html><title>pack</title>").unwrap();
        fs::write(
            staging.join("pack-manifest.json"),
            r#"{"schema_version":1,"kind":"fofline-spa","app_build_id":"1786238708853-jn25ybw0v0"}"#,
        )
        .unwrap();
        promote_staging(&root).unwrap();
        write_ready_state(&root, "1786238708853-jn25ybw0v0").unwrap();
        assert_eq!(
            fs::metadata(pack_root(&root)).unwrap().permissions().mode() & 0o777,
            PACK_PARENT_MODE
        );
        assert_eq!(
            fs::metadata(current_dir(&root)).unwrap().permissions().mode() & 0o777,
            PACK_CURRENT_MODE
        );
        assert_eq!(
            fs::metadata(pack_root(&root).join("state.json"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        let _ = fs::remove_dir_all(root);
    }
}
