use crate::command_runner;
use crate::observed_domains::RadioRecord;
use crate::state_store::atomic_write_file;
use serde::Deserialize;
use serde_json::{json, Value};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

const REQUEST_DIR: &str = "/run/empiieros";
const REQUEST_NAME: &str = "wifi-radio.request";
const UNIT: &str = "empiieros-wifi-radio.service";
const HELPER: &str = "/usr/local/sbin/empiieros-wifi-radio";
const RFKILL_ROOT: &str = "/sys/class/rfkill";
const ALLOWED_KIND: &str = "wlan";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ControlParams {
    action: String,
    #[serde(rename = "type")]
    kind: Option<String>,
    name: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
struct RadioRequest {
    action: String,
    name: String,
}

pub fn execute(params: Value) -> Result<Value, String> {
    let request = parse_request(params)?;
    if !Path::new(HELPER).is_file()
        || !Path::new("/etc/systemd/system").join(UNIT).is_file()
        || !Path::new("/usr/bin/systemctl").is_file()
    {
        return Err("radio_backend_unavailable: allowlisted wifi radio helper is not installed".to_string());
    }
    let device = resolve_wlan((!request.name.is_empty()).then_some(request.name.as_str()))?;
    if device.hard_blocked {
        return Err("wifi_radio_hard_blocked".to_string());
    }
    write_request(&request.action, &device.name)?;
    start_unit()?;
    let expected_soft = request.action == "block";
    if !wait_for_soft(&device.soft_path, expected_soft, Duration::from_secs(3)) {
        return Err("wifi_radio_verification_failed: rfkill soft state was not observed".to_string());
    }
    Ok(json!({
        "action": request.action,
        "applied": true,
        "name": device.name,
        "kind": ALLOWED_KIND,
        "soft_blocked": expected_soft,
        "hard_blocked": false,
        "source": "sysfs-rfkill"
    }))
}

fn parse_request(params: Value) -> Result<RadioRequest, String> {
    let params: ControlParams =
        serde_json::from_value(params).map_err(|error| format!("invalid_params: {error}"))?;
    if params.action != "block" && params.action != "unblock" {
        return Err("action_must_be_block_or_unblock".to_string());
    }
    match params.kind.as_deref() {
        None | Some(ALLOWED_KIND) => {}
        Some(_) => return Err("radio_type_must_be_wlan".to_string()),
    }
    if let Some(name) = params.name.as_deref() {
        validate_name(name)?;
    }
    Ok(RadioRequest {
        action: params.action,
        name: params.name.unwrap_or_default(),
    })
}

fn validate_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name.len() > 64
        || name.contains('/')
        || name.contains('\\')
        || name.contains("..")
        || !name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.')
        || !name
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_alphanumeric())
    {
        return Err("radio_name_is_invalid".to_string());
    }
    Ok(())
}

struct RadioDevice {
    name: String,
    soft_path: PathBuf,
    hard_blocked: bool,
}

fn resolve_wlan(requested: Option<&str>) -> Result<RadioDevice, String> {
    let radios = observed_wlan()?;
    let radio = match requested.filter(|value| !value.is_empty()) {
        Some(value) => {
            validate_name(value)?;
            radios
                .into_iter()
                .find(|item| item.name == value)
                .ok_or_else(|| "wifi_radio_not_observed".to_string())?
        }
        None => radios
            .into_iter()
            .next()
            .ok_or_else(|| "wifi_radio_not_observed".to_string())?,
    };
    Ok(RadioDevice {
        name: radio.name,
        soft_path: radio.soft_path,
        hard_blocked: radio.hard_blocked,
    })
}

struct ObservedWlan {
    name: String,
    soft_path: PathBuf,
    hard_blocked: bool,
}

fn observed_wlan() -> Result<Vec<ObservedWlan>, String> {
    let mut radios = Vec::new();
    let entries = fs::read_dir(RFKILL_ROOT)
        .map_err(|error| format!("rfkill_sysfs_unavailable: {error}"))?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with("rfkill") {
            continue;
        }
        let path = entry.path();
        let Some(record) = read_radio_dir(&path) else {
            continue;
        };
        if record.kind != ALLOWED_KIND {
            continue;
        }
        radios.push(ObservedWlan {
            name: record.name,
            soft_path: path.join("soft"),
            hard_blocked: record.hard_blocked,
        });
    }
    radios.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(radios)
}

