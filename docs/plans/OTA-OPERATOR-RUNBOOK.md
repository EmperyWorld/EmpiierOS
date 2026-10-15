# EmpiierOS OTA — operator runbook

**Status:** Living how-to. **Not** MATCH. **Not** official `out/` bake.
**Date:** 2026-09-21
**Contracts:** [`../../session/OTA-CAPSULE.md`](../../session/OTA-CAPSULE.md) ·
[`TZ-CAPSULE-OTA.md`](TZ-CAPSULE-OTA.md) ·
[`TZ-EMPIERO-OS-OTA-EDGE.md`](TZ-EMPIERO-OS-OTA-EDGE.md) ·
[`ELITEEW-TZ-OTA-PK-IN-AIR-LICENSE.md`](ELITEEW-TZ-OTA-PK-IN-AIR-LICENSE.md) ·
[`../../docs/foundation/RELEASE-SIGNING-STUBS.md`](../foundation/RELEASE-SIGNING-STUBS.md)

This file is the operator path: keys, container, sign, publish, seat apply,
and the traps that already burned a living seat. Destination tables and
refuse codes stay in the contracts above — do not fork them here.

## What this is

Capsule-gated **OS component** updates after a licensed, logged-in,
portal-up seat. Ops are `launcher.ota_status` / `ota_request` / `ota_apply`
only. Production registry disables `update.*`.

Not this path:

| Thing | Why not |
|-------|---------|
| `launcher.spa_sync` | First FoFline pack (pre-OIDC). Do not label it OTA. |
| Tauri updater | Living host elsewhere. Not EmpiierOS SoT. |
| P93 A/B rootfs | Frozen. Capsule may not rewrite kernel / ESP / LUKS. |
| USB / GRUB PK plant | Adult path is Air `payload.ota_artifact_pk`. |
| Public HTTPS blob pull | Helper is VIP-only `10.88.89.1`. |
| Official factory `out/` | Last stamp `5979e9cf…` (2026-09-20). Mux / overlay ≠ `out/`. |

## Planes (never conflate)

| Plane | Owns | Lives |
|-------|------|-------|
| Command post (CP) | Artifact Ed25519 SK/PK; pack; publish | Operator workstation. SK: `$XDG_DATA_HOME/empiieros/ota/` |
| Edge container | Serve `current.json` + blobs from **tmpfs** | prod-core `stack-empiieros-ota` `127.0.0.1:29181` |
| VIP front | `/empiieros-ota/` on `app.eliteew.ru` | nginx → WG `10.77.77.2:29181` → loopback stack |
| EliteEW API | Puts **verify PK** into Air entitlement | SOPS / process env `EMPIERO_OS_OTA_ARTIFACT_PK` — **PK only** |
| Seat Runtime | Verify, stage, cutover | `{state}/ota/` + `{state}/ota-capsule/` |
| Seat glass | Prerequisite chips, progress facts, typed `APPLY_OTA` | Updates host app (`OtaPanel`) |

Do **not** redeploy `stack0` / `eliteew-api-prod` to “refresh OTA”. The
OTA contour is a **separate** compose. Restarting the OTA container
**empties RAM** — public `current.json` becomes 404 until CP publishes
again.

## Artifact vs entitlement (F9)

Roles are not interchangeable. Wrong key = fail-closed.

| Role | Material | Signs / verifies | Must not |
|------|----------|------------------|----------|
| `entitlement` | EliteEW license SK | Capsule v3 / entitlement `{v,payload,sig}` | OS update blobs |
| `artifact` | CP OTA SK | Update capsule `{v,payload,sig}` | License / Air |
| Seat recipient | `{state}/device_keys/recipient.sk.hex` | Device bind | Either signing role |
| First-boot PIN | Runtime IPC | Unlock | LUKS / Air SK / artifact SK |

Living artifact **verify PK** (2026-09-21), 64 lowercase hex:

```
813cfc81a78de51b9cfe03802b8433497810b814c43558ad068235b14c32796f
```

Print PK only:

