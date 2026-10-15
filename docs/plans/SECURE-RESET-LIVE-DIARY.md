# Secure-reset live diary

**Status:** Chronological laboratory log. **Not** acceptance authority.
**Verdict:** `NOT_READY`.
**Authority:** [`MATURITY-LEDGER.md`](MATURITY-LEDGER.md) · [`../FOUNDATION-STATUS.md`](../FOUNDATION-STATUS.md).
**Companion snapshot:** [`METAL-LIVE-BASELINE.md`](METAL-LIVE-BASELINE.md).

This file keeps the 2026-09-17 returned-metal and live-seat narrative that used
to live inside `docs/FOUNDATION-STATUS.md`. Later hashes and QEMU results in
the living status document supersede older rows here when they conflict.

## Returned-metal evidence — 2026-09-17

The latest returned installer USB was mounted read-only. Its logs prove archive
hash verification, disk identity pinning, partitioning, extraction, SSH host-key
generation, GRUB installation, network-profile transfer, regional persistence,
target-contract verification, and ordered reboot. The first installed boot then
exposed `peer_executable_unavailable`: Runtime ran as root with only
`CAP_NET_ADMIN`, while Linux requires ptrace access for `/proc/<pid>/exe` when
the trusted Shell has another UID.

A temporary recovery drop-in added `CAP_SYS_PTRACE` only to prove the identity
check. That capability is no longer the source contract. Live SSH evidence now
shows a root-owned `systemd-socket-proxyd` bridge, backend socket `0600
root:root`, frontend socket `0600 eliteew-ts`, Runtime `CapEff=0000000000001000`
(`CAP_NET_ADMIN` only), `NRestarts=0`, and repeated authorized
`system.snapshot` responses. Split `empiieros-maint` SSH identity, WebKit
sandbox, and systemd `+SECCOMP` are locked in source and require the next image.
Live SSH then proved the boot/network causes. The persisted Ethernet profile
kept `RequiredForOnline=no`, so wait-online ignored the working IPv4 link and
timed out after two minutes. Rewriting that profile to `RequiredForOnline=yes`
and installing `--any --ipv4 --timeout=30` made wait-online succeed in under one
second; restoring `RequiredForOnline=no` timed out again. `ath10k_pci` is
built-in on this image and failed firmware load at 2.4s even though
`ath10k/QCA6174/hw3.0/firmware-6.bin` was already on rootfs; PCI rebind loaded
the firmware, created `wlp58s0`, and both `iw` and Runtime `network.wifi.scan`
through the frontend socket observed nearby access points. The next
image must persist `RequiredForOnline=yes`, ship modular ath10k, and include
userspace `rfkill`.

A later live Shell reload proved the GUI trust check: `index.html` owned by
`eliteew-ts` made `empiier://shell/` fail closed (`Shell asset is not trusted`),
the kiosk app loop hit `MAX_APP_RESTARTS`, and snapshots stopped. Restoring
root-owned `0644` assets plus a `getty@tty1` seat restart brought the GUI back.
Authorized `system.snapshot` then showed `sit0.kind=other`, `eno1` ethernet,
`wlp58s0` wifi, `overall_status=degraded`, `locale=ru_RU.UTF-8`, and observed
display/service/storage/input/audio domains. `pgrep -x empiieros-shell-gui`
never matches on this kernel (`comm` is 15 characters); the live harness now
matches the full path. F11/windowed and wheel scroll are in the deployed GUI
but have not been watched on the physical panel. Wi-Fi connect/restore, GPU,
power soak, and a fresh-image install remain open.

Live-proven fixes from this host are now in EmpiierOS and Trusted Substrate
source, not only on the debug machine: frontend socket + proxyd peer, wait-online
contract, modular ath10k lock, typed snapshot/events, `sit0.kind=other`,
root-owned Shell assets, and the live harness `pgrep -f` / BusyBox asset
replace path. This slice also locked confirm-gated `system.power` through
allowlisted isolate units. A frontend `system.power` request with the wrong
confirm returned `confirmation_does_not_match_action` and left both power units
inactive. `ProtectSystem=strict` made `/proc/mounts` look like a read-only root
inside Runtime; the unit now bind-mounts `/proc/1/mounts` to
`/run/empiieros/host-mounts`, and live snapshot reports `root_read_only=false`
plus `root_not_verified_read_only`. The same snapshot observed
`intel_backlight` 120000/120000, ALSA `HDA Intel PCH`, named input nodes, and
`ntp_synchronized=true`. A leftover `empiieros-wifi@wlp58s0` failure was caused
by a Runtime config this image cannot parse (`ctrl_interface=` and `SAE`); the
renderer now writes `WPA-PSK` only, the failed unit was reset, and the next
image enables WPA3/ctrl-iface. A live `empiieros-wifi@wlp58s0` stop dropped Wi-Fi while lab Ethernet stayed
up; `empiieros-wifi-restore` reassociated `EWKaska` without printing the PSK.
A later reboot returned SSH on `eno1` and wait-online succeeded, but Wi-Fi did
not: builtin `ath10k_pci` failed firmware at 2.5s and restore skipped
`wlp58s0`. PCI rebind recovered the radio. Runtime `system.watch` plus
persisted events and first-class snapshot freshness are deployed; DOM
diagnostics stay off without `EMPIIEROS_SHELL_MAINTENANCE=1`. Positive
poweroff, GPU glass checks, and a fresh-image install remain open.
Live Wi-Fi is associated to `EWKaska` on `wlp58s0` with `192.168.38.82/24` while
lab Ethernet `eno1` remains `192.168.8.10/30`. Snapshot now includes
`wifi.associations`. The Shell was rebuilt on EliteEW Desktop tokens and shared
controls, then redeployed with root-owned assets; GUI is up. Glass comparison of
the new grouped Network workspace is still operator-visible. A later
frontend capability slice (Runtime `20260917T025031Z-runtime-14174`) proved
confirm-gated `system.power` denials, production-prefix `not_available`,
malformed/oversize/unauthorized-peer rejects, `network.wifi.scan`, and
`network.configure` negatives without touching `eno1`. A later Runtime
(`20260917T030531Z-runtime-18673`, `f3bb25d6…`) owns a claimed Wi-Fi link
by rewriting the already-bound `20-wireless.network` in place: frontend
DHCP `wlp58s0` applied, lease `192.168.38.82/24` kept, `eno1`
`192.168.8.10/30` unflapped, no `05-` leftover. Wheel/touch now have a
production GTK syslog (`shell scroll observed` in GUI `55b84266…`); evdev
could not generate that line (no `REL_WHEEL` device, no `uinput`).
Next-image source locks for libseccomp/systemd-analyze, WebKit bubblewrap
helpers, modular ath10k + rfkill, and wired `RequiredForOnline=yes` are
in fragments, generated defconfig, allowlist, selftest, and full-rootfs
post-build. They are not live-true on this seat. Current-source overlay
`c910e0f7…` was baked: resume `buildroot-build.sh --resume` exit 0 at
2026-09-17T04:13:21Z into `trusted-substrate/out/` (`rootfs_sha256=5a125dfa…`,
bzImage 14:11). Factory userspace now includes systemd-analyze, bwrap,
xdg-dbus-proxy, rfkill, libseccomp; kernel has modular ath10k and
`CONFIG_BRCMFMAC_PCIE=y`. QEMU of the first current-source `out/` (`5a125dfa…`) exited 1: systemd
deleted `empiieros-shell-bridge.socket` to break
`sockets.target → socket After=runtime → runtime After=basic.target →
sockets.target`. Overlay + selftest + post-build now forbid that `After=`.
Resume assemble (no WebKit rebuild) wrote official `out/`
`rootfs_sha256=106a4ce9…` (14:22–14:23) with the fixed socket.
`ELITEEW_TS_SECURE_RESET_QEMU=1 bash scripts/selftest-secure-reset.sh`
against those artifacts exited 0: direct-boot and installer→installed-disk
smokes both produced authorized Shell snapshots; no ordering cycle.
Live SSH then replaced Runtime with
`20260917T043308Z-runtime-20737` (`a1cc73c0…`). Journal on 192.168.8.10:
`seccomp mode=0 filters=0` under systemd `-SECCOMP`. The same seat then
applied frontend `network.configure` static `wlp58s0` `192.168.38.82/24`
via `20-wireless.network`, restored DHCP, and rejected an out-of-subnet
gateway; `eno1` stayed `192.168.8.10/30`. The same seat then applied
`display.control` `75` → sysfs `90000` and restored `100` → `120000`
through `empiieros-display-brightness.service` (Runtime
`20260917T044033Z-runtime-30719`, `ba7067af…`). Frontend
`audio.control` then set ALSA selem `Speaker` to 25%
(`observed_percent=24`) and `aud-100` restored codec
`[0x3c 0x3c]` (Runtime `20260917T044644Z-runtime-6347`,
`cf3cc043…`). Live SSH then created locked `empiieros-maint` `106:114`
with no `.ssh` and did not change `AllowUsers` or sudoers. The same
seat then applied frontend `network.wifi.disconnect` and
`network.wifi.reconnect` (Runtime `20260917T045237Z-runtime-15789`,
`24e8d8d6…`): Wi-Fi dropped, then `EWKaska` `192.168.38.82/24`
returned from the persisted `0600` file; `eno1` stayed
`192.168.8.10/30`; a `psk` field on reconnect is `invalid_params`.
The same seat then applied `network.wifi.radio` (Runtime
`20260917T045753Z-runtime-1352`, `807a2ed2…`): snapshot reports
`phy1` WLAN rfkill; `radio-block-1` set sysfs `soft=1` and dropped
Wi-Fi; `radio-un-1` plus reconnect restored `EWKaska`; `eno1` stayed
`192.168.8.10/30`. There is still no `/usr/sbin/rfkill` on this image.
The same seat then deployed Runtime `20260917T050224Z-runtime-32422`
(`cbfa5f98…`) and Shell `20260917T050240Z-shell-12122`
(`index-L42cbaJL.js`). Frontend `stor-snap-1` listed `nvme0n1` /
`nvme0n1p1` / `nvme0n1p2` with the same byte sizes as
`/sys/class/block`. After `kill -9`, Runtime returned in 4s
(`NRestarts=1`), persisted `start_count` went `1` → `2`, frontend
`shell-*` snapshots resumed, and `eno1` stayed `192.168.8.10/30`.
Live journal is still `seccomp mode=0 filters=0`. The same seat then
deployed Runtime `20260917T050831Z-runtime-27614` (`0d8bcda1…`) and
Shell `20260917T050843Z-shell-31126` (`index-MZG9_8I3.js`) with
`empiieros-time-sync`. Frontend `time-sync-1` applied NTP sync
(`162.159.200.1`) without changing `Asia/Vladivostok` or
`ru_RU.UTF-8`; `eno1` stayed `192.168.8.10/30`. `time.action` and
`set_timezone` stay fail-closed. The same seat then deployed Runtime
`20260917T051118Z-runtime-10930` (`5276bad3…`) and Shell
`20260917T051130Z-shell-10375` (`index-BjXv33gt.js`). Frontend
`therm-snap-1` listed `AC`/`BAT0` and thermal zones `acpitz` 25 °C /
`x86_pkg_temp` 45 °C against `/sys/class/thermal`; `eno1` stayed
`192.168.8.10/30`. The same seat then deployed Runtime
`20260917T051330Z-runtime-29086` (`2e3e05d6…`) and Shell
`20260917T051352Z-shell-30109` (`index-C4jfk3-A.js`). Frontend
`input-snap-1` matched sysfs: 10 event nodes, `event9` mouse
`rel=3` with no `REL_WHEEL`, no `/dev/uinput`. The same seat then
deployed Runtime `20260917T052027Z-runtime-18842` (`53438625…`) and
Shell `20260917T052039Z-shell-29178` (`index-kgycIST0.js`). Frontend
`id-snap-1` reports kiosk+maint users, `AllowUsers=eliteew-ts`,
password/root SSH off, leftover sudoers present (file not read), and
no hardware watchdog. Official `out/` `648dbb33…` / bzImage `04fab7e1…` then passed
`qemu-empiieros-smoke.sh` and `ELITEEW_TS_SECURE_RESET_QEMU=1`
selftest exit 0: systemd `+SECCOMP`, Runtime `seccomp mode=2
filters=28`, `WebKit sandbox enabled`, authorized snapshots,
installer→installed-disk PASS, no ordering cycle. That image Runtime
is `a1cc73c0…` and lacks later live helpers. Live 192.168.8.10 is
still `seccomp mode=0 filters=0`. The same seat then deployed Runtime
`20260917T053812Z-runtime-32723` (`68f60068…`) and Shell
`20260917T053829Z-shell-3779` (`index-tqmGlV_c.js`). Frontend
`drm-snap-1` matched sysfs DRM: `card0-eDP-1` connected/enabled/On
`1920x1080`. The same seat then deployed Runtime
`20260917T054051Z-runtime-31170` (`fb38d2bc…`) and Shell
`20260917T054105Z-shell-27707` (`index-BrgJN3PW.js`). Frontend
`usb-snap-1` matched sysfs USB device nodes `1-11`/`1-14`/`2-2`.
`display.action` stays `not_available`. The same seat then deployed
Runtime `20260917T054421Z-runtime-26953` (`ac01be9d…`) and Shell
`20260917T054435Z-shell-3233` (`index-Bv8VeR5X.js`). Frontend
`host-svc-1` matched `systemctl` ActiveState, including
`empiieros-wifi-restore.service` `failed`, plus host CPU/memory/load
and GPU `i915` `8086:9bc4`. A later live helper
`empiieros-wifi-restore` `91714fa4…` rebound `ath10k_pci`
`0000:3a:00.0` after a controlled reboot; `Result=success` and
`wlp58s0` returned `EWKaska` `192.168.38.82/24`. `eno1` stayed
`192.168.8.10/30`. The same seat then deployed Runtime
`20260917T055316Z-runtime-27248` (`066b739c…`), helper `bacfcf12…`,
and Shell `20260917T055355Z-shell-19047` (`index-CLfsitsQ.js`).
Restore now waits for IPv4/SSID before success; live unbind+restore
returned `associated wlp58s0` in 21s with the lease already present.
Frontend `svc-result-2` reports wifi-restore `inactive`/`success`/`ran`.
The same seat then deployed Runtime `20260917T060152Z-runtime-5813`
(`83f1770b…`) and Shell `20260917T060122Z-shell-4950`
(`index-BiTgFJBc.js`). Frontend `link-truth-2` MATCH: `eno1`
`1000`/`full` and `wlp58s0` `EWKaska` `5260` MHz. Official
`--resume` then published `out/rootfs.ext2` `1d50ad39…` with image
Runtime `32605369…` and helper `bacfcf12…`. `qemu-empiieros-smoke.sh` and
`ELITEEW_TS_SECURE_RESET_QEMU=1` selftest both exit 0 on that image:
`seccomp mode=2 filters=28`, systemd `+SECCOMP`, WebKit sandbox,
installer→installed-disk PASS, no ordering cycle. Live 192.168.8.10
is still `mode=0`. After a lab reboot the SSH mux died; live
`sshd_config` had no `Include`, so the security drop-in was ignored
until `Include` was appended and sshd reloaded. Frontend `ssh-id-1`
now reads `sshd -T` (`passwordauthentication no`). After the reboot
inspection, Include is still on disk and password-only BatchMode is
`Permission denied (publickey)` for kiosk/root/maint; Include itself
is not reboot-proven. The same seat then deployed Runtime
`20260917T061643Z-runtime-10999` (`e7436007…`) and Shell
`20260917T061700Z-shell-4592` (`index-z7HVixhD.js`). Frontend
`stor-snap-1` MATCH vs `df -B1 /` (`250375106560` /
`991756288` / `236590460928`) and wait-online
`active`/`success`/`ran`. The same seat then had a USB
`YSPRINGTECH` mouse on `event9` (`rel=903`); injecting `REL_WHEEL`
produced GTK `shell scroll observed dy=1,500`. Password-only SSH
exits `255` (`Permission denied (publickey)`). A later identity
deploy that bound the missing maint keys path failed `NAMESPACE`
and rolled back; the recovered unit punches only the existing kiosk
`authorized_keys` under `ProtectHome=tmpfs`. Frontend `id-snap-2`
MATCH vs `sshd -T` (`authenticationmethods publickey`) and key
presence (kiosk yes, maint no) without key bytes. Runtime
`20260917T062645Z-runtime-13017` (`6ed6b0a4…`), Shell
`20260917T062248Z-shell-25756` (`index-DEAuA745.js`). The same seat
then deployed helper `37997f51…` and Runtime
`20260917T062954Z-runtime-13981` (`d19a1d10…`) / Shell
`20260917T062958Z-shell-112` (`index-WMtr0NoQ.js`). Frontend
`aud-obs-1` MATCH `Speaker` `100` via `--observe` without changing
codec Amp-Out; `aud-25c`/`aud-obs-2` MATCH `24`; `aud-100c` restored
`100`. The same seat then deployed Runtime
`20260917T063223Z-runtime-30511` (`3b8237d2…`) / Shell
`20260917T063237Z-shell-30207` (`index-1FZim13b.js`). Frontend
`ntp-obs-1` MATCH last-sync stamp `1789625790`, stratum `3`, server
`162.159.200.1`. `time-sync-2` advanced the stamp to `1789626773`
without changing timezone/`LANG`. The same seat then deployed Runtime
`20260917T063505Z-runtime-25019` (`3818a482…`) / Shell
`20260917T063520Z-shell-17533` (`index-CKS56ATC.js`). Frontend
`bat-obs-1` MATCH BAT0 sysfs charge/voltage/current and
`BYD`/`DELL 3PCVM0A`/`Li-poly`. After the 16:08 reboot, live SSH
still has mux pid `4066109`, `sshd` `active`, `Include` on this
booted seat, and password BatchMode `255` `Permission denied
(publickey)` for kiosk/root/maint. The same seat then deployed
Runtime `20260917T063847Z-runtime-2204` (`960314a9…`) / Shell
`20260917T063900Z-shell-13692` (`index-ItFBB6R7.js`). Frontend
`ipv6-obs-1` MATCH `ip -6` link-local on `eno1` and `wlp58s0`.
The same seat then deployed Runtime
`20260917T064247Z-runtime-24875` (`ef1885cc…`) / Shell
`20260917T064439Z-shell-28237` (`index-DhBwvXn8.js`). Frontend
`dns-src-1` MATCH `dns_source=systemd-resolved-uplink` and
nameservers `192.168.8.9`/`1.1.1.1`/`8.8.8.8`;
`systemd-resolved.service` is `active`/`success`/`ran`.
The same seat then deployed Runtime
`20260917T065021Z-runtime-28708` (`2c88dc43…`) / Shell
`20260917T065043Z-shell-1864` (`index-D58n0fTB.js`). Frontend
`host-proc-1` MATCH `/proc` `MemTotal=32445556`, `SwapTotal=0`,
load `0.10`/`0.08`/`0.04`. The same seat then deployed Runtime
`20260917T065224Z-runtime-25816` (`5fd55025…`) / Shell
`20260917T065241Z-shell-13014` (`index-B2GONB8L.js`). Frontend
`svc-nr-1` MATCH `NRestarts=0` on allowlisted services and `null`
on `empiieros-shell-bridge.socket`. `eno1` stayed
`192.168.8.10/30`. Sudoers left in place. This is not metal
acceptance and not a USB install. Verdict stays `NOT_READY`.

The same seat then received Runtime `20260917T101505Z-runtime-7332`
(`e841d8f9…`) and Shell `20260917T101533Z-shell-7076`
(`index-BQkKLESC.js`). Frontend `prod-cap-1` lists `product.open`
`unavailable` / `not_implemented`. Frontend `prod-open-1` returns
`not_available` / `capability_not_available`. `eno1` stayed
`192.168.8.10/30`. The EliteEW workspace is a host surface, not a
Desktop seed.

The same seat then received Shell `20260917T103215Z-shell-7864`
(`index-CiqUXXU1.js`, GUI still `55b84266…`). The EliteEW workspace now
renders the FoFline `fofline-connected` catalog and local tiles. Live
frontend `prod-cap-1` still lists `product.open` `unavailable` /
`not_implemented`. Frontend `prod-open-1` still returns `not_available`
/ `capability_not_available`. `eno1` stayed `192.168.8.10/30`. This is
not a signed SPA pack, not `edition.profile=fofline`, and not air-gap.

The same seat then received Runtime `20260917T110526Z-runtime-7149`
(`a6f1016d…`) and Shell `20260917T110541Z-shell-1939`
(`index-DW7ADoH2.js`). Frontend `launch-1` observes
`launcher.status` `vehicle=empiieros-shell-gui` `tauri=false`
`fingerprint=fp_v2_822e4d9b…` `pin_issued=false`
`capsule_imported=false` `product_open=unavailable`. Frontend
`prod-open-1` still returns `not_available` /
`capability_not_available`. `eno1` stayed `192.168.8.10/30`. SSH was
not changed. This is V2 observe-only, not capsule import and not an
installer gate.

The same seat then received Runtime `20260917T111403Z-runtime-29068`
(`c6e98436…`) and Shell `20260917T111416Z-shell-27602`
(`index-BEYlxg0J.js`, GUI still `55b84266…`). Lab Capsule v3 material
was planted under `/var/lib/empiieros-runtime/` (not `/home`;
`ProtectHome=tmpfs`). Frontend `launch-pre` still had
`capsule_imported=false`. `deny-passwd` and `deny-dotdot` returned
`path_not_allowed`. Audit recorded `device_keys_missing` before the
plant. Frontend `import-1` applied
`launcher.capsule_import` on
`/var/lib/empiieros-runtime/capsules/current.ecap`: `imported=true`
`bound=true` `product_id=eliteew-desktop-connected` `bytes=397`
`request_id_redacted=rid_075b13365de2`. Payload kind and activation
token were absent from the IPC body, `bound.json` (`0600`), and
`events.jsonl`. Frontend `launch-post` then observed
`capsule_imported=true` `pin_issued=false` `product_open=unavailable`
`tauri=false` `fingerprint=fp_v2_822e4d9b…`. Frontend `prod-open-1`
still returns `not_available` / `capability_not_available`. `eno1`
stayed `192.168.8.10/30`. SSH mux stayed up. This is not a signed SPA
pack, not PIN issue, and not an installer gate. The same seat then
received honesty Runtime `20260917T112449Z-runtime-15325`
(`1a8ddea2…`) and Shell `20260917T112502Z-shell-1374`
(`index-CfdUNNcq.js`). Bind survived the restart:
`launch-honest` still `capsule_imported=true`; snapshot lists
`launcher.capsule_import` `available` and `product.open`
`unavailable`. Frontend `prod-open-2` still `not_available`.

The same seat then received Runtime `20260917T113310Z-runtime-14001`
(`162f535a…`) and Shell `20260917T113423Z-shell-13414`
(`index-DBniTxHc.js`). Frontend `air-bad-1` returns
`air_code_structure`. Frontend `air-cs-1` returns
`air_code_checksum`. Frontend `air-tls-1` with a valid EWAE1 vector
returns `air_ws_tls_required` for the baked
`wss://app.eliteew.ru:{port}/api/v1/desktop-client/air-enroll/ws`
path and does not bind. The prior file bind stayed
`capsule_imported=true`. Snapshot lists `launcher.air_activate`
`available`. Frontend `prod-open-3` still `not_available`. `eno1`
stayed `192.168.8.10/30`. The access-code secret was absent from IPC.
This is not a live WSS delivery and not `product.open`.

The same seat then received Runtime `20260917T120247Z-runtime-5629`
(`d9f1e50a…`), helper `empiieros-air-enroll`
(`40a1da1a…`), and Shell `20260917T120300Z-shell-8730`
(`index-B7g3NjRR.js`, GUI `aeb9bdfe…`). Frontend `air-bad-1`
returns `air_code_structure`. Frontend `air-cs-1` with a
non-canonical Crockford body returns `crockford_noncanonical`.
Frontend `air-wss-1` with the known parse vector no longer returns
`air_ws_tls_required`: the allowlisted helper started and the living
baked WSS connect failed closed as
`air_ws_connect:Connection refused (os error 111)` (port `62555` is
not a live issued session). Request file was unlinked. Result JSON
had no secret. Prior file bind stayed `capsule_imported=true`.
Frontend `prod-open-4` still `not_available`. Runtime unit still has
`IPAddressDeny=any`. `eno1` stayed `192.168.8.10/30`. SSH mux stayed
up. This is not a successful air consume and not `product.open`.

