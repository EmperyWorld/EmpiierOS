use std::io::Read;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const MAX_CAPTURE_BYTES: usize = 512 * 1024;

#[derive(Debug)]
pub struct CommandResult {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
}

pub fn run(command: &str, args: &[&str], timeout: Duration) -> Result<CommandResult, String> {
    if !command.starts_with('/') {
        return Err("command path must be absolute".to_string());
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
        .map_err(|error| format!("spawn_failed: {error}"))?;

    let stdout_thread = child
        .stdout
        .take()
        .map(|stdout| thread::spawn(move || read_capped(stdout)));
    let stderr_thread = child
        .stderr
        .take()
        .map(|stderr| thread::spawn(move || read_capped(stderr)));

    let started = Instant::now();
    let (status, timed_out) = loop {
        match child.try_wait() {
            Ok(Some(status)) => break (status, false),
            Ok(None) if started.elapsed() < timeout => {
                thread::sleep(Duration::from_millis(25));
            }
            Ok(None) => {
                let _ = child.kill();
                let status = child
                    .wait()
                    .map_err(|error| format!("wait_after_kill_failed: {error}"))?;
                break (status, true);
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = join_capture(stdout_thread);
                let _ = join_capture(stderr_thread);
                return Err(format!("try_wait_failed: {error}"));
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
        let Ok(count) = reader.read(&mut buffer) else {
            break;
        };
        if count == 0 {
            break;
        }
        let remaining = MAX_CAPTURE_BYTES.saturating_sub(retained.len());
        retained.extend_from_slice(&buffer[..count.min(remaining)]);
    }
    retained
}

fn join_capture(handle: Option<thread::JoinHandle<Vec<u8>>>) -> String {
    handle
        .and_then(|thread| thread.join().ok())
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_runner_reports_success_and_failure() {
        let ok = run("/bin/true", &[], Duration::from_secs(1)).expect("true");
        assert!(ok.success);

        let failed = run("/bin/false", &[], Duration::from_secs(1)).expect("false");
        assert!(!failed.success);
        assert_eq!(failed.exit_code, Some(1));
    }

    #[test]
    fn command_runner_kills_timeout() {
        let result = run("/bin/sleep", &["5"], Duration::from_millis(50)).expect("sleep");
        assert!(!result.success);
        assert!(result.timed_out);
    }

    #[test]
    fn command_runner_rejects_path_lookup() {
        assert!(run("true", &[], Duration::from_secs(1)).is_err());
    }
}
