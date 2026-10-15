//! inode → (pid, comm) via `/proc/*/fd` (harvested from ewonika; MIT).

use std::collections::HashMap;
use std::fs;

pub type InodeMap = HashMap<u64, (u32, String)>;

pub fn map_socket_inodes() -> InodeMap {
    let mut map = InodeMap::new();
    let Ok(proc) = fs::read_dir("/proc") else {
        return map;
    };
    for entry in proc.flatten() {
        let name = entry.file_name();
        let Some(pid) = name.to_str().and_then(|s| s.parse::<u32>().ok()) else {
            continue;
        };
        let comm = fs::read_to_string(format!("/proc/{pid}/comm"))
            .unwrap_or_default()
            .trim()
            .to_string();
        let Ok(fds) = fs::read_dir(format!("/proc/{pid}/fd")) else {
            continue;
        };
        for fd in fds.flatten() {
            let Ok(meta) = fs::symlink_metadata(fd.path()) else {
                continue;
            };
            if !meta.file_type().is_symlink() {
                continue;
            }
            let Ok(target) = fs::read_link(fd.path()) else {
                continue;
            };
            let Some(target) = target.to_str() else {
                continue;
            };
            if let Some(rest) = target.strip_prefix("socket:[") {
                if let Some(num) = rest.strip_suffix(']') {
                    if let Ok(inode) = num.parse::<u64>() {
                        map.entry(inode).or_insert((pid, comm.clone()));
                    }
                }
            }
        }
    }
    map
}