The same seat later consumed an issued admin EWAE1 on port `62917`
(2026-09-17T13:32:13Z UTC). Helper `1a7e9a2a…` opened the living
inner `payload.activation_token` against issuer verifying PK
`26b36dae…`. IPC `launcher.air_activate` returned `ok=true`
`activated=true` `source=air` `product_id=eliteew-fofline`
`bytes=2469`. Prod session `lifecycle=completed` with
`activation_acked_at` set; portal peer `air-1932cfe8…`
`edge_sync_status=synced`. Secret, payload, and token were absent
from IPC. `product.open` stayed `unavailable`. `IPAddressDeny=any`
and `eno1` `192.168.8.10/30` stayed. This is not PIN issue, not an
installer gate, and not `product.open`.

The same generator is now in tree and on the seat: Runtime
`20260917T133732Z-runtime-20733` (`1b5fcdc1…`), helper
`cc3d2a9b…` with baked `EMPIIEROS_CAPSULE_SIGNING_PK=26b36dae…`,
Shell `20260917T133742Z-shell-9354` (`index-DYhNBOzs.js`).
Frontend `launch-lock-1` observes `capsule_imported=true`
`product_id=eliteew-fofline` `capsule_source=air`
`product_open=unavailable`. `metal-live.sh deploy-helper` is the
install path. `IPAddressDeny=any` and `eno1` `192.168.8.10/30`
stayed.

The same seat then received Runtime `20260917T134602Z-runtime-13068`
(`183ed7ce…`) and Shell `20260917T134611Z-shell-4797`
(`index-Cj6Gjhru.js`). Frontend `pin-issue-1` issued a local 4-digit
first-boot PIN bound to live `fp_v2`; the PIN was returned once and
is absent from `seat/first_boot_pin.json` (`0600`, hash only).
Frontend `pin-bad-1` returned `invalid_pin` without echoing the PIN.
Frontend `pin-unlock-1` applied `unlocked=true`. Frontend `pin-post`
observes `pin_issued=true` `pin_unlocked=true` `fingerprint_bound=true`
`capsule_imported=true` `product_id=eliteew-fofline` `capsule_source=air`
`product_open=unavailable`. Second issue is `already_issued`.
Frontend `prod-open-1` still `not_available` /
`capability_not_available`. `IPAddressDeny=any` and `eno1`
`192.168.8.10/30` stayed. This is not an installer gate and not
`product.open`.

The same tree was then baked: overlay `a9009b2f…` (Runtime
`183ed7ce…` with `launcher.pin_issue`, air helper
`16265a14…`, Shell `index-Cj6Gjhru.js`). First `--resume` wrote
`out/` but QEMU failed `NAMESPACE` because
`empiieros-runtime.service` bound `/home/eliteew-ts/.ssh/authorized_keys`
while post-build forbids that kiosk file. Post-build now drops that
bind when the file is absent (`selftest-w4`
`missing_kiosk_key_bind_dropped_ok`). Second `--resume`
`image_contract_ok`: `out/rootfs.ext2` `ca0aad4c…`,
`rootfs.tar.gz` `13ca799b…`, bzImage `04fab7e1…`.
`qemu-empiieros-smoke.sh` exit 0 (`seccomp mode=2 filters=28`).
`qemu-installed-system-smoke.sh` exit 0. Full
`selftest-secure-reset.sh` still fails on missing kinship readiness
docs. This is not proof-before-`ERASE`, not USB, and not
`product.open`. Live `192.168.8.10` was not reinstalled.

Live SPA pack OTA 2026-09-18 03:28:43 +10: first `empiieros-spa-fetch`
connect to `10.88.89.1:443` timed out because
`eliteew_ts_prelicense` output dropped TCP 443 (ICMP and TCP/53 to
the VIP still passed; public `93.77.165.185:443` stayed dropped).
Overlay `prelicense.nft` now accepts TCP 80/443 only to `10.88.89.1`.
After the live rule, TCP 443 to the VIP succeeded. Helper
`c0704308…` wrote `{state}/fofline-spa/current/`
`spa_state=ready` `app_build_id=1786238708853-jn25ybw0v0`
(`index.html` + `pack-manifest.json`, `include_audio=false`). Replay
returned already current. Runtime `IPAddressDeny=any` stayed.
`eno1` stayed `192.168.8.10/30`. This is not an official image.

Live typed `product.open` 2026-09-18: first mux Runtime `3fce7b61…`
returned `pack_publish_owner` (no `CAP_CHOWN`) and left the pack
unreadable under `StateDirectoryMode=0700`. Lock: chmod-only
publish — state/`fofline-spa` `0751`, pack `0755`/`0644`. Mux
`20260917T175134Z-runtime-11030` Runtime `08d262c6…`, earlier shell
`20260917T174706Z-shell-21735` GUI `08550607…` / `index-qYFKNMY4.js`.
Frontend `prod-open-3` `applied` `fofline_local`
`fofline-spa://localhost/index.html?eliteewFoflineClient=1`.
`prod-cloud-3` `remote_spa_forbidden`. Kiosk `head` of
`index.html` succeeded; `first_boot_pin.json` stayed unreadable;
`last_launch.json` `0600`. `IPAddressDeny=any` stayed. `eno1`
`192.168.8.10/30`. Native WebKit `fofline-spa` serve was not
syslog-proven this turn. Official `out/` still Runtime `9d062344…`.

Live portal host-map 2026-09-18: mux Runtime `eb06b81f…`
(`20260917T180522Z-runtime-5485`), portal helper `e20a8f83…`, air
helper `1f751aa4…`. Frontend `hostmap-1` `applied`. `/etc/hosts`
`0644` maps `app.eliteew.ru` / `auth.eliteew.ru` to `10.88.89.1`.
`resolvectl` synthetic VIP. TCP `10.88.89.1:443` open; public
`93.77.165.185:443` failed. `eno1` `192.168.8.10/30`. Default not
`wg-portal`. Runtime `IPAddressDeny=any`. Seat has no `curl` /
`openssl`. Official `out/` still Runtime `9d062344…`.

Live edge health + glass auto-open 2026-09-18: mux Runtime `a951fad7…`
(`20260917T181733Z-runtime-525`), SPA helper `e20c5169…`
(`20260917T181743Z-spa-helper-23896`), GUI `08550607…`
(`20260917T181903Z-shell-8268`), Shell `index-yrbp10GI.js`.
Frontend `edge-health-1` `applied` `body_prefix={"ok":true}`
`path=/api/health` `via=10.88.89.1`. First shell mux of stale GUI
`aeb9bdfe…` wrote `last_launch` without a `fofline-spa` scheme.
Current native + `autoOpened` then syslog
`product surface show origin=fofline-spa` and
`FoFline pack served path=.../index.html ... bytes=4102`.
Quote fonts are not in the core pack and stay
`FoFline pack asset is not trusted`. `IPAddressDeny=any`.
`eno1` `192.168.8.10/30`. Official `out/` still Runtime `9d062344…`.

Official FoFline payload bake 2026-09-18: overlay `c78e2d11…`,
Runtime `fa1fbc6b…`, GUI `08550607…`, Shell `index-DrgGJyyP.js`,
SPA helper `e20c5169…`, `empiieros-wg` `bbdc4dcd…` (no `wg-quick`
string). `--resume` `image_contract_ok` wrote `out/rootfs.ext2`
`30111ba6…` / `rootfs.tar.gz` `c3a60cf8…` / `bzImage` `db616a36…`.
debugfs has `/lib/modules/6.12.20/kernel/drivers/net/wireguard/wireguard.ko`.
`qemu-empiieros-smoke.sh` exit 0 (`seccomp mode=2 filters=28`).
Installer deny `E2E_PROOF_DENIED` and installed-system smoke exit 0.
Live `192.168.8.10` was not reinstalled.

Lab mux of the official payload 2026-09-18 (no installer): Runtime
`fa1fbc6b…` (`20260917T184759Z-runtime-2172`), portal `04353854…`,
air `0e5eb146…`, Shell `index-DrgGJyyP.js`. Kept the live Runtime unit
(`IPAddressDeny=any`, existing `authorized_keys` bind). `eno1`
`192.168.8.10/30`. `sshd` stayed `active`. `mux-status-1`
`product_open=allowed` `channel_code=valid_up`. `mux-health-1`
`{"ok":true}` via VIP. Auto-open served the local pack again.
No USB / `ERASE`.

Lab mux native OIDC + Launcher chrome 2026-09-18 (no installer):
cause of the stuck EliteEW glass was auto-open without a Keycloak
window and F9-only exit. Lock: removed `autoOpened`; `product.open`
requires `oidc_session`; native `shell.auth_surface` + «Лаунчер».
Harness lock: `metal-live.sh` now unsets `SSH_ASKPASS` /
`SSH_ASKPASS_REQUIRE=never` so ControlMaster slaves do not block on
`ksshaskpass` “Allow shared connection”. Mux Runtime
`20260917T192845Z-runtime-29695` `2c390a84…`, SPA
`20260917T192906Z-spa-helper-31198` `ff8ed92c…`, Shell
`20260917T192919Z-shell-16420` GUI `212f2b4c…` /
`index-B7h7ZQLJ.js`. Live unit kept (`IPAddressDeny=any`,
`authorized_keys` bind). Frontend `oidc-status-1` `observed`
`oidc_session=missing`. `oidc-open-1` `failed`
`oidc_session_required`. No `product surface` / `FoFline pack served`
after 05:28:46. Runtime restart dropped `wg-portal`; `oidc-chan-1`
`applied` `valid_up`; VIP ping 1/1 147 ms. `eno1` `192.168.8.10/30`.
`sshd` `active` `:22`. Glass Keycloak is not operator-clicked.
Official `out/` still Runtime `fa1fbc6b…`. No USB / `ERASE`.

Lab glass “login does not open” 2026-09-18: operator clicks at 05:34–05:35
logged `auth surface show` then immediate `oidc auth navigation denied`
on GUI `212f2b4c…`. Two causes: GTK3 `gtk_widget_show_all()` no-ops when
`no_show_all` is set (overlay and «Лаунчер» stay invisible); WebKit
canonicalizes Keycloak to `:443` which the auth policy rejected. Lock:
`empiier_widget_show_deferred` clears `no_show_all` before show; auth
host allowlist accepts `https://auth.eliteew.ru:443/`. Mux Shell
`20260917T193945Z-shell-29282` GUI `1a29dea4…` / `index-B7h7ZQLJ.js`.
`eno1` `192.168.8.10/30`. `IPAddressDeny=any`. `sshd` active. Glass
Keycloak form is not yet operator-reproven.

Lab glass “white window + Отмена” 2026-09-18: syslog
`oidc auth navigation denied uri=https://app.eliteew.ru/fofline-oidc-bridge.html?error=invalid_request&error_description=Pushed+Authorization+Request+is+only+allowed`.
Cause: query-authorize against a PAR-only client; overlay then ignored the error redirect (blank WebKit) and showed a raw GTK Cancel bar. Lock: helper PAR POST (`ext/par/request`) then auth URL is `client_id` + `request_uri` only; error callback hides and fails closed; no GTK «Отмена». Live `par-start-1` `applied` `request_uri=urn:ietf:params:oauth:request_uri:4562e657-…`. Mux Runtime `20260917T194512Z-runtime-1233` `b7ff6eeb…`, SPA `7d549345…`, GUI `cc4ddbc4…`. `par-chan-1` `valid_up`. `IPAddressDeny=any`. `eno1` `192.168.8.10/30`. `sshd` active.

Lab glass still white after PAR 2026-09-18 06:03: operator click ran helper
`ok` `request_uri=d22c0f0e-…`, `auth surface show`, no deny and no
`oidc_session`. Overlay sat open; WebKit never logged a load. Cause: auth
WebContext sandbox hid CA store and `/etc/hosts`, so HTTPS to the VIP
never painted. Lock: `set_sandbox_enabled(auth_context, FALSE)` plus
`auth load started/finished/failed` syslog. Mux Shell
`20260917T200934Z-shell-25717` GUI `4847b543…`. Host allowlist stays.
`eno1` `192.168.8.10/30`. `IPAddressDeny=any`.

Lab glass Keycloak form 2026-09-18 06:12: `auth load finished`
`login-actions/authenticate`, then `navigation denied uri=https://app.eliteew.ru/`
(Keycloak back-to-application / client root, not the FoFline bridge).
Overlay stayed dead; `oidc_session` still `missing`. Lock: app origin
without `code=` hides auth and returns to the launcher. Mux Shell
`20260917T201411Z-shell-12293` GUI `d09596b7…`. Remote SPA stays denied.

Lab glass “Мы сожалеем” 2026-09-18 06:17–06:25: `auth load finished`
`login-actions/authenticate` then either Keycloak `error.ftl` or
`https://app.eliteew.ru/` (Back to Application). Cause: JS
`preventDefault` does not stop WebKit `submit-form`; empty/hidden
WebAuthn POST becomes `error.ftl` (`userId=null`). Hide-on-app-root
also tore down a still-valid IdP tab. Lock: native `submit-form` gate
(fail-closed on empty fields, extra-ref so dispose cannot auto-submit);
detect `Мы сожалеем` / Back to Application and rebuild once with a
fresh PAR GET (never replay the same `request_uri`); cookie accept
always. Mux Shell `20260917T203307Z-shell-25942` GUI `a7b5ca45…`.
Glass login not yet operator-reproven. `eno1` `192.168.8.10/30`.
`IPAddressDeny=any`. `sshd` active.

Lab glass still «Мы сожалеем» after form-gate 2026-09-18 06:33–06:51:
same Tauri §9.14–§9.15 pattern (`kind=error` on first GET, no
`submit-form`). Cause: EmpiierOS PAR omitted the living binding ticket
(`eliteew_ticket` stays in the helper POST, never on the browser URL).
Lock: `oidc_ticket::issue` (challenge + ticket via VIP
`app.eliteew.ru`) then `par_form(..., Some(ticket))`. First mux used a
fresh `oidc/signing.sk`; helper logged `oidc ticket issued` and PAR
`applied`, but Keycloak still rendered `error.ftl`. Second mux signs
with the existing `device_keys/signing.sk.hex` (same identity as air
device-request). Live Runtime `20260917T205411Z-runtime-26323`
`033f24f9…`, SPA `20260917T205421Z-spa-helper-5320` `c83f1764…`.
`ticket-start-2` `applied` with PAR `request_uri` and no ticket in IPC.
Glass form after the device-key ticket is not yet operator-reproven.
Seat Shield must not sit in the FoFline catalog. Lock: host workspace
`shield` + taskbar entry (same plane as Launcher / FoFline), not a
FoFline app tile. Observe-only until the floor helper is armed. `eno1` `192.168.8.10/30`. `wg-portal` stayed up.
`IPAddressDeny=any`. `sshd` active. Official `out/` unchanged.

Lab glass still «Мы сожалеем» after device-key ticket 2026-09-18 07:01–07:16:
helper `par_fields` includes `eliteew_ticket` (`prefix_ok=1`, `atkt_`
len=48) and PAR `applied`, but first WebKit finish is still
`login-actions/authenticate` + `kind=error` with no `submit-form`. Same
Tauri §9.14 pattern after the ticket is in PAR: Keycloak preauth/precheck
denies before `login.ftl`. Lock: put `eliteew_ticket` before `dpop_jkt`
in the PAR body; identity Content-Encoding + chunked JSON extract on
IdP POST; stop auto-rebuild on first `error.ftl` (rebuild only on
back-to-app). Seat Shield muxed as host workspace (taskbar / key 5),
not FoFline catalog. Live Runtime `e6626df1…` / SPA `2ef73604…` / GUI
`9ac353b8…` / Shell `index-PpJxtPkV.js`. `ticket-start-4` `applied`.
`channel_code=valid_up`. `IPAddressDeny=any`. Glass form still not
operator-reproven. Official `out/` unchanged.

Lab glass still «Мы сожалеем» after ticket-in-PAR 2026-09-18 09:37–09:50:
operator asked for the Tauri auth component history, not another form
hook. Living cause in DESKTOP-ATOMIC §9.14–§9.17 and KEYCLOAK-ELITEEW
item 10 is Keycloak preauth (`client_request_param_eliteew_ticket` +
`/auth/internal/precheck`), which we cannot read from this seat. The
native lock we had still skipped versus `native_oidc/dpop_iat.rs` is
DPoP `iat` from issuer HTTP Date (§9.7 / §9.19). Lock: `oidc_dpop_iat.rs`
+ helper HEAD/GET `/realms/empery` Date; PAR field order matches
`par.rs` (`dpop_jkt`, then `eliteew_ticket`, then `prompt=login`). Auth
WebView was already ephemeral (Tauri `incognito`). Mux Runtime
`7b60f649…` (`20260917T234936Z-runtime-32363`) / SPA `30b23017…`
(`20260917T234928Z-spa-helper-10295`). Live-pin unit kept
(`IPAddressDeny=any`, `authorized_keys` bind). `chan-1` `valid_up`.
`oidc_session=missing`. `eno1` `192.168.8.10/30`. `wg-portal`
`10.88.89.163/32`. `sshd` active. Official `out/` unchanged. Glass
username form is not operator-reproven.

Glass click after Date-iat mux 2026-09-18 09:53–09:57 +10: disk Runtime
`7b60f649…` (unit entered 09:49:37) / SPA `30b23017…` (09:49:29). GUI
binary stayed `9ac353b8…` (07:16) PID `91247`. Helper on the click:
`prefix_ok=1` `par_fields=…,dpop_jkt,eliteew_ticket,prompt` and
`oidc dpop iat source=http_date skew_vs_local_secs=1`. First finish
still `kind=error:sorry` with no `submit-form` (09:53:55, 09:54:10,
09:57:41). One back-to-app rebuild then `budget=0`. Date iat is live
and is not the remaining deny. `IPAddressDeny=any`. `eno1`
`192.168.8.10/30`. `sshd` active. Official `out/` unchanged.

Alt+F4 black-seat 2026-09-18 10:07–10:18 +10: stock kiosk bound
`A-F4` to `pkill -f /usr/local/bin/empiieros-shell-gui`. After 12 GUI
deaths the autostart loop `exec sleep infinity`, so the OS face did not
return. A later VT1→VT2 plus `getty@tty1` `Restart=on-failure` left
tty1 empty (`GTK display initialization failed` without labwc).
Lock: EmpiierOS profile binds `A-F4`/`Close` to `true`; GTK swallows
`delete-event` and Alt+F4; autostart cools 15s and retries after a
healthy 30s run. Live: kiosk `49c0a224…` + GUI `82f1c85e…` (strings
`kiosk close ignored`), `getty@tty1` started (not a reboot), labwc PID
`207819`, GUI PID `207841` still up at +27s. `rc.xml` `command="true"`.
Runtime `7b60f649…` / `IPAddressDeny=any` / `eno1` `192.168.8.10/30` /
`sshd` stayed. Official `out/` unchanged. Glass Alt+F4 not pressed from
SSH.

Session-bus console spam 2026-09-18 10:22–10:29 +10: labwc
`import-environment` hit session `org.freedesktop.systemd1` =
`Exec=/bin/false` (`Failed to import environment ... status 1`).
GTK/WebKit then activated `org.a11y.Bus` even with `NO_AT_BRIDGE=1`.
Lock: `LABWC_UPDATE_ACTIVATION_ENV=0` plus EmpiierOS
`kiosk-session-bus.conf` (no `<standard_session_servicedirs/>`).
Live kiosk `ca80d0a6…`, `dbus-run-session --config-file=...`, GUI
`217294`, no `at-spi-bus-launcher`. Latest start log has no systemd1
and no `org.a11y`. Leftover CRITICAL is WebKit sandbox
`fofline-spa/current` canonicalize (separate). `IPAddressDeny=any`.
`eno1` `192.168.8.10/30`. `sshd` active. Official `out/` unchanged.

Pack walk 2026-09-18 10:40–10:41 +10: three generators hid
`PACK_ROOT` from `eliteew-ts` — `StateDirectoryMode=0700`,
`StateStore::open` `0700`, and `spa_pack` `0700` on `fofline-spa/`.
`product.open` chmod `0751` bounced on the next Runtime start, so GUI
start logged `Failed to canonicalize path ... Permission denied` and
`Attempted to add disallowed path to sandbox`. Lock: unit
`StateDirectoryMode=0751` (live-pin kept, `IPAddressDeny=any`);
`StateStore::open` `0751`; pack parent `0751` / `current` `0755` /
files `0644`; Runtime start + pack-ready call
`ensure_shell_traverse`. Mux Runtime `e3f3c71e…`
(`20260918T004023Z-runtime-12037`). Live: state/`fofline-spa` `0751`,
`current` `0755`, `index.html` `0644`; kiosk `runtime_x=yes spa_x=yes
pack_r=yes list_root=no pin_r=no`; PIN and `last_launch.json` `0600`.
GUI `225911` (binary still `82f1c85e…`) `WebKit sandbox enabled`; no
canonicalize CRITICAL after 10:40. `chan-walk-1` `valid_up`.
`wg-portal` `10.88.89.163/32` stayed. `eno1` `192.168.8.10/30`.
`sshd` active. Official `out/` unchanged. WebKit `Seccomp=0` is still
the next-image plane.

Post-login no session 2026-09-18 11:01–11:28 +10: operator restored
the living Keycloak SPI; glass `kind=login`, submit allowed, callback
captured. `oidc-complete` failed `callback_code`. Cause: Keycloak 26
codes are dotted `uuid.tab.client`; `parse_callback_code` used bare
b64url (no `.`). C capture only checks `code=` + `state=`. Lock:
`auth_code_ok` (alnum + `-` `_` `.`, ≤512). Mux Runtime `09e2569c…`
(`20260918T012514Z-runtime-16178`) + GUI `75cb0b7d…` (syslog now
prints `error.message`). Re-login then wrote `callback.json` and still
failed: Runtime parse passed, helper `30b23017…` re-parses on
`oidc_token`. Live shape (no secret): prefix exact, `code_len=98`
`dots=2` `other=0`. Lock: mux SPA `c9c8c6ef…`
(`20260918T012828Z-spa-helper-31016`). `oidc_session` still `missing`
until the next glass login. `wg-portal` `10.88.89.163/32` stayed.
`IPAddressDeny=any`. `eno1` `192.168.8.10/30`. `sshd` active. Official
`out/` unchanged.

Post-login false leak 2026-09-18 11:32–11:38 +10: after the helper mux,
glass login `oidc-complete` `applied` and `session.json` `0600` (11:34:21).
GUI still said `oidc complete failed: oidc token leaked`. Cause: C
`strstr(response, "refresh")` matched the complete `detail` sentence.
Lock: match only `"access_token"` / `"refresh_token"`; detail is
`secrets stayed in Runtime`; activator refreshes `launcher.status`
even when the complete event is false. Mux GUI `8bcbf0ac…` / Shell
`index-CKW9iYQk.js` (`20260918T013733Z-shell-10881`). After remux:
`oidc session injected origin=fofline-spa`, `product surface show`,
pack `index.html` `4102`. Quote fonts stay denied (`include_audio=false`).
`wg-portal` stayed. `IPAddressDeny=any`. `eno1` `192.168.8.10/30`.
`sshd` active. Official `out/` unchanged.

FoFline CORS blank 2026-09-18 11:38–11:42 +10: after inject the pack
served `index.html` and JS `200`, then WebKit blocked every module
(`Origin fofline-spa://localhost is not allowed by
Access-Control-Allow-Origin`). Cause: `serve_fofline_pack` used
`webkit_uri_scheme_request_finish` with no CORS headers. Lock: shared
`finish_scheme_ok` (`Access-Control-Allow-Origin: *`) on `empiier://`
and `fofline-spa://`. Mux GUI `9b837246…`
(`20260918T014138Z-shell-29046`). Product surface was not re-opened
from SSH. Quote fonts still absent. `wg-portal` stayed.
`IPAddressDeny=any`. `eno1` `192.168.8.10/30`. `sshd` active. Official
`out/` unchanged.

Scheme CSS ignored 2026-09-18 11:42–11:48 +10: unifying pack CORS onto
`finish_with_response` made WebKitGTK load `index-CQl8MLk0.css` (200)
but not apply it (`bodyBackground=none`, render probe fail). Vite
already strips `crossorigin` on stylesheets. Lock: CSS uses simple
`finish()`; JS/HTML keep CORS. Mux GUI `8a28901e…`
(`20260918T014751Z-shell-27125`). Live `shell render ready attempts=1`.
`eno1` `192.168.8.10/30`. Official `out/` unchanged.

