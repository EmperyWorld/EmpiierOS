# FoFline on EmpiierOS — brief (not acceptance)

**Date:** 2026-09-20
**Status:** Harvest map for the next EliteEW pack session and for
EmpiierOS agents. **Not** `READY` / MATCH / DEMO_PASSED.
**Write home:** this file + TZs under `docs/plans/`. Do not copy the
SPA into `shell-ui/`.

Read-only kinship: `powered_by_emperyworld` (sibling of EmpiierOS)
(especially `shared/web/frontend/src/`).

---

## 1. What FoFline is on this seat

FoFline is a **signed SPA pack** served at
`fofline-spa://localhost/index.html?eliteewFoflineClient=1` after
`product.open`. It is not Tauri, not Chromium+Pulse, not the HUD.

```text
Unlock → Activate → Portal → spa_sync → OIDC → Open
        → product WebKit (fofline-spa)
        → pack JS talks to VIP :443 and to host injects
```

Public clearnet TCP 443 stays denied. VIP hosts
(`app` / `auth` / `meet.eliteew.ru`) are the product network.

**Published spa-latest (EliteEW 2026-09-20):** `1789890833725-7y746j4lvw`
sha256 `da97f287…`. Prior lab pack `1789837366816-w7k51h8btf`. Product
WebKit **keeps the previous pack until hide/re-open** after
`launcher.spa_sync`. stack0 + edge live
`1789891909119-45tkayntro` (`prod-baked-20260920-1811`, runtime git
`30944c93c`) includes the catalog browser. HUD key `6` remains the OS
tile. Not MATCH.

---

## 2. Host injects already on the product WebView

Injected from `runtime/shell-gui/auth_surface.c` (OIDC inject tail).
No `__TAURI_INTERNALS__`.

| `window.*` | Pack use | OS SoT |
|------------|----------|--------|
| `EmperyDesktopNativeOidc` | bootstrap / refresh / logout JSON | `launcher.oidc_*` |
| `EmperyDesktopMeetStage` | `openMeetStage(url, bounds)` | sibling Meet WebView |
| `EmperyDesktopNetworkLink` | `getLinkSnapshot()` | wifi / ethernet chip |
| `EmperyDesktopSeatBrowser` | `openSeatBrowser` / `closeSeatBrowser` | OS overlay + `seat.browser.*` |

See [`ELITEEW-TZ-FOFLINE-SEAT-BROWSER.md`](ELITEEW-TZ-FOFLINE-SEAT-BROWSER.md).
Glass is not MATCH until spa_sync + hide/re-open on a GUI that plants
the hook.

Detection that already works in EliteEW source:

`isEmpiierosFoflineHost()` =
`foflineNative` + `EmperyDesktopNativeOidc` + **not** `hasTauriInvoke()`.

---

## 3. Pack vs OS ownership

| Surface | Owner | Honest state |
|---------|-------|----------------|
| PIN / capsule / portal / spa_sync / OIDC card | EmpiierOS Runtime | wired |
| FoFline desktop, catalog, GAF, Content Manager UI | EliteEW pack | living in monorepo; glass = last published pack |
| Meet pixels | OS sibling WebView | pack must call bounds host; **no iframe** on OS |
| Seat Browser pixels | OS overlay + `seat.browser.*` | HUD tile works. Pack host + stack0 tile live. Glass saw `kind=open`. Not MATCH |
| Seat Shield | OS host workspace | not a FoFline app |
| MusicPlayer sound | EliteEW Web Audio on OS | TZ exists; Play after re-open is the measure |
| Host click WAV / ALSA volume | EmpiierOS HUD | not the library player |
| Content Manager ingest of seat downloads | EliteEW later | OS dest is `{state}/seat_browser/downloads/` basename only |

---

## 4. Nuances EliteEW must redo (or keep)

These are the failure modes a “FoFline update” usually reintroduces.

### 4.1 `foflineNative` ≠ Tauri

`isEliteewNativeDesktopProductShell()` is Tauri. Using it as “native
FoFline” kills EmpiierOS (EBrowser today, Meet historically).

**Redo:** every `invoke(` in a FoFline path needs a sibling
`isEmpiierosFoflineHost()` adapter, or the call is dead on glass.

### 4.2 Do not nest HTTPS in the product WebView

Meet iframe on WebKitGTK SIGSEGV’d the **product** process (FoFline
went white). Canon: sibling stage. Same ban for EBrowser / arbitrary
`https://` iframes.

