# Product Open & Application Launch Contract

**Status:** Living contract. Production path: `runtime/src/launcher_product.rs`
(`LOCAL_START_URL` =
`fofline-spa://localhost/index.html?eliteewFoflineClient=1`).
**Plane:** `session/` + Runtime + native Shell. **Not:** Tauri IPC. **Not:** remote SPA.
**Host arrangement:** [`SEAT-LAUNCHER.md`](SEAT-LAUNCHER.md).
**Glass map:** [`SEAT-FACE.md`](SEAT-FACE.md).
**Kiosk / HUD:** [`SHELL-GUI.md`](SHELL-GUI.md).

Lab hashes and pack ids below are chronology, not official `out/` and
not MATCH. The invariants in this file are the living lock.

## Purpose

Launch a verified local FoFline pack on a typed origin. The frontend never
builds URLs. Runtime builds the start URL. Native WebKit serves the pack.

## Invariants

1. **Typed Target Law.**
   - Caller may pass only `fofline_local` (default), `connected_cloud`, or
     `spa_deep_link`.
   - `connected_cloud` is `remote_spa_forbidden`.
   - `spa_deep_link` is `deep_link_unavailable` this slice.
   - Runtime entrypoint is exactly
     `fofline-spa://localhost/index.html?eliteewFoflineClient=1`.
2. **Pack prerequisite.** Unlock + `allow_fofline` + `{state}/fofline-spa/current/`
   with `index.html` and `pack-manifest.json`. Otherwise `pack_not_ready`.
3. **Shell-readable pack.** Runtime `CapabilityBoundingSet` is
   `CAP_NET_ADMIN` only, so publish does not `chown`. The generators that
   used to hide the pack (`StateDirectoryMode=0700`, `StateStore::open`
   `0700`, `spa_pack` `0700` on `fofline-spa/`) now stay at traverse-only
   `0751`. Runtime start and pack-ready republish: state/`fofline-spa/`
   `0751`, pack dirs `0755`, files `0644`. Secrets stay `0600`. Files stay
   root-owned and not world-writable. Native serves only regular uid-0
   files under the pack root. The kiosk must walk `PACK_ROOT` at GUI start.
4. **Native origin.** `empiieros-shell-gui` registers `fofline-spa` and serves
   only regular files under `/var/lib/empiieros-runtime/fofline-spa/current`.
   Path `..`, `%2e%2e`, `%00`, and symlinks are denied. `fofline-spa` is
   secure and CORS-enabled, not `as_local`. JS/HTML use
   `finish_with_response` plus `Access-Control-Allow-Origin: *` so ES
   modules are not CORS-blocked. Every stylesheet uses simple `finish()`
   — WebKitGTK ignores CSS from `URISchemeResponse`. Living Vite
   `__vitePreload` waits on `<link rel=stylesheet>` for pack
   `/assets/*.css`; a CORS finish hangs that load and `App-*.js` never
   starts. Product HTTPS to the VIP uses the host CA store and
   NSS (`resolve` then `files`); the product/shell WebKit context keeps
   sandbox off for that reason — same class as the auth white page.
   `decide_policy` + custom schemes stay the boundary.
   Native product chrome is a right-edge always-on HUD overlay (not a
   full-width GTK strip): five ComfyUI icons stacked vertically and
   always fully visible (no peek). Each icon is a flyout (summary +
   quick actions, Open is a button — not the icon). EliteEW Open is
   user-initiated `product.open`. ALSA click foley, F9/Escape still
   returns to `empiier://shell/index.html`. The HUD stays after
   `hide_product`.
   The OS host is the seat launcher (PIN → portal → pack → OIDC → Open),
   not a second EliteEW catalog. The host canvas harvests EliteEW
   **visual** DNA only (wallpaper, movable OS icons on a 110px grid,
   glass windows, 46px taskbar rail with the EliteEW Start mark).
   The OS face is viewport-locked (`100dvh`, no document scroll). Host
   modules open as even-inset windows (`1rem`) on the stage above the
   taskbar; the window body scrolls (wheel → nearest `overflow:auto`
   ancestor, not `window.scrollBy`), the desktop does not. The rail
   tray shows ComfyUI icons for network/battery/volume; percents appear
   on hover. Volume adjusts with the wheel or a click slider
   (`audio.control`, 10–100). Clock, UI language, and a confirm-gated
   power flyout (`REBOOT`/`POWEROFF`) stay on the rail.
   Pack-kind tiles stay out of that canvas; they open only after
   `product.open`. The launcher window is not auto-opened.
   Lab consume/sync stay behind the activator spoiler. Portal may
   one-shot `channel_up` when the stored conf is valid and down.
   `autoOpened` stays forbidden.
