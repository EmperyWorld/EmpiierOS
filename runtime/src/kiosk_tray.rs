//! System Tray & Kiosk Notification Coordination (P72).
//!
//! Provides native tray menu coordination, bounded notification queues,
//! kiosk quiet mode enforcement, and auditable event logging under
//! `{state_dir}/kiosk_tray/`.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use serde_json::{json, Value};

pub const KIOSK_TRAY_DIR: &str = "kiosk_tray";
pub const NOTIFICATIONS_FILE: &str = "notifications.json";
pub const TRAY_STATE_FILE: &str = "tray_state.json";
pub const TRAY_AUDIT_FILE: &str = "tray_audit.json";

pub const MAX_QUEUED_NOTIFICATIONS: usize = 50;
pub const MAX_TITLE_LEN: usize = 128;
pub const MAX_MESSAGE_LEN: usize = 512;
pub const MAX_AUDIT_ENTRIES: usize = 200;

pub const ALLOWED_ACTIONS: &[&str] = &[
    "focus_shell",
    "toggle_quiet_mode",
    "request_quick_panel",
    "clear_notifications",
    "request_restart",
];

pub const ALLOWED_SEVERITIES: &[&str] = &["info", "warning", "critical"];

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn kiosk_tray_dir(state: &Path) -> PathBuf {
    let dir = state.join(KIOSK_TRAY_DIR);
    if !dir.is_dir() {
        let _ = std::fs::create_dir_all(&dir);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700));
        }
    }
    dir
}

fn notifications_path(state: &Path) -> PathBuf {
    kiosk_tray_dir(state).join(NOTIFICATIONS_FILE)
}

fn tray_state_path(state: &Path) -> PathBuf {
    kiosk_tray_dir(state).join(TRAY_STATE_FILE)
}

fn tray_audit_path(state: &Path) -> PathBuf {
    kiosk_tray_dir(state).join(TRAY_AUDIT_FILE)
}

