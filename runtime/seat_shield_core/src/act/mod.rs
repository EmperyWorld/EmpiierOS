//! Floor actuator interface — no shell strings, no nft DSL from Shell.

mod helper;
mod recording;
mod unavailable;

use crate::error::ShieldResult;

pub use helper::{helper_path_is_safe, HelperActuator};
pub use recording::RecordingActuator;
pub use unavailable::UnavailableActuator;

/// Named backends allowed in prep bay / future Runtime wiring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActuatorBackend {
    /// Production MVP honesty: acts fail closed.
    Unavailable,
    /// Durable recording backend for unit tests (in-process, not nft).
    Recording,
    /// Absolute-path pinned helper (`seat-shield-fw` / future EmpiierOS helper).
    Helper,
}

impl ActuatorBackend {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unavailable => "unavailable",
            Self::Recording => "recording",
            Self::Helper => "helper",
        }
    }
}

/// Mutate seat firewall state through an allowlisted backend.
pub trait FirewallActuator: Send {
    fn backend_name(&self) -> &'static str;

    /// `true` when destructive block/unblock may proceed (subject to policy).
    fn is_available(&self) -> bool;

    fn block_peer(&mut self, peer_ip: &str, reason: &str) -> ShieldResult<()>;

    fn unblock_peer(&mut self, peer_ip: &str) -> ShieldResult<bool>;

    fn list_blocked(&self) -> ShieldResult<Vec<String>>;
}
