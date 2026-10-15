# TZ — FoFline opens EmpiierOS Seat Browser (EliteEW pack)

**For:** EliteEW monorepo session (`powered_by_emperyworld`).
**From:** EmpiierOS 2026-09-20 17:21 +10. Do **not** edit EmpiierOS
from that session. Do **not** add a Tauri plugin, Pulse/PipeWire, or
a nested HTTPS iframe in the product WebView.

**Companion brief (read first):**
[`FOFLINE-OS-BRIEF.md`](FOFLINE-OS-BRIEF.md)

**Already-binding pack TZs (do not regress):**

- Meet sibling: [`ELITEEW-TZ-FOFLINE-MEET-BOUNDS.md`](ELITEEW-TZ-FOFLINE-MEET-BOUNDS.md)
- Sound: [`ELITEEW-TZ-FOFLINE-AUDIO-ON-OS.md`](ELITEEW-TZ-FOFLINE-AUDIO-ON-OS.md)

**Pack published (EliteEW report 2026-09-20):** `1789890833725-7y746j4lvw`
sha256 `da97f2877868ed806ece375b9643f6d43ad3e330c57938347f9ea24546ede7d8`,
`include_audio=false`. Shell `latest.json` still 0.5.36.
**stack0 / edge live:** `1789891909119-45tkayntro`
(`prod-baked-20260920-1811`, commits `30944c93c` + `f794b6a00`, not
pushed). Catalog tile is in that profile. Official EmpiierOS `out/`
is not this TZ.

This TZ is **the catalog browser tile on EmpiierOS**. It is not a
port of Tauri `EBrowserApp` / `connection-agent` `ebrowser_*`.

---

## Operator sentence

On the OS seat, FoFline’s desktop browser tile still believes it is
Tauri. `ebrowserNative.ts` requires `__TAURI_INTERNALS__.invoke`.
There is no invoke. The tile fails closed or does nothing. EmpiierOS
already has a first-party Seat Browser (`seat.browser.*` + overlay
chrome + ≤3 tabs + seat-local downloads). The pack must **open that
host**, the same way Meet opens `EmperyDesktopMeetStage`.

---

## Cause (one layer)

`foflineNative` is a **product class**, not a host I/O check. You
already split this for Meet and audio:

| Flag | Means |
|------|--------|
| `getOidcRuntimeProfile() === 'foflineNative'` | FoFline edition |
| `hasTauriInvoke()` | Tauri `invoke` exists |
| `isEmpiierosFoflineHost()` | FoFline + `EmperyDesktopNativeOidc` + **no** Tauri |

`EBrowserApp` still uses `isEliteewNativeDesktopProductShell()`
(`foflineNative` **and** Tauri invoke) then `openNativeEBrowser` →
`ebrowser_open`. On EmpiierOS that is `not_native` / dead.

Same class of bug as treating EmpiierOS Meet as a nested iframe
(product WebProcess SIGSEGV). Do not “fix” the browser by mounting
`https://` inside the FoFline WebView.

---

## What EmpiierOS already implements (do not re-litigate)

Runtime Unix NDJSON (`SO_PEERCRED`), not Tauri:

| Op | Role |
|----|------|
| `seat.browser.navigate` / `evaluate` | HTTPS + VIP allowlist SoT |
| `seat.browser.tab.open` / `select` / `close` | Cap 3 related WebViews |
| `seat.browser.media.set` | camera / microphone persist (default on) |
| `seat.browser.download.begin` / `finish` | dest only `{state}/seat_browser/downloads/<basename>`, 32 MiB, one in flight |

HUD tile **Seat Browser** (key `6`) already opens the overlay without
the pack. The missing generator is **FoFline catalog → host**.

Host inject is in EmpiierOS source (`window.EmperyDesktopSeatBrowser` via
`empiierosSeatBrowser`). Pack adapter is published. Glass still needs
`spa_sync` + hide/re-open **and** a muxed GUI that plants the hook.
If the host object is missing, fail honest (`seat_browser_unavailable`).
Do **not** fall back to Tauri, iframe, or `window.open`.

---

## Required lock (must be in the published pack)

Treat EmpiierOS as `isEmpiierosFoflineHost() === true`.

### 1. Split the host check

In `EBrowserApp` / `ebrowserNative.ts` (and any catalog click that
opens the browser):

- Tauri path: keep `hasTauriInvoke()` + `ebrowser_*`. Do not break it.
- EmpiierOS path: `isEmpiierosFoflineHost()` +
  `window.EmperyDesktopSeatBrowser`.
- Cloud / browser: keep today’s non-native UI. Do not invent a host.

`isEliteewNativeDesktopProductShell()` must **not** be the only gate
for “we have a native browser”.

### 2. Host object (mirror Meet, thinner)

```ts
type SeatBrowserBounds = { x: number; y: number; width: number; height: number };

type EmpiierosSeatBrowserHost = {
  openSeatBrowser?: (args: {
    url?: string;
    bounds?: SeatBrowserBounds;
  }) => Promise<{ ok?: boolean; code?: string }> | { ok?: boolean; code?: string };
  closeSeatBrowser?: () => Promise<{ ok?: boolean; code?: string }> | { ok?: boolean; code?: string };
};
```

