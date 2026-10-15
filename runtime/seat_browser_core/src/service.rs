//! Seat Browser service — policy/evaluate + navigate gate (S3: Shell loads WebKit overlay).

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use thiserror::Error;

use crate::policy::{
    default_missing_policy, evaluate_navigation_str, now_epoch_ms, validate_policy_for_save,
    DenyReason, NavDecision, PolicyMode, PolicySnapshot, POLICY_TTL_SECONDS,
};
use crate::protocol::{BrowserRequest, BrowserResponse};
use crate::downloads::{
    dest_is_seat_download, ensure_download_dir, file_uri, name_from_url, sanitize_basename,
    sanitize_downloads, unique_dest, DownloadState, MAX_CONCURRENT, MAX_DOWNLOADS,
    MAX_DOWNLOAD_BYTES,
};
use crate::tabs::{bounded_tab_url, sanitize_tabs, tab_host, TabState, MAX_TABS};

pub const CONFIRM_ENABLE_BROWSER_POLICY: &str = "ENABLE_BROWSER_POLICY";
pub const CONFIRM_ENABLE_BROWSER_MEDIA: &str = "ENABLE_BROWSER_MEDIA";
pub const CONFIRM_RESET_SEAT_BROWSER: &str = "RESET_SEAT_BROWSER";

#[derive(Debug, Error)]
pub enum BrowserError {
    #[error("io ({context}): {source}")]
    Io {
        context: &'static str,
        source: std::io::Error,
    },
    #[error("serialize: {0}")]
    Serialize(String),
    #[error("invalid_params: {0}")]
    InvalidParams(String),
    #[error("missing_confirmation expected={expected}")]
    MissingConfirmation { expected: &'static str },
    #[error("not_available: {0}")]
    NotAvailable(String),
    #[error("denied: {0}")]
    Denied(String),
    #[error("operation_not_found: {0}")]
    OperationNotFound(String),
}

pub type BrowserResult<T> = Result<T, BrowserError>;

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub state_root: PathBuf,
}

pub struct BrowserService {
    state_root: PathBuf,
    policy: PolicySnapshot,
    tabs: TabState,
    downloads: DownloadState,
}

impl BrowserService {
    pub fn open(config: ServiceConfig) -> BrowserResult<Self> {
        let dir = config.state_root.join("seat_browser");
        fs::create_dir_all(&dir).map_err(|source| BrowserError::Io {
            context: "seat_browser_mkdir",
            source,
        })?;
        let policy_path = dir.join("policy.json");
        let policy = if policy_path.is_file() {
            let raw = fs::read_to_string(&policy_path).map_err(|source| BrowserError::Io {
                context: "policy_read",
                source,
            })?;
            let mut loaded: PolicySnapshot =
                serde_json::from_str(&raw).map_err(|e| BrowserError::Serialize(e.to_string()))?;
            // Seat-local file is SoT until API fetch exists. Restamp so the
            // 300s EBrowser TTL does not fail-close a saved lab policy.
            loaded.fetched_at_ms = now_epoch_ms();
            write_policy(&policy_path, &loaded)?;
            loaded
        } else {
            let p = default_missing_policy(now_epoch_ms());
            write_policy(&policy_path, &p)?;
            p
        };
        let tabs_path = dir.join("tabs.json");
        let tabs = if tabs_path.is_file() {
            match fs::read_to_string(&tabs_path) {
                Ok(raw) => match serde_json::from_str::<TabState>(&raw) {
                    Ok(loaded) => sanitize_tabs(loaded),
                    Err(_) => TabState::default(),
                },
                Err(_) => TabState::default(),
            }
        } else {
            TabState::default()
        };
        let downloads_path = dir.join("downloads.json");
        let downloads = if downloads_path.is_file() {
            match fs::read_to_string(&downloads_path) {
                Ok(raw) => match serde_json::from_str::<DownloadState>(&raw) {
                    Ok(loaded) => sanitize_downloads(loaded),
                    Err(_) => DownloadState::default(),
                },
                Err(_) => DownloadState::default(),
            }
        } else {
            DownloadState::default()
        };
        Ok(Self {
            state_root: config.state_root,
            policy,
            tabs,
            downloads,
        })
    }

    fn policy_path(&self) -> PathBuf {
        self.state_root.join("seat_browser/policy.json")
    }

    fn tabs_path(&self) -> PathBuf {
        self.state_root.join("seat_browser/tabs.json")
    }

    fn persist_tabs(&self) -> BrowserResult<()> {
        write_tabs(&self.tabs_path(), &self.tabs)
    }

    fn downloads_path(&self) -> PathBuf {
        self.state_root.join("seat_browser/downloads.json")
    }

    fn persist_downloads(&self) -> BrowserResult<()> {
        write_downloads(&self.downloads_path(), &self.downloads)
    }

    fn downloads_observe(&self) -> Value {
        json!({
            "download_count": self.downloads.items.len(),
            "download_cap": MAX_DOWNLOADS,
            "download_max_bytes": MAX_DOWNLOAD_BYTES,
            "downloads": self
                .downloads
                .items
                .iter()
                .map(|item| {
                    json!({
                        "id": item.id,
                        "name": item.name,
                        "host": item.host,
                        "bytes": item.bytes,
                        "phase": item.phase,
                    })
                })
                .collect::<Vec<_>>(),
        })
    }

