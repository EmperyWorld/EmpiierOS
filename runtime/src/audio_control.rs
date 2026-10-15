use crate::command_runner;
use crate::state_store::atomic_write_file;
use serde::Deserialize;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

const REQUEST_DIR: &str = "/run/empiieros";
const REQUEST_NAME: &str = "audio-volume.request";
const RESULT_NAME: &str = "audio-volume.result";
const UNIT: &str = "empiieros-audio-volume.service";
const HELPER: &str = "/usr/local/sbin/empiieros-audio-volume";
const MIN_PERCENT: u8 = 10;
const MAX_PERCENT: u8 = 100;

const ALLOWED_NAMES: &[&str] = &["Speaker", "Headphone", "Master", "PCM"];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ControlParams {
    action: String,
    name: Option<String>,
    percent: u8,
}

#[derive(Debug, PartialEq, Eq)]
struct VolumeRequest {
    name: String,
    percent: u8,
}

pub fn execute(params: Value) -> Result<Value, String> {
    let request = parse_request(params)?;
    if !Path::new(HELPER).is_file()
        || !Path::new("/etc/systemd/system").join(UNIT).is_file()
        || !Path::new("/usr/bin/systemctl").is_file()
    {
        return Err("audio_backend_unavailable: allowlisted volume helper is not installed".to_string());
    }
    write_request(&request.name, request.percent)?;
    let _ = fs::remove_file(Path::new(REQUEST_DIR).join(RESULT_NAME));
    start_unit()?;
    let observed = wait_for_observed(request.percent, Duration::from_secs(3))?;
    Ok(json!({
        "action": "set_volume",
        "applied": true,
        "name": if request.name.is_empty() {
            Value::Null
        } else {
            Value::String(request.name)
        },
        "percent": request.percent,
        "observed_percent": observed,
        "source": "alsa-mixer"
    }))
}

fn parse_request(params: Value) -> Result<VolumeRequest, String> {
    let params: ControlParams =
        serde_json::from_value(params).map_err(|error| format!("invalid_params: {error}"))?;
    if params.action != "set_volume" {
        return Err("action_must_be_set_volume".to_string());
    }
    if !(MIN_PERCENT..=MAX_PERCENT).contains(&params.percent) {
        return Err("percent_must_be_10_to_100".to_string());
    }
    if let Some(name) = params.name.as_deref() {
        if !name.is_empty() && !ALLOWED_NAMES.contains(&name) {
            return Err("audio_control_name_not_allowlisted".to_string());
        }
    }
    Ok(VolumeRequest {
        name: params.name.unwrap_or_default(),
        percent: params.percent,
    })
}

fn write_request(name: &str, percent: u8) -> Result<(), String> {
    let body = format!("name={name}\npercent={percent}\n");
    atomic_write_file(Path::new(REQUEST_DIR), REQUEST_NAME, body.as_bytes(), 0o600)
        .map_err(|error| format!("audio_request_write_failed: {error}"))?;
    Ok(())
}

fn start_unit() -> Result<(), String> {
    let result = command_runner::run(
        "/usr/bin/systemctl",
        &["start", UNIT],
        Duration::from_secs(5),
    )
    .map_err(|error| format!("audio_service_spawn_failed: {error}"))?;
    if !result.success {
        return Err(format!(
            "audio_service_start_failed: {}",
            command_error(&result)
        ));
    }
    Ok(())
}

fn wait_for_observed(expected: u8, timeout: Duration) -> Result<u64, String> {
    let path = Path::new(REQUEST_DIR).join(RESULT_NAME);
    let started = Instant::now();
    loop {
        if let Ok(body) = fs::read_to_string(&path) {
            if let Some(observed) = parse_helper_observed(&body) {
                let delta = observed.abs_diff(u64::from(expected));
                if delta <= 2 {
                    return Ok(observed);
                }
                return Err(format!(
                    "audio_verification_failed: observed_percent={observed} expected={expected}"
                ));
            }
        }
        if started.elapsed() >= timeout {
            return Err("audio_verification_failed: volume was not observed".to_string());
        }
        thread::sleep(Duration::from_millis(50));
    }
}

fn parse_helper_observed(output: &str) -> Option<u64> {
    output.lines().find_map(|line| {
        line.strip_prefix("observed_percent=")
            .and_then(|value| value.trim().parse().ok())
    })
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
    fn parse_rejects_silent_and_unknown_names() {
        assert_eq!(
            parse_request(json!({"action":"set_volume","percent":50}))
                .unwrap()
                .percent,
            50
        );
        assert_eq!(
            parse_request(json!({"action":"set_volume","percent":9})).unwrap_err(),
            "percent_must_be_10_to_100"
        );
        assert_eq!(
            parse_request(json!({"action":"mute","percent":50})).unwrap_err(),
            "action_must_be_set_volume"
        );
        assert_eq!(
            parse_request(json!({
                "action":"set_volume",
                "name":"Capture",
                "percent":50
            }))
            .unwrap_err(),
            "audio_control_name_not_allowlisted"
        );
        assert!(parse_request(json!({
            "action":"set_volume",
            "percent":50,
            "extra":true
        }))
        .is_err());
    }
}
