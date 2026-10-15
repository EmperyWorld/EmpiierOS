use empiieros_runtime::protocol::{MAX_REQUEST_BYTES, MAX_RESPONSE_BYTES};
use empiieros_runtime::RuntimeCore;
use std::fs;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::mem::MaybeUninit;
use std::os::fd::AsRawFd;
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[cfg(all(feature = "production", feature = "dev-peer-bypass"))]
compile_error!("production and dev-peer-bypass features are mutually exclusive");

const DEFAULT_SOCKET: &str = "/run/empiieros/runtime.sock";
const DEFAULT_STATE_DIR: &str = "/var/lib/empiieros-runtime";
const DEFAULT_ALLOWED_BRIDGE_EXE: &str = "/usr/lib/systemd/systemd-socket-proxyd";
const DEFAULT_ALLOWED_BRIDGE_UID: u32 = 0;
const MAX_CONNECTIONS: usize = 16;

fn parse_seccomp_status(status: &str) -> Option<(u64, u64)> {
    let mut mode = None;
    let mut filters = 0;
    for line in status.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let value = value.trim();
        match key {
            "Seccomp" => mode = value.parse().ok(),
            "Seccomp_filters" => filters = value.parse().unwrap_or(0),
            _ => {}
        }
    }
    Some((mode?, filters))
}

fn main() {
    if let Err(error) = run() {
        eprintln!("empiieros-runtime: fatal: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let socket_path = env_path("EMPIIEROS_RUNTIME_SOCKET", DEFAULT_SOCKET);
    let state_path = env_path("EMPIIEROS_RUNTIME_STATE_DIR", DEFAULT_STATE_DIR);
    prepare_socket_parent(&socket_path)?;
    remove_stale_socket(&socket_path)?;

    let core = Arc::new(RuntimeCore::new(state_path)?);
    let listener =
        UnixListener::bind(&socket_path).map_err(|error| format!("socket_bind_failed: {error}"))?;
    fs::set_permissions(&socket_path, fs::Permissions::from_mode(0o600))
        .map_err(|error| format!("socket_mode_failed: {error}"))?;
    eprintln!(
        "empiieros-runtime: ready transport=unix-peercred-v1 socket={}",
        socket_path.display()
    );
    match fs::read_to_string("/proc/self/status") {
        Ok(status) => match parse_seccomp_status(&status) {
            Some((mode, filters)) => {
                eprintln!("empiieros-runtime: seccomp mode={mode} filters={filters}")
            }
            None => eprintln!("empiieros-runtime: seccomp status_unparsed"),
        },
        Err(error) => eprintln!("empiieros-runtime: seccomp status_unreadable: {error}"),
    }

    let active = Arc::new(AtomicUsize::new(0));
    for incoming in listener.incoming() {
        let stream = match incoming {
            Ok(stream) => stream,
            Err(error) => {
                eprintln!("empiieros-runtime: accept_failed: {error}");
                continue;
            }
        };
        if active
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                (current < MAX_CONNECTIONS).then_some(current + 1)
            })
            .is_err()
        {
            let mut stream = stream;
            let response = empiieros_runtime::protocol::RuntimeResponse::failure(
                "",
                "busy",
                "connection_limit",
                "runtime connection limit reached",
            )
            .encode_line();
            let _ = stream.write_all(&response);
            continue;
        }

        let core = Arc::clone(&core);
        let active = Arc::clone(&active);
        thread::spawn(move || {
            if let Err(error) = handle_connection(stream, &core) {
                eprintln!("empiieros-runtime: request_failed: {error}");
            }
            active.fetch_sub(1, Ordering::AcqRel);
        });
    }
    Ok(())
}

fn handle_connection(mut stream: UnixStream, core: &RuntimeCore) -> Result<(), String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|error| format!("set_read_timeout_failed: {error}"))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(30)))
        .map_err(|error| format!("set_write_timeout_failed: {error}"))?;

    let peer = peer_credentials(&stream)
        .map_err(|error| format!("peer_credentials_failed: {error}"))?;
    if let Err(reason) = authorize_peer(peer) {
        eprintln!(
            "empiieros-runtime: peer_authorization_denied pid={} uid={} gid={} code={} detail={}",
            peer.pid, peer.uid, peer.gid, reason.code, reason.detail
        );
        core.record_security_event("ipc.peer", &format!("{}: {}", reason.code, reason.detail));
        let response = empiieros_runtime::protocol::RuntimeResponse::failure(
            "",
            "denied",
            reason.code,
            format!("IPC peer is not authorized: {}", reason.code),
        )
        .encode_line();
        stream
            .write_all(&response)
            .map_err(|error| format!("write_denial_failed: {error}"))?;
        return Ok(());
    }

    let mut request = Vec::new();
    let bytes_read = BufReader::new(&stream)
        .take((MAX_REQUEST_BYTES + 2) as u64)
        .read_until(b'\n', &mut request)
        .map_err(|error| format!("request_read_failed: {error}"))?;
    if bytes_read == 0 || request.last() != Some(&b'\n') {
        return write_protocol_failure(
            &mut stream,
            "invalid_request",
            "newline_terminated_request_required",
        );
    }
    request.pop();
    if request.last() == Some(&b'\r') {
        request.pop();
    }
    let response = core.handle_bytes(&request);
    eprintln!(
        "empiieros-runtime: response id={} ok={} status={}",
        response.id, response.ok, response.status
    );
    let response = response.encode_line();
    if response.len() > MAX_RESPONSE_BYTES {
        return write_protocol_failure(&mut stream, "internal_error", "response_too_large");
    }
    stream
        .write_all(&response)
        .map_err(|error| format!("response_write_failed: {error}"))
}

