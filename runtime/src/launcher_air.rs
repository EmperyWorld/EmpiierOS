//! Production air activation by access code (EWAE1).
//!
//! Living kinship flow, without Tauri: parse EWAE1 → signed `device_request_v1`
//! → baked WSS → Capsule v3 open → `activation_ack`. Payload, secret, and
//! activation token never enter the Shell.

use crate::command_runner;
use crate::device_request;
use crate::launcher_capsule;
use crate::state_store::atomic_write_file;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::net::{TcpStream, ToSocketAddrs};
use std::path::Path;
use std::time::{Duration, Instant};
use tungstenite::client::{client, IntoClientRequest};
use tungstenite::http::{header, HeaderValue};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{client_tls, Message, WebSocket};
use zeroize::Zeroize;

pub const PREFIX: &str = "EWAE1";
pub const PORT_MIN: u16 = 62200;
pub const PORT_MAX: u16 = 63200;
const SELECTOR_BYTES: usize = 10;
const SELECTOR_CHARS: usize = 16;
const SECRET_BYTES: usize = 16;
const SECRET_CHARS: usize = 26;
const CHECKSUM_CHARS: usize = 4;
const HOST: &str = "app.eliteew.ru";
const PATH: &str = "/api/v1/desktop-client/air-enroll/ws";
const SUBPROTOCOL: &str = "eliteew-air-enroll-v1";
const INGRESS_HEADER: &str = "edge-v1";
const MAX_CODE_BYTES: usize = 128;
const MAX_CAPSULE_BYTES: usize = 256 * 1024;
const MAX_TEXT_FRAME_BYTES: usize = 16 * 1024;
const CONNECT_TIMEOUT_SECS: u64 = 20;
const FRAME_TIMEOUT_SECS: u64 = 30;
const ACK_TIMEOUT_SECS: u64 = 120;
const HELPER: &str = "/usr/local/sbin/empiieros-air-enroll";
const UNIT: &str = "empiieros-air-enroll.service";
const REQUEST_DIR: &str = "/run/empiieros";
const REQUEST_NAME: &str = "air-enroll.request";
const RESULT_NAME: &str = "air-enroll.result";

const CROCKFORD_ALPHABET: &[u8] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

type AirSocket = WebSocket<MaybeTlsStream<TcpStream>>;

struct AccessCode {
    port: u16,
    selector: String,
    secret: String,
}

impl Drop for AccessCode {
    fn drop(&mut self) {
        self.secret.clear();
    }
}

pub fn precondition(state_root: &Path) -> Option<&'static str> {
    match crate::launcher_pin::require_unlocked(state_root) {
        Ok(()) => crate::oidc_dpop_iat::wall_clock_block(),
        Err(_) => crate::launcher_pin::access_block(state_root).or(Some("pin_not_issued")),
    }
}

pub fn activate(state_root: &Path, params: Value) -> Result<Value, String> {
    if let Some(code) = precondition(state_root) {
        return Err(code.into());
    }
    let raw = params
        .get("code")
        .and_then(Value::as_str)
        .ok_or_else(|| "air_code_required".to_string())?;
    if raw.len() > MAX_CODE_BYTES {
        return Err("air_code_too_large".into());
    }
    let parsed = parse_access_code(raw)?;
    let selector_redacted = mask_selector(&parsed.selector);
    let endpoint = format!("wss://{HOST}:{}{PATH}", parsed.port);
    launcher_capsule::ensure_recipient_sk(state_root)?;
    let _ = launcher_capsule::load_issuer_pk(state_root)?;

    if mock_controller_allowed() {
        return mock_bind(state_root, &parsed, &selector_redacted);
    }
    if helper_installed() && !helper_process() {
        return activate_via_helper(state_root, raw);
    }
    exchange_and_bind(state_root, &parsed, &selector_redacted, &endpoint)
}

pub fn helper_main() -> Result<(), String> {
    let request_path = Path::new(REQUEST_DIR).join(REQUEST_NAME);
    let outcome = match helper_exchange(&request_path) {
        Ok(value) => json!({ "ok": true, "result": value }),
        Err(error) => json!({ "ok": false, "error": error }),
    };
    let _ = fs::remove_file(&request_path);
    atomic_write_file(
        Path::new(REQUEST_DIR),
        RESULT_NAME,
        format!("{outcome}\n").as_bytes(),
        0o600,
    )
    .map_err(|error| format!("air_result_write:{error}"))?;
    Ok(())
}