    fn tabs_observe(&self) -> Value {
        json!({
            "tab_count": self.tabs.items.len(),
            "tab_cap": MAX_TABS,
            "active_tab": self.tabs.active,
            "tabs": self.tabs
                .items
                .iter()
                .map(|tab| {
                    json!({
                        "id": tab.id,
                        "host": tab_host(&tab.url),
                        "url": bounded_tab_url(&tab.url),
                    })
                })
                .collect::<Vec<_>>(),
        })
    }

    fn gated_url(&mut self, url: Option<&str>) -> BrowserResult<String> {
        self.restamp_local_sot()?;
        let chosen = match url {
            None | Some("") => "about:blank".to_string(),
            Some(raw) => raw.to_string(),
        };
        match evaluate_navigation_str(&chosen, &self.policy, now_epoch_ms()) {
            NavDecision::Allow => Ok(chosen),
            NavDecision::Deny(reason) => Err(BrowserError::Denied(deny_reason_str(reason).into())),
        }
    }

    fn tab_err(reason: &str) -> BrowserError {
        if reason == "tab_limit" {
            BrowserError::Denied(reason.into())
        } else {
            BrowserError::InvalidParams(reason.into())
        }
    }

    pub fn handle_bytes(&mut self, request_line: &[u8]) -> BrowserResponse {
        let trimmed = request_line.strip_suffix(b"\n").unwrap_or(request_line);
        let req: BrowserRequest = match serde_json::from_slice(trimmed) {
            Ok(r) => r,
            Err(e) => {
                return BrowserResponse::failure("", "invalid_request", "invalid_json", e.to_string());
            }
        };
        match self.dispatch(req.id.clone(), &req.op, &req.params) {
            Ok(resp) => resp,
            Err(e) => map_error(req.id, e),
        }
    }

    /// Seat-local operator policy is SoT until an API fetch exists.
    /// Restamp on live evaluate/navigate so a long-lived Runtime does not
    /// fail-close VIP allowlist after EBrowser's 300s TTL.
    /// Restamp also on reload/history so a long-lived session stays live.
    fn restamp_local_sot(&mut self) -> BrowserResult<()> {
        if self.policy.is_default {
            return Ok(());
        }
        self.policy.fetched_at_ms = now_epoch_ms();
        write_policy(&self.policy_path(), &self.policy)
    }

    fn dispatch(&mut self, id: String, op: &str, params: &Value) -> BrowserResult<BrowserResponse> {
        match op {
            "seat.browser.status" => Ok(BrowserResponse::success(id, "observed", self.status_json())),
            "seat.browser.policy" => Ok(BrowserResponse::success(id, "observed", self.policy_json())),
            "seat.browser.policy.set" => self.op_policy_set(id, params),
            "seat.browser.evaluate" => self.op_evaluate(id, params),
            "seat.browser.navigate" | "seat.browser.open" => self.op_navigate(id, params),
            "seat.browser.close" => Ok(BrowserResponse::success(
                id,
                "applied",
                json!({"action": "close", "host_action": "hide"}),
            )),
            "seat.browser.reload" => self.op_reload(id),
            "seat.browser.history" => self.op_history(id, params),
            "seat.browser.media.set" => self.op_media_set(id, params),
            "seat.browser.tab.open" => self.op_tab_open(id, params),
            "seat.browser.tab.select" => self.op_tab_select(id, params),
            "seat.browser.tab.close" => self.op_tab_close(id, params),
            "seat.browser.download.begin" => self.op_download_begin(id, params),
            "seat.browser.download.finish" => self.op_download_finish(id, params),
            "seat.browser.reset" => self.op_reset(id, params),
            other => Err(BrowserError::OperationNotFound(other.to_string())),
        }
    }

    fn status_json(&self) -> Value {
        let now = now_epoch_ms();
        let ttl = self.policy.ttl_seconds.max(1).saturating_mul(1000);
        let fresh = now <= self.policy.fetched_at_ms.saturating_add(ttl);
        let mode = match self.policy.mode {
            PolicyMode::Open => "open",
            PolicyMode::Allowlist => "allowlist",
            PolicyMode::Denylist => "denylist",
        };
        json!({
            "product": "empiieros-seat-browser",
            "surface": "shell_overlay",
            "host": "webkit_gtk",
            "policy_mode": mode,
            "policy_fresh": fresh,
            "is_default": self.policy.is_default,
            "allow_count": self.policy.allow.len(),
            "deny_count": self.policy.deny.len(),
            "allow": policy_host_list(&self.policy.allow),
            "deny": policy_host_list(&self.policy.deny),
            "tab_count": self.tabs.items.len(),
            "tab_cap": MAX_TABS,
            "active_tab": self.tabs.active,
            "tabs": self.tabs_observe()["tabs"],
            "download_count": self.downloads.items.len(),
            "download_cap": MAX_DOWNLOADS,
            "download_max_bytes": MAX_DOWNLOAD_BYTES,
            "downloads": self.downloads_observe()["downloads"],
            "camera": self.policy.camera,
            "microphone": self.policy.microphone,
            "detail": if fresh {
                "policy gate live; native overlay chrome via shell.seat_browser_surface"
            } else {
                "policy TTL expired — fail-closed until refresh"
            },
        })
    }

    fn policy_json(&self) -> Value {
        json!({
            "mode": match self.policy.mode {
                PolicyMode::Open => "open",
                PolicyMode::Allowlist => "allowlist",
                PolicyMode::Denylist => "denylist",
            },
            "allow_count": self.policy.allow.len(),
            "deny_count": self.policy.deny.len(),
            "allow": policy_host_list(&self.policy.allow),
            "deny": policy_host_list(&self.policy.deny),
            "is_default": self.policy.is_default,
            "ttl_seconds": self.policy.ttl_seconds,
            "fetched_at_ms": self.policy.fetched_at_ms,
            "camera": self.policy.camera,
            "microphone": self.policy.microphone,
        })
    }

