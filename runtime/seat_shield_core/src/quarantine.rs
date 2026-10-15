//! Seat-shaped file hold (S5A). Allowlisted tmp/shm only. Not on-access AV.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::error::{ShieldError, ShieldResult};
use crate::event::{Severity, ShieldEvent};
use crate::store::ShieldStore;
use crate::timeutil::now_epoch;

/// S5A scan / quarantine roots. Not policy-blob expandable.
pub const SCAN_ROOTS: &[&str] = &["/tmp/", "/var/tmp/", "/dev/shm/"];
pub const MAX_FILE_BYTES: u64 = 32 * 1024 * 1024;
pub const MAX_SCAN_FILES: usize = 16;
pub const SHA256_HEX_LEN: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuarantineSidecar {
    pub orig_path: String,
    pub epoch: u64,
    pub reason: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuarantineListItem {
    pub sha256: String,
    pub size: u64,
    pub epoch: u64,
    pub reason: String,
    pub orig_basename: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanOutcome {
    pub path_basename: String,
    pub sha256: Option<String>,
    pub size: u64,
    pub skipped: bool,
    pub outside_or_illegal: bool,
    pub feed_hit: bool,
    pub events: Vec<ShieldEvent>,
}

#[must_use]
pub fn is_under_scan_root(path: &Path) -> bool {
    let raw = path.to_string_lossy();
    SCAN_ROOTS.iter().any(|root| raw.starts_with(root))
}

fn deny(detail: impl Into<String>) -> ShieldError {
    ShieldError::ActDenied(detail.into())
}

/// Absolute path, no `..`, realpath stays under a scan root, regular file.
/// Allowlist is checked before stat so `/etc/missing` is a policy deny, not ENOENT.
pub fn resolve_allowed_file(raw: &str) -> ShieldResult<PathBuf> {
    if !raw.starts_with('/') {
        return Err(deny("scan path must be absolute"));
    }
    if raw.contains("..") {
        return Err(deny("scan path must not contain .."));
    }
    let path = Path::new(raw);
    if !is_under_scan_root(path) {
        return Err(deny("path is outside allowlisted scan roots"));
    }
    let meta = fs::symlink_metadata(path).map_err(|source| ShieldError::Io {
        context: "scan_path_stat",
        source,
    })?;
    if meta.file_type().is_symlink() {
        let dest = fs::read_link(path).map_err(|source| ShieldError::Io {
            context: "scan_path_readlink",
            source,
        })?;
        let resolved = if dest.is_absolute() {
            dest
        } else {
            path.parent().unwrap_or(Path::new("/")).join(dest)
        };
        let canon = resolved.canonicalize().map_err(|_| deny("symlink escape"))?;
        if !is_under_scan_root(&canon) {
            return Err(deny("symlink escape"));
        }
        if !canon.is_file() {
            return Err(deny("scan path must be a regular file"));
        }
        return Ok(canon);
    }
    if !meta.is_file() {
        return Err(deny("scan path must be a regular file"));
    }
    let canon = path.canonicalize().map_err(|source| ShieldError::Io {
        context: "scan_path_canonicalize",
        source,
    })?;
    if !is_under_scan_root(&canon) {
        return Err(deny("path is outside allowlisted scan roots"));
    }
    Ok(canon)
}

fn restore_dest_allowed(orig_path: &str) -> ShieldResult<PathBuf> {
    if !orig_path.starts_with('/') || orig_path.contains("..") {
        return Err(deny("restore dest is no longer legal"));
    }
    let dest = Path::new(orig_path);
    if dest.exists() {
        return Err(deny("restore dest already exists"));
    }
    let parent = dest
        .parent()
        .ok_or_else(|| deny("restore dest has no parent"))?;
    let parent_canon = parent
        .canonicalize()
        .map_err(|_| deny("restore dest parent is not legal"))?;
    let name = dest
        .file_name()
        .ok_or_else(|| deny("restore dest has no name"))?;
    let joined = parent_canon.join(name);
    if !is_under_scan_root(&joined) {
        return Err(deny("restore dest is no longer under a scan root"));
    }
    Ok(joined)
}

pub fn sha256_file_capped(path: &Path) -> ShieldResult<Option<(String, u64)>> {
    let meta = fs::metadata(path).map_err(|source| ShieldError::Io {
        context: "hash_stat",
        source,
    })?;
    let size = meta.len();
    if size > MAX_FILE_BYTES {
        return Ok(None);
    }
    let mut file = File::open(path).map_err(|source| ShieldError::Io {
        context: "hash_open",
        source,
    })?;
    let mut hasher = Sha256::new();
    let mut buf = [0_u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf).map_err(|source| ShieldError::Io {
            context: "hash_read",
            source,
        })?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(Some((format!("{:x}", hasher.finalize()), size)))
}

#[must_use]
pub fn valid_sha256_hex(value: &str) -> bool {
    value.len() == SHA256_HEX_LEN && value.bytes().all(|b| b.is_ascii_hexdigit())
}

impl ShieldStore {
    #[must_use]
    pub fn quarantine_dir(&self) -> PathBuf {
        self.root().join("quarantine")
    }

    pub fn ensure_quarantine_dir(&self) -> ShieldResult<PathBuf> {
        let dir = self.quarantine_dir();
        fs::create_dir_all(&dir).map_err(|source| ShieldError::Io {
            context: "quarantine_dir_create",
            source,
        })?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&dir, fs::Permissions::from_mode(0o700));
        }
        Ok(dir)
    }

    pub fn quarantine_count(&self) -> ShieldResult<usize> {
        Ok(self.list_quarantine()?.len())
    }

    pub fn list_quarantine(&self) -> ShieldResult<Vec<QuarantineListItem>> {
        let dir = self.quarantine_dir();
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut items = Vec::new();
        let entries = fs::read_dir(&dir).map_err(|source| ShieldError::Io {
            context: "quarantine_list",
            source,
        })?;
        for entry in entries {
            let entry = entry.map_err(|source| ShieldError::Io {
                context: "quarantine_list_entry",
                source,
            })?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !name.ends_with(".json") {
                continue;
            }
            let raw = fs::read_to_string(entry.path()).map_err(|source| ShieldError::Io {
                context: "quarantine_sidecar_read",
                source,
            })?;
            let side: QuarantineSidecar =
                serde_json::from_str(&raw).map_err(|e| ShieldError::Parse {
                    context: "quarantine_sidecar",
                    detail: e.to_string(),
                })?;
            let orig = Path::new(&side.orig_path);
            items.push(QuarantineListItem {
                sha256: side.sha256,
                size: side.size,
                epoch: side.epoch,
                reason: side.reason,
                orig_basename: orig
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "unknown".into()),
            });
        }
        items.sort_by(|a, b| a.sha256.cmp(&b.sha256));
        Ok(items)
    }

    pub fn hold_file(&self, raw_path: &str, reason: &str) -> ShieldResult<QuarantineSidecar> {
        let path = resolve_allowed_file(raw_path)?;
        let hashed = sha256_file_capped(&path)?.ok_or_else(|| deny("file exceeds 32 MiB"))?;
        let (digest, size) = hashed;
        let dir = self.ensure_quarantine_dir()?;
        let dest = dir.join(&digest);
        if dest.exists() {
            return Err(deny("sha256 already held"));
        }
        move_file(&path, &dest)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&dest, fs::Permissions::from_mode(0o600));
        }
        let side = QuarantineSidecar {
            orig_path: path.to_string_lossy().into_owned(),
            epoch: now_epoch(),
            reason: reason.to_string(),
            size,
            sha256: digest.clone(),
        };
        let side_path = dir.join(format!("{digest}.json"));
        let raw = serde_json::to_string_pretty(&side)
            .map_err(|e| ShieldError::Serialize(e.to_string()))?;
        fs::write(&side_path, raw).map_err(|source| ShieldError::Io {
            context: "quarantine_sidecar_write",
            source,
        })?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&side_path, fs::Permissions::from_mode(0o600));
        }
        Ok(side)
    }

    pub fn restore_held(&self, sha256: &str) -> ShieldResult<QuarantineSidecar> {
        if !valid_sha256_hex(sha256) {
            return Err(ShieldError::InvalidParams(
                "params.sha256 must be 64 hex chars".into(),
            ));
        }
        let digest = sha256.to_ascii_lowercase();
        let dir = self.quarantine_dir();
        let blob = dir.join(&digest);
        let side_path = dir.join(format!("{digest}.json"));
        if !blob.is_file() || !side_path.is_file() {
            return Err(deny("quarantine record not found"));
        }
        let raw = fs::read_to_string(&side_path).map_err(|source| ShieldError::Io {
            context: "quarantine_sidecar_read",
            source,
        })?;
        let side: QuarantineSidecar =
            serde_json::from_str(&raw).map_err(|e| ShieldError::Parse {
                context: "quarantine_sidecar",
                detail: e.to_string(),
            })?;
        let dest = restore_dest_allowed(&side.orig_path)?;
        move_file(&blob, &dest)?;
        fs::remove_file(&side_path).map_err(|source| ShieldError::Io {
            context: "quarantine_sidecar_remove",
            source,
        })?;
        Ok(side)
    }
}

