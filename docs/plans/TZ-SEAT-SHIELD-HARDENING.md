# TZ — Seat Shield hardening (compiler · isolation · audit)

**Status:** Binding work order. **Not** acceptance. **Not** `READY` / `PASSED` / `DEMO_PASSED`. **Not** “fully invulnerable”.
**Date:** 2026-09-20
**Product:** EmpiierOS Seat Shield (`seat.shield.*`) plus the Runtime/helper floor it already uses
**Companion (product juice / S5):** [`TZ-SEAT-SHIELD-POST-S4.md`](TZ-SEAT-SHIELD-POST-S4.md)
**Living contract:** [`../../session/SEAT-SHIELD.md`](../../session/SEAT-SHIELD.md) · [`../../session/SEAT-SHIELD-FW.md`](../../session/SEAT-SHIELD-FW.md)
**Twin:** `security.integrity.*` stays **`disabled`**. This TZ does **not** enable it.
**Honest label:** harden the seat we have. Do not replace it with a second promiscuous IDS daemon.

If this file conflicts with POST-S4 on S4/S5 product scope, **POST-S4 wins**.
If this file conflicts with a generic “maximum armor / AF_PACKET / musl / MIT-only OS” brief, **this file wins**.

## Progress (2026-09-20)

Not MATCH. Not invulnerable. Auto-act still off.

| Wave | State | Evidence |
|------|-------|----------|
| H0 | done | this file |
| H1 | **measured + lab mux** | Official image now `5979e9cf…` (LUKS floor). Prior host bake `4104a67b…` superseded. Live Runtime still a lab mux. Auto-act off. Not MATCH. |
| H2 | **measured** | packager + `.cargo/config.toml`. Cross-build gnu: Runtime `0a0c2d42…`, fw `5282e024…`. `readelf`: `GNU_RELRO`, `GNU_STACK RW`, `BIND_NOW` + `PIE`. `panic=abort` on release. Not musl. |
| H3 | **measured on this host** | `cargo-audit` 0.22.2. Script audits both lockfiles `--deny unsound --deny yanked`, exit 0, 0 ignores. Report [`SEAT-SHIELD-H3-AUDIT.md`](SEAT-SHIELD-H3-AUDIT.md). Hosts without the tool still SKIP. |
| H4 | **measured** | crate lib **69** + `parser_fuzz` **1** + watch-bin **2**. Runtime `seat_shield` **9**. Stable corpus+PRNG. `cargo fuzz` optional (`fuzz/README.md`). |
| H5 | **measured** | oneshot jail + preflight `ProtectHome=tmpfs` / `ProtectSystem=strict` / `IPAddressDeny=` absent. No Landlock. |
| H6 | **design locked; live blocked** | Kinship already `BR2_PACKAGE_LIBSECCOMP=y`. Official QEMU: systemd `+SECCOMP`. Live systemd 256.7 **`-SECCOMP`**, Runtime `Seccomp: 0`. Design SoT [`TZ-SEAT-SHIELD-H6-PRIVATETMP.md`](TZ-SEAT-SHIELD-H6-PRIVATETMP.md) (2026-09-21): Runtime `PrivateTmp=true` stays; watch `false`; preflight needles. Full measure only after fresh install. |
| S5A-OS | **pasted + muxed + baked** | Crate 0.6.0. Lab `20260919T212434Z-runtime-3042` / `212449Z-shell-32029`. `/etc/passwd` denied. `/dev/shm` scan observed. Host `/tmp` hidden by live `PrivateTmp=true` (unit `dd6bb219…` reused). |
| S5B-OS | **baked + muxed** | Overlay `d1fbef23…`. Lab mux 16:13 +10 then superseded by S5C. |
| S5C | **muxed + baked** | Overlay `e0a3fb8a…` `out/` `82ac20ac…` watch `c40d6d74…`. Live unstripped Runtime `d1360890…` watch `8525c72b…`. |
| I | forbidden | |

