//! Native FoFline OIDC: Hub starts login, helper exchanges the code, Shell injects
//! access/id tokens only. Refresh token never leaves Runtime IPC.

use crate::oidc_urls::{self, BAKED_FOFLINE_REDIRECT, BAKED_TOKEN, CLIENT_ID};
use crate::state_store::atomic_write_file;
use serde::Deserialize;
use serde_json::{json, Value};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
#[cfg(test)]
use std::time::{SystemTime, UNIX_EPOCH};
use zeroize::Zeroize;

const OIDC_DIR: &str = "oidc";
const TX_NAME: &str = "tx.json";
const SESSION_NAME: &str = "session.json";
const DPOP_NAME: &str = "dpop.sk";

#[derive(Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    id_token: Option<String>,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
    error: Option<String>,
}

pub fn session_flag(state_root: &Path) -> &'static str {
    if session_present(state_root) {
        "present"
    } else {
        "missing"
    }
}

pub fn session_present(state_root: &Path) -> bool {
    load_session(state_root)
        .ok()
        .and_then(|value| {
            let token = value.get("access_token").and_then(Value::as_str)?;
            if !jwt_safe(token) {
                return None;
            }
            let expires_at = value.get("expires_at").and_then(Value::as_u64)?;
            (expires_at > now_epoch() + 30).then_some(())
        })
        .is_some()
}

pub fn start_block(state_root: &Path) -> Option<&'static str> {
    crate::launcher_pin::access_block(state_root).or_else(|| fofline_channel_block(state_root))
}

/// True when `product.open` may try a session: a live access token, or a
/// refresh token `ensure_fresh_session` will spend. A failed refresh stays
/// an operation error.
pub fn session_can_open(state_root: &Path) -> bool {
    session_present(state_root) || peek_refresh_token(state_root).is_some()
}

pub fn start(state_root: &Path, _params: Value) -> Result<Value, String> {
    if let Some(code) = start_block(state_root) {
        return Err(code.into());
    }
    let mut secret = load_or_create_dpop(state_root)?;
    let dpop_jkt = crate::oidc_dpop::jkt_from_secret(&secret)?;
    secret.zeroize();
    let verifier = crate::oidc_pkce::random_b64url_32()?;
    let challenge = crate::oidc_pkce::pkce_challenge_s256(&verifier)?;
    let state = crate::oidc_pkce::random_b64url_32()?;
    let nonce = crate::oidc_pkce::random_b64url_32()?;
    let tx_id = crate::oidc_pkce::random_b64url_32()?;
    write_json(
        state_root,
        TX_NAME,
        json!({
            "id": tx_id,
            "state": state,
            "nonce": nonce,
            "pkce_verifier": verifier,
            "pkce_challenge": challenge,
            "dpop_jkt": dpop_jkt,
            "redirect_uri": BAKED_FOFLINE_REDIRECT,
            "client_id": CLIENT_ID,
            "created": now_epoch()
        }),
    )?;
    if helper_installed() && !helper_process() {
        return activate_via_spa_helper(state_root, "oidc_par");
    }
    execute_par(state_root)
}

pub fn complete(state_root: &Path, params: Value) -> Result<Value, String> {
    crate::launcher_pin::require_unlocked(state_root)?;
    require_fofline_channel(state_root)?;
    let callback = params
        .get("callback_url")
        .and_then(Value::as_str)
        .ok_or_else(|| "callback_url".to_string())?;
    let tx = load_json(state_root, TX_NAME).ok_or_else(|| "oidc_tx_missing".to_string())?;
    let expected_state = tx
        .get("state")
        .and_then(Value::as_str)
        .ok_or_else(|| "oidc_tx_state".to_string())?;
    // Helper `oidc_token` re-parses this URL. Mux Runtime and spa-fetch
    // together when `auth_code_ok` changes, or the helper stays on the
    // old charset and returns `callback_code` after Runtime already wrote
    // callback.json.
    let _code = oidc_urls::parse_callback_code(callback, expected_state)?;
    write_json(state_root, "callback.json", json!({ "callback_url": callback }))?;
    if helper_installed() && !helper_process() {
        activate_via_spa_helper(state_root, "oidc_token")?;
    } else {
        execute_token(state_root)?;
    }
    Ok(json!({
        "oidc_session": "present",
        "detail": "native login completed; secrets stayed in Runtime"
    }))
}

