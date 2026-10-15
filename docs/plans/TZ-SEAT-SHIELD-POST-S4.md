# TZ — Seat Shield after S4 (paste · lab · bake · S5)

**Status:** Binding work order. **Not** acceptance. **Not** `READY` / `PASSED` / `DEMO_PASSED`.
**Date:** 2026-09-20
**Product:** EmpiierOS Seat Shield (`seat.shield.*`)
**Honest label today:** seat IDS + policy-gated peer block + on-demand tmp/shm hold (S5A) + S5B watch + S5C helper hash (crate 0.8.0) on glass **and** in official `out/`. Still **not** consumer on-access AV, not certified antivirus.
**Bay verdict (S4):** `READY_WITH_DEBT` (bay logic closed first).
**OS crate today:** `runtime/seat_shield_core` **0.8.0** — S5C journal `sha256` + S5B drain + S5A hold + S4 auto-observe / twin `disabled`.
**Lab today:** S5C mux 2026-09-20 16:18 +10 Runtime `d1360890…` watch `8525c72b…` Shell `index-BBUZrLR_.js` GUI `3b8992b1…` fw `1a3856a2…`. Unit `dd6bb219…` reused. `/tmp` journal carried helper hash `82f08bc5…`; Runtime `scan_path /tmp` still denied (`PrivateTmp=true`). Status `backend=helper` `act_enabled=false` `watch_pending=0`. Not MATCH.
**Official `out/`:** 2026-09-20 LUKS floor `rootfs.ext2` `e48aab38…` stamp `5979e9cf…` overlay `670612ea…` Runtime `79e06a7d…` GUI `f2fe1900…` Shell `index-DMc2IDZV.js`. `image_contract_ok`. Live USB mux SoT `f56dba1b…` + apply plant `8a9ec933…`. Prior host bake `4104a67b…` / stamp `d250a976…` / S5C `82ac20ac…` superseded. QEMU not rerun. Not MATCH.
**Twin:** `security.integrity.*` stays **`disabled`** (ledger). This TZ does **not** enable it.

Operator intent (do not lose): take the finished bay antivirus juice into the OS without mash, then grow a **seat-shaped** file-containment slice (S5). No lab reinstall. Official `out/` stays behind until a kinship bake unlock.

If a later file conflicts with this TZ on post-S4 scope, **this TZ wins**. Older “S4 = quarantine AV” is **S5**. S4 itself is closed in the bay.

Bay-only S5 execution SoT (no EmpiierOS writes):
`../EmpiierOS_spehsial_app/docs/apps/Monitoring/TZ-SEAT-SHIELD-S5.md`.

Hardening (compiler / systemd seccomp / fuzz — not a second IDS):
[`TZ-SEAT-SHIELD-HARDENING.md`](TZ-SEAT-SHIELD-HARDENING.md).

---

## 0. Read first (this order)

1. This file (SoT for post-S4).
2. Bay closeout: `EmpiierOS_spehsial_app/docs/apps/Monitoring/READY.md`, `HANDOFF.md`, `TZ-SEAT-SHIELD-S4.md` (historical, done).
3. Bay contract (newer): `EmpiierOS_spehsial_app/docs/contracts/SEAT-SHIELD.md`.
4. Living OS: `session/SEAT-SHIELD.md`, `session/SEAT-SHIELD-FW.md`, `runtime/HEALTH-IPC.md`, `runtime/src/seat_shield.rs`, `runtime/seat_shield_core/`.
5. Shell: `shell-ui/src/secure/components/SeatShieldCard.tsx`, `shell-ui/src/session/seatShield.ts`.
6. Floor: `runtime/helpers/seat-shield/`.
7. Bake debt: `EmpiierOS_spehsial_app/paste/seat-shield-s3-bake/COMMIT-PACKAGE.md`.
8. Oracle (read-only): `EmpiierOS_spehsial_app/Monitoring/ewlshield/security/malware.py`, `collector.py`, `tests/test_detection_false_positive_guards.py`.
9. Ledger: `docs/plans/MATURITY-LEDGER.md` row **Security integrity / dm-verity = disabled**.
10. Isolation: `.cursor/rules/os-north-star.mdc`, `verified-proposal.mdc`, `trash-not-rm.mdc`.

