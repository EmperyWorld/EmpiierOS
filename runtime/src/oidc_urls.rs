//! Baked FoFline native OIDC URLs. Caller never supplies the IdP host.

pub const BAKED_AUTH: &str =
    "https://auth.eliteew.ru/realms/empery/protocol/openid-connect/auth";
pub const BAKED_TOKEN: &str =
    "https://auth.eliteew.ru/realms/empery/protocol/openid-connect/token";
pub const BAKED_PAR: &str =
    "https://auth.eliteew.ru/realms/empery/protocol/openid-connect/ext/par/request";
pub const BAKED_FOFLINE_REDIRECT: &str = "https://app.eliteew.ru/fofline-oidc-bridge.html";
pub const CLIENT_ID: &str = "empery-fofline";
pub const AUTH_HOST: &str = "auth.eliteew.ru";
pub const TOKEN_PATH: &str = "/realms/empery/protocol/openid-connect/token";
pub const PAR_PATH: &str = "/realms/empery/protocol/openid-connect/ext/par/request";

const FORBIDDEN_QUERY: &[&str] = &[
    "eliteewLicenseId",
    "eliteewDeviceId",
    "eliteew_ticket",
    "ticket",
    "login_hint",
];

pub fn form_encode(pairs: &[(&str, &str)]) -> String {
    let mut out = String::new();
    for (index, (key, value)) in pairs.iter().enumerate() {
        if index > 0 {
            out.push('&');
        }
        out.push_str(&percent_encode(key));
        out.push('=');
        out.push_str(&percent_encode(value));
    }
    out
}

pub fn par_form(
    state: &str,
    nonce: &str,
    pkce_challenge: &str,
    dpop_jkt: &str,
    ticket: Option<&str>,
) -> Result<String, String> {
    for field in [state, nonce, pkce_challenge, dpop_jkt] {
        if field.is_empty() || field.len() > 256 || !b64url_chars(field) {
            return Err("oidc_field".into());
        }
    }
    // Field order matches living FoFline `native_oidc/par.rs`: known OIDC +
    // `dpop_jkt`, then `eliteew_ticket`, then `prompt=login`. Keycloak 26
    // additional-param copy is HashMap order, not form order.
    let mut pairs = vec![
        ("client_id", CLIENT_ID),
        ("redirect_uri", BAKED_FOFLINE_REDIRECT),
        ("response_type", "code"),
        ("scope", "openid"),
        ("state", state),
        ("nonce", nonce),
        ("code_challenge", pkce_challenge),
        ("code_challenge_method", "S256"),
        ("dpop_jkt", dpop_jkt),
    ];
    if let Some(ticket) = ticket.map(str::trim).filter(|item| !item.is_empty()) {
        if ticket.len() > 2048 || ticket.contains('\0') {
            return Err("oidc_ticket".into());
        }
        pairs.push(("eliteew_ticket", ticket));
    }
    pairs.push(("prompt", "login"));
    Ok(form_encode(&pairs))
}

pub fn refresh_form(refresh_token: &str) -> Result<String, String> {
    if !refresh_token_ok(refresh_token) {
        return Err("oidc_refresh_token".into());
    }
    Ok(form_encode(&[
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
        ("client_id", CLIENT_ID),
    ]))
}

pub fn refresh_token_ok(value: &str) -> bool {
    let trimmed = value.trim();
    (16..=8192).contains(&trimmed.len())
        && trimmed
            .chars()
            .all(|item| item.is_ascii_graphic() && item != '&' && item != '=')
}

pub fn build_par_authorization_url(request_uri: &str) -> Result<String, String> {
    let uri = request_uri.trim();
    if !request_uri_ok(uri) {
        return Err("oidc_request_uri".into());
    }
    let query = form_encode(&[("client_id", CLIENT_ID), ("request_uri", uri)]);
    let url = format!("{BAKED_AUTH}?{query}");
    if !authorization_url_allowed(&url) {
        return Err("oidc_auth_url".into());
    }
    Ok(url)
}

pub fn authorization_url_allowed(url: &str) -> bool {
    let trimmed = url.trim();
    if trimmed.len() > 2048 || !trimmed.starts_with(BAKED_AUTH) {
        return false;
    }
    let Some(query) = trimmed.strip_prefix(BAKED_AUTH).and_then(|rest| rest.strip_prefix('?'))
    else {
        return false;
    };
    if query.contains("://")
        || query.contains("..")
        || query.contains('\\')
        || query.contains('\0')
        || query.contains("code_challenge")
    {
        return false;
    }
    if FORBIDDEN_QUERY
        .iter()
        .any(|name| query.to_ascii_lowercase().contains(&format!("{}=", name.to_ascii_lowercase())))
    {
        return false;
    }
    query.contains("client_id=empery-fofline")
        && (query.contains("request_uri=urn%3Aietf%3Aparams%3Aoauth%3Arequest_uri%3A")
            || query.contains("request_uri=urn:ietf:params:oauth:request_uri:"))
}

pub fn request_uri_ok(uri: &str) -> bool {
    const PREFIX: &str = "urn:ietf:params:oauth:request_uri:";
    let Some(rest) = uri.strip_prefix(PREFIX) else {
        return false;
    };
    !rest.is_empty()
        && uri.len() <= 512
        && !uri.contains('?')
        && !uri.contains('#')
        && !uri.contains(' ')
        && rest
            .chars()
            .all(|item| item.is_ascii_alphanumeric() || matches!(item, '-' | '_' | ':' | '.'))
}

