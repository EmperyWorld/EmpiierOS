# LinkornEW — staff seat profile

**Status:** Scoped 2026-10-03. Source skeleton only. **Not** an image.
**Not** MATCH. **Not** official `out/`. **Not** a mode of the FoFline seat.
**Profile id:** `linkornew`. Display name: **LinkornEW**.

FoFline keeps [`OS-INSTALL.md`](OS-INSTALL.md), [`SEAT-LAUNCHER.md`](SEAT-LAUNCHER.md),
and [`SEAT-FOFLINE-CANONS.md`](SEAT-FOFLINE-CANONS.md). This file does not
change that journey.

## One image, one face

The factory may select `linkornew` or the FoFline payload. It does not
select both. There is no runtime switch on the FoFline image that opens
this face. `scripts/package-shell-gui-rootfs.sh` packs `shell-ui/` only.

## What this profile owns

| Piece | Owner | This slice |
|-------|--------|------------|
| Launcher | `shell-linkorn/` | Static desk: command post, console, browser. No FoFline steps. |
| Install | future staff apply | No EWAE1, no living `.ecap` on the stick, no FoFline PIN gate. |
| Security | Command Post L5 **WIP / Not MATCH** | Plant cpython+deck+controller+sentinel in source; lab E2E register OK. Metal proof missing. |
| Browser | external station-chromium build | Binary stays out of this git. Not `seat.browser`. |
| Console | `linkornew-host console` | Seat-session window. Not a getty on tty2–6. Those VTs stay masked on both images. |

Shared floor: operator wall clock, LUKS state volume when the staff
machine uses one, Runtime peercred socket. FoFline license ops
(`launcher.air_activate`, capsule import, `product.open`) are absent
from the staff dispatch when that binary is built. The ordered work
after this source entry is [`../docs/plans/TZ-LINKORNEW.md`](../docs/plans/TZ-LINKORNEW.md)
(L1 desk, L2 session, L3 console, L4 browser, L5 Command Post,
L6 installer, L7 factory, L8 metal).

## This slice

`runtime/src/linkorn.rs` is the staff dispatch. The FoFline build leaves
`linkorn.*` unregistered (`not_found`). The staff build is:

```text
cargo build --locked --manifest-path runtime/Cargo.toml \
  --no-default-features --features linkornew
```

`production` and `linkornew` together do not compile. On the staff build,
`launcher.*`, `product.open`, and `seat.browser.*` return `staff_profile`.
`linkorn.console`, `linkorn.browser`, and `linkorn.command_post` return a
fixed argv for the seat host. Runtime does not spawn them. A missing
binary is `console_not_installed`, `browser_not_installed`, or
`command_post_not_installed`. Params cannot carry a command.

The seat entry is `linkornew-host` (`console`, `browser`, `command-post`).
It is built only with the staff feature and is absent from
`scripts/package-shell-gui-rootfs.sh`. It execs the checked argv in the
seat session and keeps only seat display/session variables. The desk posts
one word to the seat message handler `linkornew` when that window is
holding the page, and also loads `linkornew://launch/<word>`. A page opened
elsewhere starts nothing. Enter and Space on a focused tile fire the same
word.

## USB boot (Latitude 5511, no NVIDIA)

tty1 autologin still uses `eliteew-ts-session-router`. On profile
`linkornew` it execs `/usr/lib/linkornew/linkornew-session`, not the
FoFline kiosk. That session must set `LIBSEAT_BACKEND=seatd` and
`LABWC_UPDATE_ACTIVATION_ENV=0`. This image has no `systemd-logind`.
A failed `dbus.service` (empty factory `machine-id`) must not leave
labwc waiting on the system bus. Assemble writes a machine-id when the
extracted tree has an empty file.


Target SKU: **Dell Latitude 5511** without the optional MX250.
Dell setup guide: 10th-gen Core i5/i7 H, chipset **WM490**, integrated
**Intel UHD Graphics** (i915; HDMI 2.0 / USB-C DP). MX250 is a different
SKU. Wireless is **Qualcomm QCA61x4A (DW1820)** or **Intel AX201**.
This image does not load nouveau. QEMU is not the metal path.

`scripts/flash-linkornew-usb.sh` writes GPT ESP + ext4 p2. GRUB must pass a
kernel-native root: `root=PARTUUID=<p2> rootwait console=tty1`. A filesystem
`LABEL=` is not a kernel root name; it drops `rootwait` and panics on
`unknown-block(0,0)` while the USB disk is still enumerating. Factory
`grub.cfg` uses `/dev/sda1` on a one-root layout; this stick's root is p2.

Extract `out/rootfs.tar.gz` as root (`tar --numeric-owner`) for USB. A user
extract drops setuid and makes `/var/lib/empiieros-runtime` uid 1000.
`empiieros-runtime` then exits `state_store_open_failed` (owner != euid
0) and systemd prints `Failed to start EmpiierOS privileged runtime`
before getty. Do not `chown -R 0:0` after extract: that clears setuid.
Assemble `--from-tar` plants source Chromium runtime files and refuses a
tree whose dbus helper is not setuid. Flash refuses NVMe and any disk
that is not `TRAN=usb RM=1`. Official `out/` is not written.

Cause of the earlier browser die: chrome DT_NEEDED includes
`libnss3`, `libnssutil3`, `libsmime3`, `libnspr4`, and `libcups.so.2`,
and the factory defconfig did not build them. Staff overlay plants that `out/Default` chrome (pin in
`runtime/linkornew-seat/CHROMIUM.md`) plus ninja `about_credits.html`.

Ping of an address can succeed while named sites fail: `hosts:` is
`resolve [!UNAVAIL=return] files …`. Empty systemd-resolved DNS is
NOTFOUND and does not fall through to `dns`. Staff overlay sets wired
`192.168.1.99/24` gateway/DNS `192.168.1.1` (`10-linkornew-wired.network`
beats factory DHCP `20-wired.network`) and Chromium
`--dns-over-https-mode=off`. Not the polygon `/30`. Not Google DNS.
Not MATCH. Official `out/` is unchanged.

## Out of this slice

- Official `out/` and MATCH
- Command Post LAN firewall (deck stub on 127.0.0.1:4191 only)
- Chromium pack into official `out/` (staff USB overlay is not MATCH)
- Claiming the staff seat is metal-ready after a USB write

