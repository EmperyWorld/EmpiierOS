# EmpiierOS — Capsule-gated OTA

**Status:** Living contract. **P0–P4 + E0–E4 Measure 2026-09-21**
(gates · helper · glass · staging · edge RO+tmpfs · cutover plant +
`ota_baseline.json`). **Not** MATCH. **Not** official `out/` bake.
**Plane:** `session/` + Runtime + Shell. **Not:** Tauri updater. **Not:** A/B P93.
**Work order:** [`../docs/plans/TZ-CAPSULE-OTA.md`](../docs/plans/TZ-CAPSULE-OTA.md) ·
[`../docs/plans/TZ-EMPIERO-OS-OTA-EDGE.md`](../docs/plans/TZ-EMPIERO-OS-OTA-EDGE.md).
**Operator how-to:** [`../docs/plans/OTA-OPERATOR-RUNBOOK.md`](../docs/plans/OTA-OPERATOR-RUNBOOK.md).
**Bootstrap pack (not this contract):** [`SPA-PACK.md`](SPA-PACK.md).

## Purpose

Deliver **OS component** and **subsequent FoFline** updates only as
**update capsules**, and only after the seat proves license, hardware bind,
an active EliteEW login, and a secure portal path. Attest runs **before**
the edge may transfer an update capsule.

## Production ops

Secure-reset Unix IPC ([`../runtime/HEALTH-IPC.md`](../runtime/HEALTH-IPC.md)
+ `runtime/src/core.rs`) — **not** the historical lab HTTP `update.*` path.

| Op | Kind | Notes |
|----|------|-------|
| `launcher.ota_status` | observe | Prerequisite chips + progress facts (gate / staged / apply `idle`/`ready`/`applied`/`os_before_payload`) |
| `launcher.ota_request` | mutation | Attest → VIP helper → verify → stage |
| `launcher.ota_apply` | mutation | Typed `APPLY_OTA`; **E4 planted** `os_component` cutover + baseline |

The production registry **disables** prefix `update.`. Do not call
`update.apply` / `channel.package_unpack` production-live.

## Hard gate (before transfer)

All four must hold or the op refuses (flags only; no secrets on IPC):

1. **License** — bound consume; `allow_fofline=true` from capsule inner.
2. **Hardware** — `device_keys/recipient.sk.hex` valid; device proof class
   aligned with air bind (`fp_v2` / signing material). No TPM claim day-1.
3. **EliteEW login** — `oidc_session=present`. Logout clears OTA.
4. **Secure connection** — portal iface up, host-map applied, helper
   VIP-only (`10.88.89.1`). Public edge denied.

## Bootstrap pack ≠ OTA

| Path | Op | When | OIDC required? |
|------|-----|------|----------------|
| First workspace pack | `launcher.spa_sync` | Journey stage `spa` | **No** (living) |
| Capsule OTA | `launcher.ota_*` | After login, Updates host workspace | **Yes** |

Do not rename Spa journey copy to «OTA». Do not move spa behind OIDC in
this contract (would break `LAUNCHER_JOURNEY` + first Open).

## Store

`{state}/ota-capsule/` — `staging/` / `current/` / `previous/` (`0700`);
manifest + allowlisted relative files (`runtime/` · `shell/` · `helpers/` only).
After cutover: `apply.json` (`0600`) with dest→sha256.
Artifact pubkey: `{state}/ota/artifact.pk.hex` or env `EMPIIEROS_OTA_ARTIFACT_PK`.
Shell never receives capsule bytes or signatures.

## Edge (E0–E3 — RO + tmpfs + CP)

Separate prod-core contour `stack-empiieros-ota` (`127.0.0.1:29181`):

1. Container **read_only**; capsule tree **tmpfs only**; **no** host
   `/srv/...` capsule bind (unlike Mobile/FoFline).
2. Capsules enter RAM **only from the command post** after F9 **artifact**
   Ed25519 sign. Restart → empty → fail-closed (`no_capsule`) until re-push.
3. VIP front `/empiieros-ota/` on living `app.eliteew.ru` vhost →
   `eliteew_empiieros_ota_upstream` (`10.77.77.2:29181` WG) → prod-core
   loopback stack. **E0b planted** (nginx + socat + FW rich-rule).
4. **E0c planted:** `scripts/empiieros-ota-cp-ingest.py` verifies
   `{v,payload,sig}` then `podman cp` into tmpfs; unsigned/wrong-key refuse.
5. **E1 planted:** production capsules are **blob-ref**
   (`path`+`sha256`+`href` `/empiieros-ota/v1/blobs/<sha256>`); CP
   `pack`/`publish`; seat `verify_and_stage_with_blobs`. Inline base64 is
   fixture-only.
