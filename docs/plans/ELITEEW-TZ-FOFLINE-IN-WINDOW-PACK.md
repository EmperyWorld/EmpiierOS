# TZ — FoFline call must live inside EliteEW (pack bake)

**Superseded for EmpiierOS default Meet (2026-09-19):** “inside
EliteEW” is **not** a nested iframe. Use
[ELITEEW-TZ-FOFLINE-MEET-BOUNDS.md](ELITEEW-TZ-FOFLINE-MEET-BOUNDS.md).
Pack `4jxqj4w8lj` was replaced on disk by `1789794743865-pza6u2zqzf`
(bounds sibling). Do not bake another iframe.

**For:** EliteEW monorepo session (`powered_by_emperyworld`).
**From:** EmpiierOS 2026-09-19 02:42 +10.
**Do not edit** EmpiierOS.

Living seat `current/` is now `1789750041479-4jxqj4w8lj` (helper
sync 2026-09-19 02:58 +10). Packed JS has no `openMeetStage` and
listens `empiieros-meet-stage-lost` + keepalive `/end`. Product
WebKit still holds the previous overlay pack until poet reopens
FoFline. Glass of the iframe path is **not** done. OS WebRTC and
crash-notify stay live. This TZ was a **public/lab pack**, not
another OS bake.

## Glass just now (02:39 +10)

Poet FoFline, GUI `39b3bbee…`, pack `d2ufr39pnb`:

1. `POST …/video-calls/210afcb5-…/join`
2. `fofline meet stage request kind=open` + `shown host=meet.eliteew.ru`
3. Meet is a GTK overlay (`meet_box` FILL) on the **whole** product
   WebView. That is **not** a FoFline window. Chat and `VideoCallApp`
   sit underneath. Lady Chrome stays in-page iframe.
4. Same glib SIGSEGV (`WebKitWebProcess` `at 18` / `8d317`).
5. OS lock fired: `process terminated reason=crashed visible=1` →
   hide → `fofline meet stage crashed reason=meet_process_crashed`
   and `empiieros-meet-stage-lost` `{code:meet_process_crashed}`.
6. **No** `POST …/210afcb5-…/end` after the crash. Lady keeps poet
   on the roster.

The operator's sentence: *you open the call not inside EliteEW*.
That is `openMeetStage`. Fix it in the pack.

## Required lock (must be in the published pack)

Treat EmpiierOS (`isEmpiierosFoflineHost()` / `EmperyDesktopMeetStage`
present, **no** `__TAURI_INTERNALS__`) as **browser / cloud**.

### 1. Meet pixels inside `VideoCallApp`

After `/join` is active and `iframeJoinUrl` is
`https://meet.eliteew.ru/{room}?jwt=…#config…`:

- Mount the **same** `<iframe>` as the browser branch
  (`JITSI_IFRAME_ALLOW`: camera, microphone, display-capture, autoplay).
- Do **not** call `openEmpiierosMeetStage` / `openMeetStage`.
- Do **not** rewrite the host to `app.eliteew.ru`.
- Do **not** show `videoCallNativeMeetActive*` /
  `videoCallOsWebKitPending*` («Видеозвонок идёт» /
  «Картинка встречи поверх этого окна»).
- Opening `VideoCallApp` must **not** close `GAFChatApp`.
- Hangup stays `POST /api/v1/video-calls/{id}/end`.
- Tauri `video_call_meet_stage_open` + bounds: **do not break**.

### 2. `/end` when the surface dies

OS (already muxed) does **not** call video-calls. The pack must:

- Listen `window` event `empiieros-meet-stage-lost` with
  `detail.code` (at least `meet_process_crashed`) → `endVideoCall(id)`.
- On `pagehide` → `endVideoCallKeepalive(id)`: `fetch` POST `/end`,
  `keepalive: true`, `Authorization: Bearer` **access** token only.
  No refresh token. Do not use axios / auth abort registry (they
  cancel on teardown).
