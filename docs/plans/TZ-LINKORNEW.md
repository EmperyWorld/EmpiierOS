# TZ — LinkornEW staff profile

**Status:** Scoped 2026-10-03. Source entry exists. **Not** an image.
**Not** MATCH. **Not** official `out/`.
**Not** a flag inside the FoFline launcher or installer.
**Contract:** [`../../session/LINKORNEW.md`](../../session/LINKORNEW.md).

## Cause

The FoFline face is the license journey (PIN, capsule, portal, sign-in,
`product.open`). A staff seat that starts on that screen has to bypass
those gates. LinkornEW is profile `linkornew`: its own shell, installer,
and security contour.

## Lock (every slice)

- One profile per image. FoFline and `linkornew` are never both selected.
- `shell-ui/src/main.tsx` does not import `shell-linkorn/`.
- `scripts/package-shell-gui-rootfs.sh` does not pack the staff shell,
  `linkornew-host`, or `--features linkornew`.
- `production` and `linkornew` together do not compile.
- tty2–6 stay masked on both images. The console is a seat window.
- Station Chromium and Command Post stay outside this git. No Mega Mix
  tile config. No copy of Command Post `install.sh`. No tokens in git.
- Runtime does not spawn the console, the browser, or a tty. The seat
  entry `linkornew-host` execs only an allowlisted argv.

## Landed in source

| Piece | Where | Proof |
|-------|--------|--------|
| Staff desk, three tiles, no FoFline ops | `shell-linkorn/` | Needles: desk has no `product.open` / `launcher.air_activate` |
| Staff runtime, FoFline face closed | `runtime/src/linkorn.rs`, feature `linkornew` | FoFline build: `linkorn.*` is `not_found`. Staff build: license ops return `staff_profile` |
| Seat window | `runtime/linkornew-seat/linkornew-seat.c` | Tile word is only `console`, `browser`, or `command-post`. Child is `linkornew-host`. Enter/Space and `linkornew://launch/` also fire. QEMU virtio-vga is 1280×720; session forces pixman on QEMU. Not MATCH |
| Console program | `runtime/linkornew-seat/console.c` | Built from this source. Accepts only `--title=LinkornEW`. Staff install path `/usr/lib/linkornew/console`. Distro `foot` is not a candidate |
| Chromium pin | `runtime/linkornew-seat/CHROMIUM.md` | sha256 of the source `out/Default/chrome`. `install` refuses while the license dump is empty. Overlay copies that build's runtime files, not Fedora/AppImage libs. Factory still lacks libnss3 and libcups. Not MATCH |
| Command Post units | `runtime/linkornew-seat/command-post/` | **WIP / Not MATCH.** Plant: cpython+deck+controller+sentinel; lab E2E register OK on loopback. Metal/USB proof outstanding. |
| Staff stage | `runtime/linkornew-seat/install-linkornew.sh` | Writes profile `linkornew`. Refuses a block device and a FoFline PIN file. Does not call the FoFline installer |
| Staff packager | `scripts/package-linkornew.sh` | Only `--profile linkornew`. Refuses a second profile and `out/`. Manifest hash is of the staged tree, not official `out/` |
| Session | `runtime/linkornew-seat/linkornew-session` | tty1 only. labwc autostart is `linkornew-seat`. tty2–6 masks are symlinks to `/dev/null`. Not installed on the FoFline image. A live boot is still ahead |

The desk names `linkornew-host` and does not spawn it. There is no
rootfs, no factory fragment, and no metal seat.

## Plan

Each row is one slice. The next slice starts only after the previous
measure is green. FoFline `cargo test --lib` and
`scripts/preflight-needles.py` stay green on every slice.

| Step | Work | Done when |
|------|------|-----------|
| **L1 Desk → entry** | Landed in source: `linkornew-seat` shows `shell-linkorn/` and spawns `linkornew-host` with one word. | Live click on a built desk inside that window. FoFline `shell-gui` has no LinkornEW tile. |
| **L2 Session** | Landed in source: `linkornew-session` accepts only `/dev/tty1` and labwc autostart runs `linkornew-seat`. tty2–6 mask symlinks live next to it. | A staff image boot shows LinkornEW. Switching VT does not open a shell. |
| **L3 Console program** | Landed in source: `runtime/linkornew-seat/console.c`, built with the seat Makefile. Installed path on the staff image is `/usr/lib/linkornew/console`. No distro terminal binary. | `linkornew-host console` opens that window. Missing binary still returns `console_not_installed`. |
| **L4 Browser** | Pin matches live `station-chromium` `out/Default/chrome`. Overlay plants source chrome plus from-source NSS 3.101 / CUPS 2.4.11 (not Fedora blobs). Factory defconfig still omits those packages. | Image file `/usr/lib/station-chromium/chrome.bin` plus `/usr/lib/libnss3.so`. Egress is still L5. Not MATCH. |
| **L5 Command Post** | **Not MATCH.** Source plant: `plant-cpython` + deck/controller/sentinel + run units. Lab E2E: Deck `:4191`, controller `:8088`, sentinel `:8090` register. Assemble hook via `LINKORNEW_COMMAND_POST`. | Done when staff image/USB boots with live Deck via `linkornew-host command-post` and tokens out of git. |
| **L6 Installer** | Landed in source: `install-linkornew.sh stage <dest>` writes `etc/empiieros/seat-profile` = `linkornew`. It refuses block devices, the git repo, and a tree that already has a FoFline PIN file. | A staff stick uses only this apply. A FoFline stick does not offer LinkornEW. |
| **L7 Factory** | Landed in source: `scripts/package-linkornew.sh` accepts only `--profile linkornew`. A second profile, `--build-image`, a git path, and `out/` are refused. It does not call the FoFline packager. | Official FoFline `out/` and a staff rootfs remain two factory artifacts. This script stages a tree and its manifest hash; it does not write `out/`. |
| **L8 Metal** | USB: root `--from-tar` then `scripts/flash-linkornew-usb.sh` on `TRAN=usb RM=1` only. Target SKU: Dell Latitude 5511 without MX250 (Intel UHD / i915). QEMU is not this measure. | Desk on that laptop. Official `out/` unchanged. Not MATCH. |

## Explicitly later than L8

- Calling the staff image metal-ready or replacing official `out/`.
- A runtime switch on a FoFline seat.
- Public NTP, a second TTY console, or a general desktop.

## Measure already in force

`scripts/preflight-needles.py` keeps the FoFline entry, packager, and
`linkornew-host` feature gate apart. Image and metal proof do not exist
yet.