```bash
scripts/empiieros-ota-operator-key.sh pk
# or: scripts/empiieros-ota-operator-key.sh show   # prints PK=… SK=present
```

### SK law

- Create/reuse: `scripts/empiieros-ota-operator-key.sh init` (never rotates).
- Path: `$XDG_DATA_HOME/empiieros/ota/artifact.sk.hex` (`0600`).
- Never git. Never `/tmp`. Never installer USB. Never SOPS. Never API env.
- Never commit `artifact.sk.hex`. Pack writes `artifact.pk.hex` next to
  the capsule — that file is **verify-only**.
- A `REDACTED` SK file is a stop. Restore the real SK; do not mint a
  second living key and call it the same PK.

If the SK is lost, every already-minted Air license still carries the
**old** PK. Seats will refuse capsules signed with a new SK
(`capsule_signature_required`). Recovery is a new artifact key **and**
new Air licenses (or a documented PK rotation — not planted).

## How the PK reaches the seat

Priority path (adult):

1. CP PK is planted on EliteEW API as `EMPIERO_OS_OTA_ARTIFACT_PK`
   (process env / SOPS tmpfs). Never the SK.
2. Admin Platform Ops issues FoFline Air (`EWAE1`).
3. Issuer `build_payload` sets `payload.ota_artifact_pk` when the env
   resolves. Omit the key if missing — do not write `null`.
4. Seat `launcher.air_activate` persists inner entitlement.
5. `launcher.ota_status` / import plants `{state}/ota/artifact.pk.hex`
   from `payload.ota_artifact_pk` (`plant_from_persisted_inner`).
6. Gate before request: `artifact_pk_missing` if the file is absent.

USB file import may still plant a **sibling** PK. Installer media must
not carry `artifact.pk.hex` / `artifact.sk.hex`. Flash refuses leftover
keys on ESP or live share.

Factory reset wipes `{state}/ota/` and `{state}/ota-capsule/`. A wipe
also spends the Air session — next seat needs a **new** admin code.

Issuer TZ (EliteEW tree, do not edit EmpiierOS from that session):
[`ELITEEW-TZ-OTA-PK-IN-AIR-LICENSE.md`](ELITEEW-TZ-OTA-PK-IN-AIR-LICENSE.md).

## Edge container (how it is rolled)

Kinship compose: `powered_by_emperyworld/docker/stack/stack-empiieros-ota.yml`.

| Lock | Value |
|------|--------|
| Name | `stack-empiieros-ota` |
| Bind | `127.0.0.1:29181:80` |
| Root | `read_only: true` |
| Capsule store | tmpfs `/srv/empiieros-ota` (512m class) |
| Host bind of capsules | **forbidden** |
| Health | `GET /health` → `ok` even when empty |
| Empty current | `GET /v1/current.json` → 404; attest → `no_capsule` |

Capsules enter RAM **only** after CP verify + `podman cp` into that
tmpfs (`scripts/empiieros-ota-cp-ingest.py`). Auth transport is not a
substitute for the artifact signature.

Public path the seat uses:

```text
seat helper (IPAddressAllow=10.88.89.1)
  → https://app.eliteew.ru/empiieros-ota/  (SNI, VIP 10.88.89.1:443)
  → edge nginx  /empiieros-ota/
  → WG eliteew_empiieros_ota_upstream  10.77.77.2:29181
  → prod-core loopback stack-empiieros-ota
```

Not `eliteew_app_upstream`. Public TCP 443 to the origin IP stays denied
from the seat (`IPAddressDeny=any` on Runtime).

**Restart = empty.** `podman restart stack-empiieros-ota` is a publish
rollback. Re-push from CP. Do not treat an empty edge as “no updates
exist” on a seat that already staged.

Day-1 ingest API is **not** HTTP. Operator runs ingest on the host that
owns the container, or `publish --ssh`.

## Capsule shape

Production documents are **blob-ref**, not inline base64.

