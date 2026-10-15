# EmpiierOS — Project tree

**Status:** Living layout contract during the secure reset (`NOT_READY`).
**Date:** 2026-09-21
**Normative for layout:** this file.

---

## 1. Design principles

1. **Planes match axis:** `shell-ui/` (FoFline) · `shell-linkorn/` (LinkornEW staff face) · `runtime/` · `session/` — not a root junkyard.
2. **Meta separate:** `.cursor/` = agent OS; `docs/` = human SoT (`docs/capture/` archive, `docs/foundation/` plan pack, `docs/engineering-bar/` registry coding bar).  
3. **Empty ≠ missing:** product dirs exist with `README.md` stating purpose + “no product code until preflight”.  
4. **Anti-DEWA:** no `api/`, `stack0/`, `mobile/`, `keycloak/` mirrors.  
5. **Kinship stubs only:** pointers to monorepo paths — never vendored monorepo trees.  
6. **Trash local:** `trash/` for OS-repo soft deletes (policy), gitignored contents except README.

---

## 2. Target tree

```text
EmpiierOS/                          # workspace root (= future git root)
├── README.md                       # product-line entry (OS axis + kinship + no UI until preflight)
├── START-HERE.md                   # agent handoff (short); points to docs/capture + docs/foundation
├── LICENSE                         # operator lock later
│
├── shell-ui/                       # L-shell — Vite/React FoFline OS shell + ORIGIN.md
├── shell-linkorn/                  # LinkornEW staff face. Not packed by the FoFline image script.
│   ├── README.md
│   ├── ORIGIN.md
│   ├── package.json
│   ├── public/product-chrome/      # HUD + host media; Start logo harvest; HUD SoT stays runtime/
│   └── src/
│       ├── desktop/                # Seat launcher, host tiles, icon layout (pack catalog in FoFline)
│       └── secure/                 # native transport, snapshot hook, panels, locale
│           ├── components/         # ProductHost, Taskbar/TaskbarTray, WorkspaceFrame, panels
│           └── design/             # EliteEW Desktop token harvest + OS control primitives
│
├── runtime/                        # L-bridge — authenticated local IPC + real substrate operations
│   ├── README.md
│   ├── HEALTH-IPC.md
│   ├── Cargo.toml
│   ├── src/{core.rs,daemon.rs,protocol.rs,state_store.rs,command_runner.rs}
│   ├── src/{capsule_v3.rs,device_request.rs,https_vip.rs,https_auth.rs,launcher_air.rs,launcher_capsule.rs,launcher_inner.rs,launcher_oidc.rs,launcher_ota.rs,launcher_pin.rs,launcher_portal.rs,launcher_product.rs,launcher_spa.rs,launcher_status.rs,oidc_dpop.rs,oidc_pkce.rs,oidc_urls.rs,ota_capsule_store.rs,ota_cutover.rs,portal_hostmap.rs,spa_pack.rs,spa_assets.rs,wg_conf.rs}
│   ├── src/{system_probe.rs,network_control.rs,wifi_control.rs,power_control.rs,observed_domains.rs,event_log.rs}
│   ├── src/{display_control.rs,audio_control.rs,radio_control.rs,time_control.rs}
│   ├── src/bin/{air_enroll_helper.rs,portal_up_helper.rs,spa_fetch_helper.rs,ota_fetch_helper.rs}
│   ├── helpers/{empiieros-audio-volume.c,hda_speaker_amp.c,hda_speaker_amp.h,empiieros-audio-volume.service,empiieros-audio-amp.service,empiieros-wifi-radio,empiieros-wifi-restore,empiieros-time-sync,empiieros-air-enroll.service,empiieros-portal-up.service,empiieros-spa-fetch.service,empiieros-ota-fetch.service,ota-cutover.conf}
│   ├── helpers/seat-shield/        # nft + fw oneshot + watch unit (H5 jail)
│   ├── seat_shield_core/           # Seat Shield crate + H4 parser_probe / tests/parser_fuzz / ota_observe
│   ├── seat_shield_probe/          # inet_diag dump helper (P3; no eBPF/AF_PACKET)
│   └── shell-gui/{secure_main.c,native_bridge.c,native_bridge.h,boot_splash.c,boot_splash.h,product_chrome.c,auth_surface.c,auth_bounds.c,auth_bounds.h,seat_browser_surface.c,meet_surface.c,meet_bounds.c,meet_bounds.h,fofline_api_proxy.c,fofline_api_proxy.h,product-chrome/}
│
├── session/                        # L-session — living glass/install/host contracts only
│   ├── README.md
│   ├── CONTRACT.md
│   ├── SEAT-FACE.md
│   ├── SEAT-LAUNCHER.md
│   ├── SEAT-FOFLINE-CANONS.md      # metal-proven FoFline seat canons (2026-09-21)
│   ├── SHELL-GUI.md
│   ├── SEAT-BROWSER.md
│   ├── SEAT-SHIELD.md
│   ├── SEAT-SHIELD-FW.md
│   ├── FIRST-BOOT-PIN.md
│   ├── OS-INSTALL.md
│   ├── PRODUCT-OPEN.md
│   ├── SPA-PACK.md                 # bootstrap FoFline pack (pre-OIDC; ≠ capsule OTA)
│   ├── OTA-CAPSULE.md              # capsule-gated OTA (launcher.ota_*; P0 lock)
│   ├── AIR-HANDSHAKE.md
│   ├── PORTAL-TUNNEL.md
│   ├── PORTAL-HOSTMAP.md
│   ├── AUDIO-HARDWARE.md
│   ├── NETWORK-MANAGER.md
│   └── bootstrap/BOOTSTRAP-PHASE.md
│
├── docs/                           # Human documentation SoT for this repo
│   ├── AGENT-LESSONS.md            # F1 exit (promote from 05)
│   ├── KINSHIP.md                  # monorepo + genEW read-only map
│   ├── NORTH-STAR.md               # short axis + twin risks + maturity criterion
│   ├── WHERE-OS-LIVES.md           # one-pager: layout + non-goals + seed + kinship
│   ├── OS-SHELL-V0-SOT.md          # Shell v0 planning SoT (text; no product code)
│   ├── FOUNDATION-STATUS.md        # accept + green/not-yet + gate
│   ├── capture/                    # 00–13 + SESSION-* (thinking archive)
│   │   └── README.md
│   ├── foundation/                 # plan pack ONLY (no root foundation/)
│   │   ├── FOUNDATION-MASTER-PLAN.md   # historical F0–F10 record
│   │   ├── PROJECT-TREE.md
│   │   ├── AGENT-KERNEL-SPEC.md
│   │   ├── AGENT-KERNEL-BRIEF.md
│   │   ├── LANGUAGE-BRIDGE.md          # Language bridge — disabled
│   │   ├── PRODUCT-UNLOCK.md
│   │   ├── SUBSTRATE-KINSHIP.md
│   │   ├── SECURITY-PRIVACY-DEFAULTS.md
│   │   ├── RELEASE-SIGNING-STUBS.md
│   │   ├── GIT-AND-IGNORE-SPEC.md
│   │   └── FILE-SIZE-AND-MODULARITY.md
│   ├── seed/                       # F6 — policy only
│   │   ├── SEED-POLICY.md
│   │   └── ORIGIN.template.md
│   └── plans/                      # living ledger + current scoped work only
│       ├── README.md
│       ├── MATURITY-LEDGER.md      # living capability status (NOT_READY)
│       ├── METAL-LIVE-BASELINE.md  # durable live-seat snapshot; not readiness
│       ├── SECURE-RESET-LIVE-DIARY.md  # chronological lab log; not readiness
│       ├── PRODUCT-WAVE-FOFLINE-LICENSE-INSTALL.md
│       ├── PRODUCT-WAVE-ELITEEW-HOST.md
│       ├── PRODUCT-WAVE-SEAT-CHROME-PANEL.md
│       ├── PRODUCT-WAVE-SEAT-BROWSER-MVP.md
│       ├── PRODUCT-WAVE-FOFLINE-CONTENT-AUDIO.md
│       ├── PRODUCT-WAVE-AUDIO-SPEAKER-AMP.md
│       ├── TZ-DISK-AT-REST.md
│       ├── TZ-SEAT-SHIELD-POST-S4.md
│       ├── TZ-SEAT-SHIELD-S5C.md
│       ├── TZ-SEAT-SHIELD-HARDENING.md
│       ├── TZ-SEAT-SHIELD-H6-PRIVATETMP.md
│       ├── TZ-SEAT-SHIELD-SPINE.md
│       ├── TZ-CAPSULE-OTA.md
│       ├── TZ-EMPIERO-OS-OTA-EDGE.md
│       ├── OTA-OPERATOR-RUNBOOK.md
│       ├── ELITEEW-TZ-OTA-PK-IN-AIR-LICENSE.md
│       ├── SEAT-SHIELD-H3-AUDIT.md
│       ├── ELITEEW-TZ-MEET-ON-OS.md
│       ├── ELITEEW-TZ-FOFLINE-MEET-BOUNDS.md
│       ├── ELITEEW-TZ-FOFLINE-IN-WINDOW-PACK.md
│       ├── ELITEEW-TZ-FOFLINE-AUDIO-ON-OS.md
│       ├── ELITEEW-TZ-FOFLINE-SEAT-BROWSER.md
│       ├── FOFLINE-OS-BRIEF.md
│       └── ELITEEW-KINSHIP-FOFLINE-OS-SHELL.md
│
├── .cargo/config.toml              # H2 gnu rustflags (PIC/RELRO/NX); not musl
│
├── scripts/
│   ├── preflight-foundation.sh     # semantic executable gate
│   ├── trash-move.sh               # soft-delete helper (F4)
│   ├── hooks-selftest.sh           # automated hooks verification
│   ├── runtime-health-smoke.sh     # IPC health smoke test
│   ├── bootstrap-phase-smoke.sh    # Shell transport/polling smoke test
│   ├── package-shell-gui-rootfs.sh # deterministic rootfs payload packaging
│   ├── metal-live.sh               # pinned lab probe / Runtime+Shell deploy / rollback
│   ├── runtime-cargo-audit.sh      # H3 supply-chain report (not a GPL-fail gate)
│   ├── build-wireguard-from-source.sh # factory in-tree WG module + userspace wg
│   ├── empiieros-ota-cp-ingest.py  # E0c/E1b: pack/sign/publish verify→podman tmpfs (no host bind)
│   ├── empiieros-ota-operator-key.sh # living CP artifact SK/PK (XDG, never git)
│   ├── empiieros-ota-plant-seat-pk.py # lab fixture plant only — not installer
│   ├── empiieros-installer-media-unplant-ota.sh # live USB: trash ESP crutches + stamp apply
│   ├── empiieros-installer-media-plant-runtime.sh # mux SoT + live apply Runtime (not official out/)
│   └── empiieros-ota-e3-stage-measure.sh # E3 lab: live edge fetch + plant PK → stage
│
├── .cursor/                        # Agent kernel — see AGENT-KERNEL-SPEC
│   ├── README.md
│   ├── hooks.json
│   ├── hooks/
│   ├── rules/
│   ├── skills/                     # 13 reusable workflows
│   ├── agents/                     # 5 role agents
│   └── orchestration/              # no empty commands/ directory
│       ├── SYSTEM_BLUEPRINT.md     # stub
│       ├── lifecycle.json          # stub
│       ├── designs/                # .gitkeep
│       ├── reports/                # .gitkeep
│       └── audit/                  # gitignored jsonl; keep .gitkeep
│
├── trash/                          # local soft-delete landing (gitignore *)
│   └── README.md
│
├── .gitignore
├── .cursorignore
└── .cursorindexingignore           # recommended
```

