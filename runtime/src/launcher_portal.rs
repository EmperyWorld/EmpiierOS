//! OS-plane portal bring-up. Runtime stays `IPAddressDeny=any`.
//!
//! Helper uses in-tree `wg` + `ip`. Not the quick wrapper. Not userspace WG.

use crate::command_runner;
use crate::launcher_inner;
use crate::state_store::atomic_write_file;
use crate::wg_conf::{self, PortalConf};
use serde_json::{json, Value};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::Duration;

const HELPER: &str = "/usr/local/sbin/empiieros-portal-up";
const UNIT: &str = "empiieros-portal-up.service";
const WG: &str = "/usr/local/sbin/empiieros-wg";
const IP: &str = "/usr/sbin/ip";
const REQUEST_DIR: &str = "/run/empiieros";
const REQUEST_NAME: &str = "portal-up.request";
const RESULT_NAME: &str = "portal-up.result";
const KEY_NAME: &str = "wg-portal.key";
// Portal must not steal the uplink IPv4. Any global inet on a non-portal
// iface is enough — do not pin lab `eno1` or the /30.

pub fn bring_up_block(state_root: &Path) -> Option<&'static str> {
    crate::launcher_pin::access_block(state_root).or_else(|| launcher_inner::channel_block(state_root))
}

pub fn bring_up(state_root: &Path, _params: Value) -> Result<Value, String> {
    if let Some(code) = bring_up_block(state_root) {
        return Err(code.into());
    }
    let mut result = if helper_installed() && !helper_process() {
        activate_via_helper_action(state_root, "bring_up")?
    } else {
        execute_bring_up(state_root)?
    };
    ensure_hostmap_applied(state_root, &mut result)?;
    Ok(result)
}

pub fn apply_hostmap(state_root: &Path, _params: Value) -> Result<Value, String> {
    if let Some(code) = crate::portal_hostmap::hostmap_block(state_root) {
        return Err(code.into());
    }
    if helper_installed() && !helper_process() {
        return activate_via_helper_action(state_root, "hostmap");
    }
    crate::portal_hostmap::apply_for_seat(state_root)
}

pub fn helper_main() -> Result<(), String> {
    let request_path = Path::new(REQUEST_DIR).join(REQUEST_NAME);
    let outcome = match helper_exchange(&request_path) {
        Ok(value) => json!({ "ok": true, "result": value }),
        Err(error) => json!({ "ok": false, "error": error }),
    };
    let _ = fs::remove_file(&request_path);
    atomic_write_file(
        Path::new(REQUEST_DIR),
        RESULT_NAME,
        format!("{outcome}\n").as_bytes(),
        0o600,
    )
    .map_err(|error| format!("portal_result_write:{error}"))?;
    Ok(())
}

fn helper_exchange(request_path: &Path) -> Result<Value, String> {
    let raw = fs::read_to_string(request_path).map_err(|error| format!("portal_request_read:{error}"))?;
    let request: Value =
        serde_json::from_str(&raw).map_err(|error| format!("portal_request_json:{error}"))?;
    let state = request
        .get("state_dir")
        .and_then(Value::as_str)
        .ok_or_else(|| "portal_request_state_dir".to_string())?;
    crate::launcher_pin::require_unlocked(Path::new(state))?;
    match request.get("action").and_then(Value::as_str).unwrap_or("bring_up") {
        "hostmap" => crate::portal_hostmap::apply_for_seat(Path::new(state)),
        _ => execute_bring_up(Path::new(state)),
    }
}

fn helper_process() -> bool {
    matches!(
        std::env::var("EMPIIEROS_PORTAL_HELPER").ok().as_deref(),
        Some("1") | Some("true") | Some("yes")
    )
}

fn helper_installed() -> bool {
    Path::new(HELPER).is_file()
        && Path::new("/etc/systemd/system").join(UNIT).is_file()
        && Path::new("/usr/bin/systemctl").is_file()
}

fn activate_via_helper_action(state_root: &Path, action: &str) -> Result<Value, String> {
    let body = json!({
        "state_dir": state_root.display().to_string(),
        "action": action
    });
    atomic_write_file(
        Path::new(REQUEST_DIR),
        REQUEST_NAME,
        format!("{body}\n").as_bytes(),
        0o600,
    )
    .map_err(|error| format!("portal_request_write_failed:{error}"))?;
    let _ = fs::remove_file(Path::new(REQUEST_DIR).join(RESULT_NAME));
    let started = command_runner::run(
        "/usr/bin/systemctl",
        &["start", UNIT],
        Duration::from_secs(45),
    )
    .map_err(|error| format!("portal_helper_spawn_failed:{error}"))?;
    if !started.success {
        return Err(format!(
            "portal_helper_start_failed: {}",
            started.stderr.trim()
        ));
    }
    let raw = fs::read_to_string(Path::new(REQUEST_DIR).join(RESULT_NAME))
        .map_err(|_| "portal_helper_result_missing".to_string())?;
    if raw.contains("PrivateKey") || raw.contains("wg_conf") {
        return Err("portal_secret_redaction_failed".into());
    }
    let parsed: Value =
        serde_json::from_str(raw.trim()).map_err(|error| format!("portal_helper_result_json:{error}"))?;
    if parsed.get("ok") == Some(&json!(true)) {
        return parsed
            .get("result")
            .cloned()
            .ok_or_else(|| "portal_helper_result_missing".to_string());
    }
    Err(parsed
        .get("error")
        .and_then(Value::as_str)
        .unwrap_or("portal_up_failed")
        .to_string())
}

