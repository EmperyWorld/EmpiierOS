mod audio_control;
mod capability_gate;
mod command_runner;
mod display_control;
mod event_log;
mod capsule_v3;
mod device_request;
mod launcher_air;
mod launcher_capsule;
mod launcher_inner;
mod launcher_pin;
mod launcher_portal;
mod launcher_spa;
mod launcher_ota;
mod ota_capsule_store;
mod ota_cutover;
mod launcher_status;
mod launcher_product;
mod launcher_oidc;
mod oidc_urls;
mod oidc_pkce;
mod oidc_dpop;
mod oidc_dpop_iat;
mod oidc_ticket;
mod https_auth;
mod portal_hostmap;
mod spa_pack;
mod spa_assets;
mod https_vip;
mod transport;
mod wg_conf;
mod network_control;
mod observed_domains;
mod power_control;
mod radio_control;
pub mod protocol;
mod seat_shield;
mod seat_browser;
mod state_store;
mod system_probe;
mod time_control;
mod wifi_control;
mod kiosk_tray;
pub mod linkorn;

#[cfg(all(feature = "production", feature = "linkornew"))]
compile_error!(
    "linkornew is a separate runtime profile, not a flag on the FoFline build"
);

pub fn air_enroll_helper_main() {
    if let Err(error) = launcher_air::helper_main() {
        eprintln!("empiieros-air-enroll: {error}");
        process::exit(1);
    }
}

pub fn portal_up_helper_main() {
    if let Err(error) = launcher_portal::helper_main() {
        eprintln!("empiieros-portal-up: {error}");
        process::exit(1);
    }
}

pub fn spa_fetch_helper_main() {
    if let Err(error) = launcher_spa::helper_main() {
        eprintln!("empiieros-spa-fetch: {error}");
        process::exit(1);
    }
}

pub fn ota_fetch_helper_main() {
    if let Err(error) = launcher_ota::helper_main() {
        eprintln!("empiieros-ota-fetch: {error}");
        process::exit(1);
    }
}

use protocol::{RuntimeRequest, RuntimeResponse, MAX_REQUEST_BYTES};
use serde_json::{json, Value};
use state_store::StateStore;
use std::path::PathBuf;
use std::process;
use std::time::Instant;

pub struct RuntimeCore {
    started: Instant,
    start_count: u64,
    state: StateStore,
    events: event_log::EventLog,
}

impl RuntimeCore {
    pub fn new(state_root: PathBuf) -> Result<Self, String> {
        let state = StateStore::open(state_root.clone())
            .map_err(|error| format!("state_store_open_failed: {error}"))?;
        let start_count = state
            .record_start()
            .map_err(|error| format!("start_count_write_failed: {error}"))?;
        launcher_product::ensure_shell_traverse(&state_root)
            .map_err(|error| format!("shell_traverse_failed: {error}"))?;
        Ok(Self {
            started: Instant::now(),
            start_count,
            state,
            events: event_log::EventLog::open(&state_root)?,
        })
    }

    pub fn handle_bytes(&self, bytes: &[u8]) -> RuntimeResponse {
        if bytes.is_empty() {
            return RuntimeResponse::failure(
                "",
                "invalid_request",
                "empty_request",
                "request is empty",
            );
        }
        if bytes.len() > MAX_REQUEST_BYTES {
            return RuntimeResponse::failure(
                "",
                "request_too_large",
                "request_too_large",
                format!("request exceeds {MAX_REQUEST_BYTES} bytes"),
            );
        }

        let request = match serde_json::from_slice::<RuntimeRequest>(bytes) {
            Ok(request) => request,
            Err(error) => {
                return RuntimeResponse::failure(
                    "",
                    "invalid_request",
                    "invalid_json",
                    format!("request JSON is invalid: {error}"),
                );
            }
        };
        if let Err(code) = request.validate() {
            return RuntimeResponse::failure(
                request.id,
                "invalid_request",
                code,
                "request failed protocol validation",
            );
        }
        self.dispatch(request, bytes)
    }

    pub fn record_security_event(&self, operation: &str, detail: &str) {
        let _ = self.state.append_audit(operation, "denied", detail);
    }

