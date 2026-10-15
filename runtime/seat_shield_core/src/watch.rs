//! S5B bounded tmp/shm watch journal. Helper writes; Runtime drains.
//! Helper never calls nft and never quarantines.

use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::error::{ShieldError, ShieldResult};
use crate::event::{Severity, ShieldEvent};
use crate::quarantine::{is_under_scan_root, SCAN_ROOTS};
use crate::store::ShieldStore;
use crate::timeutil::now_epoch;

pub const MAX_WATCH_RECORDS: usize = 200;
pub const WATCH_OPS: &[&str] = &["create", "close_write", "moved_to", "inject"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WatchRecord {
    pub epoch: u64,
    pub path: String,
    pub op: String,
    /// Helper-side SHA-256 when the file was visible to the watch unit.
    /// Runtime uses this when its own view of the path is gone (PrivateTmp).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
}

impl WatchRecord {
    #[must_use]
    pub fn new(epoch: u64, path: impl Into<String>, op: impl Into<String>) -> Self {
        Self {
            epoch,
            path: path.into(),
            op: op.into(),
            sha256: None,
        }
    }

    #[must_use]
    pub fn with_helper_hash(mut self) -> Self {
        self.sha256 = hash_allowed_watch_file(&self.path);
        self
    }
}

/// Hash a watch path only if it is a regular allowlisted file (no symlink escape).
#[must_use]
pub fn hash_allowed_watch_file(raw: &str) -> Option<String> {
    let path = crate::quarantine::resolve_allowed_file(raw).ok()?;
    crate::quarantine::sha256_file_capped(&path)
        .ok()
        .flatten()
        .map(|(digest, _)| digest)
}

/// Journal hash is usable only for allowlisted roots and a well-formed digest.
#[must_use]
pub fn usable_journal_hash(record: &WatchRecord) -> Option<String> {
    watch_path_allowed(&record.path).ok()?;
    let digest = record.sha256.as_deref()?;
    if crate::quarantine::valid_sha256_hex(digest) {
        Some(digest.to_ascii_lowercase())
    } else {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchCli {
    pub state_dir: PathBuf,
    pub duration: Option<Duration>,
    pub inject: Option<String>,
}

fn deny(detail: impl Into<String>) -> ShieldError {
    ShieldError::ActDenied(detail.into())
}

pub fn parse_watch_cli(args: &[String]) -> Result<WatchCli, String> {
    let mut state_dir = None;
    let mut duration = None;
    let mut inject = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--state-dir" => {
                let value = args.get(i + 1).ok_or("--state-dir requires a path")?;
                if !value.starts_with('/') {
                    return Err("--state-dir must be absolute".into());
                }
                state_dir = Some(PathBuf::from(value));
                i += 2;
            }
            "--duration-ms" => {
                let raw = args.get(i + 1).ok_or("--duration-ms requires a number")?;
                let ms: u64 = raw
                    .parse()
                    .map_err(|_| "--duration-ms must be u64".to_string())?;
                duration = Some(Duration::from_millis(ms));
                i += 2;
            }
            "--inject" => {
                let value = args.get(i + 1).ok_or("--inject requires a path")?;
                inject = Some(value.clone());
                i += 2;
            }
            "block" | "unblock" | "list" | "--nft-bin" | "--backend" => {
                return Err("watch helper must not call nft or firewall acts".into());
            }
            other => return Err(format!("unknown watch argument: {other}")),
        }
    }
    let state_dir = state_dir.ok_or("--state-dir is required")?;
    Ok(WatchCli {
        state_dir,
        duration,
        inject,
    })
}

pub fn watch_path_allowed(raw: &str) -> ShieldResult<PathBuf> {
    if !raw.starts_with('/') || raw.contains("..") {
        return Err(deny("watch path must be absolute and must not contain .."));
    }
    let path = Path::new(raw);
    if !is_under_scan_root(path) {
        return Err(deny("watch path is outside /tmp /var/tmp /dev/shm"));
    }
    Ok(path.to_path_buf())
}

impl ShieldStore {
    #[must_use]
    pub fn watch_path(&self) -> PathBuf {
        self.root().join("watch.jsonl")
    }

    pub fn watch_pending(&self) -> ShieldResult<usize> {
        Ok(self.read_watch()?.len())
    }