```json
{
  "v": 1,
  "payload": {
    "kind": "empiieros_ota_capsule_v1",
    "channel": "os_component",
    "version": "optional",
    "app_build_id": "required-for-humans",
    "files": [
      {
        "path": "shell/index.html",
        "sha256": "<64 hex>",
        "href": "/empiieros-ota/v1/blobs/<same 64 hex>"
      }
    ]
  },
  "sig": "<64-byte Ed25519 hex>"
}
```

Locks:

- Canonical payload bytes: sorted keys, `separators=(",", ":")`,
  `ensure_ascii=False`. Sign **that**. A pretty-printed payload that
  was not sorted will not verify on the seat.
- `path` allowlist: `runtime/` · `shell/` · `helpers/` only. No `..`,
  no absolute, no `:`. Max 64 files, 256 MiB/file, 512 MiB total.
- `href` exact prefix `/empiieros-ota/v1/blobs/<sha256>` — no query,
  no foreign host, digest must match `sha256`.
- `channel`: `os_component` (cutover) or `fofline_update` (seat apply
  refuses with `os_before_payload` until OS cutover exists).
- Inline `content_base64` is fixture-only.

Mapped destinations: [`TZ-CAPSULE-OTA.md`](TZ-CAPSULE-OTA.md) §Cutover.
Unmapped path → `path_escape`. No kernel, ESP, LUKS, `/etc` (except a
future unit-plant wave), arbitrary `/usr`, state secrets.

## Sign and publish (CP)

Need python3 `cryptography` on the CP for pack/sign. Prod ingest host
may verify with `openssl pkeyutl` only.

```bash
# 1) Durable key (once). Prints PK. Never rotates an existing SK.
scripts/empiieros-ota-operator-key.sh init

# 2) Pack allowlisted files. SK from XDG or --sk-file.
#    --src is a tree with runtime/ shell/ helpers/ (relative paths).
#    Or repeat --file rel=abs.
WORKDIR="$(mktemp -d /tmp/empiieros-ota-pack.XXXXXX)"
scripts/empiieros-ota-cp-ingest.py pack \
  --out-dir "$WORKDIR" \
  --channel os_component \
  --version "0.0.1" \
  --app-build-id "human-id-that-matches-the-bits" \
  --sk-file "${XDG_DATA_HOME:-$HOME/.local/share}/empiieros/ota/artifact.sk.hex" \
  --src /path/to/capsule-tree

# 3) Verify without touching the container.
scripts/empiieros-ota-cp-ingest.py ingest \
  --capsule "$WORKDIR/capsule.json" \
  --blob-dir "$WORKDIR/blobs" \
  --pk-file "$WORKDIR/artifact.pk.hex" \
  --verify-only

# 4) Publish into running tmpfs (host that owns the container, or SSH).
scripts/empiieros-ota-cp-ingest.py publish \
  --capsule "$WORKDIR/capsule.json" \
  --blob-dir "$WORKDIR/blobs" \
  --pk-file "$WORKDIR/artifact.pk.hex" \
  --ssh owned@192.168.1.111
```

`publish` without `--capsule` will pack first. Remote publish rsyncs
the script + capsule + blobs + PK (not SK) and runs `ingest` on the
host. Unsigned / wrong PK / path escape → refuse, **no** tmpfs write.

Self-test (unsigned / wrong-key / oversize / href refuse):

```bash
scripts/empiieros-ota-cp-ingest.py selftest
```

Public smoke after publish (from a host that can see the vhost):

```text
GET https://app.eliteew.ru/empiieros-ota/health          → ok
GET https://app.eliteew.ru/empiieros-ota/v1/current.json → 200 signed doc
GET https://app.eliteew.ru/empiieros-ota/v1/blobs/<sha>  → 200 bytes
POST https://app.eliteew.ru/empiieros-ota/v1/attest      → grant portal_path_v1
```

Attest is portal-path day-1 (no ticket harvest). GET attest is 403.
Empty tmpfs → attest 404 `no_capsule`.

Shield + tray notifications wave (2026-09-22):

