//! Binding ticket for Keycloak PAR. Ticket stays in the helper POST, never
//! on the browser URL. Matches living FoFline `native_oidc/ticket.rs`.

use crate::https_vip::{self, CHALLENGE_PATH, TICKET_PATH};
use crate::oidc_urls::{BAKED_FOFLINE_REDIRECT, CLIENT_ID};
use crate::state_store::atomic_write_file;
use ed25519_dalek::{Signer, SigningKey};
use serde::Deserialize;
use serde_json::{json, Value};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use zeroize::Zeroize;

const DEVICE_DIR: &str = "device_keys";
const SIGNING_NAME: &str = "signing.sk.hex";
const CHANNEL: &str = "fofline";

#[derive(Deserialize)]
struct ChallengeBody {
    nonce: Option<String>,
}

#[derive(Deserialize)]
struct TicketBody {
    ticket: Option<String>,
    ticket_id: Option<String>,
}

pub fn issue(state_root: &Path, pkce_challenge: &str, dpop_jkt: &str) -> Result<String, String> {
    let license = load_license_uuid(state_root)?;
    let (mut secret, pubkey_hex) = load_or_create_signing(state_root)?;
    let challenge_body = json!({ "device_pubkey": pubkey_hex }).to_string();
    let (challenge_status, challenge_raw) = https_vip::post_json(CHALLENGE_PATH, &challenge_body)?;
    if challenge_status != 200 {
        secret.zeroize();
        eprintln!("empiieros-runtime: oidc ticket challenge status={challenge_status}");
        return Err(format!("oidc_challenge_http_{challenge_status}"));
    }
    let challenge: ChallengeBody =
        serde_json::from_str(&challenge_raw).map_err(|_| "oidc_challenge_json".to_string())?;
    let nonce = challenge
        .nonce
        .as_deref()
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .ok_or_else(|| "oidc_challenge_nonce".to_string())?;
    let jti = crate::oidc_pkce::random_b64url_32()?;
    let issued_at = crate::https_auth::issuer_http_date()
        .map(|item| item as i64)
        .unwrap_or_else(now_unix);
    let transcript = build_transcript(
        nonce,
        &license,
        CLIENT_ID,
        CHANNEL,
        dpop_jkt,
        pkce_challenge,
        BAKED_FOFLINE_REDIRECT,
        &jti,
        issued_at,
    );
    let signing = SigningKey::from_bytes(&secret);
    secret.zeroize();
    let signature_hex = hex::encode(signing.sign(&transcript).to_bytes());
    let ticket_body = json!({
        "device_pubkey": pubkey_hex,
        "device_signature_hex": signature_hex,
        "nonce": nonce,
        "license_id": license,
        "device_id": Value::Null,
        "client_id": CLIENT_ID,
        "channel": CHANNEL,
        "dpop_jkt": dpop_jkt,
        "pkce_challenge": pkce_challenge,
        "redirect_uri": BAKED_FOFLINE_REDIRECT,
        "jti": jti,
        "issued_at": issued_at
    })
    .to_string();
    let (ticket_status, ticket_raw) = https_vip::post_json(TICKET_PATH, &ticket_body)?;
    if ticket_status != 200 {
        eprintln!("empiieros-runtime: oidc ticket status={ticket_status}");
        return Err(format!("oidc_ticket_http_{ticket_status}"));
    }
    let parsed: TicketBody =
        serde_json::from_str(&ticket_raw).map_err(|_| "oidc_ticket_json".to_string())?;
    let ticket = parsed
        .ticket
        .as_deref()
        .map(str::trim)
        .filter(|item| !item.is_empty() && item.len() <= 2048)
        .ok_or_else(|| "oidc_ticket_empty".to_string())?;
    if parsed.ticket_id.as_deref().map(str::trim).unwrap_or("").is_empty() {
        return Err("oidc_ticket_id".into());
    }
    Ok(ticket.to_string())
}