6. **E2 planted:** VIP allowlist GET `/empiieros-ota/` + POST
   `/empiieros-ota/v1/attest`; edge portal-path mint (`grant=portal_path_v1`
   or `no_capsule`); helper HTTPS attest→blobs→stage
   (`TimeoutStartSec=900`). Without helper binary → `ota_helper_required`.
7. **E3 planted:** verify-only `{state}/ota/artifact.pk.hex` is planted
   on **Air license** (`launcher.air_activate`, admin `EWAE1` code).
   The sealed entitlement carries `payload.ota_artifact_pk` (F9 artifact
   verify PK). That is the priority path — not installer media and not
   a file dropped next to `.ecap`. File import may still plant a sibling
   PK if an operator uses USB Capsule. Living CP SK lives in
   `$XDG_DATA_HOME/empiieros/ota/` via
   `scripts/empiieros-ota-operator-key.sh` (never `/tmp`, never git).
   Issuer TZ: [`../docs/plans/ELITEEW-TZ-OTA-PK-IN-AIR-LICENSE.md`](../docs/plans/ELITEEW-TZ-OTA-PK-IN-AIR-LICENSE.md)
   — Line A `55c7dc7af`; SOPS 15:43 + API recreate (operator: process
   PK SET). Public `/api/health` 200. FoFline mint `6d312e59` completed.
   Seat plant: `launcher.ota_status` plants from persisted inner.
   Lab helper `scripts/empiieros-ota-plant-seat-pk.py` stays for fixtures;
   gate `artifact_pk_missing`; live edge fetch→stage Measure.

Details: [`../docs/plans/TZ-EMPIERO-OS-OTA-EDGE.md`](../docs/plans/TZ-EMPIERO-OS-OTA-EDGE.md).

## Cutover (E4 planted)

Exact destination table and restart allowlist live in
[`../docs/plans/TZ-CAPSULE-OTA.md`](../docs/plans/TZ-CAPSULE-OTA.md) §Cutover.
Summary:

1. Atomic `dest.new` → `rename` only for mapped Runtime / Shell GUI /
   Shell assets / allowlisted helpers (no ESP/LUKS/kernel). Unmapped
   staged paths → `path_escape`. `runtime/empiieros-shell-gui` lands at
   `/usr/local/bin/empiieros-shell-gui` (right-edge HUD). A seat whose
   living Runtime predates this row must take a Runtime-only capsule
   first; otherwise that path is `path_escape`.
2. Record digests in `{state}/ota-capsule/apply.json` and write
   `{state}/seat_shield/ota_baseline.json` (production absolute dest →
   sha256). Shield rematch → `ota_baseline: ok|mismatch|…`.
3. Restart only units whose binaries changed. `shell/` assets do not
   restart GUI. Runtime and Shell GUI must not `try-restart` before
   the IPC newline (that is `invalid runtime response framing`); they
   are deferred (`systemd-run --on-active=3s`). Watch may restart
   immediately. Oneshot helpers pick up on next start.
   `ProtectSystem=strict` needs drop-in `ota-cutover.conf`
   `ReadWritePaths` for `/usr/local/{bin,sbin}` `/usr/libexec`
   `/opt/empiieros` or apply is a blink (`ota_dest_new`). Apply plants
   that drop-in after extract.
4. **OS-before-payload:** only `channel=os_component` may cutover.
   `fofline_update` → `os_before_payload` (must not rewrite `/usr` /
   `/opt/empiieros/shell`).
5. Lab/tests: `EMPIIEROS_OTA_CUTOVER_ROOT` remaps destinations under a
   temp floor (no host unit restart).

## Glass

Licensed **Updates** host app (`OtaPanel`) hosts `SeatOtaSection`.
The window title lives on `WorkspaceFrame` only. Body is prerequisites
(chips), progress (gate / staged / apply facts), request, then the
apply spoiler. Right-edge dock flyout opens that workspace. Not a
System tab. Not a launcher journey step. Typed confirm `APPLY_OTA`
before apply.
Unlicensed seats never see Updates — correct. Lab pack re-sync is
not this surface. Admin-code Air plants the verify PK inside the
license capsule; do not put it on installer media and do not recover
it with a USB GRUB helper.

## Signing

F9 roles stay separate: entitlement ≠ artifact. Artifact-role Ed25519 over
`{v,payload,sig}` (see `runtime/src/ota_capsule_store.rs`). CP signs before
ingest; edge refuses unsigned (**E0c Measure**); production payloads use
blob-ref hrefs (**E1 Measure**); seat verifies before stage and again at
cutover. Pubkey: `{state}/ota/artifact.pk.hex` or `EMPIIEROS_OTA_ARTIFACT_PK`.
Empty/missing → `capsule_signature_required` / `artifact_role_mismatch`.
