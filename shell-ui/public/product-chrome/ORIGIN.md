# Product chrome + host desktop media

**Plane:** OS Shell (`shell-ui/public`) and native HUD copies.
**Not:** Tauri. **Not:** EliteEW pack apps.

## Source

Local ComfyUI `http://127.0.0.1:31003` (container `comfyui`, RTX 3090).
2026-09-21 premium outline regen: Flux.2 Klein 4B, gold glyph + cyan rim,
white mat knocked out. Associative glyphs (no map-pin collapse except GPS).

| File | Prompt id | Model |
|------|-----------|--------|
| `launcher.png` (HUD + host) | `091689ea-38fb-4fff-8ed4-2dbbf88d124c` | Flux.2 Klein 4B |
| `eliteew.png` | `b2ba16aa-d83a-4ef7-a6ee-0c2ab92792f4` | Flux.2 Klein 4B |
| `click-launcher.wav` | `12a180a8-cb04-4c60-af55-acc61d7460ef` | Ace-Step v1 3.5B |
| `click-eliteew.wav` | `60c42a33-18d7-482f-bd30-a5d313a35176` | Ace-Step v1 3.5B |
| `host-wallpaper.jpg` | `986dc62e-e896-46e5-8299-570aa2377f4e` | Flux.2 Klein 4B |
| `host-shield.png` | `ea5d9c3e-8df6-44c4-8cc8-f61aace44a7b` | Flux.2 Klein 4B |
| `host-system.png` | `3c12899b-c39a-45f7-9e0a-527fb3335a33` | Flux.2 Klein 4B |
| `host-network.png` | `6e0e704d-c35d-40b8-b893-940e7a947a92` | Flux.2 Klein 4B |
| `host-diag.png` | `68939eb7-c840-40f3-9dfb-9daf7ed6c167` | Flux.2 Klein 4B |
| `host-audio.png` | `b2a949f9-eeda-4fc5-b51f-f288421c0c6f` | Flux.2 Klein 4B |
| `host-events.png` | `f86b3ff0-0b04-470e-b10d-3100af1d5350` | Flux.2 Klein 4B |
| `host-browser.png` | `6ccfe3ad-b733-4f80-aa85-099c77dca865` | Flux.2 Klein 4B |
| `host-ota.png` | `9006a267-a359-4e09-adeb-f9ca208fe5bf` | Flux.2 Klein 4B |
| `tray-volume.png` | `b2a949f9-eeda-4fc5-b51f-f288421c0c6f` | Flux.2 Klein 4B |
| `tray-battery.png` | `9b63f49f-1dec-422e-ab72-64efb1f9527a` | Flux.2 Klein 4B |
| `tray-network.png` | `6e0e704d-c35d-40b8-b893-940e7a947a92` | Flux.2 Klein 4B |
| `hud-panel.png` | `4701f299-f098-4702-bbcb-18b6d5b25297` | Flux.2 Klein 4B |
| `hud-shield.png` | `ea5d9c3e-8df6-44c4-8cc8-f61aace44a7b` | Flux.2 Klein 4B |
| `hud-shield-alert.png` | `f09cee45-34d7-46e6-867b-9b35e752b546` | Flux.2 Klein 4B |
| `hud-ota.png` | `9006a267-a359-4e09-adeb-f9ca208fe5bf` | Flux.2 Klein 4B |
| `hud-volume.png` | `b2a949f9-eeda-4fc5-b51f-f288421c0c6f` | Flux.2 Klein 4B |
| `panel-mic.png` | `544f4e34-de75-453e-9513-99dc2323f3ad` | Flux.2 Klein 4B |
| `panel-cam.png` | `d03c8231-0923-4c66-97fa-b4932f48b819` | Flux.2 Klein 4B |
| `panel-gps.png` | `6addaf2d-bd2b-4cf8-b243-48dbf873ba7d` | Flux.2 Klein 4B |

Native HUD SoT remains `runtime/shell-gui/product-chrome/`.
Host wallpaper/icons are Shell-only.

`emperyworld-icon.svg` is a thin allowlisted harvest of
`../powered_by_emperyworld/shared/web/frontend/public/emperyworld-icon.svg`
for the Start button. Not a pack app and not a second Desktop tree.

## Use

Vite copies this directory into `dist/product-chrome/`.
