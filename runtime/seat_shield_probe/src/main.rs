//! EmpiierOS `seat-shield-probe` — one-shot inet_diag dump.
//!
//! ORIGIN: ewonika `probe` SOCK_DIAG path only. No eBPF. No AF_PACKET.

mod diag;
mod inode;
mod json;

use crate::diag::{dump_estab_tcp, sock_json};
use crate::inode::map_socket_inodes;
use std::io::{self, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let cmd = args.next().unwrap_or_else(|| "dump".into());
    if cmd != "dump" || args.next().is_some() {
        eprintln!("usage: seat-shield-probe dump");
        return ExitCode::from(2);
    }
    match run_dump() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("seat-shield-probe: {err}");
            ExitCode::from(1)
        }
    }
}

fn run_dump() -> Result<(), Box<dyn std::error::Error>> {
    let inodes = map_socket_inodes();
    let socks = dump_estab_tcp(&inodes)?;
    let arr = socks.iter().map(sock_json).collect::<Vec<_>>().join(",");
    let line = format!(
        "{{\"t\":\"snap\",\"backend\":\"inet_diag\",\"reason\":\"dump\",\"socks\":[{}]}}",
        arr
    );
    let mut out = io::stdout();
    writeln!(out, "{line}")?;
    out.flush()?;
    Ok(())
}
