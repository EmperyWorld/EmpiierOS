# TZ — EmpiierOS OTA edge (RO + tmpfs + CP ingest)

**Status:** Binding work order. **E0–E4 Measure 2026-09-21** (RO+tmpfs + VIP
+ pack/publish + attest + plant PK + live edge fetch→stage + Shield observe +
cutover plant + `ota_baseline.json`). **Not** MATCH. **Not** metal cutover claim.
**Date:** 2026-09-21
**Living contracts:** [`../../session/OTA-CAPSULE.md`](../../session/OTA-CAPSULE.md) ·
[`../../session/SEAT-SHIELD.md`](../../session/SEAT-SHIELD.md)
**Seat wave:** [`TZ-CAPSULE-OTA.md`](TZ-CAPSULE-OTA.md) (P0–P4 local Measure)
**Operator how-to:** [`OTA-OPERATOR-RUNBOOK.md`](OTA-OPERATOR-RUNBOOK.md)
**F9:** [`../foundation/RELEASE-SIGNING-STUBS.md`](../foundation/RELEASE-SIGNING-STUBS.md)
**Kinship stack:** `powered_by_emperyworld/docker/stack/stack-empiieros-ota.yml`

## Law

- **Separate contour.** Not Mobile `:29180`, not `/desktop-client-updates/`,
  not `/fofline-updates/`.
- **Edge posture:** container **read_only**; capsule tree **tmpfs only**;
  **no host bind** of capsule content. Restart → empty → fail-closed until CP
  re-pushes.
- **CP-only ingest.** Capsules enter RAM only from the command post after
  **F9 artifact** Ed25519 sign. Auth transport ≠ substitute for signature.
- **VIP path (E0b):** `/empiieros-ota/` on `app.eliteew.ru` (edge nginx) →
  WG backhaul `eliteew_empiieros_ota_upstream` (`10.77.77.2:29181`) →
  prod-core `stack-empiieros-ota` loopback. **Not** `eliteew_app_upstream`.
  Seat reaches it as portal VIP `10.88.89.1:443` SNI `app.eliteew.ru`.
- **Seat:** `launcher.ota_*` only; E4 cutover planted for `os_component`
  (lab Measure). FoFline stays `os_before_payload`. Not metal MATCH.
- **Shield:** observe-only OTA baseline (see SEAT-SHIELD §OTA). Do not enable
  `security.integrity.*`. Do not put good digests in IOC feed.
- No private keys in git / edge image / seat state. Not MATCH from this TZ.

## Edge coordinates

| Piece | Value |
|-------|--------|
| Stack | `stack-empiieros-ota` |
| Bind | `127.0.0.1:29181:80` |
| Root | `read_only: true` |
| Capsule store | tmpfs `/srv/empiieros-ota` (size cap class **512m**) |
| Host capsule bind | **forbidden** |
| Health | `GET /health` → `ok` even when empty |
| Empty grant | blob/current absent → `no_capsule` (fail-closed) |

## Capsule signing

| Step | Who | Lock |
|------|-----|------|
| Pack + sign | CP | `{v,payload,sig}` F9 **artifact** Ed25519; blob digests in payload |
| Ingest | Edge (E0c+) | Auth **and** verify PK; refuse unsigned / wrong role before tmpfs write |
| Stage | Seat | `verify_and_stage` again with `{state}/ota/artifact.pk.hex` |
| Not | — | entitlement SK; FoFline/Tauri minisign as OS capsule SoT |

## CP ingest (E0c — planted)

- Script: `scripts/empiieros-ota-cp-ingest.py` (`selftest` / `sign-fixture` /
  `ingest` / `pack` / `publish`). Verify F9 `{v,payload,sig}` against
  verify-only PK, then `podman cp` into container tmpfs only (no host `/srv`
  capsule bind).
- Verify backend: `cryptography` when present; else `openssl pkeyutl`
  (prod-core without pip). Sign/selftest/pack still need cryptography on CP.