**Redo:** if a new pack mounts Meet or a browser inside FoFline “like
Chrome”, reject it. Bounds TZ wins.

### 4.3 Meet `/end` is pack-owned

OS does not call video-calls. Crash / hide fires
`empiieros-meet-stage-lost`. Pack must `POST /end` (and
`pagehide` keepalive). Remote hangup must `closeMeetStage` on the
other seat.

### 4.4 Sound: no media URL on `fofline-spa:`

GStreamer cancels `fofline-spa:` `/download*` and `blob:fofline-spa:`.
The failed `<audio>` holds exclusive ALSA `hw:0,0` — **whole FoFline
goes silent**, including Meet.

**Keep:** `foflineDecodedPlayback.ts` (`decodeAudioData` +
`AudioBufferSourceNode`, no `Range: bytes=0-0` probe).
**Redo:** any new player / preview / TTS that sets
`HTMLMediaElement.src` to a ticket or blob on this origin.

OS leftovers (`octet-stream` XHR, `__empiierosAudioBlob`) are
**not** acceptance. Pack must work without them.

### 4.5 Dual-flag query

`?eliteewFoflineClient=1` must win over `desktopNative` setup.
Do not ship a pack that sets both and then takes the Tauri OIDC arm.

### 4.6 Tokens

Host OIDC is access-session JSON. **No refresh token in JS**, no
refresh on the Meet/browser host objects, no `accessToken` argument
on Seat Browser open (unlike Tauri `ebrowser_open`).

### 4.7 Downloads / Content Manager

Tauri EBrowser → Content Manager «Загрузки» via API deep link
(`ebrowserOpenDownloads`). EmpiierOS Seat Browser writes a **local**
basename under Runtime state (32 MiB, one in flight). That is not
the cabinet.

**Redo:** do not open an OS folder; do not tell the user the file is
in Content Manager until a later ingest wave.

### 4.8 Pulse / sound server

Image has no Pulse/PipeWire. Analog PCM is exclusive. Do not add
`pactl` checks or “fix OS audio” from the pack session.

### 4.9 spa-latest vs glass

Publishing VIP `spa-latest` is not glass. Need `launcher.spa_sync`
**and** hide/re-open. Tell EmpiierOS the new `app_build_id`. Do not
drop a tarball into this repo.

---

## 5. Kinship files (read-only map)

| Mechanism | Path | KEEP / HARVEST / DROP |
|-----------|------|------------------------|
| EmpiierOS host detect | `…/eliteewProductHost.ts` `isEmpiierosFoflineHost` | KEEP |
| Meet sibling adapter | `…/videoCallNativeMeetStage.ts` | KEEP — pattern for Seat Browser |
| Meet app | `…/VideoCallApp.tsx` | KEEP bounds path; DROP iframe on OS |
| EBrowser Tauri | `…/ebrowserNative.ts` `ebrowser_*` | KEEP for Tauri; HARVEST shape; DROP as OS SoT |
| EBrowser app | `…/EBrowserApp.tsx` | REWRITE → host launcher on OS |
| Downloads CTA | `…/ebrowserOpenDownloads.ts` | KEEP Tauri; REWRITE copy on OS |
| Web Audio owner | `…/foflineDecodedPlayback.ts` | KEEP |
| OIDC profile | `…/prepareOidcRuntimeProfile.ts` / `setupDesktopNativeOidcProfile.ts` | KEEP FoFline query win |
| Tauri connection-agent | `pack-organization/connection-agent` | DROP for EmpiierOS plans |

---

## 6. What EmpiierOS still owes after their pack

1. Official `out/` bake of Seat Browser slices + this host inject.
2. H6 seccomp on a **fresh install**, not mux.
3. Do not ENABLE_ACTS / quarantine / integrity from a FoFline pack task.
4. stack0 catalog tile is EliteEW (`1789891909119-45tkayntro`). OS
   already plants `EmperyDesktopSeatBrowser` on this lab (GUI
   `e18051fa…`). Their “inject missing / `seat_browser_unavailable`”
   line is stale for `192.168.8.10` after hide/re-open.

---

## 7. Phrase for the EliteEW session

Pack `1789890833725-7y746j4lvw` and stack0
`1789891909119-45tkayntro` are ingested. No new spa-latest. Do not
ask EliteEW to rewrite the host unless glass shows a new miss. Their
uncommitted Seat Shield watch post-build check is not this slice.
