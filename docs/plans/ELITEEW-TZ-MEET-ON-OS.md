# TZ — EliteEW Meet on EmpiierOS (false-hope lock)

**For:** EliteEW monorepo session (`powered_by_emperyworld`).
**From:** EmpiierOS 2026-09-18. Do **not** edit EmpiierOS from that session.

Hangup `/end` already ends the callee. That stays. The lie is the
**connected** copy while poet never enters the Jitsi room.

## What the glass showed (pack `d2ufr39pnb`)

Poet FoFline: «Соединение установлено / Видео недоступно в этом окне».
That is `videoCallOsWebKitPending*` after HTTP `POST …/join` **200**.
Lady_boss Chrome entered Meet and waited. OS syslog had **no**
`fofline meet stage shown` and **no** `fofline edge start`. Poet was
not in the room. Lady thought she was.

HTTP join ≠ Jitsi presence. The pending string is a false hope.

## Required EliteEW lock

1. **Do not say connected until OS `{ok:true}`.**
   `videoCallOsWebKitPendingTitle` must not read as a live call.
   Connecting / «открываем native Meet» is honest. «Видеозвонок идёт»
   only after `openEmpiierosMeetStage` returns `{ok:true}`.
2. **Fail closed on the stack0 call.** If open returns `!ok`, throws,
   or does not resolve in ~2s: call `endVideoCall` (same path as
   hangup) so the callee is not left in the room. Show
   `videoCallNativeMeetFailed*`, never pending-as-success.
3. **Keep hangup.** `closeEmpiierosMeetStage` + `/end` on end / ended /
   unmount stays. No `join_url` / JWT on close.
4. **`join_url` pass-through.** Host `meet.eliteew.ru`. Stack11 form is
   `https://meet.eliteew.ru/{room}?jwt=…#config…`. Do not rewrite to
   `app.eliteew.ru`. Do not mount an iframe on EmpiierOS.

## OS cause (source; needs GUI remux)

`MEET_HOST_LEN` was **14**; `meet.eliteew.ru` is **15**. Every
`openMeetStage` of `https://meet.eliteew.ru/{room}?jwt=…` returned
`join_url_denied` with no syslog. Pack stayed on pending copy.
Hangup still `/end`. Source now: length 15 + `_Static_assert`;
syslog `request kind=` / `denied code=`; JWT stops at `#`.

Mux `20260918T121648Z-shell-29442` GUI `eb8148d7…` is live. Poet call
22:17 +10: `request kind=open` + `shown` + Meet CSS/JS loaded. Jitsi
then painted “WebRTC is not available in your browser”.

That is the **OS WebKit floor**, not this pack. Stock Buildroot
`webkitgtk.mk` hardcodes `-DENABLE_WEB_RTC=OFF`. The seat has **no**
GStreamer (`libgst*` absent; WebKit NEEDED has no gst/nice).
`webkit_settings_set_enable_webrtc(TRUE)` cannot invent the API.
Do **not** mount an iframe. Do **not** add Pulse in EliteEW.

## Measure

- Vitest: `!ok` / timeout → `/end` invoked; pending copy is not
  success language; `join_url_denied` is failed, not painted connected.
