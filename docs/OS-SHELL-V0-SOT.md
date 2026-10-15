# EliteEW OS Shell v0 — SoT (text)

**Status:** Living thin planning SoT. **Not** product Done. **Not** metal-ready.
**Home:** this repo — Shell displays; production Runtime enforces a default-deny Unix `SO_PEERCRED` registry at `/run/empiieros/runtime.sock` (Shell frontend `/run/empiieros-shell/runtime.sock`). Loopback HTTP / `X-EmpiierOS-Caller` is retired.
**Kinship:** Desktop Kernel chrome DNA seeded thinly (tokens/hero); Tauri = living host, not plan axis.
**Unlock:** [`foundation/PRODUCT-UNLOCK.md`](foundation/PRODUCT-UNLOCK.md) · Seed: [`seed/SEED-POLICY.md`](seed/SEED-POLICY.md).
**Production ops:** [`../runtime/HEALTH-IPC.md`](../runtime/HEALTH-IPC.md).

## What OS Shell v0 is

React as the **only session face** of EliteEW OS: one visual world for the seat (Start / system surfaces / app surfaces / recovery).
Role like a DE shell — **only** for EliteEW journeys, not a general-purpose GNOME/KDE clone.

## In scope (v0 planning)

| Surface class | Intent |
|---------------|--------|
| System status | Honest OS states (net, updates, identity, health) |
| Shell chrome | Thin layout / navigation — no browser chrome assumptions |
| App surfaces | Kernel registry / manifests — not arbitrary Linux desktop apps |
| Offline / permission / recovery | Product journeys, not sysadmin DE |
| Bridge stubs | Typed stubs toward EmpiierOS Runtime (not Tauri SoT) |

## Out of scope (v0)

- Peripheral spoolers, receipt printers, and generic retail hardware diversions
- Wholesale monorepo FE / `connection-agent` clone
- Wholesale license / Capsule / air UI that bypasses Runtime IPC
- Enabling remote `product.open` or a URL constructed in JavaScript
- Wayland compositor / WM replacement (thin WM stays on L-session floor)
- Cloud / mobile / Keycloak doors
- Admin-OS SKU
- Flipping Substrate kiosk default off without dual-flag OS profile ready ([`foundation/SUBSTRATE-KINSHIP.md`](foundation/SUBSTRATE-KINSHIP.md))

## Twin risks (must hold)

1. **Skin** — UI without bridge op + substrate policy.
2. **GNOME-bloat** — general DE; lose EliteEW focus.

## Maturity for any shell capability

```text
shell surface  +  bridge op  +  substrate policy
```

## Seed posture

Allow/deny: [`../seed/SEED-POLICY.md`](../seed/SEED-POLICY.md).
Conscious duplication until Shell contracts stabilize. ORIGIN required on any real copy.

## Planes pointer

| Plane | Path |
|-------|------|
| L-shell | `shell-ui/` |
| L-bridge | `runtime/` |
| L-session | `session/` |

Axis: [`NORTH-STAR.md`](NORTH-STAR.md) · Map: [`WHERE-OS-LIVES.md`](WHERE-OS-LIVES.md) · Security: [`foundation/SECURITY-PRIVACY-DEFAULTS.md`](foundation/SECURITY-PRIVACY-DEFAULTS.md) · Floor: [`foundation/SUBSTRATE-KINSHIP.md`](foundation/SUBSTRATE-KINSHIP.md).

## Honesty

`shell-ui/` holds the current OS face: Bootstrap FSM (`Identifying` → `System` / `BootError`), chrome layout (`--os-*` tokens), compact domain cards, and `DetailsSpoiler`. The production face is the **seat launcher host** (`src/desktop/`) plus Secure Shell workspaces — not a second EliteEW catalog. `LAUNCHER_JOURNEY` is Unlock → Activate → Portal → Workspace → Sign-in → Open. Pack-kind tiles stay out of the desktop canvas. The activator starts native EliteEW login (`launcher.oidc_start` + `shell.auth_surface`) before `product.open`. It may pass through a Runtime-built `fofline-spa://localhost` URL via `shell.product_surface` after a session exists. It does not construct URLs and does not auto-open. Native chrome is an always-on right-edge HUD (`product_chrome.c`), not a single Launcher button. Seat Browser is a dedicated overlay (`browser` workspace / key 6). Glass map: [`../session/SEAT-FACE.md`](../session/SEAT-FACE.md). It is not the 2M-line product and not air-gap. System, Network, and Diagnostics remain Runtime-backed floor surfaces. Network cards show Ethernet speed/duplex, IPv6 addresses in DetailsSpoiler, DNS source (systemd-resolved uplink/stub vs static resolv.conf), and Wi-Fi BSSID/frequency/signal/bitrate from the snapshot. Network saved-network restore uses `network.wifi.reconnect` (no PSK in the request). WLAN radio soft-block uses `network.wifi.radio`. Storage compact card shows root used/total and free space from snapshot `statvfs`; DetailsSpoiler lists filesystem type plus observed block devices and USB device nodes. Runtime start count is shown under host details. The sound card shows the observed ALSA playback percent. The clock card shows timezone, NTP yes/no, last-sync age, stratum, and jitter from `timedatectl show-timesync` plus the timesync stamp. Clock sync uses `time.sync`. Power DetailsSpoiler lists observed supplies, battery charge/voltage/current/model from sysfs, and thermal zones. Display DetailsSpoiler lists DRM connector enabled/dpms/modes and the GPU driver from sysfs. Host details show CPU, memory, swap (`0` when absent), and load 1/5/15. Services list systemd ActiveState plus Result/`ran`/NRestarts, including `systemd-networkd-wait-online.service`; a successful oneshot restore is ready when inactive/success/ran. Input DetailsSpoiler shows whether a scroll wheel or uinput was observed. Protection details show kiosk/maint identity, effective SSH methods, whether authorized_keys files are present, and whether a hardware watchdog was observed. Historical entitlement/Capsule/Device Keys CTAs in this paragraph are planning DNA, not production claims. Production Runtime is default-deny Unix IPC; disabled prefixes return `not_available` / `capability_not_available`. This file remains **planning SoT** — not product Done and not `PASSED`.
