//! Linux `/proc` TCP(6) established-connection observer.
//!
//! No shell. No `ss`. Parses `/proc/net/tcp{,6}` and maps socket inodes via
//! `/proc/<pid>/fd` → `socket:[inode]`, with bounded scans.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::error::{ShieldError, ShieldResult};
use crate::observe::{ConnSource, ObservedConn};
use crate::policy::SUSPICIOUS_EXEC_PREFIXES;

const TCP_ESTABLISHED: u8 = 0x01;
const MAX_PIDS_SCANNED: usize = 4096;
const MAX_FDS_PER_PID: usize = 512;
const MAX_CONNECTIONS: usize = 2048;
const MAX_HASHES_PER_COLLECT: usize = 32;
const MAX_EXE_HASH_BYTES: u64 = 32 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct ProcConnSource {
    proc_root: PathBuf,
}

impl Default for ProcConnSource {
    fn default() -> Self {
        Self {
            proc_root: PathBuf::from("/proc"),
        }
    }
}

impl ProcConnSource {
    #[must_use]
    pub fn new(proc_root: impl Into<PathBuf>) -> Self {
        Self {
            proc_root: proc_root.into(),
        }
    }

    /// Collect established connections and fill `exe_sha256` under the S4 budget.
    ///
    /// `hash_any_exe` is true only when a sealed feed has at least one hash.
    /// Suspicious prefixes are hashed even when the feed has no hashes.
    /// Oversized or unreadable files fail open to `None` (path is still reported).
    pub fn collect_with_hash(&self, hash_any_exe: bool) -> ShieldResult<Vec<ObservedConn>> {
        let mut out = self.collect_unhashed()?;
        fill_exe_hashes(&mut out, hash_any_exe);
        Ok(out)
    }

    fn collect_unhashed(&self) -> ShieldResult<Vec<ObservedConn>> {
        let mut table = Vec::new();
        table.extend(parse_proc_net_table(&self.proc_root.join("net/tcp"), false)?);
        table.extend(parse_proc_net_table(&self.proc_root.join("net/tcp6"), true)?);

        let inode_map = build_inode_process_map(&self.proc_root)?;
        let mut out = Vec::new();
        for row in table {
            if out.len() >= MAX_CONNECTIONS {
                break;
            }
            if row.state != TCP_ESTABLISHED {
                continue;
            }
            let (process_name, pid, exe_path) = inode_map
                .get(&row.inode)
                .cloned()
                .unwrap_or_else(|| ("unknown".into(), 0, None));
            out.push(ObservedConn {
                remote_ip: row.remote_ip,
                local_port: row.local_port,
                remote_port: row.remote_port,
                process_name,
                pid,
                exe_path,
                exe_sha256: None,
                rtt_ms: None,
            });
        }
        Ok(out)
    }
}

impl ConnSource for ProcConnSource {
    fn collect(&self) -> ShieldResult<Vec<ObservedConn>> {
        self.collect_with_hash(false)
    }

    fn backend_name(&self) -> &'static str {
        "proc_net_tcp"
    }
}

#[derive(Debug, Clone)]
struct TcpRow {
    local_port: u16,
    remote_ip: String,
    remote_port: u16,
    state: u8,
    inode: u64,
}

fn parse_proc_net_table(path: &Path, ipv6: bool) -> ShieldResult<Vec<TcpRow>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file = File::open(path).map_err(|source| ShieldError::Io {
        context: "open_proc_net",
        source,
    })?;
    let reader = BufReader::new(file);
    let mut rows = Vec::new();
    for (idx, line) in reader.lines().enumerate() {
        let line = line.map_err(|source| ShieldError::Io {
            context: "read_proc_net",
            source,
        })?;
        if idx == 0 {
            continue; // header
        }
        if let Some(row) = parse_proc_net_line(&line, ipv6) {
            rows.push(row);
        }
    }
    Ok(rows)
}

