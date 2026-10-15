//! `seat-shield-fw` — pinned floor helper (prep bay reference + nft-shaped metal path).
//!
//! Protocol (argv only, no shell):
//!   seat-shield-fw --state-dir ABS [--backend state|nft] [...] block --ip IP [--reason TEXT]
//!   seat-shield-fw --state-dir ABS [--backend state|nft] [...] unblock --ip IP
//!   seat-shield-fw --state-dir ABS [--backend state|nft] [...] list
//!
//! One JSON object on stdout. Exit 0 on success.

use std::env;
use std::process;

use serde::Serialize;
use seat_shield_core::fw::{dispatch, parse_cli, FwOutcome};

#[derive(Debug, Serialize)]
struct OutOk {
    ok: bool,
    action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    peer_ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    removed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    peers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    backend: Option<String>,
}

#[derive(Debug, Serialize)]
struct OutErr {
    ok: bool,
    error: String,
}

fn main() {
    if let Err(err) = run(env::args().skip(1).collect()) {
        emit_err(&err);
        process::exit(1);
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    let cli = parse_cli(&args)?;
    let backend = cli.backend.as_str().to_string();
    let outcome = dispatch(&cli)?;
    match outcome {
        FwOutcome::Block { peer_ip } => emit_ok(OutOk {
            ok: true,
            action: "block".into(),
            peer_ip: Some(peer_ip),
            removed: None,
            peers: None,
            backend: Some(backend),
        }),
        FwOutcome::Unblock { peer_ip, removed } => emit_ok(OutOk {
            ok: true,
            action: "unblock".into(),
            peer_ip: Some(peer_ip),
            removed: Some(removed),
            peers: None,
            backend: Some(backend),
        }),
        FwOutcome::List { peers } => emit_ok(OutOk {
            ok: true,
            action: "list".into(),
            peer_ip: None,
            removed: None,
            peers: Some(peers),
            backend: Some(backend),
        }),
    }
    Ok(())
}

fn emit_ok(value: OutOk) {
    println!("{}", serde_json::to_string(&value).expect("serialize"));
}

fn emit_err(message: &str) {
    let err = OutErr {
        ok: false,
        error: message.to_string(),
    };
    println!("{}", serde_json::to_string(&err).unwrap_or_else(|_| {
        r#"{"ok":false,"error":"serialize_failed"}"#.into()
    }));
}
