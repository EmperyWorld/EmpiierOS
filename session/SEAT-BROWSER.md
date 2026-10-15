# EmpiierOS Seat Browser

**Status:** Living session contract (S3 overlay + native chrome + ≤3
tabs, verified against `runtime/seat_browser_core/` +
`seat_browser_surface.c` on 2026-09-20).
**Glass map:** [`SEAT-FACE.md`](SEAT-FACE.md).
**Logic core:** `runtime/seat_browser_core/`
**Runtime:** `runtime/src/seat_browser.rs` via `core.rs`
**Host:** `runtime/shell-gui/seat_browser_surface.c`
**Shell:** host tile `browser` (key 6) + `SeatBrowserCard`
**Transport:** Unix NDJSON `SO_PEERCRED` per
[`../runtime/HEALTH-IPC.md`](../runtime/HEALTH-IPC.md).
**Oracle (behavior only):** EliteEW `DESKTOP-EBROWSER.md`
**Not:** Tauri EBrowser · Chromium · unrestricted clearnet in the OS
Shell `empiier://` face.

---

## 1. Principles

1. **Singular control plane.** Status, policy, navigate, and tab ops go
   through Runtime `seat.browser.*`. Never Tauri invoke as OS SoT.
2. **Shell is display-only.** Chrome never evaluates allow/deny.
3. **Navigation authority.** Overlay `decide_policy` fail-closes unless
   Runtime allows the URL (`evaluate` on show/`go`).
4. **Default deny.** Missing policy ≡ allowlist + empty allow → deny all
   HTTPS except `about:blank`.
5. **HTTPS-only shared deny.** `http` / `javascript` / `data` / `blob` /
   `file`; URL userinfo; non-443 ports.
6. **Sensitive confirms.** Policy arming / reset use F8 strings
   (`ENABLE_BROWSER_POLICY`, `RESET_SEAT_BROWSER`).
7. **Twin separation.** Do not conflate with `seat.shield.*` or
   `security.integrity.*`.
8. **Dedicated surface.** General browsing uses a second WebKit context
   — not unrestricted load in the OS Shell face.

---

## 2. Runtime ops

Registered in `seat_browser_core` `SEAT_BROWSER_OPS` and dispatched from
`core.rs`. `seat.browser.tabs` is **not** an op. Tab mutations are
`tab.open` / `tab.select` / `tab.close`.

| Op | Class | Notes |
|----|-------|-------|
| `seat.browser.status` | observe | `surface=shell_overlay`, `host=webkit_gtk`, mode, tabs, `downloads[{id,name,host,bytes,phase}]`, bounded `allow`/`deny` hosts |
| `seat.browser.policy` | observe | effective mode + bounded `allow`/`deny` host arrays |
| `seat.browser.policy.set` | mutation + confirm | seat-local; `ENABLE_BROWSER_POLICY` when tightening |
| `seat.browser.evaluate` | observe | allow/deny without navigate |
| `seat.browser.navigate` / `open` | mutation | Allow → `host_action=load` + `tab_id` (creates tab 1 if empty; else updates active URL) |
| `seat.browser.tab.open` | mutation | `{url?}` empty → `about:blank`. Cap **3**. Fourth → `denied`/`tab_limit`. `host_action=tab_open` |
| `seat.browser.tab.select` | mutation | `{id}` → `host_action=tab_select` |
| `seat.browser.tab.close` | mutation | `{id}`. Last tab → `host_action=hide`. Overlay hide does **not** purge tabs |
| `seat.browser.history` | mutation | `{direction:back\|forward}` → `host_action` |
| `seat.browser.reload` | mutation | `host_action=reload` |
| `seat.browser.close` | mutation | `host_action=hide` (tabs persist) |
| `seat.browser.media.set` | mutation | `{camera?,microphone?}`. Default both **on**. Enabling either requires `ENABLE_BROWSER_MEDIA`. Overlay asks `status` before UserMedia. Display capture stays denied. |
| `seat.browser.download.begin` | mutation | `{url,name?}`. Same HTTPS policy as navigate. Cap **1** in flight, **32 MiB**. Dest only `{state}/seat_browser/downloads/<basename>`. Status lists basename/host/bytes/phase — never a path. |
| `seat.browser.download.finish` | mutation | `{id,ok,bytes}`. Failed/over-cap unlinks the basename under downloads/. |
| `seat.browser.reset` | mutation + confirm | purge `{state}/seat_browser/` including `tabs.json` and downloads (`RESET_SEAT_BROWSER`) |