fn helper_exchange(request_path: &Path) -> Result<Value, String> {
    let raw = fs::read_to_string(request_path).map_err(|error| format!("air_request_read:{error}"))?;
    let request: Value =
        serde_json::from_str(&raw).map_err(|error| format!("air_request_json:{error}"))?;
    let state = request
        .get("state_dir")
        .and_then(Value::as_str)
        .ok_or_else(|| "air_request_state_dir".to_string())?;
    crate::launcher_pin::require_unlocked(Path::new(state))?;
    let code = request
        .get("code")
        .and_then(Value::as_str)
        .ok_or_else(|| "air_code_required".to_string())?;
    let parsed = parse_access_code(code)?;
    let selector_redacted = mask_selector(&parsed.selector);
    let endpoint = format!("wss://{HOST}:{}{PATH}", parsed.port);
    exchange_and_bind(Path::new(state), &parsed, &selector_redacted, &endpoint)
}

fn require_persisted_inner(state_root: &Path, result: &Value) -> Result<(), String> {
    let flags = state_root.join("capsule_inner").join("flags.json");
    if !flags.is_file() {
        return Err("air_helper_inner_missing".into());
    }
    let view = crate::launcher_inner::observe(state_root);
    if !view.entitlement_present && !view.channel_present {
        return Err("air_helper_inner_missing".into());
    }
    if result.get("channel_code").is_none() {
        return Err("air_helper_inner_missing".into());
    }
    Ok(())
}

fn helper_process() -> bool {
    matches!(
        std::env::var("EMPIIEROS_AIR_HELPER").ok().as_deref(),
        Some("1") | Some("true") | Some("yes")
    )
}

fn helper_installed() -> bool {
    Path::new(HELPER).is_file()
        && Path::new("/etc/systemd/system").join(UNIT).is_file()
        && Path::new("/usr/bin/systemctl").is_file()
}

fn activate_via_helper(state_root: &Path, code: &str) -> Result<Value, String> {
    let body = json!({
        "code": code,
        "state_dir": state_root.display().to_string(),
    });
    atomic_write_file(
        Path::new(REQUEST_DIR),
        REQUEST_NAME,
        format!("{body}\n").as_bytes(),
        0o600,
    )
    .map_err(|error| format!("air_request_write_failed:{error}"))?;
    let _ = fs::remove_file(Path::new(REQUEST_DIR).join(RESULT_NAME));
    let started = command_runner::run(
        "/usr/bin/systemctl",
        &["start", UNIT],
        Duration::from_secs(ACK_TIMEOUT_SECS + 60),
    )
    .map_err(|error| format!("air_helper_spawn_failed:{error}"))?;
    if !started.success {
        return Err(format!(
            "air_helper_start_failed: {}",
            started.stderr.trim()
        ));
    }
    let raw = fs::read_to_string(Path::new(REQUEST_DIR).join(RESULT_NAME))
        .map_err(|_| "air_helper_result_missing".to_string())?;
    if raw.contains(code) && code.len() > 12 {
        return Err("air_secret_redaction_failed".into());
    }
    let parsed: Value =
        serde_json::from_str(raw.trim()).map_err(|error| format!("air_helper_result_json:{error}"))?;
    if parsed.get("ok") == Some(&json!(true)) {
        let result = parsed
            .get("result")
            .cloned()
            .ok_or_else(|| "air_helper_result_missing".to_string())?;
        require_persisted_inner(state_root, &result)?;
        return Ok(result);
    }
    let error = parsed
        .get("error")
        .and_then(Value::as_str)
        .unwrap_or("air_enroll_failed");
    Err(error.to_string())
}

fn exchange_and_bind(
    state_root: &Path,
    parsed: &AccessCode,
    selector_redacted: &str,
    endpoint: &str,
) -> Result<Value, String> {
    let recipient_sk = launcher_capsule::ensure_recipient_sk(state_root)?;
    let issuer_pk = launcher_capsule::load_issuer_pk(state_root)?;
    let request = device_request::build_signed_request(state_root)?;
    let mut session = AirSession::connect(parsed, &request)?;
    let (bytes, _meta_request_id, redelivery) = match session.receive_capsule() {
        Ok(value) => value,
        Err(error) => {
            session.nack(&error);
            return Err(error);
        }
    };
    let digest = hex::encode(Sha256::digest(&bytes));
    let material = match crate::capsule_v3::open_v3_air_material(&bytes, recipient_sk, issuer_pk) {
        Ok(material) => material,
        Err(error) => {
            session.nack("capsule_open_failed");
            return Err(format!("capsule_open_failed:{error}"));
        }
    };
    let mut token = material.activation_token.unwrap_or_default();
    if token.is_empty() {
        session.nack("activation_token_missing");
        return Err("activation_token_missing".into());
    }
    let ack = match device_request::activation_ack(state_root, &request.request_id, &token) {
        Ok(ack) => ack,
        Err(error) => {
            session.nack("activation_ack_failed");
            token.zeroize();
            return Err(error);
        }
    };
    // Persist before ACK so a stale helper cannot burn the token and drop inner.
    let mut result = match launcher_capsule::bind_verified_blob(state_root, &bytes, "air") {
        Ok(value) => value,
        Err(error) => {
            session.nack("capsule_bind_failed");
            token.zeroize();
            return Err(error);
        }
    };
    if let Err(error) = session.send_ack(&ack) {
        token.zeroize();
        return Err(error);
    }
    token.zeroize();
    result["activated"] = json!(true);
    result["selector_redacted"] = json!(selector_redacted);
    result["port"] = json!(parsed.port);
    result["endpoint"] = json!(endpoint);
    result["subprotocol"] = json!(SUBPROTOCOL);
    result["redelivery"] = json!(redelivery);
    result["capsule_sha256"] = json!(digest);
    result["detail"] = json!(
        "air WSS handshake completed; capsule bound; inner entitlement/channel persisted; product.open stays unavailable"
    );
    let text = result.to_string();
    if text.contains(&parsed.secret) || text.contains("eliteew_desktop_capsule_payload") {
        return Err("air_secret_redaction_failed".into());
    }
    if text.contains("activation_token") || ack.to_string().contains(&parsed.secret) {
        return Err("air_secret_redaction_failed".into());
    }
    Ok(result)
}