    fn dispatch(&self, request: RuntimeRequest, bytes: &[u8]) -> RuntimeResponse {
        let id = request.id.clone();
        #[cfg(feature = "linkornew")]
        if linkorn::closed_on_staff(request.op.as_str()) {
            return RuntimeResponse::failure(
                id,
                "not_available",
                "staff_profile",
                "FoFline license and seat-browser ops are not in the LinkornEW runtime",
            );
        }
        match request.op.as_str() {
            #[cfg(feature = "linkornew")]
            operation if linkorn::handles(operation) => {
                linkorn::dispatch(id, operation, &request.params)
            }
            "runtime.health" => RuntimeResponse::success(
                id,
                "ok",
                json!({
                    "version": env!("CARGO_PKG_VERSION"),
                    "transport": "unix-peercred-v1",
                    "uptime_seconds": self.started.elapsed().as_secs(),
                    "start_count": self.start_count
                }),
            ),
            "system.snapshot" | "diagnostics.snapshot" => {
                self.observed_snapshot(id, &request.params)
            }
            "system.events" => match event_log::events_from_params(&self.events, &request.params) {
                Ok(data) => RuntimeResponse::success(id, "observed", data),
                Err(error) => RuntimeResponse::failure(
                    id,
                    "internal_error",
                    "event_serialization_failed",
                    error,
                ),
            },
            "system.watch" => match event_log::watch_from_params(
                &self.events,
                self.started,
                &request.params,
            ) {
                Ok(data) => RuntimeResponse::success(id, "observed", data),
                Err(error) => RuntimeResponse::failure(
                    id,
                    "invalid_params",
                    "watch_rejected",
                    error,
                ),
            },
            "network.link_snapshot" => {
                let network = system_probe::probe_network();
                let wifi = observed_domains::probe_wifi();
                let associated = wifi
                    .data
                    .associations
                    .iter()
                    .any(|row| !row.ssid.trim().is_empty());
                let chip = system_probe::network_link_chip(&network, associated);
                RuntimeResponse::success(
                    id,
                    "observed",
                    serde_json::to_value(&chip).unwrap_or_else(|_| {
                        json!({"ok": false, "kind": null, "active": false})
                    }),
                )
            }
            "network.configure" => self.mutation(id, "network.configure", || {
                network_control::configure(request.params)
            }),
            "network.wifi.scan" => {
                match wifi_control::scan(request.params) {
                    Ok(data) => RuntimeResponse::success(id, "observed", data),
                    Err(error) => {
                        RuntimeResponse::failure(id, "unavailable", "wifi_scan_failed", error)
                    }
                }
            }
            "network.wifi.connect" => self.mutation(id, "network.wifi.connect", || {
                wifi_control::connect(request.params)
            }),
            "network.wifi.disconnect" => {
                self.mutation(id, "network.wifi.disconnect", || {
                    wifi_control::disconnect(request.params)
                })
            }
            "network.wifi.reconnect" => {
                self.mutation(id, "network.wifi.reconnect", || {
                    wifi_control::reconnect(request.params)
                })
            }
            "network.wifi.radio" => self.mutation(id, "network.wifi.radio", || {
                radio_control::execute(request.params)
            }),
            "system.power" => self.mutation(id, "system.power", || {
                power_control::execute(request.params)
            }),
            "display.control" => self.mutation(id, "display.control", || {
                display_control::execute(request.params)
            }),
            "audio.control" => self.mutation(id, "audio.control", || {
                audio_control::execute(request.params)
            }),
            "time.sync" => self.mutation(id, "time.sync", || time_control::execute(request.params)),
            "launcher.status" => RuntimeResponse::success(
                id,
                "observed",
                launcher_status::observe(self.state.root()),
            ),
            "launcher.capsule_import" => self.mutation(id, "launcher.capsule_import", || {
                launcher_capsule::import(self.state.root(), request.params)
            }),
            "launcher.air_activate" => self.mutation(id, "launcher.air_activate", || {
                launcher_air::activate(self.state.root(), request.params)
            }),
            "launcher.pin_issue" => RuntimeResponse::failure(
                id,
                "not_available",
                "installer_issued_only",
                "launcher.pin_issue is disabled; the installer plants the first-boot PIN",
            ),
            "launcher.pin_unlock" => self.mutation(id, "launcher.pin_unlock", || {
                launcher_pin::unlock(self.state.root(), request.params)
            }),
            "launcher.channel_up" => self.mutation(id, "launcher.channel_up", || {
                launcher_portal::bring_up(self.state.root(), request.params)
            }),
            "launcher.hostmap_apply" => self.mutation(id, "launcher.hostmap_apply", || {
                launcher_portal::apply_hostmap(self.state.root(), request.params)
            }),
            "launcher.spa_sync" => self.mutation(id, "launcher.spa_sync", || {
                launcher_spa::sync(self.state.root(), request.params)
            }),
            "launcher.edge_health" => self.mutation(id, "launcher.edge_health", || {
                launcher_spa::edge_health(self.state.root(), request.params)
            }),
            "launcher.ota_status" => self.mutation(id, "launcher.ota_status", || {
                launcher_ota::status(self.state.root(), request.params)
            }),
            "launcher.ota_request" => self.mutation(id, "launcher.ota_request", || {
                launcher_ota::request(self.state.root(), request.params)
            }),
            "launcher.ota_apply" => self.mutation(id, "launcher.ota_apply", || {
                launcher_ota::apply(self.state.root(), request.params)
            }),
            "launcher.oidc_start" => self.mutation(id, "launcher.oidc_start", || {
                launcher_oidc::start(self.state.root(), request.params)
            }),
            "launcher.oidc_complete" => self.mutation(id, "launcher.oidc_complete", || {
                launcher_oidc::complete(self.state.root(), request.params)
            }),
            "launcher.oidc_inject" => self.mutation(id, "launcher.oidc_inject", || {
                launcher_oidc::inject(self.state.root(), request.params)
            }),
            "launcher.oidc_logout" => self.mutation(id, "launcher.oidc_logout", || {
                launcher_oidc::logout(self.state.root(), request.params)
            }),
            "product.open" => self.mutation(id, "product.open", || {
                launcher_product::open(self.state.root(), request.params)
            }),
            "tray.status" => self.tray_status(id),
            "tray.action" => self.tray_action(id, &request.params),
            "notification.post" => self.notification_post(id, &request.params),
            "notification.list" => self.notification_list(id),
            "notification.dismiss" => self.notification_dismiss(id, &request.params),
            operation if seat_shield::handles(operation) => {
                seat_shield::dispatch(self.state.root(), bytes)
            }
            operation if seat_browser::handles(operation) => {
                if operation != "seat.browser.status" {
                    if let Err(code) = launcher_inner::require_consumed(self.state.root()) {
                        return RuntimeResponse::failure(
                            id,
                            "denied",
                            code.clone(),
                            format!(
                                "seat extras stay closed until FoFline consume ({code})"
                            ),
                        );
                    }
                }
                seat_browser::dispatch(self.state.root(), bytes)
            }
            operation if disabled_operation(operation) => RuntimeResponse::failure(
                id,
                "not_available",
                "capability_not_available",
                format!("{operation} is disabled by the secure-reset production profile"),
            ),
            _ => RuntimeResponse::failure(
                id,
                "not_found",
                "operation_not_found",
                "operation is not registered",
            ),
        }
    }

