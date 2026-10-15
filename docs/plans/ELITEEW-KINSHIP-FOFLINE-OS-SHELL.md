# Kinship task — EliteEW as EmpiierOS shell (FoFline host)

**For:** a Cursor session whose **write home is the EliteEW monorepo**
(`powered_by_emperyworld`). Paste this file as the operator
prompt. Do **not** edit EmpiierOS from that session.

**From:** EmpiierOS lab glass 2026-09-18 16:02–16:13 +10
(`fofline-spa://localhost`, pack `1789443659753-rovukoqfw9`).
Verdict on the OS side stays `NOT_READY`. This brief is not an EmpiierOS
acceptance claim.

**Operator lock:** FoFline **video calls must work**. FoFline is the
EliteEW desktop living as the OS face — not a browser tab into the cloud
SPA, and not a second product universe.

---

## How to read this (planes)

| Plane | What it is | What it is not |
|-------|------------|----------------|
| **Cloud / browser** | `https://app.eliteew.ru`, SSO class `browser`, SPA and Socket.IO same-origin | The OS shell |
| **FoFline on Tauri** | Living host today. Same EliteEW React. Tokens/native bits via Tauri | The EmpiierOS axis |
| **FoFline on EmpiierOS** | Target. Same React desktop, local signed pack, native OIDC in the OS Runtime, HTTP `/api/v1` hopped by the OS GUI to portal VIP `10.88.89.1` | A kiosk, offline-AI pack, or “mini cloud” |

EmpiierOS = Linux floor + Rust Runtime (privileged IPC) + thin GTK/WebKit
host. EliteEW React **is** the desktop environment (the KDE/GNOME analog).
stack0 stays the business brain. Do not invent a FoFline API twin.

Host adapters must be **host-shaped**, not Tauri-shaped:

- Tokens: `EmperyDesktopNativeOidc` (`refreshSessionJson` / `logoutSessionJson`).
  EmpiierOS already owns `grant_type=refresh_token`. Do not replay access as
  refresh. Do not require `__TAURI_INTERNALS__.invoke` for FoFline-on-OS.
- HTTP: on EmpiierOS, relative `/api/v1` is correct (scheme proxy). Absolute
  `https://app.eliteew.ru/api` is folded back by the host. Do not “fix”
  FoFline-on-OS by forcing every caller onto absolute app origin without
  talking to EmpiierOS (that path already caused CORS/409).
- Realtime: Socket.IO **cannot** use `window.location.origin` when that
  origin is `fofline-spa://localhost`. EmpiierOS has **no** `/socket.io`
  scheme handler.

---

## Lab evidence (do not re-litigate)

- Allowlisted HTTP (`/desktop/*`, `/profile/me`, `/ugs/*`, `/content/*`,
  `/live-voice/settings`, `/product-desktop/news/counts`) → **200**.
- In-SPA OIDC refresh on EmpiierOS **works** (helper `oidc_refresh` 16:03:08
  and 16:13:13; access rotated; no 401).
- VIP `:443` ESTAB from `wg-portal` does **not** prove Socket.IO. Soup API
  proxy also uses that tuple. `/socket.io` is not scheme-proxied.
- `call_target=desktop_only` already treats `empery-fofline` as desktop
  audience.

### EliteEW kinship closeout — 2026-09-18 evening (EliteEW tree)

Recorded from the EliteEW session. **Not** EmpiierOS glass. **Not** a new
lab pack.

- Built-in W5 allowlist now includes `/api/v1/gaf-chat` and
  `/api/v1/video-calls`. Admin / marketing / factory / desktop admin /
  site-manager / product-desktop admin stay denied. Not
  `FOFLINE_REMOTE_TOOLS_EXTRA_PREFIXES`.
- `fofline-connected` + `FOFLINE_CONNECTED_APP_COMPONENTS` include
  `GAFChatApp` and `VideoCallApp`. Marketing, MusicForge, Experts,
  Admin Desktop stay out.
- One resolver `resolveSocketIoHttpOrigin`: opaque `fofline-spa:` →
  `https://app.eliteew.ru`. Browser app origin stays same-origin.
  Refresh still patches `socket.io.opts.auth` only.
- EmpiierOS class is `foflineNative` + `EmperyDesktopNativeOidc` and **no**
  Tauri invoke. `join_url` stays `meet.eliteew.ru`. EliteEW **tree**
  (2026-09-19) mounts the browser iframe in `VideoCallApp` and does
  not call `openMeetStage`. Meet WebProcess death now emits
  `empiieros-meet-stage-lost`; the tree `/end`s. Public pack
  `1789750041479-4jxqj4w8lj` is on the seat disk (02:58 +10) and
  does not call `openMeetStage`. Prior `1789730765446-d2ufr39pnb`
  opened the overlay and did not `/end` on crash. See
  the trashed iframe TZ (`ELITEEW-TZ-FOFLINE-CALL-CLOUD-LIKE`). Glass waits on a
  product reload.
