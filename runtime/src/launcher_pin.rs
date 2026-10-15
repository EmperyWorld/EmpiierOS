//! Local 4-digit first-boot PIN for FoFline V2.
//!
//! Hardware stamp is the harvested `fp_v2` fingerprint. The installer apply
//! helper plants the hash-domain record after install proof. Production IPC
//! does not mint a PIN (`launcher.pin_issue` is `installer_issued_only`).
//! `issue()` is test-only (installer-equivalent plant). Production cannot mint.
//! Air and file consume call `require_unlocked` first. This slice does not
//! open `product.open`. Not `operator.auth`. Not Tauri. The PIN is never stored.

use crate::launcher_status;
use crate::state_store::{atomic_write_file, ensure_secure_directory};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use zeroize::Zeroize;

const SEAT_DIR: &str = "seat";
const RECORD_NAME: &str = "first_boot_pin.json";
const MARKER_NAME: &str = "first_boot_pin.issued";
const HASH_DOMAIN: &[u8] = b"empiieros-first-boot-pin-v1|";
const MAX_FAILED: u32 = 5;
const LOCKOUT_SECS: u64 = 300;

pub struct PinFlags {
    pub issued: bool,
    pub unlocked: bool,
    pub bound: bool,
    pub locked: bool,
}

pub fn observe_flags(state_root: &Path, fingerprint: &str) -> PinFlags {
    match load_record(state_root) {
        Some(record) => PinFlags {
            issued: true,
            unlocked: record.unlocked,
            bound: record.fingerprint == fingerprint,
            locked: record.locked_until_epoch > now_epoch(),
        },
        None => PinFlags {
            issued: state_root.join(SEAT_DIR).join(MARKER_NAME).is_file(),
            unlocked: false,
            bound: false,
            locked: false,
        },
    }
}

#[cfg(test)]
pub fn plant_unlocked(state_root: &Path) {
    let planted = issue(state_root, json!({})).expect("plant");
    let pin = planted["pin"].as_str().expect("pin").to_string();
    unlock(state_root, json!({ "pin": pin })).expect("unlock");
}

#[cfg(test)]
pub fn issue(state_root: &Path, _params: Value) -> Result<Value, String> {
    if load_record(state_root).is_some() {
        return Err("already_issued".into());
    }
    let fingerprint = launcher_status::live_fingerprint_v2();
    let mut pin = generate_pin()?;
    let out = persist_new(state_root, &fingerprint, &pin)?;
    pin.zeroize();
    Ok(out)
}

/// First refusal of every mutation that calls `require_unlocked`.
pub fn access_block(state_root: &Path) -> Option<&'static str> {
    let fingerprint = launcher_status::live_fingerprint_v2();
    let flags = observe_flags(state_root, &fingerprint);
    if !flags.issued {
        Some("pin_not_issued")
    } else if flags.locked {
        Some("pin_locked")
    } else if !flags.unlocked {
        Some("pin_not_unlocked")
    } else if !flags.bound {
        Some("fingerprint_mismatch")
    } else {
        None
    }
}

pub fn require_unlocked(state_root: &Path) -> Result<(), String> {
    match access_block(state_root) {
        Some(code) => Err(code.into()),
        None => Ok(()),
    }
}

/// Early refusal of `unlock` itself. A waiting PIN entry is not a block:
/// format and the secret stay on the request.
pub fn unlock_block(state_root: &Path) -> Option<&'static str> {
    let Some(record) = load_record(state_root) else {
        return Some("pin_not_issued");
    };
    let fingerprint = launcher_status::live_fingerprint_v2();
    let rebound = record.fingerprint != fingerprint
        && planted_with_empty_machine_id(&record.fingerprint);
    if record.fingerprint != fingerprint && !rebound {
        return Some("fingerprint_mismatch");
    }
    if record.unlocked {
        return None;
    }
    if record.locked_until_epoch > now_epoch() {
        return Some("pin_locked");
    }
    None
}