- Refuse: unsigned, wrong PK, oversize, path escape, href escape — no write.
- Restart clears RAM → empty grant paths fail-closed until CP re-pushes.
- Day-1: operator runs ingest/publish on the host that owns
  `stack-empiieros-ota` (loopback) or `publish --ssh`. Authenticated HTTP
  ingest API = later; not this Measure.

## Capsule blob-ref (E1a — planted)

- Production entries: `path` + `sha256` + `href` =
  `/empiieros-ota/v1/blobs/<sha256>` (exact; no `..` / query / foreign prefix).
- Seat: `ota_capsule_store::verify_and_stage_with_blobs` loads bytes from a
  helper download `blob_dir`. Inline `content_base64` remains for tiny fixtures
  only (256 KiB / 1 MiB caps). Blob path caps match ingest (256 MiB / 512 MiB).
- Edge nginx already serves `/v1/blobs/` from tmpfs after ingest.

## CP pack/publish (E1b — planted)

- `pack`: allowlisted `--file rel=abs` and/or `--src` tree (`runtime/` /
  `shell/` / `helpers/`) → signed blob-ref `capsule.json` + `blobs/` +
  `artifact.pk.hex`. SK via `--sk-file` / `EMPIIEROS_OTA_ARTIFACT_SK` (CP only).
- `publish`: pack (or existing capsule) → ingest local or `--ssh` host.
- Local CP stage is audit SoT; prod host disk is **not** capsule SoT.
- **Living CP artifact SK** is operator-held at
  `$XDG_DATA_HOME/empiieros/ota/artifact.sk.hex` (mode `0600`).
  Create/reuse with `scripts/empiieros-ota-operator-key.sh init`
  (never rotates an existing SK). Never git. Never `/tmp`. Measure
  scripts may redact only their own `$WORK` key. Do not overwrite the
  living SK with `REDACTED`.

## Edge attest (E2c — planted)

- `POST /v1/attest` (VIP `/empiieros-ota/v1/attest`) — portal-path trust
  day-1 (no ticket harvest yet).
- If `/srv/empiieros-ota/current.json` present →
  `{"ok":true,"grant":"portal_path_v1","capsule_href":"/empiieros-ota/v1/current.json"}`.
- Empty tmpfs → **404** `{"error":"no_capsule"}`. GET denied (403).
- Seat helper: POST attest → GET current → GET blobs →
  `verify_and_stage_with_blobs`. Runtime stays `IPAddressDeny=any`.

## Seat Shield OTA baseline (E3c+E4 planted)

| Hook | Status |
|------|--------|
| Event `ota_capsule_staged` | **planted E3c** (fail-soft on stage) |
| `{state}/seat_shield/ota_baseline.json` | **planted E4** (written on cutover) |
| Status `ota_baseline: missing\|ok\|mismatch\|stale` | **planted** |
| VIP `10.88.89.1` never auto BlockPeer | **planted E3c** |
| Rematch P4 paths; observe-only mismatch on cycle | **planted E4** |

## Phases

