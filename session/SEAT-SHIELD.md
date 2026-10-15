# EmpiierOS Seat Shield Contract

**Status:** Living EmpiierOS session contract. S2/S3 applied 2026-09-18. **S4 paste** 2026-09-20 (wave P). **S5A paste** 2026-09-20 (on-demand tmp/shm hold). **S5B paste** 2026-09-20 (bounded inotify journal + drain). Watch helper **baked** (S5C `out/`) and **muxed** (live unstripped).
**Paste source:** EmpiierOS_spehsial_app `seat_shield_core` 0.7.0 / `paste/seat-shield-s5b/` plus OS-only S5C 0.8.0 helper hash.
**Role:** Session plane contract for EliteEW **seat** threat observe/decide and allowlisted Runtime acts.
**Logic core:** `runtime/seat_shield_core/` (crate 0.8.0).
**Twin:** `security.integrity.*` stays ledger `disabled`. Do not fork. Do not green here.
**Parent:** [`CONTRACT.md`](CONTRACT.md)
**Plan:** [`../docs/plans/TZ-SEAT-SHIELD-POST-S4.md`](../docs/plans/TZ-SEAT-SHIELD-POST-S4.md) · [`../docs/plans/TZ-SEAT-SHIELD-HARDENING.md`](../docs/plans/TZ-SEAT-SHIELD-HARDENING.md) · [`../docs/plans/TZ-SEAT-SHIELD-SPINE.md`](../docs/plans/TZ-SEAT-SHIELD-SPINE.md)

**Not:** EWLShield Flask/collector. **Not:** consumer antivirus. **Not:** production maturity / `PASSED` / metal MATCH.
**Transport:** Unix NDJSON `SO_PEERCRED` per [`../runtime/HEALTH-IPC.md`](../runtime/HEALTH-IPC.md).
**Shell surface:** first-party host workspace + HUD flyout, same plane as FoFline/Launcher. Not a FoFline catalog app.

---

## 1. Purpose

First-party Seat Shield control plane: observe hostile peers and suspicious local process signals against the live EliteEW seat (FoFline / Capsule / PIN / SSH break-glass), emit bounded redacted events, and apply **allowlisted** containment through the pinned floor helper.

Composes with Security Integrity (P84). Must not reimplement verity/lockdown or weaken Capsule / Air / PIN / `product.open`.

---

## 2. Invariants

1. **Singular control plane:** `seat.shield.*` + OS Shell display. No privileged Python collector, Flask, or unconfined nft shell strings.
2. **Twin-plane:** verity / lockdown stay `security.integrity.*`. Status may report `integrity_twin_state: "disabled"`. Never invent verity numbers.
3. **Default armed (product):** `act_enabled: true`, `auto_act_protected: true`, `auto_quarantine_hash: true`, `heuristics_enabled: true`. Seats that still have **sparse** pre-S4 factory JSON (no `auto_act_protected` / `heuristics_enabled` keys) are migrated once on `read_policy` to the armed default. A full explicit disarm (all S4 keys written) is not overwritten. Cycle mutates the floor **only** for protected-service `BlockPeer` when armed (`act_enabled` + `auto_act_protected` + helper available). Preview, behavioral, IOC-hash, tmp-exec, and `kernel_taint` never mutate the floor. Auto-hold of a tmp/shm file happens **only** after `ioc_hash_match` when armed + `ENABLE_QUARANTINE` (cap 1 / cycle).
4. **Allowlist-first:** allowlisted / loopback / unspecified peers are never acted.
5. **Protected ports:** default TCP `[22]` only. VNC/RDP/WinRM are not defaults. Expand by contract, not Shell blobs.
6. **Private peers:** auto-act only if `allow_private_on_protected` and the local port is protected.
7. **Named ops only.**
8. **Shell is display.** No feed bodies, no nft enforcement.
9. **Confirms:** `ENABLE_ACTS`, `ENABLE_QUARANTINE`, `QUARANTINE_FILE`, `RESTORE_QUARANTINE`, `BLOCK_PEER`, `UNBLOCK_PEER`, `RESET_SEAT_SHIELD`. `auto_act_protected: true` only when `act_enabled` is already true or is set true in the same call **with** `ENABLE_ACTS`. `auto_quarantine_hash: true` requires `act_enabled` + `ENABLE_QUARANTINE`.
10. **Audit** under `{state}/seat_shield/` (`0o600`).
11. **Factory reset** purges `{state}/seat_shield/`.
12. **No IDS zoo:** no AIDE, tripwire, osquery, falco, Clam, GeoIP-country SoT.

