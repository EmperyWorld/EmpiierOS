# EmpiierOS — Seat launcher (FoFline host)

**Status:** Living contract. Display path: `shell-ui/src/desktop/`.
Production IPC: `runtime/src/launcher_*.rs` + `product.open`.
**Glass map:** [`SEAT-FACE.md`](SEAT-FACE.md).
**Metal-proven canons:** [`SEAT-FOFLINE-CANONS.md`](SEAT-FOFLINE-CANONS.md).
**Plane:** `session/` + Shell + Runtime + native kiosk. **Not:** Tauri.
**Not:** a second EliteEW catalog. **Not:** remote SPA.

## Purpose

The OS host is the seat launcher. It shows **one next step** on a fixed
path, then yields to the local FoFline pack. Runtime still owns PIN,
consume, portal, pack, and OIDC. The Shell only gates what the operator
sees and which single action is offered.

```text
Unlock → Activate → Portal → Workspace → Sign-in → Open
```

## Arrangement

1. **Host canvas.** Wallpaper + movable OS icons. Without a FoFline
   license (bound capsule + entitlement) only the launcher tile and the
   network tile exist. The launcher window stays on the activation path.
   Network is seat floor — PIN lock and missing consume must not hide
   it. Other host modules (browser, shield, system, …) stay hidden.
   After consume, the launcher
   tile still does not auto-open the product. Pack-kind tiles stay out of
   that canvas; they exist only as names in the Start spoiler and only
   run after `product.open`.
2. **Journey rail.** `LAUNCHER_JOURNEY` in `foflineLauncherStage.ts` is
   the display order. Marks are `done` / `current` / `ahead` / `blocked`.
   Blockers: PIN not issued, PIN lockout, invalid stored portal conf.
3. **One current step.** `FoflineActivator` offers the current stage on
   the primary floor. Activate shows **air or file** (same step, two
   buttons). Sign-out (`launcher.oidc_logout`) is extra once
   `oidc_session=present`. Lab consume/sync/status stay in the
   `FoflineActivatorLab` spoiler.
4. **Native surfaces.** Sign-in is `shell.auth_surface` — a card on the
   same kiosk, not a second top-level window. Open is
   `shell.product_surface` after Runtime returns the baked
   `fofline-spa://localhost/index.html?eliteewFoflineClient=1` string.
   The HUD **Launcher** control (and F9/Escape) returns to
   `empiier://shell/index.html`.
5. **Copy.** Seat-facing strings live in `shell-ui/src/secure/locale/`.
   Primary copy must not claim Tauri, a pack picker, a second window, or
   product-ready. Lab jargon stays in the spoiler.

## Stage map

| Stage | Rail focus | Primary action | Runtime |
|-------|------------|----------------|---------|
| `offline` | none | none | `launcher.status` missing |
| `pin_missing` | Unlock (blocked) | none | installer did not plant PIN |
| `pin_locked` | Unlock (blocked) | none | five failures / lockout |
| `pin` | Unlock | `launcher.pin_unlock` | 4-digit PIN + `fp_v2` |
| `consume` | Activate | `launcher.air_activate` **or** `launcher.capsule_import` (two buttons, one step) | Capsule v3 / EWAE1 |
| `channel_invalid` | Activate (blocked) | none | stored portal conf fails validate |
| `portal` | Portal | `launcher.channel_up` | `wg-portal` + host-map |
| `spa` | Workspace | `launcher.spa_sync` | signed pack on VIP |
| `login` | Sign-in | `launcher.oidc_start` then `shell.auth_surface` | native PAR |
| `ready` | Open | `product.open` then `shell.product_surface` | local pack + OIDC session |

Display gating only. `launcherStage()` must not invent a second FSM.

## Invariants

1. **No `autoOpened`.** Product never opens itself. Portal may one-shot
   `channel_up` only while the stage is `portal` (`mayAutoBringUpPortal`).
2. **No `launcher.pin_issue` from Shell.** Production IPC is
   `installer_issued_only`. See [`FIRST-BOOT-PIN.md`](FIRST-BOOT-PIN.md).
3. **Unlock does not open the product.** Consume, portal, pack, and OIDC
   stay after unlock. See [`PRODUCT-OPEN.md`](PRODUCT-OPEN.md).
4. **Remote SPA denied.** `connected_cloud` is `remote_spa_forbidden`.
5. **Secrets stay in Runtime.** PIN plaintext, capsule payload, WG conf,
   and refresh tokens never enter Shell IPC or locale strings.
6. **Vehicle honesty.** `launcher.status` reports
   `vehicle=empiieros-shell-gui` and `tauri=false`.
7. **Wall clock before air / OIDC.** Junk RTC years fail closed
   (`wall_clock_required`). Operator sets the seat with `time.sync`
   `action=set` before Activate or Sign-in.
8. **Portal leaves host-map applied.** `channel_up` must report
   `portal_hostmap=applied`. A white Keycloak card after a successful
   PAR is a host-map / WebView failure, not “Keycloak is down.”
   Full canons: [`SEAT-FOFLINE-CANONS.md`](SEAT-FOFLINE-CANONS.md).

## Related

- PIN: [`FIRST-BOOT-PIN.md`](FIRST-BOOT-PIN.md)
- Portal / host-map: [`PORTAL-TUNNEL.md`](PORTAL-TUNNEL.md), [`PORTAL-HOSTMAP.md`](PORTAL-HOSTMAP.md)
- Pack: [`SPA-PACK.md`](SPA-PACK.md)
- Open: [`PRODUCT-OPEN.md`](PRODUCT-OPEN.md)
- Glass map: [`SEAT-FACE.md`](SEAT-FACE.md)
- Kiosk HUD / auth card: [`SHELL-GUI.md`](SHELL-GUI.md)
