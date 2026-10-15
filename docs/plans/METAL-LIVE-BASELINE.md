# Live metal seat baseline

**Status:** Evidence artifact only. Not an acceptance or readiness claim.
**Verdict context:** EmpiierOS remains `NOT_READY`.
**Captured:** 2026-09-17T02:28:22Z (UTC)
**Host:** `192.168.8.10` (`eliteew-ts`, kernel `6.12.20`)
**Harness pin:** `EMPIIEROS_METAL_HOSTKEY_SHA256=SHA256:msPNx+ysC+7DZLmBWoDsa2SQpUS4p0QxBsTgGX8TTuU`
**Method:** `scripts/metal-live.sh connect` / `check` / `probe` plus read-only `sudo -n` inspection over the same pinned ControlMaster. No USB write. No SSH or sudoers change.

This file records the current laboratory seat so later transactional deploys can be compared. It does not authorize removing live sudo, flashing an image, or declaring metal-ready.
Later chronological narrative: [`SECURE-RESET-LIVE-DIARY.md`](SECURE-RESET-LIVE-DIARY.md).
Current gates: [`../FOUNDATION-STATUS.md`](../FOUNDATION-STATUS.md).

## 1. How to reproduce

```bash
EMPIIEROS_METAL_HOSTKEY_SHA256=SHA256:msPNx+ysC+7DZLmBWoDsa2SQpUS4p0QxBsTgGX8TTuU \
  scripts/metal-live.sh connect
EMPIIEROS_METAL_HOSTKEY_SHA256=SHA256:msPNx+ysC+7DZLmBWoDsa2SQpUS4p0QxBsTgGX8TTuU \
  scripts/metal-live.sh check
EMPIIEROS_METAL_HOSTKEY_SHA256=SHA256:msPNx+ysC+7DZLmBWoDsa2SQpUS4p0QxBsTgGX8TTuU \
  scripts/metal-live.sh probe
```

Harness whoami on this seat is `uid=0(root)` via `eliteew-ts` NOPASSWD sudo. That path is left in place.

## 2. Identity and boot

| Fact | Observed |
|---|---|
| Hostname | `eliteew-ts` |
| Kernel | `Linux eliteew-ts 6.12.20 #8 SMP PREEMPT_DYNAMIC Thu Sep 17 07:16:30 +10 2026 x86_64` |
| OS | Buildroot `2025.02.1` |
| Boot | 2026-09-17 09:16 local (`+10`); uptime ~3h11m at capture |
| systemd | `256.7` feature string includes `-SECCOMP` (no libseccomp-backed filters) |
| `systemctl is-system-running` | `running` |
| Failed units | none |
| Locale | `LANG=ru_RU.UTF-8`; `/etc/empiieros/ui-locale` = `ru` |
| Time | `Asia/Vladivostok`; NTP active; clock synchronized |

## 3. Harness and deployed artifacts

| Artifact | SHA-256 / id |
|---|---|
| `/usr/local/bin/empiieros-runtime` | `b17b132303211fc98b0ed4c0931ea4b941b141fb05cc32f03613e6ec6931ca15` |
| `/usr/local/bin/empiieros-shell-gui` | `3dc0629b1c6052e8f22dea3f8e71a2c7c58213dbd7d24d8025a195980c4d836f` |
| `/etc/systemd/system/empiieros-runtime.service` | `2c68dfa6669ac434e18cfe24cbbff115fd99b539b1d7644fbb87b262899d1758` |
| Source unit (Trusted Substrate overlay) | same digest as live unit |
| Latest known Shell transaction | `20260917T020546Z-shell-13110` (gui hash above; assets `c84615aa12091e96c31b48a46c96b332a6fb92f49d8640d453f7f5c1e9ba09b8`) |
| Latest known Runtime transaction | `20260917T014538Z-runtime-20677` (binary hash above; unit hash above) |

Image `/opt/empiieros/build-manifest.json` still records older bake hashes (`empiieros-runtime` `5d995377…`, `empiieros-shell-gui` `4ead4319…`). Live binaries have been replaced transactionally after that bake.

Shell assets at capture:

- `/opt/empiieros/shell` root-owned `0755`
- `index.html` root `0644` 689 bytes
- `assets/index-D7ZyB0wm.js` 261316 bytes
- `assets/index-BkITMRFI.css` 10445 bytes

## 4. Runtime, IPC, capabilities, seccomp

`empiieros-runtime.service` is `active/running`, `MainPID=29066`, `NRestarts=0`, `ExecMainStatus=0`.

| Control | Unit file | `systemctl show` / `/proc` |
|---|---|---|
| `CapabilityBoundingSet` / `AmbientCapabilities` | `CAP_NET_ADMIN` | `cap_net_admin`; `CapEff=0000000000001000` |
| `CAP_SYS_PTRACE` | absent | absent |
| `NoNewPrivileges` | true | `1` |
| `ProtectSystem` | `strict` | `strict` |
| `SystemCallFilter=~@debug` | present in live **and** source unit | **not applied**: `SystemCallFilter=~`, `SystemCallArchitectures=`, `SystemCallErrorNumber=0` |
| Process seccomp | n/a | `Seccomp: 0`, `Seccomp_filters: 0` |

Sockets:

- `/run/empiieros` `0700 root:root`
- `/run/empiieros/runtime.sock` `0600 root:root`
- `/run/empiieros-shell` `0755 root:root`
- `/run/empiieros-shell/runtime.sock` `0600 eliteew-ts:eliteew-ts`
- `empiieros-shell-bridge.socket` active
- Peer: `systemd-socket-proxyd --connections-max=16 /run/empiieros/runtime.sock`

Journal at probe: repeated `ok=true status=observed` for `id=shell-*`. Six boot-lifetime runtime lines matched `denied|error|fail|peer_|unauthorized|ok=false` (historical, not a current restart storm).

**Cause of `Seccomp: 0` despite `SystemCallFilter`:** the live unit is not stale. systemd reports `-SECCOMP` and `libseccomp` is absent, so systemd ignores `SystemCallFilter`. Kernel seccomp actions are present (`/proc/sys/kernel/seccomp/actions_avail` lists `kill_process … allow`). This image cannot ship systemd filters until systemd is rebuilt with seccomp.

`systemctl show` also reports `LockPersonality=no`, `MemoryDenyWriteExecute=no`, `RestrictRealtime=no`, `RestrictSUIDSGID=no`, `RestrictNamespaces=no`, `RestrictAddressFamilies=~` even though those keys exist in the unit. Treat them as compile-feature / parser gaps on this systemd, not as proof the unit file is missing.

## 5. Identities, sudo, SSH (unchanged)

| Account | Live |
|---|---|
| `root` | uid 0, `/bin/sh` |
| `eliteew-ts` | uid 105 gid 113; groups `video,render,input,seat,tty,audio` |
| `empiieros-maint` | **absent** (source `users.table` defines uid 106; not on this install) |

Sudo (left in place):

- `/etc/sudoers.d/99-temporary-maintenance` — `eliteew-ts ALL=(ALL:ALL) NOPASSWD: ALL`
- `/etc/sudoers.d/wheel` — comments only; no ambient wheel privilege

SSH:

- `sshd.service` active; drop-in `/etc/ssh/sshd_config.d/10-empiieros-security.conf` exists (`PermitRootLogin no`, `PasswordAuthentication no`, `AllowUsers eliteew-ts`, publickey only)
- `/etc/ssh/sshd_config` has **no** `Include` of `sshd_config.d`
- `sshd -T -f /etc/ssh/sshd_config` reports `permitrootlogin without-password`, `passwordauthentication yes`, `authenticationmethods any`
- Authorized key present only on `/home/eliteew-ts/.ssh/authorized_keys` (`ssh-ed25519` comment `owned@punisher-prod-deploy`)
- Root and `empiieros-maint` authorized_keys: missing

Do not treat the drop-in as enforced until `sshd -T` agrees. Policy was not changed in this capture.

Getty: `getty@tty1` autologin `eliteew-ts`. Legacy `eliteew-ts-kiosk.service` is a no-op (`ConditionPathExists` missing). Seat is `bash /usr/local/bin/eliteew-ts-kiosk` → `dbus-run-session -- labwc`.

Session supervisor binary: absent. `empiieros-maint` binary: absent.

## 6. Shell / WebKit

Processes at capture:

| PID | Comm | Uid | CapEff | Seccomp |
|---|---|---|---|---|
| 33193 | `sudo` (revived GUI wrapper, ppid 1) | 0 | full | 0 |
| 33196 | `empiieros-shell` | 105 | 0 | 0 |
| 33211 | `WebKitNetworkPr` | 105 | 0 | 0 |
| 33214 | `WebKitWebProces` | 105 | 0 | 0 |

Present: `/usr/libexec/webkit2gtk-4.1/WebKitWebProcess`, `WebKitNetworkProcess`, `jsc`.
Absent: `libseccomp`, `*webkit*sandbox*` helper, `systemd-analyze`, `rfkill`, `wpa_cli`, `empiieros-maint`.

WebKit processes exist, so the GUI is using split process model, but there is no sandbox helper and `Seccomp=0` on UI and WebKit pids.

## 7. Network and radio

| Link | State | Address |
|---|---|---|
| `eno1` | UP, networkd `routable/configured` | `192.168.8.10/30` metric 1024; default via `192.168.8.9` |
| `wlp58s0` | UP, `routable/configured` | `192.168.38.82/24` metric 1024; associated SSID `EWKaska`, ch 52 / 80 MHz |
| Dual default routes | both DHCP defaults present | Ethernet is the lab SSH path |

wait-online: **active / success** via drop-in
`/etc/systemd/system/systemd-networkd-wait-online.service.d/10-empiieros.conf`:
`--any --ipv4 --timeout=30`, `TimeoutStartSec=35`.

Installed Ethernet profile `10-eliteew-ts-install-eno1.network`: DHCP IPv4, `RequiredForOnline=yes`.
Wireless profile `20-wireless.network`: `RequiredForOnline=no`.
PSK material exists at `/etc/wpa_supplicant/wpa_supplicant-wlp58s0.conf` (`0600`); contents were not copied into this document.