---

## 1. What this product is (do not drift)

```text
OS Shell (React, display-only)
  + EliteEW Runtime (Rust, Unix NDJSON, SO_PEERCRED)
  + Linux floor (pinned helper + nft sets + later image evidence)
```

Seat Shield defends the **EliteEW seat**: FoFline / Capsule / PIN / SSH break-glass.
It is **not** a generic PC antivirus and must never claim that.

| Law | Implication |
|-----|-------------|
| Triad | Capability = shell surface + `seat.shield.*` + floor helper/policy. Tests alone are not metal. |
| Anti-skin | Shell / Flask / JS never enforce nft or quarantine. |
| Anti-GNOME | No ClamGUI, osquery, AIDE, Falco, GeoIP-country core, weekly CDN defs. |
| Fail-closed | Unknown act → error. Missing helper → `backend=unavailable`, acts `not_available`. |
| Honesty | No “protected / MATCH / антивирус готов” from unit tests or bay netns. |
| Twin | `security.integrity.*` owns verity/lockdown. Do not fork. Do not green it here. |
| Isolation | Each wave has one write-home. Do not dual-edit OS and bay in one agent turn. |

`netns_metal=match` in bay evidence is **prep helper proof**, not image MATCH.

---

## 2. Fact check (S4 vs EmpiierOS) — Cause

S4 closed the hollow shield **in the bay only**:

| Gap S4 named | Bay 0.5.0 (closed) | EmpiierOS 0.4.0 (living) |
|--------------|--------------------|--------------------------|
| Live exe SHA-256 | `collect_with_hash` — suspicious path **or** feed has hashes; skip > 32 MiB; cap 32 files | `exe_sha256` always `None` in `observe/proc.rs` |
| Loop | `maybe_auto_observe` + harness `--auto-cycle` | Request-scoped `dispatch`; HUD ticks **status** every 12s; Observe is a **button** |
| Auto-act | `BlockPeer` only when `ENABLE_ACTS` + `auto_act_protected` + live helper; cap `max_auto_acts_per_cycle` | Cycle **never** mutates; `applied_acts` absent |
| Ports | Default `[22]` only | Same default (keep) |
| Behavioral | `network_suspicious_port`, `possible_port_scan` — events, never acts | Absent |
| Kernel taint | Event if `/proc/sys/kernel/tainted` notable and ≠ `4096`; ≤1 / 300s | Absent |
| Paste | `paste/seat-shield-s4/` says **not applied** | No S4 hunks |

**Cause the OS still has:** hash-IOC is dead; protected-port probes only propose; no autonomous cycle on the seat unless someone clicks Observe.

**Additional OS cause (must lock in paste):** Runtime opens a **new** `ShieldService` per IPC line (`runtime/src/seat_shield.rs` `dispatch`). Interval state must live on disk (`cycle_state.json.last_cycle_epoch`). `maybe_auto_observe` is useless in the OS until **some production path** calls it. The harness loop is not that path.

Monitoring oracle honesty for S5:

- `malware.py` hashes `/proc/<pid>/exe` and **recommends** quarantine in text.
- There is **no** quarantine directory, **no** on-access hook, **no** fanotify, **no** Clam.
- S5 is **new seat-shaped file containment**, not a port of a missing engine.

---

## 3. Waves (one unlock each — do not shotgun)