fn load_license_uuid(state_root: &Path) -> Result<String, String> {
    let raw = fs::read_to_string(state_root.join("capsule_inner").join("entitlement.json"))
        .map_err(|_| "oidc_license_missing".to_string())?;
    let value: Value = serde_json::from_str(&raw).map_err(|_| "oidc_license_json".to_string())?;
    let license = value
        .get("license_id")
        .and_then(Value::as_str)
        .or_else(|| {
            value
                .get("payload")
                .and_then(Value::as_object)
                .and_then(|payload| payload.get("license_id").and_then(Value::as_str))
        })
        .map(str::trim)
        .filter(|item| is_uuid(item))
        .ok_or_else(|| "oidc_license_uuid".to_string())?;
    Ok(license.to_string())
}

fn load_or_create_signing(state_root: &Path) -> Result<([u8; 32], String), String> {
    let dir = state_root.join(DEVICE_DIR);
    fs::create_dir_all(&dir).map_err(|error| format!("oidc_signing_dir:{error}"))?;
    #[cfg(unix)]
    {
        let _ = fs::set_permissions(&dir, fs::Permissions::from_mode(0o700));
    }
    let path = dir.join(SIGNING_NAME);
    if path.is_file() {
        let hex_sk = fs::read_to_string(&path).map_err(|error| format!("oidc_signing_read:{error}"))?;
        let bytes = hex::decode(hex_sk.trim()).map_err(|_| "oidc_signing_hex".to_string())?;
        if bytes.len() != 32 {
            return Err("oidc_signing_len".into());
        }
        let mut secret = [0u8; 32];
        secret.copy_from_slice(&bytes);
        let pubkey = hex::encode(SigningKey::from_bytes(&secret).verifying_key().as_bytes());
        return Ok((secret, pubkey));
    }
    let mut secret = [0u8; 32];
    getrandom::getrandom(&mut secret).map_err(|error| format!("oidc_signing_random:{error}"))?;
    let pubkey = hex::encode(SigningKey::from_bytes(&secret).verifying_key().as_bytes());
    atomic_write_file(&dir, SIGNING_NAME, hex::encode(secret).as_bytes(), 0o600)
        .map_err(|error| format!("oidc_signing_write:{error}"))?;
    Ok((secret, pubkey))
}

fn build_transcript(
    nonce: &str,
    license_id: &str,
    client_id: &str,
    channel: &str,
    dpop_jkt: &str,
    pkce_challenge: &str,
    redirect_uri: &str,
    jti: &str,
    issued_at: i64,
) -> Vec<u8> {
    let mut map = serde_json::Map::new();
    map.insert("channel".into(), json!(channel));
    map.insert("client_id".into(), json!(client_id));
    map.insert("device_id".into(), Value::Null);
    map.insert("dpop_jkt".into(), json!(dpop_jkt));
    map.insert("issued_at".into(), json!(issued_at));
    map.insert("jti".into(), json!(jti));
    map.insert("license_id".into(), json!(license_id));
    map.insert("nonce".into(), json!(nonce));
    map.insert("pkce_challenge".into(), json!(pkce_challenge));
    map.insert("redirect_uri".into(), json!(redirect_uri));
    serde_json::to_vec(&Value::Object(map)).unwrap_or_default()
}

fn is_uuid(raw: &str) -> bool {
    let trimmed = raw.trim();
    if trimmed.len() != 36 {
        return false;
    }
    let bytes = trimmed.as_bytes();
    bytes[8] == b'-'
        && bytes[13] == b'-'
        && bytes[18] == b'-'
        && bytes[23] == b'-'
        && trimmed.bytes().all(|item| item.is_ascii_hexdigit() || item == b'-')
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|item| item.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{build_transcript, is_uuid};

    #[test]
    fn transcript_is_canonical_json() {
        let bytes = build_transcript(
            "n1",
            "lic-uuid",
            "empery-fofline",
            "fofline",
            "jkt1",
            "pkce1",
            "https://app.eliteew.ru/fofline-oidc-bridge.html",
            "jti1",
            1_700_000_000,
        );
        let expected = br#"{"channel":"fofline","client_id":"empery-fofline","device_id":null,"dpop_jkt":"jkt1","issued_at":1700000000,"jti":"jti1","license_id":"lic-uuid","nonce":"n1","pkce_challenge":"pkce1","redirect_uri":"https://app.eliteew.ru/fofline-oidc-bridge.html"}"#;
        assert_eq!(bytes, expected);
        assert!(is_uuid("11111111-1111-1111-1111-111111111111"));
        assert!(!is_uuid("lic-1"));
    }
}