fn write_protocol_failure(
    stream: &mut UnixStream,
    status: &str,
    code: &str,
) -> Result<(), String> {
    let response = empiieros_runtime::protocol::RuntimeResponse::failure(
        "",
        status,
        code,
        "request framing is invalid",
    )
    .encode_line();
    stream
        .write_all(&response)
        .map_err(|error| format!("response_write_failed: {error}"))
}

#[derive(Debug, Clone, Copy)]
struct PeerCredentials {
    pid: i32,
    uid: u32,
    gid: u32,
}

#[derive(Debug)]
struct PeerAuthorizationError {
    code: &'static str,
    detail: String,
}

fn peer_credentials(stream: &UnixStream) -> io::Result<PeerCredentials> {
    let mut credentials = MaybeUninit::<libc::ucred>::uninit();
    let mut length = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    // SAFETY: `credentials` points to writable storage sized for `libc::ucred`,
    // `length` is initialized to that exact size, and `stream` owns a valid fd.
    let result = unsafe {
        libc::getsockopt(
            stream.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            credentials.as_mut_ptr().cast(),
            &mut length,
        )
    };
    if result != 0 {
        return Err(io::Error::last_os_error());
    }
    if length as usize != std::mem::size_of::<libc::ucred>() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unexpected SO_PEERCRED length",
        ));
    }
    // SAFETY: successful getsockopt initialized all bytes of `ucred`.
    let credentials = unsafe { credentials.assume_init() };
    Ok(PeerCredentials {
        pid: credentials.pid,
        uid: credentials.uid,
        gid: credentials.gid,
    })
}

fn authorize_peer(peer: PeerCredentials) -> Result<(), PeerAuthorizationError> {
    #[cfg(feature = "dev-peer-bypass")]
    if std::env::var("EMPIIEROS_DEV_ALLOW_UNVERIFIED_PEER").as_deref() == Ok("1") {
        return Ok(());
    }

    let allowed_uid = std::env::var("EMPIIEROS_ALLOWED_BRIDGE_UID")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(DEFAULT_ALLOWED_BRIDGE_UID);
    let expected = env_path(
        "EMPIIEROS_ALLOWED_BRIDGE_EXE",
        DEFAULT_ALLOWED_BRIDGE_EXE,
    );
    authorize_peer_identity(peer, allowed_uid, &expected)
}

fn authorize_peer_identity(
    peer: PeerCredentials,
    allowed_uid: u32,
    expected: &Path,
) -> Result<(), PeerAuthorizationError> {
    if peer.uid != allowed_uid {
        return Err(PeerAuthorizationError {
            code: "peer_uid_mismatch",
            detail: format!(
                "uid mismatch: observed={} expected={} gid={}",
                peer.uid, allowed_uid, peer.gid
            ),
        });
    }
    if peer.pid <= 1 {
        return Err(PeerAuthorizationError {
            code: "peer_pid_invalid",
            detail: "invalid peer pid".to_string(),
        });
    }

    let expected_metadata = fs::metadata(expected).map_err(|error| PeerAuthorizationError {
        code: "trusted_executable_unavailable",
        detail: format!("trusted executable unavailable: {error}"),
    })?;
    if !expected_metadata.is_file()
        || expected_metadata.uid() != unsafe { libc::geteuid() }
        || expected_metadata.permissions().mode() & 0o022 != 0
        || expected_metadata.permissions().mode() & 0o111 == 0
    {
        return Err(PeerAuthorizationError {
            code: "trusted_executable_insecure",
            detail: format!(
                "trusted executable owner/mode invalid: uid={} mode={:o}",
                expected_metadata.uid(),
                expected_metadata.permissions().mode() & 0o777
            ),
        });
    }
    let observed_path = format!("/proc/{}/exe", peer.pid);
    let observed_metadata =
        fs::metadata(&observed_path).map_err(|error| PeerAuthorizationError {
            code: "peer_executable_unavailable",
            detail: format!("peer executable unavailable: {error}"),
        })?;
    if observed_metadata.dev() != expected_metadata.dev()
        || observed_metadata.ino() != expected_metadata.ino()
    {
        let observed = fs::read_link(&observed_path)
            .map(|path| path.display().to_string())
            .unwrap_or_else(|_| "(unavailable)".to_string());
        return Err(PeerAuthorizationError {
            code: "peer_executable_mismatch",
            detail: format!(
                "executable identity mismatch: observed={observed} expected={}",
                expected.display()
            ),
        });
    }
    Ok(())
}