| Wave | Write-home | Unlock phrase (operator must say it) | Outcome |
|------|------------|--------------------------------------|---------|
| **P — S4 paste** | this repo | Unlock EmpiierOS write for Seat Shield S4 paste per `TZ-SEAT-SHIELD-POST-S4.md` wave P | **in source 2026-09-20** — crate 0.5.0 + contracts + Shell knobs + status-driven auto-observe. Not lab mux. |
| **L — lab mux** | lab `192.168.8.10` via `scripts/metal-live.sh` | Unlock lab `deploy-runtime` (and `deploy-shell` if card changed) for Seat Shield S4. No reinstall. No official `out/` | **muxed 2026-09-20** Runtime `a84e45eb…` / GUI `3b8992b1…` / fw `732e3b9e…` (text `nft list`). Status `ok` `backend=helper`. Auto-act off. Not MATCH |
| **B — bake** | Trusted Substrate kinship + EmpiierOS packager | Unlock Trusted Substrate kinship write + image bake per `paste/seat-shield-s3-bake/COMMIT-PACKAGE.md` | **rebaked H1 2026-09-20** overlay `daa1a5f9…` Runtime `04358653…` fw `1a3856a2…` `rootfs.ext2` `c9805fc6…`. QEMU: nftables **Finished**; smoke **FAIL** (WebKit string). Prior `c7aea2e4…`/`067ef5a2…` superseded. Not metal MATCH |
| **S5A — quarantine core** | `../EmpiierOS_spehsial_app` | Unlock bay S5A per `TZ-SEAT-SHIELD-S5.md` | Bay closed 0.6.0. EmpiierOS writes are a later paste. |
| **S5A-OS — paste + mux + bake** | this repo then lab then kinship | паста S5A в EmpiierOS по POST-S4, без S5B. После неё — mux на 192.168.8.10 (без переустановки), потом bake. | **done 2026-09-20** — crate 0.6.0; lab Runtime `f229c221…` Shell `index-DSbLUOHj.js`; official `dc8ebae8…` / overlay `451aea7a…`. No S5B. Auto-act / auto-quarantine off. Not MATCH |
| **S5B — watch helper** | bay first, then later OS paste | go S5B (inotify watch) | **bay measured 2026-09-20** — crate 0.7.0, 59+2 tests, preflight 0 evidence `20260919T213944Z`. |
| **S5B-OS — paste** | this repo | unlock на пасту S5B в EmpiierOS | **in source 2026-09-20** — crate 0.7.0 + Shell `watch_pending` + helper unit stub. |
| **S5B-OS — mux + bake** | lab then kinship | go Дальше | **bake 2026-09-20** overlay `d1fbef23…` `out/` `00e14f74…`. **mux 16:13 +10** Runtime `0a1f1d9f…` watch `2143eea5…` Shell `index-BBUZrLR_.js`. Auto-act off. Not MATCH |
| **S5C — helper hash** | this repo then lab | стекло доступно. закидывай и продолжаем со следующей wave | **mux 16:18 +10** crate 0.8.0 Runtime `d1360890…` watch `8525c72b…`. `/tmp` journal hash `82f08bc5…`. |
| **S5C — bake** | kinship | Продолжаем | **baked 16:26 +10** `out/` `82ac20ac…` overlay `e0a3fb8a…` watch `c40d6d74…`. Lab not remuxed. Auto-act off. Not MATCH |
| **I — integrity twin** | EmpiierOS only if a later integrity wave exists | **Forbidden in this TZ** | Keep `disabled`. Shield shows the twin as unavailable |

Do **not** start L without P green.
Do **not** start B to “get S4 on glass” — L is the lab path.
Do **not** start S5B inside an S5A-OS paste session.
Do **not** enable `security.integrity.*` or `product.open` from any wave here.

---

## 4. Wave P — paste S4 into EmpiierOS

### 4.1 In scope

