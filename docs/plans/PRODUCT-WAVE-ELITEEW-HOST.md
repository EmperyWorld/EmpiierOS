# Product Wave: EliteEW / FoFline host surface

**Wave:** EliteEW host (scoped, 2026-09-17)  
**Status:** Held — canvas stays as the activator host. Catalog tiles were the wrong FoFline axis. See [`PRODUCT-WAVE-FOFLINE-LICENSE-INSTALL.md`](PRODUCT-WAVE-FOFLINE-LICENSE-INSTALL.md).  
**Unlock:** Operator session asked to reach EliteEW Desktop integration on the live laboratory seat, then oriented on FoFline as the almost-offline local product environment.  
**Not:** wholesale `shared/web/frontend/src/pages/Desktop` copy. **Not:** `product.open` becoming available. **Not:** `edition.profile=fofline`. **Not:** air-gap.

## Intake

```text
Goal: production OS Shell hosts the FoFline connected catalog; local tiles use observed OS ops; pack apps stay pack_missing
Planes: shell-ui | docs
Boundaries: EmpiierOS write-only; kinship read-only; no USB; no commit
Risk: medium (live Shell deploy)
Edge: regression of system/network/diagnostics; skin (fake launch)
Proof: typecheck/build, live snapshot product.open=unavailable, live product.open probe not_available, catalog visible without claiming fofline profile
```

## Cause

FoFline already has a scoped almost-offline environment (`fofline-connected` allowlist). The production Shell had only an empty host contract card, so the seat could not show that catalog. Cloud-full Desktop and a fake `product.open` would be the wrong copy.

## Lock

1. Production workspaces stay `eliteew` (default), `system`, `network`, `diagnostics`.
2. `ProductHost` mounts `FoflineDesktop`: catalog icons, Start menu, local tiles.
3. Local tiles: Notifications = Runtime events; MusicPlayer = ALSA `audio.control`; ProjectAbout = seat facts + fail-closed probe; AppStore = catalog list.
4. Remaining FoFline apps render `pack_missing`. They do not launch a payload.
5. Snapshot still publishes `product.open` as `unavailable` / `not_implemented`.
6. The About probe calls production `product.open` and must stay fail-closed.
7. Do not write `edition.profile=fofline`.

## Measure

- `npm run build --prefix shell-ui` exit 0
- `scripts/bootstrap-phase-smoke.sh` exit 0
- Live Shell `20260917T103215Z-shell-7864` (`index-CiqUXXU1.js`)
- Live frontend snapshot includes `product.open` `unavailable` / `not_implemented`
- Live About probe returns `not_available` / `capability_not_available`
- `eno1` stays `192.168.8.10/30`

## Explicit DROP

- Wholesale Desktop / `connection-agent` / cloud SPA seed
- Enabling `product.open`, payload install, or cloud launch
- Fake fofline edition profile
- USB flash / commit / claiming `verified-real` or product Done