Post-login preloader hang 2026-09-18 11:50 +10: glass «Вход» completed.
Keycloak `UserSessionLimitsAuthenticator` logged session count 1,
terminated the oldest `empery-fofline` session (living `userClientLimit=1`).
EmpiierOS `oidc-complete` `applied`, `session.json` `0600`, inject
`applied`, product show + pack serve. FoFline stayed on loading. Cause:
living `init()` always `refreshAccessToken()` after hydrate; inject
`refreshSessionJson` returned `{ok:false}`. JS-imported CSS also
CORS-failed under the all-CSS simple finish. Lock: refresh returns the
current bootstrap access/id; CSS CORS-by-Origin. Mux GUI `bf3cfbf9…`
(`20260918T020010Z-shell-6304`). Live `oidc_session=present`
`product_open=allowed`; new PID `275319` `shell render ready attempts=1`.
`wg-portal` `10.88.89.163/32` stayed. `IPAddressDeny=any`.
`eno1` `192.168.8.10/30`. Official `out/` unchanged. Glass reopen of
FoFline is still pending on the new GUI.

Post-login preloader after CSS lock 2026-09-18 12:07–12:12 +10: glass
login again (`kind=login`), complete/inject `applied`, pack served
through `desktopUi`. CSS CORS count 0. Quote fonts still denied. Then
no further GUI lines and `ss` showed no `:443` / `10.88.89.1`. Cause:
product/shell WebKit sandbox hid host CA and NSS
`resolve [!UNAVAIL=return] files` — getaddrinfo never reached the VIP
(same class as the auth white page). Lock: sandbox off on that context;
`decide_policy` + custom schemes stay the boundary. Mux GUI `1d6b68d1…`
(`20260918T021512Z-shell-10065`). Live log `WebKit sandbox off for host
CA/NSS product HTTPS`; `shell render ready attempts=1`.
`oidc_session=present` `product_open=allowed`. `wg-portal` stayed.
`IPAddressDeny=any`. `eno1` `192.168.8.10/30`. Official `out/` unchanged.

Sandbox-off remux was the wrong cause on this seat 2026-09-18 12:15–12:16
+10: this WebKitGTK is `ENABLE_BUBBLEWRAP_SANDBOX=0`. After that mux,
glass Open 12:16:45 inject `applied`; FoFline CSS already served at
12:15:43 with CORS count 0 (`preloader` / `mainFullApp` / `styles` /
`600`). `ss` still no `:443`. The only resource FAIL is
`/fonts/elite-quote/*.woff2` — that directory is absent; the pack has
Cormorant/Manrope under `assets/`. No pack `/api/` request was logged.
Living boot `fetch('/api/v1/system/maintenance')` is relative
(`fofline-spa://localhost/...`), not the VIP. Axios to
`https://app.eliteew.ru` never started. Static `#app-preloader` stays
until React commit (`Promise.all` waits on `reactAppEntry`).
Sandbox-off stays (auth/seat already off); it is not the hang lock.

Enter lock 2026-09-18 12:25 +10: glass still «не входит». Cause: pack
`/assets/` CSS imported by `reactAppEntry` could take the simple
stylesheet finish when Origin was missing, so the module graph never
committed; relative `fetch('/api/...')` stayed on `fofline-spa://`;
inject did not set the FoFline sticky flag. Lock: pack `/assets/` CSS
always CORS-finishes (shell `/assets/` stays simple); inject sets
`eliteew.foflineClient.v1` and rewrites `/api/` fetch to
`https://app.eliteew.ru`; hide clears the inject script; boot probe
logs protocol/origin/native/root/preloader only. Mux GUI `6b3efed4…`
(`20260918T022844Z-shell-18616`). Official `out/` unchanged.

Local-scheme ESM CSS 2026-09-18 12:37 +10: operator Open on GUI
`6b3efed4…` PID `408`. Inject `applied`. Pack served JS graph plus
`mainFullApp`/`styles`/`600` CSS. Boot `n:true` `r:false` `pre:true`
twice; no further pack lines; `ss` no `10.88.89.1:443`. Quote fonts
still denied. Later `oidc_session=missing` (access TTL after 12:25
login). Cause: `fofline-spa` `as_local` is file-like; ESM CSS import
never settles so React never commits. Lock: do not register
`fofline-spa` as local; keep secure+CORS; 15 s boot probe; CSS finish
mode log. Mux GUI `d3967912…` (`20260918T025009Z-shell-10058`)
PID `5133`. Live `scheme css finish` shell `mode=simple`.
`oidc_session=missing` (TTL). `wg-portal` stayed.
`IPAddressDeny=any`. `eno1` `192.168.8.10/30`. Official `out/` unchanged.

Vite preload hang 2026-09-18 12:51 +10: login `kind=login`, complete
`applied`, inject, pack CSS CORS-finished (`cors=1 pack=1`). Boot
`r:false` `pre:true` `css:5` through 15 s. No `App-CrSAywAU.js`.
No VIP `:443`. DNS VIP synthetic OK. Cause: Vite `__vitePreload`
`<link rel=stylesheet>` never `load`s when CSS uses
`finish_with_response`. Lock: all stylesheets simple `finish()`.
`fofline-spa` stays not `as_local`. Mux GUI `2a6fbd2e…`
(`20260918T025528Z-shell-2950`). Official `out/` unchanged.

Host logout 2026-09-18: activator **Войти** now has **Выйти**
(`launcher.oidc_logout`). Session wipe is overwrite-and-unlink;
product/auth hide. Official `out/` unchanged.

Hang after login 2026-09-18 13:02 +10: pack CSS/App chunk unlocked
(`r:true`,`pre:false`, `App-CrSAywAU.js` served, VIP `:443` ESTAB then
idle). `err:err` is empty font-load (`/fonts/elite-quote` missing).
Official bake axios has no `fofline-spa` absolute API — relative
`/api/v1` stays on the custom scheme. Host inject now rewrites fetch
and XHR to `https://app.eliteew.ru`. Official `out/` unchanged.

Hang 2026-09-18 13:11–13:12 +10 after XHR rewrite: boot `r:true`
`#desktop` `tx=Подключение…` for 45s, VIP `:443` ESTAB. Living
FRONTEND-AUTH: that copy is LoginPageFallback while AuthContext
syncs after the facade already has tokens. Living FoFline: IdP
discovery CORS from `fofline-spa://localhost` is denied (ACAO only
for `https://app.eliteew.ru`); JWT `atob` is Latin-1 so Cyrillic
claims drop `tokenParsed`/`userInfo` and sync never commits.
Lock: inject UTF-8 `atob` + sessionStorage foflineNative tokens.
Official `out/` unchanged.

Official-pack login card 2026-09-18 13:16–13:18 +10: inject `applied`
`n:true`, UI «Вход / Войти», no VIP TCP. Realm `empery` /
`empery-fofline` (correct). Official `keycloak-BqrLwoGk.js` never reads
`EmperyDesktopNativeOidc`; storage without `refreshToken` is skipped;
`applyTokens` needs both fields; `fofline-spa` refresh is backend SSO;
in-SPA «Войти» assigns Keycloak and product policy ignores it. Lock:
access JWT as the pack `refreshToken` field; host `hostSsoRefresh` for
`/api/v1/auth/sso/refresh`; swallow product Keycloak assign. No
`mobileEmbeddedClient`. Official `out/` unchanged.

Same-origin API 2026-09-18 13:29 +10: token apply worked (`r:true`
`#desktop`), then «Не удалось загрузить безраздельный мир». WebKit:
`Origin fofline-spa://localhost` not in ACAO, status 409, on VIP
`/api/v1`. Cause: absApi made `/api/` cross-origin. 409 is SSO
`session_replace_confirmation_required`, not Starlette 400. Lock:
scheme proxy `fofline_api_proxy` + stop HTTPS rewrite. Mux GUI
`4babd9fa…` (`20260918T034033Z-shell-32659`) PID `22954`. Product
Open after remux is still pending. Runtime `IPAddressDeny` stays.
SSO bind after complete is the next slice. Official `out/` unchanged.

SSO registry bind 2026-09-18 13:43 +10: proxy hops were same-origin
(`maintenance` 200, `shell-bootstrap` 409). Cause: first API had no
`X-SSO-Replace-Confirm`, so XOR/replace stayed unconfirmed. Lock:
proxy plants the header; inject arms the living confirm window.
Do not POST `/auth/sso/callback`. Mux GUI `0d9809bc…`
(`20260918T034905Z-shell-17736`) PID `27900`. After Open: bootstrap
**200**, 14×200, `#desktop` icons. Runtime `IPAddressDeny` stays.
Official `out/` unchanged.

Glass walk 2026-09-18 13:49–13:51 +10: EWA / Start / EWDesk / UGS /
Music Player chunks served; API 20×200. Scheme denials only
`/fonts/elite-quote/*` and `/desktop-wallpapers/15.webp`. Cause:
EmpiierOS synced core `spa-latest` only; living optional groups and
the elite-quote tree were absent. Lock: `spa_sync` now merges
`spa-assets-latest` groups into `current/` (hash-skip, no core wipe;
`audio-guides` deferred). User media stays `/api/v1/content`. Official
`out/` unchanged.

Optional groups mux 2026-09-18 14:04–14:08 +10: helper `5859fdb9…`
(`20260918T040334Z-spa-helper-7404`) + Runtime `42ca26b9…`
(`20260918T040343Z-runtime-25398`). `spa_sync` merged 7/8 groups
(~136s); `audio-guides` deferred. Disk has wallpaper `15.webp` and
`fonts/elite-quote/*.woff2` (`0644`). Replay hash-skip 2.8s.
`oidc_session=missing` after the Runtime bounce. `IPAddressDeny=any`.
`eno1` `192.168.8.10/30`. Official `out/` unchanged.

Runtime refresh 2026-09-18 14:25–14:28 +10: helper `5aaea1e2…` /
Runtime `656ed89a…`. First Open failed: VIP `:443` down. Operator
brought the VIP up 14:48 +10 (ICMP 1/1, TCP 443 open). `up-open-1`
14:49:06 `applied` in 2.2s; helper Date-iat refresh;
`oidc_session=present`; `expires_at` renewed. IPC Open does not show
the product WebView (`shell.product_surface` is GUI-only). Operator
**Открыть** 14:52:33: inject + surface; elite-quote
`400/500/600.woff2` and `desktop-wallpapers/15.webp` scheme-served;
deny 0; API 49×200 no 409. `IPAddressDeny=any`. `eno1`
`192.168.8.10/30`. Official `out/` unchanged.

Official SPA pack bake 2026-09-18 15:07–15:11 +10: reused living
`dist-build` `1789443659753-rovukoqfw9` (web image
`eliteew-frontend:prod-baked-20260915-1340`). Core `15M` plus 8
groups; packed JS has `EmperyDesktopNativeOidc`. Signed and staged
under `docker/data/fofline-updates/`. Operator rsync (no dest prune)
15:18 +10. Public `spa-latest` `1789443659753-rovukoqfw9` HTTP 200.
Lab `spa_sync` 15:20:10–15:22:48 promoted `current/` (7/8 groups).
`oidc_session=missing`. Host stand-in stays until glass. Official
`out/` unchanged. `eno1` `192.168.8.10/30`. `IPAddressDeny` stayed.

2026-09-18 later +10: glass reached FoFline (taskbar network indicator)
then the access TTL ended. Cause: `refreshSessionJson` replayed bootstrap
instead of Runtime refresh (EliteEW `product_session_refresh`). Lock in
source: `empiierosOidc` + `inject` `force=true`. Mux
`20260918T055805Z-runtime-16367` Runtime `c6d3f8d2…` /
`20260918T055821Z-shell-14183` GUI `f70b3420…`. `wg-portal` stayed.
Glass 16:02–16:13 +10 after mux: login + `#desktop`; helper refresh
16:03:08 and 16:13:13; access rotated; `profile/me` 200; no 401.
Video-calls poll stays **403** 131 B (`fofline_remote_tools_denied`).
`eno1` `192.168.8.10/30`. `IPAddressDeny` stayed.

2026-09-18 later +10: OS observe slice (no live mux). GUI logs
`fofline edge start|failed`, boot `es`, API `reason=`. Binary
`0b5bcbd8…` at `/tmp/empiieros-shell-gui-edge-observe`. Meet hostmap
not opened. Official `out/` unchanged.

2026-09-18 16:52 +10: mux GUI observe
`20260918T065250Z-shell-18754` `0b5bcbd8…`. Glass bounced. Official
`out/` unchanged. `IPAddressDeny` stayed.

2026-09-18 later +10: seat-launcher source (no live mux yet). Cause:
host was a lab dump plus pack-kind catalog tiles. Lock: `launcherStage`,
one CTA, `foflineHostTiles` (no `pack`), one-shot portal when valid+down,
`autoOpened` still forbidden. Official `out/` unchanged.

2026-09-18 17:11 +10: mux Shell `20260918T071109Z-shell-2367` assets
`index-CsL3ri0J.js` / `index-BXp9vPrX.css`. GUI binary unchanged
`0b5bcbd8…` PID `43731`. `eno1` `192.168.8.10/30`. `wg-portal`
`10.88.89.163/32`. `sshd` active. Glass bounced to the staged host.
Official `out/` unchanged. `IPAddressDeny` stayed.

2026-09-18 17:37–17:38 +10: product HUD. Cause: full-width GTK sibling
bar. Lock: overlay top-center peek/hover; ComfyUI media; `aplay`
fallback (no `paplay`/`pw-play` on the image). Mux
`20260918T073716Z-shell-24219` then `20260918T073759Z-shell-13714`
GUI `537047f3…` PID `49516` syslog `product chrome attached hud`.
Media under `/opt/empiieros/shell/product-chrome/`. Runtime
`c6d3f8d2…` unchanged. `eno1` `192.168.8.10/30`. `wg-portal`
`10.88.89.163/32`. `sshd` active. Official `out/` unchanged.
`IPAddressDeny` stayed. Glass hover/click not walked from this host.

2026-09-18 17:46 +10: host desktop kinship. Cause: lab control-panel
skin on the OS seat. Lock: wallpaper + left OS icons + glass; launcher
not auto-open; pack names in Start spoiler only. Mux
`20260918T074633Z-shell-15680` `index-BxPh6yHu.js` GUI `537047f3…`
PID `56449`. `/opt/empiieros/shell/product-chrome/host-wallpaper.jpg`
present. Runtime unchanged. `eno1` `192.168.8.10/30`. `wg-portal`
`10.88.89.163/32`. `sshd` active. Official `out/` unchanged.
`IPAddressDeny` stayed. Glass walk of the new canvas is the operator's.

2026-09-18 18:02 +10: host chrome pass. Cause: overlay CSS margin
ignored so HUD stayed fully visible; icons were a fixed column; Start
was a fat labeled pill. Lock: `gtk_widget_set_margin_top` peek;
110px snap + persist; 46px EliteEW rail + harvested Start SVG +
running pills. Mux `20260918T080232Z-shell-17119` `index-SUpozrBO.js`
GUI `008d7f36…` PID `63895`. Start mark 90649 B on the seat. Runtime
unchanged. `eno1` `192.168.8.10/30`. `wg-portal` `10.88.89.163/32`.
`sshd` active. Official `out/` unchanged. Glass hover/drag/taskbar
not walked from this host.

2026-09-18 18:15 +10: viewport lock. Cause: `overflow-y: scroll` plus
`min-height: 100vh` and a sticky taskbar, so Settings painted to the
screen bottom and the document scrolled. Lock: `100dvh` overflow-hidden
face; in-flow 46px rail; `WorkspaceFrame` inset on the living desktop;
FoFline windows `top/left/right/bottom`. Mux
`20260918T081557Z-shell-18861` `index-BAyB-C9f.js` /
`index-DrN5BGdR.css`. GUI unchanged `008d7f36…` PID `74387`. Live
hashes match local `dist`. Runtime `c6d3f8d2…` not remuxed. `eno1`
`192.168.8.10/30`. `wg-portal` `10.88.89.163/32`. `sshd` active.
Official `out/` unchanged. Glass: no document scroll / Settings above
the rail — operator walk.

2026-09-18 18:29 +10: seat tray. Cause: `8.4rem` left gutter reserved
for the icon column, so Settings sat far from the screen edge; the
rail only painted a stale SSID chip. Lock: even `1rem` window inset;
tray reads snapshot network/battery/volume, ticking clock, UI
language, confirm-gated `system.power` flyout (`REBOOT`/`POWEROFF`).
Mux `20260918T082921Z-shell-26516` `index-Cfmg0GlP.js` /
`index-BTOAibEv.css`. GUI unchanged `008d7f36…` PID `84973`. Live
hashes match local `dist`. Runtime `c6d3f8d2…` not remuxed. `eno1`
`192.168.8.10/30`. `wg-portal` `10.88.89.163/32`. `sshd` active.
Official `out/` unchanged. Glass tray/power walk is the operator's.

2026-09-18 18:37 +10: tray icons. Cause: percent digits on the rail.
Lock: ComfyUI Flux.2 Klein tiles; hover percent/SSID; volume wheel +
click slider via `audio.control` 10–100. Mux
`20260918T083734Z-shell-21487` `index-cV9Dt2Ce.js`. GUI `008d7f36…`
PID `91496`. Prompt ids `13dd4442…` / `d6f921c8…` / `3e168cc5…`.
Live PNGs match `dist`. Runtime not remuxed. `eno1` `192.168.8.10/30`.
`wg-portal` `10.88.89.163/32`. Official `out/` unchanged. Glass wheel
is the operator's.

2026-09-19 20:13 +10: Lab mux hangup GUI + DTLS accept/enc.
Cause: lady `/end` left Meet up (`meetPost` one-shot;
dummy tracks spawned `dtlsconnection3` then
`certificate verify failed` after PEM-on-READY recreated
the connection without `on-peer-certificate`). Lock:
`meetPost` wait-map + `/end` hide + overlay-park; dummy
tracks removed; PEM on enc+dec; bind accept=TRUE after
pem set. Ninja **exit 0** `so.0.21.10` `5d776573…`.
`deploy-webkit-gst` `20260919T101341Z-webkit-gst-16482`.
`deploy-shell` `20260919T101353Z-shell-20640` GUI
`58d5566b…` assets `d89028a0…` (living pack). Runtime
`f22c0d8e…`. GUI PID `999739`. Official `out/` unchanged.
Two-in-room + hangup glass is the operator's.

2026-09-19 23:45 +10: Cable plug did not DHCP. Cause: install
persist `10-eliteew-ts-install-eno1.network` `DHCP=no`
`Address=192.168.8.10/30` won over image `20-wired.network`
(journal: carrier lost 23:27, gained 23:32, still the 10-
file). Lock: skip 10- persist for DHCP; installer GUI does
not prefill the lab `/30`; overlay example no longer ships
the address; portal check is any eno1 inet; metal-live
deploy proof is any eno1 IPv4. Glass: 10- moved to
`.disabled`, `20-wired.network` `DHCP=ipv4`. After reload:
`Network File` `/etc/systemd/network/20-wired.network`;
`192.168.8.10/30` `dynamic` via DHCPv4 from `192.168.8.9`
(`valid_lft 43196sec`). Official image bake not this turn.

2026-09-20 00:54 +10: Lab mux Seat Browser native chrome.
Cause: overlay was a naked WebKit; `reload`/`history` were
`not_available`. Lock: overlay toolbar + `host_action`
reload/back/forward; `media.set` stays debt. Transactions
`20260919T145413Z-runtime-2159` Runtime `19bf5f7f…` and
`20260919T145423Z-shell-11852` GUI `5f1207a2…` assets
`index-CaMBz6NY.js`. Unit hash unchanged `dd6bb219…`
`IPAddressDeny=any`. Syslog `seat browser chrome attached`
and `shell render ready attempts=1`. GUI PID `71294`.
`eno1` `192.168.8.10/30` (DHCP). `wg-portal` stayed.
Official `out/` unchanged. Open + chrome walk is the
operator's. Not MATCH. Not product ready.

2026-09-19 23:00 +10: Seat Browser call joined signaling then
ICE Terminated (keysray WebKit); lady Chrome ICE/DTLS/SCTP
then `Close called`. Cause: live `eliteew_ts_prelicense`
dropped UDP `13400` (tree overlay already had the hole).
Lock: persist
`udp dport 13400 ip daddr 93.77.165.185 accept` in
`/etc/eliteew-trusted-substrate/nft/prelicense.nft` and
`nft add rule` (no table delete). Backup
`prelicense.nft.before-13400-20260919T130000Z`. Public
`:443` still denied. `wg-portal` stayed UP. Runtime TTL
restamp still not muxed. Official `out/` unchanged.
Glass call walk is the operator's.

2026-09-19 22:53 +10: Seat Browser Meet went white. Cause:
VIP allowlist still listed `meet.eliteew.ru`, but live
Runtime had no restamp-on-evaluate; `fetched_at` 22:31 +
TTL 300s → fail-closed `navigation denied` at 22:38 and
22:49. Lock: restamp seat-local (`!is_default`) on
`evaluate`/`navigate`. Runtime mux this turn. GUI/Meet
path not remuxed. Official `out/` unchanged.

2026-09-19 22:21 +10: Operator opened Seat Browser, signed
into a second account, and joined a call. Jitsi painted
"WebRTC is not available in your browser". Cause: Seat
Browser WebView had `enable_media_stream=FALSE` and no
`enable_webrtc` (Meet view has both; factory WebKit already
ON). Lock: same engine flags on `browser_settings` +
UserMedia/DeviceInfo allow on the overlay. Mux
`20260919T122310Z-shell-17627` GUI `b18b186f…`. Meet
path not touched. Official `out/` unchanged.

2026-09-19 21:48 +10: Operator parked Meet and asked
for the custom OS browser. Harvest: EliteEW EBrowser
(Tauri stage, API policy, tabs/downloads/find) stays
oracle-only. EmpiierOS already had policy core +
WebKit overlay; `SeatBrowserCard` was unmounted.
Lock this slice: host tile `browser` + card + VIP
allowlist + local policy TTL restamp. First glass
URL `https://app.eliteew.ru/` (nft VIP only). Tile
icon ComfyUI `fb126fb9-2fa9-4c19-be25-66c85aab584e`
Flux.2 Klein 4B. Mux `20260919T115734Z-shell-8040`
assets `index-CQjou9Up.js` + `host-browser.png`; GUI
kept `30c2f66e…` PID `7988`. Runtime not remuxed
(VIP `policy.set` already restamps TTL). Not Tauri.
Meet untouched. Official `out/` unchanged.

2026-09-19 21:19–21:24 +10: Operator rebooted the
glass, then `725babe3-…`. Boot 21:19, one GUI `416`
`30c2f66e…`. Recycle at start + hangup (`kind=close`
21:23:45). `tx addTransceiver sendrecv`; answer
`opus ssrc a=sendrecv,a=sendrecv`. First
`media permission allowed` 21:20:45 (reboot freed
the leftover ALSA holder). Then ICE is **not**
COMPLETED: 9× checking→failed (~5 s), DTLS **0**,
PEM **0**, `conference.iceFailed` 21:21:06 and
21:22:14. JVB SDP is only public
`93.77.165.185:13400 typ srflx` (74097 lines);
no JVB host on `10.88.89.1`. Signaling VIP `:443`
open via `wg-portal`. Public `:443` closed. Re-check
21:35: `wg-portal` handshake to
`93.77.165.185:51823` via `eno1`/`192.168.8.9`
(nft allows UDP `51820-51830`). Live
`eliteew_ts_prelicense` output **drops** UDP
`13400` and TCP `3478` — seat
`/etc/eliteew-trusted-substrate/nft/prelicense.nft`
has no JVB hole; tree overlay still has
`udp dport 13400 ip daddr 93.77.165.185 accept`.
TURN `3478` SYN-SENT is that drop, not a dead
WS. Local ICE has UDP host `10.88.89.163` but
JVB is not in `AllowedIPs`. `Unknown mline`
32914. Pack `bounds` after hide →
`meet_stage_not_open`; poet `/end` 21:24:28.
Official `out/` unchanged.

2026-09-19 21:15–21:16 +10: Glass `99f2d34b-…` after
recycle mux. `tx addTransceiver sendrecv`. Answer
`opus ssrc a=sendrecv,a=sendrecv`. Fresh Meet WP
`1057795`. ICE `COMPLETED`. DTLS `CONNECTING` only
(CONNECTED **0**). Applied PEM **0** this WP. No
`media permission allowed`. ALSA `pcmC0D0p` busy —
held by product WP `1057247`. TURN `3478` SYN-SENT
on `eno1`. GST log 211 MB. Two VIP `:443` ESTAB
(WebKit). Video still absent: DTLS not up, not
recvonly. Official `out/` unchanged.

