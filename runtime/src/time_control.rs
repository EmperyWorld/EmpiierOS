use crate::command_runner;
use crate::state_store::atomic_write_file;
use serde::Deserialize;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

const REQUEST_DIR: &str = "/run/empiieros";
const REQUEST_NAME: &str = "time-sync.request";
const RESULT_NAME: &str = "time-sync.result";
const UNIT: &str = "empiieros-time-sync.service";
const HELPER: &str = "/usr/local/sbin/empiieros-time-sync";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ControlParams {
    action: String,
    datetime: Option<String>,
}

#[derive(Debug)]
enum ClockAction {
    Set(String),
}

pub fn helper_ready() -> bool {
    Path::new(HELPER).is_file()
        && Path::new("/etc/systemd/system").join(UNIT).is_file()
        && Path::new("/usr/bin/systemctl").is_file()
}

pub fn execute(params: Value) -> Result<Value, String> {
    let ClockAction::Set(datetime) = parse_request(params)?;
    if !helper_ready() {
        return Err("time_backend_unavailable: allowlisted time helper is not installed".to_string());
    }
    write_set_request(&datetime)?;
    let _ = fs::remove_file(Path::new(REQUEST_DIR).join(RESULT_NAME));
    start_unit()?;
    wait_for_set(&datetime, Duration::from_secs(15))?;
    Ok(json!({
        "action": "set",
        "applied": true,
        "datetime": datetime,
        "source": "operator_wall_clock"
    }))
}

fn parse_request(params: Value) -> Result<ClockAction, String> {
    let params: ControlParams =
        serde_json::from_value(params).map_err(|error| format!("invalid_params: {error}"))?;
    match params.action.as_str() {
        "sync" => Err("ntp_masked".to_string()),
        "set" => {
            let datetime = params
                .datetime
                .as_deref()
                .map(str::trim)
                .filter(|value| wall_clock_ok(value))
                .ok_or_else(|| "datetime_invalid".to_string())?;
            Ok(ClockAction::Set(datetime.to_string()))
        }
        _ => Err("action_must_be_set".to_string()),
    }
}

fn wall_clock_ok(value: &str) -> bool {
    let (date, time) = match value.split_once(' ') {
        Some(pair) => pair,
        None => return false,
    };
    let mut date_parts = date.split('-');
    let year: u16 = date_parts.next().and_then(|item| item.parse().ok()).unwrap_or(0);
    let month: u8 = date_parts.next().and_then(|item| item.parse().ok()).unwrap_or(0);
    let day: u8 = date_parts.next().and_then(|item| item.parse().ok()).unwrap_or(0);
    if date_parts.next().is_some()
        || !(2024..=2100).contains(&year)
        || !(1..=12).contains(&month)
        || day == 0
        || day > days_in_month(year, month)
    {
        return false;
    }
    let mut time_parts = time.split(':');
    let hour: u8 = time_parts.next().and_then(|item| item.parse().ok()).unwrap_or(99);
    let minute: u8 = time_parts.next().and_then(|item| item.parse().ok()).unwrap_or(99);
    let second = time_parts.next();
    if time_parts.next().is_some() || hour > 23 || minute > 59 {
        return false;
    }
    match second {
        None => true,
        Some(text) => text.len() == 2 && text.bytes().all(|byte| byte.is_ascii_digit()) && text.parse::<u8>().unwrap_or(99) <= 59,
    }
}

fn is_gregorian_leap(year: u16) -> bool {
    (year % 4 == 0) && (year % 100 != 0 || year % 400 == 0)
}

fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_gregorian_leap(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

fn write_set_request(datetime: &str) -> Result<(), String> {
    let body = format!("action=set\ndatetime={datetime}\n");
    atomic_write_file(Path::new(REQUEST_DIR), REQUEST_NAME, body.as_bytes(), 0o600)
        .map_err(|error| format!("time_request_write_failed: {error}"))?;
    Ok(())
}

fn start_unit() -> Result<(), String> {
    let result = command_runner::run(
        "/usr/bin/systemctl",
        &["start", UNIT],
        Duration::from_secs(30),
    )
    .map_err(|error| format!("time_service_spawn_failed: {error}"))?;
    if !result.success {
        return Err(format!(
            "time_service_start_failed: {}",
            command_error(&result)
        ));
    }
    Ok(())
}

fn wait_for_set(datetime: &str, timeout: Duration) -> Result<(), String> {
    let path = Path::new(REQUEST_DIR).join(RESULT_NAME);
    let started = Instant::now();
    loop {
        if let Ok(body) = fs::read_to_string(&path) {
            if set_result_applied(&body, datetime) {
                return Ok(());
            }
        }
        if started.elapsed() >= timeout {
            return Err("time_set_failed: operator clock was not applied".to_string());
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn set_result_applied(body: &str, datetime: &str) -> bool {
    let mut action_set = false;
    let mut applied = false;
    let mut echoed = false;
    for line in body.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key {
            "action" if value.trim() == "set" => action_set = true,
            "applied" if value.trim() == "1" => applied = true,
            "datetime" if value.trim() == datetime => echoed = true,
            _ => {}
        }
    }
    action_set && applied && echoed
}

pub(crate) fn is_ntp_server(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b':' | b'_'))
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
    fn parse_sets_operator_clock_and_refuses_ntp() {
        assert!(matches!(
            parse_request(json!({"action":"set","datetime":"2026-09-21 03:31"})),
            Ok(ClockAction::Set(_))
        ));
        assert!(matches!(
            parse_request(json!({"action":"set","datetime":"2026-09-21 03:31:40"})),
            Ok(ClockAction::Set(_))
        ));
        assert_eq!(
            parse_request(json!({"action":"sync"})).unwrap_err(),
            "ntp_masked"
        );
        assert_eq!(
            parse_request(json!({"action":"set_timezone"})).unwrap_err(),
            "action_must_be_set"
        );
        assert!(parse_request(json!({"action":"set"})).is_err());
        assert!(parse_request(json!({"action":"set","datetime":"now"})).is_err());
        assert!(parse_request(json!({"action":"set","datetime":"2026-09-21 03:31","extra":true})).is_err());
    }

    #[test]
    fn wall_clock_rejects_impossible_calendar_days() {
        assert!(wall_clock_ok("2026-01-31 12:00"));
        assert!(wall_clock_ok("2024-02-29 12:00")); // leap
        assert!(wall_clock_ok("2026-04-30 12:00"));
        assert!(!wall_clock_ok("2026-02-31 12:00"));
        assert!(!wall_clock_ok("2026-02-29 12:00")); // non-leap
        assert!(!wall_clock_ok("2026-04-31 12:00"));
        assert!(!wall_clock_ok("2100-02-29 12:00")); // century non-leap
        assert!(!wall_clock_ok("2026-00-10 12:00"));
        assert!(!wall_clock_ok("2026-13-01 12:00"));
    }

    #[test]
    fn set_result_must_echo_the_requested_clock() {
        assert!(set_result_applied(
            "action=set\napplied=1\ndatetime=2026-09-21 03:31\n",
            "2026-09-21 03:31"
        ));
        assert!(!set_result_applied(
            "action=sync\nsynchronized=1\nserver=time1.google.com\n",
            "2026-09-21 03:31"
        ));
        assert!(!set_result_applied(
            "action=set\napplied=1\ndatetime=2026-09-21 03:32\n",
            "2026-09-21 03:31"
        ));
    }
}
