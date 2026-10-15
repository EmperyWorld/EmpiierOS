# TZ — FoFline Meet is a bounds-clipped sibling stage (EliteEW pack)

**On disk (2026-09-19 15:33 +10):** public/lab
`1789794743865-pza6u2zqzf` sha256 `b5c30279…`. Helper sync
15:33:20–15:33:38 +10. Packed `VideoCallApp-bs-BnhEY.js` has
`setMeetStageBounds` / `openMeetStage(url,bounds)` /
`meet_stage_bounds_required`. Lab GUI is now
`20260919T083418Z-shell-2687` `8599856d…` (conceal + getter
`fixDesc` + `deinterleave` GST `20260919T083405Z-webkit-gst-18901`).
Seat glass after FoFline reload is the operator's. Official `out/`
is not this TZ.
If the screen goes white: **stop** — do not restore iframe or FILL.

**For:** EliteEW monorepo session (`powered_by_emperyworld`).
**From:** EmpiierOS 2026-09-19. Do **not** edit EmpiierOS from that session.

This supersedes the old iframe call path (trashed
`ELITEEW-TZ-FOFLINE-CALL-CLOUD-LIKE.md`) and
[ELITEEW-TZ-FOFLINE-IN-WINDOW-PACK.md](ELITEEW-TZ-FOFLINE-IN-WINDOW-PACK.md).
Those packs (`4jxqj4w8lj`) mount a nested Meet iframe. EliteEW canon
already forbids that on WebKitGTK (`DESKTOP-EBROWSER.md`,
`REALTIME-SOCKETIO-AND-WS.md`). Seat glass 2026-09-19 03:34 +10:
Meet loaded in the product WebProcess (`kind=iframe`) then the
**product** `WebKitWebProcess` SIGSEGV (`at 18` / `8d317`) — FoFline
went white. Do **not** keep fighting the iframe.

Cloud Chrome may keep the iframe. EmpiierOS must use the **Tauri-shaped
stage**: sibling WebView, CSS-pixel bounds over `VideoCallApp`, never
fullscreen, never a nested iframe.

Hangup `/end` and `empiieros-meet-stage-lost` stay.

## What EmpiierOS now implements

`window.EmperyDesktopMeetStage` (no `__TAURI_INTERNALS__`):

| Hook | Contract |
|------|----------|
| `openMeetStage(joinUrl, bounds)` | `joinUrl` is `https://meet.eliteew.ru/…?jwt=…`. `bounds` is `{x,y,width,height}` CSS pixels from `getBoundingClientRect()` of the VideoCall Meet stage. Bare URL is `meet_stage_bounds_required` — the OS will **not** FILL. |
| `setMeetStageBounds(bounds)` | Re-clip while the stage is open (resize / move VideoCall). Width/height `< 48` hides. |
| `closeMeetStage()` | Hide. No URL. No JWT. |

OS syslog (no query / no JWT): `kind=open` / `kind=bounds` / `kind=close`
/ `code=meet_stage_bounds_required` / `meet_stage_bounds_invalid` /
`join_url_denied`. Crash still `empiieros-meet-stage-lost`
`{code:meet_process_crashed}`. JavaScript `requestFullscreen` on Meet
is swallowed (`fullscreen denied`). Chat stays; Meet is only the
clipped sibling, not the whole product.

## Required EliteEW lock (must be in the published pack)

Treat EmpiierOS (`isEmpiierosFoflineHost()` / `EmperyDesktopMeetStage`
present, **no** Tauri invoke) as the **native Meet stage**, not as the
browser iframe.

1. **Do not mount `<iframe src=meet.eliteew.ru>` on EmpiierOS.**
   Browser / cloud iframe stays for Chrome only.
2. **After `/join` 200**, measure the Meet stage element
   (`meetStageBoundsFromElement`) and call
   `openMeetStage(joinUrl, bounds)`. Do **not** call open with only
   the URL.
3. **While the call window exists**, on layout / resize / drag, call
   `setMeetStageBounds(bounds)` from the same element rect. If
   `width`/`height` drop under 48 (minimized / offstage / a settings
   modal), the OS **conceals** the hole and keeps the Meet WebView.
   It must not `about:blank` or `/end`. Hangup is still
   `closeMeetStage`.
4. **Hangup / unmount / lost:** `closeMeetStage()` if this session
   opened the stage, then `endVideoCall` (`POST /end`) as today.
   Keep `/end` on `empiieros-meet-stage-lost` and `pagehide`.
   Remote hangup must still run this path: glass 20:58 lady `/end`
   left poet in the room (`c0b61b59-…`, **0** `kind=close`, **0**
   poet `/end`, bounds still at 21:02). `video_call.ended` is in
   `VideoCallApp-bs-BnhEY.js` and did not fire. OS hide on poet
   `/end` cannot see the callee's phone. OS fail-closed muxed
   `20260919T101353Z-shell-20640`: successful `POST …/end` hides
   the sibling even if the pack skipped `closeMeetStage`. `meetPost`
   is a wait-map with ids. Dummy tracks are gone. Hide now
   `terminate_web_process` (`process recycled`) so the next join
   cannot reuse Jicofo `session-invalid`. Inject
   `addTransceiver(sendrecv)` before `createAnswer`.
   Muxed `20260919T111415Z-shell-11032` GUI `30c2f66e…`.
   Reboot+call 21:19: hide recycle held; video died
   on ICE (`conference.iceFailed`), not on hangup.
5. **Do not rewrite** `meet.eliteew.ru` to `app.eliteew.ru`.
6. **Do not** show Tauri placeholder copy (`videoCallNativeMeetActive*`)
   as if Meet were painted inside the pack. Chat + VideoCall chrome
   stay visible around the clipped hole.
7. **Do not** ToggleFullscreen `identifier=*`. Do not ask the OS
   kiosk to go fullscreen for Meet.

`videoCallNativeMeetStage.ts` today only `invoke`s Tauri and treats
`openEmpiierosMeetStage(joinUrl)` as optional without bounds. Change
that: EmpiierOS host is first-class; pass bounds; do not fall through
to iframe when the hook exists.

## Measure

- Packed `VideoCallApp-*.js` contains `setMeetStageBounds` or
  `openMeetStage` with a second bounds argument. It must **not**
  mount Meet `<iframe` on the EmpiierOS / `foflineNative` arm.
- Vitest: EmpiierOS host + join 200 → hook called with rect; iframe
  branch not taken; hangup `/end` + `closeMeetStage`; lost `/end`.
- Seat glass (after uptake + FoFline reload): syslog `kind=open` then
  `kind=bounds`; no `kind=iframe`; GAF chat still visible; Meet is
  only over the VideoCall stage; no kiosk fullscreen.

Official EmpiierOS `out/` is not this TZ. WebKit 2.52 factory bump
is not this TZ. Do not restore FILL overlay. Do not restore iframe
as the OS default.
