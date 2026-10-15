//! Capsule v3 crypto-open (EECAP3) — harvested algorithm, Runtime SoT.
//! Returns payload existence only to callers; Shell never sees payload JSON.

use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use hkdf::Hkdf;
use serde_json::Value;
use sha2::{Digest, Sha256};
use x25519_dalek::{PublicKey, StaticSecret};

const CAPSULE_V3_MAGIC: &[u8; 8] = b"EECAP3\0\0";
const CAPSULE_V3_VER: u8 = 3;
const CAPSULE_V3_ALG: u8 = 1;
const HKDF_INFO_V3: &[u8] = b"eliteew-desktop-capsule-v3";
const PAYLOAD_KIND: &str = "eliteew_desktop_capsule_payload";
const EXPIRY_SKEW_SECS: u64 = 5 * 60;
const MIN_BLOB_LEN: usize = 8 + 1 + 1 + 32 + 12 + 4 + 64;

/// Living DEV verifying PK — lab only when EMPIIEROS_CAPSULE_DEV_KEYS=1 (debug).
const DEV_SIGNING_PK_HEX: &str =
    "a09aa5f47a6759802ff955f8dc2d2a14a5c99d23be97f864127ff9383455a4f0";

/// Living EliteEW capsule issuer verifying PK (public). The signing secret
/// never enters EmpiierOS. Metal air consume on 2026-09-17 required this key.
pub const PROD_SIGNING_PK_HEX: &str =
    "26b36dae25359712f8b328482a54f835c558c9bf708fb4a1d513ca7f165992d6";

fn capsule_dev_keys_allowed() -> bool {
    if !cfg!(debug_assertions) {
        return false;
    }
    match std::env::var("EMPIIEROS_CAPSULE_DEV_KEYS") {
        Ok(v) => matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        Err(_) => false,
    }
}

fn decode_key32(raw: &str) -> Result<[u8; 32], String> {
    let bytes = hex::decode(raw.trim()).map_err(|_| "invalid_key".to_string())?;
    if bytes.len() != 32 {
        return Err("invalid_key".into());
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

fn is_dev_signing_pk_hex(raw: &str) -> bool {
    raw.eq_ignore_ascii_case(DEV_SIGNING_PK_HEX)
}

fn decode_optional_pk(raw: &str) -> Option<[u8; 32]> {
    let t = raw.trim();
    if t.is_empty() || t.contains("REPLACE_WITH") {
        return None;
    }
    decode_key32(t).ok()
}

/// Issuer verifying PK: env · bake · state file · living prod · DEV lab.
///
/// A leftover lab DEV `signing.pk.hex` must not shadow the living issuer in
/// release. Debug tests may still use the DEV file.
pub fn load_issuer_verifying_pk(state_root: Option<&Path>) -> Result<[u8; 32], String> {
    if let Ok(raw) = std::env::var("EMPIIEROS_CAPSULE_SIGNING_PK") {
        if let Some(pk) = decode_optional_pk(&raw) {
            return Ok(pk);
        }
    }
    if let Some(raw) = option_env!("EMPIIEROS_CAPSULE_SIGNING_PK") {
        if let Some(pk) = decode_optional_pk(raw) {
            return Ok(pk);
        }
    }
    if let Some(root) = state_root {
        if let Ok(raw) = fs::read_to_string(root.join("capsule").join("signing.pk.hex")) {
            let t = raw.trim();
            if !t.is_empty() && (cfg!(debug_assertions) || !is_dev_signing_pk_hex(t)) {
                if let Some(pk) = decode_optional_pk(t) {
                    return Ok(pk);
                }
            }
        }
    }
    if capsule_dev_keys_allowed() {
        return decode_key32(DEV_SIGNING_PK_HEX);
    }
    decode_key32(PROD_SIGNING_PK_HEX)
}

/// Issuer verifying PK without a state root (env · bake · living prod).
pub fn load_signing_pk() -> Result<[u8; 32], String> {
    load_issuer_verifying_pk(None)
}

fn join_null(parts: &[&[u8]]) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            out.push(0);
        }
        out.extend_from_slice(part);
    }
    out
}

fn aad_encrypt_v3(ephemeral_pk: &[u8], nonce: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 + 2 + 32 + 12);
    out.extend_from_slice(CAPSULE_V3_MAGIC);
    out.push(CAPSULE_V3_VER);
    out.push(CAPSULE_V3_ALG);
    out.extend_from_slice(ephemeral_pk);
    out.extend_from_slice(nonce);
    out
}