| Phase | Outcome |
|-------|---------|
| **E0** | This TZ + contracts + RO/tmpfs stack + start script + autostart health — **local+prod Measure 2026-09-21** (`:29181` health ok; empty 404; `ReadonlyRootfs=true`; binds `[]`) |
| **E0b** | Living `app.eliteew.ru` `/empiieros-ota/` → WG `10.77.77.2:29181` — **Measure green 2026-09-21** (`health` ok; empty `current` 404; FW rich-rule planted) |
| **E0c** | Ingest sig verify → tmpfs — **Measure green 2026-09-21** (signed public `current` 200; unsigned/wrong-key refuse; restart → 404) |
| **E1a** | Seat blob-ref capsule model — **Measure green 2026-09-21** (store tests: href stage + escape FAIL) |
| **E1b** | CP pack/sign/publish — **Measure green 2026-09-21** (blob-ref pack; public `current`+blob 200; unsigned refuse) |
| **E2a** | Seat VIP allowlist GET `/empiieros-ota/` + POST attest — **Measure green 2026-09-21** |
| **E2b** | Helper HTTPS attest→blobs→stage + `TimeoutStartSec=900` — **planted 2026-09-21** (live stage needs E3 PK) |
| **E2c** | Edge `/v1/attest` portal-path mint — **Measure green 2026-09-21** (empty `no_capsule`; grant when current present) |
| **E3** | Artifact PK plant + live fetch→stage — **Measure green 2026-09-21** (`plant_artifact_pk`; edge bytes → staged; portal-VIP helper metal pending) |
| **E3b** | Glass/probe edge codes — **planted 2026-09-21** (`no_capsule` / `artifact_pk_missing` / helper chips) |
| **E3c** | Shield observe hooks — **Measure green 2026-09-21** (VIP exempt; `ota_baseline:missing`; `ota_capsule_staged`) |
| **E4** | Cutover plant + baseline write — **Measure green 2026-09-21** (`ota_cutover`; lab-root apply; `os_before_payload` for fofline; Not metal claim) |

## E0 Measure

- Files on disk: this TZ; OTA-CAPSULE §Edge; SEAT-SHIELD §OTA; kinship
  `stack-empiieros-ota.yml` + Dockerfile/nginx; `prod-core-empiieros-ota-start.sh`
- Prod (or lab): `curl -sf http://127.0.0.1:29181/health` → `ok`
- `podman inspect` / compose: **no** host bind for `/srv/empiieros-ota` content
- Empty tmpfs: no current capsule → fail-closed for grant paths
- Seat: `launcher.ota_apply` still `apply_not_planted`; request still
  `attest_path_unplanted` until later phases (superseded by E2 helper path)
- `scripts/preflight-foundation.sh` exit 0
- **Not** MATCH / bake / USB / cutover / VIP plant claim

## E0c Measure

- `scripts/empiieros-ota-cp-ingest.py selftest` exit 0 (unsigned / wrong-key /
  oversize / path_escape refuse)
- Prod: signed fixture ingest →
  `https://app.eliteew.ru/empiieros-ota/v1/current.json` **200**
- Unsigned / wrong PK → refuse (exit ≠ 0); no tmpfs write from refuse path
- `podman restart stack-empiieros-ota` → `/v1/current.json` **404** (empty)
- Artifact SK never committed; lab fixture SK redacted after sign
- Seat: still `apply_not_planted` / mint unplanted
- **Not** MATCH / cutover / E1 pack pipeline

## E1 Measure

- Seat: `cargo test --manifest-path runtime/Cargo.toml ota_capsule_store`
  exit 0 (blob-ref stage + href escape / digest mismatch FAIL)
- `scripts/empiieros-ota-cp-ingest.py selftest` exit 0 (includes href refuse)
- Pack blob-ref capsule (no `content_base64` in production entries) →
  `publish --ssh prod-core` (or local) →
  `https://app.eliteew.ru/empiieros-ota/v1/current.json` **200** and
  `/v1/blobs/<sha256>` **200**
- Unsigned publish/ingest refuse; SK never committed
- Seat: still `apply_not_planted` / `attest_path_unplanted`
- **Not** MATCH / mint / cutover / helper HTTP

## E2 Measure

- Seat: `https_vip` allowlist tests — GET `/empiieros-ota/…`, POST
  `/empiieros-ota/v1/attest`; `..` denied; fofline paths kept
- Helper unit: `TimeoutStartSec=900`; `parse_attest_grant` accepts only
  `portal_path_v1` + `/empiieros-ota/v1/current.json`
- Edge image `eliteew-empiieros-ota:e2`: POST `/v1/attest` — empty tmpfs →
  **404** `no_capsule`; after CP publish → **200** grant
