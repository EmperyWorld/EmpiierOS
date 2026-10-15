//! Portal WireGuard conf validate (harvested from living `conf_validate`).
//!
//! OS-plane helper starts the iface. Runtime does not start WG and does not
//! punch `IPAddressDeny`. Full-tunnel and hook keys fail closed.

use std::collections::BTreeMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use zeroize::Zeroize;

pub const EDGE_IP: &str = "10.88.89.1";
pub const REQUIRED_ALLOWED_IPS: &str = "10.88.89.1/32";
pub const REQUIRED_MTU: u16 = 1380;
pub const REQUIRED_PORT: u16 = 51823;

const INTERFACE_KEYS: &[&str] = &["PrivateKey", "Address", "DNS", "MTU"];
const PEER_KEYS: &[&str] = &["PublicKey", "Endpoint", "AllowedIPs", "PersistentKeepalive"];
const FORBIDDEN_KEYS: &[&str] = &[
    "PreUp", "PostUp", "PreDown", "PostDown", "Table", "SaveConfig", "Include",
    "ListenPort", "FwMark",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfError {
    pub code: &'static str,
}

pub struct PortalConf {
    pub private_key: String,
    pub address: String,
    #[allow(dead_code)]
    pub dns: String,
    pub mtu: u16,
    pub peer_public_key: String,
    pub endpoint: String,
    pub allowed_ips: String,
    pub keepalive: Option<u16>,
}

impl Drop for PortalConf {
    fn drop(&mut self) {
        self.private_key.zeroize();
    }
}

pub fn validate_portal_conf(conf: &str) -> Result<(), ConfError> {
    parse_portal_conf(conf).map(|_| ())
}

pub fn parse_portal_conf(conf: &str) -> Result<PortalConf, ConfError> {
    let sections = parse_sections(conf)?;
    if sections.len() != 2 {
        return Err(ConfError { code: "section_count" });
    }
    let iface = sections.get("Interface").ok_or(ConfError { code: "missing_sections" })?;
    let peer = sections.get("Peer").ok_or(ConfError { code: "missing_sections" })?;
    require_only_keys(iface, INTERFACE_KEYS)?;
    require_only_keys(peer, PEER_KEYS)?;
    for key in ["PrivateKey", "Address", "DNS", "MTU"] {
        require_key(iface, key)?;
    }
    for key in ["PublicKey", "Endpoint", "AllowedIPs"] {
        require_key(peer, key)?;
    }
    require_wg_key(iface.get("PrivateKey").expect("checked"))?;
    require_wg_key(peer.get("PublicKey").expect("checked"))?;
    parse_address_slash32(iface.get("Address").expect("checked"))?;
    let dns = parse_ipv4(iface.get("DNS").expect("checked"))?;
    if dns.to_string() != EDGE_IP {
        return Err(ConfError { code: "dns" });
    }
    let mtu = parse_u16(iface.get("MTU").expect("checked"))?;
    if mtu != REQUIRED_MTU {
        return Err(ConfError { code: "mtu" });
    }
    let endpoint = parse_endpoint(peer.get("Endpoint").expect("checked"))?;
    if endpoint.port() != REQUIRED_PORT {
        return Err(ConfError { code: "endpoint_port" });
    }
    let allowed = peer.get("AllowedIPs").expect("checked");
    if allowed_ips_forbidden(allowed) {
        return Err(ConfError { code: "full_tunnel_forbidden" });
    }
    if !allowed_ips_exact_edge(allowed) {
        return Err(ConfError { code: "allowed_ips" });
    }
    let keepalive = match peer.get("PersistentKeepalive") {
        Some(raw) => Some(parse_u16(raw)?),
        None => None,
    };
    Ok(PortalConf {
        private_key: iface.get("PrivateKey").expect("checked").clone(),
        address: iface.get("Address").expect("checked").clone(),
        dns: dns.to_string(),
        mtu,
        peer_public_key: peer.get("PublicKey").expect("checked").clone(),
        endpoint: endpoint.to_string(),
        allowed_ips: REQUIRED_ALLOWED_IPS.to_string(),
        keepalive,
    })
}

fn parse_sections(conf: &str) -> Result<BTreeMap<&'static str, BTreeMap<String, String>>, ConfError> {
    let text = conf.trim();
    if text.is_empty() {
        return Err(ConfError { code: "empty_conf" });
    }
    let mut sections = BTreeMap::new();
    let mut current: Option<&'static str> = None;
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            let name = &line[1..line.len() - 1];
            reject_unsafe(name)?;
            let canon = match name.to_ascii_lowercase().as_str() {
                "interface" => "Interface",
                "peer" => "Peer",
                _ => return Err(ConfError { code: "unknown_section" }),
            };
            if sections.contains_key(canon) {
                return Err(ConfError { code: "duplicate_section" });
            }
            sections.insert(canon, BTreeMap::new());
            current = Some(canon);
            continue;
        }
        let section = current.ok_or(ConfError { code: "orphan_line" })?;
        let (k_raw, v_raw) = line.split_once('=').ok_or(ConfError { code: "malformed_line" })?;
        let key = k_raw.trim();
        let value = v_raw.trim();
        if key.is_empty() || value.is_empty() {
            return Err(ConfError { code: "empty_field" });
        }
        reject_unsafe(key)?;
        reject_unsafe_value(value)?;
        if FORBIDDEN_KEYS.iter().any(|item| item.eq_ignore_ascii_case(key)) {
            return Err(ConfError { code: "forbidden_key" });
        }
        let canon_key = canonicalize(section, key).ok_or(ConfError { code: "unknown_key" })?;
        let map = sections.get_mut(section).expect("section");
        if map.contains_key(canon_key) {
            return Err(ConfError { code: "duplicate_key" });
        }
        map.insert(canon_key.to_string(), value.to_string());
    }
    Ok(sections)
}

