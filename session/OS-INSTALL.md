# EmpiierOS — OS install process (FoFline V2)

**Status:** Living process (updated 2026-09-22). **Not** MATCH / metal acceptance.
**Spine:** V2 locked 2026-09-17 — proof + first-boot PIN on install;
**consume after unlock**, not on the USB.
**Companion:** [`FIRST-BOOT-PIN.md`](FIRST-BOOT-PIN.md),
[`SEAT-LAUNCHER.md`](SEAT-LAUNCHER.md),
[`AIR-HANDSHAKE.md`](AIR-HANDSHAKE.md),
[`SEAT-FOFLINE-CANONS.md`](SEAT-FOFLINE-CANONS.md).

This is the **should** journey. If apply or the live GUI disagrees, the
code is the bug — do not “fix” the journey by putting a living `.ecap`
on the stick.

## What is not on the USB

| Artifact | On live USB? | When it appears |
|----------|--------------|-----------------|
| `rootfs.tar.gz` + `.sha256` | **yes** (SoT) | factory flash |
| `live-installer` stamp | **yes** | `flash-live-installer-usb.sh` |
| FoFline **install ticket** (`install-proof.json`) | **yes** (stamp) | `stamp-live-installer-media.sh` |
| Living Capsule v3 `current.ecap` | **no** | after first-boot unlock, air or file consume |
| PIN plaintext | **no** | installer plants hash only; operator sees PIN once |
| SPA pack | **no** | `launcher.spa_sync` after portal |
| OIDC tokens | **no** | Login after Open is allowed |

The spent lab `{state}/capsules/current.ecap` is a **bound seat**, not
install media. Do not copy it onto the stick. A wipe of that disk means
a **new** Activate (air or file), not a replay of the old bind.

## Factory (command-center)

1. Green EmpiierOS preflight.
2. `CARGO_TARGET_DIR=runtime/target scripts/package-shell-gui-rootfs.sh --build-image`
   (optional `--operator-key` if installed SSH must work).
3. `image_contract_ok`. Kinship `out/rootfs.tar.gz` is the SoT.
4. `pack-install-rootfs-metal.sh` so factory-work `out/` matches BR
   (flash refuses dual SoT).
5. `flash-live-installer-usb.sh /dev/sdX` only when `TRAN=usb` and `RM=1`.
   Pass a real `/dev/sdX`, not a `by-id` path (`${DEV}1` is not `-part1`).
   Stamp strips leftover OTA keys and plants overlay apply (no
   `plant_seat_ota_pk`). Flash refuses `artifact.pk.hex` on ESP or live
   share.
6. USB is a **live installer**, not the installed seat.

Do not USB-flash or run disk-install against a polygon you still need
as SSH glass unless the operator named that wipe.

## Live installer (target machine)

Boot UEFI, Secure Boot off. Stick is `/dev/sda` (or similar) and is
**not selectable** as the install disk.

```text
Welcome
  → Regional (locale / timezone / **manual** wall clock — no public NTP.
     That stamp is the seat SoT. Apply writes
     `/etc/empiieros/install-wall-clock` and plants
     `empiieros-apply-install-clock` so first boot applies it when the
     RTC is implausible. `hwclock --systohc` on the live session is
     belt. timesyncd stays masked. Do not call this “unsynced NTP”.)
  → Network (DHCP on en*; static when no DHCP; Wi-Fi is
     `network-wifi --ssid` on the clicked AP via `empiieros-wifi@`.
     Scan does not auto-pick the strongest AP. Live USB forgets leftover
     wpa/network profiles at session start. `wifi-restore` does not run
     on live-installer media. A leftover or lab `192.168.8.10/30` is
     never persisted as `10-eliteew-ts-install-*.network`. Official SoT
     `empiieros-wifi@.service` still lacks `AF_INET`; apply plants the
     live unit after extract so first-boot wpa can init nl80211.)
  → Disk (internal only; USB excluded)
  → State key (auto hex **or** operator secret; not the PIN)
  → type ERASE
  → apply (clock + IdP-Date runtime readable → proof → wipe →
     ESP+root+LUKS → extract SoT → plant PIN + fp_v2 + recipient.sk +
     install-clock + replace baked runtime **only when extract lacks
     IdP Date**). Official SoT still clamps DPoP `iat` to local+15;
     minute-precision installer time then misses Keycloak
     `[now-25, now+15]` (`oidc_http_400`). A mux SoT that already
     carries IdP Date must stay; live share/bin must not clobber it.
     Missing live runtime refuses **before** wipe.
  → Done shows the four-digit PIN **and** the disk recovery key (type ЗАПИСАЛ or ZAPISAL, then reboot)
  → reboot from disk (remove USB)
```

