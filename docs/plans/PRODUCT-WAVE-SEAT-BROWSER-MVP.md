# PRODUCT-WAVE-SEAT-BROWSER-MVP

**Wave:** S3 host tile + native chrome — **in progress 2026-09-20**
**Prep:** EmpiierOS_spehsial_app `seat_browser_core` + SeatBrowser dossiers  
**Oracle:** EliteEW `DESKTOP-EBROWSER.md` (behavior). **Not** Tauri stage.

## EmpiierOS (done)

| Artifact | Path |
|----------|------|
| Session | `session/SEAT-BROWSER.md` |
| Core | `runtime/seat_browser_core/` |
| Runtime | `runtime/src/seat_browser.rs` via `core.rs` |
| Host | dedicated ephemeral WebKit overlay + native chrome (`seat_browser_surface.c`) |
| Shell | host tile `browser` + `SeatBrowserCard` (allowlist rows + VIP + open/close + reload/history) |

## First glass slice (this turn)

1. Leave Meet as-is.
2. Do **not** port Tauri EBrowser (tabs/downloads/find/zoom/SOCKS split).
3. Launch path: Launcher → host desktop → tile **Seat Browser** (or key `6`).
4. Lab policy: VIP allowlist `app.eliteew.ru` / `auth.eliteew.ru` / `meet.eliteew.ru`.
5. First URL: `https://app.eliteew.ru/` (nft allows VIP `:443`). Public clearnet stays denied.
6. WebRTC on the Seat Browser WebView (not Meet): `enable_webrtc` +
   `media_stream` + UserMedia allow. Jitsi "WebRTC is not available in
   your browser" was the overlay settings, not the factory engine.
7. Seat-local VIP policy restamps on `evaluate`/`navigate` (not only
   Runtime start / `policy.set`). Else Meet URL goes `navigation denied`
   after 300s and the WebView stays white.
8. Lab prelicense UDP `13400` only to `93.77.165.185` — Seat Browser
   WebRTC uses the public JVB, not `wg-portal`. Tree overlay already
   had the rule; live seat did not until 2026-09-19 23:00 +10.
9. Lab mux 2026-09-20 00:54 +10: Runtime `19bf5f7f…`
   (`20260919T145413Z-runtime-2159`) + GUI `5f1207a2…`
   (`20260919T145423Z-shell-11852`) + `index-CaMBz6NY.js`.
   Native chrome strings live; unit/`IPAddressDeny` unchanged.
   Operator still walks Open + back/forward/reload on glass.

## Overlay chrome (in source, 2026-09-20)

`seat_browser_surface.c`: back / forward / reload / stop / find /
find next-prev / zoom plus a native tab strip over a GtkStack of ≤3
related WebViews. Keys: F5, Alt+Left/Right, Ctrl+F, F3, Ctrl++/−/0,
Ctrl+T/W/Tab. Find/zoom are GUI-only. Tab mutations are Runtime ops
(`seat.browser.tab.*`). Downloads stay cancelled.

## Media (2026-09-20)

`seat.browser.media.set` persists `camera`/`microphone` on the seat-local
policy (default **on**). Overlay UserMedia asks Runtime `status`. Display
capture stays denied. Enabling a device requires `ENABLE_BROWSER_MEDIA`.
Lab mux 2026-09-20 16:35 +10: Runtime `db9439bf…`
(`20260920T063509Z-runtime-13314`) + GUI `ac9ba033…`
(`20260920T063519Z-shell-24534`) + `index-BNv3b80O.js`.
Status `camera=true` `microphone=true` on VIP allowlist. Disable
without confirm; re-enable needs `ENABLE_BROWSER_MEDIA`. Official
`out/` still S5C. Not MATCH.

## Tabs (2026-09-20)

Runtime persist `{state}/seat_browser/tabs.json` cap 3.
`seat.browser.tab.open` / `tab.select` / `tab.close`. Navigate creates
tab 1 or updates the active URL. Overlay hide does not purge tabs.
Last `tab.close` emits `host_action=hide`. Overlay is three related
WebViews, not one WebView that reloads a URL list. Lab mux 2026-09-20
16:47 +10: Runtime `a913c84b…` (`20260920T064733Z-runtime-7514`) + GUI
`b07b4cd6…` (`20260920T064743Z-shell-18970`) + `index-Cj40NGTM.js`.
Probe: VIP allowlist; `about:blank` open; `example.com` allowlist_miss;
fourth `tab_limit`; overlay hide keeps tabs; last close hides. Official
`out/` still S5C (no media.set, no tabs). Not MATCH.

## Downloads (2026-09-20)

Not Content Manager and not Tauri EBrowser. Overlay asks
`seat.browser.download.begin`; WebKit writes only the Runtime
`file://` dest under `{state}/seat_browser/downloads/`. Cap 32 MiB,
one in flight. Status shows basename/host/bytes/phase. Lab mux
2026-09-20 16:56 +10: Runtime `fc3442ef…`
(`20260920T065644Z-runtime-15309`) + GUI `b58032ef…`
(`20260920T065654Z-shell-24073`) + `index-IgUPFtM_.js`. Probe: empty
begin `invalid_params`; `about:blank` scheme deny; example.com
`allowlist_miss`; VIP `favicon.ico` begin; second `download_busy`;
finish lists basename only. Official `out/` still S5C. Not MATCH.

## Still debt

- Content Manager ingest of seat downloads (EliteEW TZ)
- FoFline pack `EBrowserApp` hook (EliteEW TZ, not Tauri invoke)
- prelicense TCP 443 for general HTTPS (separate floor decision)
- Host navigation MATCH on glass (muxed; Open + chrome walk is the operator's)

## Snapshot capability (2026-09-20)

`system.snapshot` / `diagnostics.snapshot` advertise family id
`seat.browser` as `available` / `seat-browser-overlay`. Lab mux
2026-09-20 17:01 +10: Runtime `48f1275e…`
(`20260920T070138Z-runtime-22267`). Probe listed the row; GUI
`b58032ef…` / `index-IgUPFtM_.js` unchanged. Not MATCH. Not
`seat.shield`. Official `out/` still S5C.