---

## 3. Storage

```text
{state_dir}/seat_shield/
  ├── policy.json
  ├── cycle_state.json
  ├── events.json          # decision/status SoT (ring 200)
  ├── journal.jsonl        # durable browse copy (cap 1000 / 1 MiB)
  ├── peer_window.json     # heuristic memory (cap 32 peers, TTL 300s)
  ├── last_connections.json
  ├── audit.json
  ├── feed/ioc.json
  ├── watch.jsonl          # 0o600 S5B journal (cap 200); Runtime drains
  ├── ota_baseline.json    # 0o600 OTA P4 dest→sha256 after cutover (E4); absent = missing
  └── quarantine/          # 0o700; blobs + <sha256>.json 0o600
```

---

## 4. Non-Goals

- EWLShield UI / TCP `:9999` / Fedora setup scripts as SoT.
- Consumer on-access AV / fanotify of `/` / recursive `/home`. S5B is non-recursive inotify on `/tmp` `/var/tmp` `/dev/shm` only. Helper never nft/holds.
- Country/GeoIP policy.
- Enabling `security.integrity.*` or `product.open`.
- Dual privileged Python beside Runtime.
- AF_PACKET / eBPF collectors in this contract wave.

---

## 5. Operations

| Op | Class | Intent |
|----|-------|--------|
| `seat.shield.status` | observe | Redacted mode, policy flags, backends, last `connections[]`. **Drains** `watch.jsonl` then **calls `maybe_auto_observe`** when `cycle_interval_secs` elapsed (HUD 12s tick). |
| `seat.shield.events` | observe | Bounded page after cursor (`events.json` ring) |
| `seat.shield.journal` | observe | Browse `journal.jsonl` with filters / analytics / optional `group_by` |
| `seat.shield.policy` | observe | Ports, **`allowlist`**, allowlist count, act/auto/quarantine/heuristics knobs, interval |
| `seat.shield.policy.set` | mutation + confirm | Validated; `ENABLE_ACTS` to arm. `heuristics_enabled` needs **no** floor confirm. |
| `seat.shield.cycle` | observe | Snapshot `connections[]`; persist events + journal dual-write; mutate only for armed protected auto-act |
| `seat.shield.observe` | observe | Live `/proc/net/tcp{,6}` + bounded exe SHA-256 + same cycle; returns `connections[]` |
| `seat.shield.feed_status` | observe | Feed digest/version only |
| `seat.shield.feed.install` | mutation | Absolute path under state root only |
| `seat.shield.act` | mutation + confirm | Allowlisted acts; `not_available` without helper |
| `seat.shield.reset` | mutation + confirm | `RESET_SEAT_SHIELD` |
| `seat.shield.scan_path` | observe | One file under `/tmp/` `/var/tmp/` `/dev/shm/` (32 MiB, no symlink escape) |
| `seat.shield.quarantine` | mutation + confirm | `QUARANTINE_FILE` — hold into `quarantine/<sha256>` |
| `seat.shield.quarantine.list` | observe | basename + sha256 + size + reason (no orig path dump) |
| `seat.shield.quarantine.restore` | mutation + confirm | `RESTORE_QUARANTINE` if dest still legal |

### 5.1 Acts

