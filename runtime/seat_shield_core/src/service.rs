//! `seat.shield.*` dispatcher — prep-bay stand-in for Runtime wiring.

use std::net::IpAddr;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::act::{
    ActuatorBackend, FirewallActuator, HelperActuator, RecordingActuator, UnavailableActuator,
};
use crate::cycle::{
    is_allowlisted, is_loopback_or_unspecified, is_private_or_link_local, run_cycle_with_feed,
    ActDecision, ActKind,
};
use crate::error::{ShieldError, ShieldResult};
use crate::event::{Severity, ShieldEvent, SuppressionCache};
use crate::heuristics::{run_heuristics, PeerWindow};
use crate::journal::{parse_journal_query, query_journal};
use crate::observe::{
    deep_period_due, kernel_taint_event, read_host_tainted, ConnSource, ObservedConn, ProcConnSource,
    ProbeConnSource, SnapshotConnSource, PROBE_BACKEND,
};
use crate::policy::SeatPolicy;
use crate::protocol::{ShieldRequest, ShieldResponse, MAX_REQUEST_BYTES};
use crate::quarantine::{
    event_file_quarantined, event_file_restored, event_file_scan_hit, event_ioc_hash_from_scan,
    is_under_scan_root, scan_regular_file, ScanOutcome,
};
use crate::store::{ShieldStore, MAX_CONNECTIONS_WIRE};
use crate::timeutil::now_epoch;

const CONFIRM_BLOCK: &str = "BLOCK_PEER";
const CONFIRM_UNBLOCK: &str = "UNBLOCK_PEER";
const CONFIRM_ENABLE_ACTS: &str = "ENABLE_ACTS";
const CONFIRM_ENABLE_QUARANTINE: &str = "ENABLE_QUARANTINE";
const CONFIRM_QUARANTINE_FILE: &str = "QUARANTINE_FILE";
const CONFIRM_RESTORE_QUARANTINE: &str = "RESTORE_QUARANTINE";
const CONFIRM_RESET: &str = "RESET_SEAT_SHIELD";

enum ActuatorBox {
    Unavailable(UnavailableActuator),
    Recording(RecordingActuator),
    Helper(HelperActuator),
}

impl ActuatorBox {
    fn as_mut(&mut self) -> &mut dyn FirewallActuator {
        match self {
            Self::Unavailable(a) => a,
            Self::Recording(a) => a,
            Self::Helper(a) => a,
        }
    }

    fn as_ref(&self) -> &dyn FirewallActuator {
        match self {
            Self::Unavailable(a) => a,
            Self::Recording(a) => a,
            Self::Helper(a) => a,
        }
    }

    fn backend_kind(&self) -> ActuatorBackend {
        match self {
            Self::Unavailable(_) => ActuatorBackend::Unavailable,
            Self::Recording(_) => ActuatorBackend::Recording,
            Self::Helper(_) => ActuatorBackend::Helper,
        }
    }
}

/// Construction options for the prep-bay service.
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub state_root: PathBuf,
    pub actuator: ActuatorBackend,
    /// Required when `actuator == Helper`. Must be absolute.
    pub helper_path: Option<PathBuf>,
    /// Allowlisted helper argv prefix (e.g. `--backend nft --nft-bin /usr/sbin/nft`).
    pub helper_prefix: Vec<String>,
    /// When true, `seat.shield.observe` uses `/proc` (and optional probe). Tests may disable.
    pub enable_proc_observe: bool,
    /// Optional absolute path to `seat-shield-probe`. Missing/fail → `/proc`.
    pub probe_helper_path: Option<PathBuf>,
}

pub struct ShieldService {
    store: ShieldStore,
    actuator: ActuatorBox,
    helper_path: Option<PathBuf>,
    helper_prefix: Vec<String>,
    enable_proc_observe: bool,
    probe_source: Option<ProbeConnSource>,
    suppression: SuppressionCache,
}

impl ShieldService {
    pub fn open(config: ServiceConfig) -> ShieldResult<Self> {
        let store = ShieldStore::open(&config.state_root)?;
        let actuator = match config.actuator {
            ActuatorBackend::Unavailable => ActuatorBox::Unavailable(UnavailableActuator),
            ActuatorBackend::Recording => {
                ActuatorBox::Recording(RecordingActuator::open(store.root())?)
            }
            ActuatorBackend::Helper => {
                let helper = config.helper_path.clone().ok_or_else(|| {
                    ShieldError::InvalidParams(
                        "helper_path required when actuator=Helper".into(),
                    )
                })?;
                let state_dir = store.root().canonicalize().unwrap_or_else(|_| store.root().to_path_buf());
                let helper = if helper.is_absolute() {
                    helper
                } else {
                    return Err(ShieldError::Actuator(
                        "helper_path must be absolute".into(),
                    ));
                };
                ActuatorBox::Helper(HelperActuator::with_prefix(
                    helper,
                    state_dir,
                    config.helper_prefix.clone(),
                )?)
            }
        };
        let probe_source = match config.probe_helper_path {
            Some(path) => Some(ProbeConnSource::new(path)?),
            None => None,
        };
        Ok(Self {
            store,
            actuator,
            helper_path: config.helper_path,
            helper_prefix: config.helper_prefix,
            enable_proc_observe: config.enable_proc_observe,
            probe_source,
            suppression: SuppressionCache::new(120),
        })
    }

    #[must_use]
    pub fn store(&self) -> &ShieldStore {
        &self.store
    }

    pub fn handle_bytes(&mut self, bytes: &[u8]) -> ShieldResponse {
        if bytes.is_empty() {
            return ShieldResponse::from_error("", &ShieldError::InvalidRequest("empty_request"));
        }
        if bytes.len() > MAX_REQUEST_BYTES {
            return ShieldResponse::from_error(
                "",
                &ShieldError::InvalidRequest("request_too_large"),
            );
        }
        let request = match serde_json::from_slice::<ShieldRequest>(bytes) {
            Ok(req) => req,
            Err(error) => {
                return ShieldResponse::from_error(
                    "",
                    &ShieldError::InvalidParams(format!("invalid_json: {error}")),
                );
            }
        };
        if let Err(err) = request.validate() {
            return ShieldResponse::from_error(&request.id, &err);
        }
        match self.dispatch(request.id.clone(), &request.op, &request.params) {
            Ok(resp) => resp,
            Err(err) => ShieldResponse::from_error(request.id, &err),
        }
    }

    fn dispatch(
        &mut self,
        id: String,
        op: &str,
        params: &Value,
    ) -> ShieldResult<ShieldResponse> {
        match op {
            "seat.shield.status" => {
                // HUD ticks status, not observe. Interval lives on disk because
                // Runtime opens a fresh ShieldService per IPC line.
                let _ = self.drain_watch_persist()?;
                let _ = self.maybe_auto_observe()?;
                Ok(ShieldResponse::success(id, "observed", self.status_json()?))
            }
            "seat.shield.policy" => Ok(ShieldResponse::success(id, "observed", self.policy_json()?)),
            "seat.shield.policy.set" => self.op_policy_set(id, params),
            "seat.shield.events" => self.op_events(id, params),
            "seat.shield.journal" => self.op_journal(id, params),
            "seat.shield.cycle" => self.op_cycle(id, params),
            "seat.shield.observe" => self.op_observe(id),
            "seat.shield.feed_status" => {
                let status = self.store.feed_status()?;
                Ok(ShieldResponse::success(id, "observed", serde_json::to_value(status)?))
            }
            "seat.shield.feed.install" => self.op_feed_install(id, params),
            "seat.shield.act" => self.op_act(id, params),
            "seat.shield.reset" => self.op_reset(id, params),
            "seat.shield.scan_path" => self.op_scan_path(id, params),
            "seat.shield.quarantine" => self.op_quarantine(id, params),
            "seat.shield.quarantine.list" => self.op_quarantine_list(id),
            "seat.shield.quarantine.restore" => self.op_quarantine_restore(id, params),
            other => Err(ShieldError::OperationNotFound(other.to_string())),
        }
    }

    fn status_json(&self) -> ShieldResult<Value> {
        let policy = self.store.read_policy()?;
        let cycle = self.store.read_cycle()?;
        let events = self.store.read_events()?;
        let open_high = events
            .iter()
            .filter(|e| e.severity.is_high_or_above())
            .count();
        let backend = self.actuator.as_ref().backend_name();
        let mode = if !self.actuator.as_ref().is_available() {
            if policy.act_enabled {
                "unavailable"
            } else {
                "observe_only"
            }
        } else if policy.act_enabled {
            "armed"
        } else {
            "observe_only"
        };
        let detail = match mode {
            "armed" => format!("armed via actuator `{backend}`"),
            "unavailable" => {
                "act_enabled set but actuator unavailable — fail-closed".into()
            }
            _ => "observe-only; destructive acts not_available".into(),
        };
        let blocked_peers_all = self.actuator.as_ref().list_blocked()?;
        let blocked_count = blocked_peers_all.len();
        const BLOCKED_PEERS_STATUS_CAP: usize = 32;
        let blocked_peers: Vec<String> = blocked_peers_all
            .into_iter()
            .take(BLOCKED_PEERS_STATUS_CAP)
            .collect();
        let last_conns = self.store.read_last_connections()?;
        let window = if policy.heuristics_enabled {
            self.store.read_peer_window()?
        } else {
            PeerWindow::default()
        };
        let teasers = if policy.heuristics_enabled {
            window.chain_teasers(5)
        } else {
            Vec::new()
        };
        let connections = enrich_connections_for_wire(
            &last_conns.connections,
            &window,
            policy.heuristics_enabled,
        );
        let ota_baseline = crate::ota_observe::observe_ota_baseline(self.store.state_root()).as_str();
        Ok(json!({
            "product": "empiieros-seat-shield",
            "mode": mode,
            "act_enabled": policy.act_enabled,
            "auto_act_protected": policy.auto_act_protected,
            "auto_quarantine_hash": policy.auto_quarantine_hash,
            "heuristics_enabled": policy.heuristics_enabled,
            "open_chains": if policy.heuristics_enabled {
                window.open_chains_count()
            } else {
                0
            },
            "open_chain_teasers": teasers,
            "quarantine_count": self.store.quarantine_count()?,
            "watch_pending": self.store.watch_pending()?,
            "last_watch_drained": cycle.last_watch_drained,
            "cycle_interval_secs": policy.cycle_interval_secs,
            "protected_ports": policy.protected_ports,
            "allowlist_count": policy.allowlist.len(),
            "integrity_twin": "security.integrity",
            "integrity_twin_state": "disabled",
            "ota_baseline": ota_baseline,
            "last_cycle_epoch": cycle.last_cycle_epoch,
            "last_observe_backend": cycle.last_observe_backend,
            "open_high_events": open_high,
            "backend": backend,
            "blocked_count": blocked_count,
            "blocked_peers": blocked_peers,
            "connection_count": last_conns.connection_count,
            "connections": connections,
            "connections_truncated": last_conns.truncated,
            "connections_epoch": last_conns.epoch,
            "proc_observe": self.enable_proc_observe,
            "detail": detail,
        }))
    }

