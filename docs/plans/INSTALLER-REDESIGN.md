# Installer UI redesign (FHD wizard shell)

**Status:** Implementation in Trusted Substrate `installer/` (egui), 2026-09-22. **Not** metal acceptance.

## Source of truth

| Artifact | Path |
|----------|------|
| Operator TZ + audit (RU) | [`docs/capture/installer-mockups-v2-20260922/AUDIT-TZ-empiieros-installer.md`](../capture/installer-mockups-v2-20260922/AUDIT-TZ-empiieros-installer.md) |
| Target mockups (1920×1080) | [`docs/capture/installer-mockups-v2-20260922/`](../capture/installer-mockups-v2-20260922/) |
| Before captures | [`docs/capture/installer-ux-stages-20260922/`](../capture/installer-ux-stages-20260922/) |
| Process canon | [`session/OS-INSTALL.md`](../../session/OS-INSTALL.md) |

## Implemented (code)

- Palette v2 **cyan/metal** (`installer/src/brand.rs`); matrix-green removed from widgets.
- **Wizard chrome:** topbar, 8-step rail, ~1040px main panel + ~380px aside; default viewport **1920×1080**.
- Subtle grid background; boot-splash glitch backdrop **off** unless `ELITEEW_TS_INSTALL_SPLASH_BACKDROP=1`.
- Copy/layout deltas: confirm title, ERASE summary grid, disk red callout, three-secrets aside, Done PIN vs LUKS cards, progress mini-log, failed log hint.
- Progress roadmap: five phases only (no «Финал» artifact).
- LUKS console banner: cyan badge + clarified copy (`state-unlock-agent/src/tty.rs`).

## Measure

```bash
(cd "${ELITEEW_TS_ROOT:-../powered_by_emperyworld/pack-organization/trusted-substrate}/installer" && cargo build --release)
bash scripts/capture-installer-ux-stages.sh docs/capture/installer-ux-stages-$(date -u +%Y%m%d)
```

Compare PNGs to mockups under `installer-mockups-v2-20260922/`. USB remux / metal install walk is operator scope.

## USB patch (2026-09-22, not full reflash)

Mounted live root `/dev/sda3` → `/run/media/owned/eliteew-ts` (`TRAN=usb` `RM=1`). Replaced only:

- `usr/local/bin/eliteew-ts-installer` (`3a850bdf…` → `20398a4c…`, GLIBC_2.39 / zig 2.41 ceiling)
- `opt/eliteew-ts-installer/build-manifest.json`

`rootfs.tar.gz` on the stick was **not** rewritten (installed system from apply still carries old installer until next factory image). Repeat:

```bash
bash scripts/empiieros-installer-media-plant-installer-gui.sh --root /run/media/owned/eliteew-ts
```