fn parse_proc_net_line(line: &str, ipv6: bool) -> Option<TcpRow> {
    let mut parts = line.split_whitespace();
    let _sl = parts.next()?;
    let local = parts.next()?;
    let remote = parts.next()?;
    let state_hex = parts.next()?;
    // Columns after st: tx_queue:rx_queue, tr:tm->when, retrnsmt, uid, timeout → then inode.
    for _ in 0..5 {
        parts.next()?;
    }
    let inode: u64 = parts.next()?.parse().ok()?;
    let state = u8::from_str_radix(state_hex, 16).ok()?;
    let (local_ip, local_port) = parse_addr(local, ipv6)?;
    let (remote_ip, remote_port) = parse_addr(remote, ipv6)?;
    let _ = local_ip; // local IP unused for seat probe logic today
    Some(TcpRow {
        local_port,
        remote_ip,
        remote_port,
        state,
        inode,
    })
}

/// H4: feed one `/proc/net/tcp{,6}` line. Does not expose `TcpRow`.
pub(crate) fn probe_proc_net_line(line: &str, ipv6: bool) {
    let _ = parse_proc_net_line(line, ipv6);
}

fn parse_addr(raw: &str, ipv6: bool) -> Option<(String, u16)> {
    let (ip_hex, port_hex) = raw.split_once(':')?;
    let port = u16::from_str_radix(port_hex, 16).ok()?;
    if ipv6 {
        let ip = parse_ipv6_hex(ip_hex)?;
        Some((ip.to_string(), port))
    } else {
        let ip = parse_ipv4_hex(ip_hex)?;
        Some((ip.to_string(), port))
    }
}

fn parse_ipv4_hex(hex: &str) -> Option<Ipv4Addr> {
    if hex.len() != 8 {
        return None;
    }
    let n = u32::from_str_radix(hex, 16).ok()?;
    // /proc/net/tcp stores little-endian host bytes on typical Linux.
    Some(Ipv4Addr::from(n.to_le_bytes()))
}

fn parse_ipv6_hex(hex: &str) -> Option<Ipv6Addr> {
    if hex.len() != 32 {
        return None;
    }
    let mut bytes = [0_u8; 16];
    // Four little-endian 32-bit words.
    for (word_idx, chunk) in hex.as_bytes().chunks(8).enumerate() {
        let s = std::str::from_utf8(chunk).ok()?;
        let word = u32::from_str_radix(s, 16).ok()?;
        let le = word.to_le_bytes();
        let base = word_idx * 4;
        bytes[base..base + 4].copy_from_slice(&le);
    }
    Some(Ipv6Addr::from(bytes))
}

type ProcInfo = (String, i32, Option<String>);

fn build_inode_process_map(proc_root: &Path) -> ShieldResult<HashMap<u64, ProcInfo>> {
    let mut map = HashMap::new();
    let entries = fs::read_dir(proc_root).map_err(|source| ShieldError::Io {
        context: "read_proc_root",
        source,
    })?;
    let mut pids_seen = 0_usize;
    for entry in entries {
        if pids_seen >= MAX_PIDS_SCANNED {
            break;
        }
        let entry = entry.map_err(|source| ShieldError::Io {
            context: "read_proc_entry",
            source,
        })?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let pid: i32 = match name.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };
        pids_seen += 1;
        let fd_dir = entry.path().join("fd");
        let Ok(fds) = fs::read_dir(&fd_dir) else {
            continue;
        };
        for (fd_count, fd) in fds.enumerate() {
            if fd_count >= MAX_FDS_PER_PID {
                break;
            }
            let Ok(fd) = fd else {
                continue;
            };
            let Ok(link) = fs::read_link(fd.path()) else {
                continue;
            };
            let link = link.to_string_lossy();
            let Some(inode) = parse_socket_inode(&link) else {
                continue;
            };
            if map.contains_key(&inode) {
                continue;
            }
            let comm = read_comm(&entry.path()).unwrap_or_else(|| "unknown".into());
            let exe = read_exe(&entry.path());
            map.insert(inode, (comm, pid, exe));
        }
    }
    Ok(map)
}

fn parse_socket_inode(link: &str) -> Option<u64> {
    let rest = link.strip_prefix("socket:[")?.strip_suffix(']')?;
    rest.parse().ok()
}

fn read_comm(proc_pid: &Path) -> Option<String> {
    let raw = fs::read_to_string(proc_pid.join("comm")).ok()?;
    Some(raw.trim().to_string())
}

fn read_exe(proc_pid: &Path) -> Option<String> {
    let link = fs::read_link(proc_pid.join("exe")).ok()?;
    Some(link.to_string_lossy().into_owned())
}

fn suspicious_exec_path(path: &str) -> bool {
    SUSPICIOUS_EXEC_PREFIXES
        .iter()
        .any(|prefix| path.starts_with(prefix))
}

