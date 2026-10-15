# License layers

[ADR-004](adr/ADR-004-mit-public-source.md) adopts MIT for first-party code.

The map matches `LICENSE` and `NOTICE` in this repo. Do not invent a new layer.

| Component | License |
|-----------|---------|
| `shell-ui/`, main `empiieros-runtime`, `runtime/shell-gui/` | MIT. Copyright holder is named in LICENSE |
| `seat_shield_core`, `seat_browser_core`, `seat_shield_probe` | MIT (`license` in each Cargo.toml) |
| Linux kernel in the image | GPL-2.0-only WITH Linux-syscall-note. Not in this git tree. Not proprietary |
| Buildroot and its packages | per recipe, in the factory legal-info |
| Installer and state-unlock agent | MIT, in the Trusted Substrate tree, not relicensed here |

## Rules

- Do not copy GPL or LGPL sources into `shell-ui/` or the main runtime crate.
- Do not change the kernel license from this repo.
- A new statically linked GPL/AGPL crate needs an ADR before merge.
- Do not write that EmpiierOS is already in the Ministry registry.

Layer changes supersede [ADR-003](adr/ADR-003-license-layers.md) only with a newer ADR.
