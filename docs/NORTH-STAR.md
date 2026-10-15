# EmpiierOS — North star

**Status:** Living short SoT (Updated 2026-09-21).
**Current verdict:** `NOT_READY` during the secure reset.
**Not:** generic kiosk or toy appliance. Living canon for the EliteEW Operating System.

---

## Who We Are & Why We Do This (Identity & Mission)

### Who We Are
We are the creators, architects, and custodians of **EliteEW** — a battle-tested, industrial-scale platform with over **2 million lines of real production code** (`powered_by_emperyworld`). We are not building demo apps or toy prototypes. We possess a powerful, feature-rich React frontend, deep business logic, and a sophisticated backend engine.

### Why We Do This (The Mission)
Historically, complex web applications are forced to live as tenants inside foreign operating systems, wrapped in third-party containers (Electron, Tauri) that treat our software as just another window on an alien desktop.

**EmpiierOS exists to give EliteEW its own sovereign home.**

We are elevating the EliteEW stack into a true, first-party, hardened Operating System:
1. **The React UI becomes the OS Shell:** The powerful EliteEW React frontend is not merely painting content inside a window. It is elevated to the **Session Face and Desktop Shell of the entire OS** (performing the role of KDE or GNOME, but designed exclusively for EliteEW workflows).
2. **The Operations become the EliteEW Runtime:** Privileged operations are centralized in a first-party Rust daemon (`runtime/`) behind authenticated local IPC, explicit authorization, bounded inputs, and auditable failures.
3. **The Floor becomes Trusted Substrate:** A tailored Linux foundation that boots directly into our environment. Security properties such as dm-verity, measured boot, or Secure Boot are claims only after the built image and target prove them.

---

## Core Philosophy & Guiding Laws

### 1. Anti-Distraction & Anti-Peripheral Drift (No Petty Toys)
EmpiierOS is **not** a generic point-of-sale cash register or retail kiosk. We do not get sidetracked by receipt printers, peripheral spoolers, or petty hardware diversions. Every line of engineering effort must serve the core mission: integrating the massive 2M-line EliteEW platform into a unified, secure operating system.

### 2. The Triad Axis (Locked)
```text
OS Shell (React 19)  +  EliteEW Runtime (Rust)  +  Linux Floor (Substrate)
```
- **Shell (`shell-ui/`):** React 19, TypeScript 7, Vite 8. It renders observed state and exposes only Runtime-backed controls.
- **Runtime (`runtime/`):** First-party Rust daemon over `/run/empiieros/runtime.sock`. The native Shell reaches it through `/run/empiieros-shell/runtime.sock` and a root-owned socket proxy; production operations default to disabled unless backed by real substrate behavior.
- **Floor (`session/` & Trusted Substrate):** Session contracts in this repository and the explicit Buildroot kinship scope in `powered_by_emperyworld`. Boot, account, service, kernel, and filesystem properties require image evidence.

Tauri in the monorepo is a living host in kinship — **not** the language of future plans here.

### 3. Maturity Criterion
Every mature OS capability must be implemented and verified across all three planes:
```text
shell surface  +  bridge op  +  substrate policy
```
Never accept an operation without a shell surface, and never accept a UI control without a verified runtime bridge operation and substrate policy.

### 4. Twin Risks (Must Hold Both)

| Risk | Failure Mode | Guard |
|------|--------------|-------|
| **Skin** | React merely paints a cosmetic facade while ops are smeared across unverified host scripts | Every UI interaction maps to authenticated, validated, fail-closed runtime bridge operations. |
| **GNOME-bloat** | Shell expands into a bloated, generic desktop environment with an app zoo | Keep strict focus on EliteEW journeys; thin window seat floor; typed application manifests only. |

### 5. Kinship Method
Do not break the living monorepo (`powered_by_emperyworld`). New work follows **copy → paste → adapt** in this repository with strict provenance recording in `ORIGIN.md`. We practice conscious duplication until OS contracts stabilize, preventing premature shared libraries and Dual SoT corruption.

---

## The Strategic Path Forward

1. **Secure-reset automated evidence — partial**
   - EmpiierOS preflight is green. Official-image QEMU smokes are **not
     current** (not rerun after the 2026-09-20 LUKS bake; prior H1 FAIL
     on the WebKit sandbox string still stands).
   - Authenticated Runtime IPC and fail-closed production operations are built.
   - Boot, diagnostics, network, power, and security state use observed values.
   - Full `selftest-secure-reset.sh` is red on a missing kinship readiness
     document. That is factory documentation debt, not an image-hash miss.

2. **Physical-metal acceptance — pending**
   - Reinstall representative hardware from the newly built image on disposable
     media. Do not run the disk installer on the open SSH lab.
   - Prove persistent Ethernet/Wi-Fi, live hardware values, stable fullscreen
     and window-mode behavior, and key-only maintenance access.
   - Keep the global verdict `NOT_READY` until that evidence is recorded.

3. **EliteEW product integration — scoped FoFline host only**
   - Historical P0–P92 documents are retained as records only.
   - Living work: V2 disk install (LUKS2 state slot; PIN ≠ volume key),
     installer-planted PIN, consume, OS-plane portal, SPA bootstrap pack,
     capsule-gated OTA contract (`launcher.ota_*`), native OIDC card,
     typed local `product.open`, always-on HUD, Seat Browser overlay,
     Seat Shield host workspace. Glass map:
     [`../session/SEAT-FACE.md`](../session/SEAT-FACE.md). Install:
     [`../session/OS-INSTALL.md`](../session/OS-INSTALL.md).
   - Settings must not ship the lab `/30` (`192.168.8.10` /
     `192.168.8.9`). Official SoT `empiieros-wifi@.service` still lacks
     `AF_INET`; apply plants the live unit after extract.
   - A/B updates stay later (P93 frozen). Bootstrap SPA pack fetch is
     OS-plane `launcher.spa_sync` after `allow_fofline` + live portal
     (pre-OIDC). Capsule-gated OS/FoFline OTA is `launcher.ota_*`
     (license + hardware + live EliteEW login + VIP) — P0 contract only;
     see [`../session/OTA-CAPSULE.md`](../session/OTA-CAPSULE.md).
     Typed local `product.open` uses `fofline-spa://localhost`. Remote
     SPA stays denied.
   - On 2026-09-22 factory `images/rootfs.tar.gz` and `out/rootfs.tar.gz`
     share `cf0b46e1…`. `5979e9cf…` is the 2026-09-20 ledger stamp, not
     the file on disk. Metal is not accepted. Nothing here is
     `verified-real`.

---

## Pointers

- Home map: [`WHERE-OS-LIVES.md`](WHERE-OS-LIVES.md)
- Glass map: [`../session/SEAT-FACE.md`](../session/SEAT-FACE.md)
- Shell v0 (text): [`OS-SHELL-V0-SOT.md`](OS-SHELL-V0-SOT.md)
- Kinship: [`KINSHIP.md`](KINSHIP.md)
- Status / gate: [`FOUNDATION-STATUS.md`](FOUNDATION-STATUS.md)
- Intent lock archive: [`capture/00-intent-and-understanding.md`](capture/00-intent-and-understanding.md)
