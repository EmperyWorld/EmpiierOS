//! EmpiierOS Seat Shield — Runtime-shaped core (vendored in this tree).
//!
//! Contract: `session/SEAT-SHIELD.md`
//!
//! It forbids `unsafe`, shell-string firewall calls, and silent fail-open acts.

pub mod act;
pub mod bridge;
pub mod command_runner;
pub mod cycle;
pub mod error;
pub mod event;
pub mod feed;
pub mod fw;
pub mod heuristics;
pub mod journal;
pub mod observe;
pub mod ota_observe;
pub mod parser_probe;
pub mod policy;
pub mod protocol;
pub mod quarantine;
pub mod service;
pub mod store;
pub mod timeutil;
pub mod watch;

pub use act::{
    helper_path_is_safe, ActuatorBackend, FirewallActuator, HelperActuator, RecordingActuator,
    UnavailableActuator,
};
pub use bridge::{handle_runtime_line, is_seat_shield_op, runtime_status_class, SEAT_SHIELD_OPS};
pub use cycle::{run_cycle, run_cycle_with_feed, ActDecision, ActKind, CycleOutcome};
pub use error::{ShieldError, ShieldResult};
pub use event::{Severity, ShieldEvent, SuppressionCache};
pub use feed::{FeedStatus, IocFeed};
pub use observe::{
    parse_probe_snap, probe_path_is_safe, ConnSource, ObservedConn, ProcConnSource, ProbeConnSource,
    SnapshotConnSource, PROBE_BACKEND, PROBE_TTL_SECS,
};
pub use ota_observe::{
    emit_ota_capsule_staged, emit_ota_component_mismatch, is_ota_portal_vip,
    observe_ota_baseline, ota_component_mismatch_event, write_ota_baseline, OtaBaselineStatus,
    OTA_PORTAL_VIP,
};
pub use policy::{SeatPolicy, DEFAULT_PROTECTED_PORTS, SUSPICIOUS_EXEC_PREFIXES};
pub use protocol::{ShieldRequest, ShieldResponse, PROTOCOL_VERSION};
pub use service::{ServiceConfig, ShieldService};
pub use store::ShieldStore;