    fn op_evaluate(&mut self, id: String, params: &Value) -> BrowserResult<BrowserResponse> {
        self.restamp_local_sot()?;
        let url = params
            .get("url")
            .and_then(Value::as_str)
            .ok_or_else(|| BrowserError::InvalidParams("params.url required".into()))?;
        let now = now_epoch_ms();
        let decision = evaluate_navigation_str(url, &self.policy, now);
        let (allowed, reason) = match decision {
            NavDecision::Allow => (true, Value::Null),
            NavDecision::Deny(r) => (false, json!(deny_reason_str(r))),
        };
        Ok(BrowserResponse::success(
            id,
            "observed",
            json!({
                "url": url,
                "allowed": allowed,
                "reason": reason,
            }),
        ))
    }


    fn op_reload(&mut self, id: String) -> BrowserResult<BrowserResponse> {
        self.restamp_local_sot()?;
        Ok(BrowserResponse::success(
            id,
            "applied",
            json!({
                "host_action": "reload",
                "tab_id": self.tabs.active,
                "tab_count": self.tabs.items.len(),
            }),
        ))
    }

    fn op_history(&mut self, id: String, params: &Value) -> BrowserResult<BrowserResponse> {
        self.restamp_local_sot()?;
        let direction = params
            .get("direction")
            .and_then(Value::as_str)
            .unwrap_or("");
        let host_action = match direction {
            "back" => "back",
            "forward" => "forward",
            _ => {
                return Err(BrowserError::InvalidParams(
                    "params.direction must be back or forward".into(),
                ));
            }
        };
        Ok(BrowserResponse::success(
            id,
            "applied",
            json!({
                "direction": direction,
                "host_action": host_action,
                "tab_id": self.tabs.active,
                "tab_count": self.tabs.items.len(),
            }),
        ))
    }

    fn op_navigate(&mut self, id: String, params: &Value) -> BrowserResult<BrowserResponse> {
        let url = params
            .get("url")
            .and_then(Value::as_str)
            .ok_or_else(|| BrowserError::InvalidParams("params.url required".into()))?;
        let allowed = self.gated_url(Some(url))?;
        let tab_id = self
            .tabs
            .set_active_url(&allowed)
            .map_err(Self::tab_err)?;
        self.persist_tabs()?;
        Ok(BrowserResponse::success(
            id,
            "applied",
            json!({
                "url": allowed,
                "allowed": true,
                "host_action": "load",
                "tab_id": tab_id,
                "tab_count": self.tabs.items.len(),
                "active_tab": self.tabs.active,
            }),
        ))
    }

    fn op_tab_open(&mut self, id: String, params: &Value) -> BrowserResult<BrowserResponse> {
        let allowed = self.gated_url(params.get("url").and_then(Value::as_str))?;
        let tab_id = self.tabs.open(allowed.clone()).map_err(Self::tab_err)?;
        self.persist_tabs()?;
        Ok(BrowserResponse::success(
            id,
            "applied",
            json!({
                "url": allowed,
                "host_action": "tab_open",
                "tab_id": tab_id,
                "tab_count": self.tabs.items.len(),
                "active_tab": self.tabs.active,
            }),
        ))
    }

    fn op_tab_select(&mut self, id: String, params: &Value) -> BrowserResult<BrowserResponse> {
        self.restamp_local_sot()?;
        let tab_id = params
            .get("id")
            .and_then(Value::as_u64)
            .ok_or_else(|| BrowserError::InvalidParams("params.id required".into()))?;
        let tab_id = u32::try_from(tab_id)
            .map_err(|_| BrowserError::InvalidParams("params.id out of range".into()))?;
        self.tabs.select(tab_id).map_err(Self::tab_err)?;
        self.persist_tabs()?;
        let url = self
            .tabs
            .active_tab()
            .map(|tab| tab.url.clone())
            .unwrap_or_else(|| "about:blank".into());
        Ok(BrowserResponse::success(
            id,
            "applied",
            json!({
                "url": url,
                "host_action": "tab_select",
                "tab_id": tab_id,
                "tab_count": self.tabs.items.len(),
                "active_tab": self.tabs.active,
            }),
        ))
    }

    fn op_tab_close(&mut self, id: String, params: &Value) -> BrowserResult<BrowserResponse> {
        self.restamp_local_sot()?;
        let tab_id = params
            .get("id")
            .and_then(Value::as_u64)
            .ok_or_else(|| BrowserError::InvalidParams("params.id required".into()))?;
        let tab_id = u32::try_from(tab_id)
            .map_err(|_| BrowserError::InvalidParams("params.id out of range".into()))?;
        let remain = self.tabs.close(tab_id).map_err(Self::tab_err)?;
        self.persist_tabs()?;
        let host_action = if remain.is_none() { "hide" } else { "tab_close" };
        Ok(BrowserResponse::success(
            id,
            "applied",
            json!({
                "host_action": host_action,
                "tab_id": tab_id,
                "tab_count": self.tabs.items.len(),
                "active_tab": remain,
            }),
        ))
    }

