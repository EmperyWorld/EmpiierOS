//! Sealed IOC feed — body never exported on Shell-facing ops.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

use crate::error::{ShieldError, ShieldResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IocFeed {
    pub version: String,
    pub ips: Vec<String>,
    pub hashes: Vec<String>,
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeedStatus {
    pub present: bool,
    pub version: Option<String>,
    pub digest: Option<String>,
    pub ip_count: usize,
    pub hash_count: usize,
    pub detail: String,
}

impl FeedStatus {
    #[must_use]
    pub fn absent() -> Self {
        Self {
            present: false,
            version: None,
            digest: None,
            ip_count: 0,
            hash_count: 0,
            detail: "no sealed IOC feed installed".into(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct FeedFile {
    version: String,
    #[serde(default)]
    ips: Vec<String>,
    #[serde(default)]
    hashes: Vec<String>,
}

impl IocFeed {
    #[must_use]
    pub fn from_parts(version: String, ips: Vec<String>, hashes: Vec<String>) -> Self {
        let digest = digest_for(&version, &ips, &hashes);
        Self {
            version,
            ips,
            hashes,
            digest,
        }
    }

    pub fn load_file(path: &Path) -> ShieldResult<Self> {
        let raw = fs::read_to_string(path).map_err(|source| ShieldError::Io {
            context: "feed_read",
            source,
        })?;
        let parsed: FeedFile = serde_json::from_str(&raw).map_err(|e| ShieldError::Parse {
            context: "feed_parse",
            detail: e.to_string(),
        })?;
        if parsed.version.is_empty() || parsed.version.len() > 64 {
            return Err(ShieldError::Parse {
                context: "feed_parse",
                detail: "version must be 1..=64 chars".into(),
            });
        }
        for ip in &parsed.ips {
            if ip.parse::<std::net::IpAddr>().is_err() {
                return Err(ShieldError::Parse {
                    context: "feed_parse",
                    detail: format!("invalid IP in feed: {ip}"),
                });
            }
        }
        for hash in &parsed.hashes {
            if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(ShieldError::Parse {
                    context: "feed_parse",
                    detail: "hash must be 64 hex chars (sha256)".into(),
                });
            }
        }
        Ok(Self::from_parts(parsed.version, parsed.ips, parsed.hashes))
    }

    #[must_use]
    pub fn status(&self) -> FeedStatus {
        FeedStatus {
            present: true,
            version: Some(self.version.clone()),
            digest: Some(self.digest.clone()),
            ip_count: self.ips.len(),
            hash_count: self.hashes.len(),
            detail: "sealed feed present; body not exported to Shell".into(),
        }
    }
}

fn digest_for(version: &str, ips: &[String], hashes: &[String]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(version.as_bytes());
    hasher.update(b"|");
    for ip in ips {
        hasher.update(ip.as_bytes());
        hasher.update(b",");
    }
    hasher.update(b"|");
    for h in hashes {
        hasher.update(h.as_bytes());
        hasher.update(b",");
    }
    hex_encode(&hasher.finalize())
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0xf) as usize] as char);
    }
    out
}