fn execute_host() -> String {
    if cfg!(debug_assertions) {
        if let Ok(host) = std::env::var("EMPIIEROS_AIR_EXECUTE_HOST") {
            let host = host.trim();
            if !host.is_empty() {
                return host.to_string();
            }
        }
    }
    HOST.to_string()
}

fn plain_ws_allowed() -> bool {
    cfg!(debug_assertions)
        && matches!(
            std::env::var("EMPIIEROS_AIR_PLAIN_WS").ok().as_deref(),
            Some("1") | Some("true") | Some("yes")
        )
}

struct AirSession {
    socket: AirSocket,
}

impl AirSession {
    fn connect(parsed: &AccessCode, request: &device_request::DeviceRequestV1) -> Result<Self, String> {
        if !(PORT_MIN..=PORT_MAX).contains(&parsed.port) {
            return Err("air_code_port_range".into());
        }
        let _ = rustls::crypto::ring::default_provider().install_default();
        let host = execute_host();
        let scheme = if plain_ws_allowed() { "ws" } else { "wss" };
        if !plain_ws_allowed() && host != HOST {
            return Err("air_ws_host".into());
        }
        if !cfg!(debug_assertions) && scheme != "wss" {
            return Err("air_ws_tls_required".into());
        }
        let url = format!("{scheme}://{host}:{}{PATH}", parsed.port);
        let mut req = url
            .into_client_request()
            .map_err(|error| format!("air_ws_request:{error}"))?;
        if req.uri().path() != PATH {
            return Err("air_ws_path".into());
        }
        req.headers_mut().insert(
            header::SEC_WEBSOCKET_PROTOCOL,
            HeaderValue::from_static(SUBPROTOCOL),
        );
        req.headers_mut().insert(
            header::HeaderName::from_static("x-eliteew-air-ingress"),
            HeaderValue::from_static(INGRESS_HEADER),
        );
        let stream = tcp_connect(&host, parsed.port)?;
        let _ = stream.set_nodelay(true);
        let _ = stream.set_read_timeout(Some(Duration::from_secs(FRAME_TIMEOUT_SECS)));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(FRAME_TIMEOUT_SECS)));
        let (mut socket, _response) = if plain_ws_allowed() {
            client(req, MaybeTlsStream::Plain(stream)).map_err(|error| format!("air_ws_connect:{error}"))?
        } else {
            client_tls(req, stream).map_err(|error| format!("air_ws_connect:{error}"))?
        };
        apply_timeouts(&mut socket);
        let auth = json!({
            "type": "auth_and_request",
            "selector": parsed.selector,
            "secret": parsed.secret,
            "device_request": request,
        });
        let auth_text = serde_json::to_string(&auth).map_err(|error| format!("air_auth_json:{error}"))?;
        if auth_text.len() > MAX_TEXT_FRAME_BYTES {
            return Err("air_auth_frame_too_large".into());
        }
        socket
            .send(Message::Text(auth_text.into()))
            .map_err(|error| format!("air_ws_send_auth:{error}"))?;
        Ok(Self { socket })
    }

    fn receive_capsule(&mut self) -> Result<(Vec<u8>, String, bool), String> {
        let meta = loop {
            match self.recv()? {
                Message::Text(text) => {
                    if text.len() > MAX_TEXT_FRAME_BYTES {
                        return Err("air_ws_frame_too_large".into());
                    }
                    let value: Value =
                        serde_json::from_str(&text).map_err(|_| "air_ws_invalid_json".to_string())?;
                    match value.get("type").and_then(Value::as_str).unwrap_or("") {
                        "claimed" => continue,
                        "capsule_meta" => break value,
                        "failed" => return Err("air_enroll_failed".into()),
                        _ => return Err("air_ws_unexpected_frame".into()),
                    }
                }
                Message::Ping(payload) => {
                    self.socket
                        .send(Message::Pong(payload))
                        .map_err(|error| format!("air_ws_pong:{error}"))?;
                }
                Message::Close(_) => return Err("air_ws_closed".into()),
                Message::Binary(_) => return Err("air_ws_meta_expected".into()),
                Message::Pong(_) | Message::Frame(_) => {}
            }
        };
        let length = meta.get("length").and_then(Value::as_u64).unwrap_or(0) as usize;
        let sha256 = meta
            .get("sha256")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let request_id = meta
            .get("request_id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let redelivery = meta.get("redelivery").and_then(Value::as_bool).unwrap_or(false);
        if length == 0 || length > MAX_CAPSULE_BYTES {
            return Err("air_ws_capsule_size".into());
        }
        if sha256.len() != 64 || !sha256.bytes().all(|item| item.is_ascii_hexdigit()) {
            return Err("air_ws_capsule_digest".into());
        }
        let mut bytes = Vec::with_capacity(length);
        while bytes.len() < length {
            match self.recv()? {
                Message::Binary(chunk) => {
                    if bytes.len() + chunk.len() > length {
                        return Err("air_ws_capsule_oversize".into());
                    }
                    bytes.extend_from_slice(&chunk);
                }
                Message::Ping(payload) => {
                    self.socket
                        .send(Message::Pong(payload))
                        .map_err(|error| format!("air_ws_pong:{error}"))?;
                }
                Message::Text(_) => return Err("air_ws_binary_expected".into()),
                Message::Close(_) => return Err("air_ws_closed".into()),
                Message::Pong(_) | Message::Frame(_) => {}
            }
        }
        if bytes.len() != length {
            return Err("air_ws_capsule_length".into());
        }
        let digest = hex::encode(Sha256::digest(&bytes));
        if !constant_time_eq(&digest, &sha256) {
            return Err("air_ws_capsule_sha256".into());
        }
        Ok((bytes, request_id, redelivery))
    }

    fn send_ack(&mut self, ack: &Value) -> Result<(), String> {
        let text = serde_json::to_string(ack).map_err(|error| format!("air_ack_json:{error}"))?;
        if text.len() > 2 * 1024 {
            return Err("air_ack_too_large".into());
        }
        self.socket
            .send(Message::Text(text.into()))
            .map_err(|error| format!("air_ws_send_ack:{error}"))?;
        let deadline = Instant::now() + Duration::from_secs(ACK_TIMEOUT_SECS);
        while Instant::now() < deadline {
            match self.socket.read() {
                Ok(Message::Text(text)) => {
                    let value: Value =
                        serde_json::from_str(&text).map_err(|_| "air_ws_invalid_json".to_string())?;
                    match value.get("type").and_then(Value::as_str) {
                        Some("completed") => {
                            let _ = self.socket.close(None);
                            return Ok(());
                        }
                        Some("failed") => return Err("air_enroll_ack_failed".into()),
                        _ => continue,
                    }
                }
                Ok(Message::Ping(payload)) => {
                    self.socket
                        .send(Message::Pong(payload))
                        .map_err(|error| format!("air_ws_pong:{error}"))?;
                }
                Ok(Message::Close(_)) => return Err("air_ws_closed".into()),
                Ok(_) => continue,
                Err(error) => return Err(format!("air_ws_recv:{error}")),
            }
        }
        Err("air_ws_ack_timeout".into())
    }

    fn nack(&mut self, code: &str) {
        let safe = code
            .chars()
            .take(64)
            .filter(|item| item.is_ascii_alphanumeric() || *item == '_' || *item == ':')
            .collect::<String>();
        let _ = self.socket.send(Message::Text(
            json!({ "type": "import_nack", "code": safe }).to_string().into(),
        ));
        let _ = self.socket.close(None);
    }

    fn recv(&mut self) -> Result<Message, String> {
        self.socket
            .read()
            .map_err(|error| format!("air_ws_recv:{error}"))
    }
}

