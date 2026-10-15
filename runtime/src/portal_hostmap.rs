//! OS-plane hostname map for the FoFline portal VIP.
//!
//! `app.eliteew.ru` / `auth.eliteew.ru` / `meet.eliteew.ru` → `10.88.89.1`.
//! Not SOCKS. Not Tauri.
//! Runtime stays `IPAddressDeny=any`. The portal helper writes `/etc/hosts`.

use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::Path;

use crate::state_store::atomic_write_file;
use std::fs::OpenOptions;

pub const HOSTS_PATH: &str = "/etc/hosts";
pub const BEGIN: &str = "# BEGIN empiieros-portal-hostmap";
pub const END: &str = "# END empiieros-portal-hostmap";
pub const VIP: &str = "10.88.89.1";
pub const LINE: &str = "10.88.89.1 app.eliteew.ru auth.eliteew.ru meet.eliteew.ru";
const AIR_HOSTS_NAME: &str = "hosts-air";
const REQUEST_DIR: &str = "/run/empiieros";

pub fn apply_default() -> Result<Value, String> {
    apply(Path::new(HOSTS_PATH))?;
    let _ = write_air_hosts();
    Ok(json!({
        "portal_hostmap": "applied",
        "vip": VIP,
        "names": ["app.eliteew.ru", "auth.eliteew.ru", "meet.eliteew.ru"],
        "detail": "portal host-map writes /etc/hosts; public TCP 443 stays denied"
    }))
}

pub fn hostmap_block(state_root: &Path) -> Option<&'static str> {
    crate::launcher_pin::access_block(state_root).or_else(|| {
        if crate::launcher_inner::portal_iface_up() {
            None
        } else {
            Some("portal_iface_down")
        }
    })
}

pub fn apply_for_seat(state_root: &Path) -> Result<Value, String> {
    if let Some(code) = hostmap_block(state_root) {
        return Err(code.into());
    }
    apply_default()
}

pub fn apply(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| format!("hostmap_stat:{error}"))?;
    if metadata.file_type().is_symlink() {
        return Err("hostmap_symlink".into());
    }
    if !metadata.is_file() {
        return Err("hostmap_not_file".into());
    }
    if unsafe { libc::geteuid() } == 0 && metadata.uid() != 0 {
        return Err("hostmap_owner".into());
    }
    let existing = fs::read_to_string(path).map_err(|error| format!("hostmap_read:{error}"))?;
    if existing.contains('\0') {
        return Err("hostmap_binary".into());
    }
    let body = merge_hosts(&existing);
    write_in_place(path, body.as_bytes(), metadata.mode() & 0o777)?;
    if !is_applied(path) {
        return Err("hostmap_verify".into());
    }
    Ok(())
}

pub fn observe(path: &Path) -> &'static str {
    if is_applied(path) {
        "applied"
    } else {
        "missing"
    }
}

pub fn observe_default() -> &'static str {
    observe(Path::new(HOSTS_PATH))
}

fn is_applied(path: &Path) -> bool {
    let Ok(raw) = fs::read_to_string(path) else {
        return false;
    };
    let Some(block) = extract_block(&raw) else {
        return false;
    };
    block.iter().any(|line| line == LINE)
        && !block.iter().any(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty()
                && !trimmed.starts_with('#')
                && trimmed != LINE
        })
}

fn merge_hosts(existing: &str) -> String {
    let without = strip_block(existing);
    let mut out = String::from(BEGIN);
    out.push('\n');
    out.push_str(LINE);
    out.push('\n');
    out.push_str(END);
    out.push('\n');
    if !without.is_empty() {
        out.push('\n');
        out.push_str(&without);
        if !without.ends_with('\n') {
            out.push('\n');
        }
    }
    out
}

fn strip_block(existing: &str) -> String {
    let mut out = String::new();
    let mut skipping = false;
    for line in existing.lines() {
        if line.trim() == BEGIN {
            skipping = true;
            continue;
        }
        if skipping {
            if line.trim() == END {
                skipping = false;
            }
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

fn extract_block(existing: &str) -> Option<Vec<String>> {
    let mut lines = Vec::new();
    let mut inside = false;
    for line in existing.lines() {
        let trimmed = line.trim();
        if trimmed == BEGIN {
            inside = true;
            continue;
        }
        if trimmed == END {
            return if inside { Some(lines) } else { None };
        }
        if inside {
            lines.push(trimmed.to_string());
        }
    }
    None
}

fn write_in_place(path: &Path, body: &[u8], mode: u32) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|error| format!("hostmap_write:{error}"))?;
    file.write_all(body)
        .map_err(|error| format!("hostmap_write:{error}"))?;
    file.sync_all()
        .map_err(|error| format!("hostmap_write:{error}"))?;
    let mut permissions = fs::metadata(path)
        .map_err(|error| format!("hostmap_write:{error}"))?
        .permissions();
    permissions.set_mode(if mode & 0o002 != 0 { 0o644 } else { mode });
    if permissions.mode() & 0o777 == 0 {
        permissions.set_mode(0o644);
    }
    fs::set_permissions(path, permissions).map_err(|error| format!("hostmap_write:{error}"))?;
    Ok(())
}

fn write_air_hosts() -> Result<(), String> {
    let dir = Path::new(REQUEST_DIR);
    if !dir.is_dir() {
        return Ok(());
    }
    let body = "127.0.0.1 localhost\n::1 localhost\n";
    atomic_write_file(dir, AIR_HOSTS_NAME, body.as_bytes(), 0o644)
        .map_err(|error| format!("hostmap_air:{error}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_hosts() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("empiieros-hosts-{nanos}"));
        fs::write(&path, "127.0.0.1 localhost\n93.77.165.185 app.eliteew.ru\n").unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o644);
        fs::set_permissions(&path, permissions).unwrap();
        path
    }

    #[test]
    fn apply_puts_vip_first_and_is_idempotent() {
        let path = temp_hosts();
        apply(&path).unwrap();
        apply(&path).unwrap();
        let raw = fs::read_to_string(&path).unwrap();
        assert!(raw.starts_with(BEGIN));
        assert!(raw.contains(LINE));
        assert_eq!(raw.matches(BEGIN).count(), 1);
        assert!(raw.contains("127.0.0.1 localhost"));
        assert_eq!(observe(&path), "applied");
        fs::remove_file(path).ok();
    }

    #[test]
    fn apply_rejects_symlink() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dest = std::env::temp_dir().join(format!("empiieros-hosts-dest-{nanos}"));
        let link = std::env::temp_dir().join(format!("empiieros-hosts-link-{nanos}"));
        fs::write(&dest, "127.0.0.1 localhost\n").unwrap();
        let _ = fs::remove_file(&link);
        std::os::unix::fs::symlink(&dest, &link).unwrap();
        assert_eq!(apply(&link).unwrap_err(), "hostmap_symlink");
        fs::remove_file(link).ok();
        fs::remove_file(dest).ok();
    }

    #[test]
    fn apply_for_seat_requires_unlock() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("empiieros-hostmap-{nanos}"));
        fs::create_dir_all(&root).unwrap();
        assert_eq!(apply_for_seat(&root).unwrap_err(), "pin_not_issued");
        crate::launcher_pin::plant_unlocked(&root);
        assert_eq!(apply_for_seat(&root).unwrap_err(), "portal_iface_down");
        fs::remove_dir_all(root).ok();
    }
}