    pub fn read_watch(&self) -> ShieldResult<Vec<WatchRecord>> {
        let path = self.watch_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let file = fs::File::open(&path).map_err(|source| ShieldError::Io {
            context: "watch_read",
            source,
        })?;
        let mut items = Vec::new();
        for line in BufReader::new(file).lines() {
            let line = line.map_err(|source| ShieldError::Io {
                context: "watch_read_line",
                source,
            })?;
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<WatchRecord>(&line) {
                Ok(item) => items.push(item),
                Err(_) => continue,
            }
        }
        Ok(items)
    }

    pub fn append_watch(&self, record: &WatchRecord) -> ShieldResult<()> {
        watch_path_allowed(&record.path)?;
        if !WATCH_OPS.contains(&record.op.as_str()) {
            return Err(deny("watch op is not allowlisted"));
        }
        let mut items = self.read_watch()?;
        items.push(record.clone());
        if items.len() > MAX_WATCH_RECORDS {
            let drop_n = items.len() - MAX_WATCH_RECORDS;
            items.drain(0..drop_n);
        }
        self.write_watch(&items)
    }

    pub fn drain_watch(&self) -> ShieldResult<Vec<WatchRecord>> {
        let items = self.read_watch()?;
        if self.watch_path().exists() {
            fs::write(self.watch_path(), "").map_err(|source| ShieldError::Io {
                context: "watch_drain",
                source,
            })?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(self.watch_path(), fs::Permissions::from_mode(0o600));
            }
        }
        Ok(items)
    }

    fn write_watch(&self, items: &[WatchRecord]) -> ShieldResult<()> {
        let path = self.watch_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| ShieldError::Io {
                context: "watch_dir_create",
                source,
            })?;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)
            .map_err(|source| ShieldError::Io {
                context: "watch_write",
                source,
            })?;
        for item in items {
            let line = serde_json::to_string(item).map_err(|e| ShieldError::Serialize(e.to_string()))?;
            file.write_all(line.as_bytes()).map_err(|source| ShieldError::Io {
                context: "watch_write_line",
                source,
            })?;
            file.write_all(b"\n").map_err(|source| ShieldError::Io {
                context: "watch_write_nl",
                source,
            })?;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
        }
        Ok(())
    }
}

pub fn event_file_watch_seen(basename: &str) -> ShieldEvent {
    ShieldEvent {
        kind: "file_watch_seen".into(),
        severity: Severity::Low,
        summary: format!("Watch saw {basename}"),
        entity_ip: None,
        entity_process: Some(basename.to_string()),
        entity_port: None,
        explain_why: "inotify on an allowlisted tmp/shm root (non-recursive).".into(),
        explain_action: "Runtime hashed on drain. Hold only after ioc_hash_match + ENABLE_QUARANTINE."
            .into(),
        confidence_milli: 700,
        tags: vec!["seat".into(), "file".into(), "watch".into()],
        sequence: 0,
        epoch: 0,
        chain_id: None,
    }
}

pub fn inject_watch_path(store: &ShieldStore, raw: &str) -> ShieldResult<WatchRecord> {
    let path = watch_path_allowed(raw)?;
    let record = WatchRecord::new(now_epoch(), path.to_string_lossy().into_owned(), "inject")
        .with_helper_hash();
    store.append_watch(&record)?;
    Ok(record)
}