**Gates this turn:** `scripts/preflight-foundation.sh` **RESULT: PASS**. Crate 56 + Runtime lib 156. Cause of first preflight FAIL: unit **comment** contained `IPAddressDeny` and the needle was a substring. Lock: needle is now `IPAddressDeny=`.

**H6/PrivateTmp design (2026-09-21):** companion [`TZ-SEAT-SHIELD-H6-PRIVATETMP.md`](TZ-SEAT-SHIELD-H6-PRIVATETMP.md). Preflight now asserts kinship Runtime `PrivateTmp=true` + `SystemCallFilter=~@debug`. Live H6 still blocked until fresh install.

---

## 0. What the brief asked vs what EmpiierOS is

The operator brief is a generic Buildroot “security node” dump (compiler flags, musl, Landlock, etherparse + AF_PACKET, notify on `/etc`, `nft` CLI, cargo audit/fuzz, kernel menuconfig). Some of it is real engineering. Most of it is a **second product**.

EmpiierOS Seat Shield is already a triad:

```text
OS Shell (display only)
  + EliteEW Runtime (Unix NDJSON, SO_PEERCRED, IPAddressDeny=any)
  + pinned helper argv → nft sets (blocked4 / blocked6)
```

It defends the **EliteEW seat** (FoFline / Capsule / PIN / SSH break-glass). It is not Suricata, not AIDE, not a promiscuous gateway, not certified antivirus.

| Brief claim | EmpiierOS law |
|-------------|---------------|
| “Maximum invulnerability” / 100% block | **Forbidden label.** Ledger stays `NOT_READY`. |
| MIT/Apache only, no copyleft | **Refuse as OS law.** WebKitGTK, nftables, systemd, Buildroot pieces are not MIT. Shield crate is already `license = "MIT"`. License **report** is useful; a GPL-fail gate that rejects the floor is sabotage. |
| `#![forbid(unsafe_code)]` everywhere | **Keep on `seat_shield_core`.** Runtime already uses `libc` for sockets/`SO_PEERCRED`. Native GUI is C. Do not forbid unsafe on the whole Runtime or invent an AF_PACKET path that requires `libc` unsafe. |
| Zero-allocation / no heap | **Adapt.** We already **bound** alloc (32 MiB / 32 hashes / 2048 conns / 512 KiB helper capture). IPC is NDJSON + `serde_json` — that allocates. Do not rewrite the control plane as `no_std`. |
| Lock-free `crossbeam` 10k ring | **Refuse.** Events already persist under `{state}/seat_shield/events.json` (`0o600`, capped). HUD ticks `seat.shield.status` every 12s. A second bus is a second SoT. |
| `x86_64-unknown-linux-musl` + wipe `/lib` | **Refuse.** Living target is Buildroot **glibc** (`x86_64-unknown-linux-gnu`). WebKit, GTK, GStreamer, nft, ALSA need shared libs. |
| AF_PACKET + etherparse + promiscuous | **Refuse in this TZ.** Runtime must stay `IPAddressDeny=any`. Port-scan / protected-port logic already reads `/proc/net/tcp{,6}`. TCP flags (Xmas / SYN+FIN) are **invisible** on that table — that is honesty, not a missing crate. |
| `nft add element` from the analyzer | **Refuse.** Protocol is [`SEAT-SHIELD-FW.md`](../../session/SEAT-SHIELD-FW.md): absolute helper, env cleared, argv only. |
| `notify` on `/etc/passwd` | **Refuse.** That is the integrity twin. Ledger: disabled. |
| `CONFIG_MODULES` off | **Refuse.** Living seat needs modular `ath10k` and `wireguard.ko`. |
| Landlock + seccomp whitelist (`read`/`write`/`epoll` only) | **Adapt, do not copy.** Runtime must open `/proc`, the frontend socket, and exec the helper. A toy whitelist kills the product. Real missing lock is **systemd built with libseccomp** so the unit’s `SystemCallFilter` actually applies. |
| rustls / ring / no OpenSSL | **Already done** on Runtime (Air/OIDC). Not a Shield work item. No pqcrypto in this TZ. |
| cargo clippy / audit / fuzz / vendor | **Keep**, scoped. |