- Public VIP: same empty/grant evidence on
  `https://app.eliteew.ru/empiieros-ota/v1/attest`
- Without helper binary: `launcher.ota_request` → `ota_helper_required`
  (not the old forever-stub `attest_path_unplanted`)
- Apply still `apply_not_planted`; live stage still needs E3 PK plant
- **Not** MATCH / cutover

## Artifact PK plant (E3 — planted)

- Seat path: `{state}/ota/artifact.pk.hex` (`0600`) or env
  `EMPIIEROS_OTA_ARTIFACT_PK`. Verify-only — never SK.
- **License plant:** `launcher.air_activate` (admin `EWAE1`) plants the
  PK from sealed entitlement `payload.ota_artifact_pk`. That is the
  priority path. USB file import may plant a sibling PK. Installer
  media must not carry this file. Issuer TZ:
  [`ELITEEW-TZ-OTA-PK-IN-AIR-LICENSE.md`](ELITEEW-TZ-OTA-PK-IN-AIR-LICENSE.md)
  — Line A `55c7dc7af`; SOPS + API recreate (operator: process PK
  SET). FoFline Air mint not walked.
- Living CP SK: `scripts/empiieros-ota-operator-key.sh` →
  `$XDG_DATA_HOME/empiieros/ota/` (`0600`). Never `/tmp`. Never git.
- Lab/CP fixture: `scripts/empiieros-ota-plant-seat-pk.py --state-dir … --pk-file …`.
- Status chips: `artifact_pk=present|missing`, `ota_helper=present|missing`.
- Gate: `artifact_pk_missing` before request. Factory reset wipes `ota/`.
- Lab Measure: `scripts/empiieros-ota-e3-stage-measure.sh` (live edge
  current+blobs + plant PK → `verify_and_stage_with_blobs`). Portal-VIP
  helper metal walk is separate (needs seat with `10.88.89.1`).

## E3 Measure

- `plant_artifact_pk` / missing-PK fail-closed unit tests
- `EMPIERO_OS_OTA_PUBLISH_SSH=prod-core bash scripts/empiieros-ota-e3-stage-measure.sh`
  exit 0 — attest grant 200; staged current; `ota_apply=apply_not_planted`
- Glass: localized `no_capsule` / `artifact_pk_missing` / `ota_helper_required`
- **Not** MATCH / cutover / portal-VIP helper metal claim

## E3c Measure

- `cargo test --manifest-path runtime/seat_shield_core/Cargo.toml ota_observe::` — VIP + baseline missing/ok/mismatch + staged emit
- `cargo test --manifest-path runtime/seat_shield_core/Cargo.toml ota_portal_vip_never_proposes_block` — exit 0
- `seat.shield.status` carries `ota_baseline: "missing"` when baseline file absent
- Stage path calls `emit_ota_capsule_staged` (fail-soft)
- `scripts/preflight-foundation.sh` exit 0
- **Not** MATCH / metal claim

## E4 Measure

- `cargo test --manifest-path runtime/Cargo.toml ota_cutover::` — 4 ok
  (map refuse; lab-root apply + baseline `ok`; fofline `os_before_payload`)
- `launcher.ota_apply` + `APPLY_OTA` → cutover for `os_component`
- Writes `{state}/ota-capsule/apply.json` + `{state}/seat_shield/ota_baseline.json`
- Status chip `ota_apply`: `idle` / `ready` / `applied` / `os_before_payload`
- Cycle emits `ota_component_mismatch` on rematch fail (observe-only)
- `scripts/preflight-foundation.sh` exit 0
- **Not** MATCH / metal seat cutover / official `out/`

## Operator phrase

> Unlock EmpiierOS write for OTA edge contour per
> `docs/plans/TZ-EMPIERO-OS-OTA-EDGE.md`. RO+tmpfs edge, CP-only ingest,
> no host capsule bind. Seat Shield OTA baseline + E4 cutover plant.
> Not MATCH.
