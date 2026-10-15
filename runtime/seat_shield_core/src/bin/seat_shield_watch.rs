//! `seat-shield-watch` — pinned S5B helper. Writes watch.jsonl only.
//!
//!   seat-shield-watch --state-dir ABS [--duration-ms N] [--inject PATH]
//!
//! Never calls nft. Never quarantines. One JSON object on stdout.

use std::env;
use std::process;

use serde::Serialize;
use seat_shield_core::act::ActuatorBackend;
use seat_shield_core::watch::{inject_watch_path, parse_watch_cli, run_inotify_loop};
use seat_shield_core::{ServiceConfig, ShieldService};

#[derive(Debug, Serialize)]
struct OutOk {
    ok: bool,
    action: String,
    written: u32,
}

#[derive(Debug, Serialize)]
struct OutErr {
    ok: bool,
    error: String,
}

fn main() {
    if let Err(err) = run(env::args().skip(1).collect()) {
        println!(
            "{}",
            serde_json::to_string(&OutErr {
                ok: false,
                error: err,
            })
            .expect("serialize")
        );
        process::exit(1);
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    let cli = parse_watch_cli(&args)?;
    let svc = ShieldService::open(ServiceConfig {
        state_root: cli.state_dir.clone(),
        actuator: ActuatorBackend::Unavailable,
        helper_path: None,
        helper_prefix: Vec::new(),
        enable_proc_observe: false,
                probe_helper_path: None,
        })
    .map_err(|e| e.to_string())?;
    let store = svc.store();
    if let Some(path) = cli.inject.as_deref() {
        inject_watch_path(store, path).map_err(|e| e.to_string())?;
        emit_ok("inject", 1);
        return Ok(());
    }
    let written = run_inotify_loop(store, cli.duration).map_err(|e| e.to_string())?;
    emit_ok("watch", written);
    Ok(())
}

fn emit_ok(action: &str, written: u32) {
    println!(
        "{}",
        serde_json::to_string(&OutOk {
            ok: true,
            action: action.into(),
            written,
        })
        .expect("serialize")
    );
}