/// Non-recursive inotify on `/tmp` `/var/tmp` `/dev/shm` only.
pub fn run_inotify_loop(store: &ShieldStore, duration: Option<Duration>) -> ShieldResult<u32> {
    let mut inotify = inotify::Inotify::init().map_err(|source| ShieldError::Io {
        context: "watch_inotify_init",
        source,
    })?;
    let mask = inotify::WatchMask::CREATE
        | inotify::WatchMask::MOVED_TO
        | inotify::WatchMask::CLOSE_WRITE;
    let mut wds = Vec::new();
    for root in SCAN_ROOTS {
        let dir = Path::new(root.trim_end_matches('/'));
        if !dir.is_dir() {
            continue;
        }
        match inotify.watches().add(dir, mask) {
            Ok(wd) => wds.push((wd, (*root).to_string())),
            Err(_) => continue,
        }
    }
    if wds.is_empty() {
        return Err(deny("no allowlisted watch roots exist"));
    }
    let deadline = duration.map(|d| Instant::now() + d);
    let mut written = 0_u32;
    let mut buf = [0_u8; 4096];
    loop {
        if deadline.is_some_and(|end| Instant::now() >= end) {
            break;
        }
        let timeout = deadline.map_or(Duration::from_millis(250), |end| {
            end.saturating_duration_since(Instant::now())
                .min(Duration::from_millis(250))
        });
        match inotify.read_events(&mut buf) {
            Ok(events) => {
                for event in events {
                    if event.mask.contains(inotify::EventMask::ISDIR) {
                        continue;
                    }
                    let Some(name) = event.name else {
                        continue;
                    };
                    let name = name.to_string_lossy();
                    if name.contains("..") || name.contains('/') {
                        continue;
                    }
                    let Some((_, root)) = wds.iter().find(|(wd, _)| *wd == event.wd) else {
                        continue;
                    };
                    let path = Path::new(root.trim_end_matches('/')).join(name.as_ref());
                    let op = if event.mask.contains(inotify::EventMask::MOVED_TO) {
                        "moved_to"
                    } else if event.mask.contains(inotify::EventMask::CLOSE_WRITE) {
                        "close_write"
                    } else {
                        "create"
                    };
                    let record = WatchRecord::new(
                        now_epoch(),
                        path.to_string_lossy().into_owned(),
                        op,
                    )
                    .with_helper_hash();
                    if store.append_watch(&record).is_ok() {
                        written = written.saturating_add(1);
                    }
                }
            }
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(source) => {
                return Err(ShieldError::Io {
                    context: "watch_inotify_read",
                    source,
                });
            }
        }
        if duration.is_some() {
            std::thread::sleep(timeout.min(Duration::from_millis(25)));
        } else {
            std::thread::sleep(Duration::from_millis(100));
        }
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tmp_store() -> ShieldStore {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        ShieldStore::open(std::env::temp_dir().join(format!("seat_watch_{stamp}"))).unwrap()
    }

    #[test]
    fn parse_cli_rejects_nft_verbs() {
        let err = parse_watch_cli(&[
            "--state-dir".into(),
            "/tmp/x".into(),
            "block".into(),
        ])
        .unwrap_err();
        assert!(err.contains("nft"));
    }

    #[test]
    fn append_and_drain_roundtrip() {
        let store = tmp_store();
        let path = format!(
            "/tmp/seat_s5b_{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        std::fs::write(&path, b"watch-bytes").unwrap();
        store
            .append_watch(&WatchRecord::new(1, path.clone(), "create"))
            .unwrap();
        assert_eq!(store.watch_pending().unwrap(), 1);
        let drained = store.drain_watch().unwrap();
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].path, path);
        assert_eq!(store.watch_pending().unwrap(), 0);
    }

    #[test]
    fn outside_root_is_denied() {
        let store = tmp_store();
        let err = store.append_watch(&WatchRecord::new(1, "/etc/passwd", "create"));
        assert!(err.is_err());
        assert_eq!(store.watch_pending().unwrap(), 0);
    }

    #[test]
    fn cap_keeps_last_two_hundred() {
        let store = tmp_store();
        for i in 0..205 {
            store
                .append_watch(&WatchRecord::new(
                    i,
                    format!("/tmp/seat_s5b_cap_{i}"),
                    "create",
                ))
                .unwrap();
        }
        let items = store.read_watch().unwrap();
        assert_eq!(items.len(), 200);
        assert_eq!(items[0].epoch, 5);
        assert_eq!(items[199].epoch, 204);
    }

    #[test]
    fn helper_hash_skips_symlink_and_outside_root() {
        assert!(hash_allowed_watch_file("/etc/passwd").is_none());
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = PathBuf::from(format!("/tmp/seat_s5c_hash_{stamp}"));
        std::fs::write(&path, b"s5c-hash-bytes").unwrap();
        let digest = hash_allowed_watch_file(&path.to_string_lossy());
        assert!(digest.is_some(), "{digest:?}");
        assert_eq!(digest.unwrap().len(), 64);
    }

    #[test]
    fn old_journal_line_without_sha256_deserializes() {
        let rec: WatchRecord = serde_json::from_str(
            r#"{"epoch":1,"path":"/tmp/x","op":"create"}"#,
        )
        .unwrap();
        assert_eq!(rec.sha256, None);
        assert!(usable_journal_hash(&rec).is_none());
    }

    #[test]
    fn inotify_loop_exits_on_duration() {
        let store = tmp_store();
        let written = run_inotify_loop(&store, Some(Duration::from_millis(40))).unwrap();
        assert!(written < MAX_WATCH_RECORDS as u32);
    }
}