    fn observed_snapshot(&self, id: String, params: &Value) -> RuntimeResponse {
        let after_sequence = params
            .get("after_sequence")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let mut snapshot = self.events.observe(self.started);
        snapshot.runtime.start_count = self.start_count;
        let batch = self.events.after(after_sequence);
        match serde_json::to_value(snapshot) {
            Ok(mut value) => {
                value["events"] = json!(batch.events);
                value["events_truncated"] = json!(batch.truncated);
                RuntimeResponse::success(id, "observed", value)
            }
            Err(error) => RuntimeResponse::failure(
                id,
                "internal_error",
                "snapshot_serialization_failed",
                error.to_string(),
            ),
        }
    }

    fn mutation(
        &self,
        id: String,
        operation: &'static str,
        execute: impl FnOnce() -> Result<Value, String>,
    ) -> RuntimeResponse {
        if let Err(error) = self
            .state
            .append_audit(operation, "requested", "authorized mutation request")
        {
            return RuntimeResponse::failure(
                id,
                "audit_failed",
                "audit_write_failed",
                format!("operation was not attempted because audit persistence failed: {error}"),
            );
        }

        match execute() {
            Ok(data) => {
                let audit = self.state.append_audit(operation, "applied", "observed success");
                if let Err(error) = audit {
                    return RuntimeResponse::failure(
                        id,
                        "audit_failed",
                        "audit_write_failed",
                        format!("operation was applied but audit persistence failed: {error}"),
                    );
                }
                RuntimeResponse::success(id, "applied", data)
            }
            Err(error) => {
                let _ = self.state.append_audit(operation, "failed", &error);
                RuntimeResponse::failure(id, "failed", "operation_failed", error)
            }
        }
    }

    fn tray_status(&self, id: String) -> RuntimeResponse {
        match kiosk_tray::read_tray_status(self.state.root()) {
            Ok(snap) => {
                let menu_items: Vec<Value> = snap
                    .menu_items
                    .iter()
                    .map(|item| {
                        json!({
                            "id": item.id,
                            "label": item.label,
                            "enabled": item.enabled,
                        })
                    })
                    .collect();
                RuntimeResponse::success(
                    id,
                    "observed",
                    json!({
                        "ok": snap.ok,
                        "status": snap.status,
                        "tray_available": snap.tray_available,
                        "kiosk_quiet_mode": snap.kiosk_quiet_mode,
                        "active_badge_count": snap.active_badge_count,
                        "unread_count": snap.unread_count,
                        "critical_count": snap.critical_count,
                        "total_notifications": snap.total_notifications,
                        "menu_items": menu_items,
                        "detail": snap.detail,
                    }),
                )
            }
            Err(detail) => RuntimeResponse::failure(
                id,
                "failed",
                "tray_status_failed",
                detail,
            ),
        }
    }