fn move_file(from: &Path, to: &Path) -> ShieldResult<()> {
    match fs::rename(from, to) {
        Ok(()) => Ok(()),
        Err(err) if err.raw_os_error() == Some(18) => copy_then_unlink(from, to),
        Err(source) => Err(ShieldError::Io {
            context: "quarantine_rename",
            source,
        }),
    }
}

fn copy_then_unlink(from: &Path, to: &Path) -> ShieldResult<()> {
    {
        let mut src = File::open(from).map_err(|source| ShieldError::Io {
            context: "quarantine_copy_open",
            source,
        })?;
        let mut dst = File::create(to).map_err(|source| ShieldError::Io {
            context: "quarantine_copy_create",
            source,
        })?;
        let mut buf = [0_u8; 64 * 1024];
        loop {
            let n = src.read(&mut buf).map_err(|source| ShieldError::Io {
                context: "quarantine_copy_read",
                source,
            })?;
            if n == 0 {
                break;
            }
            dst.write_all(&buf[..n]).map_err(|source| ShieldError::Io {
                context: "quarantine_copy_write",
                source,
            })?;
        }
    }
    fs::remove_file(from).map_err(|source| ShieldError::Io {
        context: "quarantine_unlink_src",
        source,
    })
}

pub fn event_file_scan_hit(basename: &str, digest: &str) -> ShieldEvent {
    ShieldEvent {
        kind: "file_scan_hit".into(),
        severity: Severity::Critical,
        summary: format!("Scan hit {basename}"),
        entity_ip: None,
        entity_process: Some(basename.to_string()),
        entity_port: None,
        explain_why: format!(
            "SHA-256 matches sealed feed (digest length={}).",
            digest.len()
        ),
        explain_action: "Hold with seat.shield.quarantine + QUARANTINE_FILE if needed.".into(),
        confidence_milli: 990,
        tags: vec!["seat".into(), "file".into(), "scan".into()],
        sequence: 0,
        epoch: 0,
        chain_id: None,
    }
}