fn prepare_socket_parent(socket_path: &Path) -> Result<(), String> {
    let parent = socket_path
        .parent()
        .ok_or_else(|| "socket path has no parent".to_string())?;
    let metadata = fs::symlink_metadata(parent)
        .map_err(|error| format!("socket_parent_unavailable: {error}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("socket parent is not a secure directory".to_string());
    }
    if metadata.uid() != unsafe { libc::geteuid() } {
        return Err("socket parent is not owned by the runtime effective uid".to_string());
    }
    if metadata.permissions().mode() & 0o022 != 0 {
        return Err("socket parent is group/world writable".to_string());
    }
    Ok(())
}

fn remove_stale_socket(path: &Path) -> Result<(), String> {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return Ok(());
    };
    if !metadata.file_type().is_socket() {
        return Err("refusing to replace non-socket runtime path".to_string());
    }
    match UnixStream::connect(path) {
        Ok(_) => return Err("refusing to replace an active runtime socket".to_string()),
        Err(error) if error.kind() == io::ErrorKind::ConnectionRefused => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(format!(
                "runtime socket liveness probe failed closed: {error}"
            ));
        }
    }
    fs::remove_file(path).map_err(|error| format!("stale_socket_remove_failed: {error}"))
}

fn env_path(name: &str, default: &str) -> PathBuf {
    std::env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(default))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_socket_guard_refuses_regular_file() {
        let root = std::env::temp_dir().join(format!(
            "empiieros-daemon-guard-{}",
            std::process::id()
        ));
        let _ = fs::create_dir_all(&root);
        let path = root.join("runtime.sock");
        fs::write(&path, b"not a socket").expect("file");
        assert!(remove_stale_socket(&path).is_err());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn stale_socket_guard_refuses_active_listener_and_removes_stale_socket() {
        let root = std::env::temp_dir().join(format!(
            "empiieros-daemon-active-{}",
            std::process::id()
        ));
        let _ = fs::create_dir_all(&root);
        let socket = root.join("runtime.sock");
        let listener = UnixListener::bind(&socket).expect("bind");
        assert!(remove_stale_socket(&socket).is_err());
        drop(listener);
        remove_stale_socket(&socket).expect("remove stale socket");
        assert!(!socket.exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn socket_parent_must_not_be_group_or_world_writable() {
        let root = std::env::temp_dir().join(format!(
            "empiieros-daemon-parent-{}",
            std::process::id()
        ));
        let _ = fs::create_dir_all(&root);
        fs::set_permissions(&root, fs::Permissions::from_mode(0o777)).expect("mode");
        assert!(prepare_socket_parent(&root.join("runtime.sock")).is_err());
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).expect("mode");
        assert!(prepare_socket_parent(&root.join("runtime.sock")).is_ok());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn own_process_peer_credentials_are_observable() {
        let root = std::env::temp_dir().join(format!(
            "empiieros-daemon-peer-{}",
            std::process::id()
        ));
        let _ = fs::create_dir_all(&root);
        let socket = root.join("peer.sock");
        let listener = UnixListener::bind(&socket).expect("bind");
        let join = thread::spawn(move || UnixStream::connect(socket).expect("connect"));
        let (server, _) = listener.accept().expect("accept");
        let credentials = peer_credentials(&server).expect("credentials");
        assert_eq!(credentials.uid, unsafe { libc::geteuid() });
        drop(join.join().expect("client"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn parse_seccomp_status_reads_mode_and_filter_count() {
        let status = "Name:\tempiieros-runtime\nSeccomp:\t2\nSeccomp_filters:\t1\n";
        assert_eq!(parse_seccomp_status(status), Some((2, 1)));
        assert_eq!(parse_seccomp_status("Name:\tfoo\n"), None);
        assert_eq!(
            parse_seccomp_status("Seccomp:\t0\nNostale:\t1\n"),
            Some((0, 0))
        );
    }

    #[test]
    fn peer_authorization_uses_executable_identity_and_structured_errors() {
        let executable = std::env::current_exe().expect("current executable");
        let peer = PeerCredentials {
            pid: std::process::id() as i32,
            uid: unsafe { libc::geteuid() },
            gid: unsafe { libc::getegid() },
        };
        authorize_peer_identity(peer, peer.uid, &executable).expect("own executable accepted");

        let denied = authorize_peer_identity(peer, peer.uid.saturating_add(1), &executable)
            .expect_err("wrong uid denied");
        assert_eq!(denied.code, "peer_uid_mismatch");
    }
}
