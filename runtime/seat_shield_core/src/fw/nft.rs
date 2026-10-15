//! Allowlisted `nft` set element ops — argv only, absolute nft binary.
//!
//! Never builds shell strings. Never accepts relative `nft` paths.
//! Table/set/family names are allowlisted identifiers only.
//!
//! Dual sets: IPv4 → `set_v4` (default `blocked4`), IPv6 → `set_v6` (default `blocked6`).

use std::net::IpAddr;
use std::path::PathBuf;
use std::time::Duration;

use serde_json::Value;

use crate::command_runner;

use super::FwBackend;

pub const DEFAULT_NFT_BIN: &str = "/usr/sbin/nft";
pub const DEFAULT_SET_V4: &str = "blocked4";
pub const DEFAULT_SET_V6: &str = "blocked6";
const NFT_TIMEOUT: Duration = Duration::from_secs(5);

const ALLOWED_FAMILIES: &[&str] = &["inet", "ip", "ip6", "bridge", "netdev"];

#[derive(Debug, Clone)]
pub struct NftConfig {
    pub nft_bin: PathBuf,
    pub family: String,
    pub table: String,
    /// IPv4 set name.
    pub set_v4: String,
    /// IPv6 set name.
    pub set_v6: String,
}

impl Default for NftConfig {
    fn default() -> Self {
        Self {
            nft_bin: PathBuf::from(DEFAULT_NFT_BIN),
            family: "inet".into(),
            table: "seat_shield".into(),
            set_v4: DEFAULT_SET_V4.into(),
            set_v6: DEFAULT_SET_V6.into(),
        }
    }
}

impl NftConfig {
    pub fn validate(&self) -> Result<(), String> {
        if !self.nft_bin.is_absolute() {
            return Err("--nft-bin must be absolute".into());
        }
        if self.nft_bin.to_string_lossy().contains('\0') {
            return Err("--nft-bin contains NUL".into());
        }
        validate_nft_name("--nft-family", &self.family)?;
        if !ALLOWED_FAMILIES.contains(&self.family.as_str()) {
            return Err(format!(
                "--nft-family must be one of: {}",
                ALLOWED_FAMILIES.join("|")
            ));
        }
        validate_nft_name("--nft-table", &self.table)?;
        validate_nft_name("--nft-set", &self.set_v4)?;
        validate_nft_name("--nft-set6", &self.set_v6)?;
        Ok(())
    }

    fn set_for_ip(&self, peer_ip: &str) -> Result<&str, String> {
        let ip: IpAddr = peer_ip
            .parse()
            .map_err(|_| format!("invalid IP: {peer_ip}"))?;
        Ok(match ip {
            IpAddr::V4(_) => self.set_v4.as_str(),
            IpAddr::V6(_) => self.set_v6.as_str(),
        })
    }
}

pub fn validate_nft_name(flag: &str, value: &str) -> Result<(), String> {
    if value.is_empty() || value.len() > 64 {
        return Err(format!("{flag} length invalid"));
    }
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return Err(format!("{flag} empty"));
    };
    if !first.is_ascii_alphabetic() {
        return Err(format!("{flag} must start with a letter"));
    }
    if !chars.all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(format!(
            "{flag} must be [A-Za-z][A-Za-z0-9_]* (got {value})"
        ));
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct NftBackend {
    config: NftConfig,
}

impl NftBackend {
    pub fn new(config: NftConfig) -> Result<Self, String> {
        config.validate()?;
        if !config.nft_bin.exists() {
            return Err(format!(
                "nft binary missing: {}",
                config.nft_bin.display()
            ));
        }
        Ok(Self { config })
    }

    fn bin_str(&self) -> Result<&str, String> {
        self.config
            .nft_bin
            .to_str()
            .ok_or_else(|| "nft-bin path is not valid UTF-8".into())
    }

    fn run(&self, args: &[&str]) -> Result<command_runner::CommandResult, String> {
        let bin = self.bin_str()?;
        command_runner::run(bin, args, NFT_TIMEOUT).map_err(|e| e.to_string())
    }