- EliteEW measure: `test_fofline_w3_w5.py` 7 passed; Vitest socket/Meet/
  surface/auth 20 passed. Bake session later reported 33 Vitest on
  the call files. Two `test_desktop_install_profiles.py` fails
  are outside that wave.
- EmpiierOS promoted lab `current/` to public
  `1789720073125-5fjlwh96mc` (19:10 +10), then
  `1789730765446-d2ufr39pnb` (21:50 +10, hangup close), then
  `1789750041479-4jxqj4w8lj` (02:58 +10, in-window + `/end` on
  Meet death).

Pending-incoming **200** (`[]`) already landed on the previous pack.
Socket.IO handshake proof still needs a product reload on the new pack.

---

## Required EliteEW work

Do these in one kinship wave. Expand W5 only with matching surface/profile.
Do **not** paper over with `FOFLINE_REMOTE_TOOLS_EXTRA_PREFIXES` alone.

### 1. W5 — video calls + GAF (calls are GAF-entered)

Living product: 1:1 calls start from a GAF dialog; signaling is Socket.IO
`video_call.*`; recovery poll is `/video-calls/pending-incoming`; Meet is
`join_url` on `meet.eliteew.ru` (stack11). See
`documentation/product/INTERNAL-VIDEO-CALLS.md`.

- Add `/api/v1/video-calls` to `_FOFLINE_ALLOW_PREFIXES`.
- Add `/api/v1/gaf-chat` to the same list (create/accept need the chat).
- Keep admin / marketing / factory / `desktop/admin` /
  `desktop/site-manager` / `product-desktop/admin` **denied**.
- Lock in `shared/api/tests/test_fofline_w3_w5.py`:
  - allow ` /api/v1/video-calls/pending-incoming`
  - allow ` /api/v1/video-calls/gaf-chats/{id}`
  - allow ` /api/v1/gaf-chat/me` and `/chats`
  - still deny admin/marketing/unknown
- Update `documentation/product/desktop/DESKTOP-FOFLINE-EDITION.md` §7 / §17.5
  allowlist table in the **same phase**.
- Align `shared/apps/_profiles/fofline-connected.yaml` **and**
  `foflineConnectedSurface.ts` so GAF/VideoCall chrome that the API now
  allows is an intentional FoFline surface — not a hidden app that still
  403s, and not a catalog tile whose API stays denied.

Marketing, MusicForge, Experts ensemble (`DesktopAiChatApp`), Admin Desktop
stay out.

### 2. Socket.IO origin on `fofline-spa:`

Today:

- `hooks/useSocketConnection.tsx` — if hostname is not `eliteew.ru`, falls
  through to `VITE_WS_ORIGIN` / `VITE_API_ORIGIN` / **`location.origin`**.
  On EmpiierOS that origin is `fofline-spa://localhost`.
- `services/communication/EmperyworldSocketService.ts` `getSocketURL()`
  **always** returns `window.location.origin` in a page.

Required:

- One resolver (same idea as `resolveAbsoluteApiBaseURL`): on
  `fofline-spa:` (and other opaque FoFline origins) Socket.IO must speak
  `https://app.eliteew.ru` (Engine.IO wants http(s), not `ws://` as the
  `io()` origin).
- Use it from **both** socket entrypoints. No third copy.
- Tests: `fofline-spa://localhost` → `https://app.eliteew.ru`; browser
  `https://app.eliteew.ru` unchanged; Tauri path unchanged unless it
  already shares the helper.
- After OIDC access refresh, keep the living rule: **patch `socket.io.opts.auth`
  only** — do not `disconnect()+connect()` every TTL.
- `sso_replace_confirm` / `app_build` stay. Do not send refresh tokens
  into JS.

EmpiierOS will **not** scheme-proxy `/socket.io` in this kinship wave.
Direct WebKit `wss://app.eliteew.ru` over the portal VIP is the intended
path until a later native WS bridge exists.

### 3. Meet / WebRTC must not be Tauri-only

Living lock: nested Meet iframe in WebKit is a **white frame**; Tauri
FoFline opens a top-level `video-call-meet` WebviewWindow.

On EmpiierOS there is **no** `__TAURI_INTERNALS__`.
`videoCallNativeMeetStage.ts` therefore reports “not a native product
shell” and the in-page iframe is the leftover — which EliteEW already
knows is white on WebKit.

Required in EliteEW (honest, not a fake native window):

- `foflineNative` + `EmperyDesktopNativeOidc` + **no** Tauri invoke is a
  first-class host (`empiieros` / OS WebKit), not “browser profile”.
- Do not dead-end «Позвонить» / incoming UI when Tauri invoke is missing.
- EmpiierOS Meet is a **sibling** WebView clipped to VideoCall bounds
  (`openMeetStage(url, bounds)` / `setMeetStageBounds`). Do **not**
  mount a nested Meet iframe on this host. Do **not** add a Tauri-only
  gate that falls through to iframe (white / product crash).