    fn policy_json(&self) -> ShieldResult<Value> {
        let policy = self.store.read_policy()?;
        Ok(json!({
            "act_enabled": policy.act_enabled,
            "auto_act_protected": policy.auto_act_protected,
            "auto_quarantine_hash": policy.auto_quarantine_hash,
            "heuristics_enabled": policy.heuristics_enabled,
            "max_auto_acts_per_cycle": policy.max_auto_acts_per_cycle,
            "cycle_interval_secs": policy.cycle_interval_secs,
            "scanner_port_threshold": policy.scanner_port_threshold,
            "scanner_skip_prefix_count": policy.scanner_skip_ip_prefixes.len(),
            "protected_ports": policy.protected_ports,
            "allow_private_on_protected": policy.allow_private_on_protected,
            "suppression_ttl_secs": policy.suppression_ttl_secs,
            "allowlist_count": policy.allowlist.len(),
            "allowlist": policy.allowlist,
        }))
    }

    fn op_policy_set(&mut self, id: String, params: &Value) -> ShieldResult<ShieldResponse> {
        self.store
            .append_audit("seat.shield.policy.set", "requested", "policy mutation")?;
        let mut policy = self.store.read_policy()?;
        let enabling_acts = params
            .get("act_enabled")
            .and_then(Value::as_bool)
            .is_some_and(|v| v && !policy.act_enabled);
        if enabling_acts
            && params.get("confirmation").and_then(Value::as_str) != Some(CONFIRM_ENABLE_ACTS)
        {
            return Err(ShieldError::MissingConfirmation {
                expected: CONFIRM_ENABLE_ACTS,
            });
        }
        let acts_after = params
            .get("act_enabled")
            .and_then(Value::as_bool)
            .unwrap_or(policy.act_enabled);
        let wants_auto = params.get("auto_act_protected").and_then(Value::as_bool) == Some(true);
        if wants_auto && !acts_after {
            return Err(ShieldError::ActDenied(
                "auto_act_protected requires act_enabled (ENABLE_ACTS)".into(),
            ));
        }
        let enabling_auto_q = params
            .get("auto_quarantine_hash")
            .and_then(Value::as_bool)
            .is_some_and(|v| v && !policy.auto_quarantine_hash);
        if enabling_auto_q && !acts_after {
            return Err(ShieldError::ActDenied(
                "auto_quarantine_hash requires act_enabled (ENABLE_ACTS)".into(),
            ));
        }
        if enabling_auto_q
            && params.get("confirmation").and_then(Value::as_str) != Some(CONFIRM_ENABLE_QUARANTINE)
        {
            return Err(ShieldError::MissingConfirmation {
                expected: CONFIRM_ENABLE_QUARANTINE,
            });
        }
        if let Some(v) = params.get("act_enabled").and_then(Value::as_bool) {
            policy.act_enabled = v;
            if !v {
                policy.auto_act_protected = false;
                policy.auto_quarantine_hash = false;
            }
        }
        if let Some(v) = params.get("auto_act_protected").and_then(Value::as_bool) {
            policy.auto_act_protected = v;
        }
        if let Some(v) = params.get("auto_quarantine_hash").and_then(Value::as_bool) {
            policy.auto_quarantine_hash = v;
        }
        if let Some(v) = params.get("heuristics_enabled").and_then(Value::as_bool) {
            policy.heuristics_enabled = v;
        }
        if let Some(v) = params
            .get("allow_private_on_protected")
            .and_then(Value::as_bool)
        {
            policy.allow_private_on_protected = v;
        }
        if let Some(v) = params.get("suppression_ttl_secs").and_then(Value::as_u64) {
            policy.suppression_ttl_secs = v;
        }
        if let Some(v) = params.get("cycle_interval_secs").and_then(Value::as_u64) {
            policy.cycle_interval_secs = v;
        }
        if let Some(v) = params.get("max_auto_acts_per_cycle").and_then(Value::as_u64) {
            policy.max_auto_acts_per_cycle = u32::try_from(v).map_err(|_| {
                ShieldError::InvalidParams("max_auto_acts_per_cycle out of range".into())
            })?;
        }
        if let Some(v) = params.get("scanner_port_threshold").and_then(Value::as_u64) {
            policy.scanner_port_threshold = u32::try_from(v).map_err(|_| {
                ShieldError::InvalidParams("scanner_port_threshold out of range".into())
            })?;
        }
        if let Some(arr) = params.get("protected_ports").and_then(Value::as_array) {
            let mut ports = Vec::new();
            for item in arr {
                let p = item.as_u64().ok_or_else(|| {
                    ShieldError::InvalidParams("protected_ports entries must be u16".into())
                })?;
                ports.push(u16::try_from(p).map_err(|_| {
                    ShieldError::InvalidParams("protected_ports out of range".into())
                })?);
            }
            policy.protected_ports = ports;
        }
        if let Some(arr) = params.get("allowlist").and_then(Value::as_array) {
            let mut list = Vec::new();
            for item in arr {
                let ip = item.as_str().ok_or_else(|| {
                    ShieldError::InvalidParams("allowlist entries must be strings".into())
                })?;
                list.push(ip.to_string());
            }
            policy.allowlist = list;
        }
        if let Some(arr) = params
            .get("scanner_skip_ip_prefixes")
            .and_then(Value::as_array)
        {
            let mut prefixes = Vec::new();
            for item in arr {
                let prefix = item.as_str().ok_or_else(|| {
                    ShieldError::InvalidParams(
                        "scanner_skip_ip_prefixes entries must be strings".into(),
                    )
                })?;
                prefixes.push(prefix.to_string());
            }
            policy.scanner_skip_ip_prefixes = prefixes;
        }
        let policy = policy.validated().map_err(ShieldError::InvalidParams)?;
        self.store.write_policy(&policy)?;
        if !policy.heuristics_enabled {
            let _ = self.store.clear_peer_window();
        }
        self.suppression.set_ttl(policy.suppression_ttl_secs);
        self.store
            .append_audit("seat.shield.policy.set", "applied", "policy updated")?;
        Ok(ShieldResponse::success(
            id,
            "applied",
            json!({
                "act_enabled": policy.act_enabled,
                "auto_act_protected": policy.auto_act_protected,
                "auto_quarantine_hash": policy.auto_quarantine_hash,
                "heuristics_enabled": policy.heuristics_enabled,
                "protected_ports": policy.protected_ports,
                "allowlist_count": policy.allowlist.len(),
            }),
        ))
    }

    fn op_events(&self, id: String, params: &Value) -> ShieldResult<ShieldResponse> {
        let after = params
            .get("after_sequence")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let limit = params
            .get("limit")
            .and_then(Value::as_u64)
            .unwrap_or(50) as usize;
        let items = self.store.events_after(after, limit)?;
        Ok(ShieldResponse::success(
            id,
            "observed",
            json!({
                "items": items,
                "after_sequence": after,
                "count": items.len(),
            }),
        ))
    }

    fn op_journal(&self, id: String, params: &Value) -> ShieldResult<ShieldResponse> {
        let query = parse_journal_query(params)?;
        let result = query_journal(&self.store.journal_path(), &query)?;
        let groups: Vec<Value> = result
            .groups
            .iter()
            .map(|g| {
                json!({
                    "key": g.key,
                    "count": g.count,
                    "latest_sequence": g.latest_sequence,
                })
            })
            .collect();
        let analytics = result.analytics.as_ref().map(|a| {
            json!({
                "by_severity": {
                    "critical": a.by_severity.critical,
                    "high": a.by_severity.high,
                    "medium": a.by_severity.medium,
                    "low": a.by_severity.low,
                },
                "by_kind_top": a.by_kind_top.iter().map(|k| json!({
                    "kind": k.kind,
                    "count": k.count,
                })).collect::<Vec<_>>(),
                "top_ips": a.top_ips.iter().map(|i| json!({
                    "ip": i.ip,
                    "count": i.count,
                })).collect::<Vec<_>>(),
                "window_from_epoch": a.window_from_epoch,
                "window_to_epoch": a.window_to_epoch,
            })
        });
        Ok(ShieldResponse::success(
            id,
            "observed",
            json!({
                "items": result.items,
                "truncated": result.truncated,
                "journal_lines": result.journal_lines,
                "groups": groups,
                "analytics": analytics,
            }),
        ))
    }

    fn op_cycle(&mut self, id: String, params: &Value) -> ShieldResult<ShieldResponse> {
        let conns = parse_connections(params)?;
        self.run_cycle_persist(id, conns, "snapshot", false)
    }

    fn op_observe(&mut self, id: String) -> ShieldResult<ShieldResponse> {
        if !self.enable_proc_observe {
            return Err(ShieldError::NotAvailable(
                "proc observe disabled in this service config".into(),
            ));
        }
        let feed = self.store.load_feed()?;
        let hash_any = feed.as_ref().is_some_and(|feed| !feed.hashes.is_empty());
        let (conns, backend) = self.collect_live_connections(hash_any)?;
        self.run_cycle_persist(id, conns, backend, true)
    }

    fn collect_live_connections(
        &mut self,
        hash_any: bool,
    ) -> ShieldResult<(Vec<ObservedConn>, &'static str)> {
        if let Some(probe) = self.probe_source.as_mut() {
            match probe.collect_fresh() {
                Ok(mut conns) => {
                    // Probe lacks exe hashes; fill from /proc when feed needs them.
                    if hash_any {
                        let proc_rows = ProcConnSource::default().collect_with_hash(true)?;
                        merge_exe_hashes(&mut conns, &proc_rows);
                    }
                    return Ok((conns, PROBE_BACKEND));
                }
                Err(_) => {
                    // Fail → /proc (honest fallback).
                }
            }
        }
        let source = ProcConnSource::default();
        let conns = source.collect_with_hash(hash_any)?;
        Ok((conns, source.backend_name()))
    }

    /// Run the same path as `seat.shield.observe` when `cycle_interval_secs` has elapsed.
    pub fn maybe_auto_observe(&mut self) -> ShieldResult<bool> {
        if !self.enable_proc_observe {
            return Ok(false);
        }
        let policy = self.store.read_policy()?;
        let cycle = self.store.read_cycle()?;
        let interval = policy.cycle_interval_secs.clamp(10, 300);
        if !interval_elapsed(cycle.last_cycle_epoch, now_epoch(), interval) {
            return Ok(false);
        }
        let id = format!("auto-{}", now_epoch());
        let _ = self.op_observe(id)?;
        Ok(true)
    }

    /// Harness `--auto-cycle` writes the clamped interval into policy.
    pub fn configure_cycle_interval(&mut self, secs: u64) -> ShieldResult<()> {
        let mut policy = self.store.read_policy()?;
        policy.cycle_interval_secs = secs;
        let policy = policy.validated().map_err(ShieldError::InvalidParams)?;
        self.store.write_policy(&policy)
    }