    fn list_one_set(&self, set: &str) -> Result<Vec<String>, String> {
        let family = self.config.family.as_str();
        let table = self.config.table.as_str();
        let json = self.run(&["-j", "list", "set", family, table, set])?;
        if json.success {
            return parse_nft_set_json(&json.stdout);
        }
        let combined = format!("{}{}", json.stderr, json.stdout);
        if !combined.contains("JSON support not compiled-in") {
            return Err(format!(
                "nft list set {set} failed exit={:?}: {}",
                json.exit_code,
                json.stderr.trim()
            ));
        }
        // Buildroot nft on this floor has no libjansson. Text list is the
        // same set; do not require a second nft package.
        let text = self.run(&["list", "set", family, table, set])?;
        if !text.success {
            return Err(format!(
                "nft text list set {set} failed exit={:?}: {}",
                text.exit_code,
                text.stderr.trim()
            ));
        }
        parse_nft_set_text(&text.stdout)
    }
}

impl FwBackend for NftBackend {
    fn block(&mut self, peer_ip: &str) -> Result<(), String> {
        let family = self.config.family.as_str();
        let table = self.config.table.as_str();
        let set = self.config.set_for_ip(peer_ip)?;
        let result = self.run(&[
            "add", "element", family, table, set, "{", peer_ip, "}",
        ])?;
        if result.success {
            return Ok(());
        }
        let combined = format!("{}{}", result.stderr, result.stdout).to_ascii_lowercase();
        if combined.contains("file exists") || combined.contains("already exists") {
            return Ok(());
        }
        Err(format!(
            "nft add element failed exit={:?}: {}",
            result.exit_code,
            result.stderr.trim()
        ))
    }

    fn unblock(&mut self, peer_ip: &str) -> Result<bool, String> {
        let family = self.config.family.as_str();
        let table = self.config.table.as_str();
        let set = self.config.set_for_ip(peer_ip)?;
        let result = self.run(&[
            "delete", "element", family, table, set, "{", peer_ip, "}",
        ])?;
        if result.success {
            return Ok(true);
        }
        let combined = format!("{}{}", result.stderr, result.stdout).to_ascii_lowercase();
        if combined.contains("could not find")
            || combined.contains("no such file")
            || combined.contains("does not exist")
            || combined.contains("not found")
        {
            return Ok(false);
        }
        Err(format!(
            "nft delete element failed exit={:?}: {}",
            result.exit_code,
            result.stderr.trim()
        ))
    }

    fn list(&self) -> Result<Vec<String>, String> {
        let mut peers = self.list_one_set(&self.config.set_v4)?;
        if self.config.set_v6 != self.config.set_v4 {
            peers.extend(self.list_one_set(&self.config.set_v6)?);
        }
        peers.sort();
        peers.dedup();
        Ok(peers)
    }
}

pub(crate) fn parse_nft_set_text(raw: &str) -> Result<Vec<String>, String> {
    let mut peers = Vec::new();
    let mut rest = raw;
    while let Some(idx) = rest.find("elements") {
        let after_kw = &rest[idx + "elements".len()..];
        let Some(brace_at) = after_kw.find('{') else {
            break;
        };
        let inner_start = &after_kw[brace_at + 1..];
        let Some(end) = inner_start.find('}') else {
            return Err("nft text list: unclosed elements block".into());
        };
        for token in inner_start[..end].split(|c: char| c == ',' || c.is_ascii_whitespace()) {
            let token = token.trim();
            if token.is_empty() {
                continue;
            }
            if token.parse::<IpAddr>().is_ok() {
                peers.push(token.to_string());
            }
        }
        rest = &inner_start[end + 1..];
    }
    peers.sort();
    peers.dedup();
    Ok(peers)
}

pub(crate) fn parse_nft_set_json(raw: &str) -> Result<Vec<String>, String> {
    let value: Value =
        serde_json::from_str(raw.trim()).map_err(|e| format!("nft json parse: {e}"))?;
    let mut peers = Vec::new();
    let Some(arr) = value.get("nftables").and_then(Value::as_array) else {
        return Err("nft json missing nftables array".into());
    };
    for item in arr {
        let Some(set) = item.get("set") else {
            continue;
        };
        collect_elems(set.get("elem"), &mut peers);
    }
    peers.sort();
    peers.dedup();
    Ok(peers)
}