fn read_radio_dir(path: &Path) -> Option<RadioRecord> {
    let kind = read_trimmed(&path.join("type"))?;
    let name = read_trimmed(&path.join("name"))?;
    let soft = read_flag(&path.join("soft"))?;
    let hard = read_flag(&path.join("hard"))?;
    let state = read_trimmed(&path.join("state"))?.parse::<u8>().ok()?;
    Some(RadioRecord {
        name,
        kind,
        soft_blocked: soft,
        hard_blocked: hard,
        state,
    })
}

fn read_trimmed(path: &Path) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn read_flag(path: &Path) -> Option<bool> {
    match read_trimmed(path)?.as_str() {
        "0" => Some(false),
        "1" => Some(true),
        _ => None,
    }
}

fn write_request(action: &str, name: &str) -> Result<(), String> {
    let dir_meta = fs::symlink_metadata(REQUEST_DIR)
        .map_err(|error| format!("radio_request_dir_unavailable: {error}"))?;
    if dir_meta.file_type().is_symlink() || !dir_meta.is_dir() {
        return Err("radio_request_dir_is_not_secure".to_string());
    }
    if dir_meta.uid() != 0 || dir_meta.mode() & 0o022 != 0 {
        return Err("radio_request_dir_has_insecure_owner_or_mode".to_string());
    }
    let body = format!("action={action}\ntype={ALLOWED_KIND}\nname={name}\n");
    atomic_write_file(Path::new(REQUEST_DIR), REQUEST_NAME, body.as_bytes(), 0o600)
        .map(|_| ())
        .map_err(|error| format!("radio_request_write_failed: {error}"))
}

fn start_unit() -> Result<(), String> {
    let result = command_runner::run(
        "/usr/bin/systemctl",
        &["start", UNIT],
        Duration::from_secs(5),
    )
    .map_err(|error| format!("radio_service_spawn_failed: {error}"))?;
    if !result.success {
        return Err(format!(
            "radio_service_start_failed: {}",
            command_error(&result)
        ));
    }
    Ok(())
}

fn wait_for_soft(path: &Path, expected: bool, timeout: Duration) -> bool {
    let started = Instant::now();
    loop {
        if read_flag(path) == Some(expected) {
            return true;
        }
        if started.elapsed() >= timeout {
            return false;
        }
        thread::sleep(Duration::from_millis(50));
    }
}

fn command_error(result: &command_runner::CommandResult) -> String {
    let detail = if result.stderr.trim().is_empty() {
        result.stdout.trim()
    } else {
        result.stderr.trim()
    };
    if result.timed_out {
        "timed out".to_string()
    } else if detail.is_empty() {
        format!("exit {:?}", result.exit_code)
    } else {
        detail.replace(['\r', '\n'], " ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_rejects_non_wlan_and_unknown_fields() {
        assert!(parse_request(json!({"action":"block","type":"bluetooth"}))
            .unwrap_err()
            .contains("radio_type_must_be_wlan"));
        assert!(parse_request(json!({"action":"mute"}))
            .unwrap_err()
            .contains("action_must_be_block_or_unblock"));
        assert!(parse_request(json!({"action":"block","extra":true}))
            .unwrap_err()
            .contains("invalid_params"));
        assert_eq!(
            parse_request(json!({"action":"unblock","type":"wlan","name":"phy1"}))
                .expect("ok"),
            RadioRequest {
                action: "unblock".to_string(),
                name: "phy1".to_string(),
            }
        );
    }

    #[test]
    fn radio_dir_parser_reads_wlan_flags() {
        let root = std::env::temp_dir().join(format!(
            "empiieros-radio-{}",
            std::process::id()
        ));
        let _ = fs::create_dir_all(&root);
        fs::write(root.join("name"), "phy1\n").expect("name");
        fs::write(root.join("type"), "wlan\n").expect("type");
        fs::write(root.join("soft"), "1\n").expect("soft");
        fs::write(root.join("hard"), "0\n").expect("hard");
        fs::write(root.join("state"), "0\n").expect("state");
        let record = read_radio_dir(&root).expect("radio");
        assert_eq!(record.name, "phy1");
        assert_eq!(record.kind, "wlan");
        assert!(record.soft_blocked);
        assert!(!record.hard_blocked);
        assert_eq!(record.state, 0);
        let _ = fs::remove_dir_all(root);
    }
}
