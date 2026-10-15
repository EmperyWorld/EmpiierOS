//! Durable browse journal (`journal.jsonl`) — copy of stamped events, not decision SoT.
//!
//! `events.json` remains the short ring for status / IPC. This file is operator forensics.

use serde_json::Value;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use crate::error::{ShieldError, ShieldResult};
use crate::event::{Severity, ShieldEvent};

pub const MAX_JOURNAL_LINES: usize = 1000;
pub const MAX_JOURNAL_BYTES: u64 = 1024 * 1024;
pub const MAX_JOURNAL_QUERY: usize = 200;

/// Append stamped events as one JSON object per line. Rotates by line count and byte size.
pub fn append_journal_lines(path: &Path, events: &[ShieldEvent]) -> ShieldResult<()> {
    if events.is_empty() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| ShieldError::Io {
            context: "journal_dir",
            source,
        })?;
    }
    {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|source| ShieldError::Io {
                context: "journal_open",
                source,
            })?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
        }
        for event in events {
            let line = serde_json::to_string(event).map_err(|e| ShieldError::Parse {
                context: "journal_serialize",
                detail: e.to_string(),
            })?;
            writeln!(file, "{line}").map_err(|source| ShieldError::Io {
                context: "journal_write",
                source,
            })?;
        }
        file.flush().map_err(|source| ShieldError::Io {
            context: "journal_flush",
            source,
        })?;
    }
    rotate_journal(path)
}

fn rotate_journal(path: &Path) -> ShieldResult<()> {
    if !path.exists() {
        return Ok(());
    }
    let meta = fs::metadata(path).map_err(|source| ShieldError::Io {
        context: "journal_meta",
        source,
    })?;
    let lines = read_all_lines(path)?;
    let over_lines = lines.len() > MAX_JOURNAL_LINES;
    let over_bytes = meta.len() > MAX_JOURNAL_BYTES;
    if !over_lines && !over_bytes {
        return Ok(());
    }
    let keep = if over_lines {
        lines.len().saturating_sub(MAX_JOURNAL_LINES)
    } else {
        // Drop from the front until under byte budget (approx by re-encoding).
        let mut drop_n = 0_usize;
        let mut size = meta.len();
        for line in &lines {
            if size <= MAX_JOURNAL_BYTES {
                break;
            }
            size = size.saturating_sub(line.len() as u64 + 1);
            drop_n += 1;
        }
        drop_n
    };
    let kept: Vec<String> = lines.into_iter().skip(keep).collect();
    let tmp = path.with_extension("jsonl.tmp");
    {
        let mut out = File::create(&tmp).map_err(|source| ShieldError::Io {
            context: "journal_rotate_create",
            source,
        })?;
        for line in &kept {
            writeln!(out, "{line}").map_err(|source| ShieldError::Io {
                context: "journal_rotate_write",
                source,
            })?;
        }
        out.flush().map_err(|source| ShieldError::Io {
            context: "journal_rotate_flush",
            source,
        })?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&tmp, fs::Permissions::from_mode(0o600));
    }
    fs::rename(&tmp, path).map_err(|source| ShieldError::Io {
        context: "journal_rotate_rename",
        source,
    })?;
    Ok(())
}

fn read_all_lines(path: &Path) -> ShieldResult<Vec<String>> {
    let file = File::open(path).map_err(|source| ShieldError::Io {
        context: "journal_read",
        source,
    })?;
    let reader = BufReader::new(file);
    let mut out = Vec::new();
    for line in reader.lines() {
        let line = line.map_err(|source| ShieldError::Io {
            context: "journal_line",
            source,
        })?;
        if !line.trim().is_empty() {
            out.push(line);
        }
    }
    Ok(out)
}

