# Installer UX captures (2026-09-21)

Reference PNGs for redesign planning. **Not** acceptance evidence.

| File | Step |
|------|------|
| `01-welcome.png` | Welcome |
| `02-regional.png` | Язык, пояс, время |
| `03-network.png` | Сеть |
| `04-disk.png` | Диск |
| `05-state-key.png` | Ключ диска (LUKS recovery) |
| `06-confirm-erase.png` | ERASE confirm |
| `07-progress.png` | Progress (~62%) |
| `08-done-pin-recovery.png` | Done (sample PIN/recovery) |
| `09-failed.png` | Failed (sample error) |
| `10-installed-boot-luks-console.png` | **Mock** console LUKS prompt (not egui) |

**01–09:** real `eliteew-ts-installer` egui (1280×800), captured with
`EFRAME_SCREENSHOT_TO` + `ELITEEW_TS_INSTALLER_STEP` fixtures.
Regenerate: `bash scripts/capture-installer-ux-stages.sh`.

**10:** typography/colors aligned with `state-unlock-agent` banner; layout approximate.
