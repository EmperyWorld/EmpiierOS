# ORIGIN — shell-ui provenance

## Source

- Kinship path (sibling of this repo): `../powered_by_emperyworld/shared/web/frontend/src/pages/Desktop/constants/desktopDesignTokens.ts`
- Kinship path (sibling of this repo): `../powered_by_emperyworld/shared/web/frontend/src/theme/designTokens.ts`
- Kinship path (sibling of this repo): `../powered_by_emperyworld/pack-organization/connection-agent/src/styles/tokens.css`
- Kinship path (sibling of this repo): `../powered_by_emperyworld/pack-organization/connection-agent/src/features/chrome/chrome.css`
- Kinship path (sibling of this repo): `../powered_by_emperyworld/shared/web/frontend/src/pages/Desktop/shell/foflineConnectedSurface.ts`
- Kinship path (sibling of this repo): `../powered_by_emperyworld/shared/apps/_profiles/fofline-connected.yaml`
- Kinship path (sibling of this repo): `../powered_by_emperyworld/shared/web/frontend/src/pages/Desktop/constants/defaultApps.ts`
- Kinship path (sibling of this repo): `../powered_by_emperyworld/pack-organization/connection-agent/src/styles/base.css` (panel/eyebrow/lead/body slice only)
- Kinship path (sibling of this repo): `../powered_by_emperyworld/pack-organization/connection-agent/src/lib/launcher/journey/journeyPhase.ts`
- Upstream commit / date (if known): chrome seed `2ab70ed9b`; FSM helpers `12cfefd20` (monorepo HEAD at FSM wave)
- Operator task id / date: Wave B chrome 2026-09-13 · Bootstrap FSM 2026-09-13

## Copied

- Paths / globs copied:
  - EliteEW Desktop `--desktop-*` tokens → `shell-ui/src/theme/tokens.css` and `shell-ui/src/secure/design/`
  - historical `tokens.css` gold aliases retired in favor of Desktop accent RGB
  - `tokens.css` → `shell-ui/src/theme/tokens.css`
  - thin slice of `chrome.css` (hero + shell-stage) → `shell-ui/src/chrome/chrome.css`
  - thin slice of `base.css` (atmosphere + panel/eyebrow/lead) → folded into `chrome.css`
  - `journeyPhase.ts` pure helpers → `shell-ui/src/session/bootstrapPhase.ts`
  - `HubShell.tsx` & `hub.css` tab rail & ARIA keyboard navigation → `shell-ui/src/workspace/WorkspaceShell.tsx` & `shell-ui/src/workspace/workspace.css`
  - `DesktopWorkspaceShellLayout.tsx` structure only (canvas + taskbar, no windows/apps) → `shell-ui/src/secure/components/ProductHost.tsx`, `Taskbar.tsx`, and `secure.css` desktop stage
  - `foflineConnectedSurface.ts` + `fofline-connected.yaml` allowlist → `shell-ui/src/desktop/foflineConnectedSurface.ts`
  - FoFline catalog names from `defaultApps.ts` / yaml extras → `shell-ui/src/desktop/foflineCatalog.ts` + locale `foflineApps`
  - FoFline Start/catalog canvas (icons + start + local tiles) → `shell-ui/src/desktop/FoflineDesktop.tsx` + `fofline.css`
  - Host canvas now filters pack-kind tiles (`foflineHostTiles`); pack apps open only after `product.open`
  - workspace navigation pure helpers → `shell-ui/src/session/workspaceNavigation.ts`
  - `LauncherScreen.tsx` & `EliteewTab.tsx` application launcher card & payload lifecycle → `shell-ui/src/session/payloadPackaging.ts` & `shell-ui/src/App.tsx`
  - `flash-live-installer-usb.sh` block device safety checks & live flash layout → `shell-ui/src/session/substrateImage.ts` & `shell-ui/src/App.tsx`
  - `eliteew-ts-session-router` & `disable-kiosk` break-glass recovery hook → `shell-ui/src/session/substrateMetal.ts` & `shell-ui/src/App.tsx`
  - `eliteew-ts-kiosk` & `resilience` watchdog and crash supervision → `shell-ui/src/session/selfHealing.ts` & `shell-ui/src/App.tsx`
  - `eliteew-ts-kiosk` & `linux/watchdog.h` hardware watchdog ping and magic close → `shell-ui/src/session/watchdogHw.ts` & `shell-ui/src/App.tsx`
  - `eliteew-ts-kiosk` & WebKitGTK Wayland kiosk runner → `shell-ui/src/session/shellGui.ts` & `shell-ui/src/App.tsx`
- Why each path is on the allowlist: SEED-POLICY allows thin chrome/tokens, journey FSM **helpers**, primary workspace navigation shell, typed application payload launchers, substrate image flash controls, metal boot/recovery management, self-healing watchdogs, hardware watchdog daemon controls, and native Wayland kiosk runner (not wholesale connection-agent).

## Stripped

- What was removed: step-rail / LauncherHero / StepRail; Welcome/Activation/Hub product journeys; Tauri ping/entitlement invoke; form zoo; Admin purple tokens; secrets/env; browser window APIs; raw arbitrary URL openers; unconfined sudo shell commands; unconfined bash restart loops; raw uncoordinated writes to `/dev/watchdog`
- What was rewritten for OS Shell + Runtime (summary): `--dc-*` → `--os-*`; boot signals = Runtime `health` + `seat.status`; phases `identifying|system|boot_error`; enroll surface gated off while Identifying; workspace navigation backed by `seat.workspace.*` runtime loopback IPC rather than ephemeral Tauri window commands; application launch backed by `product.open` and `product.payload.*` runtime loopback IPC rather than Tauri commands or external browser launches; substrate image packaging, dual-flag exclusivity verification, and flash plan staging backed by `substrate.image.*` loopback IPC instead of unconfined root bash scripts; bare-metal boot reference validation, kernel command-line compliance, atomic dual-flag profile switching, and break-glass recovery backed by `substrate.metal.*` loopback IPC; automated watchdog heartbeats, compositor supervision, circuit breaker isolation, and state quarantine backed by `substrate.self_healing.*` loopback IPC; hardware watchdog driver telemetry, sysfs attributes, periodic keepalive feeds, and safe magic-close disarm backed by `substrate.watchdog_hw.*` loopback IPC

## Notes

- Conscious duplication? **yes** — no shared package with monorepo until Shell contracts stabilize.
- Follow-up debt: Welcome/Activation when enroll op exists; Identifying phase mirrored into Runtime `seat.status`; fuller Hub IA; P92 live OTA & A/B partition cutover.
