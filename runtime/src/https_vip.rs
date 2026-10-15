//! HTTPS GET to the FoFline edge VIP over the OS-plane portal.
//!
//! Connects to `10.88.89.1:443` with SNI `app.eliteew.ru`. Does not use
//! SOCKS, boringtun, or Runtime networking.

use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use std::fs::File;

pub const EDGE_HOST: &str = "app.eliteew.ru";
pub const EDGE_VIP: &str = "10.88.89.1";
pub const EDGE_PORT: u16 = 443;
pub const HEALTH_PATH: &str = "/api/health";
pub const HEALTH_MAX_BYTES: u64 = 8 * 1024;
pub const CHALLENGE_PATH: &str = "/api/v1/desktop-client/auth/challenge";
pub const TICKET_PATH: &str = "/api/v1/desktop-client/auth/ticket";
/// EmpiierOS capsule OTA VIP prefix (edge nginx strips to `/v1/...` on :29181).
pub const OTA_PREFIX: &str = "/empiieros-ota/";
pub const OTA_ATTEST_PATH: &str = "/empiieros-ota/v1/attest";
pub const OTA_CURRENT_PATH: &str = "/empiieros-ota/v1/current.json";
const JSON_MAX: usize = 32 * 1024;

fn path_allowed(path: &str) -> bool {
    if path.contains("..") || path.contains('\\') || path.contains('\0') {
        return false;
    }
    path == HEALTH_PATH
        || path.starts_with("/fofline-updates/")
        || path.starts_with(OTA_PREFIX)
}

fn post_path_allowed(path: &str) -> bool {
    if path.contains("..") || path.contains('\\') || path.contains('\0') {
        return false;
    }
    path == CHALLENGE_PATH || path == TICKET_PATH || path == OTA_ATTEST_PATH
}

const HEADER_MAX: usize = 64 * 1024;
const CONNECT_SECS: u64 = 20;
const IO_SECS: u64 = 60;