pub fn logout(state_root: &Path, _params: Value) -> Result<Value, String> {
    crate::launcher_pin::require_unlocked(state_root)?;
    wipe_oidc_file(state_root, SESSION_NAME);
    wipe_oidc_file(state_root, TX_NAME);
    wipe_oidc_file(state_root, "callback.json");
    if session_present(state_root) {
        return Err("oidc_logout_incomplete".into());
    }
    Ok(json!({
        "oidc_session": "missing",
        "detail": "native session cleared; secrets stayed in Runtime"
    }))
}

fn refresh_now(state_root: &Path) -> Result<(), String> {
    crate::launcher_pin::require_unlocked(state_root)?;
    if peek_refresh_token(state_root).is_none() {
        return Err("oidc_session_required".into());
    }
    if helper_installed() && !helper_process() {
        activate_via_spa_helper(state_root, "oidc_refresh")?;
    } else {
        execute_refresh(state_root)?;
    }
    if session_present(state_root) {
        Ok(())
    } else {
        Err("oidc_session_required".into())
    }
}

pub fn ensure_fresh_session(state_root: &Path) -> Result<(), String> {
    crate::launcher_pin::require_unlocked(state_root)?;
    if session_present(state_root) {
        return Ok(());
    }
    refresh_now(state_root)
}

pub fn inject(state_root: &Path, params: Value) -> Result<Value, String> {
    let force = params.get("force").and_then(Value::as_bool).unwrap_or(false);
    if force {
        if let Err(error) = refresh_now(state_root) {
            if !session_present(state_root) {
                return Err(error);
            }
        }
    } else {
        ensure_fresh_session(state_root)?;
    }
    let session = load_session(state_root)?;
    let token = session
        .get("access_token")
        .and_then(Value::as_str)
        .filter(|item| jwt_safe(item))
        .ok_or_else(|| "oidc_session_required".to_string())?;
    let id_token = session
        .get("id_token")
        .and_then(Value::as_str)
        .filter(|item| jwt_safe(item));
    if !session_present(state_root) {
        return Err("oidc_session_required".into());
    }
    Ok(json!({
        "ok": true,
        "token": token,
        "idToken": id_token
    }))
}

pub fn execute_par(state_root: &Path) -> Result<Value, String> {
    crate::launcher_pin::require_unlocked(state_root)?;
    require_fofline_channel(state_root)?;
    let tx = load_json(state_root, TX_NAME).ok_or_else(|| "oidc_tx_missing".to_string())?;
    let state = tx
        .get("state")
        .and_then(Value::as_str)
        .ok_or_else(|| "oidc_tx_state".to_string())?;
    let nonce = tx
        .get("nonce")
        .and_then(Value::as_str)
        .ok_or_else(|| "oidc_tx_nonce".to_string())?;
    let challenge = tx
        .get("pkce_challenge")
        .and_then(Value::as_str)
        .ok_or_else(|| "oidc_tx_challenge".to_string())?;
    let dpop_jkt = tx
        .get("dpop_jkt")
        .and_then(Value::as_str)
        .ok_or_else(|| "oidc_tx_jkt".to_string())?;
    let ticket = crate::oidc_ticket::issue(state_root, challenge, dpop_jkt)?;
    let form = oidc_urls::par_form(state, nonce, challenge, dpop_jkt, Some(&ticket))?;
    let fields: Vec<&str> = form
        .split('&')
        .filter_map(|pair| pair.split_once('=').map(|(key, _)| key))
        .collect();
    eprintln!(
        "empiieros-runtime: oidc ticket issued len={} prefix_ok={} par_fields={}",
        ticket.len(),
        if ticket.starts_with("atkt_") { 1 } else { 0 },
        fields.join(",")
    );
    let mut secret = load_or_create_dpop(state_root)?;
    let jti = crate::oidc_pkce::random_b64url_32()?;
    let iat = dpop_iat()?;
    let proof =
        crate::oidc_dpop::sign_dpop_proof(&secret, "POST", oidc_urls::BAKED_PAR, iat, &jti)?;
    secret.zeroize();
    let raw = crate::https_auth::post_par(&form, &proof)?;
    #[derive(Deserialize)]
    struct ParResponse {
        request_uri: Option<String>,
        error: Option<String>,
    }
    let parsed: ParResponse =
        serde_json::from_str(&raw).map_err(|_| "oidc_par_json".to_string())?;
    if parsed.error.as_deref().is_some() {
        return Err("oidc_par_denied".into());
    }
    let request_uri = parsed
        .request_uri
        .as_deref()
        .map(str::trim)
        .filter(|item| oidc_urls::request_uri_ok(item))
        .ok_or_else(|| "oidc_request_uri".to_string())?;
    let authorization_url = oidc_urls::build_par_authorization_url(request_uri)?;
    let mut next = tx;
    next["request_uri"] = json!(request_uri);
    write_json(state_root, TX_NAME, next)?;
    Ok(json!({
        "oidc_start": "applied",
        "authorization_url": authorization_url,
        "detail": "native Keycloak PAR login; tokens stay out of JS"
    }))
}