fn write_file_0600(path: &Path, content: &str) -> Result<(), String> {
    std::fs::write(path, content).map_err(|e| format!("write_failed: {e}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

fn read_tray_state(state: &Path) -> (bool, Option<String>, Option<u64>) {
    let p = tray_state_path(state);
    if !p.is_file() {
        return (false, None, None);
    }
    if let Ok(content) = std::fs::read_to_string(&p) {
        if let Ok(v) = serde_json::from_str::<Value>(&content) {
            let quiet = v.get("kiosk_quiet_mode").and_then(|x| x.as_bool()).unwrap_or(false);
            let last_act = v.get("last_action").and_then(|x| x.as_str()).map(|s| s.to_string());
            let last_at = v.get("last_action_at").and_then(|x| x.as_u64());
            return (quiet, last_act, last_at);
        }
    }
    (false, None, None)
}

fn write_tray_state(state: &Path, quiet_mode: bool, last_action: Option<&str>) -> Result<(), String> {
    let p = tray_state_path(state);
    let payload = json!({
        "kiosk_quiet_mode": quiet_mode,
        "last_action": last_action,
        "last_action_at": now_unix()
    });
    write_file_0600(&p, &payload.to_string())
}

fn read_notifications(state: &Path) -> Vec<Value> {
    let p = notifications_path(state);
    if !p.is_file() {
        return Vec::new();
    }
    if let Ok(content) = std::fs::read_to_string(&p) {
        if let Ok(Value::Array(arr)) = serde_json::from_str::<Value>(&content) {
            return arr;
        }
    }
    Vec::new()
}

fn write_notifications(state: &Path, notifs: &[Value]) -> Result<(), String> {
    let p = notifications_path(state);
    let payload = Value::Array(notifs.to_vec());
    write_file_0600(&p, &payload.to_string())
}

fn append_audit_log(state: &Path, event_type: &str, details: Value) -> Result<(), String> {
    let p = tray_audit_path(state);
    let mut entries: Vec<Value> = if p.is_file() {
        std::fs::read_to_string(&p)
            .ok()
            .and_then(|c| serde_json::from_str::<Value>(&c).ok())
            .and_then(|v| match v {
                Value::Array(a) => Some(a),
                _ => None,
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let entry = json!({
        "timestamp": now_unix(),
        "event": event_type,
        "details": details
    });

    entries.push(entry);
    if entries.len() > MAX_AUDIT_ENTRIES {
        let excess = entries.len() - MAX_AUDIT_ENTRIES;
        entries.drain(0..excess);
    }

    let payload = Value::Array(entries);
    write_file_0600(&p, &payload.to_string())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayMenuItemRecord {
    pub id: String,
    pub label: String,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct TrayStatusSnapshot {
    pub ok: bool,
    pub status: String,
    pub tray_available: bool,
    pub kiosk_quiet_mode: bool,
    pub active_badge_count: usize,
    pub unread_count: usize,
    pub critical_count: usize,
    pub total_notifications: usize,
    pub menu_items: Vec<TrayMenuItemRecord>,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub struct TrayActionSnapshot {
    pub ok: bool,
    pub status: String,
    pub action: String,
    pub executed: bool,
    pub kiosk_quiet_mode: bool,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub struct NotificationPostSnapshot {
    pub ok: bool,
    pub status: String,
    pub notification_id: String,
    pub title: String,
    pub severity: String,
    pub source: String,
    pub queue_size: usize,
    pub unread_count: usize,
    pub suppressed: bool,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub struct NotificationListSnapshot {
    pub ok: bool,
    pub status: String,
    pub notifications: Vec<Value>,
    pub unread_count: usize,
    pub critical_count: usize,
    pub kiosk_quiet_mode: bool,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub struct NotificationDismissSnapshot {
    pub ok: bool,
    pub status: String,
    pub notification_id: Option<String>,
    pub cleared_all: bool,
    pub marked_read: bool,
    pub count_affected: usize,
    pub remaining_count: usize,
    pub unread_count: usize,
    pub detail: String,
}

pub fn read_tray_status(state: &Path) -> Result<TrayStatusSnapshot, String> {
    let (quiet_mode, _last_act, _last_at) = read_tray_state(state);
    let notifs = read_notifications(state);

    let mut unread_count = 0;
    let mut critical_count = 0;

    for item in &notifs {
        let is_read = item.get("read").and_then(|r| r.as_bool()).unwrap_or(false);
        let sev = item.get("severity").and_then(|s| s.as_str()).unwrap_or("info");
        if !is_read {
            unread_count += 1;
            if sev == "critical" {
                critical_count += 1;
            }
        }
    }

    let menu_items = vec![
        TrayMenuItemRecord {
            id: "focus_shell".into(),
            label: "Фокус Shell".into(),
            enabled: true,
        },
        TrayMenuItemRecord {
            id: "toggle_quiet_mode".into(),
            label: if quiet_mode {
                "Выключить тихий режим".into()
            } else {
                "Включить тихий режим".into()
            },
            enabled: true,
        },
        TrayMenuItemRecord {
            id: "request_quick_panel".into(),
            label: "Панель состояния".into(),
            enabled: true,
        },
        TrayMenuItemRecord {
            id: "clear_notifications".into(),
            label: "Очистить уведомления".into(),
            enabled: !notifs.is_empty(),
        },
        TrayMenuItemRecord {
            id: "request_restart".into(),
            label: "Перезагрузка киоска".into(),
            enabled: true,
        },
    ];

    Ok(TrayStatusSnapshot {
        ok: true,
        status: "ok".into(),
        tray_available: true,
        kiosk_quiet_mode: quiet_mode,
        active_badge_count: unread_count,
        unread_count,
        critical_count,
        total_notifications: notifs.len(),
        menu_items,
        detail: format!(
            "tray.status · available, quiet_mode={quiet_mode}, unread={unread_count}, total={}",
            notifs.len()
        ),
    })
}

pub fn execute_tray_action(
    state: &Path,
    action: &str,
    param: Option<&str>,
) -> Result<TrayActionSnapshot, String> {
    let clean_action = action.trim();
    if !ALLOWED_ACTIONS.contains(&clean_action) {
        return Err(format!("unsupported_tray_action: {clean_action}"));
    }

    let (mut quiet_mode, _, _) = read_tray_state(state);
    let mut detail_msg = String::new();

    match clean_action {
        "focus_shell" => {
            detail_msg = "tray.action · focus_shell dispatched".into();
        }
        "toggle_quiet_mode" => {
            quiet_mode = !quiet_mode;
            detail_msg = format!("tray.action · toggle_quiet_mode switched to {quiet_mode}");
        }
        "request_quick_panel" => {
            detail_msg = "tray.action · request_quick_panel dispatched".into();
        }
        "clear_notifications" => {
            let notifs = read_notifications(state);
            let count = notifs.len();
            let _ = write_notifications(state, &[]);
            detail_msg = format!("tray.action · clear_notifications wiped {count} items");
        }
        "request_restart" => {
            detail_msg = "tray.action · request_restart signal registered for lifecycle".into();
        }
        _ => {}
    }

    write_tray_state(state, quiet_mode, Some(clean_action))?;
    append_audit_log(
        state,
        "tray_action",
        json!({
            "action": clean_action,
            "param": param,
            "quiet_mode_now": quiet_mode,
        }),
    )?;

    Ok(TrayActionSnapshot {
        ok: true,
        status: "ok".into(),
        action: clean_action.into(),
        executed: true,
        kiosk_quiet_mode: quiet_mode,
        detail: detail_msg,
    })
}

pub fn post_notification(
    state: &Path,
    title: &str,
    message: &str,
    severity: Option<&str>,
    source: Option<&str>,
    action_hint: Option<&str>,
) -> Result<NotificationPostSnapshot, String> {
    let clean_title = title.trim();
    if clean_title.is_empty() {
        return Err("notification_title_empty".into());
    }
    if clean_title.len() > MAX_TITLE_LEN {
        return Err(format!("notification_title_too_long: max {MAX_TITLE_LEN} chars"));
    }

    let clean_msg = message.trim();
    if clean_msg.is_empty() {
        return Err("notification_message_empty".into());
    }
    if clean_msg.len() > MAX_MESSAGE_LEN {
        return Err(format!("notification_message_too_long: max {MAX_MESSAGE_LEN} chars"));
    }

    let clean_sev = severity.unwrap_or("info").trim().to_lowercase();
    if !ALLOWED_SEVERITIES.contains(&clean_sev.as_str()) {
        return Err(format!("invalid_severity: {clean_sev}; allowed: info, warning, critical"));
    }

    let clean_source = source.unwrap_or("system").trim();
    let (quiet_mode, _, _) = read_tray_state(state);

    // Kiosk suppression policy: info and warning suppressed during quiet mode; critical always penetrates
    let suppressed = quiet_mode && clean_sev != "critical";

    let mut notifs = read_notifications(state);

    // Bounded capacity check
    if notifs.len() >= MAX_QUEUED_NOTIFICATIONS {
        // Evict oldest read item, or oldest item if all unread
        let evict_idx = notifs
            .iter()
            .position(|n| n.get("read").and_then(|r| r.as_bool()).unwrap_or(false))
            .unwrap_or(0);
        notifs.remove(evict_idx);
    }

    let notif_id = format!("notif_{}_{:04x}", now_unix(), notifs.len() as u16);
    let record = json!({
        "id": notif_id,
        "title": clean_title,
        "message": clean_msg,
        "severity": clean_sev,
        "source": clean_source,
        "created_at": now_unix(),
        "read": false,
        "suppressed": suppressed,
        "action_hint": action_hint.map(|h| h.trim())
    });

    notifs.push(record);
    write_notifications(state, &notifs)?;

    let unread_count = notifs
        .iter()
        .filter(|n| !n.get("read").and_then(|r| r.as_bool()).unwrap_or(false))
        .count();

    append_audit_log(
        state,
        "notification_posted",
        json!({
            "id": notif_id,
            "title": clean_title,
            "severity": clean_sev,
            "suppressed": suppressed
        }),
    )?;

    Ok(NotificationPostSnapshot {
        ok: true,
        status: "ok".into(),
        notification_id: notif_id,
        title: clean_title.into(),
        severity: clean_sev,
        source: clean_source.into(),
        queue_size: notifs.len(),
        unread_count,
        suppressed,
        detail: format!(
            "notification.post · queued, queue_size={}, unread={}, suppressed={suppressed}",
            notifs.len(),
            unread_count
        ),
    })
}

pub fn list_notifications(state: &Path) -> Result<NotificationListSnapshot, String> {
    let (quiet_mode, _, _) = read_tray_state(state);
    let notifs = read_notifications(state);

    let mut unread_count = 0;
    let mut critical_count = 0;

    for item in &notifs {
        let is_read = item.get("read").and_then(|r| r.as_bool()).unwrap_or(false);
        let sev = item.get("severity").and_then(|s| s.as_str()).unwrap_or("info");
        if !is_read {
            unread_count += 1;
            if sev == "critical" {
                critical_count += 1;
            }
        }
    }

    Ok(NotificationListSnapshot {
        ok: true,
        status: "ok".into(),
        notifications: notifs,
        unread_count,
        critical_count,
        kiosk_quiet_mode: quiet_mode,
        detail: format!("notification.list · unread={unread_count}, critical={critical_count}, quiet={quiet_mode}"),
    })
}

pub fn dismiss_notification(
    state: &Path,
    notification_id: Option<&str>,
    clear_all: bool,
    mark_read: bool,
) -> Result<NotificationDismissSnapshot, String> {
    let mut notifs = read_notifications(state);
    let mut count_affected = 0;
    let mut target_id = None;

    if clear_all {
        count_affected = notifs.len();
        notifs.clear();
        write_notifications(state, &notifs)?;
    } else if let Some(id) = notification_id {
        let clean_id = id.trim();
        target_id = Some(clean_id.to_string());
        if mark_read {
            for item in &mut notifs {
                if item.get("id").and_then(|v| v.as_str()) == Some(clean_id) {
                    if let Some(obj) = item.as_object_mut() {
                        obj.insert("read".into(), json!(true));
                        count_affected += 1;
                    }
                }
            }
        } else {
            let before = notifs.len();
            notifs.retain(|item| item.get("id").and_then(|v| v.as_str()) != Some(clean_id));
            count_affected = before.saturating_sub(notifs.len());
        }
        write_notifications(state, &notifs)?;
    } else {
        return Err("notification_id_or_clear_all_required".into());
    }

    let unread_count = notifs
        .iter()
        .filter(|n| !n.get("read").and_then(|r| r.as_bool()).unwrap_or(false))
        .count();

    append_audit_log(
        state,
        "notification_dismissed",
        json!({
            "target_id": target_id,
            "clear_all": clear_all,
            "mark_read": mark_read,
            "count_affected": count_affected
        }),
    )?;

    Ok(NotificationDismissSnapshot {
        ok: true,
        status: "ok".into(),
        notification_id: target_id,
        cleared_all: clear_all,
        marked_read: mark_read,
        count_affected,
        remaining_count: notifs.len(),
        unread_count,
        detail: format!(
            "notification.dismiss · affected={count_affected}, remaining={}, unread={unread_count}",
            notifs.len()
        ),
    })
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn temp_test_dir(prefix: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("empiieros_tray_test_{prefix}_{}", now_unix()));
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    #[test]
    fn status_initial_empty_queue() {
        let dir = temp_test_dir("status_empty");
        let st = read_tray_status(&dir).expect("tray status");
        assert!(st.ok);
        assert_eq!(st.status, "ok");
        assert!(st.tray_available);
        assert!(!st.kiosk_quiet_mode);
        assert_eq!(st.unread_count, 0);
        assert_eq!(st.total_notifications, 0);
        assert_eq!(st.menu_items.len(), 5);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn post_and_list_lifecycle_with_quiet_mode() {
        let dir = temp_test_dir("post_quiet");

        // 1. Post info while quiet mode is false
        let p1 = post_notification(&dir, "System Update", "Channel stable ready", Some("info"), Some("update"), None)
            .expect("post 1");
        assert!(p1.ok);
        assert!(!p1.suppressed);
        assert_eq!(p1.unread_count, 1);

        // 2. Toggle quiet mode
        let act = execute_tray_action(&dir, "toggle_quiet_mode", None).expect("toggle quiet");
        assert!(act.executed);
        assert!(act.kiosk_quiet_mode);

        // 3. Post warning while quiet mode is true -> suppressed
        let p2 = post_notification(&dir, "Disk Warning", "Staging reaching 50%", Some("warning"), Some("storage"), None)
            .expect("post 2");
        assert!(p2.ok);
        assert!(p2.suppressed);

        // 4. Post critical while quiet mode is true -> penetrates quiet mode (not suppressed)
        let p3 = post_notification(&dir, "Security Alert", "Integrity check failed", Some("critical"), Some("security"), None)
            .expect("post 3");
        assert!(p3.ok);
        assert!(!p3.suppressed);

        // 5. List notifications
        let list = list_notifications(&dir).expect("list notifs");
        assert_eq!(list.notifications.len(), 3);
        assert_eq!(list.unread_count, 3);
        assert_eq!(list.critical_count, 1);
        assert!(list.kiosk_quiet_mode);
        let suppressed: Vec<bool> = list
            .notifications
            .iter()
            .map(|n| n.get("suppressed").and_then(|v| v.as_bool()).expect("suppressed flag"))
            .collect();
        assert_eq!(suppressed, vec![false, true, false]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn post_rejects_empty_or_excessive_length() {
        let dir = temp_test_dir("post_validation");
        assert!(post_notification(&dir, "", "valid msg", None, None, None).is_err());
        assert!(post_notification(&dir, "valid title", "", None, None, None).is_err());
        let long_title = "a".repeat(150);
        assert!(post_notification(&dir, &long_title, "valid msg", None, None, None).is_err());
        let long_msg = "b".repeat(600);
        assert!(post_notification(&dir, "valid title", &long_msg, None, None, None).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn post_rejects_invalid_severity() {
        let dir = temp_test_dir("post_bad_sev");
        let res = post_notification(&dir, "title", "msg", Some("urgent_emergency"), None, None);
        assert!(res.is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn tray_action_allowlist_and_rejects_unallowlisted() {
        let dir = temp_test_dir("tray_action");
        let act = execute_tray_action(&dir, "focus_shell", None).expect("focus shell");
        assert!(act.executed);
        assert_eq!(act.action, "focus_shell");

        let bad = execute_tray_action(&dir, "rm_rf_system", None);
        assert!(bad.is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn dismiss_single_and_clear_all() {
        let dir = temp_test_dir("dismiss");
        let p1 = post_notification(&dir, "A", "msg A", None, None, None).expect("post A");
        let p2 = post_notification(&dir, "B", "msg B", None, None, None).expect("post B");

        // Mark single read
        let d1 = dismiss_notification(&dir, Some(&p1.notification_id), false, true).expect("mark read");
        assert_eq!(d1.count_affected, 1);
        assert_eq!(d1.remaining_count, 2);
        assert_eq!(d1.unread_count, 1);

        // Delete single
        let d2 = dismiss_notification(&dir, Some(&p2.notification_id), false, false).expect("delete single");
        assert_eq!(d2.count_affected, 1);
        assert_eq!(d2.remaining_count, 1);

        // Clear all
        let d3 = dismiss_notification(&dir, None, true, false).expect("clear all");
        assert_eq!(d3.count_affected, 1);
        assert_eq!(d3.remaining_count, 0);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn queue_capacity_limit_evicts_oldest() {
        let dir = temp_test_dir("capacity");
        for i in 0..52 {
            let _ = post_notification(&dir, &format!("Notif {i}"), "msg", None, None, None).expect("post");
        }
        let list = list_notifications(&dir).expect("list");
        assert_eq!(list.notifications.len(), MAX_QUEUED_NOTIFICATIONS);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