fn execute_bring_up(state_root: &Path) -> Result<Value, String> {
    if !Path::new(WG).is_file() {
        return Err("empiieros_wg_missing".into());
    }
    if !Path::new(IP).is_file() {
        return Err("ip_missing".into());
    }
    let conf_text = launcher_inner::load_validated_conf(state_root)?;
    let parsed = wg_conf::parse_portal_conf(&conf_text).map_err(|error| error.code.to_string())?;
    persist_conf(state_root, &conf_text)?;
    ensure_iface()?;
    apply_conf(&parsed)?;
    assign_address(&parsed)?;
    add_edge_route()?;
    if !launcher_inner::portal_iface_up() {
        return Err("portal_iface_down".into());
    }
    crate::portal_hostmap::apply_default()?;
    if crate::portal_hostmap::observe_default() != "applied" {
        return Err("portal_hostmap_required".into());
    }
    assert_no_full_tunnel()?;
    assert_uplink_ipv4_survived()?;
    let result = json!({
        "up": true,
        "iface": launcher_inner::PORTAL_IFACE,
        "channel_valid": true,
        "channel_up": true,
        "channel_code": launcher_inner::CHANNEL_VALID_UP,
        "edge": wg_conf::EDGE_IP,
        "portal_hostmap": "applied",
        "detail": "portal split-tunnel up; host-map app/auth to VIP; Runtime IPAddressDeny stays"
    });
    let text = result.to_string();
    if text.contains("PrivateKey") || text.contains(&parsed.private_key) {
        return Err("portal_secret_redaction_failed".into());
    }
    Ok(result)
}

/// Old helpers can leave the iface up without `/etc/hosts`. Repair once
/// before claiming channel_up — the auth WebView needs the VIP map.
fn ensure_hostmap_applied(state_root: &Path, result: &mut Value) -> Result<(), String> {
    if crate::portal_hostmap::observe_default() == "applied" {
        result["portal_hostmap"] = json!("applied");
        return Ok(());
    }
    let repaired = if helper_installed() && !helper_process() {
        activate_via_helper_action(state_root, "hostmap")?
    } else {
        crate::portal_hostmap::apply_for_seat(state_root)?
    };
    if crate::portal_hostmap::observe_default() != "applied" {
        return Err("portal_hostmap_required".into());
    }
    result["portal_hostmap"] = json!("applied");
    if let Some(detail) = repaired.get("detail").and_then(Value::as_str) {
        result["detail"] = json!(format!(
            "portal split-tunnel up; host-map repaired ({detail})"
        ));
    }
    Ok(())
}

fn persist_conf(state_root: &Path, conf_text: &str) -> Result<(), String> {
    let dir = state_root.join("wireguard");
    fs::create_dir_all(&dir).map_err(|error| format!("portal_conf_store:{error}"))?;
    let mut permissions = fs::metadata(&dir)
        .map_err(|error| format!("portal_conf_store:{error}"))?
        .permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&dir, permissions)
        .map_err(|error| format!("portal_conf_store:{error}"))?;
    atomic_write_file(&dir, "wg-portal.conf", conf_text.as_bytes(), 0o600)
        .map_err(|error| format!("portal_conf_store:{error}"))?;
    Ok(())
}

fn ensure_iface() -> Result<(), String> {
    let sys = Path::new("/sys/class/net").join(launcher_inner::PORTAL_IFACE);
    if sys.is_dir() {
        return Ok(());
    }
    let added = run_ip(&[
        "link",
        "add",
        "name",
        launcher_inner::PORTAL_IFACE,
        "type",
        "wireguard",
    ])?;
    if !added.success {
        let err = added.stderr.to_ascii_lowercase();
        if err.contains("unknown device type") || err.contains("operation not supported") {
            return Err("kernel_wg_missing".into());
        }
        return Err(format!("portal_link_add: {}", added.stderr.trim()));
    }
    Ok(())
}