fn fill_exe_hashes(conns: &mut [ObservedConn], hash_any_exe: bool) {
    let mut budget = MAX_HASHES_PER_COLLECT;
    let mut cache: HashMap<String, Option<String>> = HashMap::new();
    for conn in conns.iter_mut() {
        let Some(path) = conn.exe_path.clone() else {
            continue;
        };
        if let Some(cached) = cache.get(&path) {
            conn.exe_sha256.clone_from(cached);
            continue;
        }
        let digest = hash_exe_if_needed(&path, hash_any_exe, &mut budget);
        cache.insert(path, digest.clone());
        conn.exe_sha256 = digest;
    }
}

fn hash_exe_if_needed(path: &str, hash_any_exe: bool, budget: &mut usize) -> Option<String> {
    if !hash_any_exe && !suspicious_exec_path(path) {
        return None;
    }
    let meta = fs::metadata(path).ok()?;
    if !meta.is_file() || meta.len() > MAX_EXE_HASH_BYTES {
        return None;
    }
    if *budget == 0 {
        return None;
    }
    *budget = budget.saturating_sub(1);
    sha256_file(Path::new(path)).ok()
}

fn sha256_file(path: &Path) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0_u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::os::unix::fs::symlink;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn parse_ipv4_established_line() {
        // 198.51.100.20 → /proc hex 146433C6 on little-endian hosts.
        let line = "   0: 0100007F:0016 146433C6:9C40 01 00000000:00000000 00:00000000 00000000     0        0 12345 1 0000000000000000 100 0 0 10 0";
        let row = parse_proc_net_line(line, false).expect("row");
        assert_eq!(row.local_port, 22);
        assert_eq!(row.remote_port, 0x9c40);
        assert_eq!(row.state, 1);
        assert_eq!(row.inode, 12345);
        assert_eq!(row.remote_ip, "198.51.100.20");
    }

    #[test]
    fn fixture_proc_collects_established() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("seat_shield_proc_{stamp}"));
        fs::create_dir_all(root.join("net")).unwrap();
        let mut tcp = fs::File::create(root.join("net/tcp")).unwrap();
        writeln!(tcp, "  sl  local_address rem_address   st").unwrap();
        writeln!(
            tcp,
            "   0: 0100007F:0016 146433C6:9C40 01 00000000:00000000 00:00000000 00000000     0        0 4242 1 0000000000000000 100 0 0 10 0"
        )
        .unwrap();
        let pid_dir = root.join("4242");
        fs::create_dir_all(pid_dir.join("fd")).unwrap();
        fs::write(pid_dir.join("comm"), "sshd\n").unwrap();
        symlink("/usr/sbin/sshd", pid_dir.join("exe")).unwrap();
        symlink("socket:[4242]", pid_dir.join("fd/3")).unwrap();

        let src = ProcConnSource::new(&root);
        let conns = src.collect().unwrap();
        assert_eq!(conns.len(), 1);
        assert_eq!(conns[0].local_port, 22);
        assert_eq!(conns[0].remote_ip, "198.51.100.20");
        assert_eq!(conns[0].process_name, "sshd");
        assert_eq!(conns[0].pid, 4242);
        assert_eq!(conns[0].exe_path.as_deref(), Some("/usr/sbin/sshd"));
        assert!(conns[0].exe_sha256.is_none());
        let _ = fs::remove_dir_all(root);
    }

    fn proc_fixture(root: &Path, exe_target: &Path) {
        fs::create_dir_all(root.join("net")).unwrap();
        let mut tcp = fs::File::create(root.join("net/tcp")).unwrap();
        writeln!(tcp, "  sl  local_address rem_address   st").unwrap();
        writeln!(
            tcp,
            "   0: 0100007F:01BB 146433C6:9C40 01 00000000:00000000 00:00000000 00000000     0        0 777 1 0000000000000000 100 0 0 10 0"
        )
        .unwrap();
        let pid_dir = root.join("777");
        fs::create_dir_all(pid_dir.join("fd")).unwrap();
        fs::write(pid_dir.join("comm"), "drop\n").unwrap();
        symlink(exe_target, pid_dir.join("exe")).unwrap();
        symlink("socket:[777]", pid_dir.join("fd/3")).unwrap();
    }

    #[test]
    fn hashes_suspicious_tmp_exe_and_skips_huge_file() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let exe = PathBuf::from(format!("/tmp/seat_shield_s4_exe_{stamp}"));
        let huge = PathBuf::from(format!("/tmp/seat_shield_s4_huge_{stamp}"));
        let bytes = b"seat-shield-s4-fixture";
        fs::write(&exe, bytes).unwrap();
        let expected = format!("{:x}", Sha256::digest(bytes));

        let root = PathBuf::from(format!("/tmp/seat_shield_s4_proc_{stamp}"));
        proc_fixture(&root, &exe);
        let src = ProcConnSource::new(&root);
        let conns = src.collect().unwrap();
        assert_eq!(conns[0].exe_sha256.as_deref(), Some(expected.as_str()));

        let huge_root = PathBuf::from(format!("/tmp/seat_shield_s4_hugeproc_{stamp}"));
        let huge_file = fs::File::create(&huge).unwrap();
        huge_file.set_len(MAX_EXE_HASH_BYTES + 1).unwrap();
        proc_fixture(&huge_root, &huge);
        let huge_src = ProcConnSource::new(&huge_root);
        let huge_conns = huge_src.collect().unwrap();
        assert!(huge_conns[0].exe_path.is_some());
        assert!(huge_conns[0].exe_sha256.is_none());

        let _ = fs::remove_file(&exe);
        let _ = fs::remove_file(&huge);
        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_dir_all(huge_root);
    }

    #[test]
    fn hashes_non_suspicious_only_when_feed_has_hashes() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let home = std::env::var("HOME").expect("HOME");
        let exe = PathBuf::from(&home).join(format!(".seat_shield_s4_hash_{stamp}"));
        let exe_s = exe.to_string_lossy();
        assert!(
            !SUSPICIOUS_EXEC_PREFIXES
                .iter()
                .any(|prefix| exe_s.starts_with(prefix)),
            "fixture path must not be a suspicious prefix"
        );
        let bytes = b"not-tmp";
        fs::write(&exe, bytes).unwrap();
        let expected = format!("{:x}", Sha256::digest(bytes));
        let root = std::env::temp_dir().join(format!("seat_shield_s4_any_{stamp}"));
        proc_fixture(&root, &exe);
        let src = ProcConnSource::new(&root);
        let plain = src.collect().unwrap();
        assert!(plain[0].exe_sha256.is_none());
        let hashed = src.collect_with_hash(true).unwrap();
        assert_eq!(hashed[0].exe_sha256.as_deref(), Some(expected.as_str()));
        let _ = fs::remove_file(&exe);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn ioc_hash_match_via_proc_conn_source() {
        use crate::cycle::run_cycle_with_feed;
        use crate::feed::IocFeed;
        use crate::policy::SeatPolicy;

        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let exe = PathBuf::from(format!("/tmp/seat_shield_s4_ioc_{stamp}"));
        let bytes = b"seat-shield-ioc";
        fs::write(&exe, bytes).unwrap();
        let digest = format!("{:x}", Sha256::digest(bytes));
        let root = PathBuf::from(format!("/tmp/seat_shield_s4_iocproc_{stamp}"));
        proc_fixture(&root, &exe);
        let conns = ProcConnSource::new(&root).collect().unwrap();
        assert_eq!(conns[0].exe_sha256.as_deref(), Some(digest.as_str()));
        let feed = IocFeed::from_parts("s4".into(), Vec::new(), vec![digest]);
        let out = run_cycle_with_feed(&SeatPolicy::default(), &conns, Some(&feed));
        assert!(out.events.iter().any(|e| e.kind == "ioc_hash_match"));
        assert!(
            out.acts.is_empty(),
            "hash match must not propose BlockPeer"
        );
        let _ = fs::remove_file(&exe);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn garbage_proc_net_lines_are_skipped() {
        let long = "A".repeat(4096);
        let junk = [
            "",
            "not a proc line",
            "   0: ZZZZ:GGGG",
            "   0: 0100007F:0016",
            "   0: 0100007F:0016 146433C6:9C40 ZZ",
            long.as_str(),
        ];
        for line in junk {
            assert!(
                parse_proc_net_line(line, false).is_none(),
                "garbage must not become a row: {line:?}"
            );
            assert!(parse_proc_net_line(line, true).is_none());
        }
    }
}
