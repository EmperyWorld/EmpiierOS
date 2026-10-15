# EmpiierOS Seat Shield probe helper

**Plane:** Runtime helper (not Shell).
**Role:** One-shot `inet_diag` TCP dump for Seat Shield observe enrichment.
**No eBPF.** **No AF_PACKET.** **Not:** conntrack stream. **Not:** MATCH.

## ORIGIN

Harvested from `../ewonika/probe` (`diag.rs` + `inode.rs` SOCK_DIAG path only).
eBPF / conntrack / tracepoint modules are **not** copied.

## Build

```bash
cargo build --manifest-path runtime/seat_shield_probe/Cargo.toml --release
install -D -m 0755 runtime/target/release/seat-shield-probe \
  /usr/libexec/empiieros-seat-shield-probe
```

## CLI

```text
seat-shield-probe dump
```

Stdout: one JSON object (`t=snap`) with ESTAB TCP socks + optional `rtt_us` / `retrans`.
Exit non-zero on netlink failure (Runtime falls back to `/proc`).