fn apply_conf(parsed: &PortalConf) -> Result<(), String> {
    let key_path = Path::new(REQUEST_DIR).join(KEY_NAME);
    atomic_write_file(
        Path::new(REQUEST_DIR),
        KEY_NAME,
        format!("{}\n", parsed.private_key).as_bytes(),
        0o600,
    )
    .map_err(|error| format!("portal_key_write:{error}"))?;
    let mut args = vec![
        "set".to_string(),
        launcher_inner::PORTAL_IFACE.to_string(),
        "private-key".to_string(),
        key_path.display().to_string(),
        "peer".to_string(),
        parsed.peer_public_key.clone(),
        "allowed-ips".to_string(),
        parsed.allowed_ips.clone(),
        "endpoint".to_string(),
        parsed.endpoint.clone(),
    ];
    if let Some(keep) = parsed.keepalive {
        args.push("persistent-keepalive".into());
        args.push(keep.to_string());
    }
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let applied = command_runner::run(WG, &arg_refs, Duration::from_secs(8));
    let _ = fs::remove_file(&key_path);
    let applied = applied.map_err(|error| format!("portal_wg_set:{error}"))?;
    if !applied.success {
        return Err(format!("portal_wg_set: {}", applied.stderr.trim()));
    }
    Ok(())
}

fn assign_address(parsed: &PortalConf) -> Result<(), String> {
    let _ = run_ip(&["addr", "flush", "dev", launcher_inner::PORTAL_IFACE])?;
    let added = run_ip(&[
        "addr",
        "add",
        &parsed.address,
        "dev",
        launcher_inner::PORTAL_IFACE,
    ])?;
    if !added.success && !added.stderr.contains("File exists") {
        return Err(format!("portal_addr: {}", added.stderr.trim()));
    }
    let mtu = parsed.mtu.to_string();
    let up = run_ip(&[
        "link",
        "set",
        "dev",
        launcher_inner::PORTAL_IFACE,
        "mtu",
        &mtu,
        "up",
    ])?;
    if !up.success {
        return Err(format!("portal_link_up: {}", up.stderr.trim()));
    }
    Ok(())
}

fn add_edge_route() -> Result<(), String> {
    let dest = format!("{}/32", wg_conf::EDGE_IP);
    let added = run_ip(&[
        "route",
        "replace",
        &dest,
        "dev",
        launcher_inner::PORTAL_IFACE,
    ])?;
    if !added.success {
        return Err(format!("portal_route: {}", added.stderr.trim()));
    }
    Ok(())
}

fn assert_uplink_ipv4_survived() -> Result<(), String> {
    let shown = run_ip(&["-4", "-o", "addr", "show", "scope", "global"])?;
    if !shown.success {
        return Err("portal_broke_uplink".into());
    }
    if !uplink_has_global_ipv4(&shown.stdout, launcher_inner::PORTAL_IFACE) {
        return Err("portal_broke_uplink".into());
    }
    Ok(())
}

/// `ip -4 -o addr show scope global` — any non-loopback, non-portal inet.
pub(crate) fn uplink_has_global_ipv4(ip_o_stdout: &str, portal_iface: &str) -> bool {
    for line in ip_o_stdout.lines() {
        let line = line.trim();
        if !line.contains(" inet ") {
            continue;
        }
        let Some((_, rest)) = line.split_once(':') else {
            continue;
        };
        let name = rest.split_whitespace().next().unwrap_or("");
        if name.is_empty() || name == "lo" || name == portal_iface {
            continue;
        }
        if name.len() <= 16
            && name
                .chars()
                .all(|item| item.is_ascii_alphanumeric() || item == '_' || item == '-')
        {
            return true;
        }
    }
    false
}

fn assert_no_full_tunnel() -> Result<(), String> {
    let routes = run_ip(&["-4", "route", "show", "default"])?;
    if routes.stdout.contains(launcher_inner::PORTAL_IFACE) {
        return Err("portal_default_route_forbidden".into());
    }
    Ok(())
}

fn run_ip(args: &[&str]) -> Result<crate::command_runner::CommandResult, String> {
    command_runner::run(IP, args, Duration::from_secs(8))
        .map_err(|error| format!("portal_ip:{error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("empiieros-portal-{nanos}"));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn bring_up_requires_unlock_and_valid_conf() {
        let root = temp_root();
        assert_eq!(
            bring_up(&root, json!({})).unwrap_err(),
            "pin_not_issued"
        );
        crate::launcher_pin::plant_unlocked(&root);
        assert_eq!(
            bring_up(&root, json!({})).unwrap_err(),
            "channel_missing"
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn uplink_accepts_wifi_or_ethernet_and_ignores_portal() {
        assert!(uplink_has_global_ipv4(
            "2: wlp58s0    inet 192.168.38.82/24 brd 192.168.38.255 scope global\n",
            "wg-portal",
        ));
        assert!(uplink_has_global_ipv4(
            "2: eno1    inet 192.168.8.10/30 brd 192.168.8.11 scope global eno1\n",
            "wg-portal",
        ));
        assert!(uplink_has_global_ipv4(
            "4: wg-portal    inet 10.88.89.101/32 scope global\n\
             3: wlp58s0    inet 192.168.38.82/24 scope global\n",
            "wg-portal",
        ));
        assert!(!uplink_has_global_ipv4(
            "4: wg-portal    inet 10.88.89.101/32 scope global wg-portal\n",
            "wg-portal",
        ));
        assert!(!uplink_has_global_ipv4("", "wg-portal"));
        assert!(!uplink_has_global_ipv4(
            "1: lo    inet 127.0.0.1/8 scope host lo\n",
            "wg-portal",
        ));
    }
}