2026-09-19 21:14 +10: Lab mux Meet hide recycle +
sendrecv transceivers. Cause: `about:blank` kept WP
`1034818` (Jicofo `session-invalid`); first answer
`recvonly` / no transceivers left DTLS `CONNECTING`.
Lock: hide `terminate_web_process` (`process recycled`);
inject `addTransceiver(sendrecv)` (no dummy `addTrack`).
`deploy-shell` `20260919T111415Z-shell-11032` GUI
`30c2f66e…`. One PID `1057218`. Pack / Runtime / WebKit
/ `out/` unchanged. Glass after FoFline reload is the
operator's.

2026-09-19 21:04–21:06 +10: Second glass `de1409e1-…`
after poet `/end` of `c0b61b59`. Same Meet WP
`1034818` (park + `about:blank` did not kill it).
Jicofo `session-invalid` / Strophe `not-acceptable`
on `ew-gaf-3ceb279d…`. Answer again
`opus nossrc recvonly,recvonly`. ICE `COMPLETED`,
DTLS `CONNECTING` only; GST teardown 21:05:36 on
`kind=close`+`/end`. **0** `socket.io` / `wss://`
edge lines since 20:53; **0** ESTAB `:443` at 21:06
(only Soup TIME-WAIT). TURN `158.160.205.181:3478`
SYN-SENT from `eno1`. Leftover ICE listen on the
parked WP. Video is still DTLS, not FoFline WS.
Official `out/` unchanged.

2026-09-19 20:58–21:02 +10: Lady hung up; poet still
in `c0b61b59-…`. After `kind=open` 20:54:05: **0**
`kind=close`, **0** poet `POST …/end`. Pack
`VideoCallApp-bs-BnhEY.js` still `kind=bounds` every
500 ms at 21:02 (`useOsMeetStage` still true).
`video_call.ended` listener is in the pack; it did
not run. Jitsi `BridgeChannel` closed 20:58:47 — SCTP
timeout, not FoFline hangup. OS hide waits on poet
`close` or poet `/end`; lady's phone `/end` is
invisible here. Leftover GUI `1014401` still up.
Official `out/` unchanged.

2026-09-19 20:54 +10: Glass `c0b61b59-…` after dtls
rebind. `certificate verify failed` **0**. ICE
`COMPLETED`. DTLS stuck `CONNECTING` (never
`CONNECTED`). Answer `opus nossrc recvonly,recvonly`.
No `media permission allowed`. GST log froze at
0:00:07. Call on GUI `1015532` (labwc, `GST_DEBUG`
without `dtls*:5`); leftover GUI `1014401` still up.
`will only receive` for audio+video offer. Video
absent because DTLS never finishes, not because
Jitsi icons. Official `out/` unchanged.

2026-09-19 20:39 +10: Lab mux GST dtlsdec rebind.
Cause: stock 1.24 hooks `on-peer-certificate` only at
NULL→READY; pem-after-READY made `dtlsconnection7`
without the hook → `certificate verify failed`. Lock:
`create_connection` binds accept (already `return TRUE`).
Dropin
`gstreamer1/gst1-plugins-bad/0001-dtlsdec-rebind-peer-cert-after-pem.patch`.
`deploy-webkit-gst` `20260919T103954Z-webkit-gst-22223`
`libgstdtls.so` `ed2b13a6…`. WebKit `5d776573…` GUI
`d85ade76…` PID `1014401` (duplicate labwc GUI killed).
GST_DEBUG `dtls*:5`. Official `out/` unchanged. Next
glass: no `certificate verify failed`; syslog
`bound on-peer-certificate after create_connection`.

2026-09-19 20:32 +10: Lady mobile × poet glass
`c4c05928-…` (prior `435cc136-…` same). Meet `shown`
20:32:22, `createAnswer opus nossrc a=recvonly,a=recvonly`
20:32:34, gUM 20:32:53. GST: PEM on `dtlssrtpenc1` /
`dtlsdec1`; ICE `COMPLETED`; `dtlsconnection7`
`certificate verify failed` → PC `failed`. Same fail on
first call `dtlsconnection3`. Mute/cam crossed: no SCTP
without DTLS. `kind=close`+`/end` worked on `435cc136`.
Second stage still shown at 20:32. Official `out/`
unchanged. Lock (GST rebind accept after pem recreate)
not muxed.

2026-09-19 20:27 +10: Auth Keycloak card. Cause: `show_auth`
FILL painted Keycloak over the whole kiosk, so «Войти» looked
like a second fullscreen window. Lock: same process, same
kiosk `GtkWindow`; `auth_bounds.c` FILL dimmer eats clicks;
centered ~480×640 card; no `gtk_window_new`; allowlist /
`NEW_WINDOW` / PAR unchanged. Mux
`20260919T102720Z-shell-21800` GUI `d85ade76…` assets
`d89028a0…`. PID `1010891` `mode=kiosk`. Syslog
`auth surface card wired kind=card`. Pack/`out/` / Runtime
unchanged. Glass «Войти» is the operator's.

2026-09-19 19:56 +10: Glass after DTLS mux, lady hangup. Call
`36f729ae-…` open 19:52:23. **No** `POST /end`, **no**
`kind=close`; bounds until revive. Meet WP `985492` ~1 GB.
PEM **did** apply (`dtlssrtpdec0`/`dtlsdec0`); local fp
`75:D3:25…`, remote `C2:C3:99…`; `No remote fingerprints` 0.
ICE `CONNECTED`/`COMPLETED` on `webrtcdtlstransport0` then
`dtlsconnection3` `certificate verify failed` (OpenSSL client
PKI, not fingerprint). Dummy-track wrap still storms
`createAnswer` (17754). GUI revived PID `986582` (same
`8599856d…`). Hangup lock still not muxed. Official `out/`
unchanged.

2026-09-19 19:50 +10: Lab `deploy-webkit-gst`
`20260919T095010Z-webkit-gst-20684` replaced living
`so.0.21.10` `0c984e8e…` with DTLS PEM `ce805b65…`
(126232296 B). Seat grep: `ensureDtlsCertificatePem` /
`Applied DTLS PEM` / `X509_digest`. GST plugins unchanged
(rtpbin / videorate / deinterleave). Dual GUI after revive:
kept labwc `983615` (GST_DEBUG), killed extra `983433`.
Runtime `f22c0d8e…`. GUI binary still `8599856d…` (hangup
meetPost lock not this mux). `eno1` `192.168.8.10/30`.
`wg-portal` `10.88.89.163/32`. metal-live now fail-closed
without those DTLS strings. Official `out/` unchanged.
Two-in-room glass (JVB DTLS+SCTP for Eladio) is the operator's.

2026-09-19 19:48 +10: Incremental ninja WebKit 2.52.6 DTLS PEM
**exit 0** (started 18:58, linked 19:40 +10). Artifact
`libwebkit2gtk-4.1.so.0.21.10` `ce805b65…` (126232296 B) contains
`ensureDtlsCertificatePem` / `Applied DTLS PEM` / `X509_digest`.
Copied into slim `webkit-gst-tree-2.52.6`. Seat still
`0c984e8e…`. Not muxed. Not 2.54. Official `out/` unchanged.

2026-09-19 19:45 +10: Lady hangup left Meet up. Call
`02c5c936-…` open 19:40:18, `POST /end` 200 at 19:40:46, **no**
`kind=close`. Bounds stopped; Meet WP `979692` (~798 MB) plus
`createAnswer` until Jitsi `Channel closed` 19:42. Cause: OS
`meetPost` one `__empiierosMeetDone`; 500ms bounds overwrote the
`open` promise; pack `osMeetOpenedRef` stayed false so hangup
skipped `closeMeetStage`. Lock (source, not muxed): wait-map +
id echo; `POST …/video-calls/{uuid}/end` → `hide_meet`
`code=video_call_end`. Overlay-park still needed so hide does not
steal clicks. Lab revive (same binary `8599856d…`) PID `980951`
killed leftover Meet WP `979692`. DTLS ninja not this slice.
Official `out/` unchanged.

2026-09-19 18:58 +10: Factory WebKit 2.52.6 DTLS PEM bake started.
Cause: poet ICE reaches JVB `:13400` then DTLS dies
(`No remote fingerprints`; GST CONNECTING→CLOSED). Stock 2.52.6
leaves `configuration.certificates` as FIXME and
`generateCertificate` fingerprints empty. Lock: dropin
`0002-webrtc-dtls-pem-before-sdp.patch` (living tree + BR package
dir). Incremental ninja of `libwebkit2gtk-4.1.so.0.21.10` (not
2.54, not dirclean). Log
`eliteew-ts-factory-work/webkitgtk-2.52.6-dtls-pem-rebuild.log`.
Mux waits on the `.so`. Official `out/` not this slice.

2026-09-19 18:34 +10: Lab `deploy-webkit-gst`
`20260919T083405Z-webkit-gst-18901` added factory
`libgstinterleave.so` (67288 B, `deinterleave`). First GST mux
`…9645` rolled back: root `: >/tmp/empiieros-meet-gst.log` hits
`fs.protected_regular` in sticky `/tmp`. Lock: recreate the log
as `eliteew-ts`. Same slice: Lab `deploy-shell`
`20260919T083418Z-shell-2687` GUI `8599856d…` wraps
`localDescription` / `remoteDescription` getters with `fixDesc`.
Glass 18:29: first `createAnswer opus ssrc` held, then
`setVideoCodecs` `payload` null, then Meet SIGSEGV gst `bb00d`
at 20. Cause: WebKit `AudioSourceProviderGStreamer`
`factory_make("deinterleave")` then `g_object_set` on NULL
(Jitsi UI sounds / WebAudio). Native getter still returned
`OPUS` after the first lowercase answer. WebKit `0c984e8e…` /
Runtime `f22c0d8e…` / pack `pza6u2zqzf` / `out/` unchanged.
Seat registry has `deinterleave`. One GUI `970572` with
`GST_DEBUG`. Preflight exit 0. In-memory INTERLEAVE strip FAIL
then restore. Two-in-room + settings glass is the operator's.
Cores still armed. Not 2.54.

2026-09-19 18:28 +10: Lab `deploy-shell`
`20260919T082833Z-shell-5275` GUI `aba45257…`. Settings/mic
dropped the living call: small bounds ran `hide_meet` →
`about:blank`; NEW_WINDOW `load_uri` reloaded Meet. Lock: conceal
keeps the WebView; new-window ignore-only. Two-in-room + settings
glass is the operator's. `out/` unchanged.

2026-09-19 18:18 +10: Lab `deploy-shell`
`20260919T081852Z-shell-17619` GUI `3214522b…`. Dummy tracks
worked (`tracks 2`, `createAnswer ssrc sendrecv`) but Jitsi still
`offerAnswerFailed` `payload` null. Cause: WebKit GST rtpmap
`OPUS` (and `VP8`); Jitsi `mungeOpus` does
`const { payload } = mLine.rtp.find(c => c.codec === 'opus')`.
Lock: Meet inject lowercases `a=rtpmap` codecs before Jitsi sees
the answer. Two-in-room glass is the operator's. `out/` unchanged.

2026-09-19 18:07 +10: Lab `deploy-shell`
`20260919T080748Z-shell-6273` GUI `475452e1…`. Cause of poet
join/`gone` ~0.5s: Jitsi `conference.offerAnswerFailed`
`TypeError: Cannot destructure property 'payload'` after WebKit
GST `createAnswer` recvonly with no `a=ssrc` (0 outgoing sources;
PipeWire capture missing; no gUM). Lock: Meet inject dummy
AudioContext+canvas tracks before setRemoteDescription/createAnswer.
One GUI `960322` with kiosk `GST_DEBUG`. WebKit `0c984e8e…` /
Runtime `f22c0d8e…` / pack `pza6u2zqzf` / `out/` unchanged. Two-in-room
glass is the operator's. Cores still armed. Not 2.54.

2026-09-19 18:03 +10: Lab `deploy-shell`
`20260919T080051Z-shell-9014` GUI `0ce716b2…` Meet UserContentManager
diag (`empiierosMeetDiag`: console/reject + RTCPeerConnection wrap;
JWT redacted). TS kiosk + live labwc autostart export
`GST_DEBUG_FILE=/tmp/empiieros-meet-gst.log`. First PID after mux
`957028` had the binary without `GST_*` (old autostart inherit).
Relight of patched autostart: one GUI `958212` with `GST_DEBUG`.
WebKit `0c984e8e…` / Runtime `f22c0d8e…` / pack `pza6u2zqzf` / `out/`
unchanged. Videorate mux did not close poet join/`gone` ~0.5s.
Two-in-room glass is the operator's. Cores still armed.

2026-09-19 17:47 +10: Lab `deploy-webkit-gst`
`20260919T074738Z-webkit-gst-11999` added factory
`libgstvideorate.so` (68848 B), `libgstvideotestsrc.so` (78664 B),
`libgstaudiotestsrc.so` (66296 B). `clocksync` was already in
`libgstcoreelements.so` (not a separate plugin; no mux). WebKit
stayed `so.0.21.10` `0c984e8e…`. GUI revived PID `940402` binary
`8911f356…` with `GST_DEBUG_FILE=/tmp/empiieros-meet-gst.log`.
Kiosk-spawned second GUI `940588` killed. Runtime `f22c0d8e…`.
Seat GST registry contains `videorate` / `videotestsrc` /
`audiotestsrc` / `clocksync` / `rtpbin`. TS lock: frag/defconfig/
allowlist `PLUGIN_VIDEORATE`+`VIDEOTESTSRC`+`AUDIOTESTSRC`. Factory
`make gst1-plugins-base` exit 0. Preflight exit 0. In-memory
adversarial FAIL then restore. Two-in-room glass is the operator's.
Official `out/` unchanged. Cores still armed.

2026-09-19 17:28 +10: Lab `deploy-webkit-gst`
`20260919T072751Z-webkit-gst-25004` added factory
`libgstrtpmanager.so` (606880 B, `rtpbin`+`rtpfunnel`) and
`libgstrtp.so` (801392 B, `rtph264pay`). WebKit stayed
`so.0.21.10` `0c984e8e…`. GUI revived PID `930826` binary
`8911f356…`. Runtime `f22c0d8e…`. Seat GST registry contains
`rtpbin`. First mux `…3648` rolled back: check looked for
`rtpfunnel` in the wrong `.so`. TS lock: frag/defconfig/allowlist
`PLUGIN_RTP`+`PLUGIN_RTPMANAGER`; metal-live/preflight fail
without those plugins. Factory `make gst1-plugins-good` exit 0.
Preflight exit 0. In-memory adversarial FAIL then restore.
Two-in-room glass is the operator's. Official `out/` unchanged.
Cores still armed.

2026-09-19 17:22 +10: Meet `8d317` symbolicated from core
`core.WebKitWebProces.926792.1789802072` (17:14 +10). Cause: not
WebKit 2.52.6. `gst_webrtc_bin_init` → `factory_make("rtpbin")`
NULL → `gst_bin_add(NULL)` → `GST_OBJECT_LOCK(element)` →
`g_mutex_lock(0x18)`. Seat and factory target have **no**
`libgstrtpmanager.so` (`PLUGIN_RTPMANAGER` / `PLUGIN_RTP` off).
Jitsi only builds `webrtcbin` when the second peer joins. Report:
`.cursor/orchestration/reports/fofline_meet_gst_rtpbin_verify.md`.
Lock not applied (need TS frag + `gst1-plugins-good` rebuild +
mux). Not 2.54. Not iframe/FILL. Cores still armed.

2026-09-19 17:13 +10: Lab-only crash cores armed. Meet WebContext
already `sandbox_enabled=FALSE` (same as product; no GUI remux).
Cause of empty cores: `kernel.core_pattern=|/bin/false`. Lock:
`/var/tmp/empiieros-cores/core.%e.%p.%t` + `fs.suid_dumpable=1`.
Restore: `RESTORE.core_pattern` (`|/bin/false`) and
`RESTORE.suid_dumpable` (`0`). GUI `RLIMIT_CORE` already unlimited.
Proof sleep via `sudo -u` could not raise ulimit (not the GUI
child). Next glass two-in-room should drop a core if dumpable.
Official `out/` unchanged.

2026-09-19 17:00 +10: Glass call ended on 2.52.6. Join
`2bebe9a8…` 200 + `kind=open`/`shown` 17:00:34; Meet CSS/JS loaded.
~18s later Meet `WebKitWebProcess` pid `926408` SIGSEGV: `segfault
at 18` ip `g_mutex_lock` (`lock cmpxchg` at glib `8d317`) — `%rdi`
was `0x18` (NULL object + mutex). Same glib `2.82.5` as 2.46.4
crashes. OS hide + `meet_process_crashed` + pack `POST /end`.
FoFline GUI `926147` stayed. Not iframe. Not FILL. 2.52.6 did **not**
remove this UAF/NULL mutex. Official `out/` unchanged.

2026-09-19 16:59 +10: Lab `deploy-webkit-gst`
`20260919T065941Z-webkit-gst-20376`. Live `so.0` → `0.21.10`
sha256 `0c984e8e…` (was `0.16.6` `356011aa…`). JSC `0.10.14`.
Loader clean. RTC string present. GUI revived PID `926147` (binary
not remuxed). `eno1` `192.168.8.10/30`. `wg-portal` `10.88.89.163/32`.
Leftover unused `0.16.6` still on disk. Runtime / pack / `out/` not
this slice. Two-in-room glass is the operator's.

2026-09-19 16:52 +10: Factory `make webkitgtk` 2.52.6 **exit 0**
(~57 min). Target `libwebkit2gtk-4.1.so.0.21.10` 126225624 B
sha256 `0c984e8e003d9bb828395adc2846e3eada5ac5c2198a9ea414aeba9d02827ce3`.
`so.0` → `0.21.10` (not leftover `0.16.6` `356011aa…`). Cache:
`ENABLE_WEB_RTC=ON` `MEDIA_STREAM=ON` `USE_GSTREAMER_WEBRTC=ON`
`USE_LIBRICE=OFF` `USE_SKIA=OFF` `SPEECH_SYNTHESIS=OFF`.
`Enable RTCPeerConnection` in the new `.so`. NEEDED gstwebrtc +
seccomp + JSC `0.10.14`. `metal-live.sh` now hashes `so.0` (not
hardcoded `0.16.6`). Mux not applied yet. Official `out/` unchanged.

2026-09-19 15:55 +10: Configure PASS. Cache: `ENABLE_WEB_RTC=ON`
`ENABLE_MEDIA_STREAM=ON` `USE_GSTREAMER_WEBRTC=ON` `USE_LIBRICE=OFF`
`USE_SKIA=OFF` `ENABLE_SPEECH_SYNTHESIS=OFF`. Ninja `8592` objects
(`-j13`). cmake4 dropin applied. Not 2.54. Mux waits on
`>>> webkitgtk 2.52.6 Installing`.

2026-09-19 15:54 +10: Configure FAIL — 2.52 defaults
`ENABLE_SPEECH_SYNTHESIS` + `USE_FLITE`; no Flite on this floor. Lock:
`ENABLE_SPEECH_SYNTHESIS=OFF` in `external.mk` (Meet does not need TTS).
Not 2.54. Not adding Flite.

2026-09-19 15:53 +10: Factory `make webkitgtk` 2.52.6 started
(`nproc=12`, log `eliteew-ts-factory-work/webkitgtk-2.52.6-build.log`).
Hash `179a2ea3…` OK. cmake4 dropin applied (2.52 hunk:
`framework STREQUAL _linked_into`). CMake last `-D`:
`ENABLE_WEB_RTC=ON` `ENABLE_MEDIA_STREAM=ON` `USE_LIBRICE=OFF`
`USE_SKIA=OFF`. Stale `webkitgtk-2.46.4` tree moved to factory
`trash/`. Not 2.54. Not `.111`. Mux waits on install.

2026-09-19 15:49 +10: Factory WebKit pin 2.52.6 started. Cause: Meet
WebProcess SIGSEGV `8d317` when the second peer joins (2.46.4).
Lock: TS `0005-webkitgtk-2.52.6.patch` + `USE_LIBRICE=OFF` /
`ENABLE_MEDIA_STREAM=ON`; not 2.54 (WebRTC off until 2.56). Bake is
local factory, not `.111`. Mux and glass wait on `make webkitgtk`.
Official `out/` not this slice.

2026-09-19 15:38–15:39 +10: Glass — two in one Meet room drops the
call. Pack `pza6u2zqzf` + GUI `8911f356…`: `kind=open` + `shown` +
`kind=bounds`, no `kind=iframe`. FoFline PID `884185` stayed.
~10s later (second peer / PeerConnection) Meet `WebKitWebProcess`
SIGSEGV `at 18` glib `8d317` — same as 00:00 / 00:42 / 02:39 /
03:34. OS hide + `meet_process_crashed`; pack `POST /end`
(`7b4d067c…` then `b6ad399a…`). Not FILL. Not iframe. Residual is
factory WebKit 2.46.4 / glib, not the bounds hook. Official `out/`
unchanged.

2026-09-19 15:33 +10: Meet-bounds pack uptake. Cause: living
`current/` was `1789750041479-4jxqj4w8lj` (nested iframe). Public
`spa-latest` already `1789794743865-pza6u2zqzf` sha256 `b5c30279…`
(notes: clipped sibling stage; end on Meet death). Lock: helper
`spa-fetch` sync 15:33:20–15:33:38 +10; `current/` id matches;
`VideoCallApp-bs-BnhEY.js` `e3251158…` has bounds hook + lost +
`pagehide`. Groups `assets_core_mismatch` 0/8. GUI `8911f356…`
PID `884185` / Runtime `f22c0d8e…` not remuxed. Official `out/`
unchanged. Syslog `kind=open` / `kind=bounds` waits on product
reload. If white: stop — no iframe, no FILL.

2026-09-19 15:01 +10: Meet bounds clip muxed. Cause: FILL overlay hid
chat; nested iframe loaded Meet in the product WebProcess then
SIGSEGV (03:34) whitened FoFline. EliteEW canon already forbids that
iframe on WebKitGTK. Lock: sibling Meet START-aligned from pack
`{x,y,width,height}`; bare URL `meet_stage_bounds_required`; JS
fullscreen swallowed. Factory gcc GUI `8911f356…`
(`empiieros-shell-gui.meet-bounds-20260919T0500Z`). Mux
`20260919T050146Z-shell-22459` PID `884185`. Assets reused (19).
Runtime `f22c0d8e…` not remuxed. Pack still `4jxqj4w8lj` (iframe).
`eno1` `192.168.8.10/30`. `wg-portal` `10.88.89.163/32`. `sshd`
active. Official `out/` unchanged. Glass of bounds waits on a new
EliteEW pack (`ELITEEW-TZ-FOFLINE-MEET-BOUNDS.md`).

2026-09-19 03:33–03:34 +10: Glass after iframe mux. Meet **did**
load in-page (`kind=iframe` 03:34:10, `all.css` / `lib-jitsi-meet`
/ `app.bundle`, media `code=fofline_iframe`). Then 03:34:15 the
**product** `WebKitWebProcess` SIGSEGV (`at 18` / `8d317` — same
glib as the sibling Meet crash). FoFline went white / full-frame.
No `kind=open`. Join `5a2ab8eb…` **200** `bytes=1454`. Earlier in
the same minute poet also hit leftover `76ab5624…` and created
`2ffdf39c…` then `/end`, then a second GAF chat `d82bff3d…` — that
is how two people can sit in different rooms. EliteEW canon already
forbids nested Meet iframe on WebKitGTK (`DESKTOP-EBROWSER.md`,
`REALTIME-SOCKETIO-AND-WS.md`, `INTERNAL-VIDEO-CALLS.md` Linux Meet
media): paint is blank and PeerConnection is a sibling stage. Our
iframe allow put Jitsi in the product process — crash takes the
whole desktop. Next OS lock is the living Tauri shape: sibling
WebView **bounds over VideoCallApp**, `set_fullscreen(false)`, not
FILL, not iframe. Do not restore the overlay. Not PASSED.

2026-09-19 03:32 +10: Product Meet iframe policy muxed. Cause: product
`decide_policy` ignored `https://meet.eliteew.ru` so join 200 left a
white rectangle. Lock: subframe Meet allowed when the product URI is
`fofline-spa` and `meet_host_ok`; user-gesture main-frame Meet denied;
deny syslog `code=` without query. Product WebView USER_MEDIA /
DEVICE_INFO allowed on fofline-spa / meet host. Mux
`20260918T173205Z-shell-31586` GUI `6e0a4511…` PID `359122`. Assets
reused from live `/opt/empiieros/shell` (19 files). Runtime
`f22c0d8e…` not remuxed. Official `out/` unchanged. Preflight PASS
(rustup cargo 1.98). Glass: poet reopens FoFline and calls — expect
`kind=iframe` then Meet `edge start`. Nested paint may still be
white. Not PASSED.