    fn tray_action(&self, id: String, params: &Value) -> RuntimeResponse {
        let action = params
            .get("action")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty());
        let Some(action) = action else {
            return RuntimeResponse::failure(
                id,
                "invalid_params",
                "tray_action_missing",
                "tray.action requires non-empty action",
            );
        };
        let param = params
            .get("param")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty());
        match kiosk_tray::execute_tray_action(self.state.root(), action, param) {
            Ok(snap) => RuntimeResponse::success(
                id,
                "applied",
                json!({
                    "ok": snap.ok,
                    "status": snap.status,
                    "action": snap.action,
                    "executed": snap.executed,
                    "kiosk_quiet_mode": snap.kiosk_quiet_mode,
                    "detail": snap.detail,
                }),
            ),
            Err(detail) => RuntimeResponse::failure(
                id,
                "failed",
                "tray_action_failed",
                detail,
            ),
        }
    }

    fn notification_post(&self, id: String, params: &Value) -> RuntimeResponse {
        let title = params
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim();
        let message = params
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim();
        let severity = params
            .get("severity")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty());
        let source = params
            .get("source")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty());
        let action_hint = params
            .get("action_hint")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty());
        match kiosk_tray::post_notification(
            self.state.root(),
            title,
            message,
            severity,
            source,
            action_hint,
        ) {
            Ok(snap) => RuntimeResponse::success(
                id,
                "posted",
                json!({
                    "ok": snap.ok,
                    "status": snap.status,
                    "notification_id": snap.notification_id,
                    "title": snap.title,
                    "severity": snap.severity,
                    "source": snap.source,
                    "queue_size": snap.queue_size,
                    "unread_count": snap.unread_count,
                    "suppressed": snap.suppressed,
                    "detail": snap.detail,
                }),
            ),
            Err(detail) => RuntimeResponse::failure(
                id,
                "failed",
                "notification_post_failed",
                detail,
            ),
        }
    }

    fn notification_list(&self, id: String) -> RuntimeResponse {
        match kiosk_tray::list_notifications(self.state.root()) {
            Ok(snap) => RuntimeResponse::success(
                id,
                "observed",
                json!({
                    "ok": snap.ok,
                    "status": snap.status,
                    "notifications": snap.notifications,
                    "unread_count": snap.unread_count,
                    "critical_count": snap.critical_count,
                    "kiosk_quiet_mode": snap.kiosk_quiet_mode,
                    "detail": snap.detail,
                }),
            ),
            Err(detail) => RuntimeResponse::failure(
                id,
                "failed",
                "notification_list_failed",
                detail,
            ),
        }
    }

    fn notification_dismiss(&self, id: String, params: &Value) -> RuntimeResponse {
        let notification_id = params
            .get("notification_id")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty());
        let clear_all = params
            .get("clear_all")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let mark_read = params
            .get("mark_read")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        match kiosk_tray::dismiss_notification(
            self.state.root(),
            notification_id,
            clear_all,
            mark_read,
        ) {
            Ok(snap) => RuntimeResponse::success(
                id,
                "dismissed",
                json!({
                    "ok": snap.ok,
                    "status": snap.status,
                    "notification_id": snap.notification_id,
                    "cleared_all": snap.cleared_all,
                    "marked_read": snap.marked_read,
                    "count_affected": snap.count_affected,
                    "remaining_count": snap.remaining_count,
                    "unread_count": snap.unread_count,
                    "detail": snap.detail,
                }),
            ),
            Err(detail) => RuntimeResponse::failure(
                id,
                "failed",
                "notification_dismiss_failed",
                detail,
            ),
        }
    }
}