`rfkill`: **not found**.
`ath10k_pci` is present in `/sys/module` (built-in on this kernel; userspace `lsmod` line not required). Firmware files exist under `/lib/firmware/ath10k/QCA6174/hw3.0` including `firmware-6.bin`. dmesg: initial `could not probe fw (-2)` at 2.48s; later PCI rebind loaded firmware `WLAN.RM.4.4.1-00309`.

`empiieros-wifi-restore.service`: oneshot, `Result=success`, currently inactive (expected). Reboot Wi-Fi restore was **not** re-run in this capture.

Hardware snapshot: `BAT0` 100% Full; `thermal_zone0` 25000; `thermal_zone1` 43000; DRM `card0-eDP-1` connected, other listed connectors disconnected.

## 8. Image / package gaps versus source contract

| Source / next-image intent | This live image |
|---|---|
| Runtime unit `SystemCallFilter=~@debug` | File present; **not enforced** (`systemd -SECCOMP`) |
| `libseccomp` | absent |
| `systemd-analyze` | absent |
| WebKitGTK sandbox helper | absent |
| `empiieros-maint` user + key-only SSH | absent |
| Restricted sudo / no NOPASSWD ALL | **temporary NOPASSWD ALL is present** (operator: leave it) |
| Modular ath10k + `rfkill` | firmware on disk; driver builtin; `rfkill` absent |
| Session supervisor / privilege drop | absent |
| Persistent push-event IPC | not measured here; Shell continues to generate snapshot traffic |

## 9. What this baseline does **not** prove

- Metal-ready, image-ready, `PASSED`, or soak
- Reboot persistence of Wi-Fi
- Physical glass F11 / wheel / touch
- Positive reboot / poweroff
- That `sshd_config.d/10-empiieros-security.conf` is actually loaded
- That a future Runtime/Shell deploy still matches these hashes

## 10. Leftover for later plan items

1. Trust-floor: systemd seccomp cannot be proven on this bake; WebKit API now requests a sandbox but this WebKit was built with `ENABLE_BUBBLEWRAP_SANDBOX=0`; no maint identity; no session supervisor.
2. Boot-network: wait-online works after reboot; in-session Wi-Fi restore proved. Reboot restore **failed**: ath10k firmware probe `-2` at 2.5s, `empiieros-wifi-restore` logged `Wi-Fi interface unavailable: wlp58s0` and (old script) exited 0. Manual PCI rebind then restore brought `EWKaska` back. `rfkill` still missing. Restore script now waits 15s and exits 1 if no interface starts.
3. clean-metal: blocked this session (no USB, do not remove sudo).

## 11. Task B follow-up — 2026-09-17T02:32:31Z

Cause of live `Seccomp: 0` despite `SystemCallFilter=~@debug`: the live unit digest matches source (`2c68dfa6…`). systemd 256.7 is built `-SECCOMP` and `libseccomp` is absent, so the filter is ignored. Redeploying the unit cannot change that.

WebKitGTK 2.46.4 on this seat was compiled with `ENABLE_BUBBLEWRAP_SANDBOX=0` / `ENABLE_SANDBOX_EXTENSIONS=0`. `bwrap` exists; `xdg-dbus-proxy` and `libseccomp` do not.

Source/deployable work applied:

- `runtime/shell-gui/secure_main.c` now calls `webkit_web_context_set_sandbox_enabled(TRUE)` and `add_path_to_sandbox(SHELL_ROOT)` before the web view is created.
- Shell transaction `20260917T023206Z-shell-22476` (`gui_sha256=004862bac30b8ba4fbea8f6fd4c0de34ae1490ab1173ea78c67dbff9f7b63af0`).
- Journal: `WebKit sandbox enabled root=/opt/empiieros/shell`, `shell render ready attempts=1`, assets served, Runtime `ok=true status=observed` continues.
- Next-image lock: explicit `BR2_PACKAGE_XDG_DBUS_PROXY=y` beside existing `WEBKITGTK_SANDBOX` / `BUBBLEWRAP` / `LIBSECCOMP`.

Honest live FAIL (not rolled back; GUI stayed reachable):

| Probe | Result |
|---|---|
| Runtime `/proc/<pid>/status` `Seccomp` | `0` / `Seccomp_filters: 0` |
| WebKit UI / Network / Web processes | `Seccomp: 0`, same `pid` namespace as the GUI |
| `systemd-analyze` | absent |
| `empiieros-maint` | still absent (not added) |
| Session supervisor | not shipped (would risk the SSH/GUI seat) |

`CAP_SYS_PTRACE` was not added. Temporary sudoers left in place.

## 12. Task C boot-network follow-up — 2026-09-17T02:35:05Z

Reboot return path: Ethernet `192.168.8.10/30` returned in ~38s; harness reconnect succeeded. wait-online `Result=success` (~11s).

Wi-Fi after reboot: **no `wlp58s0`**. dmesg `could not probe fw (-2)` at 2.52s. `empiieros-wifi-restore` logged `Wi-Fi interface unavailable: wlp58s0`. Manual `ath10k_pci` unbind/bind at ~62s loaded firmware; restore then associated `EWKaska` / `192.168.38.82/24`. PSK not printed.

Source restore script now waits up to 15s and exits 1 if nothing starts. Installed live at `/usr/local/sbin/empiieros-wifi-restore` (`1de1cdbe…`). Modular ath10k remains next-image.

## 13. Truth-state follow-up — 2026-09-17T02:39:25Z

Runtime transaction `20260917T023925Z-runtime-28591` (`a09eea8b…`). Shell transactions `20260917T024148Z-shell-14186` then `20260917T024341Z-shell-5369`.

Metal frontend `system.watch` returned correlated events + snapshot + `freshness`. `wait_ms=5001` returned `watch_rejected` / `wait_ms_exceeds_5000`. Direct backend connect as the probe binary returned `peer_executable_unavailable`. Persist file `/var/lib/empiieros-runtime/events.jsonl` is `0600 root:root`.

## 14. Shell-UX follow-up — 2026-09-17T02:43:48Z

GUI `f8907a59…`. Journal: `WebKit DOM diagnostics disabled`, `shell window state observed mode=kiosk` (startup + map), sandbox still requested, render ready. F11/wheel/touch glass checks remain operator-visible.

## 15. Capability negatives + F11 — 2026-09-17T02:48:11Z / 02:50:31Z

Runtime transaction `20260917T025031Z-runtime-14174` (`39859a2d…`) after the networkd JSON address parser. Shell GUI unchanged (`f8907a59…`). Sudoers `/etc/sudoers.d/99-temporary-maintenance` left in place. No USB flash. No positive reboot/poweroff.

Frontend socket `/run/empiieros-shell/runtime.sock` (authorized via proxyd):

| Case | Result |
|---|---|
| `system.power` missing `confirm` / empty params / `confirm=NO` / reboot+`POWEROFF` / extra field | `failed` / `confirmation_does_not_match_action` or `invalid_params`; both power units stayed `inactive` |
| `audio.control`, `display.control`, `input.action`, `time.action`, `settings.action`, `security.integrity.run`, `operator.auth.action`, `update.apply`, `enroll.welcome_policy`, `power.action`, `onboarding.start`, `license.activate`, `factory_reset.execute`, `product.open`, `window.registry`, `device.keys.ensure`, `entitlement.activate`, `watchdog.trigger`, `substrate.profile.write`, `self_healing.run`, `notification.post` | `not_available` / `capability_not_available` |
| unknown op | `not_found` |
| extra request field / `params=[]` / `System.Power` / empty id / 65537-byte body | `invalid_request` or `request_too_large` |
| backend `/run/empiieros/runtime.sock` as probe binary | `denied` / `peer_executable_unavailable` |
| `network.wifi.connect` without PSK | `psk_required_unless_allow_open_is_true` |
| `network.configure` `lo` / missing iface / `method=magic` / `sit0` / `eno1` static wrong gateway | fail-closed; `eno1` stayed `192.168.8.10/30` |
| `network.wifi.scan` | `observed`; 12 access points including `EWKaska` (no PSK printed) |
| `network.configure` DHCP `wlp58s0` | `network_verification_failed`; Runtime wrote then rolled back `05-empiieros-wlp58s0.network`; `NetworkFile` stayed `/etc/systemd/network/20-wireless.network` even after a host-side write+`networkctl reload`. Cause: this image's networkd does not rebind an already-claimed link to a new equally-DHCP file without restarting `systemd-networkd` (not done; would flap `eno1`). |

Evdev `KEY_F11` on `/dev/input/event4` logged `shell window mode=windowed` then restore `shell window mode=kiosk`. Wheel/touch: no production syslog from `on_scroll_event`; DOM diagnostics stay off; **not** measured.

## 16. Leftover quarantine + QEMU selftest — 2026-09-17T02:55:00Z

Unused host-side `/etc/systemd/network/05-empiieros-wlp58s0.network` did not own either link (`eno1` → `10-eliteew-ts-install-eno1.network`, `wlp58s0` → `20-wireless.network`). Renamed in place to `05-empiieros-wlp58s0.network.unused` with no `networkctl reload` and no `systemd-networkd` restart. After: same `NetworkFile` values, `eno1` `192.168.8.10/30`, `wlp58s0` `192.168.38.82/24`, sudoers still present.

QEMU/image gate (Trusted Substrate root, no USB):

```bash
cd ../powered_by_emperyworld/pack-organization/trusted-substrate
ELITEEW_TS_SECURE_RESET_QEMU=1 bash scripts/selftest-secure-reset.sh
```

First run exit 1 before QEMU: `packages_not_allowed: WPA_SUPPLICANT_CTRL_IFACE,WPA_SUPPLICANT_NL80211,WPA_SUPPLICANT_WPA3` (already `=y` in `20-network-base.frag` and installer selftest). Allowlist gained those three names. Adversarial `TOTALLY_EVIL_BLOB` still rejected. Second run exit 0 against existing `out/bzImage` (20M, 2026-09-17 07:16), `out/rootfs.ext2` (12G, 08:34), `out/rootfs.tar.gz` (312M, 08:35). Decisive lines: `PASS  QEMU EmpiierOS Runtime + network policy + stable Wayland seat`; `PASS  QEMU real installer → installed disk → Shell/Runtime/network/regional boot`; `PASS  Trusted Substrate secure-reset source acceptance`. That image is older than live Runtime `39859a2d…` / GUI `f8907a59…`. Not metal-ready.

## 17. network.configure ownership + wheel path + next-image locks — 2026-09-17T03:05:31Z / 03:09:17Z