2026-09-19 03:05–03:09 +10: Glass — iframe white. Pack
`4jxqj4w8lj` served (`VideoCallApp-BhyD_kAW.js`). No
`meet stage request kind=open`. Join `db20d179…` and
`76ab5624…` **200** `bytes=1454` (join_url class). Then **zero**
`fofline edge start` for `meet.eliteew.ru` / `wss:` / `socket.io`
until at least 03:10. Overlay Meet at 02:39 *did* fetch Meet
assets. Cause: product `decide_policy` allows only `empiier:` /
`fofline-spa:` / `about:blank` and **ignores**
`https://meet.eliteew.ru/…` iframe navigation (`g_warning` only —
not syslog). Missing Socket.IO is real after this reload and does
**not** explain a sterile white frame: HTTP join already returned
the URL; Jitsi WS starts only after Meet HTML loads. `/end` still
200. Do not restore the overlay. Next OS slice is a subframe allow
for `meet_host_ok` plus syslog of the deny — then re-measure paint.
Not PASSED.

2026-09-19 02:58 +10: In-window call pack uptake. Cause: living
`d2ufr39pnb` called `openMeetStage` (FILL overlay hid chat) and did
not `/end` on Meet death. Public `spa-latest` already
`1789750041479-4jxqj4w8lj` sha256 `9296bc5a…` (HTTP 200; notes:
call stays in VideoCall; end on Meet death; dist-build
`2026-09-18T16:50:26Z`). Lock: helper `spa-fetch` sync
02:58:26–02:58:39 +10 over VIP. Measure: `spa_state=ready`
`spa_app_build_id=1789750041479-4jxqj4w8lj`;
`assets_core_mismatch` 0/8 (groups still `rovukoqfw9`). Packed
`VideoCallApp-BhyD_kAW.js` `474f8efd…`: `openMeetStage=0`,
iframe JSX, `empiieros-meet-stage-lost` + `pagehide`. Keepalive
`POST /api/v1/video-calls/{id}/end` in
`DesktopSocketNotificationsBridge-DItrm048.js`. GUI `39b3bbee…`
PID `259070` / Runtime `f22c0d8e…` not remuxed. Official `out/`
unchanged. Product WebKit still holds the previous JS — glass
(no `kind=open`, Meet inside VideoCall, chat stays, hangup `/end`,
crash `/end`) waits on poet reopen. Not PASSED.

2026-09-19 02:39 +10: Glass — call is OS Meet, not inside EliteEW.
Pack `d2ufr39pnb` `open` 02:39:37 + `shown` for `210afcb5…`. Same
glib SIGSEGV 02:39:44. New GUI hid + `crashed reason=meet_process_crashed`.
No `/end` after that join (pack does not listen). Overlay FILL is why
the picture is not a FoFline window. Official `out/` unchanged.

2026-09-19 01:03 +10: Meet-death GUI muxed to the lab seat.
Cause: overlay crash left lady with poet on the list (no `/end`).
Lock: factory gcc GUI `39b3bbee…` (`web-process-terminated` → hide +
`empiieros-meet-stage-lost`). Mux `20260918T150331Z-shell-28108`.
Assets reused from live `/opt/empiieros/shell` (19 files). Measure:
live binary hash matches; strings `empiieros-meet-stage-lost` /
`meet_process_crashed` / `fofline meet stage crashed reason=`; GUI
PID `259070`. Pack still `1789730765446-d2ufr39pnb` (no listener).
Runtime `f22c0d8e…`. Official `out/` unchanged. Glass `/end` waits
on a new pack. Not PASSED.

2026-09-19 ~00:50 +10: Meet-death roster lock in source (not muxed).
Cause: Meet SIGSEGV; pack never `/end`; lady kept poet on the list.
Lock: GUI `web-process-terminated` → hide +
`empiieros-meet-stage-lost`; pack `/end` on that event and pagehide
keepalive. Measure: preflight strings + EliteEW vitest. OS does not
POST video-calls. Pack `d2ufr39pnb` still silent. Official `out/`
unchanged. Not PASSED.

2026-09-19 00:42 +10: Meet WebProcess crashed twice (same glib).
Cause: `WebKitWebProcess` SIGSEGV `at 18` in
`libglib-2.0.so.0.8200.5` offset `8d317` (00:00:39 pid 225730,
00:42:59 pid 227270) after `meet stage shown` + media allow.
GUI `eb8148d7…` PID `224201` lived. Pack `d2ufr39pnb`. After the
second crash call `ca098770…` had join + open, **no** `/end`.
First crash room `544dae43…` had no `/end` for ~42 min. Lady
still seeing poet in the list is leftover roster, not a live
camera. Official `out/` unchanged. Not PASSED.

2026-09-19 ~00:26 +10: EliteEW tree closeout (not a pack).
Cause: overlay pack hid chat and painted Tauri placeholder.
Lock on disk in monorepo: `VideoCallApp` iframe like browser;
no `openEmpiierosMeetStage`; hangup still `POST /end`; chat
open test does not close `GAFChatApp`. Measure claimed by
EliteEW session (iframe + chat-open + W5). EmpiierOS did not
re-run those tests and did not touch the seat. Public pack
still `1789730765446-d2ufr39pnb` (overlay). Official `out/`
unchanged. Glass of iframe path waits on a new pack. If that
iframe is white: stop — do not restore the overlay.

2026-09-19 ~00:04 +10: Glass — Meet joined (lady sees poet
presence; system audio). Chrome wrong: second `VideoCallApp`
window + OS Meet FILL overlay hides chat; pack paints
«Видеозвонок идёт / картинка встречи поверх этого окна». Cause:
EliteEW EmpiierOS branch calls `openMeetStage` and empty-state
copy. Lock: TZ
`docs/plans/ELITEEW-TZ-FOFLINE-CALL-CLOUD-LIKE.md` — iframe like
cloud; no OS overlay on a default call. Official `out/` unchanged.
Not PASSED.

2026-09-18 23:56 +10: WebKit WebRTC muxed to the lab seat.
Cause: Jitsi banner — stock WebKit `ENABLE_WEB_RTC=OFF`, no GST.
Lock: `metal-live.sh deploy-webkit-gst` (allowlist + rollback).
First activate failed: new WebKit `NEEDED libseccomp.so.2`, seat
had none (old .so did not). Shipped factory `libseccomp.so.2.5.5`.
Mux `20260918T135636Z-webkit-gst-5105`. Measure: WebKit
`356011aa…` 106876056 B; loader `libgstwebrtc-1.0` +
`libgstreamer-1.0` + `libseccomp`; `libgstwebrtc.so` /
`libgstopus.so` / `libgstvideo4linux2.so` / `libnice.so.10`;
`Enable RTCPeerConnection`; Pulse plugin absent. GUI stayed
`eb8148d7…` PID `224201`. Runtime `f22c0d8e…`. `eno1`
`192.168.8.10/30`. `wg-portal` stayed. Official `out/` not
rewritten. Glass call is the operator's.

2026-09-18 23:21 +10: WebKit WebRTC from-source compile finished
in factory `BR_OUT` (not live). Measure: CMake
`ENABLE_WEB_RTC:BOOL=ON` / `ENABLE_VIDEO:BOOL=ON`; summary
`USE_GSTREAMER_WEBRTC ON`; `libwebkit2gtk-4.1.so.0.16.6` 106876056 B
sha256 `356011aa…` mtime 23:20:33 +10; NEEDED
`libgstreamer-1.0` + `libgstwebrtc-1.0`; strings
`Enable RTCPeerConnection`. Target also has `libgstwebrtc.so`,
`libgstopus.so`, `libgstvideo4linux2.so`, `libnice.so.10`. Pulse
plugin absent. `gst1-plugins-good` 23:21:49Z. Official `out/` not
rewritten. Live seat still GUI `eb8148d7…` without `libgst*` — mux
is a separate ask.

2026-09-18 22:30 +10: WebKit WebRTC from-source compile started.
Cause: living `BR_OUT/.config` still had `# WEBKITGTK_MULTIMEDIA is
not set` and CMake `ENABLE_WEB_RTC:BOOL=OFF` / `ENABLE_VIDEO:BOOL=OFF`.
Lock: merge-config wrote MULTIMEDIA + gst webrtc/opus/v4l2 into
`eliteew_ts_x86_64_defconfig`; patch `0004` on living `webkitgtk.mk`
(common OFF + multimedia ON); `external.mk` last `-DENABLE_WEB_RTC=ON`
+ `gst1-plugins-good` dep. Targeted `olddefconfig` +
`webkitgtk-reconfigure` (not `--resume`, no official `out/`).
CMake last `-D` is ON; cache `ENABLE_WEB_RTC:BOOL=ON` /
`ENABLE_VIDEO:BOOL=ON`; summary `USE_GSTREAMER_WEBRTC ON`.
GST 1.24.11 + libnice 0.1.21 + `libgstwebrtc.so` /
`libgstopus.so` already in `BR_OUT/target`. WebKit 2.46.4 ninja
`~300/1859`. Log `webkit-webrtc-20260918T1230Z.log`. `gst1-plugins-good`
(v4l2) is kconfig-on but not a stock WebKit dep — build after ninja.
Live seat still GUI `eb8148d7…` without `libgst*`.

2026-09-18 22:25 +10: WebKit WebRTC factory lock. Cause: Jitsi banner after
`shown` — stock `webkitgtk.mk` `ENABLE_WEB_RTC=OFF`, no GStreamer.
Lock: fragment `WEBKITGTK_MULTIMEDIA` + gst webrtc/opus/v4l2; allowlist;
`external.mk` `-DENABLE_WEB_RTC=ON`; patch `0004`; resume reconfigure
requires CMake `ENABLE_WEB_RTC:BOOL=ON`. Compile from source next
(hours). Official `out/` not rewritten this turn. Live GUI still
`eb8148d7…` without GST.

2026-09-18 22:17 +10: Meet page loaded; Jitsi WebRTC banner. Cause:
stock Buildroot `webkitgtk.mk` `-DENABLE_WEB_RTC=OFF`; seat has no
`libgst*` / libnice. GUI already `enable_webrtc` + `shown`. Measure:
22:17:31 `request kind=open` / `shown`; edge `all.css` +
`lib-jitsi-meet` / `app.bundle`; no `media permission`. Hangup
22:18:54 `kind=close`. Next image, not a pack iframe. Official `out/`
unchanged.

2026-09-18 22:16 +10: Meet host-len mux. Lock from 22:05 now live.
Mux `20260918T121648Z-shell-29442` GUI `eb8148d7…` PID `176252`.
Strings: `request kind=open|close`, `denied code=`, `charAt(23)`.
Assets kept `index-ChjX8sOO.js`. Pack still `d2ufr39pnb`. Hostmap
app/auth/meet. `wg-portal` `10.88.89.163/32`. Hook planted 22:16:58
after mux (product show). Official `out/` unchanged. Glass call is
the operator's — expect `shown` or `denied code=`.

2026-09-18 22:05 +10: Meet false-hope. Cause: poet copy «Соединение
установлено / Видео недоступно в этом окне» after join **200**; lady
waited in Jitsi; OS never `shown`. `MEET_HOST_LEN` was 14, host is 15
chars — every `openMeetStage` was silent `join_url_denied`. Hangup
`/end` still ended the callee. Lock: length 15 + `_Static_assert`;
deny/request syslog; JWT stops at `#`; TZ fail-closed `/end` if
`!ok`. Later mux 22:16. Official `out/` unchanged.

2026-09-18 21:53–21:56 +10: glass after hangup-pack reload. Product opened
21:53:49 and 21:54:55 (`hook planted`, `VideoCallApp-IsuY-m0b.js` served).
GAF create 21:55:08 **200** bytes=403. Join `670617ff-…` **409** bytes=220
×4 (ring), then **200** bytes=1454 at 21:55:20. No
`fofline meet stage shown` this boot. No `fofline edge start` today.
No `fofline meet media permission`. Hidden only on product show/hide.
Meet WebKit never loaded — not a dead camera. Pack/JS open either
never posted or C denied without syslog. Official `out/` unchanged.

2026-09-18 21:50 +10: Meet hangup pack uptake. Cause: EliteEW close
landed in public `1789730765446-d2ufr39pnb`; lab `current/` was still
`5fjlwh96mc` (open without close). Lock: helper `spa-fetch` sync
21:49:53–21:50:16 +10 over VIP. Measure: `spa_state=ready`
`spa_app_build_id=1789730765446-d2ufr39pnb` sha256 `c34c224e…`;
`assets_core_mismatch` 0/8 (groups still `rovukoqfw9`). Packed
`VideoCallApp-IsuY-m0b.js` `ce24cbc6…` has `closeMeetStage()` with no
URL and missing-hook `meet_stage_unavailable`. Hostmap still
app/auth/meet. GUI PID `139045` / Runtime `f22c0d8e…` / `out/` not
remuxed. `fofline meet stage shown` not seen — product WebKit still
holds the previous JS until poet re-opens the environment and calls
again. Camera/mic after Meet loads is the OS media path.

2026-09-18 20:50 +10: Meet stage. Cause: poet glass had no picture —
hook returned `meet_hostmap_unavailable`, `meet.eliteew.ru` resolved
public (`93.77.165.185`) and prelicense dropped both public 443 and
JVB UDP 13400. Lock: second WebKit (`meet_surface.c`); hostmap Meet
on VIP; nft `udp dport 13400 ip daddr 93.77.165.185`. Measure: VIP
SNI `config.js` HTTP 200 `CN=*.eliteew.ru`. Mux GUI
`20260918T105003Z-shell-32405` `8bbb8235…` PID `139045`; Runtime
`f22c0d8e…`; portal helper `a13bf131…`. `eno1` `192.168.8.10/30`.
`wg-portal` `10.88.89.163/32`. `sshd` active. Official `out/`
unchanged. EliteEW still owns close-on-end
(`docs/plans/ELITEEW-TZ-MEET-ON-OS.md`). Product reload + new call
are the operator glass walk.

2026-09-18 19:47 +10: speaker amp. Cause: mixer and PCM already
worked; Dell ALC3204 GPIO was all-off (no `0x09c1` quirk). Lock:
from-source hwdep GPIO `0x07` in `empiieros-audio-volume` plus boot
`empiieros-audio-amp.service`. Operator heard 440 Hz after the live
GPIO write. Helper mux `638d0046…`; `--observe` re-enabled GPIO after
an explicit zero. Amp unit `enabled`/`active`. Runtime `c6d3f8d2…` /
GUI `040a096a…` / `out/` unchanged. Kernel quirk still next image.

2026-09-18 19:13–19:15 +10: glass call poet (OS) ↔ lady_boss
(Chrome). Pack `5fjlwh96mc` reloaded 19:13; GAF `220c6689…` 200;
`POST …/video-calls/gaf-chats/…` 200 bytes=403; `POST …/join` 409
bytes=220 (no `reason=` key; ringing / awaiting accept). WebKitNetwork
ESTAB to `10.88.89.1:443` (not Soup). No `fofline edge` line. Poet
window is OS pending copy (video not in this window) — hook
`meet_hostmap_unavailable`, no iframe. Lady 502 is public Meet:
`https://meet.eliteew.ru/` 403 (jwt-gate), `config.js` **502** nginx
on `93.77.165.185`. `app.eliteew.ru/api/health` 200. Hostmap still
app/auth only. Official `out/` unchanged.

2026-09-18 19:10 +10: FoFline pack Socket.IO uptake. Cause: live
`current/` was `1789443659753-rovukoqfw9` (`location.origin` /
`fofline-spa:`). Public `spa-latest` already
`1789720073125-5fjlwh96mc`. Lock: helper `spa-fetch` sync
19:09:41–19:10:05; `current/` id matches; client `kt=https://app.eliteew.ru`
on opaque scheme. Groups `assets_core_mismatch` 0/8. Runtime/GUI/`out/`
unchanged. One pending-incoming VIP timeout during fetch, then 200.
Handshake `fofline edge` not yet seen — product WebKit still needs a
reload. Meet hostmap/surface still closed.

2026-09-18 18:52 +10: FoFline calls OS lock. Cause: W5 + Socket.IO +
Meet lived in EliteEW; the OS seat still had no Meet hook and hid GAF
from the connected surface while the pack (when rebaked) will call
them. Lock: pack names `GAFChatApp`/`VideoCallApp`; inject
`EmperyDesktopMeetStage` fail-closed (`meet_hostmap_unavailable` /
`join_url_denied`). Mux `20260918T085216Z-shell-21697`
`index-ChjX8sOO.js` / GUI `040a096a…` PID `103264` (`product chrome
attached hud`). Live hashes match local `dist`. Hostmap
`10.88.89.1 app.eliteew.ru auth.eliteew.ru` (no Meet). Pack still
`1789443659753-rovukoqfw9` (15:22). Runtime `c6d3f8d2…` not remuxed.
Official `out/` unchanged. Adversarial: strip `EmperyDesktopMeetStage`
→ preflight FAIL → restore PASS. Glass pack/W5/Socket.IO/Meet media
still wait on stack0 + a new pack + later Meet surface.

Sandbox-off remux was the wrong cause on this seat 2026-09-18 12:15–12:16
+10: this WebKitGTK is `ENABLE_BUBBLEWRAP_SANDBOX=0`. After mux GUI
`1d6b68d1…` glass Open 12:16:45 inject `applied`, FoFline CSS already
served at 12:15:43 with CORS count 0 (`preloader` / `mainFullApp` /
`styles` / `600`). `ss` still no `:443`. The only resource FAIL is
`/fonts/elite-quote/*.woff2` — that directory is absent; the pack has
Cormorant/Manrope under `assets/`. No pack `/api/` request was logged.
Living boot `fetch('/api/v1/system/maintenance')` is a relative URL
(`fofline-spa://localhost/...`), not the VIP. Axios to
`https://app.eliteew.ru` never started. Static `#app-preloader` stays
until React commit (`Promise.all` waits on `reactAppEntry`). Quote-font
404 is not the HTTPS path. Sandbox-off stays (auth/seat already off);
it is not the hang lock. Official `out/` unchanged.

2026-09-20 01:10 +10: always-on seat HUD. Cause: peek
`margin-top: -48px` plus FoFline-only show looked like a hide bug
and hid the only fast-access cluster; `audio.action` is still
production-disabled. Lock: pinned chrome from kiosk start; one
popover for observed camera/mic, `gnss_not_on_seat`, Seat Shield
status/events/observe; `hide_product` keeps the HUD. Mux Shell
`20260919T151033Z-shell-10085` GUI `e4b79cd3…` PID `84222`
`product chrome attached always-on seat chrome` +
`shell render ready attempts=1` `index-CK6nPRzU.js` `d1100a06…`.
Runtime `19bf5f7f…` not remuxed. `IPAddressDeny` stays. Official
`out/` unchanged. Unified OS A/B OTA and pack `audio-guides` still
absent.

2026-09-20 01:33 +10: FoFline MusicPlayer content download. Cause:
`download-by-ticket` returned 1.8 MiB then WebKit cancelled
`fofline-spa://` because the proxy passed `audio/*` without CORS
headers. Lock: those paths served as `application/octet-stream` +
CORS + `Content-Length` (64 MiB / 120s). Mux Shell
`20260919T153302Z-shell-7904` GUI `8f4680f1…` PID `90036`
`content=octet-stream`. Runtime not remuxed. Official `out/`
unchanged. Operator Open + Play is the remaining measure.

2026-09-20 01:43 +10: MusicPlayer still spun. Cause: ticket GET was
`206` `1446` bytes (`Range` probe). Proxy treated 206 as a full blob.
Lock: strip `Range` on content downloads; `206` is
`api_proxy_partial_content`. Mux `20260919T154330Z-shell-15242` GUI
`cc0f4af2…`. Play after Open is the measure.

2026-09-20 01:53 +10: Play again. Ticket now `200` `1851567`
`content=octet-stream` (no `206`), then still
`Load request cancelled` and `/download` 1.8 MiB. Cause: credentialed
`fetch` + `ACAO *`, and `<audio src>` on `fofline-spa:` still cancelled
after a full body. Lock: echo `fofline-spa://localhost` +
`Allow-Credentials`; rewrite those media `src` to `blob:`
(`__empiierosAudioBlob`). Mux `20260919T155305Z-shell-9402` GUI
`ff8b8213…`.

2026-09-20 02:01 +10: Play on that GUI. Ticket +
`download-by-ticket` `200` `1851567` `octet-stream`, no cancel, no
`/download` fallback, spinner stayed. Cause: stream mode sets
`<audio src>` to the ticket URL; `canplay` never fires on
`octet-stream`+nosniff, and the blob inject swallowed `src` without
`load()`. Lock: sniff audio MIME for media-Range clients; inject
calls `load()` after the `blob:`. Mux `20260919T161705Z-shell-21228`
GUI `534864d3…`.

2026-09-20 02:20 +10: Transparent layer ate FoFline clicks. Cause:
always-on HUD event box is app-paintable transparent and GtkOverlay
can allocate it larger than the cluster; GtkPopover is modal and
grabs the seat. Lock: input-shape the chrome window to the cluster;
`gtk_popover_set_modal(..., FALSE)`. Mux
`20260919T162303Z-shell-7858` GUI `680ca95b…`.

2026-09-20 02:25 +10: HUD stayed fully expanded after hover/click
and kept eating the seat. Cause: always-on was implemented as
always-open (`margin-top: 10`). Lock: 1/4-height peek
(`chrome_peek_margin`); hover expands; leave/actions
`chrome_schedule_peek`. `-48px` stay forbidden. Mux
`20260919T162655Z-shell-29977` GUI `ca20351b…`.

2026-09-20 02:38 +10: FoFline sound TZ for EliteEW. Cause: pack
plays Chromium media URLs (`download-by-ticket` / `blob:`) on
`fofline-spa:`; GStreamer cancels and holds ALSA. Lock:
`docs/plans/ELITEEW-TZ-FOFLINE-AUDIO-ON-OS.md` — decode+Web Audio
on `foflineNative`, no media `src`. Not a pack bake from this
session.

2026-09-20 02:45 +10: HUD peek against the window edge. Cause:
GtkOverlay clamps negative `margin-top`. Seat GTK has no
`gtk_widget_set_overflow`. Lock: `GtkScrolledWindow` clip host,
lip = 1/4 cluster height (`chrome_apply_clip`). Mux
`20260919T164914Z-shell-18165` GUI `de7be00f…` PID `116418`
syslog `peek=clip-1/4`.

2026-09-20 03:12 +10: Decoded-playback pack uptake. Cause: seat
`current/` was `1789794743865-pza6u2zqzf` (media URL / blob cancel).
Public `spa-latest` already `1789837366816-w7k51h8btf` sha256
`d3f6f667…`. Lock: helper `spa-fetch` sync; `current/` id matches;
`contentApi-Bstp_TrW.js` `11d0e5ad…` has `decodeAudioData` /
`createBufferSource` and skips `bytes=0-0` on `foflineNative`.
Groups `assets_core_mismatch` 0/8. GUI `de7be00f…` PID `116418`
not remuxed. Official `out/` unchanged. Play syslog waits on
product reload.

2026-09-20 03:24 +10: HUD icons + dark popover. Cause: shield text
chip resized the cluster every 12s; Adwaita `GtkSwitch` painted a
white trough. Lock: five ComfyUI icons (`192.168.1.136:31003`,
Flux.2 Klein 4B); `chrome_pointer_inside`; popover CSS at
`PRIORITY_USER`. Prompt ids in
`runtime/shell-gui/product-chrome/ORIGIN.md`. Mux
`20260919T173152Z-shell-15303` GUI `b5873fb5…`.

2026-09-20 03:32 +10: OS start splash. Cause: operator clip sat at
repo root. Lock: `runtime/shell-gui/product-chrome/start_os.mp4` +
`boot_splash.c` playbin/`alsasink` volume 0.25. HUD after EOS.
First mux `20260919T173805Z-shell-8074` GUI `3424d0b1…` PID
`157210`: attached `gtkwaylandsink`, then 20s failsafe, no EOS.
Cause: PLAYING likely never completed (wayland sink preroll /
`map-event`). Lock: `gtksink` first + realize/map start. Remux
`20260919T174207Z-shell-111` GUI `2a77ce31…` PID `160622`:
`started ret=2`, `NULL→READY→PAUSED→PLAYING`, finish 8s later
(clip length), no failsafe. Volume 0.25 is playbin soft-volume.

2026-09-20 03:48 +10: HUD per-icon flyouts. Cause: launcher/Shield
opened the app; EliteEW `workspace=eliteew` was a no-op; volume
aliased the sensor panel; `paplay` first on a Pulse-less seat.
Lock: five `chrome_toggle_flyout` popovers; Open is a button;
EliteEW Open = `product.open`; `aplay -q` first. Mux
`20260919T174854Z-shell-23877` GUI `1687881e…` PID `4777`
`peek=clip-1/4`. Flyout/click syslog waits on operator clicks.