fn tcp_connect(host: &str, port: u16) -> Result<TcpStream, String> {
    let addrs = (host, port)
        .to_socket_addrs()
        .map_err(|error| format!("air_ws_resolve:{error}"))?;
    let mut last = "no_address".to_string();
    for addr in addrs {
        match TcpStream::connect_timeout(&addr, Duration::from_secs(CONNECT_TIMEOUT_SECS)) {
            Ok(stream) => return Ok(stream),
            Err(error) => last = error.to_string(),
        }
    }
    Err(format!("air_ws_connect:{last}"))
}

fn apply_timeouts(socket: &mut AirSocket) {
    match socket.get_mut() {
        MaybeTlsStream::Plain(stream) => {
            let _ = stream.set_nodelay(true);
            let _ = stream.set_read_timeout(Some(Duration::from_secs(FRAME_TIMEOUT_SECS)));
            let _ = stream.set_write_timeout(Some(Duration::from_secs(FRAME_TIMEOUT_SECS)));
        }
        MaybeTlsStream::Rustls(stream) => {
            let tcp = stream.get_mut();
            let _ = tcp.set_nodelay(true);
            let _ = tcp.set_read_timeout(Some(Duration::from_secs(FRAME_TIMEOUT_SECS)));
            let _ = tcp.set_write_timeout(Some(Duration::from_secs(FRAME_TIMEOUT_SECS)));
        }
        #[allow(unreachable_patterns)]
        _ => {}
    }
}