fn signature_transcript_v3(
    ephemeral_pk: &[u8],
    nonce: &[u8],
    ciphertext_sha256: &[u8],
    ciphertext: &[u8],
) -> Vec<u8> {
    let ver_alg = [CAPSULE_V3_VER, CAPSULE_V3_ALG];
    join_null(&[
        b"capsule_v3",
        CAPSULE_V3_MAGIC,
        &ver_alg,
        ephemeral_pk,
        nonce,
        ciphertext_sha256,
        ciphertext,
    ])
}

fn parse_rfc3339_utc_secs(raw: &str) -> Result<u64, String> {
    let t = raw.trim();
    let core = if let Some(stripped) = t.strip_suffix('Z') {
        stripped
    } else if let Some(stripped) = t.strip_suffix("+00:00") {
        stripped
    } else {
        return Err("invalid_capsule".into());
    };
    let core = core.split('.').next().unwrap_or(core);
    let (date, time) = core
        .split_once('T')
        .ok_or_else(|| "invalid_capsule".to_string())?;
    let mut dparts = date.split('-');
    let y: i64 = dparts
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| "invalid_capsule".to_string())?;
    let m: u32 = dparts
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| "invalid_capsule".to_string())?;
    let d: u32 = dparts
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| "invalid_capsule".to_string())?;
    if dparts.next().is_some() || !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return Err("invalid_capsule".into());
    }
    let mut tparts = time.split(':');
    let hh: u64 = tparts
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| "invalid_capsule".to_string())?;
    let mm: u64 = tparts
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| "invalid_capsule".to_string())?;
    let ss: u64 = tparts
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| "invalid_capsule".to_string())?;
    if tparts.next().is_some() || hh > 23 || mm > 59 || ss > 60 {
        return Err("invalid_capsule".into());
    }
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u64;
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp as u64 + 2) / 5 + d as u64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = (era * 146_097 + doe as i64 - 719_468) as i64;
    if days < 0 {
        return Err("invalid_capsule".into());
    }
    Ok(days as u64 * 86_400 + hh * 3600 + mm * 60 + ss)
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Redacted open result — never includes payload / tokens / entitlement JSON.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenSummary {
    pub request_id_redacted: String,
    pub product_id: String,
    /// Redacted license id when outer `entitlement` object present — empty otherwise.
    pub license_id_redacted: String,
    pub has_entitlement: bool,
}

fn redact_request_id(raw: &str) -> String {
    let t = raw.trim();
    if t.is_empty() {
        return String::new();
    }
    let h = Sha256::digest(t.as_bytes());
    format!("rid_{}", hex::encode(&h[..6]))
}

fn redact_license_id(raw: &str) -> String {
    let t = raw.trim();
    if t.is_empty() {
        return String::new();
    }
    let h = Sha256::digest(t.as_bytes());
    format!("lic_{}", hex::encode(&h[..6]))
}

fn token_from_value(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(ToOwned::to_owned)
}

fn sanitize_product_id(raw: &str) -> String {
    let t = raw.trim();
    if t.is_empty() || t.len() > 64 {
        return String::new();
    }
    if t
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        t.to_string()
    } else {
        String::new()
    }
}

fn extract_license_id_raw(entitlement: &Value) -> Option<String> {
    let obj = entitlement.as_object()?;
    if let Some(s) = obj.get("license_id").and_then(|v| v.as_str()) {
        if !s.trim().is_empty() {
            return Some(s.to_string());
        }
    }
    if let Some(payload) = obj.get("payload").and_then(|v| v.as_object()) {
        if let Some(s) = payload.get("license_id").and_then(|v| v.as_str()) {
            if !s.trim().is_empty() {
                return Some(s.to_string());
            }
        }
    }
    None
}

/// Test-only wrappers. Living callers use `open_v3_air_material` /
/// `open_v3_inner` / `open_v3_with_entitlement` (crate root is `core.rs`,
/// not the leftover `lib.rs`).
#[cfg(test)]
fn open_v3_bytes_ok(blob: &[u8], recipient_sk_bytes: [u8; 32]) -> Result<(), String> {
    let _ = open_v3_summary(blob, recipient_sk_bytes)?;
    Ok(())
}