    fn run_cycle_persist(
        &mut self,
        id: String,
        conns: Vec<ObservedConn>,
        backend: &str,
        live_observe: bool,
    ) -> ShieldResult<ShieldResponse> {
        self.store.append_audit(
            "seat.shield.cycle",
            "requested",
            &format!("backend={backend} conns={}", conns.len()),
        )?;
        let policy = self.store.read_policy()?;
        let feed = self.store.load_feed()?;
        let outcome = run_cycle_with_feed(&policy, &conns, feed.as_ref());
        self.suppression.set_ttl(policy.suppression_ttl_secs);
        let (events, extra_suppressed) = self.suppression.filter(outcome.events);
        let suppressed = outcome.suppressed_duplicates.saturating_add(extra_suppressed);
        let (applied, denied, mut act_events) =
            self.apply_protected_auto_acts(&policy, &outcome.acts)?;
        let now = now_epoch();
        let mut kernel_checked = false;
        if live_observe {
            let prior = self.store.read_cycle()?;
            if deep_period_due(prior.last_kernel_taint_epoch, now) {
                kernel_checked = true;
                if let Some(value) = read_host_tainted() {
                    if let Some(event) = kernel_taint_event(value) {
                        act_events.push(event);
                    }
                }
            }
        }
        let mut all_events = events;
        all_events.append(&mut act_events);
        let (mut watch_events, watch_n) = self.drain_watch_scan(&policy)?;
        all_events.append(&mut watch_events);
        let mut q_events = self.maybe_auto_quarantine(&policy, &conns, &all_events)?;
        all_events.append(&mut q_events);
        let mut window = if policy.heuristics_enabled {
            self.store.read_peer_window()?
        } else {
            PeerWindow::default()
        };
        let mut heur_events = run_heuristics(&policy, &conns, &mut all_events, &mut window);
        all_events.append(&mut heur_events);
        // OTA baseline drift — observe-only; never quarantine /usr.
        if crate::ota_observe::observe_ota_baseline(self.store.state_root())
            == crate::ota_observe::OtaBaselineStatus::Mismatch
        {
            all_events.push(crate::ota_observe::ota_component_mismatch_event("rematch"));
        }
        if policy.heuristics_enabled {
            self.store.write_peer_window(&window)?;
        } else {
            let _ = self.store.clear_peer_window();
        }
        let stamped = self.store.append_events(all_events)?;
        let _ = self.store.write_last_connections(&conns);
        let mut cycle = self.store.read_cycle()?;
        cycle.last_cycle_epoch = now;
        cycle.last_event_count = stamped.len();
        cycle.last_act_count = outcome.acts.len();
        cycle.last_suppressed = suppressed;
        cycle.last_observe_backend = backend.into();
        if kernel_checked {
            cycle.last_kernel_taint_epoch = now;
        }
        if watch_n > 0 {
            cycle.last_watch_drained = now;
        }
        self.store.write_cycle(&cycle)?;
        self.store.append_audit(
            "seat.shield.cycle",
            "applied",
            &format!(
                "persisted_events={} applied_auto={}",
                stamped.len(),
                applied.len()
            ),
        )?;
        let proposed: Vec<Value> = outcome
            .acts
            .iter()
            .map(|act| {
                json!({
                    "act": act.act,
                    "peer_ip": act.peer_ip,
                    "reason": act.reason,
                    "allow_private": act.allow_private,
                })
            })
            .collect();
        let status = if applied.is_empty() { "observed" } else { "applied" };
        let connections_wire = enrich_connections_for_wire(&conns, &window, policy.heuristics_enabled);
        Ok(ShieldResponse::success(
            id,
            status,
            json!({
                "observe_backend": backend,
                "connection_count": conns.len(),
                "connections": connections_wire,
                "connections_truncated": conns.len() > MAX_CONNECTIONS_WIRE,
                "events": stamped,
                "proposed_acts": proposed,
                "applied_acts": applied,
                "suppressed_duplicates": suppressed,
                "note": cycle_note(applied.len(), denied),
            }),
        ))
    }

    fn apply_protected_auto_acts(
        &mut self,
        policy: &SeatPolicy,
        acts: &[ActDecision],
    ) -> ShieldResult<(Vec<Value>, usize, Vec<ShieldEvent>)> {
        let mut applied = Vec::new();
        let mut events = Vec::new();
        let mut denied = 0_usize;
        let armed = policy.act_enabled
            && policy.auto_act_protected
            && self.actuator.as_ref().is_available();
        if !armed {
            return Ok((applied, denied, events));
        }
        let cap = usize::try_from(policy.max_auto_acts_per_cycle).unwrap_or(0);
        let mut blocked = self.actuator.as_ref().list_blocked()?;
        for decision in acts {
            if applied.len() >= cap {
                break;
            }
            if decision.act != ActKind::BlockPeer {
                continue;
            }
            if !peer_eligible_for_auto_act(policy, &decision.peer_ip) {
                continue;
            }
            if blocked.iter().any(|peer| peer == &decision.peer_ip) {
                continue;
            }
            match self
                .actuator
                .as_mut()
                .block_peer(&decision.peer_ip, "auto_protected")
            {
                Ok(()) => {
                    self.store.append_audit(
                        "seat.shield.act",
                        "applied",
                        &format!("auto_protected {}", decision.peer_ip),
                    )?;
                    events.push(event_act_applied(&decision.peer_ip));
                    blocked.push(decision.peer_ip.clone());
                    applied.push(json!({
                        "act": "block_peer",
                        "peer_ip": decision.peer_ip,
                        "reason": "auto_protected",
                    }));
                }
                Err(err) => {
                    denied += 1;
                    self.store.append_audit(
                        "seat.shield.act",
                        "denied",
                        &format!("auto_protected {} {err}", decision.peer_ip),
                    )?;
                    events.push(event_act_denied(&decision.peer_ip));
                    break;
                }
            }
        }
        Ok((applied, denied, events))
    }

