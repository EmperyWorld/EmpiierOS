//! LinkornEW staff profile (`linkornew`).
//!
//! The FoFline runtime does not register these operations. The staff
//! runtime is a separate build (`--no-default-features --features linkornew`).
//! Enabling it together with `production` does not compile.
//!
//! Runtime returns a fixed argv for the seat host. It does not spawn a
//! terminal, a browser, or a tty.

use serde::Deserialize;
use serde_json::{json, Value};

use crate::protocol::RuntimeResponse;

const CONSOLE_CANDIDATES: &[&str] = &["/usr/lib/linkornew/console"];
const BROWSER_BIN: &str = "/usr/lib/station-chromium/chrome";
const BROWSER_PROFILE: &str = "/var/lib/empiieros-runtime/linkorn/chrome";
const COMMAND_DECK_UNIT: &str = "/etc/systemd/system/cp-command-deck.service";
const COMMAND_DECK_URL: &str = "http://127.0.0.1:4191";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EmptyParams {}

#[cfg_attr(not(feature = "linkornew"), allow(dead_code))]
pub fn handles(operation: &str) -> bool {
    matches!(
        operation,
        "linkorn.status" | "linkorn.console" | "linkorn.browser" | "linkorn.command_post"
    )
}

pub fn closed_on_staff(operation: &str) -> bool {
    operation.starts_with("launcher.")
        || operation == "product.open"
        || operation.starts_with("seat.browser.")
}

pub fn dispatch(id: String, operation: &str, params: &Value) -> RuntimeResponse {
    if let Err(error) = serde_json::from_value::<EmptyParams>(params.clone()) {
        return RuntimeResponse::failure(
            id,
            "invalid_params",
            "invalid_params",
            format!("linkorn params must be empty: {error}"),
        );
    }
    match operation {
        "linkorn.status" => RuntimeResponse::success(id, "observed", status_body()),
        "linkorn.console" => action(id, "open_console", console_argv()),
        "linkorn.browser" => action(id, "open_browser", browser_argv()),
        "linkorn.command_post" => action(id, "open_command_post", command_post_argv()),
        _ => RuntimeResponse::failure(
            id,
            "not_found",
            "operation_not_found",
            "operation is not registered",
        ),
    }
}

fn action(id: String, host_action: &str, argv: Result<Vec<String>, &'static str>) -> RuntimeResponse {
    match argv {
        Ok(argv) => RuntimeResponse::success(
            id,
            "accepted",
            json!({
                "profile": "linkornew",
                "host_action": host_action,
                "argv": argv,
                "detail": "seat host execs this argv; runtime does not spawn a tty"
            }),
        ),
        Err(code) => RuntimeResponse::failure(
            id,
            "not_available",
            code,
            "LinkornEW host program is not installed on this seat",
        ),
    }
}

fn status_body() -> Value {
    json!({
        "profile": "linkornew",
        "display_name": "LinkornEW",
        "console": presence(console_argv()),
        "browser": presence(browser_argv()),
        "command_post": presence(command_post_argv()),
    })
}

fn presence(argv: Result<Vec<String>, &'static str>) -> Value {
    match argv {
        Ok(argv) => json!({ "availability": "available", "argv": argv }),
        Err(code) => json!({ "availability": "unavailable", "reason": code }),
    }
}

fn console_argv() -> Result<Vec<String>, &'static str> {
    let path = CONSOLE_CANDIDATES
        .iter()
        .copied()
        .find(|path| std::path::Path::new(path).is_file())
        .ok_or("console_not_installed")?;
    Ok(vec![path.to_string(), "--title=LinkornEW".to_string()])
}

fn browser_argv() -> Result<Vec<String>, &'static str> {
    if !std::path::Path::new(BROWSER_BIN).is_file() {
        return Err("browser_not_installed");
    }
    Ok(vec![
        BROWSER_BIN.to_string(),
        format!("--user-data-dir={BROWSER_PROFILE}"),
        "--no-first-run".to_string(),
        "--disable-sync".to_string(),
        "--no-default-browser-check".to_string(),
        "about:blank".to_string(),
    ])
}

