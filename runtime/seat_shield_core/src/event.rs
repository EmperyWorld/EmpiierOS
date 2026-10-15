//! Events and suppression.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::timeutil::now_epoch;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }

    #[must_use]
    pub fn is_high_or_above(self) -> bool {
        matches!(self, Self::High | Self::Critical)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShieldEvent {
    pub kind: String,
    pub severity: Severity,
    pub summary: String,
    pub entity_ip: Option<String>,
    pub entity_process: Option<String>,
    pub entity_port: Option<u16>,
    pub explain_why: String,
    pub explain_action: String,
    pub confidence_milli: u16,
    pub tags: Vec<String>,
    #[serde(default)]
    pub sequence: u64,
    #[serde(default)]
    pub epoch: u64,
    /// Compound-event correlation id (P2). Absent on atomic-only lines.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<String>,
}

fn suppression_key(event: &ShieldEvent) -> String {
    format!(
        "{}|{}|{}|{}",
        event.kind,
        event.entity_ip.as_deref().unwrap_or(""),
        event.entity_process.as_deref().unwrap_or(""),
        event.entity_port.map(|p| p.to_string()).unwrap_or_default()
    )
}

/// Deduplicate events that share a stable key inside the TTL window.
#[derive(Debug, Default, Clone)]
pub struct SuppressionCache {
    last_seen: HashMap<String, u64>,
    ttl_secs: u64,
}

impl SuppressionCache {
    #[must_use]
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            last_seen: HashMap::new(),
            ttl_secs: ttl_secs.max(1),
        }
    }

    pub fn set_ttl(&mut self, ttl_secs: u64) {
        self.ttl_secs = ttl_secs.max(1);
    }

    pub fn filter(&mut self, events: Vec<ShieldEvent>) -> (Vec<ShieldEvent>, u32) {
        let now = now_epoch();
        let mut kept = Vec::new();
        let mut suppressed = 0_u32;
        for event in events {
            let key = suppression_key(&event);
            if let Some(prev) = self.last_seen.get(&key) {
                if now.saturating_sub(*prev) < self.ttl_secs {
                    suppressed += 1;
                    continue;
                }
            }
            self.last_seen.insert(key, now);
            kept.push(event);
        }
        (kept, suppressed)
    }
}
