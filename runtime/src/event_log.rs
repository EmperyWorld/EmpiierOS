use crate::system_probe::SystemSnapshot;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::VecDeque;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::{Condvar, Mutex};
use std::time::{Duration, Instant};

const MAX_EVENTS: usize = 128;
const MAX_WAIT_MS: u64 = 5_000;
const EVENTS_FILE: &str = "events.jsonl";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RuntimeEvent {
    pub sequence: u64,
    pub observed_at_epoch: u64,
    pub domain: String,
    pub kind: String,
    pub summary: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct EventBatch {
    pub latest_sequence: u64,
    pub truncated: bool,
    pub events: Vec<RuntimeEvent>,
}

struct Inner {
    next_sequence: u64,
    retained_from: u64,
    last_digest: Option<String>,
    events: VecDeque<RuntimeEvent>,
}

pub struct EventLog {
    inner: Mutex<Inner>,
    changed: Condvar,
    persist_path: Option<PathBuf>,
}

impl EventLog {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::from_inner(
            Inner {
                next_sequence: 0,
                retained_from: 0,
                last_digest: None,
                events: VecDeque::new(),
            },
            None,
        )
    }

    pub fn open(state_root: &Path) -> Result<Self, String> {
        let path = state_root.join(EVENTS_FILE);
        let inner = load_persisted(&path)?;
        Ok(Self::from_inner(inner, Some(path)))
    }

    fn from_inner(inner: Inner, persist_path: Option<PathBuf>) -> Self {
        Self {
            inner: Mutex::new(inner),
            changed: Condvar::new(),
            persist_path,
        }
    }

    pub fn observe(&self, started: Instant) -> SystemSnapshot {
        let state_root = self.persist_path.as_deref().and_then(|path| path.parent());
        let mut snapshot = crate::system_probe::snapshot(started, state_root);
        let digest = snapshot_digest(&snapshot);
        let mut inner = self.inner.lock().expect("event log lock");
        inner.next_sequence = inner.next_sequence.saturating_add(1);
        snapshot.sequence = inner.next_sequence;
        if inner.last_digest.as_ref() != Some(&digest) {
            let kind = if inner.last_digest.is_some() {
                "changed"
            } else {
                "observed"
            };
            let event = RuntimeEvent {
                sequence: snapshot.sequence,
                observed_at_epoch: snapshot.observed_at_epoch,
                domain: "system".to_string(),
                kind: kind.to_string(),
                summary: format!(
                    "snapshot {} overall={}",
                    snapshot.sequence, snapshot.overall_status
                ),
            };
            persist_event(self.persist_path.as_deref(), &event);
            inner.events.push_back(event);
            while inner.events.len() > MAX_EVENTS {
                inner.events.pop_front();
                inner.retained_from = inner
                    .events
                    .front()
                    .map(|item| item.sequence)
                    .unwrap_or(snapshot.sequence);
            }
            inner.last_digest = Some(digest);
            self.changed.notify_all();
        }
        snapshot
    }

    pub fn after(&self, after_sequence: u64) -> EventBatch {
        let inner = self.inner.lock().expect("event log lock");
        batch_from(&inner, after_sequence)
    }

    pub fn wait_after(&self, after_sequence: u64, wait: Duration) -> EventBatch {
        let inner = self.inner.lock().expect("event log lock");
        if inner.events.iter().any(|event| event.sequence > after_sequence) {
            return batch_from(&inner, after_sequence);
        }
        let (guard, _timed_out) = self
            .changed
            .wait_timeout(inner, wait)
            .expect("event log condvar");
        batch_from(&guard, after_sequence)
    }
}

fn batch_from(inner: &Inner, after_sequence: u64) -> EventBatch {
    let truncated = after_sequence.saturating_add(1) < inner.retained_from;
    let events = inner
        .events
        .iter()
        .filter(|event| event.sequence > after_sequence)
        .cloned()
        .collect();
    EventBatch {
        latest_sequence: inner.next_sequence,
        truncated,
        events,
    }
}

pub fn events_from_params(log: &EventLog, params: &Value) -> Result<Value, String> {
    let after_sequence = params
        .get("after_sequence")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    serde_json::to_value(log.after(after_sequence))
        .map_err(|error| format!("event_serialization_failed: {error}"))
}

