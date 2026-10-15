//! Prep-bay durable peer list (not metal nft).

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::FwBackend;

#[derive(Debug, Serialize, Deserialize, Default)]
struct BlockedFile {
    peers: Vec<String>,
}

#[derive(Debug)]
pub struct StateFileBackend {
    path: PathBuf,
    file: BlockedFile,
}

impl StateFileBackend {
    pub fn open(state_dir: &Path) -> Result<Self, String> {
        fs::create_dir_all(state_dir).map_err(|e| format!("state_dir_create: {e}"))?;
        let path = state_dir.join("fw_blocked.json");
        let file = if path.exists() {
            let raw = fs::read_to_string(&path).map_err(|e| format!("read: {e}"))?;
            serde_json::from_str(&raw).map_err(|e| format!("parse: {e}"))?
        } else {
            BlockedFile::default()
        };
        Ok(Self { path, file })
    }

    fn save(&self) -> Result<(), String> {
        let raw = serde_json::to_string_pretty(&self.file).map_err(|e| format!("serialize: {e}"))?;
        fs::write(&self.path, raw).map_err(|e| format!("write: {e}"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&self.path, fs::Permissions::from_mode(0o600));
        }
        Ok(())
    }
}

impl FwBackend for StateFileBackend {
    fn block(&mut self, peer_ip: &str) -> Result<(), String> {
        if !self.file.peers.iter().any(|p| p == peer_ip) {
            self.file.peers.push(peer_ip.to_string());
            self.save()?;
        }
        Ok(())
    }

    fn unblock(&mut self, peer_ip: &str) -> Result<bool, String> {
        let before = self.file.peers.len();
        self.file.peers.retain(|p| p != peer_ip);
        let removed = self.file.peers.len() != before;
        if removed {
            self.save()?;
        }
        Ok(removed)
    }

    fn list(&self) -> Result<Vec<String>, String> {
        Ok(self.file.peers.clone())
    }
}