fn collect_elems(elem: Option<&Value>, out: &mut Vec<String>) {
    let Some(elem) = elem else {
        return;
    };
    match elem {
        Value::Array(items) => {
            for item in items {
                collect_one_elem(item, out);
            }
        }
        other => collect_one_elem(other, out),
    }
}

fn collect_one_elem(item: &Value, out: &mut Vec<String>) {
    if let Some(s) = item.as_str() {
        out.push(s.to_string());
        return;
    }
    if let Some(obj) = item.as_object() {
        if let Some(inner) = obj.get("elem") {
            collect_one_elem(inner, out);
            return;
        }
        if let Some(val) = obj.get("val") {
            if let Some(s) = val.as_str() {
                out.push(s.to_string());
            } else if let Some(n) = val.as_u64() {
                let _ = n;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nft_json_elems() {
        let raw = r#"{"nftables":[{"metainfo":{"version":"1"}},{"set":{"family":"inet","name":"blocked4","elem":[{"elem":{"val":"198.51.100.1"}},{"elem":{"val":"198.51.100.2"}}]}}]}"#;
        let peers = parse_nft_set_json(raw).unwrap();
        assert_eq!(
            peers,
            vec!["198.51.100.1".to_string(), "198.51.100.2".to_string()]
        );
    }

    #[test]
    fn parses_nft_json_simple_string_elems() {
        let raw = r#"{"nftables":[{"set":{"family":"inet","name":"blocked4","elem":["198.51.100.20"]}}]}"#;
        let peers = parse_nft_set_json(raw).unwrap();
        assert_eq!(peers, vec!["198.51.100.20".to_string()]);
    }

    #[test]
    fn parses_nft_text_empty_set() {
        let raw = r#"table inet seat_shield {
	set blocked4 {
		type ipv4_addr
		flags interval
		comment "Seat Shield blocked IPv4 peers"
	}
}
"#;
        assert!(parse_nft_set_text(raw).unwrap().is_empty());
    }

    #[test]
    fn parses_nft_text_inline_and_multiline_elems() {
        let inline = r#"set blocked4 { type ipv4_addr; elements = { 198.51.100.1, 198.51.100.2 } }"#;
        assert_eq!(
            parse_nft_set_text(inline).unwrap(),
            vec!["198.51.100.1".to_string(), "198.51.100.2".to_string()]
        );
        let multi = r#"table inet seat_shield {
	set blocked4 {
		type ipv4_addr
		elements = {
			192.0.2.10,
			2001:db8::20
		}
	}
}
"#;
        assert_eq!(
            parse_nft_set_text(multi).unwrap(),
            vec!["192.0.2.10".to_string(), "2001:db8::20".to_string()]
        );
    }

    #[test]
    fn rejects_bad_names() {
        assert!(validate_nft_name("--nft-table", "seat;shield").is_err());
        assert!(validate_nft_name("--nft-table", "seat_shield").is_ok());
        assert!(std::path::Path::new(DEFAULT_NFT_BIN).is_absolute());
    }

    #[test]
    fn set_for_ip_picks_family() {
        let cfg = NftConfig::default();
        assert_eq!(cfg.set_for_ip("198.51.100.1").unwrap(), "blocked4");
        assert_eq!(cfg.set_for_ip("2001:db8::1").unwrap(), "blocked6");
    }

    #[test]
    fn garbage_nft_text_and_json_do_not_panic() {
        let long = "elements = { 198.51.100.1, ".repeat(64);
        let junk = [
            "",
            "{",
            "elements = {",
            "elements = { not-an-ip }",
            "elements = { 999.999.999.999 }",
            "{\"nftables\":",
            "{\"nftables\":[]}",
            "{\"nftables\":[{\"set\":{\"elem\":[1,2,3]}}]}",
            "\0\0\0",
            long.as_str(),
        ];
        for raw in junk {
            let _ = parse_nft_set_text(raw);
            let _ = parse_nft_set_json(raw);
        }
    }
}
