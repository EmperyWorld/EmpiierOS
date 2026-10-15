# EmpiierOS — Seat face (living glass map)

**Status:** Living contract. Verified against source on 2026-09-21.
**Verdict:** `NOT_READY`. Official bake 2026-09-20 (`5979e9cf…`). QEMU smoke FAIL (WebKit string) not rerun. Not metal MATCH.
**Plane:** native kiosk + Shell + Runtime. **Not:** Tauri. **Not:** HTTP `/v0/*`.

This file is the operator-facing map of what the seat shows. Production
authority for ops remains [`../runtime/HEALTH-IPC.md`](../runtime/HEALTH-IPC.md).
Display gating is not a second FSM.

## What is on the glass

Production binary is `empiieros-shell-gui` from
`runtime/shell-gui/secure_main.c` (packager does **not** compile
`main.c`). The OS face URI is `empiier://shell/index.html`. The leftover
`http://127.0.0.1:17420/` loopback in `runtime/src/main.rs` /
`runtime/src/shell_gui.rs` is not the kiosk path: `Cargo.toml` links
`src/daemon.rs` + `src/core.rs` only.

```text
boot splash (start_os.mp4, optional)
        │
        ▼
always-on HUD (right edge)  +  React host (Secure Shell + desktop)
        │
        ├── Launcher window  → Unlock → Activate → Portal → Workspace → Sign-in → Open
        ├── FoFline pack     → fofline-spa://localhost/index.html?eliteewFoflineClient=1
        ├── Seat Browser     → dedicated WebKit overlay + native chrome
        ├── Seat Shield      → host workspace (observe-only default; armed auto-act optional)
        └── Auth card        → shell.auth_surface on the same GtkWindow
```

## Native surfaces

| Surface | Code | Trigger |
|---------|------|---------|
| Boot splash | `boot_splash.c` | GUI start. `playbin` → `gtksink` (fallback `gtkwaylandsink`) + `alsasink` volume **0.25**. Missing file skips. HUD hidden until EOS/error/20s, then `empiier_product_chrome_show`. |
| HUD | `product_chrome.c` | Always-on overlay. Default right-center, then movable (grip). Position `{state}/hud-chrome-pos.json` when writable. Peek stays forbidden. Six icons. Updates carries a caption. Flyouts flip left/right with X. `hide_product` does not hide it. |
| Product WebView | `native_bridge.c` + `secure_main.c` | `shell.product_surface` after Runtime `product.open`. Scheme `fofline-spa`. Media stream/WebRTC on (iframe getUserMedia); `empiier://` still deny-closed. Wheel scrolls the nearest overflow ancestor, not `window.scrollBy`. |
| Auth card | `auth_surface.c` / `auth_bounds.c` | `shell.auth_surface` after `launcher.oidc_start`. FILL dimmer + ~480×640 card. Not a second top-level. |
| Seat Browser | `seat_browser_surface.c` | `shell.seat_browser_surface`. Dedicated WebKit + overlay chrome. |
| Meet clip | `meet_surface.c` / `meet_bounds.c` | Pack `openMeetStage(url, bounds)`. Sibling clip. No FILL. |

F9 / Escape return from FoFline, auth, and Seat Browser. Alt+F4 is swallowed
(`delete-event` TRUE; labwc `A-F4` → `true`).

## HUD icons (top → bottom)

Icons and WAVs live under `/opt/empiieros/shell/product-chrome` then
`/usr/share/empiieros/product-chrome`. SoT:
`runtime/shell-gui/product-chrome/` (ComfyUI, see ORIGIN.md).
2026-09-21 tiles are gold glyphs with a thick outline and cyan rim.

| Icon | File | Click | Open button |
|------|------|-------|-------------|
| Launcher | `launcher.png` | flyout `kind=launcher` + `click-launcher.wav` | Hide product (if open) + `empiieros-host-workspace` `{workspace:eliteew, activator:true}` |
| EliteEW | `eliteew.png` | flyout `kind=eliteew` + `click-eliteew.wav` | `product.open` then `show_product`. If already open, focus the product view. |
| Panel | `hud-panel.png` | flyout `kind=panel` | none. Mic/cam lists are observe-only. Switches log preference (`no hardware kill-switch`). GPS stays off (`gnss_not_on_seat`). |
| Shield | `hud-shield.png` (alert tile `hud-shield-alert.png`) | flyout `kind=shield` | `empiieros-host-workspace` `{workspace:shield}`. Status drains `watch.jsonl` then `maybe_auto_observe`. Watch helper not on glass. |
| Updates | `hud-ota.png` + caption «Обновления» | flyout `kind=ota` | `empiieros-host-workspace` `{workspace:ota}`. Also a button on the seat-panel flyout and a licensed tray icon. Capsule OTA, not OS A/B. |
| Volume | `hud-volume.png` | flyout `kind=volume` | none. Observe `audio.control`. Host slider is the Shell tile, not this flyout. |

Click spawn is `aplay -q` first, then `paplay` / `pw-play`. `audio.action`
stays production-disabled. Icon click never jumps into an app.

## Shell host

`shell-ui/src/secure/SecureApp.tsx` + `shell-ui/src/desktop/`.

1. **Desktop canvas.** Wallpaper + movable OS icons. Without a FoFline
   license the launcher window (activation path) and the network tile
   stay available. Pack-kind tiles stay out of the canvas. After
   consume, the launcher tile opens a glass window and does not
   auto-open the product.
2. **Journey rail.** `LAUNCHER_JOURNEY` in `foflineLauncherStage.ts`:
   `pin > consume > portal > spa > login > ready`. Display marks only.