```bash
scripts/empiieros-ota-pack-shield-wave.sh --publish
# default publish target: prod-core (VIP edge). Pack only: omit --publish.
```

### What to pack (do not downgrade the seat)

Pack the **same bits the seat should live on**, not an older morning
build.

| Capsule | When |
|---------|------|
| `shell/**` only | Glass / Settings / Updates UI. Does **not** restart GUI. |
| `runtime/empiieros-runtime` | Cutover Runtime. Deferred `try-restart` after IPC reply. |
| `runtime/empiieros-shell-gui` | Native HUD. Deferred GUI restart. |
| `helpers/empiieros-ota-fetch` | VIP helper. Oneshot — next start picks it up. |

Trap already walked: `launcher-path-1-living-pk` verified under
`813cfc81…` but carried morning `index-CV5gUiMd.js`. Applying it over
mux `index-C33yKaFx.js` **downgrades** glass. Resign the **living**
shell (the bundle the seat already has, or newer) before Apply.

`app_build_id` is operator archaeology. The seat does not accept a
capsule because the name looks new — only signature + dest allowlist.

## Seat request and apply

Hard gate (all four + helper + PK) or the op refuses. Flags only; no
secrets on IPC.

1. License — bound consume, `allow_fofline=true`.
2. Hardware — living `device_keys/recipient.sk.hex`.
3. EliteEW login — `oidc_session=present`. Logout clears OTA.
4. Portal — iface up, host-map applied, helper VIP-only.
5. `{state}/ota/artifact.pk.hex` present.
6. `/usr/local/sbin/empiieros-ota-fetch` present (`ota_helper_required`
   otherwise — official Sept-20 extract can miss this; USB apply plants
   the live helper).

Flow:

1. Glass **Request** → `launcher.ota_request` → helper POST attest →
   GET current + blobs → Runtime `verify_and_stage_with_blobs`.
2. Chip `ota_staged` + `ota_apply=ready` (os_component).
3. Typed confirm `APPLY_OTA` → `launcher.ota_apply`.
4. Cutover: re-verify; atomic `dest.new` → `rename`; write
   `{state}/ota-capsule/apply.json` and
   `{state}/seat_shield/ota_baseline.json`.
5. Restart **only changed binaries**. `shell/` does not restart GUI
   (`ota_reload_shell`). Runtime / GUI restart is
   `systemd-run --on-active=3s` **after** the IPC newline. Restarting
   them first is `invalid runtime response framing` (native bridge
   needs one JSON line).
6. `ProtectSystem=strict` without
   `empiieros-runtime.service.d/ota-cutover.conf`
   `ReadWritePaths=/usr/local/bin /usr/local/sbin /usr/libexec /opt/empiieros`
   → apply blinks `ota_dest_new`. USB apply plants the drop-in; the
   EmpiierOS overlay now ships it too.

Helper unit: `TimeoutStartSec=900`, `IPAddressAllow=10.88.89.1`.

Lab only: `EMPIIEROS_OTA_CUTOVER_ROOT` remaps dests (no host units).

## Builds — what actually contains the fixes

Honesty table. Do not quote a mux hash as official `out/`.

| Artifact | Runtime framing fix (`--on-active=3s`) | Air PK plant | Notes |
|----------|----------------------------------------|--------------|-------|
| EmpiierOS **source** | yes | yes | `runtime/src/ota_cutover.rs` / `ota_capsule_store.rs` |
| Host `cargo build --release` (USB mux) | yes `f734ed76…` | yes | Fedora-native. SoT mux `c9993f85…` on stick `071C57D0F48A4A92` |
| Previous overlay `0e0e19d8…` (2026-09-21 10:08) | **no** `6737388a…` | **no** | Superseded as current overlay link |
| Current overlay `7590c54c…` (2026-09-21 18:05 +10) | yes `2783fa8a…` | yes (`ota_artifact_pk`) | Shell `index-BKgVdGT6.js` (`ota_reload_shell`); drop-in `ota-cutover.conf`. Packager lock. **Not** `out/` |
| Official factory `out/` `5979e9cf…` | **no** | **no** | 2026-09-20 LUKS floor. QEMU not current. Needs factory assemble |