fn command_post_argv() -> Result<Vec<String>, &'static str> {
    if !std::path::Path::new(COMMAND_DECK_UNIT).is_file() {
        return Err("command_post_not_installed");
    }
    let browser = browser_argv()?;
    let mut argv = browser;
    if argv.last().is_some_and(|item| item == "about:blank") {
        argv.pop();
    }
    argv.push(COMMAND_DECK_URL.to_string());
    Ok(argv)
}

pub fn seat_action(word: &str) -> Result<&'static str, &'static str> {
    match word {
        "console" => Ok("console"),
        "browser" => Ok("browser"),
        "command-post" => Ok("command-post"),
        _ => Err("seat_action"),
    }
}

/// Argv the seat window may pass to `linkornew-host`. One action, no shell.
pub fn seat_host_argv(host: &str, word: &str) -> Result<Vec<String>, &'static str> {
    let action = seat_action(word)?;
    let name = std::path::Path::new(host).file_name().and_then(|item| item.to_str());
    if !host.starts_with('/') || host.contains('\0') || host.contains("..") || name != Some("linkornew-host")
    {
        return Err("seat_host");
    }
    Ok(vec![host.to_string(), action.to_string()])
}

const SEAT_ENV_KEYS: &[&str] = &[
    "WAYLAND_DISPLAY",
    "DISPLAY",
    "HOME",
    "XDG_RUNTIME_DIR",
    "XDG_SESSION_TYPE",
    "XDG_CONFIG_HOME",
    "DBUS_SESSION_BUS_ADDRESS",
    "LANG",
];

/// Argv for `linkornew-host`. One fixed action. Extra words are refused.
pub fn host_plan(args: &[String]) -> Result<Vec<String>, &'static str> {
    if args.len() != 1 {
        return Err("host_usage");
    }
    let operation = match args[0].as_str() {
        "console" => "linkorn.console",
        "browser" => "linkorn.browser",
        "command-post" => "linkorn.command_post",
        _ => return Err("host_usage"),
    };
    let response = dispatch("host".into(), operation, &json!({}));
    if !response.ok {
        return Err(match response.error.as_ref().map(|error| error.code.as_str()) {
            Some("console_not_installed") => "console_not_installed",
            Some("browser_not_installed") => "browser_not_installed",
            Some("command_post_not_installed") => "command_post_not_installed",
            _ => "host_failed",
        });
    }
    let argv = response
        .data
        .get("argv")
        .and_then(Value::as_array)
        .ok_or("host_argv_rejected")?;
    let argv = argv
        .iter()
        .map(|item| item.as_str().map(str::to_string))
        .collect::<Option<Vec<_>>>()
        .ok_or("host_argv_rejected")?;
    if !host_argv_allowed(&argv) {
        return Err("host_argv_rejected");
    }
    Ok(argv)
}

pub fn host_argv_allowed(argv: &[String]) -> bool {
    let Some(bin) = argv.first() else {
        return false;
    };
    match bin.as_str() {
        "/usr/lib/linkornew/console" => argv.len() == 2 && argv[1] == "--title=LinkornEW",
        "/usr/lib/station-chromium/chrome" => {
            argv.len() == 6
                && argv[1] == format!("--user-data-dir={BROWSER_PROFILE}")
                && argv[2] == "--no-first-run"
                && argv[3] == "--disable-sync"
                && argv[4] == "--no-default-browser-check"
                && (argv[5] == "about:blank" || argv[5] == COMMAND_DECK_URL)
        }
        _ => false,
    }
}

