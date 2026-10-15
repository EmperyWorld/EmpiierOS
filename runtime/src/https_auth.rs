//! HTTPS POST to the FoFline IdP over the OS-plane portal VIP.
//!
//! Connects to `10.88.89.1:443` with SNI `auth.eliteew.ru`. Runtime itself
//! stays `IPAddressDeny=any`; the SPA helper process performs this POST.

use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::Duration;

use crate::oidc_dpop_iat;
use crate::oidc_urls::{AUTH_HOST, PAR_PATH, TOKEN_PATH};

const ISSUER_PATH: &str = "/realms/empery";

const EDGE_VIP: &str = "10.88.89.1";
const EDGE_PORT: u16 = 443;
const HEADER_MAX: usize = 64 * 1024;
const BODY_MAX: usize = 32 * 1024;
const CONNECT_SECS: u64 = 20;
const IO_SECS: u64 = 30;

pub fn post_token(form: &str, dpop: &str) -> Result<String, String> {
    post_form(TOKEN_PATH, form, dpop, &[200])
}

pub fn post_par(form: &str, dpop: &str) -> Result<String, String> {
    post_form(PAR_PATH, form, dpop, &[200, 201])
}

/// HEAD/GET baked issuer; prefer HTTP `Date` for DPoP `iat`.
pub fn issuer_http_date() -> Option<u64> {
    issuer_date_via("HEAD").or_else(|| issuer_date_via("GET"))
}

fn issuer_date_via(method: &str) -> Option<u64> {
    if method != "HEAD" && method != "GET" {
        return None;
    }
    let _ = rustls::crypto::ring::default_provider().install_default();
    let mut roots = RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let config = ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let server_name = ServerName::try_from(AUTH_HOST.to_string()).ok()?;
    let conn = ClientConnection::new(Arc::new(config), server_name).ok()?;
    let sock = TcpStream::connect_timeout(
        &format!("{EDGE_VIP}:{EDGE_PORT}").parse().ok()?,
        Duration::from_secs(CONNECT_SECS),
    )
    .ok()?;
    sock.set_read_timeout(Some(Duration::from_secs(IO_SECS))).ok()?;
    sock.set_write_timeout(Some(Duration::from_secs(IO_SECS))).ok()?;
    let mut tls = StreamOwned::new(conn, sock);
    let request = format!(
        "{method} {ISSUER_PATH} HTTP/1.1\r\nHost: {AUTH_HOST}\r\nUser-Agent: empiieros-oidc/1\r\nAccept: */*\r\nAccept-Encoding: identity\r\nConnection: close\r\n\r\n"
    );
    tls.write_all(request.as_bytes()).ok()?;
    tls.flush().ok()?;
    let (_status, headers, _leftover) = read_headers(&mut tls).ok()?;
    let raw = header_value(&headers, "date")?;
    oidc_dpop_iat::parse_http_date_to_unix(&raw)
}

fn header_value(headers: &str, name: &str) -> Option<String> {
    headers.lines().find_map(|line| {
        let (key, raw) = line.split_once(':')?;
        key.eq_ignore_ascii_case(name)
            .then(|| raw.trim().to_string())
            .filter(|item| !item.is_empty())
    })
}