Injected name: **`window.EmperyDesktopSeatBrowser`** (same `EmperyDesktop*`
family as OIDC / Meet / NetworkLink).

Rules:

1. `url` if present must be `https://…`. Empty / omitted → host opens
   `about:blank` or the last VIP tab (host SoT). `http://`, `file:`,
   `fofline-spa:`, `blob:` → do not send; show pack error.
2. **Do not send `accessToken` / refresh / cookies to the host.**
   Runtime policy is the OS SoT. Pack may still GET `/api/v1/ebrowser`
   for **banners only**.
3. `bounds` is optional. If you have a stage rect, send
   `getBoundingClientRect()` CSS pixels. The OS overlay may ignore
   clip in v1 (full overlay, not Meet-shaped hole). That is OK.
   Do not fail the pack if the host ignores bounds.
4. On EmpiierOS, **do not paint a second WebView** inside
   `EBrowserApp`. The app is a launcher + honest banners (allowlist
   miss, `tab_limit`, `download_busy`). OS chrome is the address bar /
   tabs / find / zoom.
5. Close: `closeSeatBrowser()` when the user closes the EBrowser
   window. Do not `about:blank` the product WebView.

Suggested helper names (your tree): `openEmpiierosSeatBrowser` /
`closeEmpiierosSeatBrowser` next to `openEmpiierosMeetStage`.

### 3. Downloads CTA

Tauri lock (`ebrowserOpenDownloads`) opens Content Manager
«Загрузки», not an OS folder. **Keep that for Tauri.**

On EmpiierOS:

- Do **not** ask the host for `/home`, `file://`, or a folder picker.
- Do **not** claim the seat file is already in Content Manager.
- Seat-local dest is OS state (`basename` only in `seat.browser.status`).
- Content Manager **ingest** of those bytes is a **later** EliteEW
  wave (VIP `POST` content API). Out of this TZ. A Downloads button
  may open Content Manager’s empty/system folder **or** show copy
  that files are seat-local until ingest exists.

### 4. Media toggles

If the host later grows `setMedia({camera, microphone})`, call it.
Until then, leave OS defaults (both on). Do not `invoke`
`ebrowser_set_media_toggles` on EmpiierOS.

### 5. Honest failure

| Host result / missing | Pack UI |
|-----------------------|---------|
| no `EmperyDesktopSeatBrowser` | `seat_browser_unavailable` — not “open in iframe” |
| `allowlist_miss` / `scheme` / `tab_limit` / `download_busy` | existing deny banners |
| Tauri-only codes (`not_native`, `invoke_failed`) | must not be the EmpiierOS success path |

---

## Forbidden

- Porting `connection-agent` `ebrowser_*` into EmpiierOS or calling
  them when `hasTauriInvoke()` is false.
- Nested `<iframe src=https://…>` or `<webview>` inside FoFline for
  general browsing (Meet iframe ban applies).
- Pulse / `pactl` / host click WAV as “browser audio”.
- Sending refresh tokens into JS or into the host.
- `include_audio=true` / `audio-guides` as a side effect of this tile.
- Claiming MATCH / DEMO_PASSED / “EBrowser готов на ОС”.
- Editing EmpiierOS from the EliteEW session.

---

## Tests (EliteEW tree)

Name the files you change (`ebrowserNative.ts`, `EBrowserApp.tsx`,
new `ebrowserEmpiieros.ts`).

1. `isEmpiierosFoflineHost()` + `EmperyDesktopSeatBrowser.openSeatBrowser`
   called with `{url:'https://app.eliteew.ru/'}` — **no**
   `ebrowser_open` invoke.
2. `hasTauriInvoke()` path still calls `ebrowser_open` (spy, keep it).
3. EmpiierOS + missing host → `seat_browser_unavailable`, no iframe.
4. EmpiierOS + `http://` or `file:` → not sent to the host.
5. Downloads CTA on EmpiierOS does not call an OS folder opener.

---

## Measure (glass, after a published pack)

1. `launcher.spa_sync` the new `app_build_id`.
2. Hide and re-open FoFline (product WebKit keeps the old pack until
   then).
3. Open the desktop **browser** tile (not only HUD key `6`).
4. VIP `https://app.eliteew.ru/` loads in the **OS overlay**, not
   inside the FoFline page.

Pass: overlay chrome visible; FoFline desktop still there; no Tauri
error banner; syslog has no product-WebProcess crash from an iframe
nav.

Fail-closed: tile no-ops, iframe of VIP inside FoFline, or
`not_native` as the only outcome.

---

## EmpiierOS follow-up (not this EliteEW session)

Inject `window.EmperyDesktopSeatBrowser` in the product WebView
(`auth_surface.c` / `native_bridge.c` pattern of Meet) and map
`open` → `seat.browser.navigate` + `shell.seat_browser_surface` show.
Until that inject lands, pack tests use a mock host. Do not block the
pack adapter on the inject existing today.