1. Vendor bay `seat_shield_core` **0.5.0** over `runtime/seat_shield_core` **0.4.0**.
2. New file `observe/taint.rs` (and `lib.rs` / `observe/mod.rs` exports).
3. Same-phase contracts: `session/SEAT-SHIELD.md`, `runtime/HEALTH-IPC.md`, this plan’s companion wave notes, `session/SEAT-FACE.md` shield row if it still says “cycle never mutates”.
4. Shell: map `auto_act_protected`, `cycle_interval_secs`, `applied_acts` (if observe response is shown), event kinds `network_suspicious_port` / `possible_port_scan` / `kernel_taint` / `act_applied` reason `auto_protected`. Display only.
5. **OS lock that S4 bay could not do:** `seat.shield.status` calls `maybe_auto_observe()` after opening the service (interval from disk). HUD 12s status tick then becomes the autonomous loop. Observe button still forces a cycle.
6. Runtime tests: keep existing five `seat_shield.rs` tests; add (a) status does not apply acts when disarmed, (b) armed + `auto_act_protected` + state helper auto-blocks `198.51.100.20:22`, (c) status twice inside TTL does not double-apply.
7. Crate tests: `cargo test --manifest-path runtime/seat_shield_core/Cargo.toml` (expect the bay 42).
8. `scripts/preflight-foundation.sh` and `scripts/bootstrap-phase-smoke.sh` stay exit 0. Do not weaken gates.
9. Diary line in `SECURE-RESET-LIVE-DIARY.md` after paste (source only). Lab claims wait for wave L.

### 4.2 Out of scope (refuse)

- `Monitoring/` Python, Flask, venv, GeoIP, AIDE, Clam, YARA.
- New systemd security daemon.
- Seeding VNC `5900/5901`, RDP `3389`, WinRM as defaults.
- Domain IOC / DNS from the shield.
- S5 quarantine ops.
- `--build-image`, official `out/`, lab reinstall.
- Enabling `security.integrity.*`.
- `git commit` / `git push` unless the operator asks in the same turn.

### 4.3 How to copy (do not shotgun)

Bay tree and OS tree are **siblings**, not a git submodule.

1. Diff file-by-file. OS-only comments in `runtime/src/seat_shield.rs` stay.
2. Replace crate sources with bay 0.5.0 **except** do not delete OS `Cargo.toml` workspace membership (`runtime/Cargo.toml` path `seat_shield_core`). Bump crate version to `0.5.0`.
3. Keep `unsafe_code = "forbid"`. No `shell=True`. Helper protocol unchanged (`SEAT-SHIELD-FW.md`).
4. If a hunk conflicts with an EmpiierOS-only test helper (`EMPIIEROS_SEAT_SHIELD_FW`), keep the OS test harness and adapt the crate, not the other way around.

### 4.4 Architecture lock after paste

```text
HUD 12s status  ──► seat.shield.status
                         │
                         ├─ maybe_auto_observe()
                         │     if last_cycle_epoch + cycle_interval_secs elapsed
                         │     then same path as seat.shield.observe
                         │
                         └─ redacted status JSON (includes auto_act_protected)

observe button / card Observe ──► seat.shield.observe (always)

run_cycle_with_feed
  events always persist
  acts: PreviewBlock | BlockPeer
  apply BlockPeer iff act_enabled && auto_act_protected && helper available
  never apply from behavioral / hash / tmp-exec / kernel_taint

HelperActuator → /usr/libexec/empiieros-seat-shield-fw
  argv only, cleared env
  nft inet seat_shield blocked4/blocked6
```

Invariants (must stay true):

1. Allowlisted IP → no act, no auto-act.
2. Loopback / unspecified → ignore.
3. Private/LAN auto-act only if `allow_private_on_protected`.
4. Defaults: `act_enabled=false`, `auto_act_protected=false` — production stays observe-only until the operator types `ENABLE_ACTS` and sets auto-act.
5. Feed body never on Shell-facing IPC.
6. `security.integrity` badge stays the twin **name**; status must not claim the twin is live. Prefer an explicit `integrity_twin_state: "disabled"` if the field is added (honest). Do not invent verity numbers.

### 4.5 Contract delta (same phase)

`session/SEAT-SHIELD.md` must match the **bay** contract on:

- cycle may mutate floor **only** for armed protected-service auto-act;
- policy knobs `auto_act_protected`, `max_auto_acts_per_cycle` (1..=32), `cycle_interval_secs` (10..=300), `scanner_port_threshold`, `scanner_skip_ip_prefixes`;
- event kinds listed in bay §6;
- `policy.set`: `auto_act_protected:true` only when `act_enabled` is already true or is set true in the same call **with** `ENABLE_ACTS`;
- `cycle`/`observe` `data`: keep `proposed_acts`, add `applied_acts`, honest `note`.

