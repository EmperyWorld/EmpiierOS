//! Kernel taint signal from procfs. Not lsmod, not AIDE, not `/etc` baselines.

use crate::event::{Severity, ShieldEvent};

/// Deep-observe period. Not every connection cycle.
pub const KERNEL_TAINT_DEEP_SECS: u64 = 300;

/// Out-of-tree module bit alone (often NVIDIA). Monitoring already ignores this.
pub const KERNEL_TAINT_OOT_ONLY: i64 = 4096;

#[must_use]
pub fn parse_tainted(raw: &str) -> Option<i64> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    trimmed.parse().ok()
}

#[must_use]
pub fn taint_is_notable(value: i64) -> bool {
    value != 0 && value != KERNEL_TAINT_OOT_ONLY
}

#[must_use]
pub fn kernel_taint_event(value: i64) -> Option<ShieldEvent> {
    if !taint_is_notable(value) {
        return None;
    }
    Some(ShieldEvent {
        kind: "kernel_taint".into(),
        severity: Severity::High,
        summary: format!("Kernel tainted value {value}"),
        entity_ip: None,
        entity_process: None,
        entity_port: None,
        explain_why: "procfs kernel.tainted is non-zero and is not the out-of-tree-only value 4096."
            .into(),
        explain_action: "Investigate the seat kernel. This is not an lsmod or AIDE source of truth."
            .into(),
        confidence_milli: 800,
        tags: vec!["seat".into(), "kernel".into()],
        sequence: 0,
        epoch: 0,
        chain_id: None,
    })
}

#[must_use]
pub fn deep_period_due(last_epoch: u64, now: u64) -> bool {
    last_epoch == 0 || now.saturating_sub(last_epoch) >= KERNEL_TAINT_DEEP_SECS
}

#[must_use]
pub fn read_host_tainted() -> Option<i64> {
    let raw = std::fs::read_to_string("/proc/sys/kernel/tainted").ok()?;
    parse_tainted(&raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_clean_and_oot_only() {
        assert!(!taint_is_notable(0));
        assert!(!taint_is_notable(KERNEL_TAINT_OOT_ONLY));
        assert!(kernel_taint_event(0).is_none());
        assert!(kernel_taint_event(4096).is_none());
        let event = kernel_taint_event(1).expect("notable");
        assert_eq!(event.kind, "kernel_taint");
        assert_eq!(parse_tainted("  512\n"), Some(512));
        assert!(parse_tainted("nope").is_none());
    }

    #[test]
    fn deep_period_gates_repeat() {
        assert!(deep_period_due(0, 10));
        assert!(!deep_period_due(100, 200));
        assert!(deep_period_due(100, 100 + KERNEL_TAINT_DEEP_SECS));
    }
}
