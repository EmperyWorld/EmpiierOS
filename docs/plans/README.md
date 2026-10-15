# EmpiierOS plans

**Status:** Living index only.
**Current verdict:** `NOT_READY`.

Historical `PRODUCT-WAVE-*` / harvest maps (P0–P92) were implementation
records, not acceptance. The 2026-09-16 audit invalidated their `Done` /
`Wired` labels. Those files are soft-deleted under `trash/` (2026-09-21).
Current authority is the ledger and foundation status.

| Living authority | Role |
|------------------|------|
| [`MATURITY-LEDGER.md`](MATURITY-LEDGER.md) | Capability status |
| [`../FOUNDATION-STATUS.md`](../FOUNDATION-STATUS.md) | Gates + exclusions |
| [`METAL-LIVE-BASELINE.md`](METAL-LIVE-BASELINE.md) | Durable lab-seat snapshot (not readiness) |
| [`SECURE-RESET-LIVE-DIARY.md`](SECURE-RESET-LIVE-DIARY.md) | Chronological lab log (not readiness) |
| [`../../session/SEAT-FOFLINE-CANONS.md`](../../session/SEAT-FOFLINE-CANONS.md) | Metal-proven FoFline seat canons (clock, host-map, shell trust, OIDC, tty1-only console) |

Unlock SoT: [`../foundation/PRODUCT-UNLOCK.md`](../foundation/PRODUCT-UNLOCK.md).

## Current scoped work

| File | Role | Status |
|------|------|--------|
| [TZ-LINKORNEW.md](TZ-LINKORNEW.md) | Staff profile `linkornew` | **L8 USB** — flash `TRAN=usb RM=1` only. Not official `out/`. Not MATCH. |
| [PRODUCT-WAVE-FOFLINE-LICENSE-INSTALL.md](PRODUCT-WAVE-FOFLINE-LICENSE-INSTALL.md) | V2 install + launcher + PIN | **In progress** — official `out/` `5979e9cf…`; apply/Wi-Fi plant ahead. Not MATCH. |
| [TZ-DISK-AT-REST.md](TZ-DISK-AT-REST.md) | LUKS2 2 GiB state; PIN ≠ volume key | **In source + bake** — metal walk done. Not MATCH. |
| [TZ-STATE-LUKS-UNLOCK-AGENT.md](TZ-STATE-LUKS-UNLOCK-AGENT.md) | MIT console ask-password agent for state LUKS | **Phase 1 landed** — Phase 0 capture + QEMU boot pending rebake |
| [STATE-LUKS-ASK-CAPTURE.md](STATE-LUKS-ASK-CAPTURE.md) | Procedure to capture real `ask.*` for allowlist lock | Living procedure |
| [PRODUCT-WAVE-ELITEEW-HOST.md](PRODUCT-WAVE-ELITEEW-HOST.md) | Desktop canvas as launcher host | **Held** — catalog tiles were the wrong FoFline axis |
| [PRODUCT-WAVE-SEAT-CHROME-PANEL.md](PRODUCT-WAVE-SEAT-CHROME-PANEL.md) | Always-on HUD | **In source** |
| [PRODUCT-WAVE-SEAT-BROWSER-MVP.md](PRODUCT-WAVE-SEAT-BROWSER-MVP.md) | Seat Browser overlay | **In source** |
| [PRODUCT-WAVE-FOFLINE-CONTENT-AUDIO.md](PRODUCT-WAVE-FOFLINE-CONTENT-AUDIO.md) | FoFline content audio leftovers | **Lab muxed** |
| [PRODUCT-WAVE-AUDIO-SPEAKER-AMP.md](PRODUCT-WAVE-AUDIO-SPEAKER-AMP.md) | Dell ALC3204 speaker-amp GPIO | **In progress** — kernel quirk still next image |
| [TZ-SEAT-SHIELD-POST-S4.md](TZ-SEAT-SHIELD-POST-S4.md) | Seat Shield after S4 | **S5C in official bake** |
| [TZ-SEAT-SHIELD-S5C.md](TZ-SEAT-SHIELD-S5C.md) | Helper hash in watch journal | **muxed + baked** |
| [TZ-SEAT-SHIELD-HARDENING.md](TZ-SEAT-SHIELD-HARDENING.md) | Compiler / isolation / audit | H6 design locked; live `-SECCOMP` |
| [TZ-SEAT-SHIELD-H6-PRIVATETMP.md](TZ-SEAT-SHIELD-H6-PRIVATETMP.md) | H6 + Runtime/watch PrivateTmp split | **Design lock** — full H6 after reinstall |
| [TZ-SEAT-SHIELD-SPINE.md](TZ-SEAT-SHIELD-SPINE.md) | Journal · Connections · heuristics · probe | **P1–P3 local Measure** — Not MATCH |
| [TZ-CAPSULE-OTA.md](TZ-CAPSULE-OTA.md) | Capsule-gated OS + FoFline OTA (`launcher.ota_*`) | **P0–P4 + E4 cutover plant** — local Measure; Not MATCH |
| [TZ-EMPIERO-OS-OTA-EDGE.md](TZ-EMPIERO-OS-OTA-EDGE.md) | EmpiierOS OTA edge (RO+tmpfs+CP) | **E0–E4 Measure** — cutover + baseline planted; Not MATCH |
| [OTA-OPERATOR-RUNBOOK.md](OTA-OPERATOR-RUNBOOK.md) | How to sign, publish, and apply a capsule | **Living operator SoT** — not MATCH |
| [SEAT-SHIELD-H3-AUDIT.md](SEAT-SHIELD-H3-AUDIT.md) | cargo-audit report | Host evidence only |
| [ELITEEW-TZ-MEET-ON-OS.md](ELITEEW-TZ-MEET-ON-OS.md) | Meet WebRTC honesty | Glass is the operator's |
| [ELITEEW-TZ-FOFLINE-MEET-BOUNDS.md](ELITEEW-TZ-FOFLINE-MEET-BOUNDS.md) | Sibling Meet + CSS bounds | **On disk** — no iframe |
| [ELITEEW-TZ-FOFLINE-IN-WINDOW-PACK.md](ELITEEW-TZ-FOFLINE-IN-WINDOW-PACK.md) | Old iframe pack | **On disk** — do not rebake |
| [ELITEEW-TZ-FOFLINE-AUDIO-ON-OS.md](ELITEEW-TZ-FOFLINE-AUDIO-ON-OS.md) | FoFline sound TZ | EliteEW bake |
| [ELITEEW-TZ-FOFLINE-SEAT-BROWSER.md](ELITEEW-TZ-FOFLINE-SEAT-BROWSER.md) | Pack tile → OS Seat Browser | **TZ only** |
| [ELITEEW-TZ-OTA-PK-IN-AIR-LICENSE.md](ELITEEW-TZ-OTA-PK-IN-AIR-LICENSE.md) | Air entitlement carries OTA verify PK | **Issuer+SOPS+API done**; seat PK walked on mux; official `out/` still stale |
| [FOFLINE-OS-BRIEF.md](FOFLINE-OS-BRIEF.md) | FoFline-on-OS harvest map | **Brief** — not acceptance |
| [ELITEEW-KINSHIP-FOFLINE-OS-SHELL.md](ELITEEW-KINSHIP-FOFLINE-OS-SHELL.md) | Kinship prompt (write home = monorepo) | Not EmpiierOS acceptance |

## Frozen (not on disk as wave files)

P93 A/B rootfs and P94 offline crypto store stay frozen until image and
metal acceptance. Do not seed them from trash.
