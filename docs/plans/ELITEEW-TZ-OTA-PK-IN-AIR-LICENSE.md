# TZ — EmpiierOS OTA verify PK inside Air license (implement)

**For:** EliteEW monorepo session (`powered_by_emperyworld`).
**From:** EmpiierOS 2026-09-21.
**Do not edit** EmpiierOS.
**Status:** Issuer in image + SOPS/env-push + API recreate
(2026-09-21). SOPS bundle on this clone is `15:43`. Operator:
living API process `EMPIERO_OS_OTA_ARTIFACT_PK=SET` `813cfc81…`.
**FoFline Air mint not walked.** Seat plant not walked. Not MATCH.

Admin code (`EWAE1`) is the priority license path. The seat already
plants `{state}/ota/artifact.pk.hex` from
`entitlement.payload.ota_artifact_pk` on `launcher.air_activate`.
The Line A image at `55c7dc7af` contains the issuer. A living FoFline
Air mint has **not** shown `payload.ota_artifact_pk` yet. Do not
re-implement the issuer. Do not invent a USB sidecar, a second WSS
frame, or a new IPC op.

## Living path (do not replace)

1. Admin Platform Ops → Air enroll (`allow_fofline`).
2. Seat: `launcher.air_activate` with the shown `EWAE1-…` code.
3. `shared/api/services/desktop_client/air_enroll_issuer.py`
   mints the license and calls `try_mint_entitlement`.
4. `shared/api/services/desktop_client/entitlement_sign.py`
   `build_payload` → `sign_payload` → `{v, payload, sig}`.
5. `maybe_seal_for_issue` seals that entitlement into Capsule v3.
6. WSS `/api/v1/desktop-client/air-enroll/ws` sends `capsule_meta` +
   binary `.ecap`.

## Field

| Item | Value |
|------|--------|
| Name | `ota_artifact_pk` |
| Where | **Inside signed entitlement `payload`** (same object as `license_id` / `allow_fofline`) |
| Shape | 64 lowercase hex chars, 32 bytes |
| Role | F9 **artifact verify PK** |
| Not | entitlement SK, artifact SK, recipient SK, license key |

EmpiierOS reads `entitlement.payload.ota_artifact_pk` first, then
top-level `entitlement.ota_artifact_pk`. Prefer the payload field so
it is covered by the entitlement signature.

Living CP verify PK (2026-09-21), copy **only this**:

```
813cfc81a78de51b9cfe03802b8433497810b814c43558ad068235b14c32796f
```

Source on the EmpiierOS CP (do not pull SK):

`$XDG_DATA_HOME/empiieros/ota/artifact.pk.hex`

## Env on API / prod-core

| Name | Meaning |
|------|---------|
| `EMPIERO_OS_OTA_ARTIFACT_PK` | 32-byte hex PK (preferred) |
| `EMPIERO_OS_OTA_ARTIFACT_PK_FILE` | optional path to a `0644`/`0600` file with that hex |

Malformed / `REDACTED` / wrong length → treat as missing.

Plant the **PK** on the API host. Never copy `artifact.sk.hex`.

## Code lock (exact files)

### 1. `shared/api/services/desktop_client/entitlement_sign.py`

- Add `resolve_ota_artifact_pk() -> str | None` (env, else file, else
  None). Validate `^[0-9a-f]{64}$` after strip/lower.
- `build_payload`: when a PK resolves, set
  `payload["ota_artifact_pk"] = pk`. When it does not resolve, omit
  the key (do not write `null`).
- Keep canonical JSON (`sort_keys=True`, separators `(',', ':')`).
  Adding one payload key is a signature-covered change. Old documents
  without the field stay verifiable.

### 2. `shared/api/services/desktop_client/air_enroll_issuer.py`

After `try_mint_entitlement(...)`, if `session.allow_fofline`:

- entitlement is `None` → already fail-closed today for offline;
  keep that.
- entitlement exists but `payload.ota_artifact_pk` missing or not
  32-byte hex → raise `DesktopClientError` named
  `ota_artifact_pk_missing`, HTTP **503**, no seal, no WSS capsule.

Do not add a new WSS `type`. Do not put the PK on `capsule_meta`.

### 3. USB `seat_package.py` (same mint)

If USB issue also calls `try_mint_entitlement`, the field will appear
there for free. Do not make USB the SoT. Do not require a sibling
`artifact.pk.hex` next to `.ecap`.

### 4. Docs in **this** monorepo (English)

One sentence in `documentation/product/desktop/DESKTOP-CLIENT-AIR-ENROLL.md`:
FoFline Air entitlement payload carries EmpiierOS OTA verify PK
(`ota_artifact_pk`). Never an SK.

## Tests (this repo)

`shared/api/tests/test_desktop_client_entitlement.py`:

1. Env set → `build_payload(..., allow_fofline=True)` has
   `ota_artifact_pk` equal to that hex; `sign_payload` /
   `verify_document` still pass.
2. Env unset → payload has **no** `ota_artifact_pk` key.
3. Env `REDACTED` / short hex → treat as unset.

Air issuer test (extend an existing air-enroll issuer/store test, or
add one next to it):

4. FoFline session + no PK env → `ota_artifact_pk_missing`, no
   capsule bytes written.
5. FoFline session + valid PK env → sealed package entitlement
   payload contains the same hex (or the minted entitlement doc does,
   if you assert before seal).