| `params.action` | Requires | Notes |
|-----------------|----------|-------|
| `preview_block` | — | No floor mutate |
| `block_peer` | armed + helper + `BLOCK_PEER` | Allowlist/loopback fail-closed |
| `unblock_peer` | armed + helper + `UNBLOCK_PEER` | Same validation |

### 5.2 Status shape

```json
{
  "product": "empiieros-seat-shield",
  "mode": "observe_only",
  "act_enabled": false,
  "auto_act_protected": false,
  "auto_quarantine_hash": false,
  "heuristics_enabled": true,
  "quarantine_count": 0,
  "watch_pending": 0,
  "last_watch_drained": 0,
  "cycle_interval_secs": 30,
  "protected_ports": [22],
  "allowlist_count": 0,
  "integrity_twin": "security.integrity",
  "integrity_twin_state": "disabled",
  "last_cycle_epoch": 0,
  "open_high_events": 0,
  "backend": "helper",
  "blocked_count": 0,
  "blocked_peers": [],
  "connection_count": 0,
  "connections": [],
  "connections_truncated": false,
  "connections_epoch": 0
}
```

Missing helper still reports `backend: "unavailable"` and acts `not_available`.
This floor `nftables` has no JSON; helper `list` falls back to text `nft list set`.

`cycle` / `observe` `data`: `connections[]` (cap 128), `proposed_acts`, `applied_acts`, honest `note` (`preview-only; cycle did not mutate floor` / `protected auto-act applied` / `protected auto-act denied; floor unchanged`).

### 5.3 Journal and heuristics (P1–P2)

- `events.json` remains the decision/status SoT (ring 200).
- Every stamped append also dual-writes `journal.jsonl` (browse copy).
- `heuristics_enabled` (default on) gates observe-only real-time correlation inside the cycle. It does **not** change auto-act / auto-hold eligibility and needs no `ENABLE_ACTS`.
- **P2:** `peer_window.json` (cap 32, TTL 300s) accumulates flags → `risk_milli` → optional `chain_*` / `heuristic_reconnect_burst`. Disable clears the window. Status exposes `open_chains` + `open_chain_teasers` (≤5). Connections sort by `risk_milli` when heuristics are on.
- Chains are **observe-only** (no new nft/hold gates).
- Shell Journal can request `group_by=chain` and render `groups[]` (browse only).

### 5.4 Probe helper (P3)

- Optional `/usr/libexec/empiieros-seat-shield-probe dump` (inet_diag SOCK_DIAG only).
- Runtime prefers probe with TTL (~5s); timeout/fail → `/proc/net/tcp{,6}`.
- `ObservedConn.rtt_ms` from `rtt_us/1000`. RTT column always when present; `risk_milli` may add a small RTT bonus only if heuristics are on.
- **Not** eBPF. **Not** AF_PACKET. **Not** MATCH by itself.

---

## 6. Event kinds

| Kind | Typical | Meaning |
|------|---------|---------|
| `peer_probe_protected_service` | critical | Non-allowlisted peer hit protected port |
| `suspicious_exec_location` | high | Image under `/tmp/`, `/var/tmp/`, `/dev/shm/` |
| `ioc_peer_match` | high | Sealed feed IP |
| `ioc_hash_match` | critical | Sealed feed exe hash |
| `policy_suppressed` | low | Duplicate suppression |
| `backend_unavailable` | medium | Missing backend |
| `act_preview` | medium | Would block if armed |
| `act_applied` | high | Manual or `auto_protected` |
| `act_denied` | medium | Denied — not marked blocked |
| `network_suspicious_port` | medium | Remote port in `{4444,5555,6667,31337,54321,12345}` — event only |
| `possible_port_scan` | high | Distinct local ports from one IP ≥ `scanner_port_threshold` — event only |
| `kernel_taint` | high | `/proc/sys/kernel/tainted` notable and not `4096`; ≤1 / 300s on live observe |
| `file_scan_hit` | critical | On-demand scan SHA-256 matches sealed feed |
| `file_scan_denied` | low | Path outside roots or symlink escape. Vanished/unreadable tmp/shm (Runtime `PrivateTmp`, unlinked drop) is **not** this event |
| `file_quarantined` | high | File held under `seat_shield/quarantine/` |
| `file_restored` | medium | Operator restore to a still-legal dest |
| `file_watch_seen` | low | Optional; drain does **not** emit this on every clean tmp create |
| `ota_capsule_staged` | medium | Observe-only: seat staged a verified OTA capsule (E3c+; no floor mutate) |
| `ota_component_mismatch` | critical | Observe-only: P4 path digest ≠ `ota_baseline.json` (E4+; **not** auto quarantine of `/usr`) |