    fn op_download_begin(&mut self, id: String, params: &Value) -> BrowserResult<BrowserResponse> {
        let url = params
            .get("url")
            .and_then(Value::as_str)
            .ok_or_else(|| BrowserError::InvalidParams("params.url required".into()))?;
        if !url.starts_with("https://") {
            return Err(BrowserError::Denied("scheme".into()));
        }
        let allowed = self.gated_url(Some(url))?;
        let suggested = params
            .get("name")
            .and_then(Value::as_str)
            .filter(|name| !name.is_empty())
            .map(sanitize_basename)
            .unwrap_or_else(|| name_from_url(&allowed));
        let dir = ensure_download_dir(&self.state_root).map_err(|source| BrowserError::Io {
            context: "download_mkdir",
            source,
        })?;
        let dest = unique_dest(&dir, &suggested);
        let dest_name = dest
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("download.bin")
            .to_string();
        let dest_uri = file_uri(&dest);
        if !dest_is_seat_download(&dest_uri, &dest_name) {
            return Err(BrowserError::Denied("download_path".into()));
        }
        let record = self
            .downloads
            .begin(dest_name.clone(), tab_host(&allowed))
            .map_err(|reason| {
                if reason == "download_busy" {
                    BrowserError::Denied(reason.into())
                } else {
                    BrowserError::InvalidParams(reason.into())
                }
            })?;
        self.persist_downloads()?;
        Ok(BrowserResponse::success(
            id,
            "applied",
            json!({
                "download_id": record.id,
                "name": dest_name,
                "host": record.host,
                "dest_uri": dest_uri,
                "max_bytes": MAX_DOWNLOAD_BYTES,
                "phase": record.phase,
                "concurrent_cap": MAX_CONCURRENT,
            }),
        ))
    }

    fn op_download_finish(&mut self, id: String, params: &Value) -> BrowserResult<BrowserResponse> {
        let download_id = params
            .get("id")
            .and_then(Value::as_u64)
            .ok_or_else(|| BrowserError::InvalidParams("params.id required".into()))?;
        let download_id = u32::try_from(download_id)
            .map_err(|_| BrowserError::InvalidParams("params.id out of range".into()))?;
        let ok = params.get("ok").and_then(Value::as_bool).unwrap_or(false);
        let bytes = params.get("bytes").and_then(Value::as_u64).unwrap_or(0);
        let record = self
            .downloads
            .finish(download_id, ok, bytes)
            .map_err(|reason| BrowserError::InvalidParams(reason.into()))?;
        if record.phase != "finished" {
            let path = self
                .state_root
                .join("seat_browser/downloads")
                .join(&record.name);
            if dest_is_seat_download(&file_uri(&path), &record.name) {
                let _ = fs::remove_file(&path);
            }
        }
        self.persist_downloads()?;
        Ok(BrowserResponse::success(
            id,
            "applied",
            json!({
                "download_id": record.id,
                "name": record.name,
                "host": record.host,
                "bytes": record.bytes,
                "phase": record.phase,
            }),
        ))
    }

    fn op_policy_set(&mut self, id: String, params: &Value) -> BrowserResult<BrowserResponse> {
        let tightening = params.get("mode").is_some()
            || params.get("allow").is_some()
            || params.get("deny").is_some();
        if tightening
            && params.get("confirmation").and_then(Value::as_str)
                != Some(CONFIRM_ENABLE_BROWSER_POLICY)
        {
            return Err(BrowserError::MissingConfirmation {
                expected: CONFIRM_ENABLE_BROWSER_POLICY,
            });
        }
        let mut next = self.policy.clone();
        if let Some(mode) = params.get("mode").and_then(Value::as_str) {
            next.mode = match mode {
                "open" => PolicyMode::Open,
                "allowlist" => PolicyMode::Allowlist,
                "denylist" => PolicyMode::Denylist,
                other => {
                    return Err(BrowserError::InvalidParams(format!("unknown mode: {other}")));
                }
            };
        }
        if let Some(arr) = params.get("allow").and_then(Value::as_array) {
            next.allow = arr
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect();
        }
        if let Some(arr) = params.get("deny").and_then(Value::as_array) {
            next.deny = arr
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect();
        }
        next.is_default = false;
        next.fetched_at_ms = now_epoch_ms();
        next.ttl_seconds = POLICY_TTL_SECONDS;
        validate_policy_for_save(&next).map_err(|e| BrowserError::InvalidParams(e.to_string()))?;
        write_policy(&self.policy_path(), &next)?;
        self.policy = next;
        Ok(BrowserResponse::success(id, "applied", self.policy_json()))
    }

    fn op_media_set(&mut self, id: String, params: &Value) -> BrowserResult<BrowserResponse> {
        let camera = params.get("camera").and_then(Value::as_bool);
        let microphone = params.get("microphone").and_then(Value::as_bool);
        if camera.is_none() && microphone.is_none() {
            return Err(BrowserError::InvalidParams(
                "params.camera or params.microphone required".into(),
            ));
        }
        let next_camera = camera.unwrap_or(self.policy.camera);
        let next_microphone = microphone.unwrap_or(self.policy.microphone);
        let enabling = (next_camera && !self.policy.camera)
            || (next_microphone && !self.policy.microphone);
        if enabling
            && params.get("confirmation").and_then(Value::as_str)
                != Some(CONFIRM_ENABLE_BROWSER_MEDIA)
        {
            return Err(BrowserError::MissingConfirmation {
                expected: CONFIRM_ENABLE_BROWSER_MEDIA,
            });
        }
        self.policy.camera = next_camera;
        self.policy.microphone = next_microphone;
        self.policy.is_default = false;
        self.policy.fetched_at_ms = now_epoch_ms();
        write_policy(&self.policy_path(), &self.policy)?;
        Ok(BrowserResponse::success(
            id,
            "applied",
            json!({
                "camera": self.policy.camera,
                "microphone": self.policy.microphone,
            }),
        ))
    }

