//! H4: bounded corpus + PRNG against parsers. Stable rustc. No helper exec.
//!
//! `cargo fuzz` / nightly is optional host tooling. This test is the OS measure.

use seat_shield_core::parser_probe;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/parser_corpus")
}

fn xorshift64(seed: u64) -> impl Iterator<Item = u64> {
    let mut s = if seed == 0 { 0x9E37_79B9_7F4A_7C15 } else { seed };
    std::iter::from_fn(move || {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        Some(s)
    })
}

fn feed_bytes(raw: &[u8], service: &mut seat_shield_core::service::ShieldService) {
    let text = String::from_utf8_lossy(raw);
    parser_probe::nft_text(&text);
    parser_probe::nft_json(&text);
    parser_probe::proc_net_line(&text, false);
    parser_probe::proc_net_line(&text, true);
    parser_probe::runtime_line(service, raw);
}

#[test]
fn corpus_and_prng_do_not_panic_or_exec() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("seat_h4_fuzz_{stamp}"));
    let _ = fs::create_dir_all(&root);
    let mut service = parser_probe::unavailable_service(root.clone());

    let dir = corpus_dir();
    assert!(dir.is_dir(), "H4 corpus missing: {}", dir.display());
    let mut files = 0usize;
    for entry in fs::read_dir(&dir).expect("corpus dir") {
        let entry = entry.expect("corpus entry");
        if !entry.file_type().expect("ft").is_file() {
            continue;
        }
        let raw = fs::read(entry.path()).expect("corpus read");
        feed_bytes(&raw, &mut service);
        files += 1;
    }
    assert!(files >= 6, "H4 corpus too thin: {files}");

    let mut rng = xorshift64(0xA5A5_5A5A_C3C3_3C3C);
    for i in 0..2_048u32 {
        let len = (rng.next().unwrap() % 512) as usize;
        let mut buf = vec![0u8; len];
        for slot in &mut buf {
            *slot = (rng.next().unwrap() & 0xFF) as u8;
        }
        if i % 32 == 0 {
            buf.extend_from_slice(br#"{"id":"f","op":"seat.shield.act","params":{"action":"block_peer","peer_ip":"198.51.100.9"}}"#);
        }
        feed_bytes(&buf, &mut service);
    }

    let _ = fs::remove_dir_all(root);
}