pub fn get_text(path: &str, max_bytes: u64) -> Result<String, String> {
    let dest = std::env::temp_dir().join(format!(
        "empiieros-spa-text-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&dest);
    let written = get_file(path, &dest, max_bytes)?;
    let raw = std::fs::read_to_string(&dest).map_err(|error| format!("spa_text_read:{error}"))?;
    let _ = std::fs::remove_file(&dest);
    if written == 0 || raw.is_empty() {
        return Err("spa_http_empty".into());
    }
    Ok(raw)
}

pub fn get_file(path: &str, dest: &Path, max_bytes: u64) -> Result<u64, String> {
    get_file_hashed(path, dest, max_bytes).map(|result| result.bytes)
}

pub struct Download {
    pub bytes: u64,
    pub sha256_hex: String,
}

pub fn get_file_hashed(path: &str, dest: &Path, max_bytes: u64) -> Result<Download, String> {
    if !path_allowed(path) {
        return Err("spa_path_forbidden".into());
    }
    let _ = rustls::crypto::ring::default_provider().install_default();
    let mut roots = RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let config = ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let server_name = ServerName::try_from(EDGE_HOST.to_string())
        .map_err(|_| "spa_sni".to_string())?;
    let conn = ClientConnection::new(Arc::new(config), server_name)
        .map_err(|error| format!("spa_tls:{error}"))?;
    let sock = TcpStream::connect_timeout(
        &format!("{EDGE_VIP}:{EDGE_PORT}")
            .parse()
            .map_err(|_| "spa_vip_addr".to_string())?,
        Duration::from_secs(CONNECT_SECS),
    )
    .map_err(|error| format!("spa_connect:{error}"))?;
    sock.set_read_timeout(Some(Duration::from_secs(IO_SECS)))
        .map_err(|error| format!("spa_timeout:{error}"))?;
    sock.set_write_timeout(Some(Duration::from_secs(IO_SECS)))
        .map_err(|error| format!("spa_timeout:{error}"))?;
    let mut tls = StreamOwned::new(conn, sock);
    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: {EDGE_HOST}\r\nUser-Agent: empiieros-spa-fetch/1\r\nAccept: */*\r\nConnection: close\r\n\r\n"
    );
    tls.write_all(request.as_bytes())
        .map_err(|error| format!("spa_http_write:{error}"))?;
    tls.flush()
        .map_err(|error| format!("spa_http_write:{error}"))?;
    let (status, headers, leftover) = read_headers(&mut tls)?;
    if status != 200 {
        return Err(format!("spa_http_{status}"));
    }
    let chunked = header_has(&headers, "transfer-encoding", "chunked");
    let content_length = header_u64(&headers, "content-length");
    if let Some(len) = content_length {
        if len > max_bytes {
            return Err("pack_too_large".into());
        }
    }
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|error| format!("spa_dest:{error}"))?;
    }
    let mut file = File::create(dest).map_err(|error| format!("spa_dest:{error}"))?;
    let mut hasher = Sha256::new();
    let written = if chunked {
        write_chunked(&mut tls, leftover, &mut file, &mut hasher, max_bytes)?
    } else {
        write_length(
            &mut tls,
            leftover,
            &mut file,
            &mut hasher,
            max_bytes,
            content_length,
        )?
    };
    Ok(Download {
        bytes: written,
        sha256_hex: hex::encode(hasher.finalize()),
    })
}

pub fn post_json(path: &str, body: &str) -> Result<(u16, String), String> {
    if !post_path_allowed(path) {
        return Err("spa_path_forbidden".into());
    }
    if body.is_empty() || body.len() > 8 * 1024 || body.contains('\0') {
        return Err("spa_json_body".into());
    }
    let _ = rustls::crypto::ring::default_provider().install_default();
    let mut roots = RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let config = ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let server_name =
        ServerName::try_from(EDGE_HOST.to_string()).map_err(|_| "spa_sni".to_string())?;
    let conn = ClientConnection::new(Arc::new(config), server_name)
        .map_err(|error| format!("spa_tls:{error}"))?;
    let sock = TcpStream::connect_timeout(
        &format!("{EDGE_VIP}:{EDGE_PORT}")
            .parse()
            .map_err(|_| "spa_vip_addr".to_string())?,
        Duration::from_secs(CONNECT_SECS),
    )
    .map_err(|error| format!("spa_connect:{error}"))?;
    sock.set_read_timeout(Some(Duration::from_secs(IO_SECS)))
        .map_err(|error| format!("spa_timeout:{error}"))?;
    sock.set_write_timeout(Some(Duration::from_secs(IO_SECS)))
        .map_err(|error| format!("spa_timeout:{error}"))?;
    let mut tls = StreamOwned::new(conn, sock);
    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: {EDGE_HOST}\r\nUser-Agent: empiieros-spa-fetch/1\r\nAccept: application/json\r\nAccept-Encoding: identity\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    tls.write_all(request.as_bytes())
        .map_err(|error| format!("spa_http_write:{error}"))?;
    tls.flush()
        .map_err(|error| format!("spa_http_write:{error}"))?;
    let (status, _headers, leftover) = read_headers(&mut tls)?;
    let mut raw = leftover;
    let mut buf = [0u8; 4096];
    loop {
        let n = match tls.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(error) => return Err(format!("spa_http_read:{error}")),
        };
        if raw.len() + n > JSON_MAX {
            return Err("spa_http_too_large".into());
        }
        raw.extend_from_slice(&buf[..n]);
    }
    String::from_utf8(raw)
        .map(|text| (status, json_payload(&text)))
        .map_err(|_| "spa_http_utf8".to_string())
}

fn json_payload(raw: &str) -> String {
    let start = raw.find('{');
    let end = raw.rfind('}');
    match (start, end) {
        (Some(start), Some(end)) if end >= start => raw[start..=end].to_string(),
        _ => raw.trim().to_string(),
    }
}

fn read_headers<R: Read>(tls: &mut R) -> Result<(u16, String, Vec<u8>), String> {
    let mut buf = Vec::new();
    let mut byte = [0u8; 1];
    while buf.len() < HEADER_MAX {
        let n = tls.read(&mut byte).map_err(|error| format!("spa_http_read:{error}"))?;
        if n == 0 {
            return Err("spa_http_eof".into());
        }
        buf.push(byte[0]);
        if buf.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
    }
    let split = buf
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| "spa_http_headers".to_string())?;
    let header_bytes = &buf[..split];
    let leftover = buf[split + 4..].to_vec();
    let header_text = String::from_utf8_lossy(header_bytes).to_string();
    let status_line = header_text.lines().next().unwrap_or("");
    let status = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|item| item.parse().ok())
        .ok_or_else(|| "spa_http_status".to_string())?;
    Ok((status, header_text, leftover))
}

fn header_has(headers: &str, name: &str, value: &str) -> bool {
    headers.lines().any(|line| {
        let Some((key, raw)) = line.split_once(':') else {
            return false;
        };
        key.eq_ignore_ascii_case(name) && raw.to_ascii_lowercase().contains(value)
    })
}

fn header_u64(headers: &str, name: &str) -> Option<u64> {
    headers.lines().find_map(|line| {
        let (key, raw) = line.split_once(':')?;
        if key.eq_ignore_ascii_case(name) {
            raw.trim().parse().ok()
        } else {
            None
        }
    })
}

fn write_length<R: Read>(
    tls: &mut R,
    leftover: Vec<u8>,
    file: &mut File,
    hasher: &mut Sha256,
    max_bytes: u64,
    content_length: Option<u64>,
) -> Result<u64, String> {
    let mut written = 0u64;
    if !leftover.is_empty() {
        append(file, hasher, &leftover, &mut written, max_bytes)?;
    }
    let mut buf = [0u8; 32 * 1024];
    loop {
        if let Some(len) = content_length {
            if written >= len {
                break;
            }
        }
        let n = match tls.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(error) => return Err(format!("spa_http_read:{error}")),
        };
        append(file, hasher, &buf[..n], &mut written, max_bytes)?;
        if let Some(len) = content_length {
            if written > len {
                return Err("spa_http_length".into());
            }
        }
    }
    if let Some(len) = content_length {
        if written != len {
            return Err("spa_http_short".into());
        }
    }
    Ok(written)
}

fn write_chunked<R: Read>(
    tls: &mut R,
    leftover: Vec<u8>,
    file: &mut File,
    hasher: &mut Sha256,
    max_bytes: u64,
) -> Result<u64, String> {
    let mut input = leftover;
    let mut written = 0u64;
    loop {
        fill_until(tls, &mut input, b"\r\n")?;
        let line_end = input
            .windows(2)
            .position(|window| window == b"\r\n")
            .ok_or_else(|| "spa_chunk".to_string())?;
        let size_line = std::str::from_utf8(&input[..line_end])
            .map_err(|_| "spa_chunk".to_string())?
            .trim();
        let size = u64::from_str_radix(size_line.split(';').next().unwrap_or(""), 16)
            .map_err(|_| "spa_chunk".to_string())?;
        input.drain(..line_end + 2);
        if size == 0 {
            break;
        }
        fill_at_least(tls, &mut input, size as usize + 2)?;
        let chunk: Vec<u8> = input.drain(..size as usize).collect();
        if input.len() < 2 || &input[..2] != b"\r\n" {
            return Err("spa_chunk".into());
        }
        input.drain(..2);
        append(file, hasher, &chunk, &mut written, max_bytes)?;
    }
    Ok(written)
}

fn fill_until<R: Read>(tls: &mut R, input: &mut Vec<u8>, needle: &[u8]) -> Result<(), String> {
    while !input.windows(needle.len()).any(|window| window == needle) {
        if input.len() > HEADER_MAX {
            return Err("spa_chunk".into());
        }
        let mut buf = [0u8; 1024];
        let n = tls.read(&mut buf).map_err(|error| format!("spa_http_read:{error}"))?;
        if n == 0 {
            return Err("spa_http_eof".into());
        }
        input.extend_from_slice(&buf[..n]);
    }
    Ok(())
}

fn fill_at_least<R: Read>(tls: &mut R, input: &mut Vec<u8>, need: usize) -> Result<(), String> {
    while input.len() < need {
        let mut buf = [0u8; 32 * 1024];
        let n = tls.read(&mut buf).map_err(|error| format!("spa_http_read:{error}"))?;
        if n == 0 {
            return Err("spa_http_eof".into());
        }
        input.extend_from_slice(&buf[..n]);
    }
    Ok(())
}

fn append(
    file: &mut File,
    hasher: &mut Sha256,
    chunk: &[u8],
    written: &mut u64,
    max_bytes: u64,
) -> Result<(), String> {
    let next = written.saturating_add(chunk.len() as u64);
    if next > max_bytes {
        return Err("pack_too_large".into());
    }
    file.write_all(chunk)
        .map_err(|error| format!("spa_dest:{error}"))?;
    hasher.update(chunk);
    *written = next;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        path_allowed, post_path_allowed, CHALLENGE_PATH, OTA_ATTEST_PATH, OTA_CURRENT_PATH,
        OTA_PREFIX, TICKET_PATH,
    };

    #[test]
    fn path_allowlist_is_exact() {
        assert!(path_allowed("/api/health"));
        assert!(path_allowed("/fofline-updates/spa-latest.json"));
        assert!(path_allowed(OTA_CURRENT_PATH));
        assert!(path_allowed(&format!(
            "{OTA_PREFIX}v1/blobs/{}",
            "ab".repeat(32)
        )));
        assert!(!path_allowed("/api/v1/anything"));
        assert!(!path_allowed("/fofline-updates/../etc/passwd"));
        assert!(!path_allowed("/api/health/../secret"));
        assert!(!path_allowed("/empiieros-ota/../etc/passwd"));
        assert!(!path_allowed("/empiieros-ota/v1/blobs/../secret"));
        assert!(post_path_allowed(CHALLENGE_PATH));
        assert!(post_path_allowed(TICKET_PATH));
        assert!(post_path_allowed(OTA_ATTEST_PATH));
        assert!(!post_path_allowed("/api/v1/desktop-client/auth/ticket/../secret"));
        assert!(!post_path_allowed("/empiieros-ota/v1/attest/../secret"));
        assert!(!post_path_allowed("/empiieros-ota/v1/blobs/x"));
    }
}