`runtime/HEALTH-IPC.md` rows for `seat.shield.policy` / `cycle` / `observe` / `status` must drop “no host mutate” as an absolute.

S2 MVP wave is in `trash/`. Do not relabel S2 as MATCH.

### 4.6 Shell delta

| Field | UI |
|-------|----|
| `auto_act_protected` | Read-only badge + optional confirm field to set it **together with** `ENABLE_ACTS` (do not add a silent toggle) |
| New event kinds | Locale en+ru labels. No auto-block copy |
| `applied_acts` | Show count on last observe if the card already shows observe results |
| Integrity | Keep “twin: security.integrity”. If status grows `integrity_twin_state`, show `disabled` — never a green shield-from-verity |

Do not add a Start-menu “Antivirus” tile.

### 4.7 Measure (wave P)

All exit **0**. Do not weaken tests.

```bash
cargo test --manifest-path runtime/seat_shield_core/Cargo.toml
cargo test --manifest-path runtime/Cargo.toml --lib seat_shield
# plus any new status auto-observe tests
scripts/preflight-foundation.sh
scripts/bootstrap-phase-smoke.sh
```

Adversarial (must FAIL the bad path, then restore):

- `auto_act_protected=true` without `ENABLE_ACTS` → rejected.
- Disarmed observe of `198.51.100.20:22` → `applied_acts` empty.
- Default `protected_ports == [22]` (not 3389/5900).

### 4.8 Work order P

| Step | Work | Stop if |
|------|------|---------|
| P0 | Operator unlock phrase present in **this** turn | No unlock — write nothing |
| P1 | Diff bay 0.5.0 vs OS 0.4.0; list files | Blind copy of `Monitoring/` |
| P2 | Replace crate + bump 0.5.0 | `unsafe` or helper protocol change |
| P3 | Wire `maybe_auto_observe` on `status` | New systemd unit |
| P4 | Contracts + HEALTH-IPC + FACE | Docs claim MATCH |
| P5 | Shell map + locale | Silent auto-act toggle |
| P6 | Tests §4.7 | Weakening a gate |
| P7 | Diary source line. Stop. Wait for L unlock | Lab mux without ask |

---

## 5. Wave L — lab mux (not a reinstall)

### 5.1 In scope

- Cross-build Runtime (same packager CC / `cargo` target the living lab expects).
- `EMPIIEROS_METAL_HOSTKEY_SHA256=SHA256:msPNx+ysC+7DZLmBWoDsa2SQpUS4p0QxBsTgGX8TTuU`
  `scripts/metal-live.sh deploy-runtime BINARY UNIT`
- If Shell card changed: `deploy-shell GUI ASSETS` (include `product-chrome/`).
- Probe: `sha256sum` matches, `pgrep` Runtime, `seat.shield.status` via the frontend socket path the lab already uses.
- Glass: operator clicks HUD Shield → Observe. Syslog / events show new kinds if traffic exists.
- Auto-act stays **off** unless the operator types `ENABLE_ACTS` on glass.

### 5.2 Out of scope

- Reinstall `192.168.8.10`.
- Official `out/`.
- Changing SSH policy / `~/.ssh/known_hosts`. Use metal-live dedicated known_hosts only.
- Arming auto-act from SSH as a silent default.

### 5.3 Measure

Record in `SECURE-RESET-LIVE-DIARY.md`:

- transaction id, Runtime sha256, PID;
- `seat.shield.status` JSON: `auto_act_protected`, `act_enabled`, `protected_ports`;
- one Observe (button) response `note`;
- helper present or `backend=unavailable` (honest).

Not MATCH. Not `verified-real` unless the operator walks glass and you record what they saw.

---

## 6. Wave B — Substrate bake

Depends on S3 floor artifacts **already in the packager** plus the **P** Runtime.