pub fn unlock(state_root: &Path, params: Value) -> Result<Value, String> {
    if let Some(code) = unlock_block(state_root) {
        return Err(code.into());
    }
    let mut record = load_record(state_root).ok_or_else(|| "pin_not_issued".to_string())?;
    let fingerprint = launcher_status::live_fingerprint_v2();
    let rebound = record.fingerprint != fingerprint
        && planted_with_empty_machine_id(&record.fingerprint);
    if record.unlocked {
        return Ok(json!({
            "unlocked": true,
            "already": true,
            "detail": "first-boot PIN already unlocked; product.open stays unavailable"
        }));
    }
    let now = now_epoch();
    let mut pin = params
        .get("pin")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    if !is_four_digits(&pin) {
        pin.zeroize();
        return Err("pin_format".into());
    }
    let expected = hash_pin(&record.salt_hex, &record.fingerprint, &pin);
    if !hashes_equal(&expected, &record.pin_hash) {
        pin.zeroize();
        record.failed_attempts = record.failed_attempts.saturating_add(1);
        if record.failed_attempts >= MAX_FAILED {
            record.locked_until_epoch = now.saturating_add(LOCKOUT_SECS);
            persist_record(state_root, &record)?;
            return Err("pin_locked".into());
        }
        persist_record(state_root, &record)?;
        return Err(format!(
            "invalid_pin; {} attempts remaining",
            MAX_FAILED.saturating_sub(record.failed_attempts)
        ));
    }
    if rebound {
        record.fingerprint = fingerprint;
        record.pin_hash = hash_pin(&record.salt_hex, &record.fingerprint, &pin);
    }
    pin.zeroize();
    record.unlocked = true;
    record.unlocked_epoch = now;
    record.failed_attempts = 0;
    record.locked_until_epoch = 0;
    persist_record(state_root, &record)?;
    Ok(json!({
        "unlocked": true,
        "already": false,
        "detail": if rebound {
            "first-boot PIN unlocked after machine-id commit; product.open stays unavailable"
        } else {
            "first-boot PIN unlocked; product.open stays unavailable"
        }
    }))
}

fn planted_with_empty_machine_id(planted: &str) -> bool {
    let live = launcher_status::live_inputs();
    if !looks_like_machine_id(&live.machine_id) {
        return false;
    }
    let stale = launcher_status::fingerprint_v2(&launcher_status::FingerprintInputs {
        hostname: live.hostname,
        machine_id: "no-machine-id".into(),
        hardware_uuid: live.hardware_uuid,
    });
    planted == stale
}

fn looks_like_machine_id(value: &str) -> bool {
    value.len() == 32 && value.chars().all(|c| c.is_ascii_hexdigit())
}

struct PinRecord {
    fingerprint: String,
    salt_hex: String,
    pin_hash: String,
    issued_epoch: u64,
    unlocked: bool,
    unlocked_epoch: u64,
    failed_attempts: u32,
    locked_until_epoch: u64,
}

#[cfg(test)]
fn persist_new(state_root: &Path, fingerprint: &str, pin: &str) -> Result<Value, String> {
    let mut salt = [0u8; 16];
    getrandom::getrandom(&mut salt).map_err(|error| format!("rng_failed:{error}"))?;
    let salt_hex = hex::encode(salt);
    let record = PinRecord {
        fingerprint: fingerprint.to_string(),
        salt_hex: salt_hex.clone(),
        pin_hash: hash_pin(&salt_hex, fingerprint, pin),
        issued_epoch: now_epoch(),
        unlocked: false,
        unlocked_epoch: 0,
        failed_attempts: 0,
        locked_until_epoch: 0,
    };
    persist_record(state_root, &record)?;
    Ok(json!({
        "issued": true,
        "pin": pin,
        "fingerprint": fingerprint,
        "detail": "first-boot PIN issued once; write it down; product.open stays unavailable"
    }))
}

fn persist_record(state_root: &Path, record: &PinRecord) -> Result<(), String> {
    let seat = state_root.join(SEAT_DIR);
    ensure_secure_directory(&seat, 0o700).map_err(|error| format!("seat_dir_failed:{error}"))?;
    let body = json!({
        "v": 1,
        "fingerprint": record.fingerprint,
        "salt_hex": record.salt_hex,
        "pin_hash": record.pin_hash,
        "issued_epoch": record.issued_epoch,
        "unlocked": record.unlocked,
        "unlocked_epoch": record.unlocked_epoch,
        "failed_attempts": record.failed_attempts,
        "locked_until_epoch": record.locked_until_epoch
    });
    let encoded = serde_json::to_vec_pretty(&body).map_err(|error| format!("encode_failed:{error}"))?;
    atomic_write_file(&seat, RECORD_NAME, &encoded, 0o600)
        .map_err(|error| format!("write_failed:{error}"))?;
    atomic_write_file(&seat, MARKER_NAME, b"issued\n", 0o600)
        .map_err(|error| format!("marker_failed:{error}"))?;
    Ok(())
}