---

## 1. Current armor (evidence, not marketing)

Measured from this tree and the 2026-09-20 lab diary. Not MATCH.

### 1.1 Seat Shield (already stronger than the brief’s empty daemon)

| Control | Where | Honest state |
|---------|-------|----------------|
| `unsafe_code = "forbid"` | `runtime/seat_shield_core/Cargo.toml` | On. Clippy `pedantic` is still `allow`. |
| No shell nft | `command_runner.rs`, `fw/nft.rs` | Absolute argv; `env_clear`; 5s timeout; 512 KiB capture cap. |
| Observe | `observe/proc.rs` | `/proc/net/tcp{,6}` + inode map. Caps: 4096 PIDs, 512 fds/pid, 2048 conns. |
| Live exe SHA-256 | S4 paste 0.5.0 | Only tmp/shm **or** sealed feed hashes; skip > 32 MiB; cap 32 files. |
| Cycle | `seat.shield.status` → `maybe_auto_observe` | HUD 12s tick. Interval on disk (`cycle_state.json`). |
| Acts | helper + confirms | Default `act_enabled=false`, `auto_act_protected=false`. Ports `[22]` only. |
| Behavioral | events only | `possible_port_scan`, `network_suspicious_port`, `kernel_taint`. Never auto-act. |
| nft list | `fw/nft.rs` | JSON first; text fallback when nft has no libjansson (this floor). |
| Lab | `192.168.8.10` | helper `732e3b9e…`, `backend=helper`, auto-act off. |
| Official `out/` | `rootfs.ext2` `c9805fc6…` | Helper `1a3856a2…` has text-list. Previous `c7aea2e4…` / `067ef5a2…` superseded. Not metal MATCH. |

### 1.2 Runtime / floor (already a Zero-Trust split)

| Control | Honest state |
|---------|----------------|
| `IPAddressDeny=any` | Production invariant. Portal/SPA/Air helpers talk; Runtime does not. |
| Capabilities | `CAP_NET_ADMIN` only. `CAP_SYS_PTRACE` removed. |
| `ProtectSystem=strict`, `NoNewPrivileges` | On the live unit. |
| `SystemCallFilter=~@debug` | **In the unit file. Not applied.** systemd 256.7 is `-SECCOMP`; `libseccomp` absent. `/proc` shows `Seccomp: 0`. This is the real seccomp debt. |
| `SO_PEERCRED` | Frontend socket via `systemd-socket-proxyd`. |
| Crypto | `rustls` + `ring` already in `runtime/Cargo.toml`. |
| Compiler profile | Packager + `runtime/.cargo/config.toml` gnu PIC/RELRO/NX; release `panic=abort`. Official Runtime `04358653…` measured `GNU_RELRO` + `GNU_STACK RW` + `BIND_NOW` + `PIE`. Not musl. |

### 1.3 Kernel / image

| Brief ask | Living seat |
|-----------|-------------|
| `CONFIG_SECCOMP` | Kernel actions exist (`actions_avail`). Userspace systemd cannot attach filters. |
| KASLR / `STRICT_DEVMEM` | Kinship kernel fragment work — **not** a Shield crate change. Audit before flipping. |
| `CONFIG_MODULES=n` | **Must stay `y`** (`linux-wifi.fragment`). |

---

## 2. Cause (why a naive paste fails)

