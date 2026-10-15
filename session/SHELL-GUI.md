# Session Contract — EmpiierOS Native Wayland Kiosk (`empiieros-shell-gui`)

**Status:** Living contract. Verified against `scripts/package-shell-gui-rootfs.sh`
and `runtime/shell-gui/secure_main.c` on 2026-09-20.
**Glass map:** [`SEAT-FACE.md`](SEAT-FACE.md).
**Not:** Tauri. **Not:** HTTP `/v0/substrate/shell-gui/*`.
**Verdict:** `NOT_READY`.

---

## 1. Principles & Planes

1. **OS face.** React Secure Shell + desktop host run inside a locked
   Wayland kiosk (`labwc`). The production ELF is
   `/usr/local/bin/empiieros-shell-gui`.
2. **Production sources.** Packager compiles `secure_main.c`,
   `native_bridge.c`, `boot_splash.c`, `product_chrome.c`,
   `auth_surface.c`, `auth_bounds.c`, `seat_browser_surface.c`,
   `meet_surface.c`, `meet_bounds.c`, `fofline_api_proxy.c`.
   `runtime/shell-gui/main.c` (loopback `http://127.0.0.1:17420/`) is
   leftover and is **not** linked.
3. **Custom-scheme assets.** The kiosk loads `empiier://shell/index.html`
   from `/opt/empiieros/shell/`. Product pack is `fofline-spa://localhost/…`
   from `/var/lib/empiieros-runtime/fofline-spa/current`. JavaScript never
   talks to Runtime; `native_bridge.c` owns the Unix frontend socket.
4. **No HTTP caller ACL.** `ShellMain` / `HostedWeb` / `X-EmpiierOS-Caller`
   are retired. Peer identity is `SO_PEERCRED` on the Runtime daemon
   (`runtime/src/daemon.rs`). Historical `runtime/src/main.rs` +
   `shell_gui.rs` HTTP 17420 helpers are not in `Cargo.toml`.
5. **Fail-closed window.** Fullscreen, undecorated, Alt+F4 swallowed.
   See §4.

---

## 2. Native bridge ops (not Runtime)

These never enter `runtime/src/core.rs`. The Shell JS asks the GUI:

| Op | Actions |
|----|---------|
| `shell.product_surface` | `show` (exact baked `fofline-spa` string) / `hide` |
| `shell.auth_surface` | `show` (allowlisted `https://auth.eliteew.ru/…`) / `hide` |
| `shell.seat_browser_surface` | `show` / `hide` / `reload` / `back` / `forward` / `stop` / `find` / `find_next` / `find_prev` / `zoom` / `go` |
| `shell.window_mode` | Kiosk fullscreen (default) or windowed when service gate is planted |

Runtime ops that those surfaces consume: `product.open`,
`launcher.oidc_*`, `seat.browser.*`, `seat.shield.*`, `audio.control`.
See [`../runtime/HEALTH-IPC.md`](../runtime/HEALTH-IPC.md).

---

## 3. Storage & Permissions

- Binary: `/usr/local/bin/empiieros-shell-gui` (`0755`)
- Shell assets: `/opt/empiieros/shell/` (`0755` dirs, files `0644`,
  **uid 0 only**). Non-root files fail closed as
  `Shell asset is not trusted` and paint «Ошибка загрузки EmpiierOS».
  Apply must `chown -R 0:0` after planting from USB share (see
  [`SEAT-FOFLINE-CANONS.md`](SEAT-FOFLINE-CANONS.md) §4).
- HUD / splash media: `/opt/empiieros/shell/product-chrome/` then
  `/usr/share/empiieros/product-chrome/`
- Pack root: `/var/lib/empiieros-runtime/fofline-spa/current` (traverse
  `0751` / `0755` / `0644` — see [`PRODUCT-OPEN.md`](PRODUCT-OPEN.md))
- Historical `{state_dir}/shell_gui/shell_gui_audit.json` is **not**
  written by `secure_main.c`.
- **Windowed mode (service):** `/var/lib/empiieros-runtime/service/shell-window-mode.allow`
  must exist before F11 or `shell.window_mode` `{ "mode": "windowed" }` takes
  effect. Default is kiosk fullscreen. Returning to kiosk is always allowed.
  The Start menu does not expose this toggle; launcher **Advanced** may call
  the bridge op for field service when the file is planted (root, not in git).

---

## 4. Kiosk close lock

Alt+F4 must not close the OS face.

1. **labwc:** `empiieros-shell` binds `A-F4` to `true`, not `pkill`. The
   compositor consumes the chord. Tauri profile may still kill-and-respawn.
2. **GTK:** `delete-event` returns `TRUE`. The window also swallows Alt+F4
   if the key reaches the client.
3. **Autostart:** after a healthy run (≥30s) the restart budget resets. A
   crash-loop cools 15s and retries. The seat must not stay black after
   `MAX_APP_RESTARTS`.

The native Shell window is not a closeable app window. This section is
the compositor/GTK lock for that invariant.

---

## 5. Session bus (no systemd --user)

The getty kiosk uses `dbus-run-session`, not `systemd --user`.