    fn op_feed_install(&mut self, id: String, params: &Value) -> ShieldResult<ShieldResponse> {
        let path = params
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| ShieldError::InvalidParams("params.path required".into()))?;
        let path = Path::new(path);
        if !path.is_absolute() {
            return Err(ShieldError::InvalidParams(
                "feed path must be absolute".into(),
            ));
        }
        // Allow only under state_root staging or seat_shield/feed staging.
        let allowed_prefix = self.store.state_root();
        let canon = path.canonicalize().map_err(|source| ShieldError::Io {
            context: "feed_path_canonicalize",
            source,
        })?;
        if !canon.starts_with(allowed_prefix) {
            return Err(ShieldError::ActDenied(
                "feed path must stay under service state_root".into(),
            ));
        }
        self.store
            .append_audit("seat.shield.feed.install", "requested", "install feed")?;
        let feed = self.store.install_feed_file(&canon)?;
        self.store
            .append_audit("seat.shield.feed.install", "applied", &feed.version)?;
        Ok(ShieldResponse::success(
            id,
            "applied",
            json!({
                "version": feed.version,
                "digest": feed.digest,
                "ip_count": feed.ips.len(),
                "hash_count": feed.hashes.len(),
            }),
        ))
    }

    fn op_act(&mut self, id: String, params: &Value) -> ShieldResult<ShieldResponse> {
        let action = params
            .get("action")
            .and_then(Value::as_str)
            .unwrap_or_default();
        match action {
            "preview_block" => self.act_preview(id, params),
            "block_peer" => self.act_block(id, params),
            "unblock_peer" => self.act_unblock(id, params),
            "" => Err(ShieldError::InvalidParams(
                "params.action is required".into(),
            )),
            other => Err(ShieldError::UnallowlistedAction(other.into())),
        }
    }

    fn act_preview(&mut self, id: String, params: &Value) -> ShieldResult<ShieldResponse> {
        let peer = require_peer_ip(params)?;
        let policy = self.store.read_policy()?;
        if policy.allowlist.iter().any(|x| x == &peer) {
            self.store
                .append_audit("seat.shield.act", "denied", "allowlisted peer")?;
            return Err(ShieldError::ActDenied("peer is on seat allowlist".into()));
        }
        self.store
            .append_audit("seat.shield.act", "applied", &format!("preview_block {peer}"))?;
        Ok(ShieldResponse::success(
            id,
            "observed",
            json!({
                "action": "preview_block",
                "peer_ip": peer,
                "would_block": true,
            }),
        ))
    }

    fn act_block(&mut self, id: String, params: &Value) -> ShieldResult<ShieldResponse> {
        self.require_armed()?;
        if params.get("confirmation").and_then(Value::as_str) != Some(CONFIRM_BLOCK) {
            return Err(ShieldError::MissingConfirmation {
                expected: CONFIRM_BLOCK,
            });
        }
        let peer = require_peer_ip(params)?;
        let policy = self.store.read_policy()?;
        if policy.allowlist.iter().any(|x| x == &peer) {
            return Err(ShieldError::ActDenied("peer is on seat allowlist".into()));
        }
        if is_loopback_str(&peer) {
            return Err(ShieldError::ActDenied("loopback cannot be blocked".into()));
        }
        self.store
            .append_audit("seat.shield.act", "requested", &format!("block_peer {peer}"))?;
        self.actuator
            .as_mut()
            .block_peer(&peer, "seat.shield.act")?;
        self.store
            .append_audit("seat.shield.act", "applied", &format!("block_peer {peer}"))?;
        Ok(ShieldResponse::success(
            id,
            "applied",
            json!({
                "action": "block_peer",
                "peer_ip": peer,
                "backend": self.actuator.as_ref().backend_name(),
            }),
        ))
    }

    fn act_unblock(&mut self, id: String, params: &Value) -> ShieldResult<ShieldResponse> {
        self.require_armed()?;
        if params.get("confirmation").and_then(Value::as_str) != Some(CONFIRM_UNBLOCK) {
            return Err(ShieldError::MissingConfirmation {
                expected: CONFIRM_UNBLOCK,
            });
        }
        let peer = require_peer_ip(params)?;
        self.store
            .append_audit("seat.shield.act", "requested", &format!("unblock_peer {peer}"))?;
        let removed = self.actuator.as_mut().unblock_peer(&peer)?;
        self.store.append_audit(
            "seat.shield.act",
            if removed { "applied" } else { "noop" },
            &format!("unblock_peer {peer}"),
        )?;
        Ok(ShieldResponse::success(
            id,
            "applied",
            json!({
                "action": "unblock_peer",
                "peer_ip": peer,
                "removed": removed,
                "backend": self.actuator.as_ref().backend_name(),
            }),
        ))
    }

    fn op_reset(&mut self, id: String, params: &Value) -> ShieldResult<ShieldResponse> {
        if params.get("confirmation").and_then(Value::as_str) != Some(CONFIRM_RESET) {
            return Err(ShieldError::MissingConfirmation {
                expected: CONFIRM_RESET,
            });
        }
        let state_root = self.store.state_root().to_path_buf();
        let backend = self.actuator.backend_kind();
        let helper_path = self.helper_path.clone();
        let helper_prefix = self.helper_prefix.clone();
        let enable_proc_observe = self.enable_proc_observe;
        let probe_helper_path = self
            .probe_source
            .as_ref()
            .map(|p| p.helper_path().to_path_buf());
        self.store.factory_reset_clean()?;
        *self = Self::open(ServiceConfig {
            state_root,
            actuator: backend,
            helper_path,
            helper_prefix,
            enable_proc_observe,
            probe_helper_path,
        })?;
        Ok(ShieldResponse::success(
            id,
            "applied",
            json!({"reset": true}),
        ))
    }

    fn op_scan_path(&mut self, id: String, params: &Value) -> ShieldResult<ShieldResponse> {
        let path = params
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| ShieldError::InvalidParams("params.path required".into()))?;
        self.store
            .append_audit("seat.shield.scan_path", "requested", path)?;
        let feed = self.store.load_feed()?;
        let hashes = feed
            .as_ref()
            .map(|f| f.hashes.clone())
            .unwrap_or_default();
        let outcome = scan_regular_file(path, &hashes)?;
        let stamped = self.store.append_events(outcome.events)?;
        if outcome.outside_or_illegal {
            self.store
                .append_audit("seat.shield.scan_path", "denied", path)?;
            return Err(ShieldError::ActDenied(
                "scan path denied (outside roots, symlink escape, or not a regular file)".into(),
            ));
        }
        self.store
            .append_audit("seat.shield.scan_path", "applied", path)?;
        Ok(ShieldResponse::success(
            id,
            "observed",
            json!({
                "basename": outcome.path_basename,
                "sha256": outcome.sha256,
                "size": outcome.size,
                "skipped": outcome.skipped,
                "feed_hit": outcome.feed_hit,
                "events": stamped,
            }),
        ))
    }

    fn op_quarantine(&mut self, id: String, params: &Value) -> ShieldResult<ShieldResponse> {
        if params.get("confirmation").and_then(Value::as_str) != Some(CONFIRM_QUARANTINE_FILE) {
            return Err(ShieldError::MissingConfirmation {
                expected: CONFIRM_QUARANTINE_FILE,
            });
        }
        let path = params
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| ShieldError::InvalidParams("params.path required".into()))?;
        self.store
            .append_audit("seat.shield.quarantine", "requested", path)?;
        let side = self.store.hold_file(path, "operator")?;
        let basename = Path::new(&side.orig_path)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "unknown".into());
        let stamped = self.store.append_events(vec![event_file_quarantined(
            &basename,
            &side.sha256,
            "operator",
        )])?;
        self.store
            .append_audit("seat.shield.quarantine", "applied", &side.sha256)?;
        Ok(ShieldResponse::success(
            id,
            "applied",
            json!({
                "sha256": side.sha256,
                "size": side.size,
                "basename": basename,
                "events": stamped,
            }),
        ))
    }

    fn op_quarantine_list(&self, id: String) -> ShieldResult<ShieldResponse> {
        let items = self.store.list_quarantine()?;
        Ok(ShieldResponse::success(
            id,
            "observed",
            json!({
                "items": items,
                "count": items.len(),
            }),
        ))
    }

    fn op_quarantine_restore(&mut self, id: String, params: &Value) -> ShieldResult<ShieldResponse> {
        if params.get("confirmation").and_then(Value::as_str) != Some(CONFIRM_RESTORE_QUARANTINE)
        {
            return Err(ShieldError::MissingConfirmation {
                expected: CONFIRM_RESTORE_QUARANTINE,
            });
        }
        let sha256 = params
            .get("sha256")
            .and_then(Value::as_str)
            .ok_or_else(|| ShieldError::InvalidParams("params.sha256 required".into()))?;
        self.store
            .append_audit("seat.shield.quarantine.restore", "requested", sha256)?;
        let side = self.store.restore_held(sha256)?;
        let basename = Path::new(&side.orig_path)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "unknown".into());
        let stamped = self
            .store
            .append_events(vec![event_file_restored(&basename)])?;
        self.store
            .append_audit("seat.shield.quarantine.restore", "applied", sha256)?;
        Ok(ShieldResponse::success(
            id,
            "applied",
            json!({
                "sha256": side.sha256,
                "basename": basename,
                "events": stamped,
            }),
        ))
    }

    fn maybe_auto_quarantine(
        &mut self,
        policy: &SeatPolicy,
        conns: &[ObservedConn],
        events: &[ShieldEvent],
    ) -> ShieldResult<Vec<ShieldEvent>> {
        if !policy.auto_quarantine_hash || !policy.act_enabled {
            return Ok(Vec::new());
        }
        if !events.iter().any(|e| e.kind == "ioc_hash_match") {
            return Ok(Vec::new());
        }
        let feed = self.store.load_feed()?;
        let Some(feed) = feed else {
            return Ok(Vec::new());
        };
        for conn in conns {
            let Some(path) = conn.exe_path.as_deref() else {
                continue;
            };
            if !is_under_scan_root(Path::new(path)) {
                continue;
            }
            let Some(digest) = conn.exe_sha256.as_deref() else {
                continue;
            };
            if !feed
                .hashes
                .iter()
                .any(|h| h.eq_ignore_ascii_case(digest))
            {
                continue;
            }
            match self.store.hold_file(path, "auto_ioc_hash_match") {
                Ok(side) => {
                    let basename = Path::new(&side.orig_path)
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "unknown".into());
                    self.store.append_audit(
                        "seat.shield.quarantine",
                        "applied",
                        "auto_ioc_hash_match",
                    )?;
                    return Ok(vec![event_file_quarantined(
                        &basename,
                        &side.sha256,
                        "auto_ioc_hash_match",
                    )]);
                }
                Err(err) => {
                    self.store.append_audit(
                        "seat.shield.quarantine",
                        "denied",
                        &err.to_string(),
                    )?;
                    return Ok(Vec::new());
                }
            }
        }
        Ok(Vec::new())
    }

    fn drain_watch_scan(&mut self, policy: &SeatPolicy) -> ShieldResult<(Vec<ShieldEvent>, usize)> {
        let records = self.store.drain_watch()?;
        if records.is_empty() {
            return Ok((Vec::new(), 0));
        }
        let drained = records.len();
        let feed = self.store.load_feed()?;
        let hashes = feed
            .as_ref()
            .map(|f| f.hashes.clone())
            .unwrap_or_default();
        let mut events = Vec::new();
        let mut auto_held = false;
        for rec in records {
            let mut outcome = scan_regular_file(&rec.path, &hashes)?;
            // Runtime PrivateTmp / unlinked tmp: use helper hash when we could not open.
            if outcome.sha256.is_none() {
                if let Some(digest) = crate::watch::usable_journal_hash(&rec) {
                    let feed_hit = hashes.iter().any(|h| h.eq_ignore_ascii_case(&digest));
                    let basename = Path::new(&rec.path)
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "unknown".into());
                    let mut assisted = Vec::new();
                    if feed_hit {
                        assisted.push(event_file_scan_hit(&basename, &digest));
                        assisted.push(event_ioc_hash_from_scan(&basename, &digest));
                    }
                    outcome = ScanOutcome {
                        path_basename: basename,
                        sha256: Some(digest),
                        size: 0,
                        skipped: false,
                        outside_or_illegal: false,
                        feed_hit,
                        events: assisted,
                    };
                }
            }
            // Vanished tmp is skip. Persist deny only for planted outside-root paths.
            let persist_denied = outcome.outside_or_illegal
                && crate::watch::watch_path_allowed(&rec.path).is_err();
            if persist_denied || outcome.feed_hit {
                events.extend(outcome.events);
            }
            if outcome.feed_hit
                && !auto_held
                && policy.auto_quarantine_hash
                && policy.act_enabled
            {
                match self.store.hold_file(&rec.path, "auto_watch_ioc_hash") {
                    Ok(side) => {
                        let basename = Path::new(&side.orig_path)
                            .file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_else(|| "unknown".into());
                        self.store.append_audit(
                            "seat.shield.quarantine",
                            "applied",
                            "auto_watch_ioc_hash",
                        )?;
                        events.push(event_file_quarantined(
                            &basename,
                            &side.sha256,
                            "auto_watch_ioc_hash",
                        ));
                        auto_held = true;
                    }
                    Err(err) => {
                        self.store.append_audit(
                            "seat.shield.quarantine",
                            "denied",
                            &err.to_string(),
                        )?;
                    }
                }
            }
        }
        Ok((events, drained))
    }

    fn drain_watch_persist(&mut self) -> ShieldResult<usize> {
        let policy = self.store.read_policy()?;
        let (events, n) = self.drain_watch_scan(&policy)?;
        if !events.is_empty() {
            self.store.append_events(events)?;
        }
        if n > 0 {
            let mut cycle = self.store.read_cycle()?;
            cycle.last_watch_drained = now_epoch();
            self.store.write_cycle(&cycle)?;
        }
        Ok(n)
    }

    fn require_armed(&self) -> ShieldResult<()> {
        let policy = self.store.read_policy()?;
        if !policy.act_enabled {
            return Err(ShieldError::NotAvailable(
                "destructive acts disabled (observe_only)".into(),
            ));
        }
        if !self.actuator.as_ref().is_available() {
            return Err(ShieldError::NotAvailable(
                "floor firewall helper is not available".into(),
            ));
        }
        Ok(())
    }
}

fn parse_connections(params: &Value) -> ShieldResult<Vec<ObservedConn>> {
    let Some(arr) = params.get("connections").and_then(Value::as_array) else {
        // Empty cycle with no connections is valid (explicit snapshot).
        return Ok(Vec::new());
    };
    let source = SnapshotConnSource {
        connections: {
            let mut out = Vec::with_capacity(arr.len());
            for item in arr {
                out.push(parse_one_conn(item)?);
            }
            out
        },
    };
    source.collect()
}

fn parse_one_conn(item: &Value) -> ShieldResult<ObservedConn> {
    let remote_ip = item
        .get("remote_ip")
        .and_then(Value::as_str)
        .ok_or_else(|| ShieldError::InvalidParams("connection.remote_ip required".into()))?
        .to_string();
    let local_port = item
        .get("local_port")
        .and_then(Value::as_u64)
        .ok_or_else(|| ShieldError::InvalidParams("connection.local_port required".into()))?;
    if local_port > u64::from(u16::MAX) {
        return Err(ShieldError::InvalidParams(
            "connection.local_port out of range".into(),
        ));
    }
    Ok(ObservedConn {
        remote_ip,
        local_port: local_port as u16,
        remote_port: item.get("remote_port").and_then(Value::as_u64).unwrap_or(0) as u16,
        process_name: item
            .get("process_name")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
        pid: item.get("pid").and_then(Value::as_i64).unwrap_or(0) as i32,
        exe_path: item
            .get("exe_path")
            .and_then(Value::as_str)
            .map(str::to_string),
        exe_sha256: item
            .get("exe_sha256")
            .and_then(Value::as_str)
            .map(str::to_string),
        rtt_ms: item
            .get("rtt_ms")
            .and_then(Value::as_u64)
            .map(|v| v as u32),
    })
}

fn require_peer_ip(params: &Value) -> ShieldResult<String> {
    let raw = params
        .get("peer_ip")
        .and_then(Value::as_str)
        .ok_or_else(|| ShieldError::InvalidParams("params.peer_ip required".into()))?;
    let _: IpAddr = raw
        .parse()
        .map_err(|_| ShieldError::InvalidParams("params.peer_ip must be a valid IP".into()))?;
    Ok(raw.to_string())
}

fn is_loopback_str(ip: &str) -> bool {
    ip.parse::<IpAddr>().map_or(true, |addr| match addr {
        IpAddr::V4(v4) => v4.is_loopback(),
        IpAddr::V6(v6) => v6.is_loopback(),
    })
}

#[must_use]
fn interval_elapsed(last_epoch: u64, now: u64, interval_secs: u64) -> bool {
    if last_epoch == 0 {
        return true;
    }
    now.saturating_sub(last_epoch) >= interval_secs.max(1)
}

fn peer_eligible_for_auto_act(policy: &SeatPolicy, peer: &str) -> bool {
    let Ok(ip) = peer.parse::<IpAddr>() else {
        return false;
    };
    if is_loopback_or_unspecified(&ip) || is_allowlisted(policy, peer) {
        return false;
    }
    if crate::ota_observe::is_ota_portal_vip(peer) {
        return false;
    }
    if is_private_or_link_local(&ip) && !policy.allow_private_on_protected {
        return false;
    }
    true
}

fn merge_exe_hashes(dst: &mut [ObservedConn], src: &[ObservedConn]) {
    for d in dst.iter_mut() {
        if d.exe_sha256.is_some() {
            continue;
        }
        if let Some(s) = src.iter().find(|s| {
            s.remote_ip == d.remote_ip
                && s.local_port == d.local_port
                && s.remote_port == d.remote_port
                && s.pid == d.pid
        }) {
            d.exe_path = s.exe_path.clone();
            d.exe_sha256 = s.exe_sha256.clone();
        }
    }
}

