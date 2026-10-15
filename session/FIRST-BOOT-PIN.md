# EmpiierOS — First-boot PIN (FoFline V2)

**Status:** Living contract. Production path: `runtime/src/launcher_pin.rs`.
**Plane:** `session/` — seat / session contracts (L-session).
**Not:** `operator.auth`. **Not:** Tauri. **Not:** `product.open`.
**Host arrangement:** [`SEAT-LAUNCHER.md`](SEAT-LAUNCHER.md).

## Purpose

Unlock a **local 4-digit PIN** bound to the harvested hardware fingerprint
(`fp_v2`) on first boot. Trusted Substrate `eliteew-ts-install-apply` verifies
FoFline install proof **before** `wipefs`, then plants the hash-domain PIN
record and `fp_v2` hardware stamp on the target. The live installer GUI
**must show the four digits** on the Done step and refuse reboot until the
operator types `ЗАПИСАЛ` (live session XKB `us,ru` via Alt+Shift) or
the Latin fallback `ZAPISAL`. Plaintext lives only in
`/run/eliteew-ts-installer/first-boot-pin` on the USB session. The Shell
is unlock-only. Consume / activator stay after first-boot unlock.
`product.open` stays fail-closed.

## Invariants

1. **Four digits only.** Reject any other length or non-digit input (`pin_format`).
2. **Installer plants.** Production `launcher.pin_issue` is `not_available` /
   `installer_issued_only`. The record stores `salt_hex` + SHA-256
   (`empiieros-first-boot-pin-v1|{salt}|{fp}|{pin}`) at
   `{state}/seat/first_boot_pin.json` (`0600`). Plaintext PIN is never
   persisted, logged, or returned on later status.
3. **Hardware bind.** Unlock compares the live `fp_v2` with the issued
   fingerprint. Mismatch is `fingerprint_mismatch`. Apply commits a
   32-hex `/etc/machine-id` on the target **before** hashing — the SoT
   archive ships that file empty, and first-boot systemd would otherwise
   mint a new id and break the bind.
4. **Lockout.** Five failed unlocks set `pin_locked` for 300s. Wrong-PIN
   errors never echo the submitted PIN.
5. **Already issued.** A second plant is `already_issued` and does not mint
   a new PIN.
6. **Fail-closed product.** Unlock does not enable `product.open`.
7. **Not operator auth.** Technician/supervisor default PINs stay out of
   this path and out of the production registry.
8. **Installer proof.** Wipe permission is a FoFline **ticket**
   (`install-proof.json` product-family JSON), not a living seat
   `.ecap` on the USB. Consume is after first-boot unlock. See
   [`OS-INSTALL.md`](OS-INSTALL.md).
9. **Unlock-only Shell.** `FoflineActivator` has no Issue control and never
   calls `launcher.pin_issue`. Air and file consume stay after unlock.
10. **Unlock-gated consume.** Production `launcher.capsule_import` and
    `launcher.air_activate` (including the air helper) call
    `require_unlocked` before any path or WSS work. After a verified open,
    inner entitlement/channel persist in Runtime. `product.open` stays
    fail-closed.
11. **Not the disk key.** The PIN never unwraps LUKS. State recovery is
    a separate auto hex or operator secret shown once at install
    (`recovery-reveal`).
    See [`OS-INSTALL.md`](OS-INSTALL.md) and
    [`../docs/plans/TZ-DISK-AT-REST.md`](../docs/plans/TZ-DISK-AT-REST.md).

## IPC

### `launcher.pin_issue`

**Production:** `not_available` / `installer_issued_only`.

`issue()` is compiled only for tests (installer-equivalent plant). Production
cannot mint a PIN.

### `launcher.pin_unlock`

**Params:** `{ "pin": "NNNN" }`.

**Success:** `{ "unlocked": true, "already": false|true, "detail": "…" }`

### `launcher.status`

Adds `pin_issued`, `pin_unlocked`, `pin_locked`, `fingerprint_bound`.
Never includes the PIN.
