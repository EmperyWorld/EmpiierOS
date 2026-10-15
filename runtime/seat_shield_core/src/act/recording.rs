//! Recording actuator — persists blocked peers under seat_shield state.
//!
//! This is **not** nftables. It exists so act/confirm paths can be integration-
//! tested without shell crutches. EmpiierOS will replace this with a pinned
//! absolute-path helper (`Command::new("/usr/libexec/...")`).

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::act::FirewallActuator;
use crate::error::{ShieldError, ShieldResult};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct BlockedFile {
    peers: Vec<String>,
}

#[derive(Debug)]
pub struct RecordingActuator {
    path: PathBuf,
    peers: Vec<String>,
}

impl RecordingActuator {
    pub fn open(state_seat_shield_dir: impl AsRef<Path>) -> ShieldResult<Self> {
        let path = state_seat_shield_dir.as_ref().join("blocked.json");
        let peers = if path.exists() {
            let raw = fs::read_to_string(&path).map_err(|source| ShieldError::Io {
                context: "blocked_read",
                source,
            })?;
            let file: BlockedFile =
                serde_json::from_str(&raw).map_err(|e| ShieldError::Parse {
                    context: "blocked_parse",
                    detail: e.to_string(),
                })?;
            file.peers
        } else {
            Vec::new()
        };
        Ok(Self { path, peers })
    }

    fn persist(&self) -> ShieldResult<()> {
        let raw = serde_json::to_string_pretty(&BlockedFile {
            peers: self.peers.clone(),
        })
        .map_err(|e| ShieldError::Serialize(e.to_string()))?;
        fs::write(&self.path, raw).map_err(|source| ShieldError::Io {
            context: "blocked_write",
            source,
        })?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&self.path, fs::Permissions::from_mode(0o600));
        }
        Ok(())
    }
}

impl FirewallActuator for RecordingActuator {
    fn backend_name(&self) -> &'static str {
        "recording"
    }

    fn is_available(&self) -> bool {
        true
    }

    fn block_peer(&mut self, peer_ip: &str, _reason: &str) -> ShieldResult<()> {
        if !self.peers.iter().any(|p| p == peer_ip) {
            self.peers.push(peer_ip.to_string());
            self.persist()?;
        }
        Ok(())
    }

    fn unblock_peer(&mut self, peer_ip: &str) -> ShieldResult<bool> {
        let before = self.peers.len();
        self.peers.retain(|p| p != peer_ip);
        let removed = self.peers.len() != before;
        if removed {
            self.persist()?;
        }
        Ok(removed)
    }

    fn list_blocked(&self) -> ShieldResult<Vec<String>> {
        Ok(self.peers.clone())
    }
}