pub fn watch_from_params(
    log: &EventLog,
    started: Instant,
    params: &Value,
) -> Result<Value, String> {
    let after_sequence = params
        .get("after_sequence")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let wait_ms = params.get("wait_ms").and_then(Value::as_u64).unwrap_or(0);
    if wait_ms > MAX_WAIT_MS {
        return Err(format!("wait_ms_exceeds_{MAX_WAIT_MS}"));
    }
    let batch = if wait_ms == 0 {
        log.after(after_sequence)
    } else {
        log.wait_after(after_sequence, Duration::from_millis(wait_ms))
    };
    let snapshot = log.observe(started);
    let correlated = log.after(after_sequence);
    serde_json::to_value(json!({
        "latest_sequence": correlated.latest_sequence.max(batch.latest_sequence),
        "truncated": correlated.truncated || batch.truncated,
        "events": if correlated.events.is_empty() { batch.events } else { correlated.events },
        "snapshot": snapshot,
    }))
    .map_err(|error| format!("watch_serialization_failed: {error}"))
}

fn snapshot_digest(snapshot: &SystemSnapshot) -> String {
    json!({
        "overall": snapshot.overall_status,
        "network": snapshot.network.status,
        "power": snapshot.power.status,
        "security": snapshot.security.status,
        "wifi": snapshot.wifi.availability,
        "display": snapshot.display.availability,
        "services": snapshot.services.availability,
    })
    .to_string()
}

fn load_persisted(path: &Path) -> Result<Inner, String> {
    let mut inner = Inner {
        next_sequence: 0,
        retained_from: 0,
        last_digest: None,
        events: VecDeque::new(),
    };
    match fs::symlink_metadata(path) {
        Ok(meta) => {
            if meta.file_type().is_symlink() || !meta.file_type().is_file() {
                return Err("events path is not a regular file".to_string());
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(inner),
        Err(error) => return Err(format!("events_stat_failed: {error}")),
    }
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open(path)
        .map_err(|error| format!("events_open_failed: {error}"))?;
    for line in BufReader::new(file).lines() {
        let line = line.map_err(|error| format!("events_read_failed: {error}"))?;
        if line.trim().is_empty() {
            continue;
        }
        let event: RuntimeEvent = serde_json::from_str(&line)
            .map_err(|error| format!("events_parse_failed: {error}"))?;
        inner.next_sequence = inner.next_sequence.max(event.sequence);
        inner.events.push_back(event);
        while inner.events.len() > MAX_EVENTS {
            inner.events.pop_front();
        }
    }
    inner.retained_from = inner
        .events
        .front()
        .map(|event| event.sequence)
        .unwrap_or(0);
    Ok(inner)
}

fn persist_event(path: Option<&Path>, event: &RuntimeEvent) {
    let Some(path) = path else {
        return;
    };
    let Ok(mut line) = serde_json::to_vec(event) else {
        return;
    };
    line.push(b'\n');
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o600)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open(path)
    {
        let _ = file.set_permissions(fs::Permissions::from_mode(0o600));
        let _ = file.write_all(&line);
        let _ = file.sync_data();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn sequence_advances_and_events_are_bounded() {
        let log = EventLog::new();
        let first = log.observe(Instant::now());
        let second = log.observe(Instant::now());
        assert!(first.sequence >= 1);
        assert!(second.sequence > first.sequence);
        let batch = log.after(0);
        assert_eq!(batch.latest_sequence, second.sequence);
        assert!(!batch.events.is_empty());
        assert!(!batch.truncated);
        assert_eq!(first.freshness.reason, "observed_now");
        assert!(!first.freshness.stale);
    }

    #[test]
    fn persist_survives_reopen_and_rejects_symlink() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "empiieros-events-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("dir");
        let first = EventLog::open(&root).expect("open");
        let observed = first.observe(Instant::now());
        drop(first);
        let second = EventLog::open(&root).expect("reopen");
        let batch = second.after(0);
        assert!(batch
            .events
            .iter()
            .any(|event| event.sequence == observed.sequence));

        let target = root.join("target");
        fs::write(&target, b"x").expect("target");
        let link = root.join(EVENTS_FILE);
        let _ = fs::remove_file(&link);
        std::os::unix::fs::symlink(&target, &link).expect("symlink");
        assert!(EventLog::open(&root).is_err());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn watch_rejects_oversized_wait() {
        let log = EventLog::new();
        let error = watch_from_params(
            &log,
            Instant::now(),
            &json!({"after_sequence": 0, "wait_ms": 5001}),
        )
        .expect_err("wait");
        assert!(error.contains("wait_ms_exceeds_"));
    }
}