#[derive(Debug, Clone, Default)]
pub struct JournalQuery {
    pub after_sequence: u64,
    pub before_sequence: Option<u64>,
    pub limit: usize,
    pub kinds: Vec<String>,
    pub severities: Vec<String>,
    pub entity_ip: Option<String>,
    pub q: Option<String>,
    pub group_by: JournalGroupBy,
    pub analytics: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JournalGroupBy {
    #[default]
    None,
    Kind,
    Severity,
    EntityIp,
    Chain,
}

impl JournalGroupBy {
    pub fn parse(raw: Option<&str>) -> ShieldResult<Self> {
        match raw.unwrap_or("none") {
            "" | "none" => Ok(Self::None),
            "kind" => Ok(Self::Kind),
            "severity" => Ok(Self::Severity),
            "entity_ip" => Ok(Self::EntityIp),
            "chain" => Ok(Self::Chain),
            other => Err(ShieldError::InvalidParams(format!(
                "group_by unsupported: {other}"
            ))),
        }
    }
}

pub fn parse_journal_query(params: &Value) -> ShieldResult<JournalQuery> {
    let limit = params
        .get("limit")
        .and_then(Value::as_u64)
        .unwrap_or(50)
        .clamp(1, MAX_JOURNAL_QUERY as u64) as usize;
    let after_sequence = params
        .get("after_sequence")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let before_sequence = params.get("before_sequence").and_then(Value::as_u64);
    let kinds = string_list(params.get("kinds"))?;
    let severities = string_list(params.get("severities"))?;
    let entity_ip = params
        .get("entity_ip")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let q = params
        .get("q")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| {
            if s.len() > 128 {
                s[..128].to_string()
            } else {
                s.to_string()
            }
        });
    let group_by = JournalGroupBy::parse(params.get("group_by").and_then(Value::as_str))?;
    let analytics = params
        .get("analytics")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    Ok(JournalQuery {
        after_sequence,
        before_sequence,
        limit,
        kinds,
        severities,
        entity_ip,
        q,
        group_by,
        analytics,
    })
}

fn string_list(raw: Option<&Value>) -> ShieldResult<Vec<String>> {
    let Some(value) = raw else {
        return Ok(Vec::new());
    };
    let Some(arr) = value.as_array() else {
        return Err(ShieldError::InvalidParams(
            "kinds/severities must be arrays".into(),
        ));
    };
    if arr.len() > 32 {
        return Err(ShieldError::InvalidParams(
            "kinds/severities exceed max 32".into(),
        ));
    }
    let mut out = Vec::with_capacity(arr.len());
    for item in arr {
        let Some(s) = item.as_str() else {
            return Err(ShieldError::InvalidParams(
                "kinds/severities entries must be strings".into(),
            ));
        };
        out.push(s.to_string());
    }
    Ok(out)
}

#[derive(Debug, Clone)]
pub struct JournalQueryResult {
    pub items: Vec<ShieldEvent>,
    pub truncated: bool,
    pub journal_lines: usize,
    pub groups: Vec<JournalGroup>,
    pub analytics: Option<JournalAnalytics>,
}

#[derive(Debug, Clone)]
pub struct JournalGroup {
    pub key: String,
    pub count: usize,
    pub latest_sequence: u64,
}

#[derive(Debug, Clone, Default)]
pub struct JournalAnalytics {
    pub by_severity: JournalSeverityCounts,
    pub by_kind_top: Vec<JournalKindCount>,
    pub top_ips: Vec<JournalIpCount>,
    pub window_from_epoch: u64,
    pub window_to_epoch: u64,
}

#[derive(Debug, Clone, Default)]
pub struct JournalSeverityCounts {
    pub critical: u64,
    pub high: u64,
    pub medium: u64,
    pub low: u64,
}

#[derive(Debug, Clone)]
pub struct JournalKindCount {
    pub kind: String,
    pub count: u64,
}

#[derive(Debug, Clone)]
pub struct JournalIpCount {
    pub ip: String,
    pub count: u64,
}

pub fn query_journal(path: &Path, query: &JournalQuery) -> ShieldResult<JournalQueryResult> {
    let lines = if path.exists() {
        read_all_lines(path)?
    } else {
        Vec::new()
    };
    let journal_lines = lines.len();
    let mut matched = Vec::new();
    for line in &lines {
        let event: ShieldEvent = match serde_json::from_str(line) {
            Ok(e) => e,
            Err(_) => continue,
        };
        if event.sequence <= query.after_sequence {
            continue;
        }
        if let Some(before) = query.before_sequence {
            if event.sequence >= before {
                continue;
            }
        }
        if !query.kinds.is_empty() && !query.kinds.iter().any(|k| k == &event.kind) {
            continue;
        }
        if !query.severities.is_empty() {
            let sev = event.severity.as_str();
            if !query.severities.iter().any(|s| s == sev) {
                continue;
            }
        }
        if let Some(ip) = query.entity_ip.as_deref() {
            if event.entity_ip.as_deref() != Some(ip) {
                continue;
            }
        }
        if let Some(q) = query.q.as_deref() {
            if !event.summary.to_ascii_lowercase().contains(&q.to_ascii_lowercase()) {
                continue;
            }
        }
        matched.push(event);
    }
    let truncated = matched.len() > query.limit;
    let window = if truncated {
        matched[matched.len() - query.limit..].to_vec()
    } else {
        matched
    };
    // Present newest-last window but API returns chronological within page.
    let items = window;
    let analytics = if query.analytics {
        Some(build_analytics(&items))
    } else {
        None
    };
    let groups = build_groups(&items, query.group_by);
    Ok(JournalQueryResult {
        items,
        truncated,
        journal_lines,
        groups,
        analytics,
    })
}