- Idempotent `already_ended`. No JWT on close.
- Optional but wanted: closing the `VideoCallApp` window (X) also
  `/end`s. React Strict Mode must not `/end` a live call on a
  fake remount — use a real window-close path, not a blind unmount
  if that double-fires in their app.

### 3. Do not put the overlay back

If the iframe is a white rectangle on poet: **stop. Do not bake
`openMeetStage` + the placeholder.** Write “iframe still white”.
OS bounds-clip is a later EmpiierOS session.

**Measured 2026-09-19 03:05 +10:** white. Join 200/1454, no overlay
open, no Meet resource load. Product policy ignored Meet HTTPS
inside FoFline. Lab GUI `6e0a4511…` (03:32) allows the subframe.
EliteEW must not put `openMeetStage` back. Re-walk after reopen.

## Disk today (verify, do not regress)

These should already be in the monorepo. If a branch drifted,
restore them, then bake.

- `VideoCallApp.tsx` — no `openEmpiierosMeetStage`; iframe arm;
  `EMPIIEROS_MEET_STAGE_LOST_EVENT` + `endVideoCallKeepalive`.
- `videoCallNativeMeetStage.ts` — `EMPIIEROS_MEET_STAGE_LOST_EVENT`
  = `empiieros-meet-stage-lost`; `meetStageLostCode`.
- `api/videoCallApi.ts` — `endVideoCallKeepalive`.
- Tests:
  - `VideoCallApp.empiierosIframe.test.tsx` (iframe, no
    `openMeetStage`, no Tauri/OS placeholder, hangup `/end`,
    lost-event `/end`, pagehide keepalive).
  - `videoCallNativeMeetStage.test.ts` (`meetStageLostCode`).
  - `videoCallApi.keepalive.test.ts`.
  - `desktopDeepLink.test.ts` `opens VideoCall without closing GAF chat`.
- Docs: `INTERNAL-VIDEO-CALLS.md`, `DESKTOP-FOFLINE-EDITION.md`.
- W5: video-calls + gaf-chat allow; admin denied.

EmpiierOS session already ran those Vitest files: **11 passed**.
Re-run before bake.

## Bake / publish

Done on EliteEW: public `spa-latest` `1789750041479-4jxqj4w8lj`
sha256 `9296bc5a…` (`pub_date=2026-09-18T16:51:00Z`). Seat
uptake 02:58 +10. Do **not** remux EmpiierOS official `out/`.
Do **not** restore `openMeetStage` if the iframe is white.

## Measure before you call it done

| Check | Pass |
|---|---|
| Bundle has no EmpiierOS `openMeetStage(` on the default call path | yes |
| Bundle listens `empiieros-meet-stage-lost` and keepalive `/end` | yes |
| Vitest above + `test_fofline_w3_w5.py` | exit 0 |
| After seat uptake: syslog **no** `meet stage request kind=open` | yes |
| Meet pixels **inside** the FoFline `VideoCallApp` window | yes |
| GAF chat still listed / visible | yes |
| No «Видеозвонок идёт / картинка поверх» empty state | yes |
| Hangup `/end` still ends lady | yes |
| If Meet process dies: syslog `crashed reason=` **and** `POST …/end` | yes |
| Iframe is not a white rectangle | yes |

## Non-goals

- Do not edit EmpiierOS.
- Do not add Pulse.
- Do not change Keycloak / W5 beyond the existing allow.
- Do not send refresh tokens into JS.
- Do not invent `setMeetStageBounds` in this pack.
- Do not “fix” camera by reopening OS Meet.

## EmpiierOS (already done; do not redo)

- WebKit `356011aa…`, `ENABLE_WEB_RTC=ON`, GStreamer, `libseccomp`.
- GUI `39b3bbee…` mux `20260918T150331Z-shell-28108`: crash → hide +
  `empiieros-meet-stage-lost`. Official `out/` unchanged.
