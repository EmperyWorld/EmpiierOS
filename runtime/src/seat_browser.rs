//! EmpiierOS Seat Browser — Runtime bridge.
//!
//! Policy evaluate + navigate/reload/history host_action + media.set + ≤3 tabs + downloads.
//! Overlay chrome lives in `seat_browser_surface.c`.

use std::path::Path;

use seat_browser_core::{handle_runtime_line, is_seat_browser_op, BrowserService, ServiceConfig};

use crate::protocol::RuntimeResponse;

pub fn handles(op: &str) -> bool {
    is_seat_browser_op(op)
}

pub fn dispatch(state_root: &Path, request_bytes: &[u8]) -> RuntimeResponse {
    let mut service = match BrowserService::open(ServiceConfig {
        state_root: state_root.to_path_buf(),
    }) {
        Ok(svc) => svc,
        Err(error) => {
            return RuntimeResponse::failure(
                "",
                "failed",
                "seat_browser_open_failed",
                error.to_string(),
            );
        }
    };

    let encoded = handle_runtime_line(&mut service, request_bytes);
    let trimmed = encoded.strip_suffix(b"\n").unwrap_or(encoded.as_slice());
    match serde_json::from_slice::<RuntimeResponse>(trimmed) {
        Ok(response) => response,
        Err(error) => RuntimeResponse::failure(
            "",
            "internal_error",
            "seat_browser_response_parse_failed",
            error.to_string(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tmp_root() -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "empiieros-seat-browser-{}-{stamp}",
            std::process::id()
        ))
    }

    #[test]
    fn status_reports_shell_overlay_host() {
        let root = tmp_root();
        let response = dispatch(
            &root,
            br#"{"id":"sb1","op":"seat.browser.status","params":{}}"#,
        );
        assert!(response.ok, "{response:?}");
        assert_eq!(response.data["surface"], "shell_overlay");
        assert_eq!(response.data["host"], "webkit_gtk");
        assert_eq!(response.data["is_default"], true);
        assert!(response.data["allow"].is_array());
        assert_eq!(response.data["allow"].as_array().map(Vec::len), Some(0));
        assert_eq!(response.data["camera"], true);
        assert_eq!(response.data["microphone"], true);
        assert_eq!(response.data["tab_count"], 0);
        assert_eq!(response.data["tab_cap"], 3);
        assert!(response.data["tabs"].is_array());
        assert_eq!(response.data["download_count"], 0);
        assert!(response.data["downloads"].is_array());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn tab_open_blank_then_cap() {
        let root = tmp_root();
        let blank = dispatch(
            &root,
            br#"{"id":"t0","op":"seat.browser.tab.open","params":{}}"#,
        );
        assert!(blank.ok, "{blank:?}");
        assert_eq!(blank.data["tab_id"], 1);
        assert_eq!(blank.data["host_action"], "tab_open");
        let armed = dispatch(
            &root,
            br#"{"id":"arm","op":"seat.browser.policy.set","params":{"mode":"open","confirmation":"ENABLE_BROWSER_POLICY"}}"#,
        );
        assert!(armed.ok, "{armed:?}");
        let two = dispatch(
            &root,
            br#"{"id":"t2","op":"seat.browser.tab.open","params":{"url":"https://example.com/"}}"#,
        );
        assert!(two.ok, "{two:?}");
        let three = dispatch(
            &root,
            br#"{"id":"t3","op":"seat.browser.tab.open","params":{"url":"https://example.com/b"}}"#,
        );
        assert!(three.ok, "{three:?}");
        let four = dispatch(
            &root,
            br#"{"id":"t4","op":"seat.browser.tab.open","params":{"url":"https://example.com/c"}}"#,
        );
        assert!(!four.ok);
        assert_eq!(four.status, "denied");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn download_begin_denied_until_policy_then_busy() {
        let root = tmp_root();
        let denied = dispatch(
            &root,
            br#"{"id":"dl0","op":"seat.browser.download.begin","params":{"url":"https://example.com/a.pdf"}}"#,
        );
        assert!(!denied.ok);
        assert_eq!(denied.status, "denied");
        let armed = dispatch(
            &root,
            br#"{"id":"arm","op":"seat.browser.policy.set","params":{"mode":"open","confirmation":"ENABLE_BROWSER_POLICY"}}"#,
        );
        assert!(armed.ok, "{armed:?}");
        let begin = dispatch(
            &root,
            br#"{"id":"dl1","op":"seat.browser.download.begin","params":{"url":"https://example.com/a.pdf","name":"a.pdf"}}"#,
        );
        assert!(begin.ok, "{begin:?}");
        assert_eq!(begin.data["name"], "a.pdf");
        let busy = dispatch(
            &root,
            br#"{"id":"dl2","op":"seat.browser.download.begin","params":{"url":"https://example.com/b.pdf"}}"#,
        );
        assert!(!busy.ok);
        assert_eq!(busy.status, "denied");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn media_set_empty_is_invalid_and_disable_persists() {
        let root = tmp_root();
        let empty = dispatch(
            &root,
            br#"{"id":"sbm0","op":"seat.browser.media.set","params":{}}"#,
        );
        assert!(!empty.ok);
        assert_eq!(empty.status, "invalid_params");
        let off = dispatch(
            &root,
            br#"{"id":"sbm1","op":"seat.browser.media.set","params":{"camera":false,"microphone":false}}"#,
        );
        assert!(off.ok, "{off:?}");
        let status = dispatch(
            &root,
            br#"{"id":"sbm2","op":"seat.browser.status","params":{}}"#,
        );
        assert_eq!(status.data["camera"], false);
        assert_eq!(status.data["microphone"], false);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn navigate_default_denied() {
        let root = tmp_root();
        let response = dispatch(
            &root,
            br#"{"id":"sb2","op":"seat.browser.navigate","params":{"url":"https://example.com/"}}"#,
        );
        assert!(!response.ok);
        assert_eq!(response.status, "denied");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn evaluate_default_denies_https() {
        let root = tmp_root();
        let response = dispatch(
            &root,
            br#"{"id":"sb3","op":"seat.browser.evaluate","params":{"url":"https://example.com/"}}"#,
        );
        assert!(response.ok, "{response:?}");
        assert_eq!(response.data["allowed"], false);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn reload_returns_host_action() {
        let root = tmp_root();
        let response = dispatch(
            &root,
            br#"{"id":"sb4","op":"seat.browser.reload","params":{}}"#,
        );
        assert!(response.ok, "{response:?}");
        assert_eq!(response.data["host_action"], "reload");
        let _ = fs::remove_dir_all(root);
    }
}