1. **Second control plane.** An AF_PACKET daemon that calls `nft` is a skin + a privilege hole. The Shell would paint it; the floor would mutate outside `seat.shield.act`.
2. **Wrong linker.** musl static Runtime cannot host WebKit/GTK on this image.
3. **Wrong sensor.** `/proc/net/tcp` does not carry TCP flags. Adding etherparse without AF_PACKET does nothing. Adding AF_PACKET without a new capability and a new unit punches `IPAddressDeny` or creates a CAP_NET_RAW helper we do not have.
4. **Wrong jail.** Landlock/`seccomp` that blocks `/proc` or `execve(/usr/libexec/empiieros-seat-shield-fw)` makes Observe and Block die. That is not hardening.
5. **Wrong integrity.** Watching `/etc/passwd` from Shield forks the disabled twin and will be sold as “protected”.
6. **Lying compiler story.** `forbid(unsafe)` + `libc` AF_PACKET cannot both be true.

---

## 3. Target (stay on the triad)

```text
                    HUD 12s / card
                         │ seat.shield.*
                         ▼
              Runtime (glibc, PIC/RELRO/NX)
              IPAddressDeny=any
              CAP_NET_ADMIN
              SystemCallFilter  ← only after systemd +libseccomp
                         │ exec argv, env cleared
                         ▼
         empiieros-seat-shield-fw  (forbid unsafe)
                         │ nft add/delete/list (text fallback)
                         ▼
              inet seat_shield { blocked4, blocked6 }

S5A (later, bay first): confirm-gated quarantine under state, tmp/shm only
S5B (later): pinned inotify helper on tmp/shm only — drain from observe
Integrity: still disabled. Shield only reports the twin as disabled.
```

No promiscuous socket. No second event bus. No `nft` strings.

---

## 4. Waves (one unlock each — do not shotgun)

| Wave | Write-home | Unlock phrase | Outcome |
|------|------------|---------------|---------|
| **H0 — this TZ** | EmpiierOS docs | (this task) | Filter the brief. No code. |
| **H1 — S4 floor close** | EmpiierOS + packager | Already in POST-S4 B / “continue Seat Shield” | **measured 2026-09-20** — `rootfs.ext2` `c9805fc6…` fw `1a3856a2…` text-list. QEMU nftables OK; smoke FAIL WebKit string. |
| **H2 — compiler armor** | EmpiierOS `runtime/` | Unlock EmpiierOS write for Seat Shield hardening H2 (RELRO/NX/panic-abort). | **measured 2026-09-20** — Runtime `0a0c2d42…` / fw `5282e024…`: RELRO + BIND_NOW + NX stack + PIE. Not musl. |
| **H3 — supply chain** | EmpiierOS scripts | Unlock EmpiierOS write for Seat Shield hardening H3 (audit/license report). | **measured 2026-09-20** — `cargo-audit` 0.22.2 exit 0 on both locks. SKIP remains if the tool is absent. |
| **H4 — parser fuzz** | EmpiierOS `seat_shield_core` | Unlock EmpiierOS write for Seat Shield hardening H4 (fuzz parsers). | **measured 2026-09-20** — `parser_probe` + corpus/PRNG test. Nightly `cargo fuzz` optional. |
| **H5 — helper jail** | EmpiierOS helper + unit | Unlock EmpiierOS write for Seat Shield hardening H5 (helper isolation). | **measured 2026-09-20** — oneshot jail + preflight `ProtectHome`/`ProtectSystem`. No Landlock. |
| **H6 — systemd seccomp** | Trusted Substrate kinship | Unlock Trusted Substrate kinship write for systemd +libseccomp so Runtime `SystemCallFilter` applies. | **image already has the lock; live blocked 2026-09-20** — QEMU `+SECCOMP` / `seccomp mode=2`. Live mux still `-SECCOMP` / `Seccomp: 0`. Reinstall forbidden. |
| **S5A / S5B** | bay first | POST-S4 phrases only | File containment / tmp watch. **Do not re-specify here.** |
| **I — integrity** | — | **Forbidden** | Keep disabled. |

