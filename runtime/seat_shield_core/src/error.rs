//! Typed errors — fail closed, no stringly-typed soup at call sites.

use thiserror::Error;

pub type ShieldResult<T> = Result<T, ShieldError>;

#[derive(Debug, Error)]
pub enum ShieldError {
    #[error("invalid_request: {0}")]
    InvalidRequest(&'static str),

    #[error("invalid_params: {0}")]
    InvalidParams(String),

    #[error("operation_not_found: {0}")]
    OperationNotFound(String),

    #[error("capability_not_available: {0}")]
    NotAvailable(String),

    #[error("act_denied: {0}")]
    ActDenied(String),

    #[error("missing_confirmation: expected {expected}")]
    MissingConfirmation { expected: &'static str },

    #[error("unallowlisted_action: {0}")]
    UnallowlistedAction(String),

    #[error("io: {context}: {source}")]
    Io {
        context: &'static str,
        #[source]
        source: std::io::Error,
    },

    #[error("serialize: {0}")]
    Serialize(String),

    #[error("parse: {context}: {detail}")]
    Parse {
        context: &'static str,
        detail: String,
    },

    #[error("observe: {0}")]
    Observe(String),

    #[error("actuator: {0}")]
    Actuator(String),

    #[error("audit_write_failed: {0}")]
    Audit(String),
}

impl ShieldError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidRequest(_) | Self::InvalidParams(_) => "invalid_request",
            Self::OperationNotFound(_) => "operation_not_found",
            Self::NotAvailable(_) => "capability_not_available",
            Self::ActDenied(_) => "act_denied",
            Self::MissingConfirmation { .. } => "missing_confirmation",
            Self::UnallowlistedAction(_) => "unallowlisted_action",
            Self::Io { .. } => "io_error",
            Self::Serialize(_) => "serialize_failed",
            Self::Parse { .. } => "parse_failed",
            Self::Observe(_) => "observe_failed",
            Self::Actuator(_) => "actuator_failed",
            Self::Audit(_) => "audit_write_failed",
        }
    }

    pub fn httpish_status(&self) -> &'static str {
        match self {
            Self::InvalidRequest(_) | Self::InvalidParams(_) => "invalid_request",
            Self::OperationNotFound(_) => "not_found",
            Self::NotAvailable(_) => "not_available",
            Self::Audit(_) => "audit_failed",
            _ => "failed",
        }
    }

    /// Vanished or unreadable allowlisted path. Not a policy deny.
    #[must_use]
    pub fn is_scan_path_absent(&self) -> bool {
        match self {
            Self::Io { context, source }
                if matches!(
                    *context,
                    "scan_path_stat" | "scan_path_canonicalize" | "scan_path_readlink"
                ) =>
            {
                matches!(
                    source.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::PermissionDenied
                )
            }
            _ => false,
        }
    }
}