    fn op_reset(&mut self, id: String, params: &Value) -> BrowserResult<BrowserResponse> {
        if params.get("confirmation").and_then(Value::as_str) != Some(CONFIRM_RESET_SEAT_BROWSER)
        {
            return Err(BrowserError::MissingConfirmation {
                expected: CONFIRM_RESET_SEAT_BROWSER,
            });
        }
        let dir = self.state_root.join("seat_browser");
        if dir.is_dir() {
            fs::remove_dir_all(&dir).map_err(|source| BrowserError::Io {
                context: "seat_browser_reset",
                source,
            })?;
        }
        fs::create_dir_all(&dir).map_err(|source| BrowserError::Io {
            context: "seat_browser_mkdir",
            source,
        })?;
        self.policy = default_missing_policy(now_epoch_ms());
        self.tabs = TabState::default();
        self.downloads = DownloadState::default();
        write_policy(&self.policy_path(), &self.policy)?;
        write_tabs(&self.tabs_path(), &self.tabs)?;
        write_downloads(&self.downloads_path(), &self.downloads)?;
        Ok(BrowserResponse::success(
            id,
            "applied",
            json!({"reset": true, "tab_count": 0, "download_count": 0}),
        ))
    }
}

/// Host suffixes / CIDRs are operator policy, not secrets. Cap the IPC
/// arrays so a huge on-disk list cannot inflate status/policy observe.
const POLICY_HOST_LIST_CAP: usize = 64;

fn policy_host_list(hosts: &[String]) -> Vec<String> {
    hosts
        .iter()
        .map(|host| host.trim())
        .filter(|host| {
            !host.is_empty()
                && host.len() <= 253
                && host.chars().all(|ch| ch.is_ascii_graphic())
        })
        .take(POLICY_HOST_LIST_CAP)
        .map(str::to_string)
        .collect()
}

fn write_policy(path: &Path, policy: &PolicySnapshot) -> BrowserResult<()> {
    let raw = serde_json::to_vec_pretty(policy).map_err(|e| BrowserError::Serialize(e.to_string()))?;
    fs::write(path, raw).map_err(|source| BrowserError::Io {
        context: "policy_write",
        source,
    })
}

fn write_tabs(path: &Path, tabs: &TabState) -> BrowserResult<()> {
    let raw = serde_json::to_vec_pretty(tabs).map_err(|e| BrowserError::Serialize(e.to_string()))?;
    fs::write(path, raw).map_err(|source| BrowserError::Io {
        context: "tabs_write",
        source,
    })
}

fn write_downloads(path: &Path, downloads: &DownloadState) -> BrowserResult<()> {
    let raw =
        serde_json::to_vec_pretty(downloads).map_err(|e| BrowserError::Serialize(e.to_string()))?;
    fs::write(path, raw).map_err(|source| BrowserError::Io {
        context: "downloads_write",
        source,
    })
}

fn deny_reason_str(reason: DenyReason) -> &'static str {
    match reason {
        DenyReason::Expired => "expired",
        DenyReason::Scheme => "scheme",
        DenyReason::Userinfo => "userinfo",
        DenyReason::Port => "port",
        DenyReason::MissingHost => "missing_host",
        DenyReason::AllowlistMiss => "allowlist_miss",
        DenyReason::DenylistHit => "denylist_hit",
        DenyReason::DenylistEmpty => "denylist_empty",
        DenyReason::InvalidUrl => "invalid_url",
    }
}