Do **not** start H6 to “get Landlock”. Do **not** start S5A inside an H2 session. Do **not** enable auto-act from a hardening wave.

---

## 5. Wave H1 — S4 floor close (measured)

**Cause:** official image helper was JSON-only; this floor nft has no JSON. First `--build-image` then false-failed: `set -o pipefail` + `find \| rg -q` → SIGPIPE 141 while amdgpu firmware (638 files) was present.

**Lock:** `parse_nft_set_text` + preflight needles; kinship post-build presence check is `find -print -quit` into a var (same invariant: at least one amdgpu firmware file).

**Measure (2026-09-20):** `--resume` `RESUME_EXIT=0` `image_contract_ok`. `out/rootfs.ext2` `c9805fc6…` stamp `rootfs_sha256=89266529…`. Image helper `1a3856a2…` ≠ `067ef5a2…`; strings `nft text list` + `JSON support not compiled-in`. QEMU: `empiieros-seat-shield-nftables` Finished; smoke **FAIL** (`WebKit sandbox enabled` — do not weaken). Lab remux is not this image. Auto-act off.

Out of scope: ENABLE_ACTS, reinstall, S5.

---

## 6. Wave H2 — compiler armor (gnu, not musl)

### 6.1 In scope

Add **one** config that the packager already honors (`CARGO_TARGET_DIR` + Buildroot CC):

`runtime/.cargo/config.toml` (or packager `RUSTFLAGS` — prefer the packager so sandbox/Cargo target-dir cannot skip it):

```toml
[target.x86_64-unknown-linux-gnu]
rustflags = [
  "-C", "relocation-model=pic",
  "-C", "link-arg=-Wl,-z,relro,-z,now",
  "-C", "link-arg=-Wl,-z,noexecstack",
]

[profile.release]
lto = false          # keep false until a timed bake proves it; LTO+codegen-units=1 is a later opt
panic = "abort"
strip = "debuginfo"  # keep enough to diagnose; full strip is optional
```

Apply to bins the packager already cross-builds: `empiieros-runtime`, `seat-shield-fw`, Air/Portal/SPA helpers.

Keep `seat_shield_core` `unsafe_code = "forbid"`.

Optional later (not H2): `force-frame-pointers=yes` if crash dumps need it; conflicts with size goals.

### 6.2 Out of scope

- `x86_64-unknown-linux-musl`
- `panic=abort` on `cargo test` host (tests may unwind)
- `#![forbid(unsafe_code)]` in `runtime/src/daemon.rs` / `core.rs`
- Claiming ASLR from rustflags alone (kernel + PIE of the image also matter)

### 6.3 Measure

```text
readelf -l empiieros-runtime | rg 'GNU_RELRO|GNU_STACK'
# GNU_STACK PF_R|PF_W (not PF_X)
# GNU_RELRO present
# BIND_NOW: readelf -d | rg BIND_NOW
```

Same for `seat-shield-fw`. `scripts/preflight-foundation.sh` needle: rustflags strings in packager or `.cargo/config.toml`. Cross-build still uses Buildroot `x86_64-buildroot-linux-gnu-gcc`.

---

## 7. Wave H3 — supply chain

### 7.1 In scope

1. `cargo audit --manifest-path runtime/Cargo.toml --deny unsound --deny yanked` in bootstrap or a dedicated script. First run is a **report**; then pin/ignore with a dated reason. Do not delete a living crate to fake green.
2. `cargo tree --manifest-path runtime/seat_shield_core/Cargo.toml -i` + license list written under `docs/plans/` only if asked — or print in CI logs. Shield deps today: `serde`, `serde_json`, `sha2`, `thiserror` (MIT/Apache).
3. Optional `cargo geiger` on `seat_shield_core` as **advisory**. Runtime will show `libc` / rustls ring asm — that is expected.

### 7.2 Out of scope

- Fail the OS because WebKit/GTK/nft are LGPL/GPL.
- `cargo vendor` of the whole Runtime as a drive-by (large; kinship bake already pins `--locked`).
- pqcrypto, new TLS stacks.

