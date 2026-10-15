use crate::command_runner;
use crate::state_store::atomic_write_file;
use serde::Deserialize;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

const REQUEST_DIR: &str = "/run/empiieros";
const REQUEST_NAME: &str = "display-brightness.request";
const UNIT: &str = "empiieros-display-brightness.service";
const HELPER: &str = "/usr/local/sbin/empiieros-display-brightness";
const BACKLIGHT_ROOT: &str = "/sys/class/backlight";
const MIN_PERCENT: u8 = 10;
const MAX_PERCENT: u8 = 100;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ControlParams {
    action: String,
    name: Option<String>,
    percent: u8,
}

#[derive(Debug, PartialEq, Eq)]
struct BrightnessRequest {
    name: String,
    percent: u8,
}

pub fn execute(params: Value) -> Result<Value, String> {
    let request = parse_request(params)?;
    if !Path::new(HELPER).is_file()
        || !Path::new("/etc/systemd/system").join(UNIT).is_file()
        || !Path::new("/usr/bin/systemctl").is_file()
    {
        return Err("display_backend_unavailable: allowlisted brightness helper is not installed".to_string());
    }
    let device = resolve_device((!request.name.is_empty()).then_some(request.name.as_str()))?;
    if !percent_in_range(request.percent) {
        return Err("percent_must_be_10_to_100".to_string());
    }
    let max = read_u32(&device.max_path)?;
    if max == 0 {
        return Err("backlight_max_brightness_is_zero".to_string());
    }
    let expected = percent_to_value(request.percent, max);
    write_request(&device.name, request.percent)?;
    start_unit()?;
    if !wait_for_value(&device.brightness_path, expected, Duration::from_secs(3)) {
        return Err("display_verification_failed: brightness was not observed".to_string());
    }
    Ok(json!({
        "action": "set_brightness",
        "applied": true,
        "name": device.name,
        "percent": request.percent,
        "observed_brightness": expected,
        "max_brightness": max,
        "source": "sysfs-backlight"
    }))
}

fn parse_request(params: Value) -> Result<BrightnessRequest, String> {
    let params: ControlParams =
        serde_json::from_value(params).map_err(|error| format!("invalid_params: {error}"))?;
    if params.action != "set_brightness" {
        return Err("action_must_be_set_brightness".to_string());
    }
    if !percent_in_range(params.percent) {
        return Err("percent_must_be_10_to_100".to_string());
    }
    if let Some(name) = params.name.as_deref() {
        validate_name(name)?;
    }
    Ok(BrightnessRequest {
        name: params.name.unwrap_or_default(),
        percent: params.percent,
    })
}

fn percent_in_range(percent: u8) -> bool {
    (MIN_PERCENT..=MAX_PERCENT).contains(&percent)
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
        return Err("backlight_name_is_invalid".to_string());
    }
    Ok(())
}

struct BacklightDevice {
    name: String,
    brightness_path: PathBuf,
    max_path: PathBuf,
}

fn resolve_device(requested: Option<&str>) -> Result<BacklightDevice, String> {
    let names = observed_backlight_names()?;
    let name = match requested.filter(|value| !value.is_empty()) {
        Some(value) => {
            validate_name(value)?;
            if !names.iter().any(|item| item == value) {
                return Err("backlight_not_observed".to_string());
            }
            value.to_string()
        }
        None if names.len() == 1 => names[0].clone(),
        None if names.is_empty() => return Err("backlight_not_observed".to_string()),
        None => return Err("backlight_name_required".to_string()),
    };
    let root = Path::new(BACKLIGHT_ROOT).join(&name);
    let resolved = fs::canonicalize(&root).map_err(|_| "backlight_not_observed".to_string())?;
    if !resolved.starts_with("/sys/devices") && !resolved.starts_with(BACKLIGHT_ROOT) {
        return Err("backlight_path_escapes_sysfs".to_string());
    }
    let brightness_path = root.join("brightness");
    let max_path = root.join("max_brightness");
    for path in [&brightness_path, &max_path] {
        let meta = fs::symlink_metadata(path).map_err(|_| "backlight_not_observed".to_string())?;
        if meta.file_type().is_symlink() {
            return Err("backlight_sysfs_must_not_be_symlink".to_string());
        }
    }
    Ok(BacklightDevice {
        name,
        brightness_path,
        max_path,
    })
}

fn observed_backlight_names() -> Result<Vec<String>, String> {
    let entries = fs::read_dir(BACKLIGHT_ROOT)
        .map_err(|_| "backlight_not_observed".to_string())?;
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|_| "backlight_not_observed".to_string())?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if validate_name(&name).is_ok() {
            names.push(name);
        }
    }
    names.sort();
    Ok(names)
}

fn read_u32(path: &Path) -> Result<u32, String> {
    let raw = fs::read_to_string(path)
        .map_err(|error| format!("backlight_sysfs_unreadable: {error}"))?;
    raw.trim()
        .parse()
        .map_err(|_| "backlight_sysfs_unreadable: not an integer".to_string())
}

fn percent_to_value(percent: u8, max: u32) -> u32 {
    u32::try_from(u64::from(percent) * u64::from(max) / 100).unwrap_or(u32::MAX)
}

fn write_request(name: &str, percent: u8) -> Result<(), String> {
    let body = format!("name={name}\npercent={percent}\n");
    atomic_write_file(Path::new(REQUEST_DIR), REQUEST_NAME, body.as_bytes(), 0o600)
        .map_err(|error| format!("display_request_write_failed: {error}"))?;
    Ok(())
}

fn start_unit() -> Result<(), String> {
    let result = command_runner::run(
        "/usr/bin/systemctl",
        &["start", UNIT],
        Duration::from_secs(5),
    )
    .map_err(|error| format!("display_service_spawn_failed: {error}"))?;
    if !result.success {
        return Err(format!(
            "display_service_start_failed: {}",
            command_error(&result)
        ));
    }
    Ok(())
}

fn wait_for_value(path: &Path, expected: u32, timeout: Duration) -> bool {
    let started = Instant::now();
    loop {
        if read_u32(path).ok() == Some(expected) {
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
    fn parse_rejects_dark_and_unknown_fields() {
        assert_eq!(
            parse_request(json!({"action":"set_brightness","percent":75}))
                .unwrap()
                .percent,
            75
        );
        assert_eq!(
            parse_request(json!({"action":"set_brightness","percent":9})).unwrap_err(),
            "percent_must_be_10_to_100"
        );
        assert_eq!(
            parse_request(json!({"action":"set_brightness","percent":0})).unwrap_err(),
            "percent_must_be_10_to_100"
        );
        assert_eq!(
            parse_request(json!({"action":"set_brightness","percent":101})).unwrap_err(),
            "percent_must_be_10_to_100"
        );
        assert_eq!(
            parse_request(json!({"action":"dim","percent":50})).unwrap_err(),
            "action_must_be_set_brightness"
        );
        assert!(parse_request(json!({
            "action":"set_brightness",
            "percent":50,
            "extra":true
        }))
        .is_err());
        assert_eq!(
            parse_request(json!({
                "action":"set_brightness",
                "name":"../intel_backlight",
                "percent":50
            }))
            .unwrap_err(),
            "backlight_name_is_invalid"
        );
    }

    #[test]
    fn percent_math_is_exact_on_the_live_scale() {
        assert_eq!(percent_to_value(75, 120_000), 90_000);
        assert_eq!(percent_to_value(10, 120_000), 12_000);
        assert_eq!(percent_to_value(100, 120_000), 120_000);
    }
}