fn load_record(state_root: &Path) -> Option<PinRecord> {
    let path = state_root.join(SEAT_DIR).join(RECORD_NAME);
    let raw = fs::read_to_string(path).ok()?;
    let value = serde_json::from_str::<Value>(&raw).ok()?;
    if value.get("v").and_then(Value::as_u64) != Some(1) {
        return None;
    }
    let fingerprint = value.get("fingerprint").and_then(Value::as_str)?.to_string();
    if !fingerprint.starts_with("fp_v2_") {
        return None;
    }
    let salt_hex = value.get("salt_hex").and_then(Value::as_str)?.to_string();
    let pin_hash = value.get("pin_hash").and_then(Value::as_str)?.to_string();
    if salt_hex.len() != 32 || pin_hash.len() != 64 {
        return None;
    }
    if !salt_hex.chars().all(|c| c.is_ascii_hexdigit())
        || !pin_hash.chars().all(|c| c.is_ascii_hexdigit())
    {
        return None;
    }
    Some(PinRecord {
        fingerprint,
        salt_hex,
        pin_hash,
        issued_epoch: value.get("issued_epoch").and_then(Value::as_u64).unwrap_or(0),
        unlocked: value.get("unlocked").and_then(Value::as_bool).unwrap_or(false),
        unlocked_epoch: value
            .get("unlocked_epoch")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        failed_attempts: value
            .get("failed_attempts")
            .and_then(Value::as_u64)
            .unwrap_or(0) as u32,
        locked_until_epoch: value
            .get("locked_until_epoch")
            .and_then(Value::as_u64)
            .unwrap_or(0),
    })
}

fn hash_pin(salt_hex: &str, fingerprint: &str, pin: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(HASH_DOMAIN);
    hasher.update(salt_hex.as_bytes());
    hasher.update(b"|");
    hasher.update(fingerprint.as_bytes());
    hasher.update(b"|");
    hasher.update(pin.as_bytes());
    hex::encode(hasher.finalize())
}

fn hashes_equal(left: &str, right: &str) -> bool {
    let Ok(a) = hex::decode(left) else {
        return false;
    };
    let Ok(b) = hex::decode(right) else {
        return false;
    };
    if a.len() != 32 || b.len() != 32 {
        return false;
    }
    let mut acc = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        acc |= x ^ y;
    }
    acc == 0
}

fn is_four_digits(pin: &str) -> bool {
    pin.len() == 4 && pin.bytes().all(|b| b.is_ascii_digit())
}

#[cfg(test)]
fn generate_pin() -> Result<String, String> {
    for _ in 0..32 {
        let mut bytes = [0u8; 2];
        getrandom::getrandom(&mut bytes).map_err(|error| format!("rng_failed:{error}"))?;
        let n = u16::from_le_bytes(bytes) as u32;
        const LIMIT: u32 = 65536 - (65536 % 10000);
        if n >= LIMIT {
            continue;
        }
        let digits = n % 10000;
        if digits == 0 || is_trivial(digits) {
            continue;
        }
        return Ok(format!("{digits:04}"));
    }
    Err("rng_failed:pin".into())
}