`system.snapshot` advertises family id `seat.browser` (`available`,
source `seat-browser-overlay`). That is IPC registration, not glass MATCH.

---

## 3. Native overlay chrome

`shell.seat_browser_surface` is a GUI op (not Runtime). Actions:

`show` · `hide` · `load` · `tab_open` · `tab_select` · `tab_close` ·
`reload` · `stop` · `back` · `forward` · `find` · `find_next` ·
`find_prev` · `zoom` · `go`

`show` / `go` re-evaluate via `seat.browser.evaluate`. Overlay `go` also
calls `seat.browser.navigate` so tab URL SoT does not drift. Keys: F5
reload, Alt+Left/Right history, Ctrl+F find, F3 / Shift+F3 next/prev,
Ctrl++/−/0 zoom, Ctrl+T new tab, Ctrl+W close tab, Ctrl+Tab cycle.
Escape/F9 hides. Find/zoom do **not** have Runtime ops.

Host is **≤3 related WebViews** in a `GtkStack`, not one WebView that
reloads a URL list. Persist `{state}/seat_browser/tabs.json`.

Shell card (`SeatBrowserCard`) renders living allow hosts as a vertical
list (add/remove rows), not a single-line field. VIP preset remains
`app.eliteew.ru` / `auth.eliteew.ru` / `meet.eliteew.ru`. The card also
drives evaluate, open/close, reload, history, and the tab list
(new/select/close). Overlay chrome is the in-page find/zoom bar plus
the native tab strip.

---

## 4. Policy modes

| Mode | Semantics |
|------|-----------|
| `open` | Any HTTPS passing shared deny |
| `allowlist` | Only matching allow (suffix / IP / CIDR) |
| `denylist` | HTTPS except deny; empty denylist forbidden on save and evaluate |

TTL fail-closed (default 300s receive clock) for **default/missing**
policy. Seat-local operator policy (`is_default=false`) restamps on
`evaluate` / `navigate` / `reload` / `history` / service open so a
long-lived Runtime does not white-out VIP Meet after 300s. API fetch TTL
comes later.

---

## 5. Waves and debt

| Wave | Scope |
|------|-------|
| S0–S1 | Design + `seat_browser_core` |
| S2 | Runtime observe + Shell card |
| S3 | WebKit overlay + navigation evaluate + native chrome (this file) |

Still debt:

- FoFline pack `1789890833725-7y746j4lvw` + stack0
  `1789891909119-45tkayntro`. OS host muxed (GUI `e18051fa…`). Glass
  planted the hook and accepted `kind=open`. Not MATCH.
- Content Manager ingest of seat downloads (EliteEW TZ, not this slice)
- prelicense TCP 443 beyond VIP
- Host navigation MATCH on glass (operator Open + chrome + tab/download walk)

Lab mux 2026-09-20 00:54 +10 (Runtime `19bf5f7f…` / GUI `5f1207a2…`)
is recorded in
[`../docs/plans/PRODUCT-WAVE-SEAT-BROWSER-MVP.md`](../docs/plans/PRODUCT-WAVE-SEAT-BROWSER-MVP.md).
Official `out/` SoT `5979e9cf…` includes this source slice. Prior
host bake `4104a67b…` is superseded. QEMU not rerun. Not MATCH.
