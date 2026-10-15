# TZ — Capsule-gated OTA (OS components + FoFline)

**Status:** Binding work order. **P0–P4 + E4 cutover plant Measure 2026-09-21**.
**Not** MATCH. **Not** acceptance. No metal cutover claim. No official bake claim.
**Date:** 2026-09-21
**Living contract:** [`../../session/OTA-CAPSULE.md`](../../session/OTA-CAPSULE.md)
**Companion:** [`../../session/SPA-PACK.md`](../../session/SPA-PACK.md) (bootstrap pack ≠ OTA) ·
[`../../docs/foundation/RELEASE-SIGNING-STUBS.md`](../../docs/foundation/RELEASE-SIGNING-STUBS.md) ·
[`MATURITY-LEDGER.md`](MATURITY-LEDGER.md)
**Edge:** [`TZ-EMPIERO-OS-OTA-EDGE.md`](TZ-EMPIERO-OS-OTA-EDGE.md)

## Law

- **Component floor (1B).** Runtime + Shell GUI + allowlisted helpers only.
  **P93 A/B rootfs stays frozen.**
- **Capsule only.** No public-edge pull of update blobs. No Tauri updater SoT.
- **Hard gate before any update-capsule transfer** (all required):
  1. Activated license — `allow_fofline=true` + bound `current.ecap`
  2. Hardware bind — living `device_keys/recipient.sk.hex` + device proof
     class used by air (`device_request` / `fp_v2`); missing → refuse
  3. Active EliteEW login — `oidc_session=present`; logout clears OTA
  4. Secure portal — `wg-portal` up + host-map applied + VIP-only helper
- **Order:** attest → edge mints update capsule → seat verifies
  (`artifact` key role ≠ entitlement) → stage → typed consent → apply.
- **Production ops:** `launcher.ota_*` only. The secure-reset socket
  **disables** the entire `update.` prefix in `runtime/src/core.rs`.
  Do not re-enable blanket `update.*`. Do not treat historical
  `channel_package` / lab HTTP `update.apply` as production SoT.
- **Bootstrap ≠ OTA.** Journey `spa` → `launcher.spa_sync` stays pre-OIDC
  first pack. Glass must not label it «OTA».
- No private keys in git. No MATCH / metal / bake claims from this TZ alone.

## Ops (locked names)

| Op | Role | This wave |
|----|------|-----------|
| `launcher.ota_status` | Observe gate flags + apply chip | planted |
| `launcher.ota_request` | Attest + VIP fetch + verify + stage `{state}/ota-capsule/` | planted |
| `launcher.ota_apply` | Typed consent cutover (`os_component`) + baseline | **E4 planted** |

## Refuse codes (IPC flags only)

`allow_fofline_required` · `license_required` · `device_keys_required` ·
`device_keys_invalid` · `hardware_bind_required` · `oidc_session_required` ·
`portal_required` · `portal_hostmap_required` · `secure_transport_required` ·
`attest_denied` · `capsule_signature_required` · `artifact_pk_missing` ·
`artifact_role_mismatch` · `path_escape` · `ota_not_staged` ·
`os_before_payload` · `confirmation_required` · `ota_helper_required` ·
`no_capsule`

## Channels

| Channel | Payload | Gate |
|---------|---------|------|
| OS component | allowlisted binaries under `/usr/local/sbin/empiieros-*`, `/opt/empiieros/shell/`, Runtime unit binaries (exact list in contract) | full hard gate |
| FoFline update | subsequent pack/group capsules after first bootstrap | full hard gate |
| Bootstrap pack | first `launcher.spa_sync` | license + portal only (living today) — **not** OTA |

When both OS and FoFline update capsules are staged: **OS-before-payload**.

## Cutover (P4 + E4 plant — 2026-09-21)

`launcher.ota_apply` with typed `APPLY_OTA` runs the cutover for
`channel=os_component`. FoFline stays `os_before_payload`.

### Capsule relative → disk destination

Only these mappings are legal. Anything else is `path_escape`.

| Capsule path prefix | Destination |
|---------------------|-------------|
| `runtime/empiieros-runtime` | `/usr/local/bin/empiieros-runtime` |
| `runtime/empiieros-shell-gui` | `/usr/local/bin/empiieros-shell-gui` |
| `shell/` (files only) | `/opt/empiieros/shell/` |
| `helpers/empiieros-air-enroll` | `/usr/local/sbin/empiieros-air-enroll` |
| `helpers/empiieros-portal-up` | `/usr/local/sbin/empiieros-portal-up` |
| `helpers/empiieros-spa-fetch` | `/usr/local/sbin/empiieros-spa-fetch` |
| `helpers/empiieros-ota-fetch` | `/usr/local/sbin/empiieros-ota-fetch` |
| `helpers/empiieros-wg` | `/usr/local/sbin/empiieros-wg` |
| `helpers/empiieros-audio-volume` | `/usr/local/sbin/empiieros-audio-volume` |
| `helpers/empiieros-wifi-radio` | `/usr/local/sbin/empiieros-wifi-radio` |
| `helpers/empiieros-wifi-restore` | `/usr/local/sbin/empiieros-wifi-restore` |
| `helpers/empiieros-time-sync` | `/usr/local/sbin/empiieros-time-sync` |
| `helpers/seat-shield-fw` | `/usr/libexec/empiieros-seat-shield-fw` |
| `helpers/seat-shield-watch` | `/usr/libexec/empiieros-seat-shield-watch` |
| `helpers/seat-shield-probe` | `/usr/libexec/empiieros-seat-shield-probe` |

