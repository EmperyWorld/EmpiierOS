use crate::command_runner;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PowerParams {
    action: String,
    confirm: String,
}

#[derive(Debug, PartialEq, Eq)]
struct PowerRequest {
    action: &'static str,
    unit: &'static str,
}

pub fn execute(params: Value) -> Result<Value, String> {
    let request = parse_request(params)?;
    if !Path::new("/etc/systemd/system").join(request.unit).is_file() {
        return Err("power_backend_unavailable: allowlisted power unit is not installed".to_string());
    }
    if !Path::new("/usr/bin/systemctl").is_file() {
        return Err("power_backend_unavailable: systemctl is not installed".to_string());
    }
    let result = command_runner::run(
        "/usr/bin/systemctl",
        &["start", request.unit],
        Duration::from_secs(8),
    )
    .map_err(|error| format!("power_service_spawn_failed: {error}"))?;
    if !result.success {
        return Err(format!(
            "power_service_start_failed: {}",
            command_error(&result)
        ));
    }
    Ok(json!({
        "action": request.action,
        "unit": request.unit,
        "scheduled": true,
        "source": "systemd-isolate"
    }))
}

fn parse_request(params: Value) -> Result<PowerRequest, String> {
    let params: PowerParams =
        serde_json::from_value(params).map_err(|error| format!("invalid_params: {error}"))?;
    match params.action.as_str() {
        "reboot" if params.confirm == "REBOOT" => Ok(PowerRequest {
            action: "reboot",
            unit: "empiieros-power-reboot.service",
        }),
        "poweroff" if params.confirm == "POWEROFF" => Ok(PowerRequest {
            action: "poweroff",
            unit: "empiieros-power-poweroff.service",
        }),
        "reboot" | "poweroff" => Err("confirmation_does_not_match_action".to_string()),
        _ => Err("action_must_be_reboot_or_poweroff".to_string()),
    }
}

fn command_error(result: &command_runner::CommandResult) -> String {
    if result.timed_out {
        return "timed out".to_string();
    }
    let detail = if result.stderr.trim().is_empty() {
        result.stdout.trim()
    } else {
        result.stderr.trim()
    };
    if detail.is_empty() {
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
    fn confirmation_must_match_the_named_action() {
        assert!(parse_request(json!({"action":"reboot","confirm":"REBOOT"})).is_ok());
        assert!(parse_request(json!({"action":"poweroff","confirm":"POWEROFF"})).is_ok());
        assert_eq!(
            parse_request(json!({"action":"reboot","confirm":"POWEROFF"})).unwrap_err(),
            "confirmation_does_not_match_action"
        );
        assert_eq!(
            parse_request(json!({"action":"halt","confirm":"HALT"})).unwrap_err(),
            "action_must_be_reboot_or_poweroff"
        );
    }

    #[test]
    fn unknown_fields_are_rejected() {
        assert!(parse_request(json!({"action":"reboot","confirm":"REBOOT","extra":true})).is_err());
    }
}
