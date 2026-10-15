//! Absolute-path command runner (EmpiierOS Runtime kinship).
//!
//! No shell. Path must be absolute. Environment is cleared except PATH/LANG.

use std::io::Read;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::error::{ShieldError, ShieldResult};

const MAX_CAPTURE_BYTES: usize = 512 * 1024;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug)]
pub struct CommandResult {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
}

pub fn run(command: &str, args: &[&str], timeout: Duration) -> ShieldResult<CommandResult> {
    if !command.starts_with('/') {
        return Err(ShieldError::Actuator(
            "helper path must be absolute".into(),
        ));
    }
    if command.contains('\0') || args.iter().any(|a| a.contains('\0')) {
        return Err(ShieldError::Actuator(
            "NUL byte in helper argv is forbidden".into(),
        ));
    }

    let mut child = Command::new(command)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env_clear()
        .env("PATH", "/usr/sbin:/usr/bin:/sbin:/bin")
        .env("LANG", "C")
        .spawn()
        .map_err(|source| ShieldError::Io {
            context: "helper_spawn",
            source,
        })?;

    let stdout_thread = child
        .stdout
        .take()
        .map(|stdout| thread::spawn(move || read_capped(stdout)));
    let stderr_thread = child
        .stderr
        .take()
        .map(|stderr| thread::spawn(move || read_capped(stderr)));

    let started = Instant::now();
    let timeout = if timeout.is_zero() {
        DEFAULT_TIMEOUT
    } else {
        timeout
    };
    let (status, timed_out) = loop {
        match child.try_wait() {
            Ok(Some(status)) => break (status, false),
            Ok(None) if started.elapsed() < timeout => {
                thread::sleep(Duration::from_millis(25));
            }
            Ok(None) => {
                let _ = child.kill();
                let status = child.wait().map_err(|source| ShieldError::Io {
                    context: "helper_wait_after_kill",
                    source,
                })?;
                break (status, true);
            }
            Err(source) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = join_capture(stdout_thread);
                let _ = join_capture(stderr_thread);
                return Err(ShieldError::Io {
                    context: "helper_try_wait",
                    source,
                });
            }
        }
    };

    let stdout = join_capture(stdout_thread);
    let stderr = join_capture(stderr_thread);
    Ok(CommandResult {
        success: status.success() && !timed_out,
        exit_code: status.code(),
        stdout,
        stderr,
        timed_out,
    })
}

fn read_capped(mut reader: impl Read) -> Vec<u8> {
    let mut retained = Vec::new();
    let mut buffer = [0_u8; 8192];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                let room = MAX_CAPTURE_BYTES.saturating_sub(retained.len());
                retained.extend_from_slice(&buffer[..n.min(room)]);
                if retained.len() >= MAX_CAPTURE_BYTES {
                    break;
                }
            }
            Err(_) => break,
        }
    }
    retained
}

fn join_capture(handle: Option<thread::JoinHandle<Vec<u8>>>) -> String {
    let bytes = handle
        .and_then(|h| h.join().ok())
        .unwrap_or_default();
    String::from_utf8_lossy(&bytes).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_relative_path() {
        let err = run("echo", &["hi"], Duration::from_secs(1)).unwrap_err();
        assert!(err.to_string().contains("absolute"));
    }
}