fn build_analytics(items: &[ShieldEvent]) -> JournalAnalytics {
    let mut by_severity = JournalSeverityCounts::default();
    let mut kind_map: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    let mut ip_map: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    let mut from = u64::MAX;
    let mut to = 0_u64;
    for event in items {
        match event.severity {
            Severity::Critical => by_severity.critical += 1,
            Severity::High => by_severity.high += 1,
            Severity::Medium => by_severity.medium += 1,
            Severity::Low => by_severity.low += 1,
        }
        *kind_map.entry(event.kind.clone()).or_default() += 1;
        if let Some(ip) = event.entity_ip.as_ref() {
            *ip_map.entry(ip.clone()).or_default() += 1;
        }
        from = from.min(event.epoch);
        to = to.max(event.epoch);
    }
    if items.is_empty() {
        from = 0;
    }
    let mut by_kind_top: Vec<JournalKindCount> = kind_map
        .into_iter()
        .map(|(kind, count)| JournalKindCount { kind, count })
        .collect();
    by_kind_top.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.kind.cmp(&b.kind)));
    by_kind_top.truncate(5);
    let mut top_ips: Vec<JournalIpCount> = ip_map
        .into_iter()
        .map(|(ip, count)| JournalIpCount { ip, count })
        .collect();
    top_ips.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.ip.cmp(&b.ip)));
    top_ips.truncate(5);
    JournalAnalytics {
        by_severity,
        by_kind_top,
        top_ips,
        window_from_epoch: from,
        window_to_epoch: to,
    }
}

fn build_groups(items: &[ShieldEvent], group_by: JournalGroupBy) -> Vec<JournalGroup> {
    if matches!(group_by, JournalGroupBy::None) {
        return Vec::new();
    }
    let mut map: std::collections::BTreeMap<String, (usize, u64)> =
        std::collections::BTreeMap::new();
    for event in items {
        let key = match group_by {
            JournalGroupBy::None => continue,
            JournalGroupBy::Kind => event.kind.clone(),
            JournalGroupBy::Severity => event.severity.as_str().to_string(),
            JournalGroupBy::EntityIp => event
                .entity_ip
                .clone()
                .unwrap_or_else(|| "—".to_string()),
            JournalGroupBy::Chain => event
                .chain_id
                .clone()
                .unwrap_or_else(|| "—".to_string()),
        };
        let entry = map.entry(key).or_insert((0, 0));
        entry.0 += 1;
        entry.1 = entry.1.max(event.sequence);
    }
    let mut groups: Vec<JournalGroup> = map
        .into_iter()
        .map(|(key, (count, latest_sequence))| JournalGroup {
            key,
            count,
            latest_sequence,
        })
        .collect();
    groups.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.key.cmp(&b.key)));
    groups
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Severity;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tmp() -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("seat_journal_{stamp}"))
    }

    fn ev(seq: u64, kind: &str) -> ShieldEvent {
        ShieldEvent {
            kind: kind.into(),
            severity: Severity::High,
            summary: format!("summary {kind}"),
            entity_ip: Some("198.51.100.20".into()),
            entity_process: None,
            entity_port: None,
            explain_why: "why".into(),
            explain_action: "act".into(),
            confidence_milli: 800,
            tags: vec![],
            sequence: seq,
            epoch: 1_700_000_000 + seq,
            chain_id: None,
        }
    }

    #[test]
    fn append_and_query_roundtrip() {
        let dir = tmp();
        let path = dir.join("journal.jsonl");
        append_journal_lines(&path, &[ev(1, "a"), ev(2, "b")]).unwrap();
        let result = query_journal(
            &path,
            &JournalQuery {
                after_sequence: 0,
                limit: 50,
                analytics: true,
                ..JournalQuery::default()
            },
        )
        .unwrap();
        assert_eq!(result.items.len(), 2);
        assert_eq!(result.journal_lines, 2);
        assert!(result.analytics.is_some());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn rotate_by_line_cap() {
        let dir = tmp();
        let path = dir.join("journal.jsonl");
        let mut batch = Vec::new();
        for i in 1..=(MAX_JOURNAL_LINES as u64 + 10) {
            batch.push(ev(i, "x"));
            if batch.len() == 50 {
                append_journal_lines(&path, &batch).unwrap();
                batch.clear();
            }
        }
        if !batch.is_empty() {
            append_journal_lines(&path, &batch).unwrap();
        }
        let lines = read_all_lines(&path).unwrap();
        assert!(lines.len() <= MAX_JOURNAL_LINES);
        let _ = fs::remove_dir_all(dir);
    }
}