Follow `EmpiierOS_spehsial_app/paste/seat-shield-s3-bake/COMMIT-PACKAGE.md`.

### 6.1 In scope (after kinship unlock)

1. `scripts/package-shell-gui-rootfs.sh --build-image` (overlay + `--resume`).
2. Optional post-build snippet: fw + nft + wants unit present.
3. Image contract + hashes in `FOUNDATION-STATUS.md` / ledger — **honest**, no metal MATCH from QEMU alone.
4. QEMU/debugfs: `/usr/libexec/empiieros-seat-shield-fw`, `seat_shield.nft`, wants unit.
5. Boot (image or later metal): `nft list table inet seat_shield` has `blocked4`/`blocked6`.

### 6.2 Out of scope

- Greening integrity.
- Claiming READY from bay `netns_metal=match`.
- Rewriting living Tauri.
- Reinstalling the current SSH lab to “prove” the image.

---

## 7. Wave S5A — seat file containment (bay first)

**Write-home:** `../EmpiierOS_spehsial_app` only.
**Binding bay file:** `docs/apps/Monitoring/TZ-SEAT-SHIELD-S5.md` (must stay aligned with this section).
**Do not write EmpiierOS.**

### 7.1 Why S5 exists (cause)

S4 can **see** a tmp exe and a hash IOC. It cannot **hold** the file.
Monitoring’s “quarantine” is a sentence in `malware.py`, not a mechanism.
If we paste S4 and stop, the product is still an IDS with a peer block. That is honest. S5 is the first file act.

### 7.2 Product shape (do not build Clam)

| Do | Do not |
|----|--------|
| Scan **allowlisted roots only** | Walk `/`, `/usr`, `/home`, `/opt/empiieros/shell` |
| Hash + sealed feed + suspicious prefix | YARA/Clam signatures zoo |
| Quarantine = rename into Runtime state | `rm` the sample; wipe user packs |
| Confirm `QUARANTINE_FILE` / `RESTORE_QUARANTINE` | Silent delete |
| Auto-quarantine **only** `ioc_hash_match` when armed + `ENABLE_QUARANTINE` | Auto-quarantine from tmp-exec or behavioral |
| Cap 16 files / 32 MiB / no symlink escape | fanotify on the whole seat in S5A |
| Events `file_scan_hit`, `file_quarantined`, `file_restored`, `file_scan_denied` | Domain IOC, GeoIP, AIDE |

Allowlisted scan roots (S5A):

```text
/tmp/
/var/tmp/
/dev/shm/
```

Optional later (not S5A): FoFline extract dir **if** a session contract names an absolute path under Runtime state. Do not guess.

Quarantine layout:

```text
{state_dir}/seat_shield/quarantine/          # 0o700
  ├── <sha256>                               # 0o600 regular file (the bytes)
  └── <sha256>.json                          # 0o600 sidecar: orig_path, epoch, reason, size
```

Factory reset already must purge `{state}/seat_shield/` — keep that invariant.
Restore writes back **only** if `orig_path` is still under an allowlisted root and does not escape via `..`.

### 7.3 Ops (bay protocol first; OS paste is a later unlock)

| Op | Class | Intent |
|----|-------|--------|
| `seat.shield.scan_path` | `ops_only` | Params: `path` absolute, must be under allowlisted root. Hash if regular file ≤ 32 MiB. Compare sealed feed. Persist events. No mutate. |
| `seat.shield.quarantine` | `ops_only` + `QUARANTINE_FILE` | Move file → quarantine. Fail-closed if not regular, too big, symlink, or outside roots. |
| `seat.shield.quarantine.list` | observe | Redacted: sha256, size, epoch, reason, **basename** of orig_path — not a dump of `/tmp` trees |
| `seat.shield.quarantine.restore` | `ops_only` + `RESTORE_QUARANTINE` | Restore one sha256 if dest still legal |
| `seat.shield.policy.set` | existing | New knobs: `auto_quarantine_hash` default **false**; requires `act_enabled` + `ENABLE_QUARANTINE` in the same rules as auto-act |

