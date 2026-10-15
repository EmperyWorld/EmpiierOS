# EmpiierOS — Air WSS Handshake Execution Contract (P64)

**Status:** Living contract (P64), production path updated 2026-09-18.
**Plane:** `session/` — seat / session contracts (L-session).
**Runtime Module:** `runtime/src/launcher_air.rs` (production). Historical `runtime/src/air_enroll.rs` is not the metal SoT.

## Purpose

Formalize the execution of the versioned Air enrollment one-code exchange (`EWAE1`) over an RFC 6455 WebSocket connection. This allows delivery of Capsule v3 over the network without local USB file interaction, while maintaining cryptographic verification, fail-closed boundaries, and zero secret leakage.

## Invariants

1. **Strict Port Envelope:** Port must strictly fall in `62200..=63200`. Ports outside this range are rejected immediately prior to network calls.
2. **Transcript Checksum:** The access code is validated via Crockford Base32 decoding and constant-time SHA-256 transcript verification (`EWAE1|{port:05}|{selector}|{secret}`).
3. **Secret Protection:** Secrets (the 128-bit code secret and recipient secret key) are strictly zeroized on Drop and must never appear in logs, files, or IPC responses.
4. **Digest Verification:** The SHA-256 hash of the received binary capsule is verified against `capsule_meta.sha256` before opening. Mismatches abort and signal `import_nack`.
5. **Atomic State Storage:** On successful capsule open, `{state}/capsule_bind/bound.json` is created with permissions `0600`, recording redacted IDs. Inner entitlement/channel persist under `{state}/capsule_inner/` (`0600`); the sealed blob is `{state}/capsules/current.ecap`. IPC sees flags only.
6. **Fail-Closed Networking:** If the edge controller is unreachable, returns `status: unreachable`, `ok: false`, `code: air_ws_connect_unreachable`. Never fake a successful delivery without valid capsule verification.
7. **Living issuer PK:** Verify Capsule v3 against the EliteEW issuer public key (env / bake / non-DEV state file / baked `26b36dae…`). A leftover lab DEV `signing.pk.hex` must not shadow prod in release.
8. **Activation token:** Read the one-time token from inner `payload.activation_token` (living seal). Persist bind + inner **before** WSS `activation_ack`. Token never enters Shell IPC. A helper that ACKs without `{state}/capsule_inner/flags.json` is `air_helper_inner_missing`.
9. **Helper/Runtime pair:** Mux `empiieros-air-enroll` with the Runtime that owns persist. A newer Runtime in front of a stale helper is a false green.
10. **Recipient SK before WSS:** `{state}/device_keys/recipient.sk.hex` is planted on the opened LUKS volume at apply. Runtime creates the file if it is absent. An existing invalid file is `device_keys_invalid`. Production `device.keys.*` stays compiled out. Never log the SK.
11. **Sane wall clock:** Local civil year must be 2024–2100 before air starts (`wall_clock_required`). Junk RTC is not treated as a network failure. Operator sets the seat with `time.sync` `action=set`. See [`SEAT-FOFLINE-CANONS.md`](SEAT-FOFLINE-CANONS.md) §1.
12. **OTA verify PK with the license:** FoFline Air entitlement payload
    carries `ota_artifact_pk` (32-byte hex). Runtime plants
    `{state}/ota/artifact.pk.hex` on bind. Never an artifact SK. Not a
    USB sidecar. Issuer lock:
    [`../docs/plans/ELITEEW-TZ-OTA-PK-IN-AIR-LICENSE.md`](../docs/plans/ELITEEW-TZ-OTA-PK-IN-AIR-LICENSE.md)
    (issuer + SOPS/env-push/recreate 2026-09-21; FoFline Air mint
    not walked).

## IPC Operations

### `launcher.air_activate` (production)

Executes the Air WSS handshake through allowlisted `empiieros-air-enroll.service`. Historical `POST /v0/enroll/air-execute` is not the metal SoT.

**Request body:**
```json
{
  "code": "EWAE1-62555-GY0BVC277PJ1H8PV-5QWG9B35VYHVXRR6XHSD1TH7YM-8VSE"
}
```

**Response (Success):**
```json
{
  "ok": true,
  "status": "ok",
  "code": "success",
  "selector_redacted": "GY0B...H8PV",
  "request_id_redacted": "dr1_7b8d...2fdb",
  "capsule_sha256": "4a5c99d2...3455a4f0",
  "bound": true,
  "detail": "Air WSS handshake completed; Capsule v3 verified and bound"
}
```

**Response (Unreachable Host / Network Error):**
```json
{
  "ok": false,
  "status": "unreachable",
  "code": "air_ws_connect_unreachable",
  "selector_redacted": "GY0B...H8PV",
  "request_id_redacted": null,
  "capsule_sha256": null,
  "bound": false,
  "detail": "WSS endpoint app.eliteew.ru:62555 unreachable: connection refused"
}
```

**Response (Syntax or Checksum Error):**
```json
{
  "ok": false,
  "status": "invalid",
  "code": "air_code_checksum",
  "selector_redacted": null,
  "request_id_redacted": null,
  "capsule_sha256": null,
  "bound": false,
  "detail": "Air code checksum mismatch"
}
```

## Non-Goals

- Remote server firewall / security group management
- Arbitrary URL / port connections outside `62200..=63200`
- Returning raw capsule payloads or secret keys over IPC
- Permitting HostedWeb callers (denied with 403)
- FoFline SPA **bootstrap** pack (`launcher.spa_sync`, see [`SPA-PACK.md`](SPA-PACK.md)).
  Capsule-gated OTA is separate ([`OTA-CAPSULE.md`](OTA-CAPSULE.md)).
  That is a later portal fetch, not this air Capsule (≤256 KiB).
- Starting WireGuard from Runtime itself. Portal bring-up is
  `launcher.channel_up` / `empiieros-portal-up.service` (see
  [`PORTAL-TUNNEL.md`](PORTAL-TUNNEL.md)).