pub fn parse_callback_code(callback_url: &str, expected_state: &str) -> Result<String, String> {
    let raw = callback_url.trim();
    if raw.is_empty() || raw.len() > 4096 {
        return Err("callback_url".into());
    }
    let prefix = format!("{BAKED_FOFLINE_REDIRECT}?");
    let query = raw
        .strip_prefix(&prefix)
        .ok_or_else(|| "callback_redirect".to_string())?;
    let mut code = None;
    let mut state_ok = false;
    for pair in query.split('&') {
        let Some((key, value)) = pair.split_once('=') else {
            continue;
        };
        let key = percent_decode(key);
        let value = percent_decode(value);
        // Keycloak 26 codes are dotted (`uuid.tab.client`), not bare b64url.
        if key == "code" && auth_code_ok(&value) {
            code = Some(value.clone());
        }
        if key == "state" && value == expected_state {
            state_ok = true;
        }
        if key == "error" {
            return Err("callback_error".into());
        }
    }
    if !state_ok {
        return Err("callback_state".into());
    }
    code.ok_or_else(|| "callback_code".to_string())
}

pub fn percent_encode(input: &str) -> String {
    let mut out = String::new();
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let Ok(value) = u8::from_str_radix(
                std::str::from_utf8(&bytes[index + 1..index + 3]).unwrap_or(""),
                16,
            ) {
                out.push(value);
                index += 3;
                continue;
            }
        }
        out.push(bytes[index]);
        index += 1;
    }
    String::from_utf8(out).unwrap_or_default()
}

fn b64url_chars(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|item| item.is_ascii_alphanumeric() || item == '-' || item == '_')
}

fn auth_code_ok(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 512
        && value.chars().all(|item| {
            item.is_ascii_alphanumeric() || matches!(item, '-' | '_' | '.')
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn par_authorization_url_is_client_and_request_uri_only() {
        let url = build_par_authorization_url("urn:ietf:params:oauth:request_uri:abcDEFghij").unwrap();
        assert!(url.starts_with(BAKED_AUTH));
        assert!(url.contains("client_id=empery-fofline"));
        assert!(url.contains("request_uri="));
        assert!(!url.contains("code_challenge"));
        assert!(!url.to_ascii_lowercase().contains("ticket="));
        assert!(authorization_url_allowed(&url));
        assert!(!authorization_url_allowed("https://evil.example/auth"));
        assert!(!authorization_url_allowed(&format!(
            "{BAKED_AUTH}?client_id=empery-fofline&code_challenge=x&request_uri=urn:ietf:params:oauth:request_uri:abc"
        )));
        assert!(!request_uri_ok("urn:ietf:params:oauth:request_uri:abc?ticket=x"));
        assert_eq!(BAKED_PAR.ends_with(PAR_PATH), true);
        let form = par_form(
            "state01state01state01state01state0",
            "nonce01nonce01nonce01nonce01nonce0",
            "challenge01challenge01challenge01ch",
            "jkt01jkt01jkt01jkt01jkt01jkt01jkt0",
            Some("ticket-secret"),
        )
        .unwrap();
        assert!(form.contains("eliteew_ticket=ticket-secret"));
        let ticket_at = form.find("eliteew_ticket=").unwrap();
        let dpop_at = form.find("dpop_jkt=").unwrap();
        assert!(dpop_at < ticket_at);
        assert!(!authorization_url_allowed(&format!(
            "{BAKED_AUTH}?client_id=empery-fofline&eliteew_ticket=x&request_uri=urn:ietf:params:oauth:request_uri:abc"
        )));
    }

    #[test]
    fn callback_requires_exact_bridge_and_state() {
        let state = "state01state01state01state01state0";
        let url = format!("{BAKED_FOFLINE_REDIRECT}?code=abcDEFghij0123456789-_&state={state}");
        assert_eq!(parse_callback_code(&url, state).unwrap(), "abcDEFghij0123456789-_");
        assert_eq!(
            parse_callback_code(
                "https://app.eliteew.ru/?code=abcDEFghij0123456789-_&state=state01state01state01state01state0",
                state
            )
            .unwrap_err(),
            "callback_redirect"
        );
        assert_eq!(parse_callback_code(&url, "other").unwrap_err(), "callback_state");
        let kc_state = "state01state01state01state01state0";
        let kc_code = "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee.TabSessionStateValue012345.client-uuid-0000-0000-000000000000";
        let kc = format!(
            "{BAKED_FOFLINE_REDIRECT}?state={kc_state}&session_state=TabSessionStateValue012345&iss=https%3A%2F%2Fauth.eliteew.ru%2Frealms%2Fempery&code={kc_code}"
        );
        assert_eq!(parse_callback_code(&kc, kc_state).unwrap(), kc_code);
        assert!(auth_code_ok(kc_code));
        assert!(!auth_code_ok("has space"));
    }

    #[test]
    fn refresh_form_is_grant_and_client_only() {
        let refresh = "refresh-token-value-0123456789";
        let form = refresh_form(refresh).unwrap();
        assert!(form.contains("grant_type=refresh_token"));
        assert!(form.contains("client_id=empery-fofline"));
        assert!(form.contains("refresh_token=refresh-token-value-0123456789"));
        assert!(!form.contains("authorization_code"));
        assert!(!form.contains("code_verifier"));
        assert!(refresh_form("short").is_err());
        assert!(refresh_form("has space in token-0123456789").is_err());
    }
}