fn mock_controller_allowed() -> bool {
    if !cfg!(debug_assertions) {
        return false;
    }
    match std::env::var("EMPIIEROS_AIR_MOCK_CONTROLLER") {
        Ok(value) => matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on" | "mock"
        ),
        Err(_) => false,
    }
}

fn mock_bind(state_root: &Path, parsed: &AccessCode, selector_redacted: &str) -> Result<Value, String> {
    let recipient_sk = launcher_capsule::ensure_recipient_sk(state_root)?;
    let signing_sk = [0x22u8; 32];
    let blob = crate::capsule_v3::seal_v3_for_test(
        &json!({
            "kind": "eliteew_desktop_capsule_payload",
            "mode": "fofline",
            "allow_fofline": true,
            "wg_conf": "[Interface]\nPrivateKey=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=\n",
            "entitlement": {
                "v": 1,
                "payload": {
                    "license_id": "lic-mock",
                    "ota_artifact_pk": "ab".repeat(32)
                }
            }
        }),
        recipient_sk,
        signing_sk,
        "2099-01-01T00:00:00Z",
        None,
    );
    let mut result = launcher_capsule::bind_verified_blob(state_root, &blob, "air")?;
    result["activated"] = json!(true);
    result["selector_redacted"] = json!(selector_redacted);
    result["port"] = json!(parsed.port);
    result["detail"] = json!(
        "air mock controller bound capsule; inner entitlement/channel persisted; product.open stays unavailable"
    );
    let text = result.to_string();
    if text.contains(&parsed.secret) || text.contains("eliteew_desktop_capsule_payload") {
        return Err("air_secret_redaction_failed".into());
    }
    Ok(result)
}

fn parse_access_code(raw: &str) -> Result<AccessCode, String> {
    let compact: String = raw
        .chars()
        .filter(|item| *item != ' ' && *item != '-')
        .map(|item| item.to_ascii_uppercase())
        .collect();
    let expected = PREFIX.len() + 5 + SELECTOR_CHARS + SECRET_CHARS + CHECKSUM_CHARS;
    if compact.len() != expected {
        return Err("air_code_structure".into());
    }
    if !compact.starts_with(PREFIX) {
        return Err("air_code_prefix".into());
    }
    let port: u16 = compact[PREFIX.len()..PREFIX.len() + 5]
        .parse()
        .map_err(|_| "air_code_port".to_string())?;
    if !(PORT_MIN..=PORT_MAX).contains(&port) {
        return Err("air_code_port_range".into());
    }
    let selector_start = PREFIX.len() + 5;
    let selector = &compact[selector_start..selector_start + SELECTOR_CHARS];
    decode_fixed(selector, SELECTOR_BYTES, SELECTOR_CHARS)?;
    let secret_start = selector_start + SELECTOR_CHARS;
    let secret = &compact[secret_start..secret_start + SECRET_CHARS];
    decode_fixed(secret, SECRET_BYTES, SECRET_CHARS)?;
    let checksum = &compact[secret_start + SECRET_CHARS..];
    let expected_checksum = checksum_for(port, selector, secret)?;
    if !constant_time_eq(checksum, &expected_checksum) {
        return Err("air_code_checksum".into());
    }
    Ok(AccessCode {
        port,
        selector: selector.to_string(),
        secret: secret.to_string(),
    })
}

fn checksum_for(port: u16, selector: &str, secret: &str) -> Result<String, String> {
    let transcript = format!("{PREFIX}|{port:05}|{selector}|{secret}");
    let digest = Sha256::digest(transcript.as_bytes());
    let checksum_bits =
        (u32::from(digest[0]) << 16 | u32::from(digest[1]) << 8 | u32::from(digest[2])) >> 4;
    encode_value_fixed(u128::from(checksum_bits), CHECKSUM_CHARS)
}

fn mask_selector(selector: &str) -> String {
    if selector.len() >= 8 {
        format!("{}...{}", &selector[..4], &selector[selector.len() - 4..])
    } else {
        "***".into()
    }
}