3. **Primary floor.** `FoflineActivator` shows the current stage CTA.
   Activate offers **air or file** as the same step (two buttons, one
   stage). Sign-out (`launcher.oidc_logout`) appears whenever
   `oidc_session=present`. Lab consume/sync/status stay in
   `FoflineActivatorLab`.
4. **Host workspaces.** Native HUD or desktop tiles dispatch
   `empiieros-host-workspace`. `browser` (key 6) mounts `SeatBrowserCard`
   and shows allow hosts as a vertical list.
   `shield` mounts Seat Shield. `ota` mounts Updates. Host workspaces
   are floating panels — drag the title bar. Title lives on that bar
   only. Body chrome is shared (`HostAppShell`): real tabs when there
   are two or more, chips, value tiles, then the pane. Dummy single
   tabs and second inner titles stay out. EliteEW with
   `activator:true` focuses the launcher window. Licensed tray on the
   system taskbar also opens Updates.

Copy lives in `shell-ui/src/secure/locale/`. Primary copy must not claim
Tauri, a pack picker, a second window, or product-ready.

## Runtime ops that drive the face

Registered in `runtime/src/core.rs` (Unix NDJSON). Historical HTTP
`GET /v0/*` rows in [`CONTRACT.md`](CONTRACT.md) are archive.

| Operator step | Runtime | Native / Shell |
|---------------|---------|----------------|
| Unlock | `launcher.pin_unlock` | PIN pad. No `launcher.pin_issue`. |
| Activate | `launcher.air_activate` or `launcher.capsule_import` | after unlock |
| Portal | `launcher.channel_up` (one-shot auto only on stage `portal`) | `wg-portal` + host-map |
| Workspace | `launcher.spa_sync` | signed pack on VIP |
| Sign-in | `launcher.oidc_start` → `authorization_url` + `request_uri` | `shell.auth_surface` |
| Open | `product.open` `{target:fofline_local}` | exact
  `fofline-spa://localhost/index.html?eliteewFoflineClient=1` then
  `shell.product_surface` |
| Browser | `seat.browser.*` | overlay chrome may also send
  `shell.seat_browser_surface` `{action:reload\|back\|forward\|find\|zoom\|go\|stop}` |
| Volume | `audio.control` `{action:set_volume, percent:10-100}` | HUD observe; rail slider writes |

`system.snapshot` capability rows list launcher / `product.open` /
`audio.control` and family id `seat.browser` (`available`, overlay host).
They do **not** yet advertise `seat.shield.*`. Those shield ops still
dispatch from `core.rs`. `seat.browser` on the snapshot is not MATCH.

## FoFline audio (not host click)

Host click WAV and ALSA volume are not MusicPlayer.

1. Pack `1789837366816-w7k51h8btf` (`contentApi-Bstp_TrW.js`) is supposed
   to play on `foflineNative` via `decodeAudioData` +
   `AudioBufferSourceNode`. Glass Play after re-open is the remaining
   measure.
2. Native leftovers (not acceptance): `fofline_api_proxy.c` strips
   `Range`, serves XHR bodies as `application/octet-stream`, sniffs
   media-Range to an audio MIME; `secure_main.c` injects
   `__empiierosAudioBlob` (typed `blob:` + `load()`). WebKit still
   cancels `blob:fofline-spa:`. The pack must work without that inject.

No Pulse/PipeWire. Analog PCM is exclusive `hw:0,0`.

## Honesty

- Official image `out/` SoT `5979e9cf…` (Runtime `79e06a7d…`, GUI
  `f2fe1900…`, Shell `index-DMc2IDZV.js`). Prior host bake
  `4104a67b…` / USB mux `d250a976…` are superseded. Live USB now
  carries mux SoT `f56dba1b…` plus apply plant `8a9ec933…`. QEMU
  smoke FAIL stands (`WebKit sandbox enabled`) and was not rerun.
  Lab `192.168.8.10/30` is the polygon, not a Settings default.
- Host navigation MATCH on Seat Browser chrome is the operator's walk.
  FoFline pack `1789890833725-7y746j4lvw` calls
  `EmperyDesktopSeatBrowser`. OS GUI `e18051fa…` plants the hook.
  stack0 `1789891909119-45tkayntro` serves the catalog tile. Glass
  18:27 planted the hook; 18:28 `kind=open` + `seat-browser-host-nav`
  applied. Not MATCH.
- FoFline Play MATCH is the operator's walk after hide/re-open.
- OS A/B OTA is absent (P93 frozen). Production `update.*` is disabled.
  Bootstrap pack is `launcher.spa_sync` (pre-OIDC). Capsule-gated OTA
  is `launcher.ota_*` after license + hardware + live EliteEW login
  ([`OTA-CAPSULE.md`](OTA-CAPSULE.md)). Glass + HUD expose Updates;
  official `out/` still lags the living overlay.
- Seat Browser public clearnet stays debt. `media.set`, ≤3 tabs,
  seat-local downloads, and `system.snapshot` family `seat.browser` are
  wired (not MATCH). No Content Manager.

## Related

- Journey: [`SEAT-LAUNCHER.md`](SEAT-LAUNCHER.md)
- Kiosk / HUD / splash: [`SHELL-GUI.md`](SHELL-GUI.md)
- Open / pack origin: [`PRODUCT-OPEN.md`](PRODUCT-OPEN.md)
- Bootstrap pack / content hop: [`SPA-PACK.md`](SPA-PACK.md)
- Capsule OTA: [`OTA-CAPSULE.md`](OTA-CAPSULE.md)
- Browser: [`SEAT-BROWSER.md`](SEAT-BROWSER.md)
- Audio: [`AUDIO-HARDWARE.md`](AUDIO-HARDWARE.md)
- IPC table: [`../runtime/HEALTH-IPC.md`](../runtime/HEALTH-IPC.md)
