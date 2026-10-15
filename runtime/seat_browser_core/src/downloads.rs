//! Seat Browser download ledger. Destination is always under
//! `{state}/seat_browser/downloads/`. Overlay writes bytes; Runtime is SoT.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const MAX_DOWNLOADS: usize = 16;
pub const MAX_CONCURRENT: usize = 1;
pub const MAX_DOWNLOAD_BYTES: u64 = 32 * 1024 * 1024;
const MAX_NAME_CHARS: usize = 180;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownloadRecord {
    pub id: u32,
    pub name: String,
    pub host: String,
    #[serde(default)]
    pub bytes: u64,
    pub phase: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownloadState {
    #[serde(default = "default_next_id")]
    pub next_id: u32,
    #[serde(default)]
    pub items: Vec<DownloadRecord>,
}

fn default_next_id() -> u32 {
    1
}

impl Default for DownloadState {
    fn default() -> Self {
        Self {
            next_id: 1,
            items: Vec::new(),
        }
    }
}

#[must_use]
pub fn sanitize_basename(suggested: &str) -> String {
    let raw = Path::new(suggested.trim())
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .trim();
    let cleaned: String = raw
        .chars()
        .map(|c| {
            if c.is_control() || "/\\:".contains(c) {
                '_'
            } else {
                c
            }
        })
        .collect();
    let cleaned = cleaned.trim_matches('.').trim();
    if cleaned.is_empty() || cleaned == ".." || cleaned == "." {
        return "download.bin".into();
    }
    if cleaned.chars().count() > MAX_NAME_CHARS {
        return cleaned.chars().take(MAX_NAME_CHARS).collect();
    }
    cleaned.to_string()
}

#[must_use]
pub fn name_from_url(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|parsed| {
            parsed
                .path_segments()
                .and_then(|mut segs| segs.next_back().map(str::to_string))
        })
        .filter(|name| !name.is_empty())
        .map(|name| sanitize_basename(&name))
        .unwrap_or_else(|| "download.bin".into())
}

#[must_use]
pub fn file_uri(path: &Path) -> String {
    format!("file://{}", path.to_string_lossy())
}

#[must_use]
pub fn dest_is_seat_download(uri: &str, name: &str) -> bool {
    let Some(path) = uri.strip_prefix("file://") else {
        return false;
    };
    if path.contains("..") || !path.contains("/seat_browser/downloads/") {
        return false;
    }
    Path::new(path).file_name().and_then(|s| s.to_str()) == Some(name)
}

pub fn unique_dest(dir: &Path, basename: &str) -> PathBuf {
    let candidate = dir.join(basename);
    if !candidate.exists() {
        return candidate;
    }
    let stem = Path::new(basename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("download");
    let ext = Path::new(basename)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    for i in 2..=99 {
        let next = if ext.is_empty() {
            dir.join(format!("{stem}-{i}"))
        } else {
            dir.join(format!("{stem}-{i}.{ext}"))
        };
        if !next.exists() {
            return next;
        }
    }
    dir.join(format!("{stem}-overflow.bin"))
}

impl DownloadState {
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.items.iter().filter(|item| item.phase == "started").count()
    }

    pub fn begin(&mut self, name: String, host: String) -> Result<DownloadRecord, &'static str> {
        if self.active_count() >= MAX_CONCURRENT {
            return Err("download_busy");
        }
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1).max(1);
        let record = DownloadRecord {
            id,
            name,
            host,
            bytes: 0,
            phase: "started".into(),
        };
        self.items.push(record.clone());
        if self.items.len() > MAX_DOWNLOADS {
            self.items.remove(0);
        }
        Ok(record)
    }

    pub fn finish(
        &mut self,
        id: u32,
        ok: bool,
        bytes: u64,
    ) -> Result<DownloadRecord, &'static str> {
        let item = self
            .items
            .iter_mut()
            .find(|item| item.id == id)
            .ok_or("unknown_download")?;
        item.bytes = bytes.min(MAX_DOWNLOAD_BYTES);
        item.phase = if !ok {
            "failed".into()
        } else if bytes > MAX_DOWNLOAD_BYTES {
            "denied".into()
        } else {
            "finished".into()
        };
        Ok(item.clone())
    }
}

#[must_use]
pub fn sanitize_downloads(mut state: DownloadState) -> DownloadState {
    if state.next_id == 0 {
        state.next_id = 1;
    }
    let max_id = state.items.iter().map(|item| item.id).max().unwrap_or(0);
    if state.next_id <= max_id {
        state.next_id = max_id.saturating_add(1).max(1);
    }
    if state.items.len() > MAX_DOWNLOADS {
        let drop = state.items.len() - MAX_DOWNLOADS;
        state.items.drain(0..drop);
    }
    for item in &mut state.items {
        item.name = sanitize_basename(&item.name);
        if item.phase != "started" && item.phase != "finished" && item.phase != "failed" {
            item.phase = "failed".into();
        }
        if item.host.is_empty() || item.host.len() > 253 {
            item.host = "unknown".into();
        }
    }
    state
}

pub fn ensure_download_dir(state_root: &Path) -> std::io::Result<PathBuf> {
    let dir = state_root.join("seat_browser/downloads");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_rejects_path_escape() {
        assert_eq!(sanitize_basename("../../etc/passwd"), "passwd");
        assert_eq!(sanitize_basename(""), "download.bin");
        assert_eq!(sanitize_basename("ok.pdf"), "ok.pdf");
    }

    #[test]
    fn dest_uri_must_stay_under_downloads() {
        assert!(dest_is_seat_download(
            "file:///var/lib/empiieros-runtime/seat_browser/downloads/ok.pdf",
            "ok.pdf"
        ));
        assert!(!dest_is_seat_download(
            "file:///etc/passwd",
            "passwd"
        ));
        assert!(!dest_is_seat_download(
            "file:///var/lib/empiieros-runtime/seat_browser/downloads/../policy.json",
            "policy.json"
        ));
    }
}