fn crockford_value(ch: u8) -> Option<u8> {
    CROCKFORD_ALPHABET
        .iter()
        .position(|&item| item == ch)
        .map(|index| index as u8)
}

fn encode_value_fixed(mut value: u128, characters: usize) -> Result<String, String> {
    let mut encoded = vec![b'0'; characters];
    for index in (0..characters).rev() {
        encoded[index] = CROCKFORD_ALPHABET[(value & 31) as usize];
        value >>= 5;
    }
    if value != 0 {
        return Err("crockford_output_too_short".into());
    }
    String::from_utf8(encoded).map_err(|_| "crockford_utf8".to_string())
}

fn encode_fixed(raw: &[u8], characters: usize) -> Result<String, String> {
    let mut value: u128 = 0;
    for byte in raw {
        value = (value << 8) | u128::from(*byte);
    }
    encode_value_fixed(value, characters)
}

fn decode_fixed(value: &str, byte_length: usize, characters: usize) -> Result<Vec<u8>, String> {
    if value.len() != characters {
        return Err("crockford_length".into());
    }
    if byte_length == 0 || byte_length > 16 {
        return Err("crockford_byte_length".into());
    }
    let mut decoded: u128 = 0;
    for ch in value.bytes() {
        let number = crockford_value(ch).ok_or_else(|| "crockford_character".to_string())?;
        decoded = (decoded << 5) | u128::from(number);
    }
    if byte_length < 16 {
        let max = 1u128
            .checked_shl((byte_length * 8) as u32)
            .ok_or_else(|| "crockford_padding".to_string())?;
        if decoded >= max {
            return Err("crockford_padding".into());
        }
    }
    let mut raw = vec![0u8; byte_length];
    let mut remaining = decoded;
    for slot in raw.iter_mut().rev() {
        *slot = (remaining & 0xff) as u8;
        remaining >>= 8;
    }
    if remaining != 0 {
        return Err("crockford_padding".into());
    }
    let reencoded = encode_fixed(&raw, characters)?;
    if reencoded != value {
        return Err("crockford_noncanonical".into());
    }
    Ok(raw)
}