2026-09-20 03:51 +10: HUD missing after splash. Cause: finish used
`gtk_widget_show` while chrome `no_show_all` skipped children in
GTK3. Lock: `empiier_product_chrome_show` + syslog
`boot splash revealed hud`. Mux `20260919T175453Z-shell-13698`
GUI `39ced62f…` PID `8397`: splash PLAYING 8s then `revealed hud`.

2026-09-20 03:56 +10: HUD to the right edge. Cause: overlay was
top-center horizontal. Lock: `GTK_ALIGN_END`/`CENTER`, vertical
cluster, width peek `clip-1/4-end`, flyouts `GTK_POS_LEFT`. Mux
`20260919T175727Z-shell-30725` GUI `c933ea8d…` PID `10051`
syslog `peek=clip-1/4-end` then `revealed hud`.

2026-09-20 04:01 +10: Drop HUD peek. Cause: scrolled 1/4 clip never
hid behind the edge and felt like a crutch. Lock: full vertical
dock, `dock=end no-peek`; gates forbid `chrome_schedule_peek` /
`gtk_scrolled_window_new`. Mux `20260919T180226Z-shell-23391`
GUI `b94fa6be…` PID `14248` syslog `dock=end no-peek`.

2026-09-20 04:05 +10: HUD clicks only on launcher. Cause:
`click-eliteew.wav` peak 325 vs launcher 11820; `aplay` already ran
for every icon. Lock: gain-normalize to ~11820; gates require peak
>= 8000. Mux `20260919T180528Z-shell-13731`.

2026-09-20 04:12 +10: HUD icon regen. Cause: operator asked ComfyUI
retile; first Flux pass collapsed to map-pins (volume=wifi,
cam=bulb). Lock: shipped prompt ids in
`runtime/shell-gui/product-chrome/ORIGIN.md` (`d6d55a37…` launcher
3×3, `f7a358da…` sliders, `f3afd463…` speaker, `28a80ff2…` camera;
`E`/shield/mic/GPS from the same queue). Native/public SHA match.
Mux `20260919T181303Z-shell-9297` GUI `b94fa6be…` (unchanged).

2026-09-20 04:21 +10: Living docs vs code. Cause: session/docs still
described HTTP `:17420`, Pulse `audio.action`, and a catalog face
while production is `secure_main.c` + `empiier://` + ALSA
`audio.control` + launcher journey + HUD + Seat Browser overlay.
Lock: `session/SEAT-FACE.md` plus rewritten SHELL-GUI / AUDIO /
SEAT-BROWSER / CONTRACT banner; START-HERE / NORTH-STAR / ledger /
tree point at the glass map. Not MATCH. Official `out/` unchanged.

2026-09-20 04:32 +10: HUD glass measure (plan item 1). Live GUI
`b94fa6be…` PID `22262`: `dock=end no-peek`, splash `revealed hud`,
then flyouts launcher/eliteew/panel/shield/volume + `click alsa`.
Not official `out/`. Seat Browser walk is next: current policy is
allowlist `ya.ru` only (not VIP); no `seat browser shown` on this
PID.

2026-09-20 05:27 +10: Lab `deploy-shell`
`20260919T192718Z-shell-17283` GUI `b98da674…` PID `81672`.
Cause: wheel used `window.scrollBy` on a viewport-locked face;
product WebView lacked `media_stream`. Lock: view-only
`elementFromPoint` → nearest `overflow:auto`; product
`enable_media_stream=TRUE`. Splash `revealed hud`. Camera USB
`0bda:5676` `Integrated_Webcam_HD` is on the bus; no
`/sys/class/video4linux`, no `uvcvideo`, no `/dev/video*`.
`eliteew-ts` is in `video`. Glass wheel is the operator's.
Official `out/` unchanged. Not a reinstall.

2026-09-20 05:40 +10: Seat Shield S4 paste (wave P) in source.
Cause: OS crate 0.4.0 left `exe_sha256` empty and cycle never
applied protected blocks; HUD ticks status, not observe.
Lock: vendored `seat_shield_core` 0.5.0; `status` calls
`maybe_auto_observe`; contracts + Shell knobs. Defaults stay
observe-only. Not lab mux. Not official `out/`. Not MATCH.

2026-09-20 05:43 +10: Lab Seat Shield S4 mux (wave L).
`deploy-runtime` `20260919T194306Z-runtime-10850` `98710238…`
PID `87430` `NRestarts=0`. `deploy-shell`
`20260919T194317Z-shell-13927` GUI `62d66bd2…` PID `87639`.
Frontend `seat.shield.status`: `act_enabled=false`
`auto_act_protected=false` `protected_ports=[22]`
`integrity_twin_state=disabled` `backend=unavailable`
(no `/usr/libexec/empiieros-seat-shield-fw` on this image).
`last_observe_backend=proc_net_tcp` (status auto-observe).
Observe `note=preview-only; cycle did not mutate floor`
`applied_acts=[]` proposed preview for LAN `192.168.1.136:22`.
Official `out/` unchanged. Not a reinstall. Not MATCH.

2026-09-20 05:59 +10: Official Seat Shield S4 bake (wave B).
Cause: lab helper missing because this seat is not the
2026-09-18 image; overlay wants copy was ignored by
systemd 256 (not a symlink).
Lock: kinship post-build asserts fw+nft+nftables; wants
replaced with symlink via `_enable_systemd_unit`;
`image_contract_ok` requires that symlink.
Measure: `--build-image` then `--resume` exit 0.
overlay `f23ec225…` Runtime `a84e45eb…` GUI `3b8992b1…`
fw `067ef5a2…` Shell `index-i37hKALm.js`.
`out/rootfs.ext2` `c7aea2e4…` `rootfs.tar.gz` `adca8ca2…`
bzImage `db616a36…` (unchanged). QEMU: Seat Shield
nftables unit Started; smoke FAIL missing
`WebKit sandbox enabled` (product context off).
`qemu-installed-system-smoke.sh` not rerun.
Lab `192.168.8.10` not reinstalled. Not MATCH.

2026-09-20 06:06 +10: Lab mux of official S4 bake (not a reinstall).
`deploy-runtime` `20260919T200527Z-runtime-9560` `a84e45eb…`
PID `105910` `NRestarts=0`. `deploy-shell`
`20260919T200536Z-shell-30501` GUI `3b8992b1…` PID `106078`
Shell `index-i37hKALm.js`. Floor
`20260919T200545Z-seat-shield-floor-1588609` fw `067ef5a2…`
wants symlink; `nft list table inet seat_shield` has empty
`blocked4`/`blocked6`. Observe `preview-only` `applied_acts=[]`
peer `192.168.1.136:22`. Status `actuator_failed`: this seat
`nftables v1.1.0` has `JSON support not compiled-in`. Auto-act
off. Not MATCH.

2026-09-20 06:13 +10: Seat Shield status errors on glass.
Cause: HUD `seat.shield.status` lists via helper `nft -j`;
this seat `nftables v1.1.0` has no JSON.
Lock: helper falls back to text `nft list set`.
Measure: fw `20260919T201248Z-seat-shield-fw-1610499`
`732e3b9e…`. Helper `list` `ok peers=[]`. Status `ok`
`backend=helper` `act_enabled=false`. Auto-act off. Not MATCH.

2026-09-20 06:32 +10: Seat Shield source lock after HUD errors.
Cause: official `out/` still has JSON-only helper; card
wiped observe errors on refresh; living kinds had no labels.
Lock: `parse_nft_set_text` + preflight needles; contracts;
event-kind locale; observe keeps `note` / error after refresh.
Measure pending: preflight/bake/mux not re-run this turn
(sandbox mount-count 4096). Lab helper still `732e3b9e…`.
Official `out/` still `067ef5a2…`. Auto-act off. Not MATCH.

2026-09-20 06:42 +10: Seat Shield hardening H2–H5 in source.
Cause: no RELRO/NX rustflags, no panic=abort, no audit script,
parsers unfuzzed, helper units lacked a jail.
Lock: packager + `.cargo/config.toml` gnu PIC/RELRO/NX;
`panic=abort` release; `scripts/runtime-cargo-audit.sh`;
garbage tests; oneshot units `CAP_NET_ADMIN` / no `IPAddressDeny`.
Measure pending: sandbox mount-count 4096 blocked cargo/preflight.
H1 official bake and H6 systemd +libseccomp not started.
No musl. No AF_PACKET. Auto-act off. Not MATCH.

2026-09-20 06:48 +10: Seat Shield H2–H5 measured (not H1 bake).
Cause of first preflight FAIL: unit comment contained
`IPAddressDeny`; needle was a substring.
Lock: needle is `IPAddressDeny=`.
Measure: crate tests 48 ok; Runtime lib 155 ok;
`scripts/runtime-cargo-audit.sh` 0 (`cargo-audit` SKIP);
`scripts/preflight-foundation.sh` RESULT PASS.
Cross gnu Runtime `0a0c2d42…` fw `5282e024…`:
`GNU_RELRO`, `GNU_STACK RW`, `BIND_NOW` + `PIE`.
fw strings include `nft text list`. Official `out/` still
`067ef5a2…`. Auto-act off. Not MATCH.

2026-09-20 07:00 +10: Seat Shield H1 official re-bake.
Cause: official helper was JSON-only; first `--build-image`
false-FAIL at amdgpu (pipefail + `find | rg -q` SIGPIPE 141
while 638 firmware files were present).
Lock: kinship post-build uses `find -print -quit`; helper
already has `parse_nft_set_text`.
Measure: `--resume` `RESUME_EXIT=0` `image_contract_ok`.
Overlay `daa1a5f9…`. `out/rootfs.ext2` `c9805fc6…`
stamp `89266529…`. Image Runtime `04358653…` fw `1a3856a2…`
(`nft text list`, ≠ `067ef5a2…`) GUI `3b8992b1…`
Shell `index-Cxzmlg2S.js`. `readelf`: `GNU_RELRO`,
`GNU_STACK RW`, `BIND_NOW` + `PIE`. QEMU: Shield nftables
Finished; systemd `+SECCOMP`; Runtime `seccomp mode=2`;
smoke FAIL `WebKit sandbox enabled` (kept). Lab 192.168.8.10
not reinstalled (live fw still `732e3b9e…`). Auto-act off.
Not MATCH. H6/S5 not started.

2026-09-20 07:03 +10: Seat Shield H1 mux onto 192.168.8.10.
Cause: lab still ran S4 Runtime `a84e45eb…` / fw `732e3b9e…`
/ Shell `index-i37hKALm.js` after the official H1 bake.
Lock: `metal-live.sh` deploy-seat-shield + deploy-runtime
(live unit `dd6bb219…`) + deploy-shell. No reinstall.
Measure: `20260919T210328Z-seat-shield-24838` fw `1a3856a2…`;
`20260919T210329Z-runtime-25348` Runtime `04358653…`
PID `156432` `NRestarts=0`; `20260919T210331Z-shell-17680`
GUI `3b8992b1…` `index-Cxzmlg2S.js`. Status `ok`
`backend=helper` `act_enabled=false` `auto_act_protected=false`
`integrity_twin_state=disabled`. Observe preview-only
`192.168.1.136:22`. Helper `list` `ok peers=[]`. Live systemd
256.7 `-SECCOMP`; Runtime `Seccomp: 0`. Auto-act off. Not MATCH.

2026-09-20 07:04 +10: Seat Shield H6 measured blocked.
Cause: this install's systemd is `-SECCOMP`; unit
`SystemCallFilter` cannot attach.
Lock: already in kinship `BR2_PACKAGE_LIBSECCOMP=y` and in
the H1 official image (QEMU `+SECCOMP` / `seccomp mode=2`).
Measure: live `/proc` `Seccomp: 0` after the H1 mux. No
systemd rebuild. No reinstall. S5 not started. Not MATCH.

2026-09-20 07:24 +10: Seat Shield S5A-OS paste + lab mux.
Cause: OS crate was still 0.5.0 after bay 0.6.0 closed;
HUD/status had no scan/hold ops.
Lock: paste quarantine.rs + ops; keep OS `maybe_auto_observe`
and `integrity_twin_state=disabled`; live unit `dd6bb219…`
reused. No ENABLE_ACTS. No S5B. No reinstall.
Measure: crate 56 ok; Runtime `seat_shield` 8 ok;
preflight RESULT PASS. Mux `20260919T212434Z-runtime-3042`
Runtime `f229c221…` PID `176250` `NRestarts=0`;
`20260919T212449Z-shell-32029` GUI `3b8992b1…`
`index-DSbLUOHj.js`. Status `ok` `backend=helper`
`act_enabled=false` `auto_quarantine_hash=false`
`quarantine_count=0` `integrity_twin_state=disabled`.
`scan_path /etc/passwd` `act_denied`. `/dev/shm` scan
observed, no feed hit. Host `/tmp` denied by live
`PrivateTmp=true` (not a crate bug). fw still `1a3856a2…`.
Auto-act off. Not MATCH.

2026-09-20 07:28 +10: Seat Shield S5A official bake.
Cause: `out/` still H1 `c9805fc6…` / Runtime `04358653…`.
Lock: `package-shell-gui-rootfs.sh --build-image` with
`CARGO_TARGET_DIR` in EmpiierOS `runtime/target` (not bay).
Measure: `BAKE_EXIT=0` `image_contract_ok`. Overlay
`451aea7a…` Runtime `f229c221…` GUI `3b8992b1…`
fw `fff55bfe…` Shell `index-DSbLUOHj.js`.
`rootfs.ext2` `dc8ebae8…` stamp `5e7ebded…`.
bzImage `db616a36…` unchanged. QEMU not rerun; prior
WebKit string FAIL stands. Lab not reinstalled. S5B
not started. Twin stays disabled. Not MATCH.

2026-09-20 08:06 +10: Seat Shield S5B mux blocked + official bake.
Cause: glass still S5A; watch helper was source-only.
Lock: packager + `deploy-seat-shield-watch` + kinship
post-build (`PrivateTmp=false`, wants symlink). Live
Runtime unit not rewritten.
Measure: mux **blocked** — `192.168.8.10` no ICMP/SSH via
`192.168.1.1`. Bake `image_contract_ok` overlay `d1fbef23…`
Runtime `26e1828c…` fw `8f4955bc…` watch `9f9e4d83…`
Shell `index-BBUZrLR_.js` `rootfs.ext2` `00e14f74…` stamp
`94ecff5a…`. debugfs watch 0755 + wants symlink. QEMU not
rerun. Auto-act off. Twin disabled. Not MATCH.

2026-09-20 16:14 +10: Seat Shield S5B mux onto 192.168.8.10.
Cause: glass reachable again after 08:00 block.
Lock: `deploy-runtime` + `deploy-shell` +
`deploy-seat-shield-watch`. Watch unit comment no longer
contains `nft` (deploy substring). Live Runtime unit
`dd6bb219…` reused. No ENABLE_ACTS. No reinstall.
Measure: `20260920T061357Z-runtime-21507` Runtime
`0a1f1d9f…` PID then superseded; `20260920T061414Z-shell-3770`
GUI `3b8992b1…` `index-BBUZrLR_.js`;
`20260920T061432Z-seat-shield-watch-4965` watch `2143eea5…`
`PrivateTmp=no`. Status `backend=helper` `act_enabled=false`
`last_watch_drained` moved after `/dev/shm` write. Host
`/tmp` still Runtime `PrivateTmp`. Not MATCH.

2026-09-20 16:18 +10: Seat Shield S5C helper-hash + remux.
Cause: Runtime `PrivateTmp=true` cannot hash host `/tmp`
even after S5B drain.
Lock: journal `sha256` after `resolve_allowed_file`; drain
uses it only for scan-root paths Runtime cannot open;
planted `/etc` digest ignored. Helper still never holds.
Measure: crate 69 + 2 bin; Runtime `seat_shield` 9.
Mux `20260920T061821Z-runtime-28766` Runtime `d1360890…`
PID `641642` `NRestarts=0`;
`20260920T061831Z-seat-shield-watch-3066` watch `8525c72b…`.
`/tmp/seat_s5c_lab_probe` journal hash `82f08bc5…`;
`scan_path /tmp` denied; status drained `watch_pending=0`.
Official `out/` still S5B. Auto-act off. Twin disabled.
Not MATCH.

2026-09-20 16:26 +10: Seat Shield S5C official bake.
Cause: `out/` was still S5B `00e14f74…` while glass ran 0.8.0.
Lock: `package-shell-gui-rootfs.sh --build-image` with
`CARGO_TARGET_DIR` in EmpiierOS `runtime/target`. Watch unit
nft-free. Live not remuxed.
Measure: `BAKE_EXIT=0` `image_contract_ok`. Overlay
`e0a3fb8a…` Runtime `cb2a1647…` GUI `3b8992b1…`
fw `bae5e310…` watch `c40d6d74…` Shell `index-BBUZrLR_.js`.
`rootfs.ext2` `82ac20ac…` stamp `a8ead998…`.
bzImage `db616a36…` unchanged. debugfs watch 0755 + wants
symlink. QEMU not rerun; prior WebKit string FAIL stands.
Lab still unstripped `d1360890…` / `8525c72b…`. Auto-act
off. Twin disabled. Not MATCH.

2026-09-20 16:35 +10: Seat Browser media.set on glass.
Cause: overlay allowed UserMedia unconditionally; op was
not_available.
Lock: persist camera/microphone (default on); enable needs
ENABLE_BROWSER_MEDIA; overlay asks status; display capture
denied.
Measure: crate 22; Runtime seat_browser 6; seatBrowser.check
ok. Mux `20260920T063509Z-runtime-13314` Runtime `db9439bf…`
PID `656447` `NRestarts=0`; `20260920T063519Z-shell-24534`
GUI `ac9ba033…` `index-BNv3b80O.js`. Status VIP allowlist
camera=true. Empty set invalid_params. camera=false applied;
re-enable without confirm missing_confirmation; restore with
token. Official `out/` still S5C (no media.set). Not MATCH.

2026-09-20 16:47 +10: Seat Browser ≤3 tabs on glass.
Cause: status `tab_count` was hardcoded 0; overlay was one WebView.
Lock: persist `{state}/seat_browser/tabs.json` cap 3;
`tab.open`/`select`/`close`; GtkStack of related WebViews; overlay
`go` also `navigate`. Downloads stay cancelled.
Measure: crate 25; Runtime seat_browser 7; seatBrowser.check ok.
Mux `20260920T064733Z-runtime-7514` Runtime `a913c84b…` PID `667575`
`NRestarts=0`; `20260920T064743Z-shell-18970` GUI `b07b4cd6…`
`index-Cj40NGTM.js`. Unit `dd6bb219…` reused. Probe: VIP allowlist;
blank open; example.com `allowlist_miss`; fourth `tab_limit`; hide
keeps tabs; last close `host_action=hide`. Restored VIP tab id 4.
Official `out/` still S5C (no media.set, no tabs). Not MATCH.

2026-09-20 16:56 +10: Seat Browser downloads on glass.
Cause: overlay cancelled every WebKit download.
Lock: `download.begin`/`finish`; dest only
`{state}/seat_browser/downloads/<basename>`; overlay rejects dest
without that prefix; 32 MiB / one in flight; status never a path.
Not Content Manager.
Measure: crate 28; Runtime seat_browser 8; seatBrowser.check ok.
Mux `20260920T065644Z-runtime-15309` Runtime `fc3442ef…` PID `676009`
`NRestarts=0`; `20260920T065654Z-shell-24073` GUI `b58032ef…`
`index-IgUPFtM_.js`. Unit `dd6bb219…` reused. Probe: VIP begin
`favicon.ico`; example.com `allowlist_miss`; second `download_busy`;
status basename only. Official `out/` still S5C. Not MATCH.

2026-09-20 17:01 +10: Seat Browser `system.snapshot` family row.
Cause: snapshot advertised launcher/`product.open` but not
`seat.browser` while the overlay ops already dispatched.
Lock: `capability_statuses()` row `seat.browser` /
`seat-browser-overlay` / `empiieros_seat_browser_overlay`. Not MATCH.
Not `seat.shield`.
Measure: unit `seat_browser_capability_is_advertised` + core
`snapshot_and_events_share_a_sequence`. Mux
`20260920T070138Z-runtime-22267` Runtime `48f1275e…` PID `680774`
`NRestarts=0`. Unit `dd6bb219…` reused. GUI unchanged. Probe
`system.snapshot` lists the row `available`. Official `out/` still
S5C. Not MATCH.

2026-09-20 17:12 +10: Seat Shield hardening H3–H5 debts closed in
source. Not H6. Not bake. Not reinstall.
Cause: H3 was SKIP (`cargo-audit` missing); H4 had garbage unit
tests but no corpus/PRNG gate; H5 jail was in units without
`ProtectHome`/`ProtectSystem` needles.
Lock: install `cargo-audit` 0.22.2; script `--file` both locks
`--deny unsound --deny yanked`; `parser_probe` +
`tests/parser_fuzz.rs`; preflight H4/H5 needles. Stale browser
needles (permission in `secure_main`, “media not wired”) replaced
with overlay/`op_media_set`/`download.begin`/`tab.open`.
Measure: audit script 0; crate 69+1+2; Runtime `seat_shield` 9;
`scripts/preflight-foundation.sh` RESULT PASS. H6 still live
`-SECCOMP`. Official `out/` still S5C. Not MATCH.

2026-09-20 17:21 +10: EliteEW TZ + FoFline-on-OS brief.
Cause: catalog EBrowser still requires Tauri invoke; OS overlay
already exists. Pack must call a Meet-shaped host, not iframe /
`ebrowser_*`.
Lock: `docs/plans/ELITEEW-TZ-FOFLINE-SEAT-BROWSER.md` +
`docs/plans/FOFLINE-OS-BRIEF.md`. No EmpiierOS inject this turn.
Measure: files on disk. Not a published pack. Not MATCH.

2026-09-20 18:12 +10: EliteEW pack `1789890833725-7y746j4lvw` + OS
Seat Browser host inject.
Cause: catalog adapter is in spa-latest, but OS did not plant
`window.EmperyDesktopSeatBrowser`, so pack returned
`seat_browser_unavailable`.
Lock: OIDC inject tail plants Meet-shaped host; overlay
`empiierosSeatBrowser` handler; tokens/`http`/`file` fail-closed.
Measure: overlay `b8e18c68…` GUI `e18051fa…` mux
`20260920T080704Z-shell-18757`. Runtime unchanged `48f1275e…`.
`launcher.channel_up` then `launcher.spa_sync` applied. Live
`spa_app_build_id=1789890833725-7y746j4lvw` `spa_state=ready`
`assets_core_mismatch` 7/8. Pack `EBrowserApp-wtVepikT.js` has
`EmperyDesktopSeatBrowser` / `openSeatBrowser`. Syslog has no
`fofline seat browser hook planted` yet — hide/re-open required.
stack0 may still hide the catalog tile. Official `out/` still S5C.
Not MATCH. Not DEMO_PASSED.

2026-09-20 18:29 +10: EliteEW stack0/edge
`1789891909119-45tkayntro` (`prod-baked-20260920-1811`, git
`30944c93c`). spa-latest unchanged `1789890833725-7y746j4lvw` — no
second `spa_sync`. Their “OS inject missing /
`seat_browser_unavailable`” is stale on this lab: syslog
`fofline seat browser hook planted` 18:27:04; pack served
`EBrowserApp-wtVepikT.js`; host `kind=open` 18:28:25;
Runtime `seat-browser-host-nav` applied. Then overlay hidden and
`kind=close`. Not MATCH. Seat Shield watch post-build check they
left uncommitted is not this bake. Official `out/` still S5C.

2026-09-20 18:58 +10: official bake + live-installer USB.
Cause: official `out/` was still S5C (no media.set / tabs /
downloads / snapshot row / `EmperyDesktopSeatBrowser`).
Lock: `CARGO_TARGET_DIR=runtime/target`
`scripts/package-shell-gui-rootfs.sh --build-image`. Preflight
RESULT PASS. `BAKE_EXIT=0` `image_contract_ok`. Overlay
`b8e18c68…`. `out/rootfs.ext2` `4104a67b…` stamp
`rootfs_sha256=d250a976…` bzImage `db616a36…` (unchanged). Image
Runtime `fc7e6443…` GUI `e18051fa…` Shell `index-IgUPFtM_.js` fw
`7e6cd014…` watch `17560b39…` (0755 + wants symlink).
`pack-install-rootfs-metal.sh` aligned factory-work `out/rootfs.tar.gz`
to the same SoT (stale Sep-17 312M tar would have failed flash).
First flash via by-id failed after GPT wipe (`${DEV}1` is not
`-part1`). Retry `/dev/sda` after TRAN=usb RM=1 MODEL=FLASH DRIVE
and host root ≠ sda. `FLASH_EXIT=0` SoT `d250a976…` PARTUUID
`912fc022…` FSUUID `c35ced72…`. QEMU not rerun; prior WebKit FAIL
stands. Lab `192.168.8.10` not reinstalled. H6 still live `-SECCOMP`.
Not MATCH. Not metal.