fn post_form(path: &str, form: &str, dpop: &str, allowed: &[u16]) -> Result<String, String> {
    if path != TOKEN_PATH && path != PAR_PATH {
        return Err("oidc_path".into());
    }
    if form.is_empty() || form.len() > 8 * 1024 || form.contains('\0') {
        return Err("oidc_form".into());
    }
    if dpop.is_empty() || dpop.len() > 8 * 1024 || dpop.contains('\n') || dpop.contains('\r') {
        return Err("oidc_dpop".into());
    }
    let _ = rustls::crypto::ring::default_provider().install_default();
    let mut roots = RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let config = ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let server_name =
        ServerName::try_from(AUTH_HOST.to_string()).map_err(|_| "oidc_sni".to_string())?;
    let conn = ClientConnection::new(Arc::new(config), server_name)
        .map_err(|error| format!("oidc_tls:{error}"))?;
    let sock = TcpStream::connect_timeout(
        &format!("{EDGE_VIP}:{EDGE_PORT}")
            .parse()
            .map_err(|_| "oidc_vip_addr".to_string())?,
        Duration::from_secs(CONNECT_SECS),
    )
    .map_err(|error| format!("oidc_connect:{error}"))?;
    sock.set_read_timeout(Some(Duration::from_secs(IO_SECS)))
        .map_err(|error| format!("oidc_timeout:{error}"))?;
    sock.set_write_timeout(Some(Duration::from_secs(IO_SECS)))
        .map_err(|error| format!("oidc_timeout:{error}"))?;
    let mut tls = StreamOwned::new(conn, sock);
    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: {AUTH_HOST}\r\nUser-Agent: empiieros-oidc/1\r\nAccept: application/json\r\nAccept-Encoding: identity\r\nContent-Type: application/x-www-form-urlencoded\r\nDPoP: {dpop}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{form}",
        form.len()
    );
    tls.write_all(request.as_bytes())
        .map_err(|error| format!("oidc_http_write:{error}"))?;
    tls.flush()
        .map_err(|error| format!("oidc_http_write:{error}"))?;
    let (status, headers, leftover) = read_headers(&mut tls)?;
    let chunked = header_has(&headers, "transfer-encoding", "chunked");
    let raw = if chunked {
        read_chunked(&mut tls, leftover)?
    } else {
        read_body(&mut tls, leftover)?
    };
    let body = json_payload(&raw);
    if !allowed.contains(&status) {
        return Err(oidc_status_error(status, &body));
    }
    if body.is_empty() {
        return Err("oidc_http_empty".into());
    }
    Ok(body)
}

fn read_headers<R: Read>(tls: &mut R) -> Result<(u16, String, Vec<u8>), String> {
    let mut buf = Vec::new();
    let mut byte = [0u8; 1];
    while buf.len() < HEADER_MAX {
        let n = tls
            .read(&mut byte)
            .map_err(|error| format!("oidc_http_read:{error}"))?;
        if n == 0 {
            return Err("oidc_http_eof".into());
        }
        buf.push(byte[0]);
        if buf.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
    }
    let split = buf
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| "oidc_http_headers".to_string())?;
    let header_text = String::from_utf8_lossy(&buf[..split]).to_string();
    let status = header_text
        .lines()
        .next()
        .unwrap_or("")
        .split_whitespace()
        .nth(1)
        .and_then(|item| item.parse().ok())
        .ok_or_else(|| "oidc_http_status".to_string())?;
    Ok((status, header_text, buf[split + 4..].to_vec()))
}

fn read_body<R: Read>(tls: &mut R, leftover: Vec<u8>) -> Result<String, String> {
    let mut body = leftover;
    let mut buf = [0u8; 4096];
    loop {
        let n = match tls.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(error) => return Err(format!("oidc_http_read:{error}")),
        };
        if body.len() + n > BODY_MAX {
            return Err("oidc_http_too_large".into());
        }
        body.extend_from_slice(&buf[..n]);
    }
    String::from_utf8(body).map_err(|_| "oidc_http_utf8".to_string())
}

/// Allowlisted `error` token only — never `error_description` or tokens.
pub(crate) fn oidc_status_error(status: u16, body: &str) -> String {
    let payload = json_payload(body);
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&payload) {
        if let Some(error) = value.get("error").and_then(|item| item.as_str()) {
            let token = error.trim();
            if (1..=64).contains(&token.len())
                && token
                    .chars()
                    .all(|item| item.is_ascii_alphanumeric() || item == '_' || item == '-')
            {
                return format!("oidc_http_{status}:{token}");
            }
        }
    }
    format!("oidc_http_{status}")
}

