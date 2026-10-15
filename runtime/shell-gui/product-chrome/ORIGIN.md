# Product chrome media

**Plane:** native Shell GUI. **Not:** Tauri. **Not:** remote SPA.

## Source

Local ComfyUI `http://192.168.1.136:31003` (also `127.0.0.1:31003`,
container `comfyui`, RTX 3090). Flux.2 Klein 4B.

2026-09-21 premium outline regen: gold glyph, thick black stroke,
cyan rim. White mat knocked out before 64px lanczos.

| File | Prompt id | Model |
|------|-----------|--------|
| `launcher-src.png` | `091689ea-38fb-4fff-8ed4-2dbbf88d124c` | Flux.2 Klein 4B |
| `eliteew-src.png` | `b2ba16aa-d83a-4ef7-a6ee-0c2ab92792f4` | Flux.2 Klein 4B |
| `hud-panel-src.png` | `4701f299-f098-4702-bbcb-18b6d5b25297` | Flux.2 Klein 4B |
| `hud-shield-src.png` | `ea5d9c3e-8df6-44c4-8cc8-f61aace44a7b` | Flux.2 Klein 4B |
| `hud-shield-alert-src.png` | `f09cee45-34d7-46e6-867b-9b35e752b546` | Flux.2 Klein 4B |
| `hud-ota-src.png` | `9006a267-a359-4e09-adeb-f9ca208fe5bf` | Flux.2 Klein 4B |
| `hud-volume-src.png` | `b2a949f9-eeda-4fc5-b51f-f288421c0c6f` | Flux.2 Klein 4B |
| `panel-mic-src.png` | `544f4e34-de75-453e-9513-99dc2323f3ad` | Flux.2 Klein 4B |
| `panel-cam-src.png` | `d03c8231-0923-4c66-97fa-b4932f48b819` | Flux.2 Klein 4B |
| `panel-gps-src.png` | `6addaf2d-bd2b-4cf8-b243-48dbf873ba7d` | Flux.2 Klein 4B |
| `click-launcher.wav` | `12a180a8-cb04-4c60-af55-acc61d7460ef` | Ace-Step v1 3.5B |
| `click-eliteew.wav` | `60c42a33-18d7-482f-bd30-a5d313a35176` | Ace-Step v1 3.5B |

64px tiles are lanczos scales of the source stills. HUD cluster is
icons only except Updates, which keeps a caption «Обновления».
Clicks are 22050 Hz mono PCM.
`click-eliteew.wav` keeps prompt `60c42a33…`; peak was 325 (inaudible
on ALSA) and is gain-normalized to the launcher peak (~11820).

`start_os.mp4` is the operator-supplied OS start clip (H.264+AAC,
8s, 12788340 bytes). Not ComfyUI. Played by `boot_splash.c` at
ALSA volume 0.25.

## Use

Native HUD loads `/opt/empiieros/shell/product-chrome/` first, then
`/usr/share/empiieros/product-chrome/`.