2026-09-20 19:06 +10: live-installer USB returned; install did not
commit.
Cause: proof-before-`ERASE`. `/opt/eliteew-ts-install/` on media has
only `rootfs.tar.gz` + `.sha256` — no `install-proof.json`, no
`current.ecap`. Apply died `install proof missing or unsafe` **before**
`wipefs` (three `--commit` attempts on `/dev/nvme0n1` identity
`259:0|500118192|993A1ATHK3HL`). DHCP on `eno1` failed (no lease);
static `192.168.8.10/30` applied. USB `/dev/sda` correctly excluded
as live root. No `INSTALL-COMPLETE`.
Lock: keep the gate. Do not plant a fake ticket.
Measure: read-only mount of `sda3`; logs
`/var/log/eliteew-ts-install.log` +
`/home/eliteew-ts/eliteew-ts-install.log`. Lab disk not wiped. Not
MATCH.

2026-09-20 19:25 +10: V2 install ticket (no media `.ecap`).
Cause: apply treated living Capsule v3 as wipe proof; stamp did
not write `install-proof.json`. Correct USB (SoT only) died before
`wipefs`.
Lock: `verify_install_proof` accepts only
`{"v":1,"kind":"empiieros-install-proof-v1","product_id":"eliteew-fofline"}`.
`stamp-live-installer-media.sh` writes that ticket and deletes
leftover `current.ecap`. QEMU install smoke plants ticket only;
deny smoke still has no ticket. At-rest encryption is design-only
(`docs/plans/TZ-DISK-AT-REST.md`) — not this patch.
Measure: ticket needles OK (`apply proof is ticket-only`, stamp
writes/refuses ecap, off-path+missing denied). Full
`selftest-installer.sh` still FAIL on the missing kinship readiness
doc (pre-existing). Live USB `/dev/sda` serial `071C57D0F48A4A92`
patched in place: apply `1dd7389a…` + ticket `71fd7831…` (75 B),
`leftover_ecap=no`, SoT `rootfs.tar.gz` untouched. Official `out/`
still has the old helper until the next bake. Not MATCH.

2026-09-20 19:37 +10: metal install committed; PIN not shown; seat
unlocked to desktop extras.
Cause: apply planted `fp_v2_e346a382…` and wrote live
`/run/eliteew-ts-installer/first-boot-pin`, but the installer GUI
Done step never read it and rebooted in ~4s. `/run` died. Operator
cannot know the PIN. HUD + desktop still offered Seat Browser
without consume.
Lock: Done shows the four digits and refuses reboot until `ЗАПИСАЛ`.
Unlicensed seat (no capsule+entitlement) keeps only the launcher
activation path (Shell + HUD + `seat.browser` mutations).
Measure: USB install log `disk-install complete` + `step -> Done`.
This seat's PIN is not recoverable from the log. Next install needs
the rebuilt installer binary. Seat lock needs the new Shell/GUI on
the installed image (not this running rootfs). Not MATCH.

2026-09-20 19:49 +10: second metal install; PIN still not shown.
Cause: `CARGO_TARGET_DIR=runtime/target` stole the installer crate
build; `build-installer-from-source.sh` copied the stale 2026-09-17
binary (`dae941b2…`, no reveal strings). Done→reboot in 3s
(`1789897750`→`7753`). Apply did plant a new hash
(`fp_v2_e346a382…`); live reveal file is root `0600` so uid 105
could not have read it anyway.
Lock: build pins `CARGO_TARGET_DIR=${SRC}/target`. GUI reads PIN
via `pin-reveal` (sudo helper). Reveal file `0640` `root:eliteew-ts`.
Measure: USB now `eliteew-ts-installer` `f674b651…` contains
`first-boot PIN ready` + `ЗАПИСАЛ`; apply `5ee5c749…` has
`cmd_pin_reveal`. Not MATCH.

2026-09-20 19:55 +10: installed Seat Shield shows `file_scan_denied`
`io: scan_path_stat: No such file or directory`.
Cause: watch helper (host `/tmp`) journals create/close_write.
Runtime `PrivateTmp` plus unlinked tmp files make `symlink_metadata`
ENOENT. That Io error was stored as a low security event
("stay inside /tmp /var/tmp /dev/shm") even when the path was
already under a scan root and had no journal hash.
Lock: allowlist before stat. Vanished/unreadable allowlisted paths
are a silent skip. Drain persists `file_scan_denied` only for
outside-root planted lines. Helper journal hash still IOC-matches
when the file is gone.
Measure: crate tests `missing_tmp_file_is_skip_not_denied` and
`watch_journal_missing_tmp_without_hash_is_silent`. Live installed
Runtime will keep the old events until this crate is on that seat.
Not MATCH.

2026-09-20 20:06 +10: disk at-rest wave 1 in apply/GUI/defconfig.
Cause: install wrote plain ext4; seat secrets sat under
`/var/lib/empiieros-runtime` with UNIX modes only. PIN is an IPC
gate, not a volume key.
Lock: GPT ESP+root+2 GiB LUKS2 state. Recovery is 128-bit grouped
hex, shown once (`recovery-reveal`). PIN is not used for
`luksFormat`. TPM enroll is extra (`state tpm=yes|no`). Missing
`cryptsetup` is a die, not a skip. Runtime drop-in
`RequiresMountsFor=` is planted on the target only.
Measure: `selftest-state-luks.sh` + installer needles. Official
`out/` still lacks cryptsetup until bake. Not MATCH.

2026-09-20 21:13 +10: full-floor bake + live-installer USB.
Cause: official `out/` `d250a976…` had no `cryptsetup`; apply
would die or brick. Overlay-only `--build-image` cannot add the
package or `CONFIG_DM_CRYPT`.
Lock: fragment `BR2_PACKAGE_CRYPTSETUP=y` + allowlist + resume
`linux-reconfigure` / `systemd-reconfigure`. Overlay `670612ea…`
then `buildroot-build.sh --resume` `image_contract_ok`. Archive
has `/usr/sbin/cryptsetup`; kernel `CONFIG_DM_CRYPT=y`.
`pack-install-rootfs-metal.sh` aligned factory-work `out/`.
Measure: `FLASH OK` `/dev/sda` serial `071C57D0F48A4A92` SoT
`5979e9cf…` (`rootfs.ext2` `e48aab38…`, bzImage `69bc60b6…`,
Runtime `79e06a7d…`, GUI `f2fe1900…`). Stamp WARN about missing
`rootfs.tar.gz` is the script order (stamp then place). QEMU
install and metal reinstall not done. Not MATCH.

2026-09-20 21:30 +10: first-boot PIN `fingerprint_mismatch` after LUKS.
Cause: SoT `/etc/machine-id` is empty. Apply hashed the PIN with
`machine_id=no-machine-id`. First boot `systemd-machine-id-setup`
wrote a real 32-hex id. Live `fp_v2` ≠ planted stamp. Recovery
unlock of the LUKS state slot is unrelated (PIN is not the volume
key).
Lock: apply `commit_target_machine_id` before plant. Runtime unlock
rebinds only when the planted fp equals
`(hostname, no-machine-id, same DMI)` and live machine-id is 32
hex. Other mismatches stay `fingerprint_mismatch`.
Measure: apply sourced test commits empty id and keeps it.
`empty_machine_id_plant_rebinds_on_unlock`. This seat still has
the old apply/Runtime until reinstall or a Runtime mux. Not MATCH.

2026-09-20 21:40 +10: installer state key is auto or operator.
Cause: recovery was only RNG hex; operator asked to choose auto
issue or type their own secret.
Lock: `state-recovery-stage` stdin (`auto` / `user`). User secret
is 12–128 chars, not a PIN. `resolve_state_recovery` keeps auto
when nothing is staged (QEMU). GUI `StateKey` before `ERASE`.
Measure: apply sourced resolve + user reject `1234`; installer
`user_state_secret_rejects_pin_and_accepts_passphrase`. Not on
the current USB until the installer binary and apply are copied.
Not MATCH.

2026-09-20 21:45 +10: state-key choice compiled; apply log off stdout.
Cause: `log` teed to stdout, so
`recovery="$(resolve_state_recovery)"` would have become
`unix=… [APPLY] …` plus the secret — LUKS would take garbage.
Lock: `log` is stderr + file. GUI still reads stderr. User secret
stays stdin (`state-recovery-stage`), never argv. Overlay installer
rebuilt zig/GLIBC_2.41 (`1d8755eb…`); apply `b51b5ad1…`.
Measure: sourced auto is 39-hex only (`!= *APPLY*`); user
`operator-disk-key-ok`; reject `1234`. `cargo test --offline
user_state_secret` 1 ok. Payload `--check` OK.

2026-09-20 21:37 +10: live USB patched in place.
Cause: operator asked to put the state-key installer on the stick.
Lock: `/dev/sda` only when `TRAN=usb` `RM=1` serial
`071C57D0F48A4A92`; live stamp required; no wipe; no NVMe.
Measure: `USB_PATCH_OK` on `/dev/sda3` — installer `1d8755eb…`,
apply `b51b5ad1…`, needles `state-recovery-stage` +
«Задать свой ключ». SoT archive on the stick is unchanged
(`5979e9cf…` bake). Not MATCH.

2026-09-20 21:58 +10: install UX — Wi-Fi PSK, ЗАПИСАЛ, network floor.
Cause 1: live `LANG=ru_RU.UTF-8`; bash `[\ -~]` follows collation
and rejects an 8-char ASCII PSK (`12345678` → fail).
Cause 2: installer session had no XKB `us,ru`; Done asked for
`ЗАПИСАЛ` on a US-only keyboard.
Cause 3: `visibleHostModules(!licensed)` hid network with the
product tiles, so a locked launcher could not open a link.
Lock: apply `valid_wifi_psk` / `valid_wifi_country` under
`LC_ALL=C`. Session + labwc `environment` get `us,ru` and
`grp:alt_shift_toggle`. Done accepts `ЗАПИСАЛ` or `ZAPISAL`.
Shell seat floor is launcher + network.
Measure: sourced `valid_wifi_psk 12345678` under `ru_RU.UTF-8`.
`cargo test --offline` 6 ok. `foflineLauncherStage.check` ok.
USB `071C57D0F48A4A92`: installer `fd674e0c…`, apply
`64343cc5…`, session `6e0e44cc…`, SoT
`f56dba1b…` (shell assets only; cryptsetup/firmware kept).
Factory `out/rootfs.tar.gz` still `5979e9cf…`. Not MATCH.

2026-09-20 22:05 +10: installer Wi-Fi association used a rejected recipe.
Cause: apply wrote `ctrl_interface=` + `key_mgmt=WPA-PSK SAE`.
Runtime on this image already documents that wpa_supplicant
rejects both; the unit starts, the network block is skipped,
`iw link` never shows the SSID (20s), GUI says association
not observed. Wrong PSK has the same symptom.
Lock: apply matches Runtime — `key_mgmt=WPA-PSK`, no
ctrl_interface, `scan_ssid=1`, rfkill unblock.
Measure: apply needles; payload `--check` OK. USB was not
plugged (`/dev/sda` gone) so live apply was not copied.
Not MATCH.

2026-09-20 22:08 +10: live USB apply Wi-Fi recipe copied.
Measure: `USB_APPLY_WIFI_OK` `/dev/sda` serial `071C57D0F48A4A92`
apply `07a459d3…` (`WPA-PSK`, no SAE). Not MATCH.

2026-09-20 22:11 +10: Wi-Fi unit start-limit after SAE crash-loop.
Cause: `empiieros-wifi@.service` `StartLimitBurst=3` +
`Restart=on-failure`. SAE recipe exited; three auto-restarts
marked start-limit-hit. Next click: `systemctl restart` failed
(`wpa_supplicant service failed for wlp58s0`) even with the
WPA-PSK profile on disk.
Lock: apply `reset-failed` + `ip link set up` before restart;
failure includes unit Result/journal, not the PSK.
Measure: apply `5f85b56f…` on USB `071C57D0F48A4A92`. Not MATCH.

2026-09-20 22:20 +10: installer Wi-Fi SSID pick was silent-strongest.
Cause: after `wifi-scan`, GUI set `selected_wifi` to
`wifi_networks.first()` (strongest RSSI). Click path and `--ssid`
were real, but a typed PSK could target the wrong AP. Fail-closed
`iw link` then printed `SSID was not observed (Not connected)` —
no BSS at all, not a connect to a different name.
Lock: scan keeps a click only; same-SSID reclick keeps the PSK;
label `Выбрана сеть:`; apply logs `associating iface= ssid=`
without the PSK. Selftest needle refuses `.first()` after scan.
Measure: `cargo test --offline` 6 ok. USB `071C57D0F48A4A92`
`USB_WIFI_SELECT_OK` installer `dd6dc027…` apply `59b41a3a…`.
Factory `out/` still `5979e9cf…`. Not MATCH.

2026-09-20 22:32 +10: installer Wi-Fi never reached the AP.
Cause: `empiieros-wifi@.service` `RestrictAddressFamilies`
had UNIX/NETLINK/PACKET only. This image's wpa nl80211 opens
`PF_INET SOCK_DGRAM`; systemd returns EAFNOSUPPORT, wpa exits
255, no probe. `iw scan` is outside the unit so the list works.
Journal on the stick: `Failed to initialize driver 'nl80211'`
and `sudo … network-wifi --ssid EWKaska` — Далее was not a
stub. Default iface was `eno1` until the wifi row was clicked.
Lock: unit allows `AF_INET`/`AF_INET6`. Apply refuses an empty
profile, waits for a live MainPID, names the SSID on failure.
GUI picks the wifi iface when the Wi-Fi chip is selected.
Measure: `cargo test --offline` 6 ok. USB `071C57D0F48A4A92`
`USB_WIFI_AFINET_OK` installer `7ab51ce7…` apply `e413fd40…`
unit `4f5bc266…`. Factory `out/` still `5979e9cf…`. Not MATCH.

2026-09-20 22:45 +10: leftover Wi-Fi + Google NTP on live USB.
Cause 1: `empiieros-wifi-restore` starts any
`/etc/wpa_supplicant/wpa_supplicant-*.conf` left on the stick
(previous glass/install). Operator saw an automatic join, then
`associated but no IPv4 lease` on reconnect.
Cause 2: systemd compiled `FallbackNTP=time1.google.com…`;
timesyncd was wanted by sysinit; nft punched udp/123.
Lock: live session `live-network-forget`; restore skipped when
`live-installer` stamp exists; stamp wipe leftover profiles.
timesyncd masked + empty NTP drop-in; nft no udp/123; installer
Regional takes a manual `YYYY-MM-DD HH:MM`.
Measure: `cargo test --offline` 7 ok. air-egress selftest OK.
USB `071C57D0F48A4A92` `USB_FORGET_NTP_OK` installer `e3b1b734…`
apply `fe1594ca…`. Leftover wpa/install `.network` unlinked.
Factory `out/` still `5979e9cf…`. Not MATCH.

2026-09-20 23:02 +10: lab /30 baked into Settings and install persist.
Cause 1: `NetworkPanel` / `App.tsx` `useState("192.168.8.10")` and
gateway `192.168.8.9` — Settings opened with the polygon /30 even
on a clean DHCP/Wi-Fi seat.
Cause 2: apply persisted leftover static
`20-eliteew-ts-install-*.network` as `10-` on the target. Glass
`192.168.8.10/30` via `192.168.8.9` won over `20-wired.network`
and over Wi-Fi restore.
Lock: empty Settings fields (observe snapshot only). apply refuses
those two addresses; `10-` persist only after this-session
operator-static. Preflight forbids the literals. Adversarial
inject of `192.168.8.10` fails the assert.
Measure: Shell `index-Gezp4t9R.js` has no `192.168.8.10`.
USB apply `ad6f5e2e…`. A disk already installed may still have
`10-eliteew-ts-install-*.network` until deleted or reinstalled.
Factory `out/` still `5979e9cf…`. Not MATCH.

2026-09-20 23:20 +10: post-install Wi-Fi had no DHCP because wpa never
probed. Cause: official SoT `5979e9cf…` `empiieros-wifi@.service`
`RestrictAddressFamilies=` is UNIX/NETLINK/PACKET only. Live USB was
patched with `AF_INET`, so installer associate+lease works. Extract
plants the silent SoT unit; nl80211 dies `EAFNOSUPPORT`; Settings then
shows `wifi_verification_failed` (assoc and IPv4 conflated) and
rollback can add `recovery_failed`. Not a router DHCP bug.
Lock: apply plants the live AF_INET unit after extract and refuses a
target unit without it. Runtime splits assoc vs IPv4, `reset-failed`,
`scan_ssid=1`, kick DHCP after carrier, no PSK rollback on a late
lease. Restore reconfigures after SSID. nft input accepts
`udp sport 67 dport 68`.
Measure: plant unit selftest `PLANT_OK` (AF_INET6-only refused).
air-egress OK. `cargo test` wifi_control 7 ok. preflight PASS.
Factory `out/` still `5979e9cf…`. Already-installed disks still have
the silent unit until reinstall. Not MATCH.

2026-09-20 23:22 +10: live USB planted for the post-install Wi-Fi unit.
Cause: last apply copy needed sudo; stick still had apply `ad6f5e2e…`
and would extract the silent SoT unit. Lock: `/dev/sda` only when
`TRAN=usb` `RM=1` serial `071C57D0F48A4A92`; live stamp required;
SoT tar not rewritten. Measure: `USB_WIFI_PLANT_OK` on `/dev/sda3` —
apply `8a9ec933…`, unit `4f5bc266…`, restore `0b2a1181…`, nft
`ab30fcff…`. SoT on the stick stayed `f56dba1b…` (earlier shell mux).
Factory `out/` still `5979e9cf…`. Not MATCH.

2026-09-21: living SoT / rules sync after LUKS + Wi-Fi plant.
Cause: README/START-HERE still said QEMU smokes pass; FOUNDATION-STATUS
quoted superseded USB/`out/` hashes (`4104a67b…`, `d250a976…`,
`fa1fbc6b…`); HEALTH-IPC still described assoc-only connect and
timesyncd restart; kernel table said 12 skills.
Lock: living canon, ledger, glass, IPC, R1/R8, session-start now
record source vs official `5979e9cf…` vs USB mux `f56dba1b…` + plant
`8a9ec933…`. PIN ≠ LUKS. No lab `/30` Settings default. timesyncd
masked. QEMU H1 FAIL stands. Not MATCH.

2026-09-21: historical P0–P92 `PRODUCT-WAVE-*` / harvest maps
soft-deleted from `docs/plans/` to `trash/20260920T152710Z/`
(119 files). Cause: living `docs/plans/` still hosted invalidated
Done/Wired waves that outranked the ledger in a new session.
Lock: `docs/plans/README.md` is a living index only; companions
in `session/` now point at the ledger. Capture stays origin archive.
Measure: preflight after retarget. Not MATCH.

2026-09-21: historical P62–P92 session contracts soft-deleted to
`trash/20260920T153352Z/` (26 files). Cause: archive essays still
sat next to living glass/install contracts. Lock: `session/` keeps
face, launcher, HUD, browser, shield, PIN, install, open, pack,
air/portal, audio, network, CONTRACT. Time floor stays in CONTRACT.
Measure: preflight after retarget. Not MATCH.

2026-09-21: air activate on the fresh LUKS seat returned
`device_keys_missing` immediately. Cause: apply opened state, planted
PIN/`fp_v2`, never `{state}/device_keys/recipient.sk.hex`; production
`device.keys.*` is compiled out so Shell cannot keys-ensure. USB
`eliteew-ts-install.log` last commit `fp_v2_30bf9d19…` SoT
`f56dba1b…` AF_INET planted, no SK line. Lock: apply
`issue_target_device_keys` (0600, never logged) + contract verify;
Runtime `ensure_recipient_sk` if absent, `device_keys_invalid` if
garbage. Measure: `cargo test` air/capsule 10 ok; apply selftest
`APPLY_KEYS_SELFTEST_OK`; USB apply `68196433…` on
`071C57D0F48A4A92` (SoT tar not rewritten). Already installed
volume stays empty until remux Runtime or reinstall. Not MATCH.

2026-09-21: after air completed, glass stayed on Portal until the
launcher was closed and reopened. Cause: helper
`assert_uplink_ipv4_survived` required global IPv4 on baked `eno1`.
Wi-Fi seat (`wlp58s0`) has none; `wg-portal` was already up, host-map
skipped, IPC `portal_broke_eno1`. FoflineActivator only refreshed
status on mount / success, so the rail did not move. Lock: uplink is
any non-portal global IPv4 (`portal_broke_uplink`); host-map before
that assert; Activator polls `launcher.status` 4s and refreshes after
a portal error if `channel_up` is already true. Measure: portal unit
test wifi/eno1/portal-only. Current seat helper still the old eno1
check until remux. Not MATCH.

2026-09-21: glass «Войти в EliteEW» returned `oidc_http_400`. Cause:
VIP PAR reached Keycloak; DPoP `iat` was `min(Date, local+15)`.
timesyncd is masked, so an installer clock a minute slow ages `iat`
out of KC `[now-25, now+15]`. Lock: Date is IdP SoT (no local cap);
no Date → `oidc_dpop_date_required`; 400 maps allowlisted `error`
only. Measure: `date_minutes_ahead_stays_inside_idp_window` +
`oidc_status_error` unit. Current seat helper still the old clamp
until remux. Not MATCH.

2026-09-21: install clock audit. Cause: Regional asks for wall
clock and `date -s` + optional `hwclock` on the **live USB only**.
Apply wrote `/etc/empiieros/install-wall-clock` and never consumed
it; timesyncd is masked; Runtime `ProtectClock=true`. After reboot
the seat trusted CMOS. OIDC then treated local as “unsynced NTP.”
Lock: stamp required; apply plants
`empiieros-apply-install-clock` (apply when RTC year implausible,
never rewind a 2024–2100 RTC); hwclock fail is logged, not
`|| true` silence. DPoP still uses IdP Date. Measure:
selftest-installer clock checks; script `bash -n`. Current seat
already booted without the oneshot. Not MATCH.

2026-09-21: USB clock plant on serial `071C57D0F48A4A92`
(`/dev/sdb`, TRAN=usb RM=1). Apply
`2063aeff…` + helper `f5cb7fa8…` + unit `c1d1e18d…`. SoT tar still
`f56dba1b…` (untouched). Live-installer stamp present. This is a
USB plant, not official `out/`, not MATCH. Already-installed seat
still lacks the oneshot until the next apply from this stick.

2026-09-21: next metal apply died `live install-clock helper
missing` **after** wipe+extract+LUKS. Cause: first USB plant used
`mount -o ro,noload` then remount rw; journal orphans deleted the
new helper/unit inodes (`e2fsck` cleared unused 585/591/592).
Retry started a second wipe; log cut mid-GPT. Lock: apply
`require_install_clock_sources` before wipe; share copy is plant
source; refuse unreadable (EUCLEAN) files. Measure: one-root
replant apply `3ed05c43…` helper `f5cb7fa8…` on both sbin and
share; e2fsck clean; SoT still `f56dba1b…`. Target NVMe is
mid-wipe — reinstall from this stick. Not MATCH.

2026-09-21: reinstall completed (`disk-install complete`,
clock planted, stamp `2026-09-21 02:38`). Login still
`oidc_http_400`. Cause: SoT runtime `79e06a7d…` clamps DPoP
`iat` to local+15. Installer time is minute precision
(`date -s` at `:00`). Keycloak accepts `[now-25, now+15]`,
so a clock a minute behind fails even when the operator set
it. The IdP-Date source fix was not in `f56dba1b…`. Lock:
apply plants share runtime `cb4abfc3…` (BR gcc, same
`/lib64` interpreter) and refuses wipe if that binary lacks
`oidc dpop iat missing Date`. Measure: one-root plant apply
`aeeede78…`, e2fsck clean, SoT unchanged. This seat still
runs the baked clamp until the next apply. Not MATCH.

2026-09-21: operator clock and portal step move into the running
OS, not the installer. Cause: `time.sync` only started masked
timesyncd; a stale `launcher.status` could put the portal step
back after `channel_up`. Lock: `action=set` writes the wall
clock via the helper (`ntp_masked` for sync); a portal-up
result leaves the portal step even if that status is late.
Measure: `time_control` unit + `foflineLauncherStage.check`.
The installed seat does not have these binaries yet. Not MATCH.

