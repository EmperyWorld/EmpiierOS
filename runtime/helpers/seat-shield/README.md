# EmpiierOS Seat Shield — floor helpers (S3)

Shipped into the EmpiierOS overlay by `scripts/package-shell-gui-rootfs.sh`.
Trusted Substrate `--resume` / image bake still required for metal MATCH.

| Tree file | Overlay install path |
|-----------|----------------------|
| `seat_shield.nft` | `/usr/share/empiieros/seat-shield/seat_shield.nft` |
| `README.md` | `/usr/share/empiieros/seat-shield/README.md` |
| `empiieros-seat-shield-nftables.service` | `/etc/systemd/system/` + `multi-user.target.wants/` copy |
| built `seat-shield-fw` | `/usr/libexec/empiieros-seat-shield-fw` |
| `empiieros-seat-shield-watch.service` | `/etc/systemd/system/` + wants (S5B) |
| built `seat-shield-watch` | `/usr/libexec/empiieros-seat-shield-watch` |
| built `seat-shield-probe` (P3) | `/usr/libexec/empiieros-seat-shield-probe` |

Runtime auto-arms Helper when `/usr/libexec/empiieros-seat-shield-fw` (or `/usr/local/sbin/…`) exists; otherwise observe-only Unavailable.
Optional `empiieros-seat-shield-probe` enriches observe via inet_diag (RTT); missing/fail → `/proc`.
No eBPF. No AF_PACKET.
The fw unit is `Type=oneshot` (not a listener). Both units take `CAP_NET_ADMIN` and
must not set `IPAddressDeny` (nft is netlink).
The watch helper writes `{state}/seat_shield/watch.jsonl` only, uses
`PrivateTmp=false`, and has no `CAP_NET_ADMIN`. Journal lines may include
`sha256` (S5C) so Runtime can IOC-match when its `PrivateTmp` hides host
`/tmp`. Hold still needs a path Runtime can open. Do not share the Runtime
unit's `PrivateTmp` namespace.

Manual build (lab):

```bash
cargo build --manifest-path runtime/seat_shield_core/Cargo.toml --bin seat-shield-fw --release
install -D -m 0755 runtime/seat_shield_core/target/release/seat-shield-fw \
  /usr/libexec/empiieros-seat-shield-fw

cargo build --manifest-path runtime/seat_shield_probe/Cargo.toml --release
install -D -m 0755 runtime/target/release/seat-shield-probe \
  /usr/libexec/empiieros-seat-shield-probe
```

Contract: `session/SEAT-SHIELD-FW.md`. Seat Shield plane: `session/SEAT-SHIELD.md`.
