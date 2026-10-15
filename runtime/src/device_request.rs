//! Signed `device_request_v1` — harvested from kinship `device_request.rs`.
//! Private keys never enter IPC.

use crate::launcher_capsule;
use crate::launcher_status;
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use x25519_dalek::{PublicKey, StaticSecret};

pub const KIND: &str = "device_request_v1";
pub const PRODUCT_FOFLINE: &str = "eliteew-fofline";
const SIGNING_SK_FILE: &str = "signing.sk.hex";
const APP_IDENTIFIER: &str = "world.empery.fofline";

#[derive(Debug, Clone, Serialize)]
pub struct DeviceRequestV1 {
    pub kind: String,
    pub request_id: String,
    pub product_id: String,
    pub app_identifier: String,
    pub device_signing_pubkey: String,
    pub capsule_recipient_pubkey: String,
    pub recipient_key_id: String,
    pub machine_fingerprint_hash: String,
    pub created_at: String,
    pub nonce: String,
    pub signature: String,
}

pub fn build_signed_request(state_root: &Path) -> Result<DeviceRequestV1, String> {
    let recipient_sk = launcher_capsule::load_recipient_sk(state_root)
        .ok_or_else(|| "device_keys_missing".to_string())?;
    let recipient_pk = PublicKey::from(&StaticSecret::from(recipient_sk));
    let recipient_pk_hex = hex::encode(recipient_pk.as_bytes());
    let signing_sk = load_or_create_signing_sk(state_root)?;
    let signing = SigningKey::from_bytes(&signing_sk);
    let verifying: VerifyingKey = signing.verifying_key();
    let mut req = DeviceRequestV1 {
        kind: KIND.into(),
        request_id: format!("dr1_{}", random_hex(16)),
        product_id: PRODUCT_FOFLINE.into(),
        app_identifier: APP_IDENTIFIER.into(),
        device_signing_pubkey: hex::encode(verifying.to_bytes()),
        capsule_recipient_pubkey: recipient_pk_hex.clone(),
        recipient_key_id: recipient_key_id(&recipient_pk_hex),
        machine_fingerprint_hash: fingerprint_hash(&launcher_status::live_fingerprint_v2()),
        created_at: now_rfc3339()?,
        nonce: random_hex(32),
        signature: String::new(),
    };
    let msg = signature_transcript(&req);
    req.signature = hex::encode(signing.sign(msg.as_bytes()).to_bytes());
    Ok(req)
}

pub fn signature_transcript(req: &DeviceRequestV1) -> String {
    format!(
        "device_request_v1|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        req.request_id,
        req.product_id,
        req.app_identifier,
        req.device_signing_pubkey,
        req.capsule_recipient_pubkey,
        req.recipient_key_id,
        req.machine_fingerprint_hash,
        req.created_at,
        req.nonce
    )
}

pub fn activation_ack(
    state_root: &Path,
    request_id: &str,
    token: &str,
) -> Result<serde_json::Value, String> {
    if token.trim().is_empty() {
        return Err("activation_token_missing".into());
    }
    let signing_sk = load_or_create_signing_sk(state_root)?;
    let signing = SigningKey::from_bytes(&signing_sk);
    let verifying: VerifyingKey = signing.verifying_key();
    let token_sha256 = hex::encode(Sha256::digest(token.as_bytes()));
    let proof_nonce = random_hex(16);
    let proof_msg = format!("activate_v1|{request_id}|{token_sha256}|{proof_nonce}");
    Ok(serde_json::json!({
        "type": "activation_ack",
        "request_id": request_id,
        "activation_token": token,
        "device_signing_pubkey": hex::encode(verifying.to_bytes()),
        "proof_nonce": proof_nonce,
        "proof_signature": hex::encode(signing.sign(proof_msg.as_bytes()).to_bytes()),
    }))
}

fn load_or_create_signing_sk(state_root: &Path) -> Result<[u8; 32], String> {
    let dir = state_root.join("device_keys");
    fs::create_dir_all(&dir).map_err(|error| format!("device_keys_store_failed:{error}"))?;
    let path = dir.join(SIGNING_SK_FILE);
    if let Ok(raw) = fs::read_to_string(&path) {
        let bytes = hex::decode(raw.trim()).map_err(|_| "invalid_key".to_string())?;
        if bytes.len() == 32 {
            let mut out = [0u8; 32];
            out.copy_from_slice(&bytes);
            return Ok(out);
        }
    }
    let mut sk = [0u8; 32];
    getrandom::getrandom(&mut sk).map_err(|error| format!("rng_failed:{error}"))?;
    fs::write(&path, hex::encode(sk)).map_err(|error| format!("device_keys_store_failed:{error}"))?;
    let mut permissions = fs::metadata(&path)
        .map_err(|error| format!("device_keys_store_failed:{error}"))?
        .permissions();
    permissions.set_mode(0o600);
    fs::set_permissions(&path, permissions)
        .map_err(|error| format!("device_keys_store_failed:{error}"))?;
    Ok(sk)
}

fn recipient_key_id(recipient_pk_hex: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"eliteew-recipient-key-id-v1|");
    hasher.update(recipient_pk_hex.as_bytes());
    format!("rk1_{}", hex::encode(&hasher.finalize()[..8]))
}

fn fingerprint_hash(machine_fingerprint: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"eliteew-machine-fp-hash-v1|");
    hasher.update(machine_fingerprint.as_bytes());
    format!("mfh1_{}", hex::encode(hasher.finalize()))
}

fn random_hex(nbytes: usize) -> String {
    let mut buf = vec![0u8; nbytes];
    getrandom::getrandom(&mut buf).ok();
    hex::encode(buf)
}

fn now_rfc3339() -> Result<String, String> {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("clock:{error}"))?
        .as_secs();
    let z = (secs / 86_400) as i64 + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    let tod = secs % 86_400;
    Ok(format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        tod / 3600,
        (tod % 3600) / 60,
        tod % 60
    ))
}

#[cfg(test)]
mod tests {
    use super::{build_signed_request, signature_transcript, KIND};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn signed_request_matches_living_transcript() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("empiieros-dr-{nanos}"));
        fs::create_dir_all(root.join("device_keys")).unwrap();
        fs::write(
            root.join("device_keys/recipient.sk.hex"),
            "1111111111111111111111111111111111111111111111111111111111111111",
        )
        .unwrap();
        let req = build_signed_request(&root).unwrap();
        assert_eq!(req.kind, KIND);
        assert_eq!(req.product_id, "eliteew-fofline");
        assert!(req.request_id.starts_with("dr1_"));
        assert_eq!(signature_transcript(&req).matches('|').count(), 9);
        fs::remove_dir_all(root).ok();
    }
}