1. `LABWC_UPDATE_ACTIVATION_ENV=0` — labwc must not call
   `import-environment`. Session `org.freedesktop.systemd1` is
   `Exec=/bin/false` and prints `Failed to import environment ... status 1`.
2. `NO_AT_BRIDGE=1` / `GTK_A11Y=none` / `AT_SPI_BUS=unix:path=/tmp/empiieros-no-at-spi`
   — GTK/WebKit must not activate `org.a11y.Bus` (at-spi) on the seat.
3. EmpiierOS `dbus-run-session` uses `kiosk-session-bus.conf` with no
   `<standard_session_servicedirs/>`, so `org.a11y.Bus` / session
   `systemd1` / keyring are not activated.
4. labwc/dbus stderr for `empiieros-shell` goes to
   `$XDG_RUNTIME_DIR/eliteew-ts-app.log`, not tty1.

---

## 6. Boot splash

On kiosk GUI start, `boot_splash.c` plays
`product-chrome/start_os.mp4` through GStreamer `playbin` →
`alsasink` at volume **0.25**. Video sink prefers `gtksink`
(`gtkwaylandsink` is fallback; it can preroll-hang on labwc). HUD
stays hidden until EOS/error/20s failsafe, then the pipeline goes
`NULL` so ALSA is released and `empiier_product_chrome_show` runs
(`show_all` after `no_show_all` — GTK3 `gtk_widget_show` leaves the
HUD unmapped). Missing file or sink skips the splash; the shell
still starts. Pulse/PipeWire are not added. This is not FoFline
MusicPlayer.

## 7. Product HUD (always-on seat chrome)

Native chrome is a **right-edge** overlay on the same `GtkOverlay` —
not a sibling GTK box (that sibling painted a full-width white
strip). The cluster is **always-on** and fully visible from kiosk
start. It is not FoFline-only. Icons stack vertically. There is no
peek, hover-expand, or clip host.

1. **Dock.** `GTK_ALIGN_END` / `GTK_ALIGN_CENTER`, `CHROME_PIN_END`
   inset. No `GtkScrolledWindow` clip. No `chrome_schedule_peek`.
   Flyouts open `GTK_POS_LEFT`. Six fixed ComfyUI icons (no growing
   shield text). `hide_product` does not hide the HUD. Hardcoded
   `-48px` peek is forbidden.
2. **Media.** Icons and per-button click WAV files live under
   `/opt/empiieros/shell/product-chrome` then
   `/usr/share/empiieros/product-chrome`. Source:
   `runtime/shell-gui/product-chrome/` (ComfyUI, see ORIGIN.md).
3. **Actions.** Each of the six icons opens its **own** non-modal
   flyout (summary + quick actions). The icon itself does not jump
   into an app. **Лаунчер** / **EliteEW** / **Shield** / **Updates**
   have a separate Open button. EliteEW Open is user-initiated `product.open` +
   `shell.product_surface` (not `autoOpened`, not a no-op
   `workspace=eliteew` event). Panel is sensors only. Volume is ALSA
   observe only. The chrome event box input-shapes to the cluster so
   transparent pixels do not eat FoFline clicks. F9/Escape still
   return from FoFline. Missing media falls back to `image-missing` /
   silent click. Click spawn is **`aplay -q` first** (raw ALSA), then
   `paplay` / `pw-play`.
4. **Honesty.** `audio.action` stays production-disabled. The panel
   does not pretend to select a capture device or provide OS A/B OTA.

## 8. Native auth card

`shell.auth_surface` is a FILL overlay on the same kiosk window: a click-
eating dimmer plus a centered Keycloak card (`auth_bounds.c`). It must
not `gtk_window_new` / `gtk_window_unfullscreen`. Host allowlist stays
`https://auth.eliteew.ru`. Escape/F9 still hide the surface.

## 9. Host glass family (Shield format)

Host workspaces and Seat Shield share one glass composition: full-bleed
root in `WorkspaceFrame` body, tabs, status chips, facts, one active
panel. Shared React helper: `shell-ui/src/secure/design/HostAppShell.tsx`.
CSS primitives `host-app-*` are aliased with `seat-shield-*` in
`secure.css`. Seat Shield markup stays on `seat-shield-*` class names.
System · Network · Diagnostics · Browser use `HostAppShell` (P1).
Launcher (`FoflineActivator`) uses Путь · Статус · Лаборатория (P2);
`LAUNCHER_JOURNEY` FSM unchanged.
Desktop Start flyout, icon tiles, taskbar Start/tray chips, Music and
Events windows use the same glass density (P3). Existing
`product-chrome/host-*.png` and `launcher.png` stay; ComfyUI regen
only when an audited gap list exists.
Capsule-gated OTA glass is the **Updates** host app
(`OtaPanel` / `SeatOtaSection` / `launcher.ota_*`) on the right-edge
dock — not a System tab and not a launcher journey step. Bootstrap
spa stays on Path; Lab may re-sync pack with honesty note. Air
license plants `{state}/ota/artifact.pk.hex` (not installer media).
Pack catalog is not a second desktop. Not MATCH by itself.