5. **Native OIDC first.** Living FoFline opens a separate Keycloak window
   (`oidc-auth`) after RFC 9126 PAR. EmpiierOS does the same: helper PAR POST,
   then `launcher.oidc_start` returns `client_id` + `request_uri` only, then
   `shell.auth_surface` on baked `https://auth.eliteew.ru` as a **card**
   (`auth_bounds.c`: FILL dimmer + centered ~480×640 WebView). The kiosk
   `GtkWindow` stays fullscreen and undecorated. This is not a second
   top-level and not `windowed` mode. The dimmer swallows clicks so the
   launcher cannot race OIDC. Allowlist, `NEW_WINDOW` ignore, and PAR
   complete stay the boundary. Then
   `launcher.oidc_complete`. Query-authorize is rejected (`Pushed Authorization
   Request is only allowed`).    The product WebView gets
   `window.EmperyDesktopNativeOidc` with access/id only. The living pack
   reads that bridge. Public / lab pack `1789837366816-w7k51h8btf`
   (dist-build `2026-09-19T17:05:57Z`) includes that reader, maps
   opaque `fofline-spa:` Socket.IO to `https://app.eliteew.ru`, keeps
   Meet `openMeetStage(joinUrl, bounds)`, and on `foflineNative`
   plays user audio from memory (`contentApi` `decodeAudioData` /
   `createBufferSource`) instead of a media URL. Prior lab
   `1789794743865-pza6u2zqzf` (dist-build `2026-09-19T05:13:50Z`)
   had the Meet bounds path without decoded playback. It
   `/end`s on `empiieros-meet-stage-lost` and `pagehide` keepalive
   (access token only). Prior lab `1789750041479-4jxqj4w8lj`
   (dist-build `2026-09-18T16:50:26Z`) mounted the in-page iframe.
   Prior lab `1789730765446-d2ufr39pnb` (dist-build
   `2026-09-18T11:27:25Z`) still opened OS Meet and called
   `closeMeetStage()` on hangup / ended / unmount. Prior lab
   `1789720073125-5fjlwh96mc` (web bake `20260918-1759`) opened the
   OS stage but did not close it. Prior lab
   `1789443659753-rovukoqfw9` used `location.origin`. Older public pack
   `1786238708853-jn25ybw0v0` does not:
   it persists only tokens that include a `refreshToken` field, sets
   `authenticated` only when both fields exist, then refreshes via
   `/api/v1/auth/sso/refresh` (baked discovery on `fofline-spa:`). Inject
   plants the access JWT in that `refreshToken` field and answers the SSO
   refresh fetch from the current access/id. Runtime refresh never leaves
   Runtime or IPC. Do not set `mobileEmbeddedClient` (that profile is
   `empery-mobile`, not `empery-fofline`). Realm stays `empery`. In-SPA
   `location.assign` to Keycloak / the HTTPS bridge is swallowed; product
   `decide_policy` cannot finish browser PKCE. Inject also sets
   `eliteew.foflineClient.v1`. Relative `/api/v1` stays on
   `fofline-spa://`; the scheme proxy hops allowlisted paths to VIP
   `10.88.89.1` with SNI/Origin `https://app.eliteew.ru`. Absolute
   `app.eliteew.ru/api/` is folded back to the path. The hop plants
   `X-SSO-Replace-Confirm: 1` so stack0 can insert/replace the fofline
   `sid` (XOR with a live browser session, or a newer native `sid`).
   Native complete does not POST `/api/v1/auth/sso/callback`. Inject
   arms the living `sessionReplaceConfirmedUntil` window for Socket.IO.
   Product WebKit logs `fofline edge start` / `failed` (no query) for
   `socket.io`, `ws(s):`, `meet.eliteew.ru`, and `https://app.eliteew.ru/`
   — observe only; `/socket.io` is not scheme-proxied this slice.
   Inject plants `window.EmperyDesktopMeetStage.openMeetStage(url,bounds)`
   / `setMeetStageBounds` / `closeMeetStage` via `empiierosMeet`.
   Open without `{x,y,width,height}` is `meet_stage_bounds_required`
   — the OS will **not** FILL the product WebView. Meet is a START-aligned
   sibling clip over `VideoCallApp` (`meet_bounds.c`). JavaScript
   fullscreen on that view is swallowed.    Nested Meet iframe in the
   product WebProcess is a documented WebKitGTK dead end (glass
   2026-09-19 03:34 +10: `kind=iframe` then product SIGSEGV). Living
   pack `1789794743865-pza6u2zqzf` sends bounds; prior
   `1789750041479-4jxqj4w8lj` mounted the iframe; prior
   `1789730765446-d2ufr39pnb` called open without bounds (FILL
   overlay). Product `decide_policy` still allows subframe Meet HTTPS
   from `fofline-spa` (legacy / Chrome path). Host length is 15
   (`meet.eliteew.ru`). Missing map is `meet_hostmap_unavailable`.
   Other hosts are `join_url_denied`. Open/close/bounds/deny syslog
   `kind=` / `code=` (no query). A visible Meet
   `web-process-terminated` hides the stage and dispatches
   `empiieros-meet-stage-lost` `{code:meet_process_crashed}`. Hangup
   `hide` parks, then `webkit_web_view_terminate_web_process` (syslog
   `process recycled`) so the next join cannot reuse Jicofo
   `session-invalid`. `about:blank` alone left ICE/XMPP in the same
   WebProcess. The inject adds `addTransceiver(sendrecv)` before
   `createAnswer` (no dummy `addTrack`). The OS does **not**
   `POST /end` (no call id). The pack must. Living
   pack `4jxqj4w8lj` listens; prior `d2ufr39pnb` did not. Boot
   probe carries `err` plus `es` (resource target). API 4xx/5xx bodies
   ≤256 B may log a machine `reason=` (no tokens).
   Runtime stays `IPAddressDeny=any`. Pack `/assets/*.css` uses simple `finish()` so Vite
   preload `<link>` tags fire `load`.    Living FoFline `init()` hydrates then
   always calls `refreshAccessToken()`. `refreshSessionJson` /
   `logoutSessionJson` post to host `empiierosOidc` (product origin
   only). The host calls `launcher.oidc_inject` `{force:true}` or
   `launcher.oidc_logout`. Replay of the injected access JWT is not a
   refresh. The old-pack SSO fetch `/api/v1/auth/sso/refresh` uses the
   same host refresh.
   Host **Выйти из EliteEW** (`launcher.oidc_logout`) sits next to
   **Войти**. It wipes the Runtime session and hides the product surface.
   Auto-open is disabled so the seat is not trapped in a half-loaded SPA.
   Auth chrome is the Keycloak page inside that card (Escape/F9 dismiss;
   dimmer click does not cancel PAR).
