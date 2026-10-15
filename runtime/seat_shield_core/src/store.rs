//! Durable `{state_dir}/seat_shield/` layout.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{ShieldError, ShieldResult};
use crate::event::ShieldEvent;
use crate::feed::{FeedStatus, IocFeed};
use crate::heuristics::PeerWindow;
use crate::journal::append_journal_lines;
use crate::observe::ObservedConn;
use crate::policy::SeatPolicy;
use crate::timeutil::now_epoch;

pub const MAX_EVENTS: usize = 200;
pub const MAX_AUDIT: usize = 100;
/// Cap for last-observe connections persisted for Shell / status wire.
pub const MAX_CONNECTIONS_WIRE: usize = 128;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CycleState {
    pub last_cycle_epoch: u64,
    pub last_event_count: usize,
    pub last_act_count: usize,
    pub last_suppressed: u32,
    pub next_sequence: u64,
    pub last_observe_backend: String,
    /// Last time live observe considered `/proc/sys/kernel/tainted` (deep period).
    #[serde(default)]
    pub last_kernel_taint_epoch: u64,
    /// Last time Runtime drained `{state}/seat_shield/watch.jsonl`.
    #[serde(default)]
    pub last_watch_drained: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub epoch: u64,
    pub op: String,
    pub result: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct EventsFile {
    items: Vec<ShieldEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct AuditFile {
    items: Vec<AuditEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LastConnectionsFile {
    pub epoch: u64,
    pub connection_count: usize,
    #[serde(default)]
    pub truncated: bool,
    pub connections: Vec<ObservedConn>,
}

pub struct ShieldStore {
    state_root: PathBuf,
    root: PathBuf,
}

impl ShieldStore {
    pub fn open(state_root: impl Into<PathBuf>) -> ShieldResult<Self> {
        let state_root = state_root.into();
        let root = state_root.join("seat_shield");
        fs::create_dir_all(&root).map_err(|source| ShieldError::Io {
            context: "state_dir_create",
            source,
        })?;
        fs::create_dir_all(root.join("feed")).map_err(|source| ShieldError::Io {
            context: "feed_dir_create",
            source,
        })?;
        fs::create_dir_all(root.join("quarantine")).map_err(|source| ShieldError::Io {
            context: "quarantine_dir_create",
            source,
        })?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(root.join("quarantine"), fs::Permissions::from_mode(0o700));
        }
        let store = Self { state_root, root };
        if !store.policy_path().exists() {
            store.write_policy(&SeatPolicy::default())?;
        }
        if !store.cycle_path().exists() {
            store.write_cycle(&CycleState {
                last_observe_backend: "none".into(),
                ..CycleState::default()
            })?;
        }
        if !store.events_path().exists() {
            store.write_events(&[])?;
        }
        if !store.audit_path().exists() {
            store.write_audit(&[])?;
        }
        Ok(store)
    }

    #[must_use]
    pub fn state_root(&self) -> &Path {
        &self.state_root
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    fn policy_path(&self) -> PathBuf {
        self.root.join("policy.json")
    }

    fn cycle_path(&self) -> PathBuf {
        self.root.join("cycle_state.json")
    }

    fn events_path(&self) -> PathBuf {
        self.root.join("events.json")
    }

    #[must_use]
    pub fn journal_path(&self) -> PathBuf {
        self.root.join("journal.jsonl")
    }

    fn last_connections_path(&self) -> PathBuf {
        self.root.join("last_connections.json")
    }

    fn peer_window_path(&self) -> PathBuf {
        self.root.join("peer_window.json")
    }

    fn audit_path(&self) -> PathBuf {
        self.root.join("audit.json")
    }

    fn feed_file_path(&self) -> PathBuf {
        self.root.join("feed").join("ioc.json")
    }

    fn write_json(path: &Path, value: &impl Serialize) -> ShieldResult<()> {
        let raw = serde_json::to_string_pretty(value)
            .map_err(|e| ShieldError::Serialize(e.to_string()))?;
        fs::write(path, raw).map_err(|source| ShieldError::Io {
            context: "state_write",
            source,
        })?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
        }
        Ok(())
    }

    pub fn read_policy(&self) -> ShieldResult<SeatPolicy> {
        let raw = fs::read_to_string(self.policy_path()).map_err(|source| ShieldError::Io {
            context: "policy_read",
            source,
        })?;
        let mut policy: SeatPolicy = serde_json::from_str(&raw).map_err(|e| ShieldError::Parse {
            context: "policy_parse",
            detail: e.to_string(),
        })?;
        if crate::policy::needs_product_armed_migration(&raw, &policy) {
            policy = SeatPolicy::default().validated().map_err(ShieldError::InvalidParams)?;
            self.write_policy(&policy)?;
        }
        Ok(policy)
    }

    pub fn write_policy(&self, policy: &SeatPolicy) -> ShieldResult<()> {
        Self::write_json(&self.policy_path(), policy)
    }

    pub fn read_cycle(&self) -> ShieldResult<CycleState> {
        let raw = fs::read_to_string(self.cycle_path()).map_err(|source| ShieldError::Io {
            context: "cycle_read",
            source,
        })?;
        serde_json::from_str(&raw).map_err(|e| ShieldError::Parse {
            context: "cycle_parse",
            detail: e.to_string(),
        })
    }

    pub fn write_cycle(&self, state: &CycleState) -> ShieldResult<()> {
        Self::write_json(&self.cycle_path(), state)
    }

    pub fn read_events(&self) -> ShieldResult<Vec<ShieldEvent>> {
        let raw = fs::read_to_string(self.events_path()).map_err(|source| ShieldError::Io {
            context: "events_read",
            source,
        })?;
        let file: EventsFile = serde_json::from_str(&raw).map_err(|e| ShieldError::Parse {
            context: "events_parse",
            detail: e.to_string(),
        })?;
        Ok(file.items)
    }

    pub fn write_events(&self, items: &[ShieldEvent]) -> ShieldResult<()> {
        Self::write_json(
            &self.events_path(),
            &EventsFile {
                items: items.to_vec(),
            },
        )
    }

    pub fn append_events(&self, new_events: Vec<ShieldEvent>) -> ShieldResult<Vec<ShieldEvent>> {
        let mut cycle = self.read_cycle()?;
        let mut items = self.read_events()?;
        let mut stamped = Vec::with_capacity(new_events.len());
        for mut event in new_events {
            cycle.next_sequence = cycle.next_sequence.saturating_add(1);
            event.sequence = cycle.next_sequence;
            if event.epoch == 0 {
                event.epoch = now_epoch();
            }
            stamped.push(event.clone());
            items.push(event);
        }
        if items.len() > MAX_EVENTS {
            let drop_n = items.len() - MAX_EVENTS;
            items.drain(0..drop_n);
        }
        self.write_events(&items)?;
        // Browse copy — fail soft so ring persist still wins.
        if let Err(err) = append_journal_lines(&self.journal_path(), &stamped) {
            let _ = self.append_audit(
                "seat.shield.journal",
                "failed",
                &format!("append: {err}"),
            );
        }
        self.write_cycle(&cycle)?;
        Ok(stamped)
    }

    pub fn write_last_connections(&self, conns: &[ObservedConn]) -> ShieldResult<()> {
        let capped: Vec<&ObservedConn> = conns.iter().take(MAX_CONNECTIONS_WIRE).collect();
        let payload = LastConnectionsFile {
            epoch: now_epoch(),
            connection_count: conns.len(),
            truncated: conns.len() > MAX_CONNECTIONS_WIRE,
            connections: capped.into_iter().cloned().collect(),
        };
        Self::write_json(&self.last_connections_path(), &payload)
    }

    pub fn read_last_connections(&self) -> ShieldResult<LastConnectionsFile> {
        let path = self.last_connections_path();
        if !path.exists() {
            return Ok(LastConnectionsFile::default());
        }
        let raw = fs::read_to_string(&path).map_err(|source| ShieldError::Io {
            context: "last_connections_read",
            source,
        })?;
        serde_json::from_str(&raw).map_err(|e| ShieldError::Parse {
            context: "last_connections_parse",
            detail: e.to_string(),
        })
    }

    pub fn read_peer_window(&self) -> ShieldResult<PeerWindow> {
        let path = self.peer_window_path();
        if !path.exists() {
            return Ok(PeerWindow::default());
        }
        let raw = fs::read_to_string(&path).map_err(|source| ShieldError::Io {
            context: "peer_window_read",
            source,
        })?;
        serde_json::from_str(&raw).map_err(|e| ShieldError::Parse {
            context: "peer_window_parse",
            detail: e.to_string(),
        })
    }

    pub fn write_peer_window(&self, window: &PeerWindow) -> ShieldResult<()> {
        Self::write_json(&self.peer_window_path(), window)
    }

    pub fn clear_peer_window(&self) -> ShieldResult<()> {
        self.write_peer_window(&PeerWindow::default())
    }

    pub fn events_after(&self, after_sequence: u64, limit: usize) -> ShieldResult<Vec<ShieldEvent>> {
        let limit = limit.clamp(1, 200);
        let items = self.read_events()?;
        Ok(items
            .into_iter()
            .filter(|e| e.sequence > after_sequence)
            .take(limit)
            .collect())
    }

    pub fn read_audit(&self) -> ShieldResult<Vec<AuditEntry>> {
        let raw = fs::read_to_string(self.audit_path()).map_err(|source| ShieldError::Io {
            context: "audit_read",
            source,
        })?;
        let file: AuditFile = serde_json::from_str(&raw).map_err(|e| ShieldError::Parse {
            context: "audit_parse",
            detail: e.to_string(),
        })?;
        Ok(file.items)
    }

    pub fn write_audit(&self, items: &[AuditEntry]) -> ShieldResult<()> {
        Self::write_json(
            &self.audit_path(),
            &AuditFile {
                items: items.to_vec(),
            },
        )
    }

    pub fn append_audit(&self, op: &str, result: &str, detail: &str) -> ShieldResult<()> {
        let mut items = self.read_audit()?;
        items.push(AuditEntry {
            epoch: now_epoch(),
            op: op.into(),
            result: result.into(),
            detail: detail.into(),
        });
        if items.len() > MAX_AUDIT {
            let drop_n = items.len() - MAX_AUDIT;
            items.drain(0..drop_n);
        }
        self.write_audit(&items).map_err(|e| ShieldError::Audit(e.to_string()))
    }

    pub fn load_feed(&self) -> ShieldResult<Option<IocFeed>> {
        let path = self.feed_file_path();
        if !path.exists() {
            return Ok(None);
        }
        Ok(Some(IocFeed::load_file(&path)?))
    }

    pub fn install_feed_file(&self, source: &Path) -> ShieldResult<IocFeed> {
        let feed = IocFeed::load_file(source)?;
        let dest = self.feed_file_path();
        fs::copy(source, &dest).map_err(|source_err| ShieldError::Io {
            context: "feed_install",
            source: source_err,
        })?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&dest, fs::Permissions::from_mode(0o600));
        }
        Ok(feed)
    }

    pub fn feed_status(&self) -> ShieldResult<FeedStatus> {
        Ok(match self.load_feed()? {
            Some(feed) => feed.status(),
            None => FeedStatus::absent(),
        })
    }

    pub fn factory_reset_clean(&self) -> ShieldResult<()> {
        if self.root.exists() {
            fs::remove_dir_all(&self.root).map_err(|source| ShieldError::Io {
                context: "factory_reset",
                source,
            })?;
        }
        let _ = Self::open(&self.state_root)?;
        Ok(())
    }
}