pub fn execute_token(state_root: &Path) -> Result<Value, String> {
    crate::launcher_pin::require_unlocked(state_root)?;
    require_fofline_channel(state_root)?;
    let tx = load_json(state_root, TX_NAME).ok_or_else(|| "oidc_tx_missing".to_string())?;
    let callback = load_json(state_root, "callback.json")
        .ok_or_else(|| "oidc_callback_missing".to_string())?;
    let callback_url = callback
        .get("callback_url")
        .and_then(Value::as_str)
        .ok_or_else(|| "callback_url".to_string())?;
    let expected_state = tx
        .get("state")
        .and_then(Value::as_str)
        .ok_or_else(|| "oidc_tx_state".to_string())?;
    let code = oidc_urls::parse_callback_code(callback_url, expected_state)?;
    let verifier = tx
        .get("pkce_verifier")
        .and_then(Value::as_str)
        .ok_or_else(|| "oidc_tx_verifier".to_string())?;
    let mut secret = load_or_create_dpop(state_root)?;
    let jti = crate::oidc_pkce::random_b64url_32()?;
    let proof = crate::oidc_dpop::sign_dpop_proof(&secret, "POST", BAKED_TOKEN, dpop_iat()?, &jti)?;
    secret.zeroize();
    let form = oidc_urls::form_encode(&[
        ("grant_type", "authorization_code"),
        ("code", &code),
        ("redirect_uri", BAKED_FOFLINE_REDIRECT),
        ("client_id", CLIENT_ID),
        ("code_verifier", verifier),
    ]);
    let raw = crate::https_auth::post_token(&form, &proof)?;
    if raw.contains("refreshToken") && !raw.contains("refresh_token") {
        return Err("oidc_token_shape".into());
    }
    let parsed: TokenResponse =
        serde_json::from_str(&raw).map_err(|_| "oidc_token_json".to_string())?;
    if parsed.error.as_deref().is_some() {
        return Err("oidc_token_denied".into());
    }
    persist_tokens(state_root, &parsed, None, None)?;
    let _ = fs::remove_file(oidc_dir(state_root).join(TX_NAME));
    let _ = fs::remove_file(oidc_dir(state_root).join("callback.json"));
    Ok(json!({ "oidc_session": "present" }))
}

pub fn execute_refresh(state_root: &Path) -> Result<Value, String> {
    crate::launcher_pin::require_unlocked(state_root)?;
    require_fofline_channel(state_root)?;
    let prior = load_session(state_root)?;
    let refresh = prior
        .get("refresh_token")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|item| oidc_urls::refresh_token_ok(item))
        .ok_or_else(|| "oidc_session_required".to_string())?
        .to_string();
    let prior_id = prior
        .get("id_token")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|item| jwt_safe(item))
        .map(str::to_string);
    let mut secret = load_or_create_dpop(state_root)?;
    let jti = crate::oidc_pkce::random_b64url_32()?;
    let proof = crate::oidc_dpop::sign_dpop_proof(&secret, "POST", BAKED_TOKEN, dpop_iat()?, &jti)?;
    secret.zeroize();
    let form = oidc_urls::refresh_form(&refresh)?;
    let raw = crate::https_auth::post_token(&form, &proof).map_err(|error| {
        eprintln!("empiieros-runtime: oidc refresh failed");
        error
    })?;
    if raw.contains("refreshToken") && !raw.contains("refresh_token") {
        return Err("oidc_token_shape".into());
    }
    let parsed: TokenResponse =
        serde_json::from_str(&raw).map_err(|_| "oidc_token_json".to_string())?;
    if parsed.error.as_deref().is_some() {
        wipe_oidc_file(state_root, SESSION_NAME);
        return Err("oidc_token_denied".into());
    }
    persist_tokens(state_root, &parsed, Some(refresh.as_str()), prior_id.as_deref())?;
    Ok(json!({ "oidc_session": "present" }))
}