6. **Pass-through only.** Shell compares the Runtime URL to the baked constant
   and calls `shell.product_surface`. Native loads only that exact string.
7. **Audit.** `{state}/product/last_launch.json` `0600` records target,
   `app_build_id`, entrypoint, and time. No filesystem path of the pack.
8. **Host-map.** After portal up, `launcher.hostmap_apply` sends
   `app.eliteew.ru` / `auth.eliteew.ru` to VIP `10.88.89.1`. Public TCP 443
   stays denied. See [`PORTAL-HOSTMAP.md`](PORTAL-HOSTMAP.md).

## IPC

`product.open` — after unlock + `allow_fofline` + pack ready + native OIDC
session. Expired access is refreshed in Runtime from the stored refresh
token (helper DPoP, no tokens on IPC). Without a refreshable session the
error is `oidc_session_required`.
`launcher.status` reports `product_open=allowed|unavailable` and
`oidc_session=present|missing`.

## Live lab (2026-09-18, not an official image)

Cause of the first fail: `chown` needs `CAP_CHOWN`, and
`StateDirectoryMode=0700` hid even `0644` files. Lock: chmod-only
`0751`/`0755`/`0644` on the unit, `StateStore::open`, pack store, and
Runtime start — not only on `product.open`. Cause of the first auto-open with no serve: muxed
GUI `aeb9bdfe…` had no `fofline-spa` scheme. Cause of the stuck SPA with
no login and no exit: `autoOpened` skipped native OIDC and chrome was
F9-only. Lock: no `autoOpened`; `oidc_session_required`; native
«Лаунчер» + `shell.auth_surface`.

- Runtime `09e2569c…` (`20260918T012514Z-runtime-16178`).
- SPA helper `c9c8c6ef…` (`20260918T012828Z-spa-helper-31016`).
- GUI `0d9809bc…` (`20260918T034905Z-shell-17736`) / Shell
  `index-Dih6JgA2.js`. Pack walk live:
  state/`fofline-spa` `0751`, `current` `0755`/`0644`.
- Keycloak 26 auth codes are dotted (`uuid.tab.client`). Runtime and
  the SPA helper both use `auth_code_ok` (`.` allowed). Mux them
  together: helper `oidc_token` re-parses `callback.json`.
- Complete IPC must not be treated as a leak because `detail` mentions
  the word refresh. GUI matches only `"access_token"` / `"refresh_token"`.
- Live 11:38:07 `oidc session injected` + local `fofline-spa` serve.
  `product.open` without a session is `oidc_session_required`.
- Keycloak `UserSessionLimitsAuthenticator` with `userClientLimit=1`
  logs out the previous `empery-fofline` session on each new login.
  That is IdP policy, not an EmpiierOS complete fail.
- FoFline `init()` calls native `refreshSessionJson` after hydrate.
  A stub `{ok:false}` left the SPA on the preloader after a good
  complete/inject. Lock: return the current bootstrap access/id.
- Quote fonts are absent from the core pack and stay denied.
- `fofline-spa` must not be `as_local`. Live 12:51 +10: login/complete
  `applied`, pack CSS CORS-finished, boot `r:false` `css:5`. Vite
  `__vitePreload` never loaded `App-CrSAywAU.js`. Lock: all CSS simple.
- `IPAddressDeny=any`. Lab `eno1` `192.168.8.10/30` is the polygon,
  not a product Settings default.
- Official `out/` SoT `5979e9cf…` (Runtime `79e06a7d…`, Shell
  `index-DMc2IDZV.js`). Slice `fa1fbc6b…` is superseded. QEMU did
  not walk Open. Not MATCH.