Harness pin `EMPIIEROS_METAL_HOSTKEY_SHA256=SHA256:msPNx+ysC+7DZLmBWoDsa2SQpUS4p0QxBsTgGX8TTuU`. No USB flash. No sudoers change. No `systemd-networkd` restart. No matching `system.power`.

### Ownership (live)

Cause: this image's networkd rematches the already-bound `.network` path after `networkctl reload` and ignores a newly created `05-empiieros-*.network`. Excluding `Name=wl* !wlp58s0` unmanaged the radio.

Lock: Runtime `claim_plan` rewrites `/etc/systemd/network/20-wireless.network` in place (mode `0644`, previous bytes saved for rollback) when the claimed interface is Wi-Fi and that file exists. Ethernet still gets a new `05-empiieros-<iface>.network`. Apply is `networkctl reload` + `reconfigure` only.

Measure:

- `cargo test --locked --manifest-path runtime/Cargo.toml --lib`: 42 passed, including `wifi_claim_rewrites_the_already_bound_wireless_file` and systemd octet `Address` parse.
- Deploy Runtime `20260917T030531Z-runtime-18673` sha256 `f3bb25d6b72c4c319b8d5397d87aa9afa4c155bfb6b06eca62b537a9107242b8`.
- Frontend `network.configure` `id=net-wifi-own-3` `wlp58s0`/`dhcp`: `ok=true` `status=applied` `config_file=20-wireless.network` `observed_ipv4=["192.168.38.82/24"]`.
- After: `20-wireless.network` is Runtime-rendered `Name=wlp58s0`; `NetworkFile` stayed `/etc/systemd/network/20-wireless.network`; `eno1` stayed `192.168.8.10/30` / `10-eliteew-ts-install-eno1.network`; SSID `EWKaska`; no `05-empiieros-*` under `/etc/systemd/network`; sudoers present.

### Wheel/touch

GUI `55b84266afd0483b9f9f17e7b27ff278b5ffe17b5d69667ab50e7da6e475cd51` (`20260917T030246Z-shell-14544`) logs `shell scroll observed` from GTK `on_scroll_event` before `window.scrollBy` (not a CSS fake). Evdev `REL_WHEEL` writes to every `/dev/input/event*` returned 0. No journal line. Seat is Wayland (`wayland-0`). Only pointer is `PS/2 Generic Mouse` (`event9`, `REL=3`, no wheel bit). `/dev/uinput` and `uinput.ko` are absent (`modprobe: Module uinput not found`). `ydotool`/`wtype`/`evemu` absent. **BLOCKED:** evdev cannot generate a genuine GTK scroll on this kernel.

### Next-image source locks (not live-true on 192.168.8.10)

Fragments + generated defconfig + allowlist now select `LIBSECCOMP`, `SYSTEMD_ANALYZE`, `UTIL_LINUX_RFKILL`, `WEBKITGTK_SANDBOX`, `BUBBLEWRAP`, `XDG_DBUS_PROXY`. `CONFIG_ATH10K=m` / `ATH10K_PCI=m`. Wired overlay `20-wired.network` keeps `RequiredForOnline=yes`. `selftest-secure-reset.sh` asserts fragment, defconfig, and allowlist together. Full-rootfs `post-build.sh` requires `systemd-analyze`, `bwrap`, `xdg-dbus-proxy`, `libseccomp.so.2`, and `rfkill` when pid1 is present (W4 fixture has no pid1). Factory `.config` at bake start still had those packages unset, so they are not live-true here.

### Current-source overlay + bake

`bash scripts/package-shell-gui-rootfs.sh` wrote overlay `c910e0f701513a0a694cbb57bba0965e778c84751e32a8befb7b6a11a1d77135` (Runtime `b61f7db3…` with `claim_plan`, GUI `55b84266…`). Trusted Substrate `buildroot-build.sh` started 2026-09-17T03:09:17Z against that overlay; QEMU of the new `out/` is pending bake exit. Previous QEMU exit 0 remains the older 07:16/08:34 artifacts. Verdict stays `NOT_READY`.

## 18. Bake unstick — 2026-09-17T03:20:00Z

Two factory runs failed at WebKitGTK 2.46.4 configure on host CMake ≥4 (`if (WTF STREQUAL )` empty). Lock: `config/buildroot/patches/package-dropins/webkitgtk/0001-cmake4-empty-strequal.patch`. Third bake (pid 2721015, started 03:16Z) passed configure and was compiling JavaScriptCore (~1786/6791) when inspected. Host disk was 97% full (`1.6T/1.7T`). Parent forbade a fourth extract/bake.

Reconfirm on metal, no networkd restart: frontend `network.configure` `id=cfg-own-2` `wlp58s0`/`dhcp` → `ok=true` `status=applied` `config_file=20-wireless.network` `observed_ipv4=["192.168.38.82/24"]`. `eno1` stayed `192.168.8.10/30`. Wheel still BLOCKED: only pointer is `PS/2 Generic Mouse` `event9` `REL=3`; no `/dev/uinput`.

## 19. Current-source bake DONE, QEMU FAIL — 2026-09-17T04:13:21Z / 04:14:42Z

Attached only to bake 571825 (pid 2721015). No second extract. That bake compiled WebKit with `ENABLE_BUBBLEWRAP_SANDBOX=ON`, then failed two **new** post-make gates. Both were patched and resumed on the same `../eliteew-ts-factory-work/buildroot-output` via `bash scripts/buildroot-build.sh --resume` (no `rm -rf` of Buildroot source):

1. `CONFIG_BRCMFMAC_PCIE=m` is impossible (Kconfig bool). Fragment now `CONFIG_BRCMFMAC=m` + `CONFIG_BRCMFMAC_PCIE=y`; image gate matches.
2. Stale `target/home/eliteew-ts/.ssh/authorized_keys` from an older overlay survived overlay copy. `post-build.sh` now strips kiosk keys. W4 fixture proves the strip.

Resume 571828 exit **0**. `image_contract_ok`. `[buildroot-build] DONE images → …/trusted-substrate/out/`.

New artifacts (not the 07:16/08:34 set):

| Path | Time | Note |
|---|---|---|
| `out/bzImage` | 2026-09-17 14:11 | 18M |
| `out/rootfs.ext2` | 14:12 | 12G |
| `out/rootfs.tar.gz` | 14:13 | 313M; `rootfs_sha256=5a125dfa…` |
| `out/eliteew-ts-rootfs.stamp` | 14:13 | `payload_profile=empiieros-shell` |

Target userspace now has `systemd-analyze`, `bwrap`, `xdg-dbus-proxy`, `rfkill`, `libseccomp.so.2`. Kernel: `CONFIG_ATH10K=m`, `CONFIG_ATH10K_PCI=m`, `CONFIG_BRCMFMAC=m`, `CONFIG_BRCMFMAC_PCIE=y`. Overlay Runtime `b61f7db3…` / GUI `55b84266…`. These are **image** facts, not live-true on 192.168.8.10.

QEMU against those artifacts:

```bash
cd ../powered_by_emperyworld/pack-organization/trusted-substrate
ELITEEW_TS_SECURE_RESET_QEMU=1 bash scripts/selftest-secure-reset.sh
```

Exit **1**. Source/W4 gates passed. Direct-boot QEMU: Runtime logged `ready transport=unix-peercred-v1 socket=/run/empiieros/runtime.sock` at 1.56s; GUI logged `WebKit sandbox enabled` and `shell render ready`; frontend connects failed `connect: No such file or directory` for the whole 65s window. Decisive line: `verify-runtime-bridge-log: successful Shell snapshot evidence missing`. Installed-system QEMU did not run. No USB. Verdict stays `NOT_READY`.

## 20. Frontend socket ordering cycle — 2026-09-17T04:20:00Z

Re-ran QEMU on official `out/` (`/tmp/empiieros-selftest-newimage-20260917.log`) exit **1**. Cause (not a connect race):

```
sockets.target: Found ordering cycle on empiieros-shell-bridge.socket/start
sockets.target: Found dependency on empiieros-runtime.service/start
sockets.target: Found dependency on basic.target/start
sockets.target: Found dependency on sockets.target/start
Job empiieros-shell-bridge.socket/start deleted to break ordering cycle
```

Runtime `DefaultDependencies=yes` implies `After=basic.target`; `basic.target` is after `sockets.target`. The socket was `WantedBy=sockets.target` and also `After=empiieros-runtime.service`.

Lock: overlay socket no longer has `After=empiieros-runtime.service` or `Before=getty@tty1.service`. `Wants=empiieros-runtime.service` remains. Proxyd service still `Requires=`/`After=` Runtime. `selftest-secure-reset.sh` and `post-build.sh` reject the cycle-forming `After=`.

Measure: source selftest exit 0 (QEMU skipped). `qemu-empiieros-smoke.sh` against a debugfs copy of `out/rootfs.ext2` with only that unit replaced: exit **0**, no ordering cycle, repeated `ok=true status=observed`.

## 21. Official assemble + QEMU of the cycle fix — 2026-09-17T04:23:00Z

`bash scripts/buildroot-build.sh --resume` (no WebKit rebuild) exit 0, `image_contract_ok`, `DONE images → trusted-substrate/out/`. Stamp `rootfs_sha256=106a4ce9c648cbf6e3af4dd9c9dc1b249608995e6fe65dd2988656a12d10dfe3`. Official `out/rootfs.ext2` socket has `Wants=runtime` and no `After=runtime`.

```bash
cd ../powered_by_emperyworld/pack-organization/trusted-substrate
ELITEEW_TS_SECURE_RESET_QEMU=1 bash scripts/selftest-secure-reset.sh
```

Log `/tmp/empiieros-selftest-official-cyclefix.log`. Exit **0**. Decisive lines: `PASS  QEMU EmpiierOS Runtime + network policy + stable Wayland seat`; `PASS  QEMU real installer → installed disk → Shell/Runtime/network/regional boot`. No `Ordering cycle`. No USB. Not live-true on 192.168.8.10. Verdict stays `NOT_READY`.

## 22. Live SSH Runtime with seccomp log — 2026-09-17T04:33:08Z