---

## 7. Policy defaults

| Knob | Default |
|------|---------|
| `act_enabled` | `true` |
| `auto_act_protected` | `true` (needs `act_enabled` + helper for floor acts) |
| `auto_quarantine_hash` | `true` (needs `ENABLE_QUARANTINE` + `act_enabled`) |
| `heuristics_enabled` | `true` (observe-only correlation; no extra floor gate) |
| `max_auto_acts_per_cycle` | `5` (`1..=32`) |
| `cycle_interval_secs` | `30` (`10..=300`) |
| `scanner_port_threshold` | `6` |
| `scanner_skip_ip_prefixes` | Monitoring oracle DNS/CDN prefixes (scanner events only) |
| `protected_ports` | `[22]` |
| `allow_private_on_protected` | `true` |
| `suppression_ttl_secs` | `120` |
| `allowlist` | empty |

Live exe SHA-256: only when the sealed feed has hashes **or** the path is `/tmp/`, `/var/tmp/`, `/dev/shm/`. Skip > 32 MiB. Cap 32 hashes per collect. IO errors → `null`.

Autonomous loop in this OS: `seat.shield.status` → drain `watch.jsonl` → `maybe_auto_observe`. Watch helper `empiieros-seat-shield-watch` is a later bake; unit stub must not use `PrivateTmp=true`.

---

## 8. Shell

One host-workspace card (tabs, Monitoring-style) + desktop tile.
Brand on glass: short **Защита места** / **Seat Shield**.

Tabs: **Overview · Events · Blocks · Files · Settings**.
Day-to-day copy stays plain; confirm tokens (`ENABLE_ACTS`, `BLOCK_PEER`, …)
live only on the tab that needs them. Show auto-block / auto-hold / held /
watch counts without jargon walls. Arm may set auto-act in the same
`ENABLE_ACTS` call. Auto-hold needs a separate `ENABLE_QUARANTINE`.
No silent toggle. Scan/hold/restore stay confirm-gated and tmp/shm-only.
Integrity badge may show `disabled`. Never a green verity claim.

**Tray notifications:** Shell polls `seat.shield.events` and enqueues
medium+ severities via Runtime `notification.post` (dedupe by event sequence in
session storage). Taskbar bell shows unread/critical counts from
`notification.list`; dismiss/mark-read uses `notification.dismiss`.

**Settings (2026-09-22):** Overview shows floor-missing / OTA baseline / integrity
disabled banners; chips include `ota_baseline` + twin state. Settings tab:
`feed_status` + `feed.install` (absolute path under state), allowlist add/remove
via `policy.set`, arm/disarm and auto knobs unchanged.

Logic plane is already Rust (`runtime/seat_shield_core`). Do **not** restore
EWLShield Flask/GeoIP/charts from `Monitoring/` — harvest the tab UX only.

---

## 9. Floor

