//! Connection observation sources.

pub(crate) mod proc;
pub(crate) mod probe;
mod taint;

use serde::{Deserialize, Serialize};

use crate::error::ShieldResult;

pub use proc::ProcConnSource;
pub use probe::{
    parse_probe_snap, probe_path_is_safe, ProbeConnSource, PROBE_BACKEND, PROBE_TTL_SECS,
};
pub use taint::{deep_period_due, kernel_taint_event, read_host_tainted, KERNEL_TAINT_DEEP_SECS};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservedConn {
    pub remote_ip: String,
    pub local_port: u16,
    pub remote_port: u16,
    pub process_name: String,
    pub pid: i32,
    pub exe_path: Option<String>,
    #[serde(default)]
    pub exe_sha256: Option<String>,
    /// Round-trip time in milliseconds from inet_diag (probe helper). Absent on `/proc`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rtt_ms: Option<u32>,
}

/// Source of observed seat connections.
pub trait ConnSource: Send {
    fn collect(&self) -> ShieldResult<Vec<ObservedConn>>;

    fn backend_name(&self) -> &'static str;
}

/// Deterministic source for tests and injected cycle params.
#[derive(Debug, Clone, Default)]
pub struct SnapshotConnSource {
    pub connections: Vec<ObservedConn>,
}

impl ConnSource for SnapshotConnSource {
    fn collect(&self) -> ShieldResult<Vec<ObservedConn>> {
        Ok(self.connections.clone())
    }

    fn backend_name(&self) -> &'static str {
        "snapshot"
    }
}