Cause of the QEMU-only detour: `+SECCOMP` and process `Seccomp: 2` cannot appear on this laboratory image. Live systemd 256.7 reports `-SECCOMP`; `SystemCallFilter=~@debug` stays in the unit and is ignored; `/proc` stays `Seccomp: 0`. USB flash is not authorized, so the new factory image can show `+SECCOMP` only in QEMU. That does not replace metal SSH for deployable Runtime/Shell.

Measure on `192.168.8.10` via `scripts/metal-live.sh` pin `SHA256:msPNx+ysC+7DZLmBWoDsa2SQpUS4p0QxBsTgGX8TTuU`. No USB. No sudoers change.

- Probe: system `running`; `eno1` `192.168.8.10/30`; `wlp58s0` `EWKaska` `192.168.38.82/24`; previous Runtime `f3bb25d6…` `Seccomp: 0`.
- Deploy Runtime `20260917T043308Z-runtime-20737` sha256 `a1cc73c0c071b59cad3c5e2e04ce6c296b431c19a31e578c25c05d01ca1b9ef0`. Unit unchanged `2c68dfa6…`.
- After restart: Runtime `active`, frontend socket `active`, `CapEff=0000000000001000` (`CAP_NET_ADMIN` only).
- Journal: `empiieros-runtime: ready transport=unix-peercred-v1 socket=/run/empiieros/runtime.sock` then `empiieros-runtime: seccomp mode=0 filters=0`.
- Authorized snapshot `id=shell-773b72bb-… ok=true status=observed`.

Official `out/` after a later `--resume` (seccomp log in the packaged binary) is `bzImage`/`rootfs.ext2` 14:29, `rootfs.ext2` sha256 `648dbb33a774d9a2f83d81507b2b1d89b6152aa4b5ccca17667750f668095f64`. `qemu-empiieros-smoke.sh` now requires `seccomp mode=2`, `+SECCOMP`, `WebKit sandbox enabled`, and forbids `Ordering cycle`. That QEMU rerun was not completed in this slice (spawn aborted). Image `mode=2` is not live-true here. Verdict stays `NOT_READY`.

## 23. Live SSH static IPv4 on wlp58s0 — 2026-09-17T04:35:29Z

Harness pin `EMPIIEROS_METAL_HOSTKEY_SHA256=SHA256:msPNx+ysC+7DZLmBWoDsa2SQpUS4p0QxBsTgGX8TTuU`. Frontend `/tmp/empiieros-ipc-probe` as `eliteew-ts` on `/run/empiieros-shell/runtime.sock`. No USB. No sudoers change. No `systemd-networkd` restart.

| Step | Result |
|---|---|
| `network.configure` `id=cfg-static-1` `wlp58s0` static `192.168.38.82/24` gw `192.168.38.1` DNS `1.1.1.1` `8.8.8.8` | `ok=true` `status=applied` `config_file=20-wireless.network` `observed_ipv4=["192.168.38.82/24"]` |
| After static | file `DHCP=no` `Address=192.168.38.82/24`; `networkctl` Address without DHCPv4 tag; `eno1` stayed `192.168.8.10/30` / `10-eliteew-ts-install-eno1.network`; SSID `EWKaska`; no `05-` leftover |
| `id=cfg-dhcp-restore-1` DHCP | `ok=true` `status=applied`; Address again `192.168.38.82 (DHCPv4 via 192.168.38.1)`; `eno1` unflapped |
| `id=cfg-static-badgw` gw `192.168.8.9` | `ok=false` `gateway_is_outside_interface_subnet`; DHCP file unchanged |
| `system.snapshot` `id=live-snap-1` | `ok=true` `sequence=42` `freshness.stale=false`; `wifi.associations` `wlp58s0`/`EWKaska`; `security.status=degraded` |
| `network.wifi.scan` `id=live-scan-1` | `ok=true`; 14 APs including `EWKaska`; no PSK field |
| Runtime `/proc` | `CapEff=0000000000001000`; `Seccomp: 0`; `Seccomp_filters: 0` |

Verdict stays `NOT_READY`.

## 24. Live SSH display.control brightness — 2026-09-17T04:40:33Z

Cause of the first fail: `/sys/class/backlight/intel_backlight` is the normal sysfs class symlink. Treating that directory as a traversal reject blocked a real write. Lock: validate the name, canonicalize into `/sys/devices` or `/sys/class/backlight`, and refuse only symlink `brightness`/`max_brightness` nodes.

Measure on `192.168.8.10` via pinned `metal-live.sh`. No USB. No sudoers change. `eno1` stayed `192.168.8.10/30`.

- Runtime `20260917T044033Z-runtime-30719` sha256 `ba7067afa9756c76154c81ef714675b634a804291196d83e755a577a0fb60143`.
- Helper `/usr/local/sbin/empiieros-display-brightness` `28333544…`; unit `2284e7d6…`.
- Shell `20260917T044056Z-shell-1180` GUI `55b84266…`; assets `index-D7sUgkIj.js` contain `applyBrightness`.
- `display.action` → `not_available`.
- `display.control` `percent=0` → `percent_must_be_10_to_100`; sysfs stayed `120000`.
- Extra field → `invalid_params`.
- `id=disp-75b` `intel_backlight` `75` → `applied` `observed_brightness=90000`; sysfs `90000`.
- `id=disp-100b` → `applied` `120000`; sysfs restored `120000`.
- Snapshot capability `display.control` `available` / `allowlisted_backlight_helper_observed`.

Verdict stays `NOT_READY`.

## 25. Live SSH audio.control volume — 2026-09-17T04:46:44Z

First deploy attempt exit 127: live image has no `ldd`. Helper install was aborted before Runtime deploy.

Cause of the next fail: allowlist used codec names (`Speaker Playback Volume`). ALSA selem names on this seat are `Speaker` / `Headphone` / `Master`. Lock: allowlist those selem names only.

Measure on `192.168.8.10` via pinned `metal-live.sh`. No USB. No sudoers change. `eno1` stayed `192.168.8.10/30`.

- Runtime `20260917T044644Z-runtime-6347` sha256 `cf3cc043d040b494480fe8254f30135973a96973d33f717352e492e5faa0ce9f`.
- Helper `eb402b82…`.
- Shell `20260917T044735Z-shell-32509` assets `index--j1A-AMo.js`.
- `audio.action` → `not_available`.
- `percent=0` → `percent_must_be_10_to_100`.
- `name=Capture` → `audio_control_name_not_allowlisted`.
- `id=aud-25b` `Speaker` `25` → `applied` `observed_percent=24`.
- `id=aud-69b` → `applied` `observed_percent=68`.
- `id=aud-100` restored codec Speaker `Amp-Out vals: [0x3c 0x3c]` (same as before the slice).

Verdict stays `NOT_READY`.

## 26. Live SSH maintenance identity without SSH or sudo change — 2026-09-17T04:52:00Z

Cause: the live seat had `eliteew-ts` only. Factory `users.table` already defines `empiieros-maint` 106:114 with a locked `*` password. This lab cannot receive a provisioned maint key, and `scripts/metal-live.sh` must not change SSH policy.

Measure on `192.168.8.10` via pinned `metal-live.sh`. No USB. No `AllowUsers` change. Sudoers left in place.

- `useradd` created `empiieros-maint` `uid=106` `gid=114` home `/home/empiieros-maint` mode `0700`.
- `/etc/shadow` field is `*`. No `.ssh` directory and no `authorized_keys`.
- `id empiieros-maint` shows only group `114`. The account is not in sudoers.
- `/etc/ssh/sshd_config.d/10-empiieros-security.conf` still has `AllowUsers eliteew-ts`.
- `/etc/sudoers.d/99-temporary-maintenance` is still `eliteew-ts ALL=(ALL:ALL) NOPASSWD: ALL`.
- Kiosk SSH over `eno1` continued to work after the account existed.

Key-only maint SSH is still incomplete: no operator key, and lab SSH remains kiosk-only. Next image source still permits only `empiieros-maint`.

Verdict stays `NOT_READY`.

## 27. Live SSH `network.wifi.reconnect` without PSK in IPC — 2026-09-17T04:52:37Z

Cause: reconnect previously required a new `network.wifi.connect` that carries a PSK, or a substrate restore unit. Lock: `network.wifi.reconnect` only checks that `wpa_supplicant-<iface>.conf` is a root `0600` regular file and restarts the supervised unit. It never reads the file.

Measure on `192.168.8.10` via pinned `metal-live.sh`. No USB. No sudoers change.