Helper `/usr/libexec/empiieros-seat-shield-fw` (or `/usr/local/sbin/…`). Protocol: [`SEAT-SHIELD-FW.md`](SEAT-SHIELD-FW.md).
Watch helper `/usr/libexec/empiieros-seat-shield-watch` is in official `out/` (`3a93cc44…` on the LUKS bake) and was muxed on glass (`8525c72b…`, S5C hash-in-journal). Must not share Runtime `PrivateTmp`.
Missing fw helper → `backend=unavailable`, acts `not_available`. Overlay packager ships fw+nft+watch+units.
Current official SoT `5979e9cf…` (watch `3a93cc44…`, fw `c0d89634…`). Prior S5C `82ac20ac…` / S5B `00e14f74…` superseded in `out/`.
Lab S5C mux 2026-09-20 16:18 +10 Runtime `d1360890…` watch `8525c72b…` Shell `index-BBUZrLR_.js` GUI `3b8992b1…` fw `1a3856a2…`. Unit `dd6bb219…` reused. `/tmp` journal hash `82f08bc5…`; Runtime `scan_path /tmp` denied (`PrivateTmp`). Status `backend=helper` `act_enabled=false`. Not remuxed after bake. Not MATCH.

### 9.1 Runtime / watch namespaces (locked)

| Unit | `PrivateTmp` | Why |
|------|--------------|-----|
| `empiieros-runtime.service` | **`true`** | Privilege plane jail. Do not flip to make `scan_path /tmp` pass. |
| `empiieros-seat-shield-watch.service` | **`false`** | Must see host `/tmp` `/var/tmp` `/dev/shm`. No `CAP_NET_ADMIN`. |

Bridge: S5C journal may carry `sha256` so drain can IOC-match when Runtime cannot open the host path. Auto-hold still needs a path Runtime can open. Design SoT: [`../docs/plans/TZ-SEAT-SHIELD-H6-PRIVATETMP.md`](../docs/plans/TZ-SEAT-SHIELD-H6-PRIVATETMP.md).

### 9.2 H6 (systemd seccomp)

Unit carries `SystemCallFilter=~@debug`. That filter applies only when pid1 is built `+SECCOMP` (kinship `BR2_PACKAGE_LIBSECCOMP=y`; QEMU already measures `seccomp mode=2`). Live lab installs that still show systemd `-SECCOMP` / Runtime `Seccomp: 0` are **not** H6-closed. Mux does not close H6. Measure checklist is in the H6/PrivateTmp TZ — after a fresh install only.

### 9.3 OTA baseline (E3c+E4 planted)

Companion: [`OTA-CAPSULE.md`](OTA-CAPSULE.md) · [`../docs/plans/TZ-EMPIERO-OS-OTA-EDGE.md`](../docs/plans/TZ-EMPIERO-OS-OTA-EDGE.md).

1. **Not** `security.integrity.*`. Status keeps `integrity_twin_state: "disabled"`.
2. After cutover (E4): write `ota_baseline.json` (P4 absolute dest → sha256) from applied digests.
3. Observe/status reports `ota_baseline: missing|ok|mismatch|stale` by rematching **only** that path set (bound, skip >32 MiB).
4. Mismatch → `ota_component_mismatch` **observe-only** on cycle — no auto quarantine of `/usr/local` / `/opt/empiieros`.
5. Portal VIP `10.88.89.1` is never auto BlockPeer (cycle + auto-act eligibility).
6. Verified stage emits observe-only `ota_capsule_staged` (fail-soft).
   That event is **stage**, not cutover. E4 `launcher.ota_apply` is planted.
7. Do **not** put good OTA digests into IOC `feed/ioc.json`.

**Planted:** VIP exemption, status field, staged emit, baseline write on cutover, mismatch event on cycle.

---

## 10. Maturity

```text
shell surface  +  seat.shield.*  +  substrate/helper evidence
```

Crate tests are necessary and **not** metal MATCH. Lab mux is wave L. Bake is wave B.

Hardening H2–H5 are in source (H3 audit on hosts with `cargo-audit`;
H4 `parser_probe` + corpus test; H5 oneshot jail). **H6 design is locked**
(`TZ-SEAT-SHIELD-H6-PRIVATETMP.md`); **live H6 is not closed** (systemd
`-SECCOMP` until fresh install). Integrity twin stays disabled.