Forbidden destinations: kernel/initrd, ESP, LUKS headers, `/etc` except via
a future separate unit-plant wave, arbitrary `/usr`, `/home`, state secrets.

### Install procedure (planted — `runtime/src/ota_cutover.rs`)

1. Require gates + typed `APPLY_OTA` + `ota-capsule/current/manifest.json`.
2. Re-verify artifact signature; refuse non-`os_component` with
   `os_before_payload`.
3. For each mapped file: write `dest.new` (mode `0755` bins / `0644` shell
   assets), `fsync`, `rename` over `dest`. No in-place truncate.
4. Record digests in `{state}/ota-capsule/apply.json` (`0600`) and
   `{state}/seat_shield/ota_baseline.json` (`0600`) — flags only on IPC.
5. Restart **only** units whose **binaries** changed. `shell/` assets
   do not restart GUI (glass reload / next start picks them up).
   Restarting `empiieros-runtime.service` or `empiieros-shell-gui.service`
   before the IPC newline is `invalid runtime response framing` — those
   two are `systemd-run --on-active=3s systemctl try-restart` after
   the reply. Watch may `try-restart` immediately. Oneshot helpers
   (`*-fetch`, portal, air) are not restarted; next start loads the
   new binary.
6. Lab: `EMPIIEROS_OTA_CUTOVER_ROOT` remaps destinations (no host restart).

### OS-before-payload

If both `os_component` and `fofline_update` capsules are staged:

1. Apply **OS component** first (this cutover).
2. Only after Runtime/Shell restart observes healthy status may a
   `fofline_update` capsule promote pack content (separate channel;
   must not rewrite `/usr` or `/opt/empiieros/shell` via fofline channel).
3. Glass must surface the order; no silent FoFline-first.
4. Day-1: `fofline_update` apply → `os_before_payload`.

### Plant unlock

**E4 unlocked 2026-09-21** (operator). Local Measure under cutover root.
Metal apply on a living seat is operator-gated evidence — **Not MATCH**.

## Phases

| Phase | Outcome |
|-------|---------|
| **P0** | This file + `session/OTA-CAPSULE.md` + CONTRACT/SPA-PACK/SEAT-FACE/ledger honesty — **done 2026-09-21** |
| **P1a** | `launcher.ota_*` in `core.rs`; gate module; `system_probe` rows; refuse tests — **local Measure 2026-09-21** |
| **P1b** | `empiieros-ota-fetch` VIP helper skeleton; attest builder; edge stub `attest_path_unplanted` — **local Measure 2026-09-21** |
| **P2** | System Protection OTA section + typed consent + EN/RU + Lab honesty — **local Measure 2026-09-21** |
| **P3** | Staging + fixture verify; factory_reset wipe (+ `recipient.sk.hex` debt) — **local Measure 2026-09-21** |
| **P4** | Cutover allowlist + OS-before-payload locked — **done 2026-09-21** |
| **E4** | Cutover code plant + `ota_baseline.json` write — **local Measure 2026-09-21** |

## P1 Measure (local)

- `cargo test --manifest-path runtime/Cargo.toml launcher_ota::` — gate tests ok
- Production allowlist: `launcher.ota_status` / `ota_request` / `ota_apply` in `core.rs`
- Helper unit VIP-only
- `scripts/preflight-foundation.sh` exit 0

## P3 Measure (local)

- `cargo test --manifest-path runtime/Cargo.toml ota_capsule_store::` — fixture verify + path escape + wrong key
- `{state}/ota-capsule/{staging,current}` + `{state}/ota/artifact.pk.hex` (or `EMPIIEROS_OTA_ARTIFACT_PK`)
- `factory_reset.rs` inventory wipes `ota-capsule` and living `recipient.sk.hex` (production `factory_reset.*` still disabled)
- `scripts/preflight-foundation.sh` exit 0

## P4 / E4 Measure (local)

- Cutover destination table + restart allowlist + OS-before-payload in
  this file and `session/OTA-CAPSULE.md`
- `cargo test --manifest-path runtime/Cargo.toml ota_cutover::` — map refuse +
  lab-root apply writes baseline `ok` + fofline `os_before_payload`
- `write_ota_baseline` + status `ota_baseline` rematch
- `scripts/preflight-foundation.sh` exit 0
- **Not** MATCH / bake / USB plant / metal cutover claim

## Operator phrase

> Unlock EmpiierOS write for capsule-gated OTA cutover per
> `docs/plans/TZ-CAPSULE-OTA.md` and `TZ-EMPIERO-OS-OTA-EDGE.md` E4.
> Ops are `launcher.ota_*` only. No A/B. Not MATCH.