- Runtime `20260917T045237Z-runtime-15789` sha256 `24e8d8d66d16f597e5db2219b2e01cca61e88a02051936268ac91537deee1577`. Ready log: `seccomp mode=0 filters=0`.
- Shell `20260917T045249Z-shell-24232` GUI `55b84266…`; assets `index-D338WvBx.js` contain `network.wifi.reconnect` and `reconnectSaved`.
- `wifi-extra-2` with a `psk` field → `invalid_params: unknown field \`psk\`, expected \`interface\``.
- `wifi-extra-3` with `extra` → `invalid_params: unknown field \`extra\`.
- `wifi-dc-1` → `applied` `associated=false`. `wlp58s0` had no SSID; `eno1` stayed `192.168.8.10/30`.
- `wifi-rc-1` → `applied` `ssid=EWKaska` `observed_ipv4=["192.168.38.82/24"]` `source=wpa_supplicant+persist`. Persist file remained `0600 root:root`. Runtime journal for those ids has no PSK.

Reboot firmware restore stays image-blocked. In-session WLAN rfkill is now live-true in §28.

Verdict stays `NOT_READY`.

## 28. Live SSH `network.wifi.radio` WLAN rfkill — 2026-09-17T04:57:53Z

Cause: this image has no `rfkill` binary, but `/sys/class/rfkill/rfkill0` (`phy1`, `type=wlan`) is present. Runtime cannot write `/sys` under `ProtectKernelTunables`. Lock: observe radios in `system.snapshot` and mutate only through `empiieros-wifi-radio.service`, `type=wlan` only.

Measure on `192.168.8.10` via pinned `metal-live.sh`. No USB. No sudoers change.

- Runtime `20260917T045753Z-runtime-1352` sha256 `807a2ed2158634a92260a3d396af429d6b96493b94d47dfd5528c4a0b00c6e1b`.
- Helper `/usr/local/sbin/empiieros-wifi-radio` `f864629a…`.
- Shell `20260917T045755Z-shell-6663` assets `index-CgyAi5JV.js`.
- Snapshot capability `network.wifi.radio` `available` / `allowlisted_wifi_radio_helper_observed`. Wifi domain radios: `phy1` `soft_blocked=false`.
- `type=bluetooth` → `radio_type_must_be_wlan`.
- Extra `psk` field → `invalid_params`.
- `radio-block-1` → `applied` `soft_blocked=true`; sysfs `soft=1 state=0`; `wlp58s0` `DOWN` / not connected; `eno1` stayed `192.168.8.10/30`.
- `radio-un-1` → `applied` `soft_blocked=false`; sysfs `soft=0 state=1`.
- `wifi-rc-radio` restored `EWKaska` `192.168.38.82/24`.

`/usr/sbin/rfkill` is still absent on this seat. Reboot firmware restore stays image-blocked.

Verdict stays `NOT_READY`.

## 29. Live SSH storage devices + Runtime `start_count` — 2026-09-17T05:02:24Z

Cause: snapshot storage reported only a count, so Shell could not show which block devices existed. Runtime recovery after a crash had no durable start counter.

Lock: `probe_block_devices()` lists `/sys/class/block` names (skip `loop*`/`ram*`/`zram*`) with `size` sectors × 512. `StateStore::record_start()` increments `/var/lib/empiieros-runtime/start_count` (`0600` regular file, no symlink follow) and publishes it on `runtime.health` and `system.snapshot`.

Measure on `192.168.8.10` via pinned `metal-live.sh`. No USB. No sudoers change.

- Runtime `20260917T050224Z-runtime-32422` sha256 `cbfa5f988047eecb454b108f3a1ac981cc2599e3ec1e77f72ec7b7865d0cda92`.
- Shell `20260917T050240Z-shell-12122` assets `index-L42cbaJL.js`.
- `stor-snap-1` storage devices: `nvme0n1` `256060514304`, `nvme0n1p1` `536870912`, `nvme0n1p2` `255522242560` — identical to sysfs. `start_count=1`.
- `kill -9` pid `43524` → recovered in 4s as pid `44125`. `NRestarts=1`. File and IPC `start_count=2`.
- Frontend `shell-*` snapshots resumed (`shell_responses=7` after 15:04:00). `eno1` stayed `192.168.8.10/30`. `wlp58s0` stayed `192.168.38.82/24`.
- Live journal still `seccomp mode=0 filters=0`.

Verdict stays `NOT_READY`.

## 30. Live SSH `time.sync` — 2026-09-17T05:08:31Z

Cause: locale/time was observe-only. A clock sync that also changed timezone would be the wrong mutation. Runtime cannot touch the clock under `ProtectClock`.

Lock: `time.sync` `{ "action": "sync" }` only. Helper `empiieros-time-sync.service` runs `timedatectl set-ntp true` and restarts `systemd-timesyncd`. It never writes timezone or a manual clock.

Measure on `192.168.8.10` via pinned `metal-live.sh`. No USB. No sudoers change.

- Runtime `20260917T050831Z-runtime-27614` sha256 `0d8bcda1ddecd0c516db21e8b0e4f5de7007e76a3c4c69e309ddf745c7d75d09`.
- Helper `/usr/local/sbin/empiieros-time-sync` `7910e0b4…`.
- Shell `20260917T050843Z-shell-31126` assets `index-MZG9_8I3.js`.
- Snapshot capability `time.sync` `available` / `allowlisted_time_sync_helper_observed`. Locale: `ru_RU.UTF-8`, `Asia/Vladivostok`, `ntp_synchronized=true`, `ntp_server=162.159.200.1`.
- `action=set_timezone` → `action_must_be_sync`. Extra `timezone` → `invalid_params`. `time.action` → `not_available`.
- `time-sync-1` → `applied` `ntp_synchronized=true` `ntp_server=162.159.200.1`. Timezone and `LANG` unchanged. UTC clock `05:09:25Z` → `05:09:26Z`. `eno1` stayed `192.168.8.10/30`.

Verdict stays `NOT_READY`.

## 31. Live SSH power supplies + thermal zones — 2026-09-17T05:11:18Z

Cause: power snapshot only published a single `highest_temperature_c` and flattened AC/battery, so Shell could not show which sysfs nodes were observed.

Lock: `probe_power()` lists `/sys/class/power_supply` names and `/sys/class/thermal/thermal_zone*` (cooling devices skipped). Millidegree `temp` is divided by 1000. `highest_temperature_c` is the max observed zone.

Measure on `192.168.8.10` via pinned `metal-live.sh`. No USB. No sudoers change.

- Runtime `20260917T051118Z-runtime-10930` sha256 `5276bad3adc1228c68c82681b205eb43f566a2acb796d80ea3c684528f57a08b`.
- Shell `20260917T051130Z-shell-10375` assets `index-BjXv33gt.js`.
- `therm-snap-1` supplies: `AC` `mains` `online=true`; `BAT0` `battery` `capacity=100` `state=full`.
- Thermal zones: `thermal_zone0` `acpitz` `25.0`; `thermal_zone1` `x86_pkg_temp` `45.0` (sysfs dump in the same SSH session was `25000` / `44000` — 1 °C drift). `highest_temperature_c=45.0`.
- `eno1` stayed `192.168.8.10/30`.

Verdict stays `NOT_READY`.

## 32. Live SSH input evdev bits — 2026-09-17T05:13:30Z

Cause: input snapshot listed node names only, so Shell could not show why wheel scroll is BLOCKED.

Lock: `probe_input()` reads sysfs `capabilities/ev` and `capabilities/rel`. Snapshot publishes `has_key` / `has_rel` / `has_abs` / `has_rel_wheel` per event node, plus `rel_wheel_observed` and `uinput_observed`.

Measure on `192.168.8.10` via pinned `metal-live.sh`. No USB. No sudoers change.

- Runtime `20260917T051330Z-runtime-29086` sha256 `2e3e05d6f56242d1c69fdcac83d057323099b2cccd06bfbb64b7698064d11b89`.
- Shell `20260917T051352Z-shell-30109` assets `index-C4jfk3-A.js`.
- `input-snap-1`: 10 event nodes. `event4` keyboard `has_key=true`. `event9` `PS/2 Generic Mouse` `has_rel=true` `has_rel_wheel=false` (sysfs `ev=7` `rel=3`). `rel_wheel_observed=false`. `uinput_observed=false`. `/dev/uinput` absent.
- `eno1` stayed `192.168.8.10/30`.

Verdict stays `NOT_READY`.

## 33. Live SSH identity observe + watchdog absence — 2026-09-17T05:20:27Z

Cause: a first watchdog probe hung the SSH mux by reading sysfs `timeleft`. This image also has no `/dev/watchdog` and no `/sys/class/watchdog`. Seat identity (kiosk vs maint, SSH allow list, leftover sudoers) was not in the snapshot.

Lock: snapshot `identity` reads `/etc/passwd` and sshd drop-ins only. It records whether `/etc/sudoers.d/99-temporary-maintenance` exists and never reads that file. Watchdog observation lists `/sys/class/watchdog/watchdog*` identity/state/timeout only — never `timeleft`, never `/dev/watchdog`.

Measure on `192.168.8.10` via pinned `metal-live.sh`. No USB. No sudoers change.

- Runtime `20260917T052027Z-runtime-18842` sha256 `534386250d92c1a00d7e6c6fe076c857069394ae0f8fb9eed05fc2b1bdb004ec`.
- Shell `20260917T052039Z-shell-29178` assets `index-kgycIST0.js`.
- `id-snap-1`: `kiosk_user_observed=true`, `maint_user_observed=true`, `ssh_allow_users=["eliteew-ts"]`, `password_authentication=false`, `permit_root_login=false`, `temporary_sudoers_present=true`.
- Watchdog `present=false` `status=unavailable`. Capability `hardware.watchdog` `unavailable` / `watchdog_not_observed`.
- `eno1` stayed `192.168.8.10/30`.

Verdict stays `NOT_READY`.

## 34. Official QEMU smoke `+SECCOMP` / `mode=2` — 2026-09-17T05:23:53Z

Cause: live 192.168.8.10 systemd is `-SECCOMP` (`mode=0`). The next-image locks had to be proven on official `out/`, not on this laboratory seat.

Lock: `qemu-empiieros-smoke.sh` requires `seccomp mode=2`, `+SECCOMP`, `WebKit sandbox enabled`, and forbids `Ordering cycle`.

Measure on official artifacts only. No USB. Not live-true on 192.168.8.10.

- `out/rootfs.ext2` sha256 `648dbb33a774d9a2f83d81507b2b1d89b6152aa4b5ccca17667750f668095f64` (14:29).
- `out/bzImage` sha256 `04fab7e1f7ba6dcd4dfdbc4e72519120d934263f25ded22762871cf6308dd4f1`.
- Image Runtime sha256 `a1cc73c0c071b59cad3c5e2e04ce6c296b431c19a31e578c25c05d01ca1b9ef0` (older than live display/audio/time/radio helpers).
- Image userspace: `bwrap`, `xdg-dbus-proxy`, `systemd-analyze`, `/usr/sbin/rfkill`, `libseccomp.so.2`, proxyd. Helpers `empiieros-time-sync` / `wifi-radio` / `display-brightness` / `audio-volume` are absent from this image.
- Factory kernel: `CONFIG_SECCOMP=y`, `CONFIG_ATH10K=m`, `CONFIG_ATH10K_PCI=m`.
- `scripts/qemu-empiieros-smoke.sh` exit **0**. Journal: systemd `+SECCOMP`; Runtime `seccomp mode=2 filters=28`; `WebKit sandbox enabled`; authorized Shell snapshots; no `Ordering cycle`.
- Live journal on 192.168.8.10 remains `seccomp mode=0 filters=0`.

Verdict stays `NOT_READY`.

## 35. Official QEMU selftest `648dbb33…` — 2026-09-17T05:26:56Z

Cause: smoke proved the image seat. The full official installer path still had to be re-run after those smoke locks.

Lock: `ELITEEW_TS_SECURE_RESET_QEMU=1 bash scripts/selftest-secure-reset.sh` against official `out/`.

Measure on official artifacts only. No USB. Not live-true on 192.168.8.10.

- Same `out/rootfs.ext2` `648dbb33a774d9a2f83d81507b2b1d89b6152aa4b5ccca17667750f668095f64` and bzImage `04fab7e1f7ba6dcd4dfdbc4e72519120d934263f25ded22762871cf6308dd4f1`.
- Selftest exit **0** in 148242 ms. Direct-boot and installer→installed-disk both logged systemd `+SECCOMP`, Runtime `seccomp mode=2 filters=28`, `WebKit sandbox enabled`, and authorized Shell snapshots.
- `PASS  QEMU EmpiierOS Runtime + network policy + stable Wayland seat`
- `PASS  QEMU real installer → installed disk → Shell/Runtime/network/regional boot`
- `PASS  Trusted Substrate secure-reset source acceptance`
- No `Ordering cycle`. No `FAIL` lines.
- Image Runtime remains `a1cc73c0…` and still lacks later live helpers.
- Live 192.168.8.10 remains `seccomp mode=0 filters=0`.

Verdict stays `NOT_READY`.

## 36. Live SSH after official selftest — 2026-09-17T05:35:00Z

Cause: `wlp58s0` was DOWN (`empiieros-wifi@wlp58s0` inactive) before the next slice. A later `deploy-runtime` of DRM connector fields did not observe an authorized Shell snapshot. During the rollback wait, PID 1 received `SIGINT` and activated `reboot.target`. That is not `system.power` (power unit journal empty).

Lock: keep the previous Runtime `53438625…` on this seat until a deploy proves a Shell snapshot without a reboot. Reboot Wi-Fi restore remains fail-closed; radio returns only after `ath10k_pci` unbind/bind.

Measure on `192.168.8.10` via pinned `metal-live.sh`. No USB. No sudoers change.

- `wifi-rc-live-1` applied `EWKaska` `192.168.38.82/24`. `eno1` stayed `192.168.8.10/30`.
- Deploy of Runtime `68f60068…` failed: no authorized snapshot; rollback then `CRITICAL` exit 70. Seat rebooted at 15:33:12 (`Received SIGINT`).
- After boot: Runtime hash `534386250d92c1a00d7e6c6fe076c857069394ae0f8fb9eed05fc2b1bdb004ec`, `seccomp mode=0 filters=0`, GUI `empiieros-shell-gui` active, sudoers `440` still present.
- `empiieros-wifi-restore.service` failed. `wlp58s0` absent until PCI rebind loaded `WLAN.RM.4.4.1-00309`. `wifi-rc-boot-1` then applied `EWKaska` `192.168.38.82/24`. `eno1` stayed `192.168.8.10/30`.
- DRM connector modes are in the working tree only. They are not live-true.

Verdict stays `NOT_READY`.

## 37. Live SSH DRM connector modes — 2026-09-17T05:38:12Z

Cause: snapshot connectors published only `name`/`status`, so Shell could not show why eDP is the live panel. The previous deploy failed because the kiosk autostart loop holds after 12 GUI deaths; `pkill` then cannot revive a snapshot.

Lock: `probe_display()` reads sysfs `enabled`, `dpms`, `modes`, and `connector_id`. Modes are bounded ASCII tokens. `metal-live.sh` starts the GUI on the current Wayland seat if it is still absent after the pkill nudge.

Measure on `192.168.8.10` via pinned `metal-live.sh`. No USB. No sudoers change.

- Runtime `20260917T053812Z-runtime-32723` sha256 `68f600680f6038e775c7ae115d18060f5eab43702bde48c509f5fbb9040a868e`.
- Shell `20260917T053829Z-shell-3779` assets `index-tqmGlV_c.js`.
- `drm-snap-1` matches sysfs: `card0-eDP-1` `connected` `enabled` `dpms=On` `connector_id=99` `modes=["1920x1080","1920x1080"]`. Five other connectors disconnected/disabled/`dpms=Off`/empty modes.
- `eno1` stayed `192.168.8.10/30`. `wlp58s0` stayed `EWKaska` `192.168.38.82/24`.
- Live journal remains `seccomp mode=0 filters=0`.

Verdict stays `NOT_READY`.

## 38. Live SSH USB device observe — 2026-09-17T05:40:51Z

Cause: snapshot storage listed block devices only, so Shell could not show which USB nodes were present.

Lock: `probe_usb()` lists `/sys/bus/usb/devices` device names (not `usb*` buses or `*:iface` interfaces). Vendor/product IDs are 4 hex digits. Text fields are bounded ASCII.

Measure on `192.168.8.10` via pinned `metal-live.sh`. No USB flash. No sudoers change.

- Runtime `20260917T054051Z-runtime-31170` sha256 `fb38d2bc914c474cd1f70574dc4013763971f73c1a8146166584b54b00ca980b`.
- Shell `20260917T054105Z-shell-27707` assets `index-BrgJN3PW.js`.
- `usb-snap-1` matches sysfs: `1-11` `0bda:5676` `480` `Integrated_Webcam_HD`; `1-14` `0cf3:e007` `12` (no manufacturer/product); `2-2` `413c:81d7` `5000` `Dell Inc.` `DW5821e Snapdragon X20 LTE`.
- `drm-snap-1` fields remain: `card0-eDP-1` connected/enabled/On/`1920x1080`.
- `display.action` stays `not_available`.
- `eno1` stayed `192.168.8.10/30`. `wlp58s0` stayed `EWKaska` `192.168.38.82/24`.
- Live journal remains `seccomp mode=0 filters=0`.

Verdict stays `NOT_READY`.

## 39. Live SSH services ActiveState + host/GPU — 2026-09-17T05:44:21Z

Cause: services were socket-witness booleans only, so a failed `empiieros-wifi-restore` looked the same as a running unit. Host/GPU identity was also missing from the snapshot.

Lock: allowlisted `systemctl show -p ActiveState` (cgroup fallback). Host reads `/proc/cpuinfo`, `/proc/meminfo`, `/proc/loadavg`. Display reads the `card0` driver symlink and PCI vendor/device.

Measure on `192.168.8.10` via pinned `metal-live.sh`. No USB flash. No sudoers change.

- Runtime `20260917T054421Z-runtime-26953` sha256 `ac01be9dd84c48361a9e1b436f5a9767b3d835eaf5e179c0c6097ef313069e58`.
- Shell `20260917T054435Z-shell-3233` assets `index-Bv8VeR5X.js`.
- `host-svc-1` services match `systemctl is-active`: runtime/bridge/networkd/timesyncd/seatd `active`; `empiieros-wifi-restore.service` `failed`.
- Host: `Intel(R) Core(TM) i7-10850H CPU @ 2.70GHz` ×12, `MemTotal=32445560`, `loadavg_1=0.36`. `MemAvailable` drifted 31724340 → 31721032 in the same SSH session.
- Display driver `i915` `8086:9bc4` matches sysfs.
- `eno1` stayed `192.168.8.10/30`. `wlp58s0` stayed `EWKaska` `192.168.38.82/24`.
- Live journal remains `seccomp mode=0 filters=0`. Reboot Wi-Fi restore remains failed on this image.

Verdict stays `NOT_READY`.

## 40. Live reboot Wi-Fi restore via ath10k_pci rebind — 2026-09-17T05:48:53Z

Cause: builtin `ath10k_pci` still probes firmware at ~2.5s with `-2`. The restore unit waited 15s and failed. Manual unbind/bind later loaded `WLAN.RM.4.4.1-00309`.

Lock: `empiieros-wifi-restore` rebinds allowlisted Atheros PCI (`vendor=0x168c`) through `ath10k_pci` sysfs once if no wireless netdev appears. The unit no longer sets `ProtectKernelTunables`. Persist files stay root `0600` and are never read for a PSK.

Measure on `192.168.8.10` via pinned `metal-live.sh`. No USB flash. No sudoers change.

- Helper sha256 `91714fa47f449381df9354e47548355ba3a2a2205413b4a55881a97f213c9ab2`.
- In-session unbind of `0000:3a:00.0` then `systemctl start empiieros-wifi-restore` returned `Result=success` and restored `EWKaska` `192.168.38.82/24`. `eno1` stayed `192.168.8.10/30`.
- Controlled `systemctl reboot` (not `system.power`). Host returned in 21s. Boot journal: `wireless interface missing; trying ath10k_pci rebind` then `rebind 0000:3a:00.0`. `Result=success` `ExecMainStatus=0`.
- At 15:48:54 `wlp58s0` was up but not yet associated. At 15:49:12 it was `EWKaska` `192.168.38.82/24`. `eno1` stayed `192.168.8.10/30`.
- Runtime remained `ac01be9d…`. Shell GUI was running. sudoers `440` still present.
- This is not modular ath10k on this kernel and not a USB install.

Verdict stays `NOT_READY`.

## 41. Restore waits for IPv4/SSID; services publish Result — 2026-09-17T05:55:18Z

Cause: §40 `Result=success` exited when `wlp58s0` appeared, before association. Snapshot services also treated a successful oneshot as “not running” because they only showed ActiveState.

Lock: `empiieros-wifi-restore` waits up to 25s for IPv4 plus SSID after the netdev starts and exits non-zero if the lease never appears. Persist files are still never read for a PSK. The restore unit allows `AF_UNIX AF_NETLINK` so `ip`/`iw` can observe association. Snapshot `ServiceRecord` now includes allowlisted systemd `Result` and `ran` from `ExecMainStartTimestamp`. Shell treats `inactive/success/ran` as ready.

Measure on `192.168.8.10` via pinned `metal-live.sh`. No USB flash. No sudoers change.

- Runtime `20260917T055316Z-runtime-27248` sha256 `066b739c…`. Helper `bacfcf12…`. Shell `20260917T055355Z-shell-19047` assets `index-CLfsitsQ.js`.
- Frontend `svc-result-1` reported wifi-restore `inactive` / `success` / `ran=true` while runtime/networkd/timesyncd/seatd stayed `active`.
- In-session unbind of `0000:3a:00.0` then `systemctl start empiieros-wifi-restore` returned in 21s. Journal: `rebind 0000:3a:00.0` then `associated wlp58s0`. No PSK in the restore journal.
- Immediately after unit exit: `wlp58s0` was already `EWKaska` `192.168.38.82/24`. `eno1` stayed `192.168.8.10/30`. `Result=success` `ExecMainStatus=0`.
- Frontend `svc-result-2` still showed wifi-restore `inactive` / `success` / `ran=true` and association `EWKaska`.
- This is not modular ath10k on this kernel, not live `+SECCOMP`, and not a USB install.

Verdict stays `NOT_READY`.

## 42. Ethernet speed/duplex and Wi-Fi link quality — 2026-09-17T06:01:52Z

Cause: snapshot Ethernet/Wi-Fi domains named the iface and IPv4/SSID but not the observed link. Shell cards could not show speed or signal. `iw` on this radio prints `freq: 5260.0`, so an integer-only parser would drop frequency.

Lock: Ethernet `speed`/`duplex` come from sysfs with bounded parsers (`-1` and wifi Invalid argument stay null). Wi-Fi association adds BSSID, frequency, signal, and rounded rx/tx Mbps from `iw dev link`. Frequency accepts integer or fractional MHz. No PSK is read.

Measure on `192.168.8.10` through the existing pinned ControlMaster (new `ssh-keyscan` to the host timed out from this workstation). No USB flash. No sudoers change.

- Runtime `20260917T060152Z-runtime-5813` sha256 `83f1770b…`. Shell `20260917T060122Z-shell-4950` assets `index-BiTgFJBc.js`.
- Frontend `link-truth-2` MATCH: `eno1` `1000`/`full`/`192.168.8.10/30`; `wlp58s0` `EWKaska` BSSID `52:ff:20:aa:80:99` `5260` MHz rx `780` tx `867`. Signal stayed within 2 dBm of `iw` (`-36` vs `-35`).
- `eno1` stayed `192.168.8.10/30`. sudoers `440` still present.
- This is not live `+SECCOMP` and not a USB install.

Verdict stays `NOT_READY`.

## 43. Official resume assemble includes current Runtime — 2026-09-17T06:05:48Z

Cause: official `out/` still packaged Runtime `a1cc73c0…` and lacked later helpers, so QEMU `+SECCOMP` proof was not the live source.

Lock: `scripts/package-shell-gui-rootfs.sh` overlay `2f9c572a…` then `buildroot-build.sh --resume` (no WebKit rebuild). Image Runtime matches the packaged binary. Helpers include `empiieros-wifi-restore` `bacfcf12…`. Shell-bridge socket still has `Wants=runtime` and no `After=runtime`.

Measure: `--resume` exit 0, `image_contract_ok`, `DONE images`. No USB.

- Overlay Runtime / image Runtime sha256 `32605369…` (current packaged tree). Live 192.168.8.10 still runs the earlier mux deploy `83f1770b…` from §42.
- `out/rootfs.ext2` sha256 `1d50ad39…` (16:04). `out/bzImage` unchanged `04fab7e1…`. Stamp `rootfs_sha256=4631028d…`.
- `qemu-empiieros-smoke.sh` exit 0: `seccomp mode=2 filters=28`, systemd `+SECCOMP`, `WebKit sandbox enabled`, authorized snapshots, no `Ordering cycle`.
- `ELITEEW_TS_SECURE_RESET_QEMU=1` selftest exit 0 on the same `out/`: installer→installed-disk PASS, `mode=2 filters=28` on both boots.
- This is image proof, not live `+SECCOMP` and not a USB install.

Verdict stays `NOT_READY`.

## 44. Live sshd drop-in was ignored until Include — 2026-09-17T06:13:40Z

Cause: the lab host rebooted (~16:08). The ControlMaster died. New TCP to 192.168.8.10 came back, but `/etc/ssh/sshd_config` had no `Include`, so `10-empiieros-security.conf` was inert. `sshd -T` still offered `passwordauthentication yes` / `permitrootlogin without-password`. Snapshot identity read the drop-in file and reported password off anyway. `metal-live.sh check` only mux-checked and failed closed when the socket was gone.

Lock: append `Include /etc/ssh/sshd_config.d/*.conf` to the live `sshd_config` (do not replace `AllowUsers eliteew-ts`). Reload sshd. Identity probe prefers `sshd -T`. `metal-live.sh check` now calls `ensure_connection`. Next-image post-build already writes a stub `sshd_config` with Include.

Measure on `192.168.8.10`. No USB. No sudoers change.

- After reload: `sshd -T` `passwordauthentication no` `permitrootlogin no` `allowusers eliteew-ts` `authenticationmethods publickey`.
- Password-only BatchMode: `Permission denied (publickey)`. Fresh pubkey and the mux still work.
- Frontend `ssh-id-1` source `linux-passwd+sshd-T`: password/root false, `AllowUsers=eliteew-ts`, sudoers still present.
- Runtime `20260917T061340Z-runtime-22555` sha256 `971d2324…`. Host key still `SHA256:msPNx+ysC+7DZLmBWoDsa2SQpUS4p0QxBsTgGX8TTuU`.
- Key-only `empiieros-maint` is still absent (`authorized_keys` missing). This is not live `+SECCOMP`.

Verdict stays `NOT_READY`.

## 45. Live SSH after Include plus root usage — 2026-09-17T06:17:00Z

Cause: the operator asked to inspect SSH after the 16:08 reboot. Include was added at 16:13 on the already-booted seat, so the reboot itself did not prove Include persistence. Snapshot storage listed block devices only, so the compact disk card could not show used/free against `df`. `systemd-networkd-wait-online.service` was observed on the host but missing from the services allowlist.

Lock: keep `Include /etc/ssh/sshd_config.d/*.conf` and `AllowUsers eliteew-ts` on this lab. Identity continues to prefer `sshd -T`. Snapshot storage adds root `statvfs` totals. Services allowlist includes `systemd-networkd-wait-online.service`. Shell compact storage card shows used/total and free space.

Measure on `192.168.8.10`. No USB. No sudoers change. Include was **not** reboot-proven in this slice.

- sshd `active`, listen `:22`. Drop-in `10-empiieros-security.conf` is loaded. `sshd -T`: `passwordauthentication no`, `permitrootlogin no`, `pubkeyauthentication yes`, `allowusers eliteew-ts`.
- Password-only BatchMode for `eliteew-ts`, `root`, and `empiieros-maint`: `Permission denied (publickey)`.
- `eliteew-ts` has one `ssh-ed25519` authorized key. `empiieros-maint` still has no `.ssh`. Temporary sudoers remains.
- Frontend `stor-snap-1` MATCH vs `df -B1 /`: `root_total_bytes=250375106560`, `root_used_bytes=991756288`, `root_available_bytes=236590460928`, `root_fstype=ext4`, `root_read_only=false`.
- Frontend wait-online MATCH: `active`/`success`/`ran=true` vs `systemctl show`. wifi-restore still `inactive`/`success`/`ran`.
- Identity still `linux-passwd+sshd-T`: password/root false, `AllowUsers=eliteew-ts`, sudoers present.
- Runtime `20260917T061643Z-runtime-10999` sha256 `e743600732332a8012e1a21212ccdb79fb3b211e60de0d0b120aa95226a66fb2`. Shell `20260917T061700Z-shell-4592` assets `index-z7HVixhD.js`.
- Input now also shows USB `YSPRINGTECH USB OPTICAL MOUSE` `event9` `has_rel_wheel=true` (sysfs `rel=903`). GTK journal was empty before the inject.
- Password-only BatchMode exits `255` with `Permission denied (publickey)` for `eliteew-ts`, `root`, and `empiieros-maint`.
- Live journal remains `seccomp mode=0`. Host key still `SHA256:msPNx+ysC+7DZLmBWoDsa2SQpUS4p0QxBsTgGX8TTuU`.

Verdict stays `NOT_READY`.

## 46. USB wheel reached GTK scroll — 2026-09-17T06:19:42Z

Cause: earlier wheel proof was BLOCKED because the only pointer was PS/2 `REL_X/REL_Y`. A USB `YSPRINGTECH` mouse is now on `event9` with `REL_WHEEL`.

Lock: keep snapshot `has_rel_wheel` / `rel_wheel_observed`. GTK `on_scroll_event` still logs `shell scroll observed` and calls `window.scrollBy`.

Measure on `192.168.8.10`. Inject `REL_WHEEL` on `/dev/input/event9` only (no uinput). No USB flash. No sudoers change.

- Journal `empiieros-shell-gui[5837]`: `shell scroll observed dx=0,000 dy=1,500` and `dy=-1,500` at 16:19:42.
- `uinput` still absent. Glass still unwatched. This is not live `+SECCOMP`.

Verdict stays `NOT_READY`.

## 47. Live SSH identity methods and keys — 2026-09-17T06:26:45Z

Cause: snapshot identity reported `empiieros-maint` present and password off, but not `sshd -T` methods or whether authorized_keys files exist. `ProtectHome=true` made `/home` an inaccessible dir, so a naive stat lied `kiosk_authorized_keys_present=false`. `ProtectHome=tmpfs` plus `BindReadOnlyPaths` to the missing maint keys path then failed the unit at `NAMESPACE` (226). Harness rolled back; Runtime returned.

Lock: parse `pubkeyauthentication`, `kbdinteractiveauthentication`, and allowlisted `authenticationmethods` from `sshd -T`. Stat authorized_keys presence only (no key bytes). `ProtectHome=tmpfs` punches only files that exist (`/home/eliteew-ts/.ssh/authorized_keys`). A missing bind source is forbidden. Services allowlist includes `sshd.service`.

Measure on `192.168.8.10`. No USB. No sudoers change.

- Failed deploy rolled back after `authorized Shell snapshot was not observed`. Journal: `Failed to set up mount namespacing: /home/empiieros-maint/.ssh/authorized_keys: No such file or directory`.
- Recovered Runtime `20260917T062645Z-runtime-13017` sha256 `6ed6b0a4ed8f1ad45e0a11d060ffd07cc4d2db15b8ab1913487bd1cad7876d48`. Shell `20260917T062248Z-shell-25756` assets `index-DEAuA745.js`.
- Frontend `id-snap-2` MATCH: `authentication_methods=publickey`, pubkey yes, password/kbd/root no, `AllowUsers=eliteew-ts`, kiosk keys present, maint keys absent, sudoers present. Snapshot JSON has no key material.
- `sshd.service` `active`/`success`/`ran`. `eno1` stayed `192.168.8.10/30`. Live journal still `seccomp mode=0`.

Verdict stays `NOT_READY`.

## 48. Live SSH audio observe percent — 2026-09-17T06:29:58Z

Cause: snapshot audio listed ALSA cards only, so the sound card could not show the current playback level. Mutation already returned `observed_percent`, but the next snapshot forgot it.

Lock: `empiieros-audio-volume --observe` reads allowlisted selems and writes nothing. Runtime `probe_audio()` parses `name=` / `observed_percent=` and rejects `Capture` or `>100`. Shell compact sound card shows the percent.

Measure on `192.168.8.10`. No USB. No sudoers change.

- Helper sha256 `37997f516ec9d95e5cd16fdf290e5984d7ae3006801d57f659cfaa378ef29daa`. Runtime `20260917T062954Z-runtime-13981` sha256 `d19a1d104d08513a285027ef8e00c49915c802a00d7a228c7f5773933f32cfd0`. Shell `20260917T062958Z-shell-112` assets `index-WMtr0NoQ.js`.
- Host `--observe` twice around a codec dump: Amp-Out unchanged. Frontend `aud-obs-1` MATCH `Speaker` `100` source `linux-proc+alsa-mixer`.
- `aud-25c` applied 25% (`observed_percent=24`); `aud-obs-2` MATCH `24`; `aud-100c` / `aud-obs-3` restored `100`.
- `eno1` stayed `192.168.8.10/30`. This is not live `+SECCOMP`.

Verdict stays `NOT_READY`.

## 49. Live SSH NTP last-sync observe — 2026-09-17T06:32:37Z

Cause: locale snapshot only said NTP yes/no and the server, so Shell could not show when the clock last synced or the stratum.

Lock: last sync is the mtime of `/run/systemd/timesync/synchronized` (regular file only). One `timedatectl show-timesync` parse supplies server, `Stratum` 1–16, and bounded jitter. Timezone/locale writes stay fail-closed.

Measure on `192.168.8.10`. No USB. No sudoers change.

- Runtime `20260917T063223Z-runtime-30511` sha256 `3b8237d2ef9588eb7feb27bd7c54a4e8fbaaaf6a66628fb622085c885d6a5f1e`. Shell `20260917T063237Z-shell-30207` assets `index-1FZim13b.js`.
- Frontend `ntp-obs-1` MATCH stamp `1789625790`, server `162.159.200.1`, stratum `3`, jitter `12` ms vs `Jitter=11.945ms`. Timezone `Asia/Vladivostok`, `LANG=ru_RU.UTF-8`.
- `time-sync-2` applied. `ntp-obs-2` last sync advanced to `1789626773`. Timezone/LANG unchanged. `eno1` stayed `192.168.8.10/30`.
- After sync, jitter was `null` (NTPMessage had no usable Jitter token). That is observed absence, not a green default.

Verdict stays `NOT_READY`.

## 50. Live SSH battery pack observe — 2026-09-17T06:35:20Z

Cause: the power snapshot only published capacity and status, so Shell could not show charge, voltage, current, or the pack identity.

Lock: `probe_power_supplies()` reads bounded sysfs integers (`charge_*`, `voltage_now`, `current_now`) and sanitized manufacturer/model/technology. No computed health percent.

Measure on `192.168.8.10`. No USB. No sudoers change.

- Runtime `20260917T063505Z-runtime-25019` sha256 `3818a482b876af412563ef4b6cf9a0da100f69450f3520c7af4384d3d932e904`. Shell `20260917T063520Z-shell-17533` assets `index-CKS56ATC.js`.
- Frontend `bat-obs-1` MATCH `/sys/class/power_supply/BAT0`: `charge_now=2381000`, `charge_full=2381000`, `charge_full_design=4474000`, `voltage_now=15884000`, `current_now=1000`, `BYD` / `DELL 3PCVM0A` / `Li-poly`, capacity `100` `full`.
- `eno1` stayed `192.168.8.10/30`. This is not live `+SECCOMP`.

Verdict stays `NOT_READY`.

## 51. Live SSH inspect + IPv6 observe — 2026-09-17T06:39:00Z

Cause: after the 16:08 lab reboot the operator asked to look at SSH. The same seat's network snapshot published IPv4 only, so link-local addresses on `eno1` and `wlp58s0` were invisible to Shell.

Lock: `probe_network()` now runs `ip -j -6 address show` and publishes sanitized `ipv6_addresses`. `classify_network` still requires routed IPv4. Live `sshd_config` already has `Include /etc/ssh/sshd_config.d/*.conf`; the drop-in keeps `AllowUsers eliteew-ts` and publickey-only.

Measure on `192.168.8.10`. No USB. No sudoers change. Mux pid `4066109`. Host key `SHA256:msPNx+ysC+7DZLmBWoDsa2SQpUS4p0QxBsTgGX8TTuU`.

- Boot id `1b034346-b71a-4a6a-a8a9-34e6309acf19`, uptime ~29 min after 16:08 reboot. `sshd.service` `active`/`success` `NRestarts=0`. Effective `sshd -T`: `allowusers eliteew-ts`, `authenticationmethods publickey`, password/kbd/root no. Password BatchMode exit `255` `Permission denied (publickey)` for kiosk/root/maint. `empiieros-wifi-restore` remains `inactive`/`success` from 16:08:10. Include is still on this booted seat and not reboot-proven.
- Runtime `20260917T063847Z-runtime-2204` sha256 `960314a95672c822fbddb24e30dfb9e198180a133cc1c2e8bf440a4c56e1cc50`. Shell `20260917T063900Z-shell-13692` assets `index-ItFBB6R7.js`.
- Frontend `ipv6-obs-1` MATCH `ip -6`: `eno1` `fe80::ce48:3aff:fe73:ec75/64`, `wlp58s0` `fe80::5eba:efff:feeb:8a1/64`. `eno1` IPv4 stayed `192.168.8.10/30`.
- This is not live `+SECCOMP`.

Verdict stays `NOT_READY`.

## 52. Live SSH DNS source observe — 2026-09-17T06:44:39Z

Cause: `system.snapshot` followed `/etc/resolv.conf` and listed nameservers without saying the file is a systemd-resolved uplink symlink, so Shell could not tell stub `127.0.0.53` from uplink servers.

Lock: `classify_dns_source()` maps an allowlisted resolv symlink to `systemd-resolved-uplink` / `systemd-resolved-stub` / `static-resolv-conf`. Nameservers stay sanitized. Services now include `systemd-resolved.service` with witness `/run/systemd/resolve/resolv.conf`.

Measure on `192.168.8.10`. No USB. No sudoers change.

- Runtime `20260917T064247Z-runtime-24875` sha256 `ef1885cc7f2a07bee287fc88b5f4099114a7350100fbc575cda43fcad0aaaa3a`. Shell `20260917T064439Z-shell-28237` assets `index-DhBwvXn8.js`.
- Frontend `dns-src-1` MATCH: `dns_source=systemd-resolved-uplink`, nameservers `192.168.8.9` `1.1.1.1` `8.8.8.8` equal `readlink` `../run/systemd/resolve/resolv.conf` plus that file. `systemd-resolved.service` `active`/`success`/`ran`. `eno1` stayed `192.168.8.10/30`.
- This is not live `+SECCOMP`.

Verdict stays `NOT_READY`.

## 53. Live SSH host swap + loadavg — 2026-09-17T06:50:43Z

Cause: host snapshot published only `loadavg_1` and RAM, so Shell could not show that this seat has no swap or the 5/15 load averages.

Lock: `HostStatus` now reads `SwapTotal`/`SwapFree` from `/proc/meminfo` and load fields 0/1/2 from `/proc/loadavg`. Zero swap is published as `0`, not hidden.

Measure on `192.168.8.10`. No USB. No sudoers change.

- Runtime `20260917T065021Z-runtime-28708` sha256 `2c88dc433919dbf67a87ea3f0a3e370048b700b72f4d635168079815f430d62b`. Shell `20260917T065043Z-shell-1864` assets `index-D58n0fTB.js`.
- Frontend `host-proc-1` MATCH `/proc`: `MemTotal=32445556`, `SwapTotal=0`, `SwapFree=0`, load `0.10` `0.08` `0.04`. `MemAvailable` drifted 31722356 → 31720004 in the same SSH session. `eno1` stayed `192.168.8.10/30`.
- This is not live `+SECCOMP`.

Verdict stays `NOT_READY`.

## 54. Live SSH service NRestarts — 2026-09-17T06:52:41Z

Cause: service snapshot published state/result/ran but not `NRestarts`, so Shell could not show a restart storm even though `systemctl show` already has the counter.

Lock: `systemd_unit_status()` now reads bounded `NRestarts`. Missing property (sockets) stays `null`; values above 10000 are dropped.

Measure on `192.168.8.10`. No USB. No sudoers change.

- Runtime `20260917T065224Z-runtime-25816` sha256 `5fd55025fe3c34581de3026719b41f6b9808d6280a78af00fcf8811daadb55f1`. Shell `20260917T065241Z-shell-13014` assets `index-B2GONB8L.js`.
- Frontend `svc-nr-1` MATCH `systemctl show -p NRestarts`: allowlisted services `0`; `empiieros-shell-bridge.socket` `null`. `eno1` stayed `192.168.8.10/30`.
- This is not live `+SECCOMP`.

Verdict stays `NOT_READY`.

## 55. Live SSH Dell ALC3204 speaker-amp GPIO — 2026-09-18T09:47:00Z

Cause: tray volume and HUD clicks wrote ALSA / played PCM, but the analog amp stayed off. `snd_hda_intel` is from-source and built-in; `patch_realtek.c` has no `0x09c1` quirk.

Lock: `empiieros-audio-volume` enables AFG GPIO `0x07` through `/dev/snd/hwC0D0` only when sysfs vendor is `0x10ec0236` and subsystem is `0x102809c1`. Mixer observe still does not change selem values. Boot oneshot `empiieros-audio-amp.service` re-applies after `sound.target`.

Measure on `192.168.8.10`. No USB. No official `out/` remux.

- GPIO dump before the probe: `IO[0..2] enable=0`. After `0x07` verbs: all three `enable=1 data=1`. Operator heard 1 s 440 Hz on `hw:0,0`.
- Helper mux sha256 `638d0046daac2c98a023df005be21813dc30c20758a086e37775580816e4b96e`. After an explicit GPIO zero, `--observe` printed `speaker amp gpio enabled` and restored `enable=1`. `empiieros-audio-amp.service` is `enabled`/`active`. Runtime `c6d3f8d2…` and GUI `040a096a…` stayed.

Verdict stays `NOT_READY`.