`scripts/package-shell-gui-rootfs.sh` is the application overlay
generator. It now refuses a Runtime without `plant_from_persisted_inner`
and `--on-active=3s`, refuses a Shell bundle without `ota_reload_shell`,
and plants `ota-cutover.conf`. That overlay is **not** MATCH and is
**not** official `out/` unless the operator runs a factory assemble.

USB mux is a live-installer plant (`empiieros-installer-media-plant-runtime.sh`).
Apply on that stick is keep-SoT (`extracted Runtime already IdP Date DPoP;
keep SoT`) so extract is not clobbered by the live installer ELF.

## Operator sequence (living seat)

1. Confirm chips: license, device, OIDC, portal+hostmap, **artifact PK**,
   **helper**, staged, apply `ready`.
2. Confirm public `current.json` verifies under **the same** PK the seat
   planted (`813cfc81…` today). A morning signature under a prior F9
   key stages as VERIFY FAIL / yellow.
3. Confirm the capsule bits are **not older** than the seat glass.
4. Type `APPLY_OTA`. Expect `ota_applied` + optional glass reload.
   Framing error = the seat Runtime predates the deferred-restart lock
   (reinstall from the mux USB that carries `f734ed76…`, or wait for
   official overlay + bake).
5. Shield `ota_baseline` is observe-only. Do not put good dest digests
   in the IOC feed. VIP `10.88.89.1` is never auto BlockPeer.

## Refuse codes (glass)

`allow_fofline_required` · `license_required` · `device_keys_required` ·
`device_keys_invalid` · `hardware_bind_required` · `oidc_session_required` ·
`portal_required` · `portal_hostmap_required` · `secure_transport_required` ·
`attest_denied` · `capsule_signature_required` · `artifact_pk_missing` ·
`artifact_role_mismatch` · `path_escape` · `ota_not_staged` ·
`os_before_payload` · `confirmation_required` · `ota_helper_required` ·
`no_capsule` · `ota_dest_new` · `invalid runtime response framing`
(native bridge; not an IPC refuse token).

## Traps (walked 2026-09-21)

1. **SK in `/tmp` or REDACTED** — lost. Durable XDG only.
2. **PK not in Air** — activation green, artifact chip dark. Issuer +
   SOPS + API process env, then a **new** FoFline code. Living Runtime
   must know `plant_from_persisted_inner`.
3. **Apply clobbers extract** — old `plant_idp_date_runtime` overwrote
   SoT Runtime with the live installer ELF. Overlay apply now keep-SoT.
4. **No sudoers on the seat** — cannot live-mux Runtime; USB reinstall.
5. **`ProtectSystem=strict`** — dests RO without cutover drop-in.
6. **Helper missing** on Sept-20 extract — request → `ota_helper_required`.
7. **Glass `refresh()` wiped errors** — status pull must not clear
   request/apply errors (`clearFeedback` only on explicit refresh).
8. **Unconditional GUI/Runtime restart** — framing. Selective + defer.
9. **Resign ≠ newer glass** — signature OK can still downgrade UI.
10. **Container restart** — tmpfs empty; republish.
11. **USB mux ≠ official `out/`** — never MATCH from a stick plant.
12. **Wipe spends Air** — `6d312e59` and `013834cf` are spent.

## Measure commands

```bash
scripts/preflight-foundation.sh
cargo test --manifest-path runtime/Cargo.toml ota_cutover::
cargo test --manifest-path runtime/Cargo.toml launcher_ota::
scripts/empiieros-ota-cp-ingest.py selftest
scripts/empiieros-ota-operator-key.sh pk
# overlay (after package-shell-gui-rootfs.sh):
#   rg -a -- '--on-active=3s' $ELITEEW_TS_WORK_ROOT/empiieros-rootfs-overlay/usr/local/bin/empiieros-runtime
```

Official `out/` / QEMU / metal MATCH stay **not claimed** from this file.
