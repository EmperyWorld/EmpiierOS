//! FoFline optional SPA groups. Harvested from living Tauri `spa_assets.rs`.
//!
//! Core Start never waits. Groups merge into `current/` and do not wipe it.
//! `audio-guides` stays deferred (1.5 GiB); user media stays `/api/v1/content`.

use crate::spa_pack::{self, current_dir, pack_root, sha256_file, GROUP_PACK_ARCHIVE_MAX_BYTES};
use crate::state_store::atomic_write_file;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

pub const ASSETS_MANIFEST_PATH: &str = "/fofline-updates/spa-assets-latest.json";
pub const KIND_FOFLINE_SPA_ASSETS: &str = "fofline-spa-assets";
pub const KIND_FOFLINE_SPA_FILES: &str = "fofline-spa-files";
pub const ASSETS_MANIFEST_MAX_BYTES: u64 = 128 * 1024;
const DEFERRED_GROUP: &str = "audio-guides";

#[derive(Debug, Clone, Deserialize)]
pub struct SpaAssetsGroupEntry {
    pub id: String,
    pub url: String,
    pub sha256: String,
    pub signature: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpaAssetsLatestManifest {
    pub schema_version: u32,
    pub kind: String,
    pub app_build_id: String,
    #[serde(default)]
    pub groups: Vec<SpaAssetsGroupEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FilesEntry {
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FilesManifest {
    pub kind: String,
    #[serde(default)]
    pub files: Vec<FilesEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupLocalRecord {
    pub archive_sha256: String,
    pub ready: bool,
    #[serde(default)]
    pub deferred: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AssetsLocalState {
    pub schema_version: u32,
    #[serde(default)]
    pub app_build_id: Option<String>,
    #[serde(default)]
    pub groups: HashMap<String, GroupLocalRecord>,
    #[serde(default)]
    pub catalog_ids: Vec<String>,
    #[serde(default)]
    pub last_error_code: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AssetsView {
    pub ready: u32,
    pub total: u32,
    pub code: String,
}

pub fn observe(state_root: &Path) -> AssetsView {
    let state = load_state(state_root);
    let catalog = if state.catalog_ids.is_empty() {
        state.groups.keys().cloned().collect::<Vec<_>>()
    } else {
        state.catalog_ids.clone()
    };
    let total = catalog.len() as u32;
    let ready = catalog
        .iter()
        .filter(|id| state.groups.get(*id).map(|row| row.ready).unwrap_or(false))
        .count() as u32;
    let code = state
        .last_error_code
        .unwrap_or_else(|| {
            if total == 0 {
                "assets_idle".into()
            } else if ready == total {
                "assets_ready".into()
            } else {
                "assets_partial".into()
            }
        });
    AssetsView { ready, total, code }
}

pub fn parse_spa_assets_latest(raw: &str) -> Result<SpaAssetsLatestManifest, String> {
    let manifest: SpaAssetsLatestManifest =
        serde_json::from_str(raw).map_err(|error| format!("spa_assets_parse:{error}"))?;
    if manifest.kind != KIND_FOFLINE_SPA_ASSETS {
        return Err("spa_assets_kind".into());
    }
    if manifest.schema_version < 1 {
        return Err("spa_assets_schema".into());
    }
    if !valid_app_build_id(&manifest.app_build_id) {
        return Err("spa_assets_app_build_id".into());
    }
    for group in &manifest.groups {
        if !valid_group_id(&group.id) {
            return Err("spa_assets_group_id".into());
        }
        if group.sha256.trim().len() != 64
            || !group.sha256.bytes().all(|item| item.is_ascii_hexdigit())
        {
            return Err("spa_assets_sha256".into());
        }
        if group.signature.trim().is_empty() {
            return Err("signature_required".into());
        }
        group_artifact_path(&manifest.app_build_id, &group.id, &group.url)?;
    }
    Ok(manifest)
}

pub fn parse_files_manifest(raw: &str) -> Result<FilesManifest, String> {
    let manifest: FilesManifest =
        serde_json::from_str(raw).map_err(|error| format!("spa_files_parse:{error}"))?;
    if manifest.kind != KIND_FOFLINE_SPA_FILES {
        return Err("spa_files_kind".into());
    }
    Ok(manifest)
}

pub fn group_artifact_path(
    app_build_id: &str,
    group_id: &str,
    url: &str,
) -> Result<String, String> {
    if !valid_group_id(group_id) || !valid_app_build_id(app_build_id) {
        return Err("spa_url_forbidden".into());
    }
    let expected = format!(
        "{}/artifacts/EliteEW_FoFline_SPA_GROUP-{}-{}.tar.zst",
        spa_pack::PUBLIC_BASE,
        app_build_id,
        group_id
    );
    if url.trim() != expected || url.contains("desktop-client-updates") {
        return Err("spa_url_forbidden".into());
    }
    Ok(format!(
        "/fofline-updates/artifacts/EliteEW_FoFline_SPA_GROUP-{app_build_id}-{group_id}.tar.zst"
    ))
}

pub fn local_files_complete(current: &Path, files: &FilesManifest) -> bool {
    if files.files.is_empty() {
        return false;
    }
    for entry in &files.files {
        let rel = entry.path.trim().trim_start_matches('/');
        if rel.is_empty() || rel.contains("..") {
            return false;
        }
        if rel == "files.json" || rel == "group-manifest.json" {
            continue;
        }
        let path = current.join(rel);
        match sha256_file(&path) {
            Ok(got) if got.eq_ignore_ascii_case(entry.sha256.trim()) => {}
            _ => return false,
        }
    }
    true
}

pub fn group_skip_download(
    state_root: &Path,
    group_id: &str,
    remote_archive_sha: &str,
    state: &AssetsLocalState,
) -> bool {
    let Some(record) = state.groups.get(group_id) else {
        return false;
    };
    if record.deferred
        || !record.ready
        || !record
            .archive_sha256
            .eq_ignore_ascii_case(remote_archive_sha.trim())
    {
        return false;
    }
    let meta = group_meta_dir(state_root, group_id).join("files.json");
    let Ok(raw) = fs::read_to_string(meta) else {
        return false;
    };
    let Ok(files) = parse_files_manifest(&raw) else {
        return false;
    };
    local_files_complete(&current_dir(state_root), &files)
}

pub fn merge_group_into_current(staging: &Path, current: &Path) -> Result<(), String> {
    fs::create_dir_all(current).map_err(|error| format!("spa_assets_merge:{error}"))?;
    for entry in walk_files(staging)? {
        let rel = entry
            .strip_prefix(staging)
            .map_err(|error| format!("spa_assets_merge:{error}"))?;
        let rel_s = rel.to_string_lossy().replace('\\', "/");
        if skip_merge_name(&rel_s) || rel_s.contains("..") {
            if rel_s.contains("..") {
                return Err("archive_unsafe_path".into());
            }
            continue;
        }
        let dest = current.join(rel);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|error| format!("spa_assets_merge:{error}"))?;
        }
        fs::copy(&entry, &dest).map_err(|error| format!("spa_assets_merge:{error}"))?;
    }
    Ok(())
}

pub fn sync_groups(state_root: &Path, core_app_build_id: &str) -> Result<AssetsView, String> {
    let raw = match crate::https_vip::get_text(ASSETS_MANIFEST_PATH, ASSETS_MANIFEST_MAX_BYTES) {
        Ok(text) => text,
        Err(error) => {
            let _ = save_error(state_root, "assets_channel_missing");
            return Ok(AssetsView {
                ready: 0,
                total: 0,
                code: format!("assets_channel_missing:{error}"),
            });
        }
    };
    let manifest = match parse_spa_assets_latest(&raw) {
        Ok(value) => value,
        Err(error) => {
            let _ = save_error(state_root, &error);
            return Ok(AssetsView {
                ready: 0,
                total: 0,
                code: error,
            });
        }
    };
    if manifest.app_build_id != core_app_build_id {
        let _ = save_error(state_root, "assets_core_mismatch");
        return Ok(AssetsView {
            ready: 0,
            total: manifest.groups.len() as u32,
            code: "assets_core_mismatch".into(),
        });
    }
    let mut state = load_state(state_root);
    state.schema_version = 1;
    state.app_build_id = Some(core_app_build_id.to_string());
    state.catalog_ids = manifest.groups.iter().map(|group| group.id.clone()).collect();
    state.last_error_code = None;
    for group in &manifest.groups {
        if group.id == DEFERRED_GROUP {
            state.groups.insert(
                group.id.clone(),
                GroupLocalRecord {
                    archive_sha256: group.sha256.clone(),
                    ready: false,
                    deferred: true,
                },
            );
            continue;
        }
        if group_skip_download(state_root, &group.id, &group.sha256, &state) {
            continue;
        }
        match install_group(state_root, &manifest.app_build_id, group) {
            Ok(()) => {
                state.groups.insert(
                    group.id.clone(),
                    GroupLocalRecord {
                        archive_sha256: group.sha256.clone(),
                        ready: true,
                        deferred: false,
                    },
                );
            }
            Err(error) => {
                state.last_error_code = Some(error.clone());
                state.groups.insert(
                    group.id.clone(),
                    GroupLocalRecord {
                        archive_sha256: String::new(),
                        ready: false,
                        deferred: false,
                    },
                );
            }
        }
    }
    save_state(state_root, &state)?;
    Ok(observe(state_root))
}

fn install_group(
    state_root: &Path,
    app_build_id: &str,
    group: &SpaAssetsGroupEntry,
) -> Result<(), String> {
    let path = group_artifact_path(app_build_id, &group.id, &group.url)?;
    let spool = pack_root(state_root).join("download-spool");
    fs::create_dir_all(&spool).map_err(|error| format!("spa_assets_spool:{error}"))?;
    set_dir_mode(&spool, 0o700)?;
    let archive = spool.join(format!("{}.tar.zst", sanitize_group_id(&group.id)));
    let downloaded = crate::https_vip::get_file_hashed(
        &path,
        &archive,
        GROUP_PACK_ARCHIVE_MAX_BYTES,
    )
    .map_err(|error| {
        let _ = fs::remove_file(&archive);
        error
    })?;
    if !downloaded
        .sha256_hex
        .eq_ignore_ascii_case(group.sha256.trim())
    {
        let _ = fs::remove_file(&archive);
        return Err("sha256_mismatch".into());
    }
    let bytes = fs::read(&archive).map_err(|error| format!("spa_assets_read:{error}"))?;
    if let Err(error) = spa_pack::verify_tauri_minisign(&bytes, &group.signature) {
        let _ = fs::remove_file(&archive);
        return Err(error);
    }
    drop(bytes);
    let staging = pack_root(state_root)
        .join("group-staging")
        .join(sanitize_group_id(&group.id));
    if let Err(error) = spa_pack::extract_group_tar_zst_file(&archive, &staging) {
        let _ = fs::remove_file(&archive);
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }
    let _ = fs::remove_file(&archive);
    merge_group_into_current(&staging, &current_dir(state_root))?;
    persist_group_meta(state_root, &group.id, &staging)?;
    let _ = fs::remove_dir_all(&staging);
    Ok(())
}

fn persist_group_meta(state_root: &Path, group_id: &str, staging: &Path) -> Result<(), String> {
    let meta = group_meta_dir(state_root, group_id);
    fs::create_dir_all(&meta).map_err(|error| format!("spa_assets_meta:{error}"))?;
    for name in ["files.json", "group-manifest.json"] {
        let src = staging.join(name);
        if src.is_file() {
            fs::copy(&src, meta.join(name)).map_err(|error| format!("spa_assets_meta:{error}"))?;
        }
    }
    Ok(())
}

fn skip_merge_name(rel: &str) -> bool {
    matches!(
        rel,
        "files.json"
            | "group-manifest.json"
            | "index.html"
            | "pack-manifest.json"
            | "state.json"
            | "assets-state.json"
    )
}

fn walk_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    walk_files_inner(root, &mut out)?;
    out.sort();
    Ok(out)
}

fn walk_files_inner(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let reader = fs::read_dir(dir).map_err(|error| format!("spa_assets_walk:{error}"))?;
    for entry in reader {
        let entry = entry.map_err(|error| format!("spa_assets_walk:{error}"))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|error| format!("spa_assets_walk:{error}"))?;
        if file_type.is_dir() {
            walk_files_inner(&path, out)?;
        } else if file_type.is_file() {
            out.push(path);
        }
    }
    Ok(())
}

fn group_meta_dir(state_root: &Path, group_id: &str) -> PathBuf {
    current_dir(state_root)
        .join(".fofline-group-meta")
        .join(sanitize_group_id(group_id))
}

fn sanitize_group_id(group_id: &str) -> String {
    group_id
        .chars()
        .map(|item| {
            if item.is_ascii_alphanumeric() || item == '-' || item == '_' {
                item
            } else {
                '_'
            }
        })
        .collect()
}

fn valid_group_id(raw: &str) -> bool {
    let trimmed = raw.trim();
    (1..=64).contains(&trimmed.len())
        && trimmed
            .bytes()
            .all(|item| item.is_ascii_alphanumeric() || item == b'-' || item == b'_')
}

fn valid_app_build_id(raw: &str) -> bool {
    let Some((digits, rest)) = raw.split_once('-') else {
        return false;
    };
    !digits.is_empty()
        && digits.bytes().all(|item| item.is_ascii_digit())
        && !rest.is_empty()
        && rest
            .bytes()
            .all(|item| item.is_ascii_lowercase() || item.is_ascii_digit())
}

fn assets_state_path(state_root: &Path) -> PathBuf {
    pack_root(state_root).join("assets-state.json")
}

fn load_state(state_root: &Path) -> AssetsLocalState {
    match fs::read_to_string(assets_state_path(state_root)) {
        Ok(raw) => serde_json::from_str(&raw).unwrap_or_default(),
        Err(_) => AssetsLocalState {
            schema_version: 1,
            ..Default::default()
        },
    }
}

fn save_state(state_root: &Path, state: &AssetsLocalState) -> Result<(), String> {
    let root = pack_root(state_root);
    fs::create_dir_all(&root).map_err(|error| format!("spa_assets_store:{error}"))?;
    let raw = format!(
        "{}\n",
        serde_json::to_string(state).map_err(|error| format!("spa_assets_store:{error}"))?
    );
    atomic_write_file(&root, "assets-state.json", raw.as_bytes(), 0o600)
        .map(|_| ())
        .map_err(|error| format!("spa_assets_store:{error}"))
}

fn save_error(state_root: &Path, code: &str) -> Result<(), String> {
    let mut state = load_state(state_root);
    state.schema_version = 1;
    state.last_error_code = Some(code.to_string());
    save_state(state_root, &state)
}

fn set_dir_mode(dir: &Path, mode: u32) -> Result<(), String> {
    let mut permissions = fs::metadata(dir)
        .map_err(|error| format!("spa_assets_store:{error}"))?
        .permissions();
    permissions.set_mode(mode);
    fs::set_permissions(dir, permissions).map_err(|error| format!("spa_assets_store:{error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};

    const BUILD: &str = "1786238708853-jn25ybw0v0";

    fn good_group(id: &str) -> String {
        format!(
            r#"{{
              "schema_version": 1,
              "kind": "fofline-spa-assets",
              "app_build_id": "{BUILD}",
              "groups": [{{
                "id": "{id}",
                "url": "https://app.eliteew.ru/fofline-updates/artifacts/EliteEW_FoFline_SPA_GROUP-{BUILD}-{id}.tar.zst",
                "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "signature": "dW50cnVzdGVk"
              }}]
            }}"#
        )
    }

    fn temp_root() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("empiieros-spa-assets-{nanos}"));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn parse_rejects_empty_group_signature() {
        let raw = good_group("desktop-wallpapers").replace("dW50cnVzdGVk", "");
        assert_eq!(parse_spa_assets_latest(&raw).unwrap_err(), "signature_required");
    }

    #[test]
    fn parse_rejects_desktop_channel() {
        let raw = good_group("desktop-wallpapers").replace(
            "fofline-updates/artifacts",
            "desktop-client-updates/artifacts",
        );
        assert_eq!(parse_spa_assets_latest(&raw).unwrap_err(), "spa_url_forbidden");
    }

    #[test]
    fn artifact_path_is_exact() {
        let url = format!(
            "https://app.eliteew.ru/fofline-updates/artifacts/EliteEW_FoFline_SPA_GROUP-{BUILD}-ewa-media.tar.zst"
        );
        assert_eq!(
            group_artifact_path(BUILD, "ewa-media", &url).unwrap(),
            format!("/fofline-updates/artifacts/EliteEW_FoFline_SPA_GROUP-{BUILD}-ewa-media.tar.zst")
        );
    }

    #[test]
    fn merge_keeps_core_index() {
        let root = temp_root();
        let current = root.join("current");
        let staging = root.join("staging");
        fs::create_dir_all(current.join("assets")).unwrap();
        fs::write(current.join("index.html"), b"core").unwrap();
        fs::write(current.join("pack-manifest.json"), b"{}").unwrap();
        fs::create_dir_all(staging.join("desktop-wallpapers")).unwrap();
        fs::write(staging.join("desktop-wallpapers/15.webp"), b"wp").unwrap();
        fs::write(staging.join("index.html"), b"evil").unwrap();
        fs::write(staging.join("files.json"), b"{}").unwrap();
        merge_group_into_current(&staging, &current).unwrap();
        assert_eq!(fs::read(current.join("index.html")).unwrap(), b"core");
        assert_eq!(
            fs::read(current.join("desktop-wallpapers/15.webp")).unwrap(),
            b"wp"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn hash_skip_needs_matching_files() {
        let root = temp_root();
        let current = current_dir(&root);
        fs::create_dir_all(current.join("desktop-wallpapers")).unwrap();
        fs::write(current.join("desktop-wallpapers/15.webp"), b"wp").unwrap();
        let digest = sha256_file(&current.join("desktop-wallpapers/15.webp")).unwrap();
        let files = FilesManifest {
            kind: KIND_FOFLINE_SPA_FILES.into(),
            files: vec![FilesEntry {
                path: "desktop-wallpapers/15.webp".into(),
                sha256: digest,
            }],
        };
        assert!(local_files_complete(&current, &files));
        fs::write(current.join("desktop-wallpapers/15.webp"), b"xx").unwrap();
        assert!(!local_files_complete(&current, &files));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn observe_empty_is_idle() {
        let root = temp_root();
        let view = observe(&root);
        assert_eq!(view.code, "assets_idle");
        assert_eq!(json!(view.ready), json!(0));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn audio_guides_is_the_deferred_id() {
        assert_eq!(DEFERRED_GROUP, "audio-guides");
        assert!(valid_group_id(DEFERRED_GROUP));
    }
}