#[cfg(test)]
fn is_trivial(digits: u32) -> bool {
    matches!(
        digits,
        1111 | 2222 | 3333 | 4444 | 5555 | 6666 | 7777 | 8888 | 9999 | 1234 | 4321
    )
}

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "empiieros-pin-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn consume_requires_unlock_and_bound_fingerprint() {
        let root = temp_root();
        assert_eq!(require_unlocked(&root).unwrap_err(), "pin_not_issued");
        let planted = issue(&root, json!({})).expect("plant");
        assert_eq!(require_unlocked(&root).unwrap_err(), "pin_not_unlocked");
        let pin = planted["pin"].as_str().unwrap().to_string();
        unlock(&root, json!({ "pin": pin })).expect("unlock");
        require_unlocked(&root).expect("unlocked");
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn issue_returns_pin_once_and_never_stores_it() {
        let root = temp_root();
        let first = issue(&root, json!({})).expect("issue");
        let pin = first["pin"].as_str().expect("pin").to_string();
        assert!(is_four_digits(&pin), "{pin}");
        assert_ne!(pin, "0000");
        let record = fs::read_to_string(root.join("seat/first_boot_pin.json")).unwrap();
        assert!(!record.contains(&pin), "plaintext pin persisted");
        assert!(record.contains("pin_hash"));
        assert!(record.contains("fp_v2_"));
        let mode = fs::metadata(root.join("seat/first_boot_pin.json"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
        let second = issue(&root, json!({})).expect_err("second");
        assert_eq!(second, "already_issued");
        let flags = observe_flags(&root, first["fingerprint"].as_str().unwrap());
        assert!(flags.issued);
        assert!(!flags.unlocked);
        assert!(flags.bound);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn unlock_rejects_wrong_pin_then_accepts_right_pin() {
        let root = temp_root();
        let issued = issue(&root, json!({})).expect("issue");
        let pin = issued["pin"].as_str().unwrap().to_string();
        let wrong = unlock(&root, json!({"pin": "0001"})).expect_err("wrong");
        assert!(wrong.starts_with("invalid_pin"), "{wrong}");
        assert!(!wrong.contains(&pin));
        let ok = unlock(&root, json!({"pin": pin})).expect("unlock");
        assert_eq!(ok["unlocked"], true);
        let again = unlock(&root, json!({})).expect("already");
        assert_eq!(again["already"], true);
        let flags = observe_flags(&root, issued["fingerprint"].as_str().unwrap());
        assert!(flags.unlocked);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn five_failures_lock_and_hash_binds_fingerprint() {
        let root = temp_root();
        let issued = issue(&root, json!({})).expect("issue");
        for _ in 0..4 {
            let err = unlock(&root, json!({"pin": "0001"})).expect_err("fail");
            assert!(err.starts_with("invalid_pin"), "{err}");
        }
        let locked = unlock(&root, json!({"pin": "0001"})).expect_err("lock");
        assert_eq!(locked, "pin_locked");
        let right = issued["pin"].as_str().unwrap();
        let still = unlock(&root, json!({"pin": right})).expect_err("still");
        assert_eq!(still, "pin_locked");
        let other = hash_pin("ab".repeat(16).as_str(), "fp_v2_aaa", "2468");
        let same_pin_other_fp = hash_pin("ab".repeat(16).as_str(), "fp_v2_bbb", "2468");
        assert_ne!(other, same_pin_other_fp);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn fingerprint_mismatch_is_fail_closed() {
        let root = temp_root();
        let issued = issue(&root, json!({})).expect("issue");
        let pin = issued["pin"].as_str().unwrap().to_string();
        let mut record = load_record(&root).unwrap();
        record.fingerprint = "fp_v2_ffffffffffffffffffffffffffffffff".into();
        persist_record(&root, &record).unwrap();
        let err = unlock(&root, json!({"pin": pin})).expect_err("mismatch");
        assert_eq!(err, "fingerprint_mismatch");
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn empty_machine_id_plant_rebinds_on_unlock() {
        let root = temp_root();
        let live = crate::launcher_status::live_inputs();
        if live.machine_id.len() != 32 {
            fs::remove_dir_all(root).ok();
            return;
        }
        let pin = "2468";
        let stale = crate::launcher_status::fingerprint_v2(
            &crate::launcher_status::FingerprintInputs {
                hostname: live.hostname.clone(),
                machine_id: "no-machine-id".into(),
                hardware_uuid: live.hardware_uuid.clone(),
            },
        );
        persist_new(&root, &stale, pin).expect("plant stale");
        let before = load_record(&root).unwrap();
        assert_eq!(before.fingerprint, stale);
        let wrong = unlock(&root, json!({"pin": "0000"})).expect_err("wrong");
        assert!(wrong.starts_with("invalid_pin"), "{wrong}");
        let still = load_record(&root).unwrap();
        assert_eq!(still.fingerprint, stale);
        let out = unlock(&root, json!({"pin": pin})).expect("rebind");
        assert_eq!(out["unlocked"], true);
        let after = load_record(&root).unwrap();
        assert_eq!(after.fingerprint, crate::launcher_status::live_fingerprint_v2());
        assert_ne!(after.fingerprint, stale);
        fs::remove_dir_all(root).ok();
    }
}
