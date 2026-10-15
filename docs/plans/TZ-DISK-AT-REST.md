# EmpiierOS — disk at-rest encryption

**Status:** Wave 1 implemented in apply/GUI/defconfig and baked in official
`out/` `5979e9cf…`. Metal walk: LUKS open + PIN `fp_v2`. **Not** MATCH.
**Cause this TZ names:** install wrote a plain `ext4` root. Seat
secrets (`recipient.sk`, opened `capsule_inner/`, `wg_conf`, PIN
hash) lived under `/var/lib/empiieros-runtime` with UNIX modes only.
A stolen disk was a stolen seat. PIN is a Runtime IPC gate, not a
volume key.
**Companion:** [`../../session/OS-INSTALL.md`](../../session/OS-INSTALL.md),
kinship `trusted-substrate/docs/IMAGE-INTEGRITY.md`.

Do not use the 4-digit first-boot PIN as a LUKS passphrase.

## What must not happen

| Wrong move | Why |
|------------|-----|
| 4-digit PIN as LUKS key | 10000 guesses; PIN is already a hash on disk |
| Bake a passphrase into `rootfs.tar.gz` | Image is public SoT; every seat would share the key |
| Encrypt only after Activate | Disk is readable from first boot; keys exist before consume |
| Whole-disk LUKS with no recovery | Lab/field brick when TPM PCR drift |
| Claim Trusted while root/state stay plain | IMAGE-INTEGRITY already forbids that label |
| Skip LUKS when `cryptsetup` is missing | Fail closed. Do not install a plain state slot |

## Target shape

```text
ESP     vfat     /boot/efi     firmware/GRUB (unencrypted)
root    ext4     /             OS image (optional later: dm-verity)
state   LUKS2    /var/lib/empiieros-runtime   2 GiB
```

- **State slot** is the first shippable close of the hole: device
  keys, PIN record, bind, inner entitlement, portal conf, SPA cache.
- **Root** stays readable in wave 1. Binaries are not the seat.
- Unlock **disk** ≠ Unlock **PIN**. cryptsetup opens state at boot;
  the 4-digit PIN still gates Activate in Runtime after the volume is
  mounted.

## Key hierarchy

1. **Volume master key** — LUKS2, created on the target at apply
   time (never in the factory archive).
2. **TPM2 bind** (when `/dev/tpmrm0` or `/dev/tpm0` exists and
   `systemd-cryptenroll` works) — extra slot. Fail closed to
   recovery, not to “skip crypt”.
3. **Recovery secret** — operator chooses at install:
   **auto** (128-bit, eight hex groups
   `xxxx-xxxx-xxxx-xxxx-xxxx-xxxx-xxxx-xxxx`) or **user** (12–128
   characters, not a PIN). Shown **once** on the live session
   (`recovery-reveal`). Never persist plaintext on the target. The
   4-digit PIN is never accepted as this secret.
4. **PIN** — stays `SHA-256(domain|salt|fp|pin)` inside the already
   opened state dir. It does not unwrap LUKS.

No TPM on a box: every boot asks for the recovery secret. Honest,
not a hidden default.

## Installer (wave 1)

Apply after extract proves the target has `cryptsetup`, **before** planting PIN:

1. `cryptsetup luksFormat --type luks2` state with the staged
   recovery (auto from `/dev/urandom`, or the operator secret
   staged on stdin — never argv).
2. Open, `mkfs.ext4`, mount on the target
   `/var/lib/empiieros-runtime`.
3. Write `crypttab` + fstab mapper line + Runtime
   `RequiresMountsFor=` drop-in. No keyfile in the image.
4. Plant PIN / `fp_v2` and `{state}/device_keys/recipient.sk.hex`
   (`0600`, never logged) on the opened volume. PIN is not this key.
5. Reveal recovery + PIN on the live session.
6. Enroll TPM2 if present; log `state tpm=yes|no`.

Live USB stays unencrypted installer media. Ticket + SoT + stamp
unchanged. Official bake 2026-09-20 ships `BR2_PACKAGE_CRYPTSETUP`
and `CONFIG_DM_CRYPT` (`rootfs_sha256=5979e9cf…`). Apply on that
image can format the LUKS2 state slot. Older images still die on
`cryptsetup missing`.

## Boot

- Root is plain; GRUB still uses `root=PARTUUID=` of p2.
- `systemd-cryptsetup` opens `empiieros-state` from crypttab.
- Missing TPM + missing recovery prompt → no Runtime state → no
  consume, no portal. Fail closed.
- After mount, today’s Runtime path is unchanged.

## Measure

- Factory: `scripts/selftest-state-luks.sh` — LUKS2, recovery opens,
  PIN does not, raw mount denied.
- Apply needles in `selftest-installer.sh`. Apply `log` is stderr +
  file so `recovery="$(resolve_state_recovery)"` is secret-only.
- QEMU install (after bake): p3 `LUKS\xba\xbe`; recovery reveal
  consumed. Installed Runtime-ready boot is deferred (no-TPM prompt).
- Do not call MATCH until a fresh metal install of a baked image
  shows `cryptsetup isLuks` on the state slot and a raw mount of that
  partition fails.

## Out of scope here

- Fresh metal install of this bake (operator follows; not MATCH)
- Secure Boot ceremony
- dm-verity root
- Changing PIN length or hash
- Rewriting the live Runtime unit on the current lab seat