pub fn event_ioc_hash_from_scan(basename: &str, digest: &str) -> ShieldEvent {
    ShieldEvent {
        kind: "ioc_hash_match".into(),
        severity: Severity::Critical,
        summary: format!("Exec hash match for {basename}"),
        entity_ip: None,
        entity_process: Some(basename.to_string()),
        entity_port: None,
        explain_why: format!(
            "SHA-256 matches sealed feed (digest length={}).",
            digest.len()
        ),
        explain_action: "On-demand scan hit. Quarantine is confirm-gated.".into(),
        confidence_milli: 990,
        tags: vec!["seat".into(), "ioc".into(), "hash".into()],
        sequence: 0,
        epoch: 0,
        chain_id: None,
    }
}

pub fn event_file_scan_denied(detail: &str) -> ShieldEvent {
    ShieldEvent {
        kind: "file_scan_denied".into(),
        severity: Severity::Low,
        summary: "Scan path denied".into(),
        entity_ip: None,
        entity_process: None,
        entity_port: None,
        explain_why: detail.to_string(),
        explain_action: "Stay inside /tmp /var/tmp /dev/shm. No directory walk of the seat.".into(),
        confidence_milli: 800,
        tags: vec!["seat".into(), "file".into(), "denied".into()],
        sequence: 0,
        epoch: 0,
            chain_id: None,
    }
}