fn map_error(id: String, err: BrowserError) -> BrowserResponse {
    match err {
        BrowserError::NotAvailable(msg) => {
            BrowserResponse::failure(id, "not_available", "not_available", msg)
        }
        BrowserError::MissingConfirmation { expected } => BrowserResponse::failure(
            id,
            "missing_confirmation",
            "missing_confirmation",
            expected,
        ),
        BrowserError::InvalidParams(msg) => {
            BrowserResponse::failure(id, "invalid_params", "invalid_params", msg)
        }
        BrowserError::OperationNotFound(op) => {
            BrowserResponse::failure(id, "failed", "operation_not_found", op)
        }
        BrowserError::Denied(msg) => BrowserResponse::failure(id, "denied", "denied", msg),
        other => BrowserResponse::failure(id, "failed", "seat_browser_error", other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tmp() -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("seat-browser-svc-{stamp}"))
    }

    #[test]
    fn status_observe_default_deny() {
        let root = tmp();
        let mut svc = BrowserService::open(ServiceConfig {
            state_root: root.clone(),
        })
        .unwrap();
        let resp = svc.handle_bytes(br#"{"id":"1","op":"seat.browser.status","params":{}}"#);
        assert!(resp.ok, "{resp:?}");
        assert_eq!(resp.data["surface"], "shell_overlay");
        assert_eq!(resp.data["is_default"], true);
        assert_eq!(resp.data["allow"], json!([]));
        assert_eq!(resp.data["deny"], json!([]));
        assert_eq!(resp.data["camera"], true);
        assert_eq!(resp.data["microphone"], true);
        assert_eq!(resp.data["tab_count"], 0);
        assert_eq!(resp.data["tab_cap"], 3);
        assert_eq!(resp.data["tabs"], json!([]));
        assert_eq!(resp.data["download_count"], 0);
        assert_eq!(resp.data["downloads"], json!([]));
        let ev = svc.handle_bytes(
            br#"{"id":"2","op":"seat.browser.evaluate","params":{"url":"https://example.com/"}}"#,
        );
        assert!(ev.ok);
        assert_eq!(ev.data["allowed"], false);
        let nav = svc.handle_bytes(
            br#"{"id":"3","op":"seat.browser.navigate","params":{"url":"https://example.com/"}}"#,
        );
        assert!(!nav.ok);
        assert_eq!(nav.status, "denied");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn policy_set_requires_confirm_then_allows() {
        let root = tmp();
        let mut svc = BrowserService::open(ServiceConfig {
            state_root: root.clone(),
        })
        .unwrap();
        let denied = svc.handle_bytes(
            br#"{"id":"a","op":"seat.browser.policy.set","params":{"mode":"open"}}"#,
        );
        assert!(!denied.ok);
        let ok = svc.handle_bytes(
            br#"{"id":"b","op":"seat.browser.policy.set","params":{"mode":"open","confirmation":"ENABLE_BROWSER_POLICY"}}"#,
        );
        assert!(ok.ok, "{ok:?}");
        let ev = svc.handle_bytes(
            br#"{"id":"c","op":"seat.browser.evaluate","params":{"url":"https://example.com/"}}"#,
        );
        assert_eq!(ev.data["allowed"], true);
        let nav = svc.handle_bytes(
            br#"{"id":"d","op":"seat.browser.navigate","params":{"url":"https://example.com/"}}"#,
        );
        assert!(nav.ok, "{nav:?}");
        assert_eq!(nav.data["host_action"], "load");
        assert_eq!(nav.data["tab_id"], 1);
        assert_eq!(nav.data["tab_count"], 1);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn local_policy_restamp_survives_stale_ttl() {
        let root = tmp();
        let dir = root.join("seat_browser");
        fs::create_dir_all(&dir).unwrap();
        let stale = PolicySnapshot {
            mode: PolicyMode::Open,
            allow: Vec::new(),
            deny: Vec::new(),
            fetched_at_ms: 1,
            ttl_seconds: POLICY_TTL_SECONDS,
            is_default: false,
            camera: true,
            microphone: true,
        };
        super::write_policy(&dir.join("policy.json"), &stale).unwrap();
        let mut svc = BrowserService::open(ServiceConfig {
            state_root: root.clone(),
        })
        .unwrap();
        let ev = svc.handle_bytes(
            br#"{"id":"ttl","op":"seat.browser.evaluate","params":{"url":"https://example.com/"}}"#,
        );
        assert!(ev.ok, "{ev:?}");
        assert_eq!(ev.data["allowed"], true);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn policy_observe_returns_allow_hosts() {
        let root = tmp();
        let mut svc = BrowserService::open(ServiceConfig {
            state_root: root.clone(),
        })
        .unwrap();
        let armed = svc.handle_bytes(
            br#"{"id":"arm","op":"seat.browser.policy.set","params":{"mode":"allowlist","allow":["ya.ru","app.eliteew.ru"],"confirmation":"ENABLE_BROWSER_POLICY"}}"#,
        );
        assert!(armed.ok, "{armed:?}");
        assert_eq!(armed.data["allow"], json!(["ya.ru", "app.eliteew.ru"]));
        assert_eq!(armed.data["allow_count"], 2);
        let policy = svc.handle_bytes(br#"{"id":"p","op":"seat.browser.policy","params":{}}"#);
        assert!(policy.ok, "{policy:?}");
        assert_eq!(policy.data["allow"], json!(["ya.ru", "app.eliteew.ru"]));
        let status = svc.handle_bytes(br#"{"id":"s","op":"seat.browser.status","params":{}}"#);
        assert!(status.ok, "{status:?}");
        assert_eq!(status.data["allow"], json!(["ya.ru", "app.eliteew.ru"]));
        assert_eq!(status.data["allow_count"], 2);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn evaluate_restamps_stale_vip_allowlist_in_memory() {
        let root = tmp();
        let mut svc = BrowserService::open(ServiceConfig {
            state_root: root.clone(),
        })
        .unwrap();
        let armed = svc.handle_bytes(
            br#"{"id":"arm","op":"seat.browser.policy.set","params":{"mode":"allowlist","allow":["app.eliteew.ru","meet.eliteew.ru"],"confirmation":"ENABLE_BROWSER_POLICY"}}"#,
        );
        assert!(armed.ok, "{armed:?}");
        svc.policy.fetched_at_ms = 1;
        let meet = svc.handle_bytes(
            br#"{"id":"meet","op":"seat.browser.evaluate","params":{"url":"https://meet.eliteew.ru/ew-gaf-deadbeef?jwt=x"}}"#,
        );
        assert!(meet.ok, "{meet:?}");
        assert_eq!(meet.data["allowed"], true, "{meet:?}");
        assert!(svc.policy.fetched_at_ms > 1);
        let other = svc.handle_bytes(
            br#"{"id":"miss","op":"seat.browser.evaluate","params":{"url":"https://example.com/"}}"#,
        );
        assert_eq!(other.data["allowed"], false);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn default_policy_does_not_restamp_into_open() {
        let root = tmp();
        let mut svc = BrowserService::open(ServiceConfig {
            state_root: root.clone(),
        })
        .unwrap();
        assert!(svc.policy.is_default);
        svc.policy.fetched_at_ms = 1;
        let ev = svc.handle_bytes(
            br#"{"id":"def","op":"seat.browser.evaluate","params":{"url":"https://meet.eliteew.ru/"}}"#,
        );
        assert!(ev.ok, "{ev:?}");
        assert_eq!(ev.data["allowed"], false);
        assert_eq!(ev.data["reason"], "expired");
        assert_eq!(svc.policy.fetched_at_ms, 1);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn reload_and_history_emit_host_action() {
        let root = tmp();
        let mut svc = BrowserService::open(ServiceConfig {
            state_root: root.clone(),
        })
        .unwrap();
        let reload = svc.handle_bytes(br#"{"id":"r","op":"seat.browser.reload","params":{}}"#);
        assert!(reload.ok, "{reload:?}");
        assert_eq!(reload.data["host_action"], "reload");
        let back = svc.handle_bytes(
            br#"{"id":"b","op":"seat.browser.history","params":{"direction":"back"}}"#,
        );
        assert!(back.ok, "{back:?}");
        assert_eq!(back.data["host_action"], "back");
        let forward = svc.handle_bytes(
            br#"{"id":"f","op":"seat.browser.history","params":{"direction":"forward"}}"#,
        );
        assert!(forward.ok, "{forward:?}");
        assert_eq!(forward.data["host_action"], "forward");
        let bad = svc.handle_bytes(
            br#"{"id":"x","op":"seat.browser.history","params":{"direction":"sideways"}}"#,
        );
        assert!(!bad.ok);
        assert_eq!(bad.status, "invalid_params");
        let media = svc.handle_bytes(br#"{"id":"m","op":"seat.browser.media.set","params":{}}"#);
        assert!(!media.ok);
        assert_eq!(media.status, "invalid_params");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn media_set_disables_without_confirm_and_enable_needs_token() {
        let root = tmp();
        let mut svc = BrowserService::open(ServiceConfig {
            state_root: root.clone(),
        })
        .unwrap();
        let off = svc.handle_bytes(
            br#"{"id":"off","op":"seat.browser.media.set","params":{"camera":false,"microphone":false}}"#,
        );
        assert!(off.ok, "{off:?}");
        assert_eq!(off.data["camera"], false);
        assert_eq!(off.data["microphone"], false);
        let denied = svc.handle_bytes(
            br#"{"id":"on","op":"seat.browser.media.set","params":{"camera":true}}"#,
        );
        assert!(!denied.ok);
        assert_eq!(denied.status, "missing_confirmation");
        let on = svc.handle_bytes(
            br#"{"id":"on2","op":"seat.browser.media.set","params":{"camera":true,"confirmation":"ENABLE_BROWSER_MEDIA"}}"#,
        );
        assert!(on.ok, "{on:?}");
        assert_eq!(on.data["camera"], true);
        assert_eq!(on.data["microphone"], false);
        let status = svc.handle_bytes(br#"{"id":"st","op":"seat.browser.status","params":{}}"#);
        assert_eq!(status.data["camera"], true);
        assert_eq!(status.data["microphone"], false);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn old_policy_without_media_fields_defaults_on() {
        let root = tmp();
        let dir = root.join("seat_browser");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("policy.json"),
            r#"{"mode":"allowlist","allow":[],"deny":[],"fetched_at_ms":1,"ttl_seconds":300,"is_default":true}"#,
        )
        .unwrap();
        let mut svc = BrowserService::open(ServiceConfig {
            state_root: root.clone(),
        })
        .unwrap();
        let status = svc.handle_bytes(br#"{"id":"old","op":"seat.browser.status","params":{}}"#);
        assert!(status.ok, "{status:?}");
        assert_eq!(status.data["camera"], true);
        assert_eq!(status.data["microphone"], true);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn tabs_cap_three_and_close_last_hides() {
        let root = tmp();
        let mut svc = BrowserService::open(ServiceConfig {
            state_root: root.clone(),
        })
        .unwrap();
        let armed = svc.handle_bytes(
            br#"{"id":"arm","op":"seat.browser.policy.set","params":{"mode":"open","confirmation":"ENABLE_BROWSER_POLICY"}}"#,
        );
        assert!(armed.ok, "{armed:?}");
        let blank = svc.handle_bytes(br#"{"id":"b","op":"seat.browser.tab.open","params":{}}"#);
        assert!(blank.ok, "{blank:?}");
        assert_eq!(blank.data["url"], "about:blank");
        assert_eq!(blank.data["tab_id"], 1);
        assert_eq!(blank.data["host_action"], "tab_open");
        let second = svc.handle_bytes(
            br#"{"id":"t2","op":"seat.browser.tab.open","params":{"url":"https://app.eliteew.ru/"}}"#,
        );
        assert!(second.ok, "{second:?}");
        assert_eq!(second.data["tab_id"], 2);
        let third = svc.handle_bytes(
            br#"{"id":"t3","op":"seat.browser.tab.open","params":{"url":"https://meet.eliteew.ru/"}}"#,
        );
        assert!(third.ok, "{third:?}");
        assert_eq!(third.data["tab_count"], 3);
        let fourth = svc.handle_bytes(
            br#"{"id":"t4","op":"seat.browser.tab.open","params":{"url":"https://auth.eliteew.ru/"}}"#,
        );
        assert!(!fourth.ok);
        assert_eq!(fourth.status, "denied");
        assert_eq!(fourth.error.as_ref().unwrap().message, "tab_limit");
        let nav = svc.handle_bytes(
            br#"{"id":"n","op":"seat.browser.navigate","params":{"url":"https://auth.eliteew.ru/"}}"#,
        );
        assert!(nav.ok, "{nav:?}");
        assert_eq!(nav.data["tab_id"], 3);
        assert_eq!(nav.data["tab_count"], 3);
        let select = svc.handle_bytes(br#"{"id":"s","op":"seat.browser.tab.select","params":{"id":1}}"#);
        assert!(select.ok, "{select:?}");
        assert_eq!(select.data["host_action"], "tab_select");
        assert_eq!(select.data["tab_id"], 1);
        let close_one = svc.handle_bytes(br#"{"id":"c1","op":"seat.browser.tab.close","params":{"id":1}}"#);
        assert!(close_one.ok, "{close_one:?}");
        assert_eq!(close_one.data["host_action"], "tab_close");
        assert_eq!(close_one.data["tab_count"], 2);
        let hide = svc.handle_bytes(br#"{"id":"h","op":"seat.browser.close","params":{}}"#);
        assert!(hide.ok);
        assert_eq!(hide.data["host_action"], "hide");
        let still = svc.handle_bytes(br#"{"id":"st","op":"seat.browser.status","params":{}}"#);
        assert_eq!(still.data["tab_count"], 2);
        let close_2 = svc.handle_bytes(br#"{"id":"c2","op":"seat.browser.tab.close","params":{"id":2}}"#);
        assert!(close_2.ok);
        let close_3 = svc.handle_bytes(br#"{"id":"c3","op":"seat.browser.tab.close","params":{"id":3}}"#);
        assert!(close_3.ok, "{close_3:?}");
        assert_eq!(close_3.data["host_action"], "hide");
        assert_eq!(close_3.data["tab_count"], 0);
        let empty = svc.handle_bytes(br#"{"id":"e","op":"seat.browser.status","params":{}}"#);
        assert_eq!(empty.data["tab_count"], 0);
        assert_eq!(empty.data["tabs"], json!([]));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn tab_open_https_denied_on_default_and_reset_clears() {
        let root = tmp();
        let mut svc = BrowserService::open(ServiceConfig {
            state_root: root.clone(),
        })
        .unwrap();
        let denied = svc.handle_bytes(
            br#"{"id":"d","op":"seat.browser.tab.open","params":{"url":"https://example.com/"}}"#,
        );
        assert!(!denied.ok);
        assert_eq!(denied.status, "denied");
        let blank = svc.handle_bytes(br#"{"id":"b","op":"seat.browser.tab.open","params":{}}"#);
        assert!(blank.ok, "{blank:?}");
        assert_eq!(blank.data["tab_count"], 1);
        let missing = svc.handle_bytes(br#"{"id":"r0","op":"seat.browser.reset","params":{}}"#);
        assert!(!missing.ok);
        assert_eq!(missing.status, "missing_confirmation");
        let reset = svc.handle_bytes(
            br#"{"id":"r","op":"seat.browser.reset","params":{"confirmation":"RESET_SEAT_BROWSER"}}"#,
        );
        assert!(reset.ok, "{reset:?}");
        let status = svc.handle_bytes(br#"{"id":"s","op":"seat.browser.status","params":{}}"#);
        assert_eq!(status.data["tab_count"], 0);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn download_begin_gates_and_finish_updates() {
        let root = tmp();
        let mut svc = BrowserService::open(ServiceConfig {
            state_root: root.clone(),
        })
        .unwrap();
        let missing = svc.handle_bytes(
            br#"{"id":"d0","op":"seat.browser.download.begin","params":{}}"#,
        );
        assert!(!missing.ok);
        assert_eq!(missing.status, "invalid_params");
        let blank = svc.handle_bytes(
            br#"{"id":"d1","op":"seat.browser.download.begin","params":{"url":"about:blank"}}"#,
        );
        assert!(!blank.ok);
        assert_eq!(blank.status, "denied");
        let denied = svc.handle_bytes(
            br#"{"id":"d2","op":"seat.browser.download.begin","params":{"url":"https://example.com/a.pdf","name":"a.pdf"}}"#,
        );
        assert!(!denied.ok);
        assert_eq!(denied.status, "denied");
        let armed = svc.handle_bytes(
            br#"{"id":"arm","op":"seat.browser.policy.set","params":{"mode":"open","confirmation":"ENABLE_BROWSER_POLICY"}}"#,
        );
        assert!(armed.ok, "{armed:?}");
        let begin = svc.handle_bytes(
            br#"{"id":"d3","op":"seat.browser.download.begin","params":{"url":"https://example.com/a.pdf","name":"a.pdf"}}"#,
        );
        assert!(begin.ok, "{begin:?}");
        assert_eq!(begin.data["name"], "a.pdf");
        assert_eq!(begin.data["download_id"], 1);
        let dest = begin.data["dest_uri"].as_str().unwrap();
        assert!(dest.starts_with("file://"));
        assert!(dest.contains("/seat_browser/downloads/a.pdf"));
        let busy = svc.handle_bytes(
            br#"{"id":"d4","op":"seat.browser.download.begin","params":{"url":"https://example.com/b.pdf"}}"#,
        );
        assert!(!busy.ok);
        assert_eq!(busy.status, "denied");
        let finish = svc.handle_bytes(
            br#"{"id":"d5","op":"seat.browser.download.finish","params":{"id":1,"ok":true,"bytes":12}}"#,
        );
        assert!(finish.ok, "{finish:?}");
        assert_eq!(finish.data["phase"], "finished");
        let status = svc.handle_bytes(br#"{"id":"st","op":"seat.browser.status","params":{}}"#);
        assert_eq!(status.data["download_count"], 1);
        assert_eq!(status.data["downloads"][0]["name"], "a.pdf");
        assert!(status.data["downloads"][0].get("dest_uri").is_none());
        let _ = fs::remove_dir_all(root);
    }
}