### 7.3 Measure

Audit exit 0 or a checked deny-list file. Preflight does **not** weaken existing needles.

**Measured 2026-09-20:** `PATH="$HOME/.cargo/bin:$PATH" bash scripts/runtime-cargo-audit.sh` exit 0. `cargo-audit` 0.22.2 uses `--file` (not `--manifest-path`). Report [`SEAT-SHIELD-H3-AUDIT.md`](SEAT-SHIELD-H3-AUDIT.md).

---

## 8. Wave H4 — fuzz the parsers we actually have

Zero-alloc poetry does not replace tests. Fuzz the code that already parses untrusted bytes.

| Target | Bytes in | Fail if |
|--------|----------|---------|
| `parse_nft_set_text` | nft text | panic / hang |
| `parse_nft_set_json` | nft JSON | panic / hang |
| `parse_proc_net_table` line | `/proc/net/tcp` line | panic |
| `handle_runtime_line` | NDJSON | panic; never exec helper in fuzz |

Keep existing unit tests (empty set, inline elems, IPv6). `cargo fuzz` is optional host tooling; check in the targets, do not require 24h in preflight.

**Measured 2026-09-20:** `parser_probe` + `tests/parser_fuzz.rs` (corpus ≥6 + 2048 PRNG buffers). `Unavailable` actuator only. `CARGO_TARGET_DIR=runtime/target cargo test --manifest-path runtime/seat_shield_core/Cargo.toml --test parser_fuzz` exit 0. Nightly libFuzzer note in `runtime/seat_shield_core/fuzz/README.md`.

**Do not** add etherparse to get “Xmas” in H4. That signature needs a capture plane we refuse.

---

## 9. Wave H5 — helper isolation (narrow)

The helper is already short-lived `exec`, not a daemon. Jail the **binary**, do not invent Landlock on Runtime.

### 9.1 In scope

- Document and, if the image unit path is used, set on a **oneshot/template** (optional): `NoNewPrivileges=true`, `ProtectSystem=strict`, `ProtectHome=tmpfs`, `PrivateTmp=true`, `CapabilityBoundingSet=CAP_NET_ADMIN`, `AmbientCapabilities=CAP_NET_ADMIN`, **no** `IPAddressDeny` that breaks netlink to nft if that is how this nft talks.
- Runtime path stays `command_runner` (already `env_clear`, absolute path, timeout, cap).
- Refuse relative `--nft-bin` (already).
- Preflight: helper service file must not gain `WantedBy` that starts a listening daemon.
- Preflight needles: `ProtectHome=tmpfs`, `ProtectSystem=strict`, `Type=oneshot`, no `ListenStream`/`ListenDatagram`, no `IPAddressDeny=`.

**Measured 2026-09-20:** `scripts/preflight-foundation.sh` exit 0 with those needles. Landlock still later.

### 9.2 Out of scope

- Landlock that denies `/usr/sbin/nft` or netlink.
- `seccompiler` inside the helper that blocks `execve`/`socket`.
- Non-root Runtime (separate OS wave: sockets, state dir, CAP).
- Promiscuous `CAP_NET_RAW`.

### 9.3 Landlock — only if a later review says yes

Candidate: **S5B watch helper** (tmp/shm only), after it opens the watch fds. Not Runtime. Not the nft helper unless we prove `nft` still works.

---

## 10. Wave H6 — the real seccomp lock (kinship)

**Companion design SoT:** [`TZ-SEAT-SHIELD-H6-PRIVATETMP.md`](TZ-SEAT-SHIELD-H6-PRIVATETMP.md) (PrivateTmp split + post-reinstall checklist).

**Cause:** unit has `SystemCallFilter=~@debug`; the **installed** lab systemd was built `-SECCOMP`. Filters are theater on this seat. Separately, Runtime `PrivateTmp=true` hides host `/tmp` from `scan_path` — that is the jail, not a bug; watch stays `PrivateTmp=false` + S5C journal hash.