#[cfg(test)]
fn open_v3_summary(blob: &[u8], recipient_sk_bytes: [u8; 32]) -> Result<OpenSummary, String> {
    let issuer = load_signing_pk()?;
    open_v3_summary_with_issuer(blob, recipient_sk_bytes, issuer)
}

#[cfg(test)]
fn open_v3_summary_with_issuer(
    blob: &[u8],
    recipient_sk_bytes: [u8; 32],
    issuer_pk: [u8; 32],
) -> Result<OpenSummary, String> {
    let opened = open_v3_bytes(blob, recipient_sk_bytes, issuer_pk)?;
    Ok(opened.summary)
}

pub struct AirMaterial {
    #[allow(dead_code)]
    pub summary: OpenSummary,
    #[allow(dead_code)]
    pub request_id: String,
    pub activation_token: Option<String>,
}

pub fn open_v3_air_material(
    blob: &[u8],
    recipient_sk_bytes: [u8; 32],
    issuer_pk: [u8; 32],
) -> Result<AirMaterial, String> {
    let opened = open_v3_bytes(blob, recipient_sk_bytes, issuer_pk)?;
    Ok(AirMaterial {
        summary: opened.summary,
        request_id: opened.request_id,
        activation_token: opened.activation_token,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InnerFlags {
    pub allow_offline: Option<bool>,
    pub allow_online: Option<bool>,
    pub allow_fofline: Option<bool>,
    pub has_entitlement: bool,
    pub has_channel: bool,
    pub mode: String,
}

#[derive(Debug, Clone)]
pub struct InnerCapsule {
    pub summary: OpenSummary,
    pub entitlement: Option<Value>,
    pub channel: Option<Value>,
    pub flags: InnerFlags,
}

/// Verify + decrypt → Runtime-only inner material. Never send over Shell IPC.
pub fn open_v3_inner(
    blob: &[u8],
    recipient_sk_bytes: [u8; 32],
    issuer_pk: [u8; 32],
) -> Result<InnerCapsule, String> {
    let opened = open_v3_bytes(blob, recipient_sk_bytes, issuer_pk)?;
    Ok(InnerCapsule {
        summary: opened.summary,
        entitlement: opened.entitlement,
        channel: opened.channel,
        flags: opened.flags,
    })
}

/// Verify + decrypt → redacted summary + outer entitlement object (for lab verify only).
/// Entitlement Value must never cross Shell IPC.
#[allow(dead_code)]
pub fn open_v3_with_entitlement(
    blob: &[u8],
    recipient_sk_bytes: [u8; 32],
) -> Result<(OpenSummary, Option<Value>), String> {
    let issuer = load_signing_pk()?;
    let opened = open_v3_bytes(blob, recipient_sk_bytes, issuer)?;
    Ok((opened.summary, opened.entitlement))
}

struct OpenedCapsule {
    summary: OpenSummary,
    entitlement: Option<Value>,
    channel: Option<Value>,
    flags: InnerFlags,
    request_id: String,
    activation_token: Option<String>,
}

fn open_v3_bytes(
    blob: &[u8],
    recipient_sk_bytes: [u8; 32],
    issuer_pk: [u8; 32],
) -> Result<OpenedCapsule, String> {
    if blob.len() < MIN_BLOB_LEN {
        return Err("invalid_capsule".into());
    }
    if !blob.starts_with(CAPSULE_V3_MAGIC) {
        return Err("invalid_capsule".into());
    }
    let ver = blob[8];
    let alg = blob[9];
    if ver != CAPSULE_V3_VER {
        return Err("unsupported_ver".into());
    }
    if alg != CAPSULE_V3_ALG {
        return Err("invalid_capsule".into());
    }
    let eph_pk = &blob[10..42];
    let nonce = &blob[42..54];
    let ct_len = u32::from_be_bytes([blob[54], blob[55], blob[56], blob[57]]) as usize;
    let ct_end = 58 + ct_len;
    if ct_end + 64 != blob.len() {
        return Err("invalid_capsule".into());
    }
    let ciphertext = &blob[58..ct_end];
    let sig_bytes = &blob[ct_end..];
    let ct_hash = Sha256::digest(ciphertext);

    let msg = signature_transcript_v3(eph_pk, nonce, ct_hash.as_slice(), ciphertext);
    let verifying =
        VerifyingKey::from_bytes(&issuer_pk).map_err(|_| "invalid_capsule".to_string())?;
    let signature =
        Signature::from_slice(sig_bytes).map_err(|_| "invalid_capsule".to_string())?;
    verifying
        .verify(&msg, &signature)
        .map_err(|_| "invalid_capsule".to_string())?;

    let aad = aad_encrypt_v3(eph_pk, nonce);
    let mut eph_arr = [0u8; 32];
    eph_arr.copy_from_slice(eph_pk);
    let recipient_sk = StaticSecret::from(recipient_sk_bytes);
    let shared = recipient_sk.diffie_hellman(&PublicKey::from(eph_arr));
    let hk = Hkdf::<Sha256>::new(None, shared.as_bytes());
    let mut key_bytes = [0u8; 32];
    hk.expand(HKDF_INFO_V3, &mut key_bytes)
        .map_err(|_| "invalid_capsule".to_string())?;
    let cipher = ChaCha20Poly1305::new(&key_bytes.into());
    key_bytes.fill(0);
    let nonce12: [u8; 12] = nonce
        .try_into()
        .map_err(|_| "invalid_capsule".to_string())?;
    let nonce_arr = Nonce::from(nonce12);
    let plaintext = cipher
        .decrypt(
            &nonce_arr,
            Payload {
                msg: ciphertext,
                aad: &aad,
            },
        )
        .map_err(|_| "invalid_capsule".to_string())?;
    let text = String::from_utf8(plaintext).map_err(|_| "invalid_capsule".to_string())?;
    let inner: Value =
        serde_json::from_str(&text).map_err(|_| "invalid_capsule".to_string())?;
    let obj = inner
        .as_object()
        .ok_or_else(|| "invalid_capsule".to_string())?;

    if let Some(expires_at) = obj.get("expires_at").and_then(|v| v.as_str()) {
        if !expires_at.is_empty() {
            let expires_secs = parse_rfc3339_utc_secs(expires_at)?;
            if now_unix() > expires_secs.saturating_add(EXPIRY_SKEW_SECS) {
                return Err("expired".into());
            }
        }
    }

    let payload = obj
        .get("payload")
        .ok_or_else(|| "invalid_capsule".to_string())?;
    let payload_obj = payload
        .as_object()
        .ok_or_else(|| "invalid_capsule".to_string())?;
    if payload_obj.get("kind").and_then(|k| k.as_str()) != Some(PAYLOAD_KIND) {
        return Err("invalid_capsule".into());
    }
    if payload_obj.contains_key("license_key") {
        return Err("invalid_capsule".into());
    }
    let request_id = obj
        .get("request_id")
        .and_then(|value| value.as_str())
        .or_else(|| payload_obj.get("request_id").and_then(|value| value.as_str()))
        .unwrap_or("")
        .to_string();
    // Living seal keeps the one-time token inside payload, not the outer envelope.
    let activation_token = token_from_value(payload_obj.get("activation_token"))
        .or_else(|| token_from_value(obj.get("activation_token")));
    let request_id_redacted = redact_request_id(&request_id);
    let product_id = sanitize_product_id(
        obj.get("product_id")
            .and_then(|v| v.as_str())
            .or_else(|| payload_obj.get("product_id").and_then(|v| v.as_str()))
            .unwrap_or(""),
    );
    let entitlement_doc = object_field(obj.get("entitlement"))
        .or_else(|| object_field(payload_obj.get("entitlement")));
    let has_entitlement = entitlement_doc.is_some();
    let license_id_redacted = entitlement_doc
        .as_ref()
        .and_then(extract_license_id_raw)
        .map(|raw| redact_license_id(&raw))
        .unwrap_or_default();
    let channel = channel_from_payload(payload_obj);
    let has_channel = channel_has_transport(&channel);
    let flags = InnerFlags {
        allow_offline: bool_field(payload_obj.get("allow_offline"))
            .or_else(|| bool_field(obj.get("allow_offline"))),
        allow_online: bool_field(payload_obj.get("allow_online"))
            .or_else(|| bool_field(obj.get("allow_online"))),
        allow_fofline: bool_field(payload_obj.get("allow_fofline"))
            .or_else(|| bool_field(obj.get("allow_fofline"))),
        has_entitlement,
        has_channel,
        mode: sanitize_mode(
            payload_obj
                .get("mode")
                .and_then(|v| v.as_str())
                .or_else(|| obj.get("mode").and_then(|v| v.as_str()))
                .unwrap_or(""),
        ),
    };
    Ok(OpenedCapsule {
        summary: OpenSummary {
            request_id_redacted,
            product_id,
            license_id_redacted,
            has_entitlement,
        },
        entitlement: entitlement_doc,
        channel,
        flags,
        request_id,
        activation_token,
    })
}

fn object_field(value: Option<&Value>) -> Option<Value> {
    value.filter(|item| item.is_object()).cloned()
}

fn bool_field(value: Option<&Value>) -> Option<bool> {
    value.and_then(Value::as_bool)
}

fn sanitize_mode(raw: &str) -> String {
    let t = raw.trim();
    if t.is_empty() || t.len() > 32 {
        return String::new();
    }
    if t
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        t.to_string()
    } else {
        String::new()
    }
}

const CHANNEL_KEYS: &[&str] = &[
    "mode",
    "mode_label_ru",
    "mode_hint_ru",
    "allow_offline",
    "allow_online",
    "allow_fofline",
    "wg_conf",
    "peer",
    "label",
    "license_id",
    "edition",
    "profile_id",
    "apps",
    "notes_ru",
    "issued_at",
];

fn channel_from_payload(payload: &serde_json::Map<String, Value>) -> Option<Value> {
    let mut seat = serde_json::Map::new();
    seat.insert(
        "kind".into(),
        Value::String("eliteew_desktop_seat".into()),
    );
    for key in CHANNEL_KEYS {
        if let Some(value) = payload.get(*key) {
            seat.insert((*key).into(), value.clone());
        }
    }
    if seat.len() <= 1 {
        return None;
    }
    Some(Value::Object(seat))
}

fn channel_has_transport(channel: &Option<Value>) -> bool {
    let Some(obj) = channel.as_ref().and_then(Value::as_object) else {
        return false;
    };
    if obj
        .get("wg_conf")
        .and_then(Value::as_str)
        .map(str::trim)
        .is_some_and(|conf| !conf.is_empty())
    {
        return true;
    }
    match obj.get("peer") {
        Some(Value::Object(peer)) => !peer.is_empty(),
        Some(Value::String(peer)) => !peer.trim().is_empty(),
        _ => false,
    }
}

/// Seal a lab blob for tests (known keys). Not a product issuer.
pub fn seal_v3_for_test(
    payload: &Value,
    recipient_sk: [u8; 32],
    signing_sk: [u8; 32],
    expires_at: &str,
    entitlement: Option<Value>,
) -> Vec<u8> {
    use ed25519_dalek::{Signer, SigningKey};

    let recipient_pk = PublicKey::from(&StaticSecret::from(recipient_sk));
    let mut inner = serde_json::json!({
        "product_id": "eliteew-desktop-connected",
        "channel": "connected",
        "request_id": "dr1_test",
        "recipient_key_id": "rk1_test",
        "created_at": "2026-08-12T00:00:00Z",
        "expires_at": expires_at,
        "payload": payload,
    });
    if let Some(ent) = entitlement {
        inner
            .as_object_mut()
            .expect("obj")
            .insert("entitlement".into(), ent);
    }
    if let Some(token) = payload
        .get("activation_token")
        .or_else(|| payload.get("air_activation_token"))
        .and_then(|value| value.as_str())
    {
        inner
            .get_mut("payload")
            .and_then(Value::as_object_mut)
            .expect("payload")
            .insert(
                "activation_token".into(),
                serde_json::Value::String(token.to_string()),
            );
    }
    let plaintext = serde_json::to_vec(&inner).expect("json");
    let eph_sk = StaticSecret::from([9u8; 32]);
    let eph_pk = PublicKey::from(&eph_sk);
    let shared = eph_sk.diffie_hellman(&recipient_pk);
    let hk = Hkdf::<Sha256>::new(None, shared.as_bytes());
    let mut key_bytes = [0u8; 32];
    hk.expand(HKDF_INFO_V3, &mut key_bytes).expect("hkdf");
    let nonce = [7u8; 12];
    let aad = aad_encrypt_v3(eph_pk.as_bytes(), &nonce);
    let cipher = ChaCha20Poly1305::new(&key_bytes.into());
    let ct = cipher
        .encrypt(
            &nonce.into(),
            Payload {
                msg: plaintext.as_ref(),
                aad: &aad,
            },
        )
        .expect("encrypt");
    let ct_hash = Sha256::digest(&ct);
    let msg = signature_transcript_v3(eph_pk.as_bytes(), &nonce, ct_hash.as_slice(), &ct);
    let sk = SigningKey::from_bytes(&signing_sk);
    let sig = sk.sign(&msg);
    let mut blob = Vec::new();
    blob.extend_from_slice(CAPSULE_V3_MAGIC);
    blob.push(CAPSULE_V3_VER);
    blob.push(CAPSULE_V3_ALG);
    blob.extend_from_slice(eph_pk.as_bytes());
    blob.extend_from_slice(&nonce);
    blob.extend_from_slice(&(ct.len() as u32).to_be_bytes());
    blob.extend_from_slice(&ct);
    blob.extend_from_slice(&sig.to_bytes());
    blob
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEV_SIGNING_SK_HEX: &str =
        "2222222222222222222222222222222222222222222222222222222222222222";
    const DEV_RECIPIENT_SK_HEX: &str =
        "1111111111111111111111111111111111111111111111111111111111111111";

    #[test]
    fn open_roundtrip_dev_keys() {
        std::env::set_var("EMPIIEROS_CAPSULE_DEV_KEYS", "1");
        std::env::set_var("EMPIIEROS_CAPSULE_SIGNING_PK", DEV_SIGNING_PK_HEX);
        let recipient_sk = decode_key32(DEV_RECIPIENT_SK_HEX).unwrap();
        let signing_sk = decode_key32(DEV_SIGNING_SK_HEX).unwrap();
        let payload = serde_json::json!({
            "kind": PAYLOAD_KIND,
            "mode": "online",
        });
        let blob = seal_v3_for_test(&payload, recipient_sk, signing_sk, "2099-01-01T00:00:00Z", None);
        assert!(open_v3_bytes_ok(&blob, recipient_sk).is_ok());
        let bad = [0u8; 32];
        assert_eq!(open_v3_bytes_ok(&blob, bad).unwrap_err(), "invalid_capsule");
    }

    #[test]
    fn air_material_reads_living_payload_token() {
        let recipient_sk = decode_key32(DEV_RECIPIENT_SK_HEX).unwrap();
        let signing_sk = decode_key32(DEV_SIGNING_SK_HEX).unwrap();
        let issuer_pk = decode_key32(DEV_SIGNING_PK_HEX).unwrap();
        let blob = seal_v3_for_test(
            &serde_json::json!({
                "kind": PAYLOAD_KIND,
                "activation_token": "tok_living_shape",
            }),
            recipient_sk,
            signing_sk,
            "2099-01-01T00:00:00Z",
            None,
        );
        let material = open_v3_air_material(&blob, recipient_sk, issuer_pk).unwrap();
        assert_eq!(material.activation_token.as_deref(), Some("tok_living_shape"));
    }

    #[test]
    fn inner_reads_payload_entitlement_and_channel() {
        let recipient_sk = decode_key32(DEV_RECIPIENT_SK_HEX).unwrap();
        let signing_sk = decode_key32(DEV_SIGNING_SK_HEX).unwrap();
        let issuer_pk = decode_key32(DEV_SIGNING_PK_HEX).unwrap();
        let blob = seal_v3_for_test(
            &serde_json::json!({
                "kind": PAYLOAD_KIND,
                "mode": "fofline",
                "allow_fofline": true,
                "allow_online": false,
                "wg_conf": "[Interface]\nPrivateKey=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=\n",
                "activation_token": "tok_must_not_enter_channel",
                "entitlement": {"license_id": "lic-live-1"},
            }),
            recipient_sk,
            signing_sk,
            "2099-01-01T00:00:00Z",
            None,
        );
        let inner = open_v3_inner(&blob, recipient_sk, issuer_pk).unwrap();
        assert!(inner.flags.has_entitlement);
        assert!(inner.flags.has_channel);
        assert_eq!(inner.flags.allow_fofline, Some(true));
        assert_eq!(inner.summary.product_id, "eliteew-desktop-connected");
        let channel = inner.channel.expect("channel");
        assert!(channel.get("wg_conf").is_some());
        assert!(channel.get("activation_token").is_none());
        assert!(inner.entitlement.is_some());
    }

    #[test]
    fn prod_issuer_pk_is_not_dev_vector() {
        let pk = decode_key32(PROD_SIGNING_PK_HEX).unwrap();
        assert_eq!(hex::encode(pk), PROD_SIGNING_PK_HEX);
        assert!(is_dev_signing_pk_hex(DEV_SIGNING_PK_HEX));
        assert!(!is_dev_signing_pk_hex(PROD_SIGNING_PK_HEX));
    }
}
