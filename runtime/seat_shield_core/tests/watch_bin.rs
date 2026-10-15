//! S5B helper binary: inject journal, reject nft verbs.

use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn bin() -> Command {
    let path = option_env!("CARGO_BIN_EXE_seat-shield-watch")
        .map(str::to_string)
        .or_else(|| std::env::var("CARGO_BIN_EXE_seat-shield-watch").ok())
        .unwrap_or_else(|| {
            format!(
                "{}/debug/seat-shield-watch",
                std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| format!(
                    "{}/target",
                    env!("CARGO_MANIFEST_DIR")
                ))
            )
        });
    Command::new(path)
}

#[test]
fn watch_bin_rejects_nft_verbs() {
    let out = bin()
        .args(["--state-dir", "/tmp/seat_s5b_bin_nft", "block"])
        .output()
        .expect("run watch bin");
    assert!(!out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("nft"), "{text}");
}

#[test]
fn watch_bin_inject_writes_journal() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let state = std::env::temp_dir().join(format!("seat_s5b_bin_{stamp}"));
    std::fs::create_dir_all(&state).unwrap();
    let inject = format!("/tmp/seat_s5b_bin_inject_{stamp}");
    std::fs::write(&inject, b"s5c-bin-inject").unwrap();
    let out = bin()
        .args([
            "--state-dir",
            state.to_str().expect("utf8 state"),
            "--inject",
            &inject,
        ])
        .output()
        .expect("inject");
    assert!(out.status.success(), "{out:?}");
    let journal = state.join("seat_shield").join("watch.jsonl");
    let body = std::fs::read_to_string(journal).unwrap();
    assert!(body.contains(&inject), "{body}");
    assert!(body.contains("sha256"), "{body}");
    assert!(!body.contains("nft"));
}