fn persist_tokens(
    state_root: &Path,
    parsed: &TokenResponse,
    prior_refresh: Option<&str>,
    prior_id: Option<&str>,
) -> Result<(), String> {
    let access = parsed
        .access_token
        .as_deref()
        .map(str::trim)
        .filter(|item| jwt_safe(item))
        .ok_or_else(|| "oidc_token_missing".to_string())?;
    let refresh = parsed
        .refresh_token
        .as_deref()
        .map(str::trim)
        .filter(|item| oidc_urls::refresh_token_ok(item))
        .or(prior_refresh);
    let id_token = parsed
        .id_token
        .as_deref()
        .map(str::trim)
        .filter(|item| jwt_safe(item))
        .or(prior_id);
    let expires_at = now_epoch().saturating_add(parsed.expires_in.unwrap_or(300).min(3600));
    write_json(
        state_root,
        SESSION_NAME,
        json!({
            "access_token": access,
            "id_token": id_token,
            "refresh_token": refresh,
            "expires_at": expires_at,
            "client_id": CLIENT_ID
        }),
    )
}

fn fofline_channel_block(state_root: &Path) -> Option<&'static str> {
    if crate::launcher_inner::observe(state_root).allow_fofline != Some(true) {
        return Some("allow_fofline_required");
    }
    if !crate::launcher_inner::portal_iface_up() {
        return Some("channel_not_up");
    }
    // Auth WebView resolves auth.eliteew.ru via /etc/hosts. PAR uses the VIP
    // IP directly, so a missing map still returns applied and then paints a
    // white Keycloak card. Refuse until host-map is applied.
    if crate::portal_hostmap::observe_default() != "applied" {
        return Some("portal_hostmap_required");
    }
    crate::oidc_dpop_iat::wall_clock_block()
}

fn require_fofline_channel(state_root: &Path) -> Result<(), String> {
    match fofline_channel_block(state_root) {
        Some(code) => Err(code.into()),
        None => Ok(()),
    }
}

fn helper_process() -> bool {
    matches!(
        std::env::var("EMPIIEROS_SPA_HELPER").ok().as_deref(),
        Some("1") | Some("true") | Some("yes")
    )
}

fn helper_installed() -> bool {
    Path::new("/usr/local/sbin/empiieros-spa-fetch").is_file()
        && Path::new("/etc/systemd/system/empiieros-spa-fetch.service").is_file()
}

fn activate_via_spa_helper(state_root: &Path, action: &str) -> Result<Value, String> {
    if action != "oidc_token" && action != "oidc_par" && action != "oidc_refresh" {
        return Err("oidc_helper_action".into());
    }
    let body = json!({
        "state_dir": state_root.display().to_string(),
        "action": action
    });
    atomic_write_file(
        Path::new("/run/empiieros"),
        "spa-fetch.request",
        format!("{body}\n").as_bytes(),
        0o600,
    )
    .map_err(|error| format!("oidc_request_write_failed:{error}"))?;
    let _ = fs::remove_file(Path::new("/run/empiieros/spa-fetch.result"));
    let started = crate::command_runner::run(
        "/usr/bin/systemctl",
        &["start", "empiieros-spa-fetch.service"],
        std::time::Duration::from_secs(90),
    )
    .map_err(|error| format!("oidc_helper_spawn_failed:{error}"))?;
    if !started.success {
        return Err(format!("oidc_helper_start_failed: {}", started.stderr.trim()));
    }
    let raw = fs::read_to_string("/run/empiieros/spa-fetch.result")
        .map_err(|_| "oidc_helper_result_missing".to_string())?;
    if raw.contains("refresh_token") || raw.contains("access_token") || raw.contains("PrivateKey") {
        return Err("oidc_secret_redaction_failed".into());
    }
    let parsed: Value =
        serde_json::from_str(raw.trim()).map_err(|error| format!("oidc_helper_result_json:{error}"))?;
    if parsed.get("ok") != Some(&json!(true)) {
        return Err(parsed
            .get("error")
            .and_then(Value::as_str)
            .unwrap_or("oidc_helper_failed")
            .to_string());
    }
    let result = parsed
        .get("result")
        .cloned()
        .ok_or_else(|| "oidc_helper_result_missing".to_string())?;
    if action == "oidc_token" || action == "oidc_refresh" {
        if !session_present(state_root) {
            return Err("oidc_session_required".into());
        }
        return Ok(json!({ "oidc_session": "present" }));
    }
    let url = result
        .get("authorization_url")
        .and_then(Value::as_str)
        .ok_or_else(|| "oidc_auth_url".to_string())?;
    if !oidc_urls::authorization_url_allowed(url) {
        return Err("oidc_auth_url".into());
    }
    Ok(result)
}