fn disabled_operation(operation: &str) -> bool {
    if operation == "display.control" || operation == "audio.control" || operation == "time.sync" {
        return false;
    }
    const PREFIXES: &[&str] = &[
        "audio.",
        "display.",
        "power.action",
        "input.",
        "time.action",
        "settings.",
        "onboarding.",
        "security.integrity.",
        "operator.auth.",
        "device.keys.",
        "entitlement.",
        "license.",
        "enroll.",
        "update.",
        "factory_reset.",
        "product.",
        "offline.",
        "fs_bridge.",
        "notification.",
        "window.",
        "self_healing.",
        "watchdog.",
        "substrate.image.",
        "substrate.profile.write",
    ];
    PREFIXES
        .iter()
        .any(|prefix| operation.starts_with(prefix))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn core() -> (RuntimeCore, PathBuf) {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "empiieros-core-{}-{nonce}",
            std::process::id()
        ));
        (RuntimeCore::new(root.clone()).expect("core"), root)
    }

    #[test]
    fn snapshot_and_events_share_a_sequence() {
        let (core, root) = core();
        let snapshot =
            core.handle_bytes(br#"{"id":"s1","op":"system.snapshot","params":{}}"#);
        assert!(snapshot.ok);
        assert_eq!(snapshot.status, "observed");
        assert!(snapshot.data["sequence"].as_u64().unwrap_or(0) >= 1);
        assert_eq!(snapshot.data["provenance"]["collector"], "runtime-system-probe");
        assert_eq!(snapshot.data["freshness"]["reason"], "observed_now");
        assert_eq!(snapshot.data["freshness"]["stale"], false);
        assert!(snapshot.data["events"].as_array().is_some_and(|items| !items.is_empty()));
        let events =
            core.handle_bytes(br#"{"id":"e1","op":"system.events","params":{"after_sequence":0}}"#);
        assert!(events.ok);
        assert!(events.data["latest_sequence"].as_u64().unwrap_or(0) >= 1);
        assert!(events.data["events"].as_array().is_some_and(|items| !items.is_empty()));
        let denied = core.handle_bytes(
            br#"{"id":"w1","op":"system.watch","params":{"after_sequence":0,"wait_ms":5001}}"#,
        );
        assert!(!denied.ok);
        assert_eq!(denied.status, "invalid_params");
        let watched = core.handle_bytes(
            br#"{"id":"w2","op":"system.watch","params":{"after_sequence":0,"wait_ms":0}}"#,
        );
        assert!(watched.ok);
        assert!(watched.data["snapshot"]["freshness"]["limit_ms"].as_u64() == Some(8_000));
        let caps = snapshot.data["capabilities"]
            .as_array()
            .expect("capabilities");
        let browser = caps
            .iter()
            .find(|item| item["id"] == "seat.browser")
            .expect("seat.browser capability");
        assert_eq!(browser["availability"], "available");
        assert_eq!(browser["source"], "seat-browser-overlay");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn health_is_real_runtime_liveness() {
        let (core, root) = core();
        let response =
            core.handle_bytes(br#"{"id":"r1","op":"runtime.health","params":{}}"#);
        assert!(response.ok);
        assert_eq!(response.status, "ok");
        assert_eq!(response.data["transport"], "unix-peercred-v1");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn seat_shield_s2_armed_unavailable_via_core() {
        let _guard = crate::seat_shield::HELPER_ENV_LOCK.lock().expect("helper env lock");
        std::env::remove_var("EMPIIEROS_SEAT_SHIELD_FW");
        assert!(!disabled_operation("seat.shield.status"));
        let (core, root) = core();
        let status =
            core.handle_bytes(br#"{"id":"ss1","op":"seat.shield.status","params":{}}"#);
        assert!(status.ok, "{status:?}");
        assert_eq!(status.data["mode"], "unavailable");
        assert_eq!(status.data["backend"], "unavailable");
        assert_eq!(status.data["act_enabled"], true);
        let act = core.handle_bytes(
            br#"{"id":"ss3","op":"seat.shield.act","params":{"action":"block_peer","peer_ip":"198.51.100.20","confirmation":"BLOCK_PEER"}}"#,
        );
        assert!(!act.ok);
        assert_eq!(act.status, "not_available");
        let cycle = core.handle_bytes(
            br#"{"id":"ss4","op":"seat.shield.cycle","params":{"connections":[{"remote_ip":"198.51.100.20","local_port":22,"process_name":"sshd","pid":1}]}}"#,
        );
        assert!(cycle.ok, "{cycle:?}");
        assert!(cycle.data["events"].as_array().is_some_and(|e| !e.is_empty()));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn seat_browser_s3_via_core() {
        assert!(!disabled_operation("seat.browser.status"));
        let (core, root) = core();
        let status =
            core.handle_bytes(br#"{"id":"sb1","op":"seat.browser.status","params":{}}"#);
        assert!(status.ok, "{status:?}");
        assert_eq!(status.data["surface"], "shell_overlay");
        let nav = core.handle_bytes(
            br#"{"id":"sb2","op":"seat.browser.navigate","params":{"url":"https://example.com/"}}"#,
        );
        assert!(!nav.ok);
        assert_eq!(nav.status, "denied");
        assert_eq!(
            nav.error.as_ref().map(|error| error.code.as_str()),
            Some("pin_not_issued")
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn legacy_simulation_is_fail_closed() {
        let (core, root) = core();
        let response = core.handle_bytes(
            br#"{"id":"r2","op":"security.integrity.run","params":{}}"#,
        );
        assert!(!response.ok);
        assert_eq!(response.status, "not_available");
        let product = core.handle_bytes(br#"{"id":"r3","op":"product.open","params":{}}"#);
        assert!(!product.ok);
        assert_eq!(product.status, "failed");
        assert!(product
            .error
            .as_ref()
            .unwrap()
            .message
            .contains("pin_not_issued"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn launcher_status_is_observed_and_not_tauri() {
        assert!(!disabled_operation("launcher.status"));
        assert!(!disabled_operation("launcher.edge_health"));
        assert!(!disabled_operation("launcher.ota_status"));
        assert!(!disabled_operation("launcher.ota_request"));
        assert!(!disabled_operation("launcher.ota_apply"));
        assert!(!disabled_operation("launcher.oidc_start"));
        assert!(!disabled_operation("launcher.oidc_complete"));
        assert!(!disabled_operation("launcher.oidc_logout"));
        let (core, root) = core();
        let response =
            core.handle_bytes(br#"{"id":"l1","op":"launcher.status","params":{}}"#);
        assert!(response.ok);
        assert_eq!(response.status, "observed");
        assert_eq!(response.data["vehicle"], "empiieros-shell-gui");
        assert_eq!(response.data["tauri"], false);
        assert_eq!(response.data["pin_issued"], false);
        assert_eq!(response.data["pin_unlocked"], false);
        assert_eq!(response.data["capsule_imported"], false);
        assert_eq!(response.data["entitlement_present"], false);
        assert_eq!(response.data["channel_present"], false);
        assert_eq!(response.data["channel_valid"], false);
        assert_eq!(response.data["channel_up"], false);
        assert_eq!(response.data["channel_code"], "missing");
        assert_eq!(response.data["product_open"], "unavailable");
        assert_eq!(response.data["oidc_session"], "missing");
        assert_eq!(response.data["portal_hostmap"], "missing");
        let fingerprint = response.data["fingerprint"].as_str().unwrap();
        assert!(fingerprint.starts_with("fp_v2_"), "{fingerprint}");
        let denied = core.handle_bytes(
            br#"{"id":"l2","op":"launcher.capsule_import","params":{"path":"/etc/passwd"}}"#,
        );
        assert!(!denied.ok);
        assert!(denied.error.as_ref().unwrap().message.contains("pin_not_issued"));
        let air = core.handle_bytes(
            br#"{"id":"l3","op":"launcher.air_activate","params":{"code":"not-a-code"}}"#,
        );
        assert!(!air.ok);
        assert!(air.error.as_ref().unwrap().message.contains("pin_not_issued"));
        let portal = core.handle_bytes(br#"{"id":"l3b","op":"launcher.channel_up","params":{}}"#);
        assert!(portal.error.as_ref().unwrap().message.contains("pin_not_issued"));
        let spa = core.handle_bytes(br#"{"id":"l3c","op":"launcher.spa_sync","params":{}}"#);
        assert!(spa.error.as_ref().unwrap().message.contains("pin_not_issued"));
        let ota = core.handle_bytes(br#"{"id":"l3ota","op":"launcher.ota_status","params":{}}"#);
        assert!(ota.error.as_ref().unwrap().message.contains("pin_not_issued"));
        let ota_req = core.handle_bytes(br#"{"id":"l3otar","op":"launcher.ota_request","params":{}}"#);
        assert!(ota_req.error.as_ref().unwrap().message.contains("pin_not_issued"));
        let health = core.handle_bytes(br#"{"id":"l3e","op":"launcher.edge_health","params":{}}"#);
        assert!(health.error.as_ref().unwrap().message.contains("pin_not_issued"));
        let oidc = core.handle_bytes(br#"{"id":"l3f","op":"launcher.oidc_start","params":{}}"#);
        assert!(oidc.error.as_ref().unwrap().message.contains("pin_not_issued"));
        let oidc_out = core.handle_bytes(br#"{"id":"l3g","op":"launcher.oidc_logout","params":{}}"#);
        assert!(oidc_out.error.as_ref().unwrap().message.contains("pin_not_issued"));
        let hostmap = core.handle_bytes(br#"{"id":"l3d","op":"launcher.hostmap_apply","params":{}}"#);
        assert!(hostmap.error.as_ref().unwrap().message.contains("pin_not_issued"));
        assert_eq!(response.data["spa_state"], "denied");
        let refused = core.handle_bytes(br#"{"id":"l4","op":"launcher.pin_issue","params":{}}"#);
        assert!(!refused.ok);
        assert_eq!(refused.status, "not_available");
        assert_eq!(
            refused.error.as_ref().map(|error| error.code.as_str()),
            Some("installer_issued_only")
        );
        let planted =
            super::launcher_pin::issue(&root, serde_json::json!({})).expect("plant like installer");
        let pin = planted["pin"].as_str().expect("pin").to_string();
        assert_eq!(pin.len(), 4);
        let locked_import = core.handle_bytes(
            br#"{"id":"l4b","op":"launcher.capsule_import","params":{"path":"/etc/passwd"}}"#,
        );
        assert!(locked_import.error.as_ref().unwrap().message.contains("pin_not_unlocked"));
        let locked_air = core.handle_bytes(
            br#"{"id":"l4c","op":"launcher.air_activate","params":{"code":"not-a-code"}}"#,
        );
        assert!(locked_air.error.as_ref().unwrap().message.contains("pin_not_unlocked"));
        let unlock_body = format!(
            r#"{{"id":"l5","op":"launcher.pin_unlock","params":{{"pin":"{pin}"}}}}"#
        );
        let unlocked = core.handle_bytes(unlock_body.as_bytes());
        assert!(unlocked.ok, "{unlocked:?}");
        assert_eq!(unlocked.data["unlocked"], true);
        let after = core.handle_bytes(br#"{"id":"l6","op":"launcher.status","params":{}}"#);
        assert_eq!(after.data["pin_issued"], true);
        assert_eq!(after.data["pin_unlocked"], true);
        assert_eq!(after.data["fingerprint_bound"], true);
        assert_eq!(after.data["product_open"], "unavailable");
        let path_denied = core.handle_bytes(
            br#"{"id":"l6b","op":"launcher.capsule_import","params":{"path":"/etc/passwd"}}"#,
        );
        assert!(path_denied.error.as_ref().unwrap().message.contains("path_not_allowed"));
        let air_denied = core.handle_bytes(
            br#"{"id":"l6c","op":"launcher.air_activate","params":{"code":"not-a-code"}}"#,
        );
        assert!(air_denied.error.as_ref().unwrap().message.contains("air_code_structure"));
        let portal = core.handle_bytes(br#"{"id":"l6d","op":"launcher.channel_up","params":{}}"#);
        assert!(portal.error.as_ref().unwrap().message.contains("channel_missing"));
        let spa = core.handle_bytes(br#"{"id":"l6e","op":"launcher.spa_sync","params":{}}"#);
        assert!(spa.error.as_ref().unwrap().message.contains("allow_fofline_required"));
        let health = core.handle_bytes(br#"{"id":"l6g","op":"launcher.edge_health","params":{}}"#);
        assert!(health.error.as_ref().unwrap().message.contains("allow_fofline_required"));
        let oidc = core.handle_bytes(br#"{"id":"l6h","op":"launcher.oidc_start","params":{}}"#);
        assert!(oidc.error.as_ref().unwrap().message.contains("allow_fofline_required"));
        let hostmap = core.handle_bytes(br#"{"id":"l6f","op":"launcher.hostmap_apply","params":{}}"#);
        assert!(hostmap.error.as_ref().unwrap().message.contains("portal_iface_down"));
        assert_eq!(after.data["spa_state"], "denied");
        let product = core.handle_bytes(br#"{"id":"l7","op":"product.open","params":{}}"#);
        assert!(!product.ok);
        assert!(product
            .error
            .as_ref()
            .unwrap()
            .message
            .contains("allow_fofline_required"));
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(not(feature = "linkornew"))]
    #[test]
    fn linkorn_ops_stay_unregistered_on_the_fofline_runtime() {
        let (core, root) = core();
        let console = core.handle_bytes(br#"{"id":"k1","op":"linkorn.console","params":{}}"#);
        assert!(!console.ok);
        assert_eq!(console.status, "not_found");
        let browser = core.handle_bytes(br#"{"id":"k2","op":"linkorn.browser","params":{}}"#);
        assert_eq!(browser.status, "not_found");
        let product = core.handle_bytes(br#"{"id":"k3","op":"product.open","params":{}}"#);
        assert!(product
            .error
            .as_ref()
            .is_some_and(|error| error.message.contains("pin_not_issued")));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn display_control_is_registered_and_fail_closed() {
        assert!(!disabled_operation("display.control"));
        assert!(disabled_operation("display.action"));
        let (core, root) = core();
        let response = core.handle_bytes(
            br#"{"id":"d1","op":"display.control","params":{"action":"set_brightness","percent":0}}"#,
        );
        assert!(!response.ok);
        assert_eq!(response.status, "failed");
        assert!(
            response
                .error
                .as_ref()
                .is_some_and(|error| error.message.contains("percent_must_be_10_to_100"))
        );
        let disabled = core.handle_bytes(
            br#"{"id":"d2","op":"display.action","params":{}}"#,
        );
        assert_eq!(disabled.status, "not_available");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn audio_control_is_registered_and_fail_closed() {
        assert!(!disabled_operation("audio.control"));
        assert!(disabled_operation("audio.action"));
        let (core, root) = core();
        let response = core.handle_bytes(
            br#"{"id":"a1","op":"audio.control","params":{"action":"set_volume","percent":0}}"#,
        );
        assert!(!response.ok);
        assert_eq!(response.status, "failed");
        assert!(
            response
                .error
                .as_ref()
                .is_some_and(|error| error.message.contains("percent_must_be_10_to_100"))
        );
        let disabled = core.handle_bytes(br#"{"id":"a2","op":"audio.action","params":{}}"#);
        assert_eq!(disabled.status, "not_available");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn retired_operation_families_stay_not_available() {
        let (core, root) = core();
        for op in [
            "factory_reset.execute",
            "onboarding.advance",
            "operator.auth.login",
            "device.keys.issue",
            "entitlement.grant",
            "license.activate",
            "enroll.start",
            "update.apply",
            "offline.mount",
            "fs_bridge.copy",
            "window.focus",
            "self_healing.run",
            "watchdog.arm",
            "settings.apply",
            "input.authorize",
            "security.integrity.enable",
            "substrate.image.write",
            "substrate.profile.write",
        ] {
            assert!(disabled_operation(op), "{op}");
            let request = format!(r#"{{"id":"r1","op":"{op}","params":{{}}}}"#);
            let response = core.handle_bytes(request.as_bytes());
            assert!(!response.ok, "{op}");
            assert_eq!(response.status, "not_available", "{op}");
        }
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn time_sync_is_registered_and_fail_closed() {
        assert!(!disabled_operation("time.sync"));
        assert!(disabled_operation("time.action"));
        let (core, root) = core();
        let response = core.handle_bytes(
            br#"{"id":"t1","op":"time.sync","params":{"action":"set_timezone","timezone":"UTC"}}"#,
        );
        assert!(!response.ok);
        assert_eq!(response.status, "failed");
        assert!(
            response
                .error
                .as_ref()
                .is_some_and(|error| error.message.contains("invalid_params")
                    || error.message.contains("action_must_be_set")
                    || error.message.contains("ntp_masked"))
        );
        let disabled = core.handle_bytes(br#"{"id":"t2","op":"time.action","params":{}}"#);
        assert_eq!(disabled.status, "not_available");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn wifi_radio_is_registered_and_rejects_unknown_fields() {
        assert!(!disabled_operation("network.wifi.radio"));
        let (core, root) = core();
        let response = core.handle_bytes(
            br#"{"id":"r1","op":"network.wifi.radio","params":{"action":"block","type":"bluetooth"}}"#,
        );
        assert!(!response.ok);
        assert_eq!(response.status, "failed");
        assert!(
            response
                .error
                .as_ref()
                .is_some_and(|error| error.message.contains("radio_type_must_be_wlan"))
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn wifi_reconnect_is_registered_and_rejects_unknown_fields() {
        assert!(!disabled_operation("network.wifi.reconnect"));
        let (core, root) = core();
        let response = core.handle_bytes(
            br#"{"id":"w1","op":"network.wifi.reconnect","params":{"extra":true}}"#,
        );
        assert!(!response.ok);
        assert_eq!(response.status, "failed");
        assert!(
            response
                .error
                .as_ref()
                .is_some_and(|error| error.message.contains("invalid_params"))
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn system_power_is_registered_and_confirm_gated() {
        assert!(!disabled_operation("system.power"));
        assert!(disabled_operation("power.action"));
        let (core, root) = core();
        let response = core.handle_bytes(
            br#"{"id":"p1","op":"system.power","params":{"action":"reboot","confirm":"NO"}}"#,
        );
        assert!(!response.ok);
        assert_eq!(response.status, "failed");
        assert!(
            response
                .error
                .as_ref()
                .is_some_and(|error| error.message.contains("confirmation_does_not_match_action"))
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn tray_status_and_action_are_registered() {
        assert!(!disabled_operation("tray.status"));
        assert!(!disabled_operation("tray.action"));
        let (core, root) = core();
        let status =
            core.handle_bytes(br#"{"id":"tr1","op":"tray.status","params":{}}"#);
        assert!(status.ok, "{status:?}");
        assert_eq!(status.status, "observed");
        assert_eq!(status.data["tray_available"], true);
        assert_eq!(status.data["ok"], true);
        assert!(status.data["menu_items"].as_array().is_some_and(|m| !m.is_empty()));

        let missing = core.handle_bytes(br#"{"id":"tr2","op":"tray.action","params":{}}"#);
        assert!(!missing.ok);
        assert_eq!(missing.status, "invalid_params");

        let action = core.handle_bytes(
            br#"{"id":"tr3","op":"tray.action","params":{"action":"toggle_quiet_mode"}}"#,
        );
        assert!(action.ok, "{action:?}");
        assert_eq!(action.status, "applied");
        assert_eq!(action.data["action"], "toggle_quiet_mode");
        assert_eq!(action.data["executed"], true);
        assert_eq!(action.data["kiosk_quiet_mode"], true);

        let bad = core.handle_bytes(
            br#"{"id":"tr4","op":"tray.action","params":{"action":"not_allowlisted"}}"#,
        );
        assert!(!bad.ok);
        assert_eq!(bad.status, "failed");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn oversized_and_unknown_requests_fail() {
        let (core, root) = core();
        let oversized = vec![b'x'; MAX_REQUEST_BYTES + 1];
        assert_eq!(core.handle_bytes(&oversized).status, "request_too_large");

        let unknown =
            core.handle_bytes(br#"{"id":"r3","op":"unknown.operation","params":{}}"#);
        assert_eq!(unknown.status, "not_found");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn mutation_is_not_attempted_when_pre_audit_fails() {
        let (core, root) = core();
        fs::write(root.join("audit.jsonl"), vec![b'x'; 8 * 1024 * 1024])
            .expect("fill audit");
        let called = Cell::new(false);
        let response = core.mutation("r4".to_string(), "test.mutation", || {
            called.set(true);
            Ok(json!({"applied": true}))
        });
        assert!(!response.ok);
        assert_eq!(response.status, "audit_failed");
        assert!(!called.get());
        let _ = fs::remove_dir_all(root);
    }
}