2026-09-21: USB seat plant on `071C57D0F48A4A92` (`/dev/sda`,
TRAN=usb RM=1). Apply `9f45f0a0…` plants runtime
`553c1342…` (IdP Date DPoP + `operator_wall_clock`), shell
`4e9e4473…` (set clock + portal step), time helper
`8b991b0c…` (`ntp_masked`, no `set-ntp`). SoT tar still
`f56dba1b…`. e2fsck clean. Not official `out/`. Not MATCH.

2026-09-21: planted seat shell was not trusted. Cause: `cp -a`
kept USB uid 1000, and `empiieros-shell-gui` rejects any
`empiier://shell/` file whose uid is not 0. QEMU install
recorded `PLANTED_SHELL uid=1000` and the GUI logged
`Shell asset is not trusted`. Lock: `plant_seat_shell` now
`chown -R 0:0`, mode 0644/0755, and
`verify_target_install_contract` dies with
`seat shell is not root-owned` if that slips. Measure: QEMU
install `PLANTED_SHELL uid=0 mode=644` for
`index-DSJBkgY9.js`, then the same GUI served that file and
logged `shell render ready`. USB apply on
`071C57D0F48A4A92` is `3f1749cc…`, owner 0:0 mode 755.
SoT tar still `f56dba1b…`. e2fsck clean. Not official
`out/`. Not MATCH. The running seat still has the old
shell until the next install from this stick.

2026-09-21: white Keycloak card after workspace download.
Cause: PAR POSTs to VIP by IP, so it can succeed while the
auth WebView still resolves `auth.eliteew.ru` without
`/etc/hosts` → white card and LOGIN silence in Keycloak.
Buildroot also locks `eliteew-ts` in shadow, so tty2 has no
password for `journalctl`. Lock: `launcher.oidc_start`
requires `portal_hostmap=applied` (`portal_hostmap_required`);
Shell shows host-map status and repairs it before sign-in;
apply plants tty2 break-glass `eliteew-ts` /
`eliteew-ts-change-me` and enables `getty@tty2`. Measure:
USB `071C57D0F48A4A92` apply `5822b206…`, runtime
`3dc42799…` (`portal_hostmap_required`), shell index
`3dbfa15b…` (RU host-map copy), SoT still `f56dba1b…`,
e2fsck clean. Not official `out/`. Not MATCH until the next
install from this stick.

2026-09-21: after a successful metal install the operator still had
to click host-map repair, and clock skew hit air plus Keycloak.
Cause: `channel_up` could succeed while `/etc/hosts` lacked the VIP
map; junk RTC years still reached air WSS and OIDC. Lock:
`channel_up` repairs host-map once and refuses unless applied; air
and OIDC fail closed with `wall_clock_required`; Shell shows clock
status and disables air/login until sane; apply plants
`empiieros-portal-up` with the host-map lock. Measure: unit
`oidc_dpop_iat` + stage check green; release runtime `780df186…` /
portal-up `fcf745fb…` / shell index `fa89b04a…`. USB was not in
the command center at close — plant when the stick returns. Not
MATCH.

2026-09-21: operator paused metal reinstalls. Living canons for the
proven nuances (operator clock, VIP vs WebView DNS, PAR/tokens-out-of-JS,
root-owned Shell, tty2 break-glass, USB plant ≠ SoT) are locked in
`session/SEAT-FOFLINE-CANONS.md` and mirrored into session contracts +
MATURITY-LEDGER. Source gates for host-map-on-`channel_up` and
`wall_clock_required` remain on disk; USB plant waits for the next
operator ask. Not MATCH.

2026-09-21: Seat Shield H6 + PrivateTmp design lock (no bake, no
reinstall). Cause: live systemd still `-SECCOMP` so unit
`SystemCallFilter` is theater; Runtime `PrivateTmp=true` correctly
hides host `/tmp` from `scan_path` while watch must stay
`PrivateTmp=false` + S5C journal hash. Lock:
`docs/plans/TZ-SEAT-SHIELD-H6-PRIVATETMP.md`, contract §§9.1–9.2,
preflight needles on kinship Runtime `PrivateTmp=true` +
`SystemCallFilter=~@debug` and watch namespace comment. Measure:
`scripts/preflight-foundation.sh` RESULT PASS; adversarial FAIL on
Runtime `PrivateTmp=false`, missing `SystemCallFilter`, and watch
`PrivateTmp=true`, then restore PASS. Full H6 metal close waits for
a fresh install. Not MATCH.

2026-09-21: Seat Shield spine P1–P3 local Measure (journal ·
connections · heuristics · inet_diag probe). Cause: Shield glass was
tabbed but lacked full-bleed + live `connections[]` + durable
`journal.jsonl` browse + observe-only chain heuristics; observe used
`/proc` only. Lock: `docs/plans/TZ-SEAT-SHIELD-SPINE.md`;
`journal.jsonl` dual-write; `heuristics_enabled` kill-switch;
`peer_window` / `risk_milli` / `chain_*` observe-only;
`runtime/seat_shield_probe/` SOCK_DIAG dump (no eBPF / AF_PACKET);
TTL fail→`/proc`; Shell RTT + journal `group_by=chain`. Measure:
`probe_helper_*` + `parses_snap_with_rtt` ok; Runtime `seat_shield`
tests green; `scripts/preflight-foundation.sh` RESULT PASS; packager
wires `/usr/libexec/empiieros-seat-shield-probe`. Metal MATCH still
requires Substrate bake + plant. Not MATCH.

2026-09-21: Host glass unify P0–P3 local Measure. Cause: host apps,
Fofline launcher, and desktop chrome drifted from Seat Shield glass
(full-bleed · tabs · chips · facts). Lock: `HostAppShell` +
`host-app-*`/`seat-shield-*` CSS alias; System≤5 tabs; launcher
Путь/Статус/Лаборатория with `LAUNCHER_JOURNEY` unchanged; Music/
Events thin Overview shells; Start/tray/icon glass density in
`fofline.css` + taskbar. ComfyUI probed (`:31003` 200) but regen
skipped — `product-chrome/host-*.png` and `launcher.png` present,
no audited gaps. Measure: `npm run build --prefix shell-ui`;
`tsx shell-ui/src/desktop/foflineLauncherStage.check.ts`;
`scripts/preflight-foundation.sh` exit 0. Not MATCH / not bake /
not USB plant.

2026-09-21: Capsule-gated OTA P0 design lock. Cause: production
blocks `update.*`; SPA bootstrap is pre-OIDC VIP tar; no
license+hardware+OIDC+portal gate before update transfer; CONTRACT
oversold lab `update.apply`. Lock: `docs/plans/TZ-CAPSULE-OTA.md`,
`session/OTA-CAPSULE.md`; ops `launcher.ota_*` only; bootstrap
`launcher.spa_sync` ≠ OTA; A/B P93 frozen; apply fail-closed until
plant. Honesty: CONTRACT / SEAT-FACE / SPA-PACK / HEALTH-IPC /
MATURITY-LEDGER / PROJECT-TREE / plans index. Measure: files on disk;
P1 code not claimed. Not MATCH.

2026-09-21: Capsule-gated OTA P1 local Measure. Cause: production needed
`launcher.ota_*` (not disabled `update.*`) with hard gates before any
update-capsule transfer. Lock: `runtime/src/launcher_ota.rs`; core
allowlist; VIP `empiieros-ota-fetch` helper (attest stub →
`attest_path_unplanted`); `ota_apply` → `apply_not_planted`; probe rows;
packager + preflight needles. Measure: `launcher_ota::` 4 tests ok;
`scripts/preflight-foundation.sh` RESULT PASS. Edge mint / cutover plant
/ Shell glass (P2) still open. Not MATCH.

2026-09-21: Capsule-gated OTA P2 local Measure. Cause: no licensed glass
for `launcher.ota_*`; Lab spa CTA blocked when ready hid intentional
bootstrap re-fetch. Lock: `SeatOtaSection` under System → Protection
(no 6th tab); `seatOta.ts` confirm `APPLY_OTA`; EN/RU; Lab note + Lab
spa re-sync allowed; Path spa still ready-gated. Measure: shell-ui
build; `foflineLauncherStage.check`; preflight PASS. Apply still
`apply_not_planted`. Not MATCH.

2026-09-21: Capsule-gated OTA P3 local Measure. Cause: no verified
staging store; factory_reset wiped wrong `recipient_sk.hex` and ignored
`ota-capsule`. Lock: `runtime/src/ota_capsule_store.rs` (artifact-role
Ed25519 fixture → `{state}/ota-capsule/current`); helper `stage` action;
factory_reset clears `recipient.sk.hex` + `ota-capsule`; apply still
`apply_not_planted`. Measure: `ota_capsule_store::` tests ok; preflight
PASS. Cutover plant (P4) open. Not MATCH.

2026-09-21: Capsule-gated OTA P4 design-lock. Cause: cutover destinations
and restart units were underspecified before any plant. Lock: TZ §Cutover
maps capsule paths → `/usr/local/bin|sbin` + `/opt/empiieros/shell` +
libexec seat-shield helpers; restart allowlist Runtime/Shell/watch;
OS-before-payload. Measure: docs on disk; `launcher.ota_apply` still
`apply_not_planted`; preflight needles on cutover table. No code plant.
Not MATCH / not bake / not USB.

2026-09-21: EmpiierOS OTA edge E0 skeleton. Cause: need separate contour from
Mobile/FoFline host-bind OTA. Lock: `TZ-EMPIERO-OS-OTA-EDGE.md`; OTA-CAPSULE
§Edge; SEAT-SHIELD §9.3 OTA baseline design; kinship
`stack-empiieros-ota` read_only + tmpfs `/srv/empiieros-ota` (no host
capsule bind) on `:29181`; start + autostart health. Measure: health ok;
empty `/v1/current.json` → 404; seat still `attest_path_unplanted` /
`apply_not_planted`. Not MATCH.

2026-09-21: EmpiierOS OTA edge E0b plant. Cause: living app vhost proxied
`/` to SPA; dedicated path required. Lock: `nginx-app-https-vm-mkcert.conf`
`^~ /empiieros-ota/` → `eliteew_empiieros_ota_upstream` (`10.77.77.2:29181`);
cloud-spa locations twin; prod-core `eliteew-wg-tcp-proxy@29181`; harden
port list + `open-empiieros-ota-fw.sh`. Measure: edge vhost has location;
edge→29180 ok; edge→29181 blocked by FedoraServer until rich-rule
tcp/29181 from `10.77.77.1` (sudo on prod-core). Public
`/empiieros-ota/health` stays 502 until that rule. Not MATCH.

2026-09-21: EmpiierOS OTA edge E0b Measure. Operator opened FedoraServer
rich-rule tcp/29181 from `10.77.77.1`. Evidence:
`https://app.eliteew.ru/empiieros-ota/health` → `ok` (200 text/plain);
`/v1/current.json` → 404 (empty tmpfs); edge→`10.77.77.2:29181` ok.
Not MATCH / not ingest / not cutover.

2026-09-21: EmpiierOS OTA edge E0c Measure. Cause: prod-core lacked
`python3-cryptography` (no passwordless sudo). Lock:
`scripts/empiieros-ota-cp-ingest.py` verify→`podman cp` into RO+tmpfs only;
Ed25519 via cryptography **or** `openssl pkeyutl`. Evidence: selftest OK;
signed ingest → public `/empiieros-ota/v1/current.json` 200; unsigned and
wrong-PK refuse exit 2; `podman restart` → current 404. SK not in git.
Not MATCH / not E1 pack / not cutover.

2026-09-21: EmpiierOS OTA edge E1a+E1b Measure. Cause: inline 256KiB store
could not carry OS binaries; host `/srv` publish forbidden. Lock: seat
`verify_and_stage_with_blobs` + href
`/empiieros-ota/v1/blobs/<sha256>`; CP `pack`/`publish` blob-ref → E0c
ingest (local or `--ssh`). Evidence: 8/8 `ota_capsule_store` tests;
selftest href refuse; public `current` 200 + blob 200
(`61be06aa…`); unsigned refuse. Apply still `apply_not_planted`. Not MATCH.

2026-09-21: EmpiierOS OTA edge E2a–E2c Measure. Cause: seat VIP blocked
`/empiieros-ota/`; helper stub forever `attest_path_unplanted`; edge had
no POST mint. Lock: `https_vip` GET prefix + POST attest; helper
attest→current→blobs→stage (`TimeoutStartSec=900`); edge nginx
`/v1/attest` portal-path grant when `current.json` exists (image `:e2`).
Evidence: allowlist + grant-parse tests; public empty attest 404
`no_capsule`; after publish grant 200. Without helper →
`ota_helper_required`. Apply still `apply_not_planted`. Live stage needs
E3 PK. Not MATCH.

2026-09-21: EmpiierOS OTA edge E3+E3b Measure. Cause: seat lacked planted
artifact PK and honest empty-edge glass codes. Lock:
`plant_artifact_pk` → `{state}/ota/artifact.pk.hex`; status
`artifact_pk`/`ota_helper`; gate `artifact_pk_missing`;
`scripts/empiieros-ota-plant-seat-pk.py` + `empiieros-ota-e3-stage-measure.sh`;
SeatOtaSection locale for `no_capsule` / PK / helper. Evidence: plant
unit tests; live public edge fetch→stage exit 0; apply still
`apply_not_planted`. Portal-VIP helper metal not claimed. Not MATCH.

2026-09-21: EmpiierOS OTA edge E3c Measure. Cause: Shield had no OTA observe
hooks; VIP could auto-BlockPeer when private-on-protected armed. Lock:
`ota_observe` (VIP exempt in cycle + eligibility); status `ota_baseline`
(absent → `missing`); fail-soft `ota_capsule_staged` on stage. Evidence:
`ota_observe::` + `ota_portal_vip_never_proposes_block` tests; status field
assert. Not MATCH.

2026-09-21: EmpiierOS OTA edge E4 Measure. Cause: cutover stayed
`apply_not_planted` after stage. Lock: `runtime/src/ota_cutover.rs`
(P4 dest table; `dest.new`→rename; `apply.json` + `ota_baseline.json`;
`os_before_payload` for fofline; lab `EMPIIEROS_OTA_CUTOVER_ROOT`).
Evidence: `ota_cutover::` 3 ok; preflight PASS. Not metal cutover /
MATCH.

2026-09-21 14:58 +10: EliteEW Line A
`prod-baked-20260921-1447` `deploy_id` `1789966077824-cyp3ky1gbw`
git `55c7dc7af` (issuer `payload.ota_artifact_pk`). Also local
`3307247d7` (OTA edge `:29181`) + `c0773ca43` (installer wall
clock). No `git push`; tag not on this clone. Staging
`.env.stack0.backend` has verify PK `813cfc81…` (no SK).
Independent: `https://app.eliteew.ru/api/health` 200 `{"ok":true}`;
`/empiieros-ota/health` 200 `ok`. FoFline Air mint not walked.
EmpiierOS glass not walked. Not MATCH.

2026-09-21 15:10 +10: live-installer USB unplant. Cause: live
`eliteew-ts-install-apply` still had `require_seat_ota_pk` /
`plant_seat_ota_pk` (ESP + share carried old `bb55d7df…`). Overlay
apply and SoT tar already clean. Lock: stamp
`unplant_live_ota_keys` + `plant_live_apply`; flash strips ESP
crutches; `scripts/empiieros-installer-media-unplant-ota.sh`.
Measure: live apply sha256 `20b3cb41…` = overlay; no
`artifact.pk.hex` on ESP/share; SoT `5843eea8…`; preflight PASS.
`selftest-installer` still FAIL on unrelated prior checks (PIN
reveal / timesyncd symlink / readiness doc). Not official `out/`
`5979e9cf…`. Not MATCH. Not Air mint.

2026-09-21 15:38 +10: living API still omits `ota_artifact_pk`.
Cause: SOPS `prod-core.env.tar.sops` 2026-09-18; staging
`.env.stack0.backend` 2026-09-21 14:54 has verify PK (64 hex).
`.111` tmpfs + `stack0-eliteew-api-prod-baked` UNSET. Connected
Air omits; FoFline 503 not hit. Next: rebuild+encrypt SOPS →
`--validate-only` (key present) → `--yes` → recreate API → new
FoFline session. Not MATCH.

2026-09-21 15:49 +10: EliteEW SOPS→tmpfs→API recreate. SOPS
`prod-core.env.tar.sops` on this clone 15:43 (was 2026-09-18).
Operator: `--yes` `runtime_tmpfs`; `eliteew-api-prod` force-recreate
15:46; process PK SET `813cfc81…`; no SK. Independent: `/api/health`
200. FoFline Air session not created (Admin OAuth). Seat
`artifact.pk.hex` not walked. Connected `17793a75-…` is not Measure.
Not MATCH. Do not publish new-SK capsule.

2026-09-21 16:04 +10: seat PK chip still missing after Air
`6d312e59`. Cause: living Runtime had no `plant_from_license`. Lock:
`plant_from_persisted_inner` on `launcher.ota_status`. Measure: unit
tests ok. Lab `192.168.8.10` host-key scan failed — mux pending.
Not MATCH.

2026-09-21 16:13 +10: USB SoT mux Runtime plant. Cause: no sudoers
on seat; extract was old ELF. Lock:
`scripts/empiieros-installer-media-plant-runtime.sh`. Measure: SoT
`e150127f…` = tar; member has `plant_from_persisted_inner`; witness
matches; apply clean; no PK on media. Not official `5979e9cf…`.
Not MATCH. Reinstall required.

2026-09-21 16:34 +10: reinstall did not plant PK. Cause: apply
`plant_idp_date_runtime` always copied live share/bin over SoT
extract. New Air `013834cf` sealed (`edition=connected`,
`capsule_issue_sealed`); living ELF still lacked
`plant_from_license`. Lock: keep extracted IdP-Date Runtime; plant
script also writes share + live bin. Measure pending USB replug +
reinstall. `013834cf` spent. Not MATCH.

2026-09-21 16:43 +10: USB stamped keep-SoT apply `c6618d61…` and
mux Runtime `74a24895…` (SoT `245efa91…` = witness; share + live
match). No PK on media. Not official `5979e9cf…`. Not MATCH.
Reinstall + new FoFline Air (`013834cf` spent).

2026-09-21 17:14 +10: PK chip green, Stage yellow, Apply blinks.
Cause: public `current.json` `launcher-path-1` does **not** verify
under living PK `813cfc81…` (signed 01:48Z with the prior key);
glass `refresh()` wiped request/apply errors; official extract likely
lacks `empiieros-ota-fetch`; cutover dests are RO under
`ProtectSystem=strict`. Lock: keep request/apply errors; helper in
`ota_gate`; apply plants `ota-cutover.conf`; resign current to living
SK. Measure pending republish + helper on seat. Not MATCH.

2026-09-21 17:24 +10: USB mux plant (not official `out/`). Apply
`01fd70dd…` plants helper + cutover RW; share helper `d8ec3cc8…`;
seat-shell `index-C33yKaFx.js` (`clearFeedback`). SoT tar still
`245efa91…` / Runtime `74a24895…`. No PK on media. Not MATCH.

2026-09-21 17:42 +10: Apply on the living seat returned
`invalid runtime response framing`. Cause: `restart_allowlisted`
`try-restart`ed Shell GUI then Runtime before the IPC newline.
Lock: restart only changed binaries; Runtime/GUI via
`systemd-run --on-active=3s`; `shell/` → `ota_reload_shell`.
Measure: `cargo test ota_cutover::` 4 ok; host release
`f734ed76…`. USB unplugged — mux plant of this Runtime not done.
Not MATCH. Do not apply `launcher-path-1-living-pk` over mux
`index-C33yKaFx.js` (that pack is the morning shell).

2026-09-21 17:57 +10: USB mux Runtime plant (not official `out/`).
Stick `071C57D0F48A4A92`. SoT `c9993f85…` = share = live =
host `f734ed76…` (`--on-active=3s`). Apply still `01fd70dd…`
keep-SoT. Helper `d8ec3cc8…`. No PK on ESP/share. Supersedes
SoT `245efa91…` / Runtime `74a24895…`. Not MATCH. Reinstall
required; do not apply morning `launcher-path-1-living-pk`.

2026-09-21 18:05 +10: Operator: PK + stage green. Official
application overlay `0e0e19d8…` Runtime `6737388a…` has neither
`--on-active=3s` nor `plant_from_persisted_inner`. Lock:
packager refuses those misses + plants `ota-cutover.conf`;
`docs/plans/OTA-OPERATOR-RUNBOOK.md`. Official `out/` still
`5979e9cf…`. Not MATCH.

2026-09-21 18:06 +10: `package-shell-gui-rootfs.sh` (no
`--build-image`) → overlay `7590c54c…` Runtime `2783fa8a…`
(`--on-active=3s`, `ota_artifact_pk`), helper `3287ed88…`,
Shell `index-BKgVdGT6.js`, drop-in `ota-cutover.conf`. Not
official `out/` `5979e9cf…`. Not MATCH.

2026-09-22 02:50 +10: QEMU Phase 0/2 state LUKS (not MATCH).
Official `out/` rebake 22 Sep 02:28 (`rootfs.tar.gz` +
agent). `scripts/qemu-installed-system-smoke.sh`: install →
`ask-captured-20260921.ini` (Id `cryptsetup:…uuid…`, Message
`empiieros-state`) → recovery on serial → mount
`/var/lib/empiieros-runtime`. Log:
`eliteew-ts-factory-work/logs/qemu-installed-system-smoke-20260921T170005Z.log`.
`cargo test` kinship `state-unlock-agent` 7 ok. Metal capture
still recommended; no TPM path in this run.

2026-09-22 QEMU M5 adversarial (not MATCH). Same smoke script:
wrong recovery `1111-…-8888` → branded prompt, no state mount;
empty passphrase → cancel path, no mount; then valid recovery →
mount. Log:
`eliteew-ts-factory-work/logs/qemu-smoke-m5-20260921T174405Z.log`.
Cause fix: bash `${3:-$RECOVERY}` treated empty `$3` as missing and
sent real recovery on the “empty” boot.

2026-09-22 live USB flash OK (not MATCH). `/dev/sda` FLASH DRIVE
`071C57D0F48A4A92` (`TRAN=usb` `RM=1`). BR SoT
`cf0b46e1b92e9efb3820dd5a9da456dd120ebab9fcf39f9f159c6511bb30247b`;
seat-shell `assets/index-2MlFf2OC.js`; stamp gate fix: clock copy
needle `Установите время перед активацией` (replacing stale
`Сначала поставьте часы`). Operator log: flash PASS,
`PARTUUID=de99be1c-0594-4141-b169-04ef32af7916`. Metal install +
installed-boot LUKS walk pending.

2026-09-22 OTA publish (not MATCH). `scripts/empiieros-ota-pack-shield-wave.sh`
→ `app_build_id=shield-armed-notify-20260922`, `runtime/empiieros-runtime`
`ce10436a…`, shell `index-B94tq4K2.js`. Ingest via `prod-core`
(`EMPIERO_OS_OTA_PUBLISH_SSH=prod-core`). Public
`https://app.eliteew.ru/empiieros-ota/v1/current.json` → same build id.
Seat apply not run from this workstation.

2026-09-22 OTA publish wave A+B (not MATCH). Repack with Buildroot-cross
runtime + `helpers/seat-shield-{fw,watch,probe}` + shell toast/Shield UI:
`app_build_id=shield-full-abc-20260922`, runtime `fc6bab38…`, shell
`index-oudjqhjO.js` (45 files). Ingest via `prod-core`; public edge
`current.json` matches build id. Metal APPLY_OTA not run here.

2026-09-22 OTA publish scroll-pass (not MATCH). Cause of dead wheel after
`scroll-shell-20260922`: GTK `on_scroll_event` returned TRUE and swallowed
the event; injected JS was the only path and missed scrollports. Lock:
observe-only scroll handler (return FALSE); SecureApp always installs
`installViewportLockedWheelScroll`. `app_build_id=scroll-pass-20260922`,
shell-gui `d80a888c…`, shell js `01ca1cf9…`. Edge published via prod-core.
Seat: Request → APPLY_OTA again.

2026-09-22 OTA publish scroll-ota-ui (not MATCH). Cause: GTK scroll-event
handler (even FALSE) still blocked WebKit wheel; OTA glass hid Install in
a spoiler and stuck `ota_apply=applied` after any prior cutover so a new
stage looked already installed. Lock: remove GTK scroll connect/masks;
DOM walker only; status exposes applied/staged build ids + next_action;
Install block visible when ready. `app_build_id=scroll-ota-ui-20260922`.
Seat: Updates → Check → Install (APPLY_OTA) → wait GUI restart.