fn enrich_connections_for_wire(
    conns: &[ObservedConn],
    window: &PeerWindow,
    heuristics_on: bool,
) -> Vec<Value> {
    let mut rows: Vec<(u16, Value)> = conns
        .iter()
        .take(MAX_CONNECTIONS_WIRE)
        .map(|c| {
            let risk = if heuristics_on {
                window.risk_for(&c.remote_ip)
            } else {
                0
            };
            (
                risk,
                json!({
                    "remote_ip": c.remote_ip,
                    "local_port": c.local_port,
                    "remote_port": c.remote_port,
                    "process_name": c.process_name,
                    "pid": c.pid,
                    "exe_path": c.exe_path,
                    "rtt_ms": c.rtt_ms,
                    "risk_milli": risk,
                }),
            )
        })
        .collect();
    if heuristics_on {
        rows.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| {
            let a_ip = a.1.get("remote_ip").and_then(Value::as_str).unwrap_or("");
            let b_ip = b.1.get("remote_ip").and_then(Value::as_str).unwrap_or("");
            a_ip.cmp(b_ip)
        }));
    }
    rows.into_iter().map(|(_, v)| v).collect()
}

fn cycle_note(applied: usize, denied: usize) -> &'static str {
    if applied > 0 {
        "protected auto-act applied"
    } else if denied > 0 {
        "protected auto-act denied; floor unchanged"
    } else {
        "preview-only; cycle did not mutate floor"
    }
}

fn event_act_applied(peer: &str) -> ShieldEvent {
    ShieldEvent {
        kind: "act_applied".into(),
        severity: Severity::High,
        summary: format!("Protected auto-act blocked {peer}"),
        entity_ip: Some(peer.to_string()),
        entity_process: None,
        entity_port: None,
        explain_why: "Armed auto_act_protected applied BlockPeer for a protected-service probe."
            .into(),
        explain_action: "Review audit. Unblock only with UNBLOCK_PEER.".into(),
        confidence_milli: 950,
        tags: vec!["seat".into(), "act".into(), "auto_protected".into()],
        sequence: 0,
        epoch: 0,
        chain_id: None,
    }
}

fn event_act_denied(peer: &str) -> ShieldEvent {
    ShieldEvent {
        kind: "act_denied".into(),
        severity: Severity::Medium,
        summary: format!("Protected auto-act failed for {peer}"),
        entity_ip: Some(peer.to_string()),
        entity_process: None,
        entity_port: None,
        explain_why: "Floor helper rejected the protected-service auto-act. Not marked blocked."
            .into(),
        explain_action: "Fail closed. Do not treat the peer as blocked.".into(),
        confidence_milli: 900,
        tags: vec!["seat".into(), "act".into(), "auto_protected".into()],
        sequence: 0,
        epoch: 0,
        chain_id: None,
    }
}