- W5 still allow video-calls + GAF; admin denied.
- OS factory lock is in Trusted Substrate fragments + `external.mk`.
  Lab seat now has WebKit 2.52.6 `so.0.21.10` (`0c984e8e…`) plus
  `libgstrtpmanager.so` / `libgstrtp.so` via
  `20260919T072751Z-webkit-gst-25004` and WebKit outgoing
  `videorate` / `videotestsrc` / `audiotestsrc` via
  `20260919T074738Z-webkit-gst-11999`. Pin is **2.52.6**, not 2.54.
  Two-in-room SIGSEGV `8d317` was missing `rtpbin` (GST, not WebKit).
  Factory `PLUGIN_RTP` + `PLUGIN_RTPMANAGER` + `PLUGIN_INTERLEAVE`
  + base videorate/testsrc are on. `clocksync` is in
  `libgstcoreelements.so`. `deinterleave` mux
  `20260919T083405Z-webkit-gst-18901`. Living GUI
  `20260919T083418Z-shell-2687` `8599856d…` lowercases rtpmap
  and wraps `localDescription` getters (`fixDesc`). Glass 18:29:
  first answer was `opus ssrc`; `setVideoCodecs` still saw
  `OPUS` via the native getter; WebProcess then SIGSEGV gst
  `bb00d` on missing `deinterleave` (Jitsi WebAudio / file
  reader).   Those two locks are muxed. Poet media then dies after ICE:
  JVB `No remote fingerprints` / `ICE connected, but not DTLS`.
  Factory lock muxed `20260919T101341Z-webkit-gst-16482`: dropin
  `0002-webrtc-dtls-pem-before-sdp.patch` on **2.52.6** — PEM on
  enc+dec, `dtlsAcceptPeerCertificate` after pem recreate,
  sha-256 fingerprints. Living `so.0.21.10` is `5d776573…`.
  Hangup GUI `20260919T101353Z-shell-20640` `58d5566b…` (no
  dummy tracks; `/end` hides). Auth-card GUI later
  `d85ade76…`. Not 2.54.
  Glass 20:29 / 20:32 +10 (lady mobile, poet glass,
  `435cc136…` then `c4c05928…`): PEM **did** apply on
  `dtlsdec1`; ICE `COMPLETED`; then `dtlsconnection7`
  `certificate verify failed` → PC `failed`. Jitsi mute/cam
  icons stay crossed: mute state is SCTP, SCTP rides DTLS.
  Cause: GST 1.24 `dtlsdec` hooks `on-peer-certificate`
  only at NULL→READY; setting `pem` after READY makes a new
  `GstDtlsConnection` without that hook. WebKit recurse on
  elements never sees the connection object. First answer
  `opus nossrc recvonly` is Jitsi before gUM — not the mute
  icon. Lock muxed `20260919T103954Z-webkit-gst-22223`
  `libgstdtls.so` `ed2b13a6…` (rebind accept on every
  `create_connection`). WebKit still `5d776573…`. Glass
  20:54 `c0b61b59-…`: DTLS stayed `CONNECTING`; no
  video. 20:58 lady hangup: poet **0** `kind=close`,
  **0** `/end`; bounds still 21:02. Pack listener for
  `video_call.ended` did not fire — FoFline still
  lists lady_boss. OS cannot see the remote `/end`.
  21:04 `de1409e1-…`: same Meet WP `1034818` after
  `about:blank` → Jicofo `session-invalid`.   OS hide now
  `terminate_web_process` (`process recycled`). Inject
  `addTransceiver(sendrecv)` before `createAnswer` (no
  dummy `addTrack`). Muxed `20260919T111415Z-shell-11032`
  GUI `30c2f66e…`. Reboot+call 21:19 `725babe3-…`:
  gUM fired; ICE then **failed**. Signaling stays
  VIP `:443` inside `wg-portal`. JVB
  `93.77.165.185:13400` is outside the tunnel.
  Lab prelicense now has the tree hole
  `udp dport 13400 ip daddr 93.77.165.185 accept`
  (file + live chain, 2026-09-19 23:00 +10; table
  not reloaded). TURN `3478` is still not in nft.
  Seat Browser / Meet ICE walk is the operator's.

**Chrome follow-up (2026-09-19):** join + audio + lady-sees-poet
presence work. The iframe default (`4jxqj4w8lj`) is **closed**:
glass 03:34 +10 loaded Meet in the product WebProcess then SIGSEGV
took down FoFline. EliteEW canon already forbids nested Meet iframe
on WebKitGTK. EmpiierOS Meet is again a **sibling** stage, now
bounds-clipped (not FILL). Next pack:
[ELITEEW-TZ-FOFLINE-MEET-BOUNDS.md](ELITEEW-TZ-FOFLINE-MEET-BOUNDS.md).