Do not log the PK in application info logs as a “secret”. It is
public. Still do not log SK.

## Prod plant (operator, same session)

On the host that runs Air issue (prod-core API):

```text
EMPIERO_OS_OTA_ARTIFACT_PK=813cfc81a78de51b9cfe03802b8433497810b814c43558ad068235b14c32796f
```

Staging file has this value. SOPS bundle on this clone is
`2026-09-21 15:43` (was 2026-09-18). Operator: env-push `--yes`
`storage_mode=runtime_tmpfs`, stack0 API force-recreate, process
`EMPIERO_OS_OTA_ARTIFACT_PK=SET` prefix `813cfc81`, no SK. `.env.stack0.api`
does not hold the key (shared + api mount — expected). EmpiierOS did
not inspect `.111` tmpfs. FoFline mint is still the Measure.

## Forbidden

- Artifact **SK** in git, API env, Admin UI, capsule, logs.
- New WSS frame / USB drop as the generator.
- Changing EmpiierOS Runtime, Shell, or installer from that session.
- Claiming EmpiierOS metal / MATCH / OTA apply from this TZ.
- Using the entitlement signing key to sign OTA capsules (wrong F9
  role).

## EmpiierOS already (do not reimplement)

- `ota_capsule_store::plant_from_license` on air bind.
- IPC `ota_artifact_pk=planted|missing` (flags only, no hex).
- Living CP SK at `$XDG_DATA_HOME/empiieros/ota/artifact.sk.hex`.

## Landed (EliteEW tree, 2026-09-21)

Disk + independent `uv run pytest` (13 passed):

- `entitlement_sign.py`: `resolve_ota_artifact_pk`, env then file,
  `^[0-9a-f]{64}$`, `REDACTED`/short omit the key.
- `air_enroll_issuer.py`: `require_fofline_ota_artifact_pk` after
  `try_mint_entitlement`, before `maybe_seal_for_issue`. Named
  `ota_artifact_pk_missing` HTTP 503.
- Staging Config Plane
  `docker/env_file/.env.stack0.backend`:
  `EMPIERO_OS_OTA_ARTIFACT_PK` length 64, hex, equals
  `813cfc81…796f`. No SK in that file.
- Example env is `CHANGE_ME`. Product docs name the field.

## Line A bake (operator, 2026-09-21)

Reported, no `git push`:

| Item | Value |
|------|--------|
| Tag | `prod-baked-20260921-1447` (not on this clone) |
| `deploy_id` | `1789966077824-cyp3ky1gbw` |
| Git | `55c7dc7af` `feat(desktop-client): plant EmpiierOS OTA verify PK in Air entitlements` (on this clone) |
| Also committed (no push) | `3307247d7` OTA edge `:29181`; `c0773ca43` installer wall clock |
| Left uncommitted | `opt/eliteew-ts-installer/build-manifest.json` (installer junk, not product) |

Independent from EmpiierOS (2026-09-21):

```text
curl -sS -o /dev/null -w '%{http_code}' https://app.eliteew.ru/api/health
# 200  body {"ok":true}
curl -sS -o /dev/null -w '%{http_code}' https://app.eliteew.ru/empiieros-ota/health
# 200  body ok
```

That is **not** an Air mint and **not** EmpiierOS glass.

## SOPS → tmpfs → recreate (operator, 2026-09-21 15:46 +10)

Reported from EliteEW. Independent here: SOPS
`prod-core.env.tar.sops` mtime **2026-09-21 15:43**; staging PK
still 64 hex `813cfc81…`; `/api/health` 200. Named docs exist
(`DESKTOP-CLIENT-AIR-ENROLL.md`, pitfall 7b, fixed-error
`2026-09-21-air-enroll-ota-pk-sops-lag.md`).

| Step | Result |
|------|--------|
| Bundle + encrypt | 10 files; no `--plaintext-bundle`; no SK |
| `--validate-only` | PK 64 / `813cfc81` |
| `--yes` | `env_push_ok` `runtime_tmpfs` |
| Recreate `eliteew-api-prod` | health 200; process SET (operator) |

## Remaining (not landed)

1. Seat PK + stage walked (operator: chips green) after FoFline Air
   and USB mux Runtime. EmpiierOS overlay `7590c54c…` now carries the
   cutover/PK locks. Factory `out/` `5979e9cf…` still does not.
2. Edge `current.json` may still be morning `launcher-path-1-living-pk`
   — do not apply it over mux `index-C33yKaFx.js`. Operator how-to:
   [`OTA-OPERATOR-RUNBOOK.md`](OTA-OPERATOR-RUNBOOK.md).
   Do not redeploy stack0.

Do not publish an OTA capsule signed with the new SK until (2).
A stale later SOPS push without the key → FoFline 503 again.

## Measure (EliteEW tree — done)

```text
UV_PROJECT_ENVIRONMENT=../../.venv uv run pytest \
  tests/test_desktop_client_entitlement.py \
  tests/test_desktop_client_air_enroll_issuer.py -q
```

13 passed (re-run 2026-09-21 from EmpiierOS, read-only). FoFline + no
env → `ota_artifact_pk_missing`. FoFline + env →
`payload.ota_artifact_pk` equals `813cfc81…796f`.

Issuer planted **in tree**. EmpiierOS glass / prod-core Air **not**
walked from this closeout.