impl From<serde_json::Error> for ShieldError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serialize(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tmp_cfg(actuator: ActuatorBackend) -> ServiceConfig {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        ServiceConfig {
            state_root: std::env::temp_dir().join(format!("seat_shield_svc2_{stamp}")),
            actuator,
            helper_path: None,
            helper_prefix: Vec::new(),
            enable_proc_observe: false,
            probe_helper_path: None,
        }
    }

    fn req(op: &str, params: Value) -> Vec<u8> {
        serde_json::to_vec(&json!({"id":"t1","op":op,"params":params})).unwrap()
    }

    #[test]
    fn probe_helper_snap_sets_backend_and_rtt() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("seat_probe_ok_{stamp}"));
        std::fs::create_dir_all(&dir).unwrap();
        let helper = dir.join("fake-probe");
        std::fs::write(
            &helper,
            r#"#!/bin/sh
echo '{"t":"snap","backend":"inet_diag","socks":[{"proto":"tcp","state":"ESTAB","src":"10.0.0.1","sport":22,"dst":"198.51.100.90","dport":40000,"inode":1,"pid":9,"comm":"sshd","rtt_us":25000,"retrans":0}]}'
"#,
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&helper, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        let mut cfg = tmp_cfg(ActuatorBackend::Unavailable);
        cfg.enable_proc_observe = true;
        cfg.probe_helper_path = Some(helper);
        let mut svc = ShieldService::open(cfg).unwrap();
        let resp = svc.handle_bytes(&req("seat.shield.observe", json!({})));
        assert!(resp.ok, "{resp:?}");
        assert_eq!(resp.data["observe_backend"], "probe_inet_diag");
        assert_eq!(resp.data["connections"][0]["remote_ip"], "198.51.100.90");
        assert_eq!(resp.data["connections"][0]["rtt_ms"], 25);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn probe_helper_timeout_falls_back_to_proc() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("seat_probe_to_{stamp}"));
        std::fs::create_dir_all(&dir).unwrap();
        let helper = dir.join("slow-probe");
        std::fs::write(&helper, "#!/bin/sh\nsleep 5\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&helper, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        let mut cfg = tmp_cfg(ActuatorBackend::Unavailable);
        cfg.enable_proc_observe = true;
        cfg.probe_helper_path = Some(helper);
        let mut svc = ShieldService::open(cfg).unwrap();
        let resp = svc.handle_bytes(&req("seat.shield.observe", json!({})));
        assert!(resp.ok, "{resp:?}");
        assert_eq!(resp.data["observe_backend"], "proc_net_tcp");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn default_status_observe_only_unavailable_backend() {
        let mut svc = ShieldService::open(tmp_cfg(ActuatorBackend::Unavailable)).unwrap();
        let resp = svc.handle_bytes(&req("seat.shield.status", json!({})));
        assert!(resp.ok);
        assert_eq!(resp.data["mode"], "unavailable");
        assert_eq!(resp.data["backend"], "unavailable");
        assert_eq!(resp.data["act_enabled"], true);
        assert_eq!(resp.data["auto_act_protected"], true);
        assert_eq!(resp.data["auto_quarantine_hash"], true);
        assert_eq!(resp.data["quarantine_count"], 0);
        assert_eq!(resp.data["watch_pending"], 0);
        assert_eq!(resp.data["last_watch_drained"], 0);
        assert_eq!(resp.data["integrity_twin_state"], "disabled");
        assert_eq!(resp.data["ota_baseline"], "missing");
        assert_eq!(resp.data["protected_ports"][0], 22);
        assert_eq!(resp.data["protected_ports"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn status_does_not_reobserve_inside_interval() {
        let mut cfg = tmp_cfg(ActuatorBackend::Unavailable);
        cfg.enable_proc_observe = true;
        let mut svc = ShieldService::open(cfg).unwrap();
        let seed = svc.handle_bytes(&req(
            "seat.shield.cycle",
            json!({"connections": [ssh_conn("198.51.100.20")]}),
        ));
        assert!(seed.ok, "{seed:?}");
        let first_epoch = svc.store().read_cycle().unwrap().last_cycle_epoch;
        assert!(first_epoch > 0);
        let status1 = svc.handle_bytes(&req("seat.shield.status", json!({})));
        assert!(status1.ok, "{status1:?}");
        assert_eq!(svc.store().read_cycle().unwrap().last_cycle_epoch, first_epoch);
        let status2 = svc.handle_bytes(&req("seat.shield.status", json!({})));
        assert!(status2.ok, "{status2:?}");
        assert_eq!(svc.store().read_cycle().unwrap().last_cycle_epoch, first_epoch);
    }

    #[test]
    fn block_requires_policy_confirm_and_recording_backend() {
        let mut svc = ShieldService::open(tmp_cfg(ActuatorBackend::Recording)).unwrap();
        let denied = svc.handle_bytes(&req(
            "seat.shield.act",
            json!({
                "action": "block_peer",
                "peer_ip": "198.51.100.20",
                "confirmation": "BLOCK_PEER"
            }),
        ));
        assert_eq!(denied.status, "not_available");

        let enable = svc.handle_bytes(&req(
            "seat.shield.policy.set",
            json!({
                "act_enabled": true,
                "confirmation": "ENABLE_ACTS"
            }),
        ));
        assert!(enable.ok, "{enable:?}");

        let missing = svc.handle_bytes(&req(
            "seat.shield.act",
            json!({
                "action": "block_peer",
                "peer_ip": "198.51.100.20"
            }),
        ));
        assert_eq!(
            missing.error.as_ref().map(|e| e.code.as_str()),
            Some("missing_confirmation")
        );

        let ok = svc.handle_bytes(&req(
            "seat.shield.act",
            json!({
                "action": "block_peer",
                "peer_ip": "198.51.100.20",
                "confirmation": "BLOCK_PEER"
            }),
        ));
        assert!(ok.ok);
        assert_eq!(ok.data["backend"], "recording");
    }

    #[test]
    fn cycle_and_events_and_reset() {
        let mut svc = ShieldService::open(tmp_cfg(ActuatorBackend::Unavailable)).unwrap();
        let cycle = svc.handle_bytes(&req(
            "seat.shield.cycle",
            json!({
                "connections": [{
                    "remote_ip": "198.51.100.20",
                    "local_port": 22,
                    "process_name": "sshd",
                    "pid": 1
                }]
            }),
        ));
        assert!(cycle.ok);
        assert_eq!(cycle.data["events"].as_array().unwrap().len(), 1);
        assert_eq!(cycle.data["connection_count"], 1);
        assert_eq!(cycle.data["connections"].as_array().unwrap().len(), 1);
        assert_eq!(cycle.data["connections"][0]["remote_ip"], "198.51.100.20");
        let events = svc.handle_bytes(&req(
            "seat.shield.events",
            json!({"after_sequence": 0, "limit": 10}),
        ));
        assert_eq!(events.data["count"], 1);
        let journal = svc.handle_bytes(&req(
            "seat.shield.journal",
            json!({"after_sequence": 0, "limit": 10, "analytics": true}),
        ));
        assert!(journal.ok, "{journal:?}");
        assert_eq!(journal.data["items"].as_array().unwrap().len(), 1);
        assert_eq!(journal.data["journal_lines"], 1);
        assert!(journal.data["analytics"].is_object());
        let status = svc.handle_bytes(&req("seat.shield.status", json!({})));
        assert!(status.ok, "{status:?}");
        assert_eq!(status.data["heuristics_enabled"], true);
        assert_eq!(status.data["connection_count"], 1);
        assert_eq!(status.data["connections"].as_array().unwrap().len(), 1);
        let heur_off = svc.handle_bytes(&req(
            "seat.shield.policy.set",
            json!({"heuristics_enabled": false}),
        ));
        assert!(heur_off.ok, "{heur_off:?}");
        assert_eq!(heur_off.data["heuristics_enabled"], false);
        let reset = svc.handle_bytes(&req(
            "seat.shield.reset",
            json!({"confirmation": "RESET_SEAT_SHIELD"}),
        ));
        assert!(reset.ok);
        let events_after = svc.handle_bytes(&req(
            "seat.shield.events",
            json!({"after_sequence": 0, "limit": 10}),
        ));
        assert_eq!(events_after.data["count"], 0);
        let journal_after = svc.handle_bytes(&req(
            "seat.shield.journal",
            json!({"after_sequence": 0, "limit": 10}),
        ));
        assert!(journal_after.ok, "{journal_after:?}");
        assert_eq!(journal_after.data["items"].as_array().unwrap().len(), 0);
        assert_eq!(journal_after.data["journal_lines"], 0);
    }

    #[test]
    fn heuristics_scan_then_probe_chain_and_kill_switch() {
        let mut svc = ShieldService::open(tmp_cfg(ActuatorBackend::Recording)).unwrap();
        // Seed scan on first cycle, probe on second (same peer window TTL).
        let scan = svc.handle_bytes(&req(
            "seat.shield.cycle",
            json!({
                "connections": [{
                    "remote_ip": "198.51.100.44",
                    "local_port": 443,
                    "remote_port": 50000,
                    "process_name": "nginx",
                    "pid": 2
                },{
                    "remote_ip": "198.51.100.44",
                    "local_port": 80,
                    "remote_port": 50001,
                    "process_name": "nginx",
                    "pid": 2
                },{
                    "remote_ip": "198.51.100.44",
                    "local_port": 8080,
                    "remote_port": 50002,
                    "process_name": "nginx",
                    "pid": 2
                },{
                    "remote_ip": "198.51.100.44",
                    "local_port": 8443,
                    "remote_port": 50003,
                    "process_name": "nginx",
                    "pid": 2
                },{
                    "remote_ip": "198.51.100.44",
                    "local_port": 9000,
                    "remote_port": 50004,
                    "process_name": "nginx",
                    "pid": 2
                },{
                    "remote_ip": "198.51.100.44",
                    "local_port": 9001,
                    "remote_port": 50005,
                    "process_name": "nginx",
                    "pid": 2
                }]
            }),
        ));
        assert!(scan.ok, "{scan:?}");
        let probe = svc.handle_bytes(&req(
            "seat.shield.cycle",
            json!({"connections": [ssh_conn("198.51.100.44")]}),
        ));
        assert!(probe.ok, "{probe:?}");
        let kinds: Vec<String> = probe.data["events"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["kind"].as_str().unwrap().to_string())
            .collect();
        assert!(
            kinds.iter().any(|k| k == "chain_scan_then_probe"),
            "expected chain in {kinds:?}"
        );
        let status = svc.handle_bytes(&req("seat.shield.status", json!({})));
        assert!(status.ok, "{status:?}");
        assert!(status.data["open_chains"].as_u64().unwrap() >= 1);
        assert!(!status.data["open_chain_teasers"]
            .as_array()
            .unwrap()
            .is_empty());

        let off = svc.handle_bytes(&req(
            "seat.shield.policy.set",
            json!({"heuristics_enabled": false}),
        ));
        assert!(off.ok, "{off:?}");
        let window = svc.store().read_peer_window().unwrap();
        assert!(window.peers.is_empty());
        let status_off = svc.handle_bytes(&req("seat.shield.status", json!({})));
        assert_eq!(status_off.data["open_chains"], 0);
        assert_eq!(status_off.data["heuristics_enabled"], false);

        // With heuristics off, same fixture must not emit chains.
        let again = svc.handle_bytes(&req(
            "seat.shield.cycle",
            json!({
                "connections": [
                    {"remote_ip":"198.51.100.55","local_port":80,"remote_port":1,"process_name":"n","pid":1},
                    {"remote_ip":"198.51.100.55","local_port":443,"remote_port":2,"process_name":"n","pid":1},
                    {"remote_ip":"198.51.100.55","local_port":8080,"remote_port":3,"process_name":"n","pid":1},
                    {"remote_ip":"198.51.100.55","local_port":8443,"remote_port":4,"process_name":"n","pid":1},
                    {"remote_ip":"198.51.100.55","local_port":9000,"remote_port":5,"process_name":"n","pid":1},
                    {"remote_ip":"198.51.100.55","local_port":9001,"remote_port":6,"process_name":"n","pid":1},
                    ssh_conn("198.51.100.55")
                ]
            }),
        ));
        assert!(again.ok, "{again:?}");
        let kinds_off: Vec<_> = again.data["events"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|e| e["kind"].as_str())
            .filter(|k| k.starts_with("chain_"))
            .collect();
        assert!(kinds_off.is_empty(), "{kinds_off:?}");
    }

    #[test]
    fn heuristics_do_not_change_auto_act_count() {
        let mut svc = ShieldService::open(tmp_cfg(ActuatorBackend::Recording)).unwrap();
        let _ = svc.handle_bytes(&req(
            "seat.shield.policy.set",
            json!({
                "act_enabled": true,
                "auto_act_protected": true,
                "confirmation": "ENABLE_ACTS"
            }),
        ));
        let baseline = svc.handle_bytes(&req(
            "seat.shield.cycle",
            json!({"connections": [ssh_conn("198.51.100.66")]}),
        ));
        assert!(baseline.ok, "{baseline:?}");
        let applied_n = baseline.data["applied_acts"].as_array().unwrap().len();
        assert_eq!(applied_n, 1);

        // Reset acts state via new service root would be heavy; second probe same IP
        // is suppressed for events but auto-act may skip already blocked.
        // Compare: with heuristics off, same single probe still applies once on fresh peer.
        let mut svc2 = ShieldService::open(tmp_cfg(ActuatorBackend::Recording)).unwrap();
        let _ = svc2.handle_bytes(&req(
            "seat.shield.policy.set",
            json!({
                "act_enabled": true,
                "auto_act_protected": true,
                "heuristics_enabled": false,
                "confirmation": "ENABLE_ACTS"
            }),
        ));
        let no_heur = svc2.handle_bytes(&req(
            "seat.shield.cycle",
            json!({"connections": [ssh_conn("198.51.100.77")]}),
        ));
        assert!(no_heur.ok, "{no_heur:?}");
        assert_eq!(
            no_heur.data["applied_acts"].as_array().unwrap().len(),
            applied_n
        );
    }

    #[test]
    fn feed_install_rejects_outside_state_root() {
        let mut svc = ShieldService::open(tmp_cfg(ActuatorBackend::Unavailable)).unwrap();
        let resp = svc.handle_bytes(&req(
            "seat.shield.feed.install",
            json!({"path": "/etc/passwd"}),
        ));
        assert!(!resp.ok);
        assert_eq!(
            resp.error.as_ref().map(|e| e.code.as_str()),
            Some("act_denied")
        );
    }

    #[test]
    fn helper_actuator_block_via_pinned_binary() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let helper = manifest_dir
            .join("target/debug/seat-shield-fw");
        if !helper.exists() {
            // Binary built by the same `cargo test` workspace compile of bins.
            let _ = std::process::Command::new("cargo")
                .args([
                    "build",
                    "--manifest-path",
                    manifest_dir.join("Cargo.toml").to_str().unwrap(),
                    "--bin",
                    "seat-shield-fw",
                    "-q",
                ])
                .status();
        }
        assert!(
            helper.exists(),
            "seat-shield-fw must exist at {}",
            helper.display()
        );

        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let state_root = std::env::temp_dir().join(format!("seat_shield_helper_{stamp}"));
        let mut svc = ShieldService::open(ServiceConfig {
            state_root,
            actuator: ActuatorBackend::Helper,
            helper_path: Some(helper),
            helper_prefix: Vec::new(),
            enable_proc_observe: false,
                    probe_helper_path: None,
        })
        .unwrap();

        let enable = svc.handle_bytes(&req(
            "seat.shield.policy.set",
            json!({
                "act_enabled": true,
                "confirmation": "ENABLE_ACTS"
            }),
        ));
        assert!(enable.ok, "{enable:?}");

        let block = svc.handle_bytes(&req(
            "seat.shield.act",
            json!({
                "action": "block_peer",
                "peer_ip": "198.51.100.77",
                "confirmation": "BLOCK_PEER"
            }),
        ));
        assert!(block.ok, "{block:?}");
        assert_eq!(block.data["backend"], "helper");

        let status = svc.handle_bytes(&req("seat.shield.status", json!({})));
        assert_eq!(status.data["backend"], "helper");
        assert_eq!(status.data["blocked_count"], 1);
    }

    fn ssh_conn(ip: &str) -> Value {
        json!({
            "remote_ip": ip,
            "local_port": 22,
            "remote_port": 50123,
            "process_name": "sshd",
            "pid": 1
        })
    }

    fn arm(svc: &mut ShieldService, extra: Value) {
        let mut params = json!({
            "act_enabled": true,
            "confirmation": "ENABLE_ACTS",
        });
        if let (Some(dst), Some(src)) = (params.as_object_mut(), extra.as_object()) {
            for (key, value) in src {
                dst.insert(key.clone(), value.clone());
            }
        }
        let resp = svc.handle_bytes(&req("seat.shield.policy.set", params));
        assert!(resp.ok, "{resp:?}");
    }

    #[test]
    fn auto_act_requires_enable_acts() {
        let mut svc = ShieldService::open(tmp_cfg(ActuatorBackend::Recording)).unwrap();
        let disarm = svc.handle_bytes(&req(
            "seat.shield.policy.set",
            json!({"act_enabled": false}),
        ));
        assert!(disarm.ok, "{disarm:?}");
        let denied = svc.handle_bytes(&req(
            "seat.shield.policy.set",
            json!({"auto_act_protected": true}),
        ));
        assert!(!denied.ok);
        assert_eq!(
            denied.error.as_ref().map(|e| e.code.as_str()),
            Some("act_denied")
        );
        let missing = svc.handle_bytes(&req(
            "seat.shield.policy.set",
            json!({
                "act_enabled": true,
                "auto_act_protected": true
            }),
        ));
        assert_eq!(
            missing.error.as_ref().map(|e| e.code.as_str()),
            Some("missing_confirmation")
        );
    }

    #[test]
    fn observe_only_cycle_does_not_mutate_floor() {
        let mut svc = ShieldService::open(tmp_cfg(ActuatorBackend::Recording)).unwrap();
        let blocked_path = svc.store().root().join("blocked.json");
        let cycle = svc.handle_bytes(&req(
            "seat.shield.cycle",
            json!({"connections": [ssh_conn("198.51.100.20")]}),
        ));
        assert!(cycle.ok, "{cycle:?}");
        assert!(cycle.data["applied_acts"].as_array().unwrap().is_empty());
        assert!(cycle.data["note"]
            .as_str()
            .unwrap()
            .contains("preview-only"));
        assert!(!blocked_path.exists());
    }

    #[test]
    fn armed_auto_act_blocks_protected_probe_only() {
        let mut svc = ShieldService::open(tmp_cfg(ActuatorBackend::Recording)).unwrap();
        arm(
            &mut svc,
            json!({"auto_act_protected": true}),
        );
        let cycle = svc.handle_bytes(&req(
            "seat.shield.cycle",
            json!({"connections": [ssh_conn("198.51.100.20")]}),
        ));
        assert!(cycle.ok, "{cycle:?}");
        assert_eq!(cycle.data["note"], "protected auto-act applied");
        let applied = cycle.data["applied_acts"].as_array().unwrap();
        assert_eq!(applied.len(), 1);
        assert_eq!(applied[0]["peer_ip"], "198.51.100.20");
        assert_eq!(applied[0]["reason"], "auto_protected");
        assert!(cycle.data["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["kind"] == "act_applied"));
        let status = svc.handle_bytes(&req("seat.shield.status", json!({})));
        assert_eq!(status.data["blocked_count"], 1);
        assert_eq!(status.data["blocked_peers"][0], "198.51.100.20");
    }

    #[test]
    fn auto_act_off_never_applies() {
        let mut svc = ShieldService::open(tmp_cfg(ActuatorBackend::Recording)).unwrap();
        arm(&mut svc, json!({}));
        let cycle = svc.handle_bytes(&req(
            "seat.shield.cycle",
            json!({"connections": [ssh_conn("198.51.100.20")]}),
        ));
        assert!(cycle.ok, "{cycle:?}");
        assert_eq!(cycle.data["proposed_acts"][0]["act"], "block_peer");
        assert!(cycle.data["applied_acts"].as_array().unwrap().is_empty());
        let status = svc.handle_bytes(&req("seat.shield.status", json!({})));
        assert_eq!(status.data["blocked_count"], 0);
    }

    #[test]
    fn auto_act_skips_allowlist_loopback_and_private_denied() {
        let mut svc = ShieldService::open(tmp_cfg(ActuatorBackend::Recording)).unwrap();
        arm(
            &mut svc,
            json!({
                "auto_act_protected": true,
                "allow_private_on_protected": false,
                "allowlist": ["198.51.100.7"]
            }),
        );
        for ip in ["198.51.100.7", "127.0.0.1", "192.168.1.77"] {
            let cycle = svc.handle_bytes(&req(
                "seat.shield.cycle",
                json!({"connections": [ssh_conn(ip)]}),
            ));
            assert!(cycle.ok, "{ip} {cycle:?}");
            assert!(
                cycle.data["applied_acts"].as_array().unwrap().is_empty(),
                "{ip} must not auto-block"
            );
        }
        let status = svc.handle_bytes(&req("seat.shield.status", json!({})));
        assert_eq!(status.data["blocked_count"], 0);
    }

    #[test]
    fn auto_act_blocks_private_when_allowed() {
        let mut svc = ShieldService::open(tmp_cfg(ActuatorBackend::Recording)).unwrap();
        arm(
            &mut svc,
            json!({
                "auto_act_protected": true,
                "allow_private_on_protected": true
            }),
        );
        let cycle = svc.handle_bytes(&req(
            "seat.shield.cycle",
            json!({"connections": [ssh_conn("192.168.1.77")]}),
        ));
        assert!(cycle.ok, "{cycle:?}");
        assert_eq!(cycle.data["applied_acts"][0]["peer_ip"], "192.168.1.77");
        let status = svc.handle_bytes(&req("seat.shield.status", json!({})));
        assert_eq!(status.data["blocked_count"], 1);
    }

    #[test]
    fn auto_act_respects_per_cycle_cap() {
        let mut svc = ShieldService::open(tmp_cfg(ActuatorBackend::Recording)).unwrap();
        arm(
            &mut svc,
            json!({
                "auto_act_protected": true,
                "max_auto_acts_per_cycle": 1
            }),
        );
        let cycle = svc.handle_bytes(&req(
            "seat.shield.cycle",
            json!({
                "connections": [
                    ssh_conn("198.51.100.20"),
                    ssh_conn("198.51.100.21")
                ]
            }),
        ));
        assert!(cycle.ok, "{cycle:?}");
        assert_eq!(cycle.data["applied_acts"].as_array().unwrap().len(), 1);
        let status = svc.handle_bytes(&req("seat.shield.status", json!({})));
        assert_eq!(status.data["blocked_count"], 1);
    }

    #[test]
    fn behavioral_events_are_not_applied_acts() {
        let mut svc = ShieldService::open(tmp_cfg(ActuatorBackend::Recording)).unwrap();
        arm(
            &mut svc,
            json!({"auto_act_protected": true, "act_enabled": true}),
        );
        let mut conns = Vec::new();
        for port in 3000_u16..3006 {
            conns.push(json!({
                "remote_ip": "198.51.100.50",
                "local_port": port,
                "remote_port": 4444,
                "process_name": "scan",
                "pid": 9
            }));
        }
        let cycle = svc.handle_bytes(&req(
            "seat.shield.cycle",
            json!({"connections": conns}),
        ));
        assert!(cycle.ok, "{cycle:?}");
        let events = cycle.data["events"].as_array().unwrap();
        assert!(events.iter().any(|e| e["kind"] == "possible_port_scan"));
        assert!(events.iter().any(|e| e["kind"] == "network_suspicious_port"));
        assert!(cycle.data["applied_acts"].as_array().unwrap().is_empty());
        assert!(cycle.data["proposed_acts"].as_array().unwrap().is_empty());
    }

    #[test]
    fn two_cycles_inside_ttl_do_not_double_flood() {
        let mut svc = ShieldService::open(tmp_cfg(ActuatorBackend::Unavailable)).unwrap();
        let first = svc.handle_bytes(&req(
            "seat.shield.cycle",
            json!({"connections": [ssh_conn("198.51.100.20")]}),
        ));
        assert_eq!(first.data["events"].as_array().unwrap().len(), 1);
        let second = svc.handle_bytes(&req(
            "seat.shield.cycle",
            json!({"connections": [ssh_conn("198.51.100.20")]}),
        ));
        assert!(second.ok, "{second:?}");
        assert!(second.data["events"].as_array().unwrap().is_empty());
        assert!(second.data["suppressed_duplicates"].as_u64().unwrap() >= 1);
        assert!(!svc.maybe_auto_observe().unwrap());
        assert!(interval_elapsed(0, 50, 30));
        assert!(!interval_elapsed(40, 50, 30));
        assert!(interval_elapsed(10, 40, 30));
    }

    fn sample_path(tag: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        PathBuf::from(format!("/tmp/seat_s5a_{stamp}_{tag}"))
    }

    fn write_sample(tag: &str, bytes: &[u8]) -> PathBuf {
        let path = sample_path(tag);
        std::fs::write(&path, bytes).unwrap();
        path
    }

    fn digest_of(bytes: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        format!("{:x}", Sha256::digest(bytes))
    }

    fn install_hash_feed(svc: &mut ShieldService, digest: &str) {
        let stage = svc.store().state_root().join("feed-stage.json");
        std::fs::write(
            &stage,
            serde_json::to_vec(&json!({
                "version": "s5a-test",
                "ips": [],
                "hashes": [digest]
            }))
            .unwrap(),
        )
        .unwrap();
        let resp = svc.handle_bytes(&req(
            "seat.shield.feed.install",
            json!({"path": stage.to_str().expect("utf8 feed stage")}),
        ));
        assert!(resp.ok, "{resp:?}");
    }

    fn enable_auto_quarantine(svc: &mut ShieldService) {
        arm(svc, json!({}));
        let resp = svc.handle_bytes(&req(
            "seat.shield.policy.set",
            json!({
                "auto_quarantine_hash": true,
                "confirmation": "ENABLE_QUARANTINE"
            }),
        ));
        assert!(resp.ok, "{resp:?}");
    }

    fn hashed_tmp_conn(ip: &str, path: &Path, digest: &str) -> Value {
        json!({
            "remote_ip": ip,
            "local_port": 22,
            "remote_port": 50123,
            "process_name": "drop",
            "pid": 9,
            "exe_path": path.to_string_lossy(),
            "exe_sha256": digest
        })
    }

    #[test]
    fn scan_tmp_file_with_feed_hash_emits_hit() {
        let mut svc = ShieldService::open(tmp_cfg(ActuatorBackend::Unavailable)).unwrap();
        let bytes = b"s5a-hit-sample";
        let path = write_sample("hit", bytes);
        let digest = digest_of(bytes);
        install_hash_feed(&mut svc, &digest);
        let scan = svc.handle_bytes(&req(
            "seat.shield.scan_path",
            json!({"path": path.to_string_lossy()}),
        ));
        assert!(scan.ok, "{scan:?}");
        assert_eq!(scan.data["feed_hit"], true);
        assert_eq!(scan.data["sha256"], digest);
        let kinds: Vec<_> = scan.data["events"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["kind"].as_str().unwrap().to_string())
            .collect();
        assert!(kinds.contains(&"file_scan_hit".into()));
        assert!(kinds.contains(&"ioc_hash_match".into()));
    }

    #[test]
    fn scan_etc_passwd_is_denied_and_stores_no_hash() {
        let mut svc = ShieldService::open(tmp_cfg(ActuatorBackend::Unavailable)).unwrap();
        let scan = svc.handle_bytes(&req(
            "seat.shield.scan_path",
            json!({"path": "/etc/passwd"}),
        ));
        assert!(!scan.ok);
        assert_eq!(
            scan.error.as_ref().map(|e| e.code.as_str()),
            Some("act_denied")
        );
        let events = svc.handle_bytes(&req(
            "seat.shield.events",
            json!({"after_sequence": 0, "limit": 20}),
        ));
        let items = events.data["items"].as_array().unwrap();
        assert!(items.iter().any(|e| e["kind"] == "file_scan_denied"));
        assert!(!items.iter().any(|e| e["kind"] == "file_scan_hit"));
    }

    #[test]
    fn quarantine_tmp_then_restore_bytes_match() {
        let mut svc = ShieldService::open(tmp_cfg(ActuatorBackend::Unavailable)).unwrap();
        let bytes = b"s5a-hold-bytes";
        let path = write_sample("hold", bytes);
        let missing = svc.handle_bytes(&req(
            "seat.shield.quarantine",
            json!({"path": path.to_string_lossy()}),
        ));
        assert_eq!(
            missing.error.as_ref().map(|e| e.code.as_str()),
            Some("missing_confirmation")
        );
        let hold = svc.handle_bytes(&req(
            "seat.shield.quarantine",
            json!({
                "path": path.to_string_lossy(),
                "confirmation": "QUARANTINE_FILE"
            }),
        ));
        assert!(hold.ok, "{hold:?}");
        assert!(!path.exists());
        let digest = hold.data["sha256"].as_str().unwrap().to_string();
        assert!(svc.store().quarantine_dir().join(&digest).is_file());
        let list = svc.handle_bytes(&req("seat.shield.quarantine.list", json!({})));
        assert_eq!(list.data["count"], 1);
        assert_eq!(
            list.data["items"][0]["orig_basename"],
            path.file_name().unwrap().to_string_lossy().as_ref()
        );
        assert!(list.data["items"][0]["orig_path"].is_null());
        let restore = svc.handle_bytes(&req(
            "seat.shield.quarantine.restore",
            json!({
                "sha256": digest,
                "confirmation": "RESTORE_QUARANTINE"
            }),
        ));
        assert!(restore.ok, "{restore:?}");
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }

    #[test]
    fn symlink_escape_and_huge_file_are_denied_or_skipped() {
        let mut svc = ShieldService::open(tmp_cfg(ActuatorBackend::Unavailable)).unwrap();
        let link = sample_path("symlink");
        std::os::unix::fs::symlink("/etc/passwd", &link).unwrap();
        let scan = svc.handle_bytes(&req(
            "seat.shield.scan_path",
            json!({"path": link.to_string_lossy()}),
        ));
        assert!(!scan.ok);
        let hold = svc.handle_bytes(&req(
            "seat.shield.quarantine",
            json!({
                "path": link.to_string_lossy(),
                "confirmation": "QUARANTINE_FILE"
            }),
        ));
        assert!(!hold.ok);
        let huge = sample_path("huge");
        let file = std::fs::File::create(&huge).unwrap();
        file.set_len(33 * 1024 * 1024).unwrap();
        drop(file);
        let skip = svc.handle_bytes(&req(
            "seat.shield.scan_path",
            json!({"path": huge.to_string_lossy()}),
        ));
        assert!(skip.ok, "{skip:?}");
        assert_eq!(skip.data["skipped"], true);
        assert!(skip.data["sha256"].is_null());
    }

    #[test]
    fn auto_quarantine_off_does_not_move_hash_match() {
        let mut svc = ShieldService::open(tmp_cfg(ActuatorBackend::Recording)).unwrap();
        let bytes = b"s5a-auto-off";
        let path = write_sample("autooff", bytes);
        let digest = digest_of(bytes);
        install_hash_feed(&mut svc, &digest);
        arm(&mut svc, json!({}));
        let cycle = svc.handle_bytes(&req(
            "seat.shield.cycle",
            json!({"connections": [hashed_tmp_conn("198.51.100.20", &path, &digest)]}),
        ));
        assert!(cycle.ok, "{cycle:?}");
        assert!(cycle.data["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["kind"] == "ioc_hash_match"));
        assert!(!cycle.data["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["kind"] == "file_quarantined"));
        assert!(path.exists());
    }

    #[test]
    fn auto_quarantine_hash_moves_only_ioc_match_not_tmp_exec() {
        let mut svc = ShieldService::open(tmp_cfg(ActuatorBackend::Recording)).unwrap();
        let hit = b"s5a-auto-on";
        let miss = b"s5a-tmp-only";
        let hit_path = write_sample("autoon", hit);
        let miss_path = write_sample("tmpexec", miss);
        let digest = digest_of(hit);
        install_hash_feed(&mut svc, &digest);
        enable_auto_quarantine(&mut svc);
        let miss_cycle = svc.handle_bytes(&req(
            "seat.shield.cycle",
            json!({"connections": [hashed_tmp_conn("198.51.100.21", &miss_path, &digest_of(miss))]}),
        ));
        assert!(miss_cycle.ok, "{miss_cycle:?}");
        assert!(miss_cycle.data["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["kind"] == "suspicious_exec_location"));
        assert!(!miss_cycle.data["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["kind"] == "file_quarantined"));
        assert!(miss_path.exists());
        let hit_cycle = svc.handle_bytes(&req(
            "seat.shield.cycle",
            json!({"connections": [hashed_tmp_conn("198.51.100.22", &hit_path, &digest)]}),
        ));
        assert!(hit_cycle.ok, "{hit_cycle:?}");
        assert!(hit_cycle.data["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["kind"] == "file_quarantined"));
        assert!(!hit_path.exists());
    }

    #[test]
    fn behavioral_events_never_quarantine() {
        let mut svc = ShieldService::open(tmp_cfg(ActuatorBackend::Recording)).unwrap();
        enable_auto_quarantine(&mut svc);
        let bytes = b"s5a-behavior";
        let path = write_sample("behave", bytes);
        let mut conns = Vec::new();
        for port in 3000_u16..3006 {
            conns.push(json!({
                "remote_ip": "198.51.100.50",
                "local_port": port,
                "remote_port": 4444,
                "process_name": "scan",
                "pid": 9,
                "exe_path": path.to_string_lossy(),
                "exe_sha256": digest_of(bytes)
            }));
        }
        let cycle = svc.handle_bytes(&req(
            "seat.shield.cycle",
            json!({"connections": conns}),
        ));
        assert!(cycle.ok, "{cycle:?}");
        assert!(cycle.data["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["kind"] == "possible_port_scan"));
        assert!(!cycle.data["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["kind"] == "file_quarantined"));
        assert!(path.exists());
    }

    #[test]
    fn auto_quarantine_requires_enable_quarantine() {
        let mut svc = ShieldService::open(tmp_cfg(ActuatorBackend::Recording)).unwrap();
        arm(&mut svc, json!({}));
        let denied = svc.handle_bytes(&req(
            "seat.shield.policy.set",
            json!({"auto_quarantine_hash": true}),
        ));
        assert_eq!(
            denied.error.as_ref().map(|e| e.code.as_str()),
            Some("missing_confirmation")
        );
        let mut without_acts = ShieldService::open(tmp_cfg(ActuatorBackend::Unavailable)).unwrap();
        let blocked = without_acts.handle_bytes(&req(
            "seat.shield.policy.set",
            json!({
                "auto_quarantine_hash": true,
                "confirmation": "ENABLE_QUARANTINE"
            }),
        ));
        assert_eq!(
            blocked.error.as_ref().map(|e| e.code.as_str()),
            Some("act_denied")
        );
    }

    #[test]
    fn watch_drain_on_status_hashes_tmp_and_does_not_hold() {
        let mut svc = ShieldService::open(tmp_cfg(ActuatorBackend::Unavailable)).unwrap();
        let bytes = b"s5b-watch-clean";
        let path = write_sample("watchclean", bytes);
        svc.store()
            .append_watch(&crate::watch::WatchRecord::new(
                1,
                path.to_string_lossy().into_owned(),
                "create",
            ))
            .unwrap();
        let status = svc.handle_bytes(&req("seat.shield.status", json!({})));
        assert!(status.ok, "{status:?}");
        assert_eq!(status.data["watch_pending"], 0);
        assert_eq!(status.data["integrity_twin_state"], "disabled");
        assert!(path.exists());
        let events = svc.handle_bytes(&req(
            "seat.shield.events",
            json!({"after_sequence": 0, "limit": 20}),
        ));
        let items = events.data["items"].as_array().unwrap();
        assert!(!items.iter().any(|e| e["kind"] == "file_quarantined"));
        assert!(!items.iter().any(|e| e["kind"] == "file_scan_hit"));
    }

    #[test]
    fn watch_drain_feed_hit_holds_only_when_auto_quarantine_on() {
        let mut svc = ShieldService::open(tmp_cfg(ActuatorBackend::Recording)).unwrap();
        let bytes = b"s5b-watch-hit";
        let path = write_sample("watchhit", bytes);
        let digest = digest_of(bytes);
        install_hash_feed(&mut svc, &digest);
        svc.store()
            .append_watch(&crate::watch::WatchRecord::new(
                1,
                path.to_string_lossy().into_owned(),
                "close_write",
            ))
            .unwrap();
        let off = svc.handle_bytes(&req("seat.shield.status", json!({})));
        assert!(off.ok, "{off:?}");
        assert!(path.exists());
        let events = svc.handle_bytes(&req(
            "seat.shield.events",
            json!({"after_sequence": 0, "limit": 20}),
        ));
        assert!(events.data["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["kind"] == "file_scan_hit"));
        enable_auto_quarantine(&mut svc);
        let path2 = write_sample("watchhit2", bytes);
        svc.store()
            .append_watch(&crate::watch::WatchRecord::new(
                2,
                path2.to_string_lossy().into_owned(),
                "close_write",
            ))
            .unwrap();
        let on = svc.handle_bytes(&req("seat.shield.status", json!({})));
        assert!(on.ok, "{on:?}");
        assert!(!path2.exists());
        assert_eq!(on.data["quarantine_count"], 1);
    }

    #[test]
    fn watch_journal_etc_passwd_does_not_store_hash() {
        let mut svc = ShieldService::open(tmp_cfg(ActuatorBackend::Unavailable)).unwrap();
        let denied = svc
            .store()
            .append_watch(&crate::watch::WatchRecord::new(1, "/etc/passwd", "create"));
        assert!(denied.is_err());
        let status = svc.handle_bytes(&req("seat.shield.status", json!({})));
        assert!(status.ok, "{status:?}");
        assert_eq!(status.data["watch_pending"], 0);
    }

    #[test]
    fn watch_journal_planted_etc_passwd_is_denied_without_hash() {
        let mut svc = ShieldService::open(tmp_cfg(ActuatorBackend::Unavailable)).unwrap();
        let planted = concat!(
            r#"{"epoch":1,"path":"/etc/passwd","op":"create"}"#,
            "\n"
        );
        std::fs::write(svc.store().watch_path(), planted).unwrap();
        let status = svc.handle_bytes(&req("seat.shield.status", json!({})));
        assert!(status.ok, "{status:?}");
        assert_eq!(status.data["watch_pending"], 0);
        let events = svc.handle_bytes(&req(
            "seat.shield.events",
            json!({"after_sequence": 0, "limit": 20}),
        ));
        let items = events.data["items"].as_array().unwrap();
        assert!(items.iter().any(|e| e["kind"] == "file_scan_denied"));
        assert!(!items.iter().any(|e| e["kind"] == "file_scan_hit"));
        assert!(!items.iter().any(|e| e["sha256"].is_string()));
    }

    #[test]
    fn watch_journal_missing_tmp_without_hash_is_silent() {
        let mut svc = ShieldService::open(tmp_cfg(ActuatorBackend::Unavailable)).unwrap();
        let planted = concat!(
            r#"{"epoch":1,"path":"/tmp/seat_s5c_gone_nohash","op":"close_write"}"#,
            "\n"
        );
        std::fs::write(svc.store().watch_path(), planted).unwrap();
        let status = svc.handle_bytes(&req("seat.shield.status", json!({})));
        assert!(status.ok, "{status:?}");
        assert_eq!(status.data["watch_pending"], 0);
        let events = svc.handle_bytes(&req(
            "seat.shield.events",
            json!({"after_sequence": 0, "limit": 20}),
        ));
        let items = events.data["items"].as_array().unwrap();
        assert!(
            !items.iter().any(|e| e["kind"] == "file_scan_denied"),
            "{items:?}"
        );
        assert!(!items.iter().any(|e| e["kind"] == "file_scan_hit"));
    }

    #[test]
    fn scan_missing_tmp_is_skipped_not_denied() {
        let mut svc = ShieldService::open(tmp_cfg(ActuatorBackend::Unavailable)).unwrap();
        let scan = svc.handle_bytes(&req(
            "seat.shield.scan_path",
            json!({"path": "/tmp/seat_s5c_operator_missing"}),
        ));
        assert!(scan.ok, "{scan:?}");
        assert_eq!(scan.data["skipped"], true);
        assert_eq!(scan.data["feed_hit"], false);
        let events = svc.handle_bytes(&req(
            "seat.shield.events",
            json!({"after_sequence": 0, "limit": 20}),
        ));
        let items = events.data["items"].as_array().unwrap();
        assert!(!items.iter().any(|e| e["kind"] == "file_scan_denied"));
    }

    #[test]
    fn watch_journal_hash_matches_feed_when_file_gone() {
        let mut svc = ShieldService::open(tmp_cfg(ActuatorBackend::Unavailable)).unwrap();
        let digest = digest_of(b"s5c-helper-hash");
        install_hash_feed(&mut svc, &digest);
        // File is gone; Runtime cannot open it. Journal hash still matches the feed.
        let planted = format!(
            r#"{{"epoch":1,"path":"/tmp/seat_s5c_missing","op":"close_write","sha256":"{digest}"}}"#
        );
        std::fs::write(svc.store().watch_path(), planted).unwrap();
        let status = svc.handle_bytes(&req("seat.shield.status", json!({})));
        assert!(status.ok, "{status:?}");
        assert_eq!(status.data["watch_pending"], 0);
        let events = svc.handle_bytes(&req(
            "seat.shield.events",
            json!({"after_sequence": 0, "limit": 20}),
        ));
        let items = events.data["items"].as_array().unwrap();
        assert!(
            items.iter().any(|e| e["kind"] == "file_scan_hit"),
            "{items:?}"
        );
        assert!(!items.iter().any(|e| e["kind"] == "file_quarantined"));
        assert_eq!(status.data["quarantine_count"], 0);
    }

    #[test]
    fn watch_planted_etc_sha256_is_ignored() {
        let mut svc = ShieldService::open(tmp_cfg(ActuatorBackend::Unavailable)).unwrap();
        let digest = digest_of(b"s5c-etc-ignore");
        install_hash_feed(&mut svc, &digest);
        let planted = format!(
            r#"{{"epoch":1,"path":"/etc/passwd","op":"create","sha256":"{digest}"}}"#
        );
        std::fs::write(svc.store().watch_path(), planted).unwrap();
        let status = svc.handle_bytes(&req("seat.shield.status", json!({})));
        assert!(status.ok, "{status:?}");
        let events = svc.handle_bytes(&req(
            "seat.shield.events",
            json!({"after_sequence": 0, "limit": 20}),
        ));
        let items = events.data["items"].as_array().unwrap();
        assert!(items.iter().any(|e| e["kind"] == "file_scan_denied"));
        assert!(!items.iter().any(|e| e["kind"] == "file_scan_hit"));
    }
}
