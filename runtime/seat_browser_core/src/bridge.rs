//! Bridge helpers for EmpiierOS Runtime paste (S2).

use crate::service::BrowserService;

pub const SEAT_BROWSER_OPS: &[&str] = &[
    "seat.browser.status",
    "seat.browser.policy",
    "seat.browser.policy.set",
    "seat.browser.evaluate",
    "seat.browser.navigate",
    "seat.browser.open",
    "seat.browser.close",
    "seat.browser.reload",
    "seat.browser.history",
    "seat.browser.media.set",
    "seat.browser.tab.open",
    "seat.browser.tab.select",
    "seat.browser.tab.close",
    "seat.browser.download.begin",
    "seat.browser.download.finish",
    "seat.browser.reset",
];

#[must_use]
pub fn is_seat_browser_op(op: &str) -> bool {
    SEAT_BROWSER_OPS.contains(&op)
}

#[must_use]
pub fn handle_runtime_line(service: &mut BrowserService, request_line: &[u8]) -> Vec<u8> {
    service.handle_bytes(request_line).encode_line()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::ServiceConfig;
    use serde_json::Value;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn recognizes_ops() {
        assert!(is_seat_browser_op("seat.browser.status"));
        assert!(is_seat_browser_op("seat.browser.tab.open"));
        assert!(is_seat_browser_op("seat.browser.tab.select"));
        assert!(is_seat_browser_op("seat.browser.tab.close"));
        assert!(is_seat_browser_op("seat.browser.download.begin"));
        assert!(is_seat_browser_op("seat.browser.download.finish"));
        assert!(!is_seat_browser_op("seat.browser.tabs"));
        assert!(!is_seat_browser_op("seat.shield.status"));
    }

    #[test]
    fn handle_line_runtime_shaped() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("seat_browser_bridge_{stamp}"));
        let mut svc = BrowserService::open(ServiceConfig {
            state_root: root.clone(),
        })
        .unwrap();
        let line = handle_runtime_line(
            &mut svc,
            br#"{"id":"b1","op":"seat.browser.status","params":{}}"#,
        );
        assert_eq!(line.last(), Some(&b'\n'));
        let value: Value = serde_json::from_slice(&line[..line.len() - 1]).unwrap();
        assert_eq!(value["protocol_version"], 1);
        assert_eq!(value["ok"], true);
        let _ = std::fs::remove_dir_all(root);
    }
}
