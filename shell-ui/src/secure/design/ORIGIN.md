# ORIGIN — EmpiierOS Shell design center

Harvest ≠ restore. EliteEW Desktop tokens are the visual SoT; EmpiierOS owns
mechanics, IPC, and substrate policy.

## Source

- `../powered_by_emperyworld/shared/web/frontend/src/pages/Desktop/constants/desktopDesignTokens.ts`
- `../powered_by_emperyworld/shared/web/frontend/src/pages/Desktop/constants/desktopChromeTokens.ts`
- `../powered_by_emperyworld/shared/web/frontend/src/theme/designTokens.ts`
- `../powered_by_emperyworld/documentation/product/desktop/DESKTOP_DESIGN_SYSTEM.md`
- `../powered_by_emperyworld/shared/web/frontend/src/styles/design-tokens.css` (spacing / semantic accents only)

## Copied

- Desktop CSS variable names and default RGB values into
  `shell-ui/src/theme/tokens.css` and `shell-ui/src/secure/design/tokens.ts`.
- Interaction pattern: setting rows, accent RGB, surface/border/radius/shadow
  hierarchy, 8px spacing scale.
- Start-button mark: `public/product-chrome/emperyworld-icon.svg` from
  `shared/web/frontend/public/emperyworld-icon.svg`. Taskbar rail geometry
  from `TASKBAR.HEIGHT_PX` (46) and `HIT_MIN_CLASS` (34). Desktop icon snap
  from `DESKTOP_GRID.GRID_SIZE` (110). No Taskbar/DesktopIcon React tree.

## Stripped

- Tailwind class maps, Personalize runtime, paper-kraft preset, marketing pink
  cabinet palette, Admin New World purple, Tauri chrome, Lucide icon package.
- No wholesale Desktop/React tree copy.

## Merge hook

Later EliteEW merge may override `--desktop-*` (including `--desktop-accent-rgb`)
from product theme. EmpiierOS `--os-*` aliases must keep reading those variables
so Shell chrome follows the same center without rewriting panel mechanics.

## EmpiierOS-owned (not harvest)

- `HostAppShell.tsx` — shared host-window tabs / status chips / facts / panel
  composition (Seat Shield format). CSS primitives: `host-app-*` aliased with
  `seat-shield-*` in `secure/secure.css`.