fn oidc_dir(state_root: &Path) -> std::path::PathBuf {
    state_root.join(OIDC_DIR)
}

fn wipe_oidc_file(state_root: &Path, name: &str) {
    let path = oidc_dir(state_root).join(name);
    if let Ok(raw) = fs::read(&path) {
        let mut wiped = raw;
        wiped.zeroize();
        let _ = atomic_write_file(&oidc_dir(state_root), name, &wiped, 0o600);
    }
    let _ = fs::remove_file(path);
}

fn write_json(state_root: &Path, name: &str, body: Value) -> Result<(), String> {
    let dir = oidc_dir(state_root);
    fs::create_dir_all(&dir).map_err(|error| format!("oidc_store:{error}"))?;
    let mut permissions = fs::metadata(&dir)
        .map_err(|error| format!("oidc_store:{error}"))?
        .permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&dir, permissions).map_err(|error| format!("oidc_store:{error}"))?;
    atomic_write_file(&dir, name, format!("{body}\n").as_bytes(), 0o600)
        .map(|_| ())
        .map_err(|error| format!("oidc_store:{error}"))
}

fn load_json(state_root: &Path, name: &str) -> Option<Value> {
    serde_json::from_str(&fs::read_to_string(oidc_dir(state_root).join(name)).ok()?).ok()
}

fn load_session(state_root: &Path) -> Result<Value, String> {
    load_json(state_root, SESSION_NAME).ok_or_else(|| "oidc_session_required".to_string())
}

fn peek_refresh_token(state_root: &Path) -> Option<String> {
    load_json(state_root, SESSION_NAME)
        .and_then(|value| {
            value
                .get("refresh_token")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|item| oidc_urls::refresh_token_ok(item))
                .map(str::to_string)
        })
}

fn load_or_create_dpop(state_root: &Path) -> Result<[u8; 32], String> {
    let dir = oidc_dir(state_root);
    fs::create_dir_all(&dir).map_err(|error| format!("oidc_store:{error}"))?;
    let path = dir.join(DPOP_NAME);
    if let Ok(raw) = fs::read(&path) {
        if raw.len() == 32 {
            let mut secret = [0u8; 32];
            secret.copy_from_slice(&raw);
            return Ok(secret);
        }
    }
    let secret = crate::oidc_dpop::generate_secret()?;
    atomic_write_file(&dir, DPOP_NAME, &secret, 0o600).map_err(|error| format!("oidc_store:{error}"))?;
    Ok(secret)
}

fn jwt_safe(value: &str) -> bool {
    let trimmed = value.trim();
    let dots = trimmed.chars().filter(|item| *item == '.').count();
    dots == 2
        && trimmed.len() >= 16
        && trimmed.len() <= 8192
        && trimmed
            .chars()
            .all(|item| item.is_ascii_alphanumeric() || item == '-' || item == '_' || item == '.')
}

fn now_epoch() -> u64 {
    crate::oidc_dpop_iat::local_unix()
}

fn dpop_iat() -> Result<u64, String> {
    let local = now_epoch();
    let server = crate::https_auth::issuer_http_date().ok_or_else(|| {
        eprintln!("empiieros-runtime: oidc dpop iat missing Date; refusing local fallback");
        "oidc_dpop_date_required".to_string()
    })?;
    let iat = crate::oidc_dpop_iat::choose_dpop_iat_from_http_date(server, local);
    eprintln!(
        "empiieros-runtime: oidc dpop iat source=http_date skew_vs_local_secs={} bias_secs={} iat={iat}",
        server as i64 - local as i64,
        crate::oidc_dpop_iat::DPOP_HTTP_DATE_FORWARD_BIAS_SECS
    );
    Ok(iat)
}