`seat.shield.observe` / cycle may call scan **only** on `exe_path` already collected (the live connection set), never a directory walk.
If `auto_quarantine_hash` and armed and helper/state available: quarantine **that one file** after `ioc_hash_match`. Cap 1 per cycle.

### 7.4 Tests (S5A)

Keep all S4 gold. Add:

| Test | Assert |
|------|--------|
| scan tmp file with feed hash | `file_scan_hit` + `ioc_hash_match` |
| scan `/etc/passwd` | denied, no hash stored |
| quarantine tmp hit | file gone from tmp, present in quarantine dir |
| restore | bytes match, dest under tmp |
| symlink escape | denied |
| > 32 MiB | skip, no panic |
| auto-quarantine off | observe hash match does not move file |
| auto-quarantine on + ENABLE_QUARANTINE | hash match moves file; tmp-exec alone does not |
| behavioral still never quarantines | |

Commands:

```bash
cargo test --manifest-path seat_shield_core/Cargo.toml
cargo test --manifest-path seat_shield_core/Cargo.toml --bins
./scripts/seat-shield-bay-preflight.sh
```

All exit 0. Do not weaken preflight needles.

### 7.5 Mash

`Monitoring/` stays the frozen oracle. Do not add features there.
No Flask UI for quarantine. `shell_preview/` may list quarantine count.

Paste stub `paste/seat-shield-s5/` — **not applied**, no EmpiierOS write script.

Verdict after S5A bay: still `READY_WITH_DEBT`. Never `READY`.

---

## 8. Wave S5B — on-access (bay measured 2026-09-20)

**Cause:** S5A only sees files that already have a live TCP inode or an operator path. A dropper that does not connect is invisible.

**Lock (bay 0.7.0, not OS):**

- Pinned helper `seat-shield-watch` (OS paste name `empiieros-seat-shield-watch`). Rust, **not** Python.
- Non-recursive `inotify` on **only** `/tmp`, `/var/tmp`, `/dev/shm`.
- Writes `{state}/seat_shield/watch.jsonl` (`0o600`, cap 200).
- Runtime `observe` / `status` / `cycle` **drains** that file. Helper never calls nft and never quarantines.
- Auto-quarantine still only hash+ENABLE_QUARANTINE (cap 1 / drain).
- No recursive watch of `/home` or pack trees.
- Image bake of the helper is a later kinship item.
- Watch helper must **not** share Runtime `PrivateTmp=true`.

**Bay measure:** `CARGO_TARGET_DIR=…/EmpiierOS_spehsial_app/seat_shield_core/target` crate tests 59+2, clippy `-D warnings`, `./scripts/seat-shield-bay-preflight.sh` exit 0, evidence `20260919T213944Z` `netns_metal=match`. Stub `paste/seat-shield-s5b/` **not applied**.

**S5B-OS paste (this unlock):** crate + contracts + Shell + unit stub. Do **not** mux or bake the watch helper from the paste session.

---

## 9. Integrity twin (honesty, not work)

Ledger: **Security integrity / dm-verity = `disabled`**.
Production prefix `security.integrity.*` returns `not_available`.

Seat Shield may:

- keep `integrity_twin: "security.integrity"`;
- add `integrity_twin_state: "disabled"` so the card cannot look greener than the ledger.

Seat Shield must not:

- spawn AIDE / tripwire / osquery;
- SHA `/etc/passwd` as SoT;
- implement `trigger_lockdown` / `release_lockdown`;
- treat `kernel_taint` as verity failure;
- claim the seat is “integrity protected”.

A future integrity wave is a **different TZ**.

---

## 10. Forbidden zoo (every wave)