**Lock:** already in kinship (`BR2_PACKAGE_LIBSECCOMP=y` in `10-base.frag` + defconfig) and in the official image (QEMU `systemd +SECCOMP`, Runtime `seccomp mode=2`). Keep `SystemCallFilter=~@debug`. Keep Runtime `PrivateTmp=true`. Keep watch `PrivateTmp=false`. Preflight asserts all three needles.

**Measure (2026-09-20):** QEMU already meets the image bar. Live mux: systemd **`-SECCOMP`**, Runtime `Seccomp: 0`. Not metal MATCH.

**Measure (2026-09-21 design):** EmpiierOS preflight asserts kinship Runtime `PrivateTmp=true` + `SystemCallFilter=~@debug` and watch `PrivateTmp=false`. Full H6 metal close waits for operator reinstall phrase — see companion TZ §4.2.

Out of scope: Rust `seccompiler` rewrite, Runtime `PrivateTmp=false`, killing `execve` for the helper, WebKit bubblewrap (separate image debt).

---

## 11. What maps from the brief (cheat sheet)

| Brief item | Wave | Notes |
|------------|------|-------|
| PIC / RELRO / NOW / NX | H2 | gnu target |
| `panic=abort` | H2 | release bins only |
| `forbid(unsafe)` | already | core crate only |
| clippy pedantic | H3/H2 optional | do not flip to deny-all in one turn |
| cargo audit | H3 | |
| cargo geiger | H3 advisory | |
| cargo fuzz | H4 | our parsers, not etherparse |
| cargo vendor / `--offline` | later | `--locked` already |
| musl / wipe `/lib` | refuse | |
| AF_PACKET / etherparse / Xmas | refuse | |
| crossbeam ring | refuse | |
| tokio + `/var/log/messages` | refuse | SSH probes already via port 22 table |
| notify `/etc` | refuse | twin |
| `nft` CLI from Rust | refuse | helper only |
| Landlock | H5 later / S5B | |
| seccomp crate | refuse | H6 systemd |
| rustls/ring | already | |
| pqcrypto | refuse this TZ | |
| `CONFIG_MODULES=n` | refuse | |
| KASLR / STRICT_DEVMEM | kinship audit | not Shield |

S5 file containment stays POST-S4. The brief’s “HIDS” is S5B (tmp only), not `/etc`.

---

## 12. Acceptance

A wave is closed when its Measure commands exit 0 and a diary line exists. Verdict stays `NOT_READY`.

This TZ never authorizes:

- `DEMO_PASSED` / “антивирус готов” / “неуязвим”
- Lab reinstall
- Official `out/` without the named bake unlock
- Enabling `security.integrity.*` or `product.open`
- Punching `IPAddressDeny`
- A second privileged Python or Tokio security daemon

---

## 13. Operator phrases

H2:

> Unlock EmpiierOS write for Seat Shield hardening H2 (RELRO/NX/panic-abort).

H3:

> Unlock EmpiierOS write for Seat Shield hardening H3 (audit/license report).

H4:

> Unlock EmpiierOS write for Seat Shield hardening H4 (fuzz parsers).

H5:

> Unlock EmpiierOS write for Seat Shield hardening H5 (helper isolation).

H6 (image already locked; metal measure):

> Unlock metal reinstall from current official `out/` to measure Seat Shield H6 (+SECCOMP). No Runtime PrivateTmp flip.

H6 / PrivateTmp design (docs + preflight only):

> EmpiierOS write for Seat Shield H6/PrivateTmp design lock only (docs + preflight). No bake. No reinstall.

S5A (other workspace, unchanged):

> Unlock bay S5A per `docs/apps/Monitoring/TZ-SEAT-SHIELD-S5.md`. Do not write EmpiierOS.