fn canonicalize(section: &str, key: &str) -> Option<&'static str> {
    let list = match section {
        "Interface" => INTERFACE_KEYS,
        "Peer" => PEER_KEYS,
        _ => return None,
    };
    list.iter().copied().find(|item| item.eq_ignore_ascii_case(key))
}

fn require_only_keys(map: &BTreeMap<String, String>, allowed: &[&str]) -> Result<(), ConfError> {
    for key in map.keys() {
        if !allowed.iter().any(|item| item.eq_ignore_ascii_case(key)) {
            return Err(ConfError { code: "unknown_key" });
        }
    }
    Ok(())
}

fn require_key(map: &BTreeMap<String, String>, key: &str) -> Result<(), ConfError> {
    if map.contains_key(key) {
        Ok(())
    } else {
        Err(ConfError { code: "missing_field" })
    }
}

fn reject_unsafe(token: &str) -> Result<(), ConfError> {
    if !token.is_ascii() || token.chars().any(char::is_whitespace) {
        return Err(ConfError { code: "unsafe_token" });
    }
    Ok(())
}

fn reject_unsafe_value(value: &str) -> Result<(), ConfError> {
    if !value.is_ascii() {
        return Err(ConfError { code: "non_ascii" });
    }
    const BAD: &[char] = &[';', '|', '&', '`', '$', '(', ')', '<', '>', '\n', '\r', '\0', '\\', '"', '\''];
    if value.chars().any(|c| BAD.contains(&c)) {
        return Err(ConfError { code: "shell_metachar" });
    }
    Ok(())
}

fn require_wg_key(raw: &str) -> Result<(), ConfError> {
    if decode_wg_key(raw).is_err() {
        return Err(ConfError { code: "keys" });
    }
    Ok(())
}

fn decode_wg_key(raw: &str) -> Result<[u8; 32], ()> {
    if raw.len() != 44 || !raw.ends_with('=') {
        return Err(());
    }
    let mut out = [0u8; 32];
    let mut acc = 0u32;
    let mut bits = 0u32;
    let mut written = 0usize;
    for (idx, ch) in raw.bytes().enumerate() {
        if idx == 43 {
            break;
        }
        let val = match ch {
            b'A'..=b'Z' => ch - b'A',
            b'a'..=b'z' => ch - b'a' + 26,
            b'0'..=b'9' => ch - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return Err(()),
        } as u32;
        acc = (acc << 6) | val;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            if written >= 32 {
                return Err(());
            }
            out[written] = (acc >> bits) as u8;
            written += 1;
        }
    }
    if written != 32 {
        return Err(());
    }
    Ok(out)
}

fn parse_address_slash32(raw: &str) -> Result<Ipv4Addr, ConfError> {
    let (host, prefix) = raw.split_once('/').ok_or(ConfError { code: "address" })?;
    if prefix != "32" {
        return Err(ConfError { code: "address" });
    }
    host.parse().map_err(|_| ConfError { code: "address" })
}

fn parse_ipv4(raw: &str) -> Result<Ipv4Addr, ConfError> {
    raw.parse().map_err(|_| ConfError { code: "parse" })
}

fn parse_u16(raw: &str) -> Result<u16, ConfError> {
    raw.parse().map_err(|_| ConfError { code: "parse" })
}

fn parse_endpoint(raw: &str) -> Result<SocketAddr, ConfError> {
    raw.parse::<SocketAddr>()
        .or_else(|_| {
            let (host, port) = raw.rsplit_once(':').ok_or(())?;
            Ok(SocketAddr::new(
                host.parse::<IpAddr>().map_err(|_| ())?,
                port.parse::<u16>().map_err(|_| ())?,
            ))
        })
        .map_err(|_: ()| ConfError { code: "endpoint" })
}

fn allowed_ips_forbidden(raw: &str) -> bool {
    raw.split(',').any(|part| {
        let item = part.trim().to_ascii_lowercase();
        item == "0.0.0.0/0" || item == "::/0"
    })
}

fn allowed_ips_exact_edge(raw: &str) -> bool {
    let parts: Vec<&str> = raw.split(',').map(str::trim).filter(|item| !item.is_empty()).collect();
    parts.len() == 1 && parts[0].eq_ignore_ascii_case(REQUIRED_ALLOWED_IPS)
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = "[Interface]\n\
PrivateKey = AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE=\n\
Address = 10.88.89.101/32\n\
DNS = 10.88.89.1\n\
MTU = 1380\n\
\n\
[Peer]\n\
PublicKey = AgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgI=\n\
Endpoint = 93.77.165.185:51823\n\
AllowedIPs = 10.88.89.1/32\n\
PersistentKeepalive = 25\n";

    #[test]
    fn accepts_portal_handoff() {
        validate_portal_conf(GOOD).expect("good");
    }

    #[test]
    fn rejects_full_tunnel() {
        let bad = GOOD.replace("10.88.89.1/32", "0.0.0.0/0");
        assert_eq!(validate_portal_conf(&bad).unwrap_err().code, "full_tunnel_forbidden");
    }

    #[test]
    fn rejects_preup_hook() {
        let bad = GOOD.replace("MTU = 1380", "MTU = 1380\nPreUp = /bin/true");
        assert_eq!(validate_portal_conf(&bad).unwrap_err().code, "forbidden_key");
    }

    #[test]
    fn rejects_stub_key() {
        assert_eq!(
            validate_portal_conf("[Interface]\nPrivateKey=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=\n")
                .unwrap_err()
                .code,
            "section_count"
        );
    }
}
