//! Seat entry for LinkornEW. Runs in the seat session and execs only the
//! argv `linkorn::host_plan` allows. The FoFline image does not build it.

use std::fs::OpenOptions;
use std::process::{Command, Stdio};

const LAST: &str = "/var/lib/empiieros-runtime/linkorn/chrome/host.last";

fn note(message: &str) {
    let _ = std::fs::write(LAST, format!("{message}\n"));
    eprintln!("linkornew-host: {message}");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let argv = match empiieros_runtime::linkorn::host_plan(&args) {
        Ok(argv) => argv,
        Err(code) => {
            note(code);
            std::process::exit(1);
        }
    };
    note(&format!("running {}", argv.join(" ")));
    let mut command = Command::new(&argv[0]);
    command
        .args(&argv[1..])
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("LIBGL_ALWAYS_SOFTWARE", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::inherit());
    if let Ok(file) = OpenOptions::new().create(true).append(true).open(LAST) {
        command.stderr(Stdio::from(file));
    } else {
        command.stderr(Stdio::inherit());
    }
    for (key, value) in empiieros_runtime::linkorn::seat_env_from(|key| std::env::var(key).ok()) {
        command.env(key, value);
    }
    for (key, value) in std::env::vars() {
        if !(key.starts_with("XDG_") || key.starts_with("WAYLAND_")) {
            continue;
        }
        if value.is_empty() || value.len() > 512 || value.chars().any(|ch| ch.is_control()) {
            continue;
        }
        command.env(key, value);
    }
    if std::env::var("HOME").ok().filter(|h| !h.is_empty()).is_none() {
        command.env("HOME", "/tmp/linkornew-home");
    }
    let status = match command.status() {
        Ok(status) => status,
        Err(error) => {
            note(&format!("spawn_failed:{error}"));
            std::process::exit(1);
        }
    };
    let code = status.code().unwrap_or(1);
    if code != 0 {
        let extra = std::fs::read_to_string(LAST).unwrap_or_default();
        if extra.trim().is_empty() {
            note(&format!("child_exit:{code}"));
        }
    }
    std::process::exit(code);
}