- ClamAV, YARA rule packs, VirusTotal, weekly definition CDN.
- Flask / Socket.IO / TCP `:9999` / Fernet collector.
- `shell=True` nft/firewalld.
- GeoIP country risk as policy.
- Default VNC/RDP/WinRM protected ports.
- Domain IOC (DNS leak + stall).
- Dual privileged Python next to Runtime.
- `rm -rf` — use `scripts/trash-move.sh`.
- Surprise `git commit` / `git push`.
- Lab reinstall. Official `out/` without wave B unlock.
- Fake `MATCH` / `DEMO_PASSED` / “антивирус готов”.

---

## 11. Acceptance by wave

### P (EmpiierOS source)

1. Crate 0.5.0 in `runtime/seat_shield_core`.
2. §4.7 commands exit 0.
3. `status` invokes `maybe_auto_observe` (code + test).
4. Contracts no longer say “cycle never mutates” as an absolute.
5. No `Monitoring/` files in the OS tree.
6. Verdict stays `NOT_READY` / shield `READY_WITH_DEBT`.

### L (lab)

1. metal-live transaction applied; hashes match.
2. Diary line. Glass Observe is the operator’s.
3. Auto-act still default off.

### B (image)

1. `image_contract_ok` + fw/nft/wants in the image.
2. Ledger updated with hashes. No metal MATCH from QEMU alone.

### S5A (bay)

1. §7.4 tests exit 0.
2. `paste/seat-shield-s5/` exists and says not applied.
3. EmpiierOS git tree untouched by that agent.

### S5A-OS (this repo + lab + official `out/`)

1. Crate 0.6.0; crate tests include §7.4 gold.
2. `status` still calls `maybe_auto_observe`; `integrity_twin_state=disabled`.
3. Lab mux without reinstall; `scan_path /etc/passwd` denied; auto-quarantine default off.
4. Official `image_contract_ok`. QEMU optional; do not weaken WebKit string.
5. S5B was not in that crate. Verdict stays `NOT_READY`.

### S5B (bay)

1. Crate 0.7.0; helper CLI rejects nft verbs.
2. `/etc/passwd` cannot enter `watch.jsonl` via append; planted line drains as `file_scan_denied`.
3. Clean tmp create does not quarantine; hash+ENABLE_QUARANTINE can.
4. Bay preflight exit 0. Not MATCH.

### S5B-OS (this repo + official `out/`)

1. Crate 0.7.0; `status` drains then `maybe_auto_observe`; `integrity_twin_state=disabled`.
2. Packager ships `seat-shield-watch`; unit `PrivateTmp=false`; post-build wants symlink.
3. Official `image_contract_ok` `00e14f74…`. debugfs watch helper present.
4. Lab mux 2026-09-20 16:13 +10. `/dev/shm` drain. Auto-act off. Not MATCH.

### S5C (this repo + lab)

1. Crate 0.8.0; journal may carry `sha256`; planted `/etc` digest ignored.
2. Live `/tmp` journal hash present; Runtime `scan_path /tmp` still denied (`PrivateTmp`).
3. Auto-hold still needs `ENABLE_QUARANTINE` + a path Runtime can open.
4. Official `image_contract_ok` `82ac20ac…`. QEMU not rerun. Not MATCH.

---

## 12. What “done” is not

- Not metal-ready. Official `out/` is the 2026-09-20 bake; QEMU smoke is red.
- Not “антивирус готов”.
- Not permission to reinstall the lab.
- Not permission to enable `security.integrity.*`.

Stop at the wave the operator unlocked. Report Cause / Lock / Measure, commands + exit codes, leftover debt. Wait.

---

## 13. Suggested operator phrases

Paste (this repo):

> Unlock EmpiierOS write for Seat Shield S4 paste per `docs/plans/TZ-SEAT-SHIELD-POST-S4.md` wave P.

Lab:

> Unlock lab deploy-runtime (and deploy-shell if needed) for Seat Shield S4. No reinstall. No official out/.

Bake:

> Unlock Trusted Substrate kinship write + image bake per `paste/seat-shield-s3-bake/COMMIT-PACKAGE.md`.

S5A (other workspace):

> Unlock bay S5A per `docs/apps/Monitoring/TZ-SEAT-SHIELD-S5.md`. Do not write EmpiierOS.