- `join_url` host is `meet.eliteew.ru`. EmpiierOS portal hostmap does
  **not** include that name yet (OS follow-up). EliteEW must still emit
  the same `join_url`; do not rewrite it to `app.eliteew.ru`.

### 4. Stop treating “native FoFline” as “has Tauri”

Audit and split host checks that today mean `__TAURI_INTERNALS__` while
the comment says FoFline:

- `videoCallNativeMeetStage.ts`
- `foflineProductWebviewWipe.ts`
- `hostNetworkLinkSnapshot.ts` (taskbar network indicator already shows
  on the OS glass via some other path — do not break it)
- any `foflineNative && invoke` that would skip Socket.IO, calls, or
  session wipe on EmpiierOS

Prefer: `getOidcRuntimeProfile() === 'foflineNative'` for product class;
separate `hasTauriInvoke()` / future `hasEmpiierosHost()` for host I/O.

---

## Non-goals (EliteEW session)

- Do not edit this repo.
- Do not add a second stack0 or “FoFline API twin”.
- Do not allow admin / marketing / factory / site-manager.
- Do not put refresh tokens or DPoP secrets in the SPA.
- Do not change Keycloak clients.
- Do not bake a new public SPA unless the operator asks; code + tests +
  docs first. EmpiierOS lab remux/OTA is a later OS turn.

---

## Measure (EliteEW tree)

1. `test_fofline_w3_w5.py` — video-calls + gaf-chat allow; admin still deny.
2. New or extended unit tests for the socket origin resolver on
   `fofline-spa:`.
3. `DESKTOP-FOFLINE-EDITION.md` §7 / §17.5 matches the new prefixes.
4. Profile / `FOFLINE_CONNECTED_APP_COMPONENTS` matches the call surface.
5. No new `FOFLINE_REMOTE_TOOLS_EXTRA_PREFIXES` dependency for this
   baseline.

Glass on EmpiierOS after a new pack (OS session, not this one):
pending-incoming **200** (empty list is OK); no 8s 403; Socket.IO
handshake to `https://app.eliteew.ru` (not `fofline-spa:`); 1:1 ring
events can land. Meet media over `meet.eliteew.ru` may still wait on the
OS hostmap/surface — say so honestly if that is still closed.

---

## EmpiierOS follow-ups (this OS session)

1. Observe Socket.IO (`fofline edge start|failed`). **Not** a `/socket.io`
   scheme proxy. Lab pack is now `1789750041479-4jxqj4w8lj` (maps opaque
   `fofline-spa:` → `https://app.eliteew.ru`). Lab pack is now
   `1789794743865-pza6u2zqzf` (`openMeetStage(url,bounds)`). Glass
   waits on a product reload.
2. Host API hop already accepts any `/api/v1/` path. W5 403 was stack0
   `fofline_remote_tools_denied`, not the OS proxy. Pending-incoming **200**
   `bytes=2` (`[]`) already observed on the previous pack; still 200 after
   promote (one VIP timeout during the fetch, then recovered).
3. Portal hostmap now includes `meet.eliteew.ru` → `10.88.89.1` (same
   VIP SNI as Tauri SOCKS). Live `/etc/hosts` must match before a call.
   `channel_up` on an old Runtime would rewrite the old two-name line —
   remux Runtime/portal helper before that.
4. Host hook `EmperyDesktopMeetStage.openMeetStage(url,bounds)` /
   `setMeetStageBounds` / `closeMeetStage` posts JSON to
   `empiierosMeet`. Bare URL is `meet_stage_bounds_required` (no FILL).
   Lab GUI `8599856d…` (`20260919T083418Z-shell-2687`, conceal
   + rtpmap lowercase + `localDescription` getter `fixDesc`). Crash still
   `empiieros-meet-stage-lost`. Living pack `1789794743865-pza6u2zqzf`
   sends bounds. Prior `4jxqj4w8lj` iframed. Glass after reload.
   Host length is 15. Prelicense UDP `13400` only to `93.77.165.185`.
   WebKit 2.52.6 `0c984e8e…` + GST + libnice + `libgstrtpmanager.so`
   (`rtpbin`) + `videorate` / `videotestsrc` / `audiotestsrc`
   + `libgstinterleave.so` (`deinterleave`,
   `20260919T083405Z-webkit-gst-18901`). Do not restore iframe or FILL.
5. OS `FOFLINE_CONNECTED_APP_COMPONENTS` now includes `GAFChatApp` and
   `VideoCallApp` as **pack** names (Start spoiler only). Not host tiles.
6. Official `out/` remux later. Do not remux the live seat while an
   operator is inside FoFline.
7. Analog audio on this Dell is GPIO amp, not a missing driver:
   helper `empiieros-audio-volume` now enables AFG `0x07` for
   `10ec0236` / `102809c1`. Kernel Realtek quirk remains a later
   image bake.