**Not in tree (forbidden day-1):**

- `pack-cloud/`, `pack-organization/` mirrors  
- `node_modules/`, build outs (ignored)  
- Full monorepo checkout / submodule as UI SoT  
- `connection-agent/` copy  

---

## 3. Folder owners (accountability)

| Path | Owns | Must not |
|------|------|----------|
| `shell-ui/` | FoFline OS Shell UI source | Ops/privileged execution; Tauri host; LinkornEW face |
| `shell-linkorn/` | LinkornEW staff Shell source | FoFline license journey; vendored Chromium or Command Post |
| `runtime/` | First-party bridge / IPC | Browser UI; Substrate BR factory |
| `session/` | Seat/session **contracts** & notes | Full Trusted Substrate tree |
| `docs/` | Canon for humans + capture archive | Executable product code |
| `.cursor/` | Agent law & automation | Product business logic |
| `scripts/` | Preflight, packaging, smoke, trash, and bounded operator-approved metal-lab tooling | Unpinned remote access; release deployment or USB writes without an explicit gate |
| `trash/` | Soft-deleted paths | Secrets (scrub); never `rm -rf` culture |
| `docs/seed/` | Copy policy | Actual bulk FE copies without ORIGIN |

---

## 4. Root hygiene

**Allowed at root:** README, START-HERE, LICENSE, ignore files, plane dirs (`shell-ui/`, `shell-linkorn/`, `runtime/`, `session/`), `docs/`, `scripts/`, `.cursor/`, `trash/`.

**Not at root:** capture files (`00`–`13`, SESSION-*) — they live in `docs/capture/`; plan pack — only in `docs/foundation/` (no root `foundation/`).

---

## 5. Layout status (after Task 1–3)

1. Core plane and metadata directories exist.
2. Historical wave sources may remain on disk but are not active Runtime authority
   unless referenced by `runtime/Cargo.toml`.
3. Capture remains archival under `docs/capture/`. Historical P0–P92
   `PRODUCT-WAVE-*` and P62–P92 session contract files are in `trash/`.
   Living status and maturity documents supersede historical completion
   prose.
4. The Shell uses the native bridge; raw Tauri commands and browser-supplied
   privileged URLs remain forbidden.
5. The Trusted Substrate factory remains in its monorepo kinship path and is
   changed only under an explicit operator scope.

---

## 6. Naming (operator lock still open)

Provisional dir names above match capture (`shell-ui` / `runtime` / `session`).  
Repo display name: **EmpiierOS** / product line **EliteEW OS** — final git remote name TBD.