fn json_payload(raw: &str) -> String {
    let start = raw.find('{');
    let end = raw.rfind('}');
    match (start, end) {
        (Some(start), Some(end)) if end >= start => raw[start..=end].to_string(),
        _ => raw.trim().to_string(),
    }
}

fn header_has(headers: &str, name: &str, value: &str) -> bool {
    headers.lines().any(|line| {
        let Some((key, raw)) = line.split_once(':') else {
            return false;
        };
        key.eq_ignore_ascii_case(name) && raw.to_ascii_lowercase().contains(value)
    })
}

fn read_chunked<R: Read>(tls: &mut R, leftover: Vec<u8>) -> Result<String, String> {
    let mut input = leftover;
    let mut body = Vec::new();
    loop {
        fill_until(tls, &mut input, b"\r\n")?;
        let line_end = input
            .windows(2)
            .position(|window| window == b"\r\n")
            .ok_or_else(|| "oidc_chunk".to_string())?;
        let size_line = std::str::from_utf8(&input[..line_end])
            .map_err(|_| "oidc_chunk".to_string())?
            .trim();
        let size = u64::from_str_radix(size_line.split(';').next().unwrap_or(""), 16)
            .map_err(|_| "oidc_chunk".to_string())?;
        input.drain(..line_end + 2);
        if size == 0 {
            break;
        }
        fill_at_least(tls, &mut input, size as usize + 2)?;
        let chunk: Vec<u8> = input.drain(..size as usize).collect();
        if input.len() < 2 || &input[..2] != b"\r\n" {
            return Err("oidc_chunk".into());
        }
        input.drain(..2);
        if body.len() + chunk.len() > BODY_MAX {
            return Err("oidc_http_too_large".into());
        }
        body.extend_from_slice(&chunk);
    }
    String::from_utf8(body).map_err(|_| "oidc_http_utf8".to_string())
}

fn fill_until<R: Read>(tls: &mut R, input: &mut Vec<u8>, needle: &[u8]) -> Result<(), String> {
    while !input.windows(needle.len()).any(|window| window == needle) {
        if input.len() > HEADER_MAX {
            return Err("oidc_chunk".into());
        }
        let mut buf = [0u8; 1024];
        let n = tls
            .read(&mut buf)
            .map_err(|error| format!("oidc_http_read:{error}"))?;
        if n == 0 {
            return Err("oidc_http_eof".into());
        }
        input.extend_from_slice(&buf[..n]);
    }
    Ok(())
}

fn fill_at_least<R: Read>(tls: &mut R, input: &mut Vec<u8>, need: usize) -> Result<(), String> {
    while input.len() < need {
        let mut buf = [0u8; 4096];
        let n = tls
            .read(&mut buf)
            .map_err(|error| format!("oidc_http_read:{error}"))?;
        if n == 0 {
            return Err("oidc_http_eof".into());
        }
        input.extend_from_slice(&buf[..n]);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::TOKEN_PATH;
    use crate::oidc_urls;

    #[test]
    fn token_path_is_baked() {
        assert_eq!(TOKEN_PATH, oidc_urls::TOKEN_PATH);
        assert!(oidc_urls::BAKED_TOKEN.ends_with(TOKEN_PATH));
        assert!(oidc_urls::BAKED_PAR.ends_with(oidc_urls::PAR_PATH));
        assert_eq!(super::ISSUER_PATH, "/realms/empery");
    }

    #[test]
    fn status_error_keeps_allowlisted_token_only() {
        assert_eq!(
            super::oidc_status_error(400, r#"{"error":"invalid_dpop_proof","error_description":"nope"}"#),
            "oidc_http_400:invalid_dpop_proof"
        );
        assert_eq!(super::oidc_status_error(400, "not-json"), "oidc_http_400");
        assert_eq!(
            super::oidc_status_error(400, r#"{"error":"bad error"}"#),
            "oidc_http_400"
        );
    }
}