#[cfg(test)]
pub fn plant_session(state_root: &Path) {
    let _ = write_json(
        state_root,
        SESSION_NAME,
        json!({
            "access_token": "aaaaaaaa.bbbbbbbb.cccccccc",
            "id_token": "aaaaaaaa.bbbbbbbb.dddddddd",
            "expires_at": now_epoch() + 3600,
            "client_id": CLIENT_ID
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn temp_root() -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!(
            "empiieros-oidc-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn start_requires_unlock_and_channel() {
        let root = temp_root();
        assert_eq!(start(&root, json!({})).unwrap_err(), "pin_not_issued");
        crate::launcher_pin::plant_unlocked(&root);
        assert_eq!(start(&root, json!({})).unwrap_err(), "allow_fofline_required");
        assert_eq!(session_flag(&root), "missing");
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn inject_omits_refresh_and_requires_session() {
        let root = temp_root();
        crate::launcher_pin::plant_unlocked(&root);
        assert_eq!(inject(&root, json!({})).unwrap_err(), "oidc_session_required");
        plant_session(&root);
        let view = inject(&root, json!({})).unwrap();
        assert_eq!(view["token"], "aaaaaaaa.bbbbbbbb.cccccccc");
        assert_eq!(view["idToken"], "aaaaaaaa.bbbbbbbb.dddddddd");
        assert!(view.get("refresh_token").is_none());
        assert_eq!(session_flag(&root), "present");
        let cleared = logout(&root, json!({})).unwrap();
        assert_eq!(cleared["oidc_session"], "missing");
        assert!(cleared.get("access_token").is_none());
        assert!(cleared.get("refresh_token").is_none());
        assert_eq!(session_flag(&root), "missing");
        assert_eq!(inject(&root, json!({})).unwrap_err(), "oidc_session_required");
        assert_eq!(logout(&root, json!({})).unwrap()["oidc_session"], "missing");
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn expired_access_without_refresh_stays_required() {
        let root = temp_root();
        crate::launcher_pin::plant_unlocked(&root);
        write_json(
            &root,
            SESSION_NAME,
            json!({
                "access_token": "aaaaaaaa.bbbbbbbb.cccccccc",
                "expires_at": 1,
                "client_id": CLIENT_ID
            }),
        )
        .unwrap();
        assert_eq!(session_flag(&root), "missing");
        assert_eq!(
            ensure_fresh_session(&root).unwrap_err(),
            "oidc_session_required"
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn expired_access_with_refresh_needs_channel() {
        let root = temp_root();
        crate::launcher_pin::plant_unlocked(&root);
        write_json(
            &root,
            SESSION_NAME,
            json!({
                "access_token": "aaaaaaaa.bbbbbbbb.cccccccc",
                "refresh_token": "refresh-token-value-0123456789",
                "expires_at": 1,
                "client_id": CLIENT_ID
            }),
        )
        .unwrap();
        assert_eq!(session_flag(&root), "missing");
        assert_eq!(
            ensure_fresh_session(&root).unwrap_err(),
            "allow_fofline_required"
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn force_inject_keeps_unexpired_access_without_refresh() {
        let root = temp_root();
        crate::launcher_pin::plant_unlocked(&root);
        plant_session(&root);
        let view = inject(&root, json!({ "force": true })).unwrap();
        assert_eq!(view["token"], "aaaaaaaa.bbbbbbbb.cccccccc");
        assert!(view.get("refresh_token").is_none());
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn force_inject_expired_without_refresh_stays_required() {
        let root = temp_root();
        crate::launcher_pin::plant_unlocked(&root);
        write_json(
            &root,
            SESSION_NAME,
            json!({
                "access_token": "aaaaaaaa.bbbbbbbb.cccccccc",
                "expires_at": 1,
                "client_id": CLIENT_ID
            }),
        )
        .unwrap();
        assert_eq!(
            inject(&root, json!({ "force": true })).unwrap_err(),
            "oidc_session_required"
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn complete_rejects_wrong_redirect() {
        let root = temp_root();
        crate::launcher_pin::plant_unlocked(&root);
        write_json(
            &root,
            TX_NAME,
            json!({"state":"state01state01state01state01state0"}),
        )
        .unwrap();
        assert_eq!(
            complete(
                &root,
                json!({"callback_url":"https://app.eliteew.ru/?code=abcDEFghij0123456789-_&state=state01state01state01state01state0"})
            )
            .unwrap_err(),
            "allow_fofline_required"
        );
        fs::remove_dir_all(root).ok();
    }
}