Disk layout after commit: EFI 512 MiB, root ext4 (remainder minus 2 GiB),
state 2 GiB LUKS2 mounted at `/var/lib/empiieros-runtime`. Apply plants
the first-boot PIN hash and `{state}/device_keys/recipient.sk.hex`
(`0600`) on that opened volume. OTA verify PK is **not** an install
plant — it arrives inside the Air license capsule
(`payload.ota_artifact_pk` on admin-code `launcher.air_activate`).
Never an artifact SK. Production
`device.keys.*` is compiled out; without the recipient SK file
`launcher.air_activate` cannot sign `device_request_v1`. The 4-digit
PIN is not the volume key and is not that recipient SK. See
[`docs/plans/TZ-DISK-AT-REST.md`](../docs/plans/TZ-DISK-AT-REST.md).

Apply **before** `wipefs`:

- SoT `rootfs.tar.gz` sha256 matches the live copy.
- If the wipe gate is on: allowlisted FoFline **ticket**
  (`install-proof.json`). That ticket is install permission, not
  Activate.
- **No** living `.ecap` on the media.

After commit: target has `INSTALL-COMPLETE`, no `live-installer` stamp,
no installer GUI binaries. Host SSH keys are generated on the target
(never baked into the archive).

## First boot (installed seat)

```text
Unlock (4-digit PIN from install, bound to fp_v2)
  → Activate (air EWAE1 **or** file Capsule v3)     ← .ecap / air lives here
  → Portal (split-tunnel wg-portal + host-map)
  → spa_sync (signed pack, not a URL from JS)
  → Login (native OIDC)
  → Open (fofline-spa:// …)
```

Shell never calls `launcher.pin_issue` (`installer_issued_only`).
`product.open` stays fail-closed until unlock + consume + pack + OIDC
session. No `autoOpened`.

Login needs `portal_hostmap=applied` (`auth.eliteew.ru` → `10.88.89.1`).
`launcher.channel_up` must leave that map applied (repairs once if needed).
Without it the Keycloak card stays white while PAR may still succeed by
VIP IP. Air and OIDC also refuse junk RTC years (`wall_clock_required`);
set the seat clock before Activate / Sign-in. Extra VT consoles (tty2–6)
stay masked; operator logs use provisioned key-only SSH, not a local password
VT.

## Installed boot (state LUKS)

After apply, root (p2) is plain ext4; state (p3) is LUKS2
(`empiieros-state` → `/var/lib/empiieros-runtime`). On boot,
`systemd-cryptsetup` requests the recovery secret when TPM does not
unlock. The image ships `empiieros-state-unlock-agent` (MIT console on
`/dev/console`) instead of the stock systemd console ask-password agent.
Wrong or missing recovery → state stays closed → Runtime has no `{state}`
(fail closed). First-boot PIN is after unlock, not the LUKS key.

**Metal walk (after QEMU M5):** flash official kinship `out/` (`TRAN=usb`,
`RM=1` only on the named disk), run a fresh install, reboot installed root.
Expect the branded console prompt (`EmpiierOS`, «ключ восстановления диска»)
on tty1 when apply logged `state tpm=no`. Confirm wrong recovery and Enter on
empty input do **not** mount `/var/lib/empiieros-runtime`; valid recovery
then unlocks and kiosk proceeds. Record apply log + console notes in
`docs/plans/SECURE-RESET-LIVE-DIARY.md`. QEMU smoke is **not** MATCH.

## Apply lock (2026-09-20)

`eliteew-ts-install-apply` `verify_install_proof` accepts only the
canonical ticket JSON (product family). It does **not** look for
`.ecap`. Factory stamp writes that ticket and deletes leftover
`current.ecap` on live media. Apply formats a LUKS2 state slot
(recovery ≠ PIN) and dies if `cryptsetup` is missing. Recovery is
either auto hex or an operator secret staged via
`state-recovery-stage` stdin. Official bake 2026-09-20 ships
cryptsetup (`rootfs_sha256=5979e9cf…` on that date). The archive on
disk on 2026-09-22 is `cf0b46e1…` in both factory `images/` and `out/`.
The live installer binary
must include the State key step before that choice appears on glass.

## Reinstall meaning

The same live USB installs EmpiierOS on **any** allowed internal
disk — not a lab-only path and not bound to `192.168.8.10`. That
address is one glass host, not the product.

A wipe throws away that machine's mux unit debt (live
`-SECCOMP` theater while the unit still carries
`SystemCallFilter`) and its old PIN/bind. It does **not**
move a previous capsule onto the new seat. After PIN, Activate
again on that machine. H6 is measured on a fresh install of this
image, not on mux. Runtime `PrivateTmp=true` stays across
reinstalls — that jail is law, not debt; watch stays
`PrivateTmp=false` (see `docs/plans/TZ-SEAT-SHIELD-H6-PRIVATETMP.md`).