fn constant_time_eq(left: &str, right: &str) -> bool {
    let left = left.as_bytes();
    let right = right.as_bytes();
    if left.len() != right.len() {
        return false;
    }
    let mut diff = 0u8;
    for (a, b) in left.iter().zip(right.iter()) {
        diff |= a.to_ascii_lowercase() ^ b.to_ascii_lowercase();
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::{
        activate, checksum_for, encode_fixed, parse_access_code, SECRET_BYTES, SECRET_CHARS,
        SELECTOR_BYTES, SELECTOR_CHARS,
    };
    use serde_json::json;
    use sha2::{Digest, Sha256};
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::net::TcpListener;
    use std::sync::Mutex;
    use std::thread;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tungstenite::handshake::server::{Request, Response};
    use tungstenite::{accept_hdr, Message};

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    const VECTOR: &str = "EWAE1-62555-GY0BVC277PJ1H8PV-5QWG9B35VYHVXRR6XHSD1TH7YM-8VSE";
    const DEV_RECIPIENT_SK_HEX: &str =
        "1111111111111111111111111111111111111111111111111111111111111111";
    const DEV_SIGNING_PK_HEX: &str =
        "a09aa5f47a6759802ff955f8dc2d2a14a5c99d23be97f864127ff9383455a4f0";

    fn temp_root() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("empiieros-air-{nanos}"));
        fs::create_dir_all(root.join("device_keys")).unwrap();
        fs::create_dir_all(root.join("capsule")).unwrap();
        root
    }

    fn force_in_process() {
        std::env::set_var("EMPIIEROS_AIR_HELPER", "1");
        std::env::remove_var("EMPIIEROS_AIR_MOCK_CONTROLLER");
    }

    fn echo_subprotocol(req: &Request, mut response: Response) -> Result<Response, tungstenite::http::Response<Option<String>>> {
        if let Some(proto) = req.headers().get("Sec-WebSocket-Protocol") {
            response.headers_mut().insert("Sec-WebSocket-Protocol", proto.clone());
        }
        Ok(response)
    }

    fn bind_loopback() -> TcpListener {
        for port in [62207_u16, 62208, 62209, 62210] {
            if let Ok(listener) = TcpListener::bind(("127.0.0.1", port)) {
                return listener;
            }
        }
        panic!("no air loopback port in 62207..=62210");
    }

    #[test]
    fn parses_living_vector_and_hides_secret() {
        let parsed = parse_access_code(VECTOR).expect("vector");
        assert_eq!(parsed.port, 62555);
        assert_eq!(parsed.selector, "GY0BVC277PJ1H8PV");
        match parse_access_code("EWAE1-invalid") {
            Err(code) => assert_eq!(code, "air_code_structure"),
            Ok(_) => panic!("invalid code parsed"),
        }
        match parse_access_code("EWAE1-62555-GY0BVC277PJ1H8PV-5QWG9B35VYHVXRR6XHSD1TH7YM-0000") {
            Err(code) => assert_eq!(code, "air_code_checksum"),
            Ok(_) => panic!("bad checksum parsed"),
        }
    }

    #[test]
    fn helper_success_without_inner_is_rejected() {
        let root = temp_root();
        crate::launcher_pin::plant_unlocked(&root);
        let stale = json!({
            "activated": true,
            "bound": true,
            "product_id": "eliteew-fofline"
        });
        assert_eq!(
            super::require_persisted_inner(&root, &stale).unwrap_err(),
            "air_helper_inner_missing"
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn missing_keys_are_created_once() {
        let root = temp_root();
        crate::launcher_pin::plant_unlocked(&root);
        assert!(crate::launcher_capsule::load_recipient_sk(&root).is_none());
        let first = crate::launcher_capsule::ensure_recipient_sk(&root).expect("create");
        let second = crate::launcher_capsule::ensure_recipient_sk(&root).expect("reuse");
        assert_eq!(first, second);
        let meta = fs::metadata(root.join("device_keys/recipient.sk.hex")).unwrap();
        assert_eq!(meta.permissions().mode() & 0o777, 0o600);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn invalid_keys_fail_closed() {
        let root = temp_root();
        assert_eq!(
            activate(&root, json!({ "code": VECTOR })).unwrap_err(),
            "pin_not_issued"
        );
        crate::launcher_pin::plant_unlocked(&root);
        fs::write(root.join("device_keys/recipient.sk.hex"), "not-a-key").unwrap();
        assert_eq!(
            activate(&root, json!({ "code": VECTOR })).unwrap_err(),
            "device_keys_invalid"
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn valid_code_connects_living_frames_or_fails_closed() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        let root = temp_root();
        fs::write(root.join("device_keys/recipient.sk.hex"), DEV_RECIPIENT_SK_HEX).unwrap();
        fs::write(root.join("capsule/signing.pk.hex"), DEV_SIGNING_PK_HEX).unwrap();
        crate::launcher_pin::plant_unlocked(&root);
        force_in_process();
        std::env::set_var("EMPIIEROS_AIR_EXECUTE_HOST", "127.0.0.1");
        std::env::set_var("EMPIIEROS_AIR_PLAIN_WS", "1");
        let error = activate(&root, json!({ "code": VECTOR })).unwrap_err();
        std::env::remove_var("EMPIIEROS_AIR_EXECUTE_HOST");
        std::env::remove_var("EMPIIEROS_AIR_PLAIN_WS");
        std::env::remove_var("EMPIIEROS_AIR_HELPER");
        assert!(
            error.contains("air_ws_connect") || error.contains("air_enroll_failed"),
            "{error}"
        );
        assert!(!error.contains("5QWG9B35VYHVXRR6XHSD1TH7YM"));
        assert!(!root.join("capsule_bind/bound.json").is_file());
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn loopback_wss_frames_bind_without_payload() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        let listener = bind_loopback();
        let port = listener.local_addr().unwrap().port();
        let selector = encode_fixed(&[0x11; SELECTOR_BYTES], SELECTOR_CHARS).unwrap();
        let secret = encode_fixed(&[0x22; SECRET_BYTES], SECRET_CHARS).unwrap();
        let checksum = checksum_for(port, &selector, &secret).unwrap();
        let code = format!("EWAE1-{port:05}-{selector}-{secret}-{checksum}");
        let root = temp_root();
        fs::write(root.join("device_keys/recipient.sk.hex"), DEV_RECIPIENT_SK_HEX).unwrap();
        fs::write(root.join("capsule/signing.pk.hex"), DEV_SIGNING_PK_HEX).unwrap();
        crate::launcher_pin::plant_unlocked(&root);
        let blob = crate::capsule_v3::seal_v3_for_test(
            &json!({
                "kind": "eliteew_desktop_capsule_payload",
                "mode": "fofline",
                "allow_fofline": true,
                "wg_conf": "[Interface]\nPrivateKey=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=\n",
                "entitlement": {
                    "v": 1,
                    "payload": {
                        "license_id": "lic-air-1",
                        "ota_artifact_pk": "cd".repeat(32)
                    }
                },
                "activation_token": "tok_air_lab"
            }),
            [0x11u8; 32],
            [0x22u8; 32],
            "2099-01-01T00:00:00Z",
            None,
        );
        let digest = hex::encode(Sha256::digest(&blob));
        let blob_for_server = blob;
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut socket = accept_hdr(stream, echo_subprotocol).unwrap();
            let auth = match socket.read().unwrap() {
                Message::Text(text) => text.to_string(),
                _ => panic!("auth text"),
            };
            assert!(auth.contains("auth_and_request"));
            assert!(auth.contains("device_request"));
            assert!(auth.contains("device_request_v1"));
            socket
                .send(Message::Text(json!({"type":"claimed"}).to_string().into()))
                .unwrap();
            socket
                .send(Message::Text(
                    json!({
                        "type": "capsule_meta",
                        "length": blob_for_server.len(),
                        "sha256": digest,
                        "request_id": "air_req_lab",
                        "redelivery": false
                    })
                    .to_string()
                    .into(),
                ))
                .unwrap();
            socket.send(Message::Binary(blob_for_server.into())).unwrap();
            let ack = match socket.read().unwrap() {
                Message::Text(text) => text.to_string(),
                _ => panic!("ack text"),
            };
            assert!(ack.contains("activation_ack"));
            assert!(ack.contains("tok_air_lab"));
            assert!(ack.contains("proof_signature"));
            socket
                .send(Message::Text(json!({"type":"completed"}).to_string().into()))
                .unwrap();
            let _ = socket.close(None);
        });
        force_in_process();
        std::env::set_var("EMPIIEROS_AIR_EXECUTE_HOST", "127.0.0.1");
        std::env::set_var("EMPIIEROS_AIR_PLAIN_WS", "1");
        let result = activate(&root, json!({ "code": code })).expect("loopback air");
        std::env::remove_var("EMPIIEROS_AIR_EXECUTE_HOST");
        std::env::remove_var("EMPIIEROS_AIR_PLAIN_WS");
        std::env::remove_var("EMPIIEROS_AIR_HELPER");
        server.join().unwrap();
        assert_eq!(result["activated"], true);
        assert_eq!(result["bound"], true);
        assert_eq!(result["source"], "air");
        assert_eq!(result["entitlement_present"], true);
        assert_eq!(result["channel_present"], true);
        assert_eq!(result["ota_artifact_pk"], "planted");
        assert!(!result.to_string().contains(&"cd".repeat(32)));
        assert_eq!(
            fs::read_to_string(root.join("ota/artifact.pk.hex")).unwrap().trim(),
            "cd".repeat(32)
        );
        assert!(!result.to_string().contains("tok_air_lab"));
        assert!(!result.to_string().contains("eliteew_desktop_capsule_payload"));
        assert!(!result.to_string().contains("activation_token"));
        assert!(!result.to_string().contains("PrivateKey"));
        assert!(root.join("capsule_inner/channel.json").is_file());
        assert!(root.join("capsules/current.ecap").is_file());
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn mock_controller_binds_without_payload() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        let root = temp_root();
        fs::write(root.join("device_keys/recipient.sk.hex"), DEV_RECIPIENT_SK_HEX).unwrap();
        fs::write(root.join("capsule/signing.pk.hex"), DEV_SIGNING_PK_HEX).unwrap();
        crate::launcher_pin::plant_unlocked(&root);
        std::env::set_var("EMPIIEROS_AIR_MOCK_CONTROLLER", "1");
        std::env::remove_var("EMPIIEROS_AIR_PLAIN_WS");
        std::env::remove_var("EMPIIEROS_AIR_EXECUTE_HOST");
        let result = activate(&root, json!({ "code": VECTOR })).unwrap();
        std::env::remove_var("EMPIIEROS_AIR_MOCK_CONTROLLER");
        assert_eq!(result["activated"], true);
        assert_eq!(result["bound"], true);
        assert_eq!(result["source"], "air");
        assert_eq!(result["selector_redacted"], "GY0B...H8PV");
        assert_eq!(result["entitlement_present"], true);
        assert_eq!(result["channel_present"], true);
        assert_eq!(result["channel_valid"], false);
        assert_eq!(result["channel_up"], false);
        assert_eq!(result["allow_fofline"], true);
        assert_eq!(result["ota_artifact_pk"], "planted");
        assert!(!result.to_string().contains(&"ab".repeat(32)));
        let planted = fs::read_to_string(root.join("ota/artifact.pk.hex")).unwrap();
        assert_eq!(planted.trim(), "ab".repeat(32));
        assert!(!result.to_string().contains("eliteew_desktop_capsule_payload"));
        assert!(!result.to_string().contains("5QWG9B35VYHVXRR6XHSD1TH7YM"));
        assert!(!result.to_string().contains("PrivateKey"));
        let status = crate::launcher_status::observe(&root);
        assert_eq!(status["capsule_imported"], true);
        assert_eq!(status["entitlement_present"], true);
        assert_eq!(status["channel_present"], true);
        fs::remove_dir_all(root).ok();
    }
}
