//! Local NDJSON harness for EmpiierOS Seat Shield (prep bay).
//!
//! ```text
//! seat-shield-harness --state DIR \
//!   [--actuator unavailable|recording|helper] \
//!   [--helper-path /abs/seat-shield-fw] \
//!   [--helper-backend state|nft] \
//!   [--nft-bin /abs/nft] [--nft-table NAME] [--nft-set NAME] \
//!   [--auto-cycle SECS] \
//!   [--no-proc]
//! ```
//!
//! `--auto-cycle` loops until stdin EOF or SIGINT. Without the flag the
//! harness stays one-shot (read stdin until EOF). No extra systemd unit.

use std::env;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::process;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::Duration;

use seat_shield_core::{ActuatorBackend, ServiceConfig, ShieldService};

fn usage() -> ! {
    eprintln!(
        "usage: seat-shield-harness --state DIR \\\n\
         \t[--actuator unavailable|recording|helper] \\\n\
         \t[--helper-path /abs/path/seat-shield-fw] \\\n\
         \t[--helper-backend state|nft] [--nft-bin ABS] \\\n\
         \t[--nft-family inet] [--nft-table NAME] [--nft-set NAME] [--nft-set6 NAME] \\\n\
         \t[--auto-cycle SECS] \\\n\
         \t[--no-proc]\n\
         policy/acts via NDJSON ops only"
    );
    process::exit(2);
}

fn main() {
    let mut args = env::args().skip(1);
    let mut state: Option<PathBuf> = None;
    let mut actuator = ActuatorBackend::Unavailable;
    let mut helper_path: Option<PathBuf> = None;
    let mut helper_backend: Option<String> = None;
    let mut nft_bin: Option<String> = None;
    let mut nft_family: Option<String> = None;
    let mut nft_table: Option<String> = None;
    let mut nft_set: Option<String> = None;
    let mut nft_set6: Option<String> = None;
    let mut enable_proc = true;
    let mut auto_cycle: Option<u64> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--state" => {
                state = Some(PathBuf::from(args.next().unwrap_or_else(|| usage())));
            }
            "--actuator" => {
                let value = args.next().unwrap_or_else(|| usage());
                actuator = match value.as_str() {
                    "unavailable" => ActuatorBackend::Unavailable,
                    "recording" => ActuatorBackend::Recording,
                    "helper" => ActuatorBackend::Helper,
                    _ => usage(),
                };
            }
            "--helper-path" => {
                helper_path = Some(PathBuf::from(args.next().unwrap_or_else(|| usage())));
            }
            "--helper-backend" => {
                helper_backend = Some(args.next().unwrap_or_else(|| usage()));
            }
            "--nft-bin" => {
                nft_bin = Some(args.next().unwrap_or_else(|| usage()));
            }
            "--nft-family" => {
                nft_family = Some(args.next().unwrap_or_else(|| usage()));
            }
            "--nft-table" => {
                nft_table = Some(args.next().unwrap_or_else(|| usage()));
            }
            "--nft-set" => {
                nft_set = Some(args.next().unwrap_or_else(|| usage()));
            }
            "--nft-set6" => {
                nft_set6 = Some(args.next().unwrap_or_else(|| usage()));
            }
            "--no-proc" => enable_proc = false,
            "--auto-cycle" => {
                let raw = args.next().unwrap_or_else(|| usage());
                let secs: u64 = raw.parse().unwrap_or_else(|_| usage());
                if !(10..=300).contains(&secs) {
                    eprintln!("seat-shield-harness: --auto-cycle must be 10..=300");
                    process::exit(2);
                }
                auto_cycle = Some(secs);
            }
            "-h" | "--help" => usage(),
            _ => usage(),
        }
    }

    let state = state.unwrap_or_else(|| usage());
    if actuator == ActuatorBackend::Helper && helper_path.is_none() {
        eprintln!("seat-shield-harness: --helper-path required with --actuator helper");
        process::exit(2);
    }

    let mut helper_prefix = Vec::new();
    if let Some(backend) = helper_backend {
        helper_prefix.push("--backend".into());
        helper_prefix.push(backend);
    }
    if let Some(bin) = nft_bin {
        helper_prefix.push("--nft-bin".into());
        helper_prefix.push(bin);
    }
    if let Some(family) = nft_family {
        helper_prefix.push("--nft-family".into());
        helper_prefix.push(family);
    }
    if let Some(table) = nft_table {
        helper_prefix.push("--nft-table".into());
        helper_prefix.push(table);
    }
    if let Some(set) = nft_set {
        helper_prefix.push("--nft-set".into());
        helper_prefix.push(set);
    }
    if let Some(set6) = nft_set6 {
        helper_prefix.push("--nft-set6".into());
        helper_prefix.push(set6);
    }

    let mut service = ShieldService::open(ServiceConfig {
        state_root: state,
        actuator,
        helper_path,
        helper_prefix,
        enable_proc_observe: enable_proc,
                probe_helper_path: None,
        })
    .unwrap_or_else(|e| {
        eprintln!("seat-shield-harness: open failed: {e}");
        process::exit(1);
    });

    if let Some(secs) = auto_cycle {
        service.configure_cycle_interval(secs).unwrap_or_else(|e| {
            eprintln!("seat-shield-harness: cycle interval: {e}");
            process::exit(1);
        });
        run_auto_cycle(&mut service, secs);
        return;
    }

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("seat-shield-harness: read failed: {e}");
                process::exit(1);
            }
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let response = service.handle_bytes(trimmed.as_bytes());
        if let Err(e) = stdout.write_all(&response.encode_line()) {
            eprintln!("seat-shield-harness: write failed: {e}");
            process::exit(1);
        }
        let _ = stdout.flush();
    }
}

fn write_line(stdout: &mut impl Write, service: &mut ShieldService, line: &str) {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return;
    }
    let response = service.handle_bytes(trimmed.as_bytes());
    if let Err(e) = stdout.write_all(&response.encode_line()) {
        eprintln!("seat-shield-harness: write failed: {e}");
        process::exit(1);
    }
    let _ = stdout.flush();
}

fn run_auto_cycle(service: &mut ShieldService, interval: u64) {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            if tx.send(line).is_err() {
                break;
            }
        }
    });
    let mut stdout = io::stdout();
    if let Err(e) = service.maybe_auto_observe() {
        eprintln!("seat-shield-harness: auto-cycle: {e}");
    }
    loop {
        match rx.recv_timeout(Duration::from_secs(interval)) {
            Ok(Ok(line)) => write_line(&mut stdout, service, &line),
            Ok(Err(e)) => {
                eprintln!("seat-shield-harness: read failed: {e}");
                process::exit(1);
            }
            Err(RecvTimeoutError::Timeout) => {
                if let Err(e) = service.maybe_auto_observe() {
                    eprintln!("seat-shield-harness: auto-cycle: {e}");
                }
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
}