pub fn event_file_quarantined(basename: &str, digest: &str, reason: &str) -> ShieldEvent {
    ShieldEvent {
        kind: "file_quarantined".into(),
        severity: Severity::High,
        summary: format!("Held {basename}"),
        entity_ip: None,
        entity_process: Some(basename.to_string()),
        entity_port: None,
        explain_why: format!(
            "File renamed into seat_shield/quarantine ({reason}, digest length={}).",
            digest.len()
        ),
        explain_action: "Restore only with RESTORE_QUARANTINE if dest is still legal.".into(),
        confidence_milli: 950,
        tags: vec!["seat".into(), "file".into(), "quarantine".into()],
        sequence: 0,
        epoch: 0,
        chain_id: None,
    }
}

pub fn event_file_restored(basename: &str) -> ShieldEvent {
    ShieldEvent {
        kind: "file_restored".into(),
        severity: Severity::Medium,
        summary: format!("Restored {basename}"),
        entity_ip: None,
        entity_process: Some(basename.to_string()),
        entity_port: None,
        explain_why: "Operator restored a held file to an allowlisted dest.".into(),
        explain_action: "Re-scan if the file must stay held.".into(),
        confidence_milli: 900,
        tags: vec!["seat".into(), "file".into(), "restore".into()],
        sequence: 0,
        epoch: 0,
        chain_id: None,
    }
}

fn scan_basename(raw_path: &str) -> String {
    Path::new(raw_path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unknown".into())
}

fn skipped_absent(raw_path: &str) -> ScanOutcome {
    ScanOutcome {
        path_basename: scan_basename(raw_path),
        sha256: None,
        size: 0,
        skipped: true,
        outside_or_illegal: false,
        feed_hit: false,
        events: Vec::new(),
    }
}

pub fn scan_regular_file(raw_path: &str, feed_hashes: &[String]) -> ShieldResult<ScanOutcome> {
    let path = match resolve_allowed_file(raw_path) {
        Ok(path) => path,
        Err(err) if err.is_scan_path_absent() || matches!(err, ShieldError::Io { .. }) => {
            return Ok(skipped_absent(raw_path));
        }
        Err(err) => {
            return Ok(ScanOutcome {
                path_basename: scan_basename(raw_path),
                sha256: None,
                size: 0,
                skipped: true,
                outside_or_illegal: true,
                feed_hit: false,
                events: vec![event_file_scan_denied(&err.to_string())],
            });
        }
    };
    let basename = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unknown".into());
    let Some((digest, size)) = sha256_file_capped(&path)? else {
        return Ok(ScanOutcome {
            path_basename: basename,
            sha256: None,
            size: 0,
            skipped: true,
            outside_or_illegal: false,
            feed_hit: false,
            events: vec![event_file_scan_denied("file exceeds 32 MiB")],
        });
    };
    let feed_hit = feed_hashes
        .iter()
        .any(|h| h.eq_ignore_ascii_case(&digest));
    let mut events = Vec::new();
    if feed_hit {
        events.push(event_file_scan_hit(&basename, &digest));
        events.push(event_ioc_hash_from_scan(&basename, &digest));
    }
    let _ = MAX_SCAN_FILES;
    Ok(ScanOutcome {
        path_basename: basename,
        sha256: Some(digest),
        size,
        skipped: false,
        outside_or_illegal: false,
        feed_hit,
        events,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_tmp_file_is_skip_not_denied() {
        let out = scan_regular_file("/tmp/seat_s5c_absent_unit", &[]).unwrap();
        assert!(out.skipped);
        assert!(!out.outside_or_illegal);
        assert!(out.events.is_empty());
    }

    #[test]
    fn missing_etc_is_policy_denied() {
        let out = scan_regular_file("/etc/seat_s5c_absent_unit", &[]).unwrap();
        assert!(out.outside_or_illegal);
        assert!(out.events.iter().any(|e| e.kind == "file_scan_denied"));
    }

    #[test]
    fn etc_passwd_is_policy_denied() {
        let out = scan_regular_file("/etc/passwd", &[]).unwrap();
        assert!(out.outside_or_illegal);
        assert!(out.events.iter().any(|e| e.kind == "file_scan_denied"));
        assert!(out.sha256.is_none());
    }
}