pub fn seat_env_from(get: impl Fn(&str) -> Option<String>) -> Vec<(&'static str, String)> {
    SEAT_ENV_KEYS
        .iter()
        .filter_map(|key| {
            let value = get(key)?;
            if value.is_empty()
                || value.len() > 512
                || value.chars().any(|ch| ch.is_control())
            {
                return None;
            }
            Some((*key, value))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn staff_profile_closes_the_fofline_face_only() {
        assert!(closed_on_staff("launcher.air_activate"));
        assert!(closed_on_staff("launcher.capsule_import"));
        assert!(closed_on_staff("product.open"));
        assert!(closed_on_staff("seat.browser.navigate"));
        assert!(!closed_on_staff("time.sync"));
        assert!(!closed_on_staff("linkorn.console"));
        assert!(!closed_on_staff("seat.shield.status"));
    }

    #[test]
    fn params_cannot_name_a_command() {
        let response = dispatch(
            "k".into(),
            "linkorn.console",
            &json!({ "command": "/bin/sh" }),
        );
        assert!(!response.ok);
        assert_eq!(response.status, "invalid_params");
    }

    #[test]
    fn missing_host_programs_stay_unavailable() {
        let response = dispatch("k".into(), "linkorn.console", &json!({}));
        if std::path::Path::new("/usr/lib/linkornew/console").is_file() {
            assert!(response.ok, "{response:?}");
            let argv = response.data["argv"].as_array().expect("argv");
            assert!(argv.iter().all(|item| item.as_str().is_some_and(|text| text != "/bin/sh")));
        } else {
            assert!(!response.ok);
            assert_eq!(response.status, "not_available");
            assert_eq!(
                response.error.as_ref().map(|error| error.code.as_str()),
                Some("console_not_installed")
            );
        }
    }

    #[test]
    fn host_plan_refuses_extra_words_and_foreign_argv() {
        assert_eq!(
            host_plan(&["console".into(), "/bin/sh".into()]).unwrap_err(),
            "host_usage"
        );
        assert_eq!(host_plan(&["shell".into()]).unwrap_err(), "host_usage");
        assert!(!host_argv_allowed(&["/bin/sh".into()]));
        assert!(!host_argv_allowed(&[
            "/usr/lib/station-chromium/chrome".into(),
            format!("--user-data-dir={BROWSER_PROFILE}"),
            "--no-first-run".into(),
            "--disable-sync".into(),
            "--no-default-browser-check".into(),
            "https://example.com".into(),
        ]));
        let env = seat_env_from(|key| match key {
            "WAYLAND_DISPLAY" => Some("wayland-0".into()),
            "LD_PRELOAD" => Some("/tmp/lib.so".into()),
            "PATH" => Some("/tmp".into()),
            "LANG" => Some("ru_RU.UTF-8\nLD_PRELOAD=/tmp/x".into()),
            _ => None,
        });
        assert_eq!(env, vec![("WAYLAND_DISPLAY", "wayland-0".into())]);
        assert_eq!(seat_action("console").unwrap(), "console");
        assert_eq!(seat_action("browser").unwrap(), "browser");
        assert_eq!(seat_action("command-post").unwrap(), "command-post");
        assert_eq!(seat_action("console /bin/sh").unwrap_err(), "seat_action");
        assert_eq!(seat_action("/bin/sh").unwrap_err(), "seat_action");
        assert_eq!(
            seat_host_argv("/usr/lib/linkornew/linkornew-host", "console").unwrap(),
            vec![
                "/usr/lib/linkornew/linkornew-host".to_string(),
                "console".to_string()
            ]
        );
        assert!(seat_host_argv("/bin/sh", "console").is_err());
        assert!(seat_host_argv("/tmp/linkornew-host/../../bin/sh", "console").is_err());
    }

    #[cfg(feature = "linkornew")]
    #[test]
    fn staff_runtime_closes_the_fofline_face_and_serves_linkorn() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("empiieros-linkorn-{nanos}"));
        let core = crate::RuntimeCore::new(root.clone()).expect("core");
        let product = core.handle_bytes(br#"{"id":"p","op":"product.open","params":{}}"#);
        assert!(!product.ok);
        assert_eq!(
            product.error.as_ref().map(|error| error.code.as_str()),
            Some("staff_profile")
        );
        let air = core.handle_bytes(br#"{"id":"a","op":"launcher.air_activate","params":{}}"#);
        assert_eq!(
            air.error.as_ref().map(|error| error.code.as_str()),
            Some("staff_profile")
        );
        let status = core.handle_bytes(br#"{"id":"s","op":"linkorn.status","params":{}}"#);
        assert!(status.ok, "{status:?}");
        assert_eq!(status.data["profile"], "linkornew");
        let injected = core.handle_bytes(
            br#"{"id":"c","op":"linkorn.console","params":{"command":"/bin/sh"}}"#,
        );
        assert!(!injected.ok);
        assert_eq!(injected.status, "invalid_params");
        let _ = std::fs::remove_dir_all(root);
    }
}
